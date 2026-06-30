#!/usr/bin/env python3

import argparse
import json
import os
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

DS_FILENAME = "!!!TIME&.DAT"
DS_RECORD_LEN = 16
DS_SIGNATURE = b"!!!TIME\x92"
DS_CHECKSUM_SPAN = 0x7F
DS_SIGNATURE_OFFSET = 0x0F
DS_STAMP_LEN = 5
DS_STAMP_COUNT = 3
DS_YEAR_MIN = 1978
DS_YEAR_MAX = 2078
SECONDS_PER_DAY = 86400
SECONDS_PER_HOUR = 3600
SECONDS_PER_MINUTE = 60

CPM_FORMATS = {
    "z9001-800k": (819200, 2048, 3, True, 0, False),
    "msdos-1200k": (1228800, 4096, 2, True, 0, False),
    "msdos-1440k": (1474560, 4096, 2, True, 0, False),
    "mldos-1738k": (1802240, 4096, 2, True, 22528, True),
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


def to_bcd(value):
    return (((value // 10) % 10) << 4) | (value % 10)


def from_bcd(value):
    high = value >> 4
    low = value & 0x0F
    if high > 9 or low > 9:
        return None
    return high * 10 + low


def civil_from_unix(secs):
    days = secs // SECONDS_PER_DAY
    rem = secs % SECONDS_PER_DAY
    hour = rem // SECONDS_PER_HOUR
    minute = (rem % SECONDS_PER_HOUR) // SECONDS_PER_MINUTE
    z = days + 719468
    era = (z if z >= 0 else z - 146096) // 146097
    doe = z - era * 146097
    yoe = (doe - doe // 1460 + doe // 36524 - doe // 146096) // 365
    doy = doe - (365 * yoe + yoe // 4 - yoe // 100)
    mp = (5 * doy + 2) // 153
    day = doy - (153 * mp + 2) // 5 + 1
    month = mp + 3 if mp < 10 else mp - 9
    year = yoe + era * 400 + (1 if month <= 2 else 0)
    return year, month, day, hour, minute


def days_from_civil(year, month, day):
    y = year - 1 if month <= 2 else year
    era = (y if y >= 0 else y - 399) // 400
    yoe = y - era * 400
    mp = month - 3 if month > 2 else month + 9
    doy = (153 * mp + 2) // 5 + day - 1
    doe = yoe * 365 + yoe // 4 - yoe // 100 + doy
    return era * 146097 + doe - 719468


def write_stamp(buf, offset, secs):
    if secs is None:
        return
    year, month, day, hour, minute = civil_from_unix(secs)
    if DS_YEAR_MIN <= year < DS_YEAR_MAX and offset + DS_STAMP_LEN <= len(buf):
        buf[offset] = to_bcd(year % 100)
        buf[offset + 1] = to_bcd(month)
        buf[offset + 2] = to_bcd(day)
        buf[offset + 3] = to_bcd(hour)
        buf[offset + 4] = to_bcd(minute)


def stamp_to_unix(buf, offset):
    if offset + DS_STAMP_LEN > len(buf):
        return None
    fields = [from_bcd(buf[offset + index]) for index in range(DS_STAMP_LEN)]
    if any(field is None for field in fields):
        return None
    yy, month, day, hour, minute = fields
    if month == 0 or month > 12 or day == 0 or day > 31:
        return None
    year = 1900 + yy if yy >= DS_YEAR_MIN % 100 else 2000 + yy
    return (
        days_from_civil(year, month, day) * SECONDS_PER_DAY
        + hour * SECONDS_PER_HOUR
        + minute * SECONDS_PER_MINUTE
    )


def build_datestamp(dir_entries, slot_paths):
    buf = bytearray(dir_entries * DS_RECORD_LEN)
    signature = 0
    pos = DS_SIGNATURE_OFFSET
    while pos < len(buf):
        buf[pos] = DS_SIGNATURE[signature % len(DS_SIGNATURE)]
        signature += 1
        pos += DS_RECORD_LEN
    for slot, path in slot_paths:
        base = slot * DS_RECORD_LEN
        if base + DS_STAMP_COUNT * DS_STAMP_LEN > len(buf):
            continue
        try:
            info = path.stat()
        except OSError:
            continue
        write_stamp(buf, base, int(getattr(info, "st_ctime", info.st_mtime)))
        write_stamp(buf, base + DS_STAMP_LEN, int(info.st_atime))
        write_stamp(buf, base + 2 * DS_STAMP_LEN, int(info.st_mtime))
    pos = 0
    while pos < len(buf):
        checksum = 0
        count = 0
        while count < DS_CHECKSUM_SPAN and pos < len(buf):
            checksum = (checksum + buf[pos]) & 0xFFFFFFFF
            pos += 1
            count += 1
        if pos < len(buf):
            buf[pos] = checksum & 0xFF
            pos += 1
    return bytes(buf)


def data_area(image, params):
    sys_bytes = params[4]
    return image[sys_bytes:] if sys_bytes else image


def read_block(data, block_size, block):
    start = block * block_size
    return data[start : start + block_size]


def parse_directory(image, params):
    _size, block_size, dir_blocks, block_num_16bit, _sys_bytes, _datestamp = params
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


def apply_datestamp(image, params, out_dir):
    block_size = params[1]
    dir_blocks = params[2]
    data = data_area(image, params)
    _order, files = parse_directory(image, params)
    ds_key = next((key for key in files if key[1].upper() == DS_FILENAME), None)
    if ds_key is None:
        return
    stamps = assemble_file(files[ds_key], data, block_size)
    directory = data[: dir_blocks * block_size]
    slot = 0
    for pos in range(0, len(directory), DIR_ENTRY_LEN):
        entry = directory[pos : pos + DIR_ENTRY_LEN]
        if len(entry) < DIR_ENTRY_LEN:
            break
        user = entry[0]
        if user <= MAX_USER and entry_extent_number(entry) == 0:
            filename = entry_filename(entry)
            if filename.upper() != DS_FILENAME:
                secs = stamp_to_unix(stamps, slot * DS_RECORD_LEN + 2 * DS_STAMP_LEN)
                base = (
                    out_dir if user == 0 else out_dir / f"{USER_DIR_PREFIX}{user:02d}"
                )
                path = base / filename
                if secs is not None and path.exists():
                    os.utime(path, (secs, secs))
        slot += 1


def unpack(image, format_name, params, out_dir):
    data = data_area(image, params)
    block_size = params[1]
    datestamp = params[5]
    order, files = parse_directory(image, params)
    out_dir.mkdir(parents=True, exist_ok=True)
    manifest = {"format": format_name, "files": []}
    for key in order:
        user, filename = key
        if datestamp and filename.upper() == DS_FILENAME:
            continue
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
    if datestamp:
        apply_datestamp(image, params, out_dir)


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
    size, block_size, dir_blocks, block_num_16bit, sys_bytes, datestamp = params
    discovered = discover_files(in_dir)
    if listing is None:
        listing = discovered
    else:
        known = {(user, filename) for user, filename, _ in listing}
        listing += [item for item in discovered if (item[0], item[1]) not in known]
    if datestamp:
        listing = [item for item in listing if item[1].upper() != DS_FILENAME]

    total_blocks = size // block_size
    max_dir_entries = dir_blocks * block_size // DIR_ENTRY_LEN
    image = bytearray([FILL_BYTE] * size)
    data_offset = sys_bytes
    directory = bytearray()
    free_block = dir_blocks
    slot_paths = []
    ds_blocks = []

    if datestamp:
        ds_records = (max_dir_entries * DS_RECORD_LEN + RECORD_SIZE - 1) // RECORD_SIZE
        ds_block_count = (
            ds_records + records_per_block(block_size) - 1
        ) // records_per_block(block_size)
        if free_block + ds_block_count > total_blocks:
            raise FilesystemError("image is full")
        ds_blocks = list(range(free_block, free_block + ds_block_count))
        free_block += ds_block_count
        directory += b"".join(
            file_entries(
                0,
                DS_FILENAME,
                (False, False, False),
                ds_records,
                ds_blocks,
                block_size,
                block_num_16bit,
            )
        )

    for user, filename, attributes in listing:
        path = resolve_path(in_dir, user, filename)
        content = path.read_bytes()
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
        slot_paths.append((len(directory) // DIR_ENTRY_LEN, path))
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

    if datestamp and ds_blocks:
        stamps = build_datestamp(max_dir_entries, slot_paths)
        ds_start = data_offset + ds_blocks[0] * block_size
        image[ds_start : ds_start + len(stamps)] = stamps

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
