#!/usr/bin/env python3

import argparse
import json
from pathlib import Path

RECORD_SIZE = 128
RECORDS_PER_EXTENT = 128
EXTENT_SIZE = RECORDS_PER_EXTENT * RECORD_SIZE
DIR_ENTRY_LEN = 32
DELETED_MARK = 0xE5
FILL_BYTE = 0xE5
MAX_USER = 15

NAME_OFFSET = 1
NAME_LEN = 8
EXT_OFFSET = 9
EXT_LEN = 3
EXTENT_LOW_OFFSET = 12
EXTENT_LOW_MASK = 0x1F
RESERVED_OFFSET = 13
EXTENT_HIGH_OFFSET = 14
EXTENT_HIGH_SHIFT = 5
EXTENT_HIGH_MASK = 0x07E0
RECORD_COUNT_OFFSET = 15
BLOCK_LIST_OFFSET = 16
BLOCK_LIST_LEN = 16

ATTR_READ_ONLY = 0
ATTR_SYSTEM = 1
ATTR_ARCHIVE = 2
ATTR_BIT = 0x80
ATTR_FLAGS = (("R", ATTR_READ_ONLY), ("S", ATTR_SYSTEM), ("A", ATTR_ARCHIVE))

MANIFEST_NAME = "manifest.json"
USER_DIR_PREFIX = "user"

CPM_FORMATS = {
    "z9001-800k": (819200, 2048, 3, True, 0),
    "msdos-1200k": (1228800, 4096, 2, True, 0),
    "msdos-1440k": (1474560, 4096, 2, True, 0),
}

DEFAULT_FORMAT = "z9001-800k"


class FilesystemError(Exception):
    pass


def block_pointer_count(block_num_16bit):
    return BLOCK_LIST_LEN // 2 if block_num_16bit else BLOCK_LIST_LEN


def blocks_per_extent(block_size, block_num_16bit):
    return min(EXTENT_SIZE // block_size, block_pointer_count(block_num_16bit))


def records_per_block(block_size):
    return block_size // RECORD_SIZE


def format_for_size(size):
    for name, params in CPM_FORMATS.items():
        if params[0] == size:
            return name
    return None


def decode_attributes(entry):
    return tuple(bool(entry[EXT_OFFSET + index] & ATTR_BIT) for _, index in ATTR_FLAGS)


def attributes_to_text(attributes):
    return "".join(letter for (letter, _), flag in zip(ATTR_FLAGS, attributes) if flag)


def attributes_from_text(text):
    flags = [False, False, False]
    if text:
        for letter, index in ATTR_FLAGS:
            if letter in text:
                flags[index] = True
    return tuple(flags)


def clean_name(raw):
    return bytes(byte & 0x7F for byte in raw).decode("latin-1").rstrip()


def entry_filename(entry):
    name = clean_name(entry[NAME_OFFSET : NAME_OFFSET + NAME_LEN])
    ext = clean_name(entry[EXT_OFFSET : EXT_OFFSET + EXT_LEN])
    return f"{name}.{ext}" if ext else name


def entry_extent_number(entry):
    return (entry[EXTENT_LOW_OFFSET] & EXTENT_LOW_MASK) | (
        (entry[EXTENT_HIGH_OFFSET] << EXTENT_HIGH_SHIFT) & EXTENT_HIGH_MASK
    )


def entry_block_pointers(entry, block_num_16bit):
    pointers = []
    if block_num_16bit:
        for index in range(BLOCK_LIST_OFFSET, BLOCK_LIST_OFFSET + BLOCK_LIST_LEN, 2):
            pointers.append(entry[index] | (entry[index + 1] << 8))
    else:
        pointers = list(entry[BLOCK_LIST_OFFSET : BLOCK_LIST_OFFSET + BLOCK_LIST_LEN])
    return pointers


def data_area(image, params):
    _size, block_size, _dir_blocks, _block_num_16bit, sys_tracks = params
    return image[sys_tracks * block_size :] if sys_tracks else image


def read_block(data, block_size, block):
    start = block * block_size
    return data[start : start + block_size]


def parse_directory(image, params):
    _size, block_size, dir_blocks, block_num_16bit, _sys_tracks = params
    data = data_area(image, params)
    directory = data[: dir_blocks * block_size]
    files = {}
    order = []
    for pos in range(0, len(directory), DIR_ENTRY_LEN):
        entry = directory[pos : pos + DIR_ENTRY_LEN]
        user = entry[0]
        if user > MAX_USER:
            continue
        filename = entry_filename(entry)
        key = (user, filename)
        if key not in files:
            files[key] = []
            order.append(key)
        files[key].append(
            (
                entry_extent_number(entry),
                entry[RECORD_COUNT_OFFSET],
                [
                    block
                    for block in entry_block_pointers(entry, block_num_16bit)
                    if block
                ],
                decode_attributes(entry),
            )
        )
    return order, files


def assemble_file(extents, data, block_size):
    extents.sort(key=lambda extent: extent[0])
    content = bytearray()
    for _extent_number, record_count, blocks, _attributes in extents:
        extent_bytes = bytearray()
        for block in blocks:
            extent_bytes += read_block(data, block_size, block)
        content += extent_bytes[: record_count * RECORD_SIZE]
    return bytes(content)


def unpack(image, format_name, params, out_dir):
    data = data_area(image, params)
    block_size = params[1]
    order, files = parse_directory(image, params)
    out_dir.mkdir(parents=True, exist_ok=True)
    manifest = {"format": format_name, "files": []}
    for key in order:
        user, filename = key
        extents = files[key]
        content = assemble_file(extents, data, block_size)
        attributes = extents[0][3]
        target_dir = out_dir if user == 0 else out_dir / f"{USER_DIR_PREFIX}{user:02d}"
        target_dir.mkdir(parents=True, exist_ok=True)
        (target_dir / filename).write_bytes(content)
        manifest["files"].append(
            {"name": filename, "user": user, "flags": attributes_to_text(attributes)}
        )
    (out_dir / MANIFEST_NAME).write_text(
        json.dumps(manifest, indent=2) + "\n", encoding="latin-1"
    )


def split_filename(filename):
    stem, _, ext = filename.partition(".")
    return stem[:NAME_LEN].upper(), ext[:EXT_LEN].upper()


def build_entry(
    user, filename, attributes, extent_number, record_count, blocks, block_num_16bit
):
    entry = bytearray([0x00] * DIR_ENTRY_LEN)
    entry[0] = user
    name, ext = split_filename(filename)
    entry[NAME_OFFSET : NAME_OFFSET + NAME_LEN] = name.ljust(NAME_LEN).encode("latin-1")
    ext_bytes = bytearray(ext.ljust(EXT_LEN).encode("latin-1"))
    for (_letter, index), flag in zip(ATTR_FLAGS, attributes):
        if flag:
            ext_bytes[index] |= ATTR_BIT
    entry[EXT_OFFSET : EXT_OFFSET + EXT_LEN] = ext_bytes
    entry[EXTENT_LOW_OFFSET] = extent_number & EXTENT_LOW_MASK
    entry[RESERVED_OFFSET] = 0
    entry[EXTENT_HIGH_OFFSET] = (extent_number >> EXTENT_HIGH_SHIFT) & 0x3F
    entry[RECORD_COUNT_OFFSET] = record_count
    if block_num_16bit:
        for slot, block in enumerate(blocks):
            entry[BLOCK_LIST_OFFSET + slot * 2] = block & 0xFF
            entry[BLOCK_LIST_OFFSET + slot * 2 + 1] = (block >> 8) & 0xFF
    else:
        for slot, block in enumerate(blocks):
            entry[BLOCK_LIST_OFFSET + slot] = block & 0xFF
    return bytes(entry)


def file_entries(
    user, filename, attributes, records, blocks, block_size, block_num_16bit
):
    entries = []
    pointers_per_extent = block_pointer_count(block_num_16bit)
    blocks_each = blocks_per_extent(block_size, block_num_16bit)
    extent_number = 0
    consumed = 0
    remaining = records
    while True:
        extent_records = min(remaining, RECORDS_PER_EXTENT)
        extent_blocks = blocks[consumed : consumed + blocks_each][:pointers_per_extent]
        entries.append(
            build_entry(
                user,
                filename,
                attributes,
                extent_number,
                extent_records,
                extent_blocks,
                block_num_16bit,
            )
        )
        consumed += len(extent_blocks)
        remaining -= extent_records
        extent_number += 1
        if remaining <= 0:
            break
    return entries


def read_manifest(in_dir):
    path = in_dir / MANIFEST_NAME
    if not path.exists():
        return None
    data = json.loads(path.read_text(encoding="latin-1"))
    listing = [
        (
            entry.get("user", 0),
            entry["name"],
            attributes_from_text(entry.get("flags", "")),
        )
        for entry in data.get("files", [])
    ]
    return data.get("format"), listing


def discover_files(in_dir):
    listing = []
    for entry in sorted(in_dir.iterdir(), key=lambda item: item.name.lower()):
        if entry.is_file() and entry.name != MANIFEST_NAME:
            listing.append((0, entry.name, (False, False, False)))
    for sub in sorted(in_dir.iterdir(), key=lambda item: item.name.lower()):
        if sub.is_dir() and sub.name.startswith(USER_DIR_PREFIX):
            user = int(sub.name[len(USER_DIR_PREFIX) :])
            for entry in sorted(sub.iterdir(), key=lambda item: item.name.lower()):
                if entry.is_file():
                    listing.append((user, entry.name, (False, False, False)))
    return listing


def resolve_path(in_dir, user, filename):
    if user == 0:
        flat = in_dir / filename
        if flat.exists():
            return flat
        return in_dir / f"{USER_DIR_PREFIX}{user:02d}" / filename
    return in_dir / f"{USER_DIR_PREFIX}{user:02d}" / filename


def repack(in_dir, params, listing):
    size, block_size, dir_blocks, block_num_16bit, sys_tracks = params
    discovered = discover_files(in_dir)
    if listing is None:
        listing = discovered
    else:
        known = {(user, filename) for user, filename, _ in listing}
        listing += [item for item in discovered if (item[0], item[1]) not in known]

    total_blocks = size // block_size
    max_dir_entries = dir_blocks * block_size // DIR_ENTRY_LEN
    image = bytearray([FILL_BYTE] * size)
    data_offset = sys_tracks * block_size
    directory = bytearray()
    free_block = dir_blocks

    for user, filename, attributes in listing:
        content = resolve_path(in_dir, user, filename).read_bytes()
        records = (len(content) + RECORD_SIZE - 1) // RECORD_SIZE
        block_count = (
            records + records_per_block(block_size) - 1
        ) // records_per_block(block_size)
        if free_block + block_count > total_blocks:
            raise FilesystemError("image is full")
        blocks = list(range(free_block, free_block + block_count))
        start = data_offset + free_block * block_size
        image[start : start + len(content)] = content
        slack_end = start + block_count * block_size
        image[start + len(content) : slack_end] = bytes(
            slack_end - start - len(content)
        )
        free_block += block_count
        directory += b"".join(
            file_entries(
                user, filename, attributes, records, blocks, block_size, block_num_16bit
            )
        )

    if len(directory) > dir_blocks * block_size:
        raise FilesystemError(
            f"directory needs {len(directory) // DIR_ENTRY_LEN} entries, only {max_dir_entries} fit"
        )
    image[data_offset : data_offset + len(directory)] = directory
    return bytes(image)


def select_format(name, image_size):
    if name is not None:
        return name, CPM_FORMATS[name]
    detected = format_for_size(image_size) if image_size is not None else None
    chosen = detected or DEFAULT_FORMAT
    return chosen, CPM_FORMATS[chosen]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("input", help="raw image or directory")
    parser.add_argument("output", nargs="?", help="destination directory or image")
    parser.add_argument(
        "-d", "--decode", action="store_true", help="unpack an image into a directory"
    )
    parser.add_argument(
        "-e", "--encode", action="store_true", help="repack a directory into an image"
    )
    parser.add_argument(
        "-f",
        "--format",
        choices=sorted(CPM_FORMATS),
        default=None,
        help="CP/M disk format",
    )
    arguments = parser.parse_args()

    source = Path(arguments.input)
    encoding = arguments.encode or (not arguments.decode and source.is_dir())

    if encoding:
        manifest = read_manifest(source)
        name = arguments.format or (manifest[0] if manifest else None) or DEFAULT_FORMAT
        if name not in CPM_FORMATS:
            parser.error(f"unknown disk format '{name}'")
        listing = manifest[1] if manifest else None
        output = (
            Path(arguments.output) if arguments.output else source.with_suffix(".img")
        )
        output.write_bytes(repack(source, CPM_FORMATS[name], listing))
    else:
        image = source.read_bytes()
        name, params = select_format(arguments.format, len(image))
        output = (
            Path(arguments.output) if arguments.output else source.with_suffix(".dir")
        )
        unpack(image, name, params, output)


if __name__ == "__main__":
    main()
