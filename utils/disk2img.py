#!/usr/bin/env python3

import argparse
import gzip
from pathlib import Path

SIZE_CODE_BASE = 0x80
MAX_SIZE_CODE = 6
TWO_SIDES = 2
MAX_IMAGE_SIZE = 4 * 1024 * 1024

GZIP_MAGIC = b"\x1f\x8b"
COPYQM_MAGIC = b"CQ\x14"
IMAGEDISK_MAGIC = b"IMD "
TELEDISK_MAGIC_PLAIN = b"TD"
TELEDISK_MAGIC_PACKED = b"td"
CPC_HEADER_STD = b"MV - CPCEMU"
CPC_HEADER_EXT = b"EXTENDED CPC DSK"
CPC_TRACK_HEADER = b"Track-Info\r\n"

ANADISK_EXTENSIONS = ("dump", "anadisk", "adl")
RAW_EXTENSIONS = ("img", "raw", "dd", "image")

IMD_END_OF_COMMENT = 0x1A
IMD_HEAD_CYL_MAP = 0x80
IMD_HEAD_HEAD_MAP = 0x40
IMD_HEAD_NUMBER_MASK = 0x01
IMD_TRANSFER_RATE = 0x05
IMD_RECORD_NORMAL = 0x01
IMD_BANNER = b"IMD 1.18 disk2img\x1a"

TELEDISK_SUPPORTED_VERSION = 0x15
TELEDISK_HEADER_LEN = 12
TELEDISK_VERSION_OFFSET = 4
TELEDISK_STEPPING_OFFSET = 7
TELEDISK_REMARK_FLAG = 0x80
TELEDISK_SECTOR_CRC_ERROR = 0x02
TELEDISK_SECTOR_DELETED = 0x04
TELEDISK_SECTOR_BOGUS_HEADER = 0x40
TELEDISK_SECTOR_NO_DATA_MASK = 0x30
TELEDISK_SECTOR_PHANTOM = 0xFF
TELEDISK_ENCODING_RAW = 0

CPC_FILE_HEADER_LEN = 0x100
CPC_TRACK_HEADER_LEN = 0x100
CPC_CYL_COUNT_OFFSET = 0x30
CPC_SIDE_COUNT_OFFSET = 0x31
CPC_STD_TRACK_SIZE_OFFSET = 0x32
CPC_EXT_TRACK_SIZE_TABLE = 0x34
CPC_TRACK_CYL_OFFSET = 0x10
CPC_TRACK_SIDE_OFFSET = 0x11
CPC_TRACK_SIZE_CODE_OFFSET = 0x14
CPC_TRACK_SECTOR_COUNT_OFFSET = 0x15
CPC_TRACK_SECTOR_LIST_OFFSET = 0x18
CPC_SECTOR_INFO_LEN = 8
CPC_BIG_SECTOR_SIZE = 0x1800
CPC_STD_BANNER = b"MV - CPCEMU Disk-File\r\nDisk-Info\r\ndisk2img\x00"

COPYQM_HEADER_LEN = 133
COPYQM_SECTOR_SIZE_OFFSET = 3
COPYQM_SECTORS_PER_TRACK_OFFSET = 0x10
COPYQM_SIDES_OFFSET = 0x12
COPYQM_USED_CYLS_OFFSET = 0x5A
COPYQM_CYLS_OFFSET = 0x5B
COPYQM_COMMENT_LEN_OFFSET = 0x6F
COPYQM_SECTOR_OFFSET_OFFSET = 0x71
COPYQM_RUN_FLAG = 0x8000
COPYQM_RUN_MODULO = 0x10000
COPYQM_MAX_CHUNK = 0x7FFF

LZHUF_WINDOW_SIZE = 4096
LZHUF_WINDOW_MASK = LZHUF_WINDOW_SIZE - 1
LZHUF_LOOKAHEAD = 60
LZHUF_THRESHOLD = 2
LZHUF_SYMBOL_COUNT = 256 - LZHUF_THRESHOLD + LZHUF_LOOKAHEAD
LZHUF_TABLE_SIZE = LZHUF_SYMBOL_COUNT * 2 - 1
LZHUF_ROOT = LZHUF_TABLE_SIZE - 1
LZHUF_MAX_FREQ = 0x8000
LZHUF_FREQ_SENTINEL = 0xFFFF
LZHUF_LITERAL_LIMIT = 256
LZHUF_MATCH_SYMBOL_BASE = 255 - LZHUF_THRESHOLD
LZHUF_RING_FILL = 0x20
LZHUF_LEADING_BITS = 8
LZHUF_POSITION_LOW_BITS = 6
LZHUF_POSITION_HIGH_BITS = 6
LZHUF_POSITION_CODE_RUNS = ((3, 1), (4, 3), (5, 8), (6, 12), (7, 24), (8, 16))

NAMED_FORMATS = {
    "llc2-400k": (80, 1, 5, 1024, 1),
    "pcm-624k": (80, 2, 16, 256, 1),
    "mldos-702k": (80, 2, 9, 512, 3),
    "basdos-711k": (80, 2, 9, 512, 5),
    "a5105-720k": (80, 2, 9, 512, 1),
    "caos-780k": (80, 2, 5, 1024, 1),
    "scpx-780k": (80, 2, 5, 1024, 2),
    "microdos-780k": (80, 2, 5, 1024, 3),
    "z9001-800k": (80, 2, 5, 1024, 4),
    "msdos-1200k": (80, 2, 15, 512, 1),
    "msdos-1440k": (80, 2, 18, 512, 1),
    "mldos-1738k": (80, 2, 11, 1024, 3),
}

DEFAULT_FORMAT = "z9001-800k"
DEFAULT_CONTAINER_SUFFIX = ".dump"


class FormatError(Exception):
    pass


def size_code_by_size(sector_size):
    if sector_size <= SIZE_CODE_BASE:
        return 0
    value = 0
    size = SIZE_CODE_BASE
    while size < sector_size and value < MAX_SIZE_CODE:
        value += 1
        size <<= 1
    return value


def size_by_size_code(size_code):
    return SIZE_CODE_BASE << min(size_code, MAX_SIZE_CODE)


def interleave_order(sectors_per_track, interleave):
    if interleave <= 1 or interleave >= sectors_per_track or sectors_per_track <= 2:
        return list(range(sectors_per_track))
    order = [None] * sectors_per_track
    src = 0
    dst = 0
    while src < sectors_per_track:
        while order[dst] is not None:
            dst = (dst + 1) % sectors_per_track
        order[dst] = src
        src += 1
        dst = (dst + interleave) % sectors_per_track
    return order


def sector(cylinder, head, sector_num, size_code, data, deleted=False, error=False):
    return {
        "cylinder": cylinder,
        "head": head,
        "sector_num": sector_num,
        "size_code": size_code,
        "data": bytes(data),
        "deleted": deleted,
        "error": error,
    }


def sized_buffer(size_code, raw):
    length = size_by_size_code(size_code)
    data = bytearray(length)
    copy = min(len(raw), length)
    data[:copy] = raw[:copy]
    return bytes(data)


class Reader:
    def __init__(self, data):
        self.data = data
        self.pos = 0

    def remaining(self):
        return len(self.data) - self.pos

    def next(self):
        if self.pos >= len(self.data):
            return None
        value = self.data[self.pos]
        self.pos += 1
        return value

    def byte(self):
        value = self.next()
        if value is None:
            raise FormatError("unexpected end of image")
        return value

    def word_le(self):
        low = self.byte()
        high = self.byte()
        return (high << 8) | low

    def take(self, length):
        end = self.pos + length
        if end > len(self.data):
            raise FormatError("unexpected end of image")
        slice_ = self.data[self.pos : end]
        self.pos = end
        return slice_


class Builder:
    def __init__(self):
        self.tracks = {}
        self.max_cylinder = 0
        self.max_head = 0

    def add(self, phys_cyl, phys_head, sec):
        self.max_cylinder = max(self.max_cylinder, phys_cyl)
        self.max_head = max(self.max_head, phys_head)
        self.tracks.setdefault((phys_cyl, phys_head), []).append(sec)

    def finish(self):
        if not self.tracks:
            raise FormatError("image contains no sectors")
        cylinders = self.max_cylinder + 1
        sides = min(self.max_head + 1, TWO_SIDES)
        return Disk(self.tracks, cylinders, sides)


class Disk:
    def __init__(self, tracks, cylinders, sides):
        self.tracks = tracks
        self.cylinders = cylinders
        self.sides = sides

    def track(self, cylinder, head):
        return self.tracks.get((cylinder, head), [])

    def ordered_tracks(self):
        for cyl in range(self.cylinders):
            for head in range(self.sides):
                yield cyl, head, self.track(cyl, head)


def disk_from_raw(data, fmt):
    cylinders, sides, sectors_per_track, sector_size, interleave = fmt
    order = interleave_order(sectors_per_track, interleave)
    size_code = size_code_by_size(sector_size)
    tracks = {}
    for cyl in range(cylinders):
        for head in range(sides):
            sectors = []
            for logical in order:
                block = ((sides * cyl) + head) * sectors_per_track + logical
                start = block * sector_size
                sectors.append(
                    sector(
                        cyl,
                        head,
                        logical + 1,
                        size_code,
                        sized_buffer(size_code, data[start : start + sector_size]),
                    )
                )
            tracks[(cyl, head)] = sectors
    return Disk(tracks, cylinders, sides)


def raw_from_disk(disk):
    image = bytearray()
    for _cyl, _head, sectors in disk.ordered_tracks():
        for sec in sorted(sectors, key=lambda s: s["sector_num"]):
            image += sec["data"]
    return bytes(image)


def maybe_gunzip(data):
    if data[: len(GZIP_MAGIC)] == GZIP_MAGIC:
        try:
            return gzip.decompress(data)
        except OSError:
            return data
    return data


def detect_kind(data, suffix):
    if data[: len(COPYQM_MAGIC)] == COPYQM_MAGIC:
        return "copyqm"
    if data[: len(IMAGEDISK_MAGIC)] == IMAGEDISK_MAGIC:
        return "imagedisk"
    if (
        data[: len(CPC_HEADER_STD)] == CPC_HEADER_STD
        or data[: len(CPC_HEADER_EXT)] == CPC_HEADER_EXT
    ):
        return "cpcdisk"
    if data[:2] == TELEDISK_MAGIC_PLAIN or data[:2] == TELEDISK_MAGIC_PACKED:
        return "teledisk"
    if suffix in ANADISK_EXTENSIONS:
        return "anadisk"
    return "raw"


def parse_anadisk(data):
    reader = Reader(data)
    builder = Builder()
    while True:
        phys_cyl = reader.next()
        if phys_cyl is None:
            break
        phys_head = reader.byte()
        id_cyl = reader.byte()
        id_head = reader.byte()
        id_record = reader.byte()
        id_size_code = reader.byte()
        data_len = reader.word_le()
        if phys_head > 1 or id_head > 1 or id_record < 1 or id_size_code > 3:
            break
        payload = reader.take(data_len) if data_len > 0 else b""
        builder.add(
            phys_cyl,
            phys_head,
            sector(
                id_cyl,
                id_head,
                id_record,
                id_size_code,
                sized_buffer(id_size_code, payload),
            ),
        )
    return builder.finish()


def parse_imagedisk(data):
    reader = Reader(data)
    while reader.byte() != IMD_END_OF_COMMENT:
        pass
    builder = Builder()
    while True:
        if reader.next() is None:
            break
        cyl = reader.byte()
        head_flags = reader.byte()
        sector_count = reader.byte()
        size_code = reader.byte()
        if size_code > MAX_SIZE_CODE:
            raise FormatError(f"IMD sector size code {size_code}")
        sector_size = size_by_size_code(size_code)
        sector_nums = reader.take(sector_count)
        sector_cyls = (
            reader.take(sector_count) if head_flags & IMD_HEAD_CYL_MAP else None
        )
        sector_heads = (
            reader.take(sector_count) if head_flags & IMD_HEAD_HEAD_MAP else None
        )
        phys_head = head_flags & IMD_HEAD_NUMBER_MASK
        for index in range(sector_count):
            sector_type = reader.byte()
            if sector_type == 0:
                payload = bytes(sector_size)
            elif sector_type in (1, 3, 5, 7):
                payload = reader.take(sector_size)
            elif sector_type in (2, 4, 6, 8):
                payload = bytes([reader.byte()]) * sector_size
            else:
                raise FormatError(f"IMD sector record type {sector_type}")
            id_cyl = sector_cyls[index] if sector_cyls is not None else cyl
            id_head = sector_heads[index] if sector_heads is not None else phys_head
            builder.add(
                cyl,
                phys_head,
                sector(
                    id_cyl,
                    id_head,
                    sector_nums[index],
                    size_code,
                    payload,
                    deleted=sector_type in (3, 4, 7, 8),
                    error=5 <= sector_type <= 8,
                ),
            )
    return builder.finish()


def parse_cpcdisk(data):
    if len(data) < CPC_FILE_HEADER_LEN:
        raise FormatError("truncated CPC image")
    header = data[:CPC_FILE_HEADER_LEN]
    extended = header[: len(CPC_HEADER_EXT)] == CPC_HEADER_EXT
    cyls = header[CPC_CYL_COUNT_OFFSET]
    sides = header[CPC_SIDE_COUNT_OFFSET]
    std_track_size = header[CPC_STD_TRACK_SIZE_OFFSET] | (
        header[CPC_STD_TRACK_SIZE_OFFSET + 1] << 8
    )
    reader = Reader(data)
    reader.take(CPC_FILE_HEADER_LEN)
    builder = Builder()
    for track_index in range(cyls * sides):
        if reader.remaining() < CPC_TRACK_HEADER_LEN:
            break
        track_header = reader.take(CPC_TRACK_HEADER_LEN)
        if track_header[: len(CPC_TRACK_HEADER)] != CPC_TRACK_HEADER:
            break
        if extended:
            entry = CPC_EXT_TRACK_SIZE_TABLE + track_index
            if entry >= CPC_FILE_HEADER_LEN:
                break
            track_size = header[entry] << 8
        else:
            track_size = std_track_size
        cyl = track_header[CPC_TRACK_CYL_OFFSET]
        side = track_header[CPC_TRACK_SIDE_OFFSET]
        track_size_code = track_header[CPC_TRACK_SIZE_CODE_OFFSET]
        if track_size_code > MAX_SIZE_CODE:
            break
        sector_count = track_header[CPC_TRACK_SECTOR_COUNT_OFFSET]
        track_buf = (
            reader.take(track_size - CPC_TRACK_HEADER_LEN)
            if track_size > CPC_TRACK_HEADER_LEN
            else b""
        )
        info_pos = CPC_TRACK_SECTOR_LIST_OFFSET
        data_pos = 0
        for _ in range(sector_count):
            if info_pos + CPC_SECTOR_INFO_LEN > len(track_header):
                break
            id_cyl = track_header[info_pos]
            id_head = track_header[info_pos + 1]
            id_record = track_header[info_pos + 2]
            id_size_code = track_header[info_pos + 3]
            if extended:
                stored_len = track_header[info_pos + 6] | (
                    track_header[info_pos + 7] << 8
                )
            elif track_size_code == MAX_SIZE_CODE:
                stored_len = CPC_BIG_SECTOR_SIZE
            else:
                stored_len = size_by_size_code(track_size_code)
            info_pos += CPC_SECTOR_INFO_LEN
            available = max(len(track_buf) - data_pos, 0)
            copy = min(stored_len, available)
            payload = sized_buffer(id_size_code, track_buf[data_pos : data_pos + copy])
            data_pos += stored_len
            builder.add(
                cyl, side, sector(id_cyl, id_head, id_record, id_size_code, payload)
            )
    return builder.finish()


def parse_copyqm(data):
    if len(data) < COPYQM_HEADER_LEN:
        raise FormatError("truncated CopyQM image")
    header = data[:COPYQM_HEADER_LEN]

    def word(offset):
        return header[offset] | (header[offset + 1] << 8)

    sector_size = word(COPYQM_SECTOR_SIZE_OFFSET)
    sectors_per_track = word(COPYQM_SECTORS_PER_TRACK_OFFSET)
    sides = min(max(header[COPYQM_SIDES_OFFSET], 1), TWO_SIDES)
    used_cyls = header[COPYQM_USED_CYLS_OFFSET]
    total_cyls = max(header[COPYQM_CYLS_OFFSET], used_cyls)
    sector_offset = header[COPYQM_SECTOR_OFFSET_OFFSET]
    comment_len = word(COPYQM_COMMENT_LEN_OFFSET)
    if sector_size == 0 or sectors_per_track == 0 or total_cyls == 0:
        raise FormatError("CopyQM geometry")

    reader = Reader(data)
    reader.take(COPYQM_HEADER_LEN)
    reader.take(comment_len)

    disk_size = total_cyls * sides * sectors_per_track * sector_size
    if disk_size > MAX_IMAGE_SIZE:
        raise FormatError("CopyQM image size")
    disk_bytes = bytearray(disk_size)
    dst = 0
    while dst < disk_size:
        if reader.remaining() < 2:
            break
        length = reader.word_le()
        if length & COPYQM_RUN_FLAG:
            run = (COPYQM_RUN_MODULO - length) & 0xFFFF
            value = reader.byte()
            while dst < disk_size and run > 0:
                disk_bytes[dst] = value
                dst += 1
                run -= 1
        else:
            count = length
            while dst < disk_size and count > 0:
                if reader.remaining() == 0:
                    break
                disk_bytes[dst] = reader.byte()
                dst += 1
                count -= 1

    size_code = size_code_by_size(sector_size)
    builder = Builder()
    block = 0
    for cyl in range(total_cyls):
        for head in range(sides):
            for sector_idx in range(sectors_per_track):
                start = block * sector_size
                payload = sized_buffer(
                    size_code, disk_bytes[start : start + sector_size]
                )
                builder.add(
                    cyl,
                    head,
                    sector(
                        cyl, head, sector_idx + 1 + sector_offset, size_code, payload
                    ),
                )
                block += 1
    return builder.finish()


def parse_teledisk(data):
    if len(data) < TELEDISK_HEADER_LEN:
        raise FormatError("truncated TeleDisk image")
    signature = data[:2]
    compressed = signature == TELEDISK_MAGIC_PACKED
    if not compressed and signature != TELEDISK_MAGIC_PLAIN:
        raise FormatError("not a TeleDisk image")
    version = data[TELEDISK_VERSION_OFFSET]
    if version != TELEDISK_SUPPORTED_VERSION:
        raise FormatError(f"TeleDisk format version {version:#04X}")
    has_remark = data[TELEDISK_STEPPING_OFFSET] & TELEDISK_REMARK_FLAG != 0

    body = data[TELEDISK_HEADER_LEN:]
    if compressed:
        body = lzhuf_decompress(body, MAX_IMAGE_SIZE)
    reader = Reader(body)

    if has_remark:
        reader.word_le()
        comment_len = reader.word_le()
        reader.take(6)
        reader.take(comment_len)

    builder = Builder()
    while True:
        sector_count = reader.next()
        if sector_count is None or sector_count == TELEDISK_SECTOR_PHANTOM:
            break
        track = reader.next()
        if track is None:
            break
        head = reader.next()
        if head is None:
            break
        if reader.next() is None:
            break
        phys_head = head & 0x01
        for _ in range(sector_count):
            sec_track = reader.byte()
            sec_head = reader.byte()
            sec_num = reader.byte()
            sec_size_code = reader.byte()
            sec_ctrl = reader.byte()
            reader.byte()
            if sec_size_code > MAX_SIZE_CODE:
                raise FormatError(f"TeleDisk sector size code {sec_size_code}")
            payload = bytearray(size_by_size_code(sec_size_code))
            if sec_ctrl & TELEDISK_SECTOR_NO_DATA_MASK == 0:
                decode_teledisk_data(reader, payload)
            bogus_header = sec_ctrl & TELEDISK_SECTOR_BOGUS_HEADER != 0
            id_cyl = track if bogus_header else sec_track
            id_head = (head & 0x01) if bogus_header else sec_head
            builder.add(
                track,
                phys_head,
                sector(
                    id_cyl,
                    id_head,
                    sec_num,
                    sec_size_code,
                    payload,
                    deleted=sec_ctrl & TELEDISK_SECTOR_DELETED != 0,
                    error=sec_ctrl & TELEDISK_SECTOR_CRC_ERROR != 0,
                ),
            )
    return builder.finish()


def decode_teledisk_data(reader, data):
    length = reader.word_le()
    if length == 0:
        return
    encoding = reader.byte()
    length -= 1
    pos = 0

    def put(value):
        nonlocal pos
        if pos < len(data):
            data[pos] = value
            pos += 1

    if encoding == 0:
        while length > 0:
            put(reader.byte())
            length -= 1
    elif encoding == 1:
        if length >= 4:
            count = reader.word_le()
            b0 = reader.byte()
            b1 = reader.byte()
            length -= 4
            while count > 0:
                put(b0)
                put(b1)
                count -= 1
    elif encoding == 2:
        while length >= 2:
            kind = reader.byte()
            count = reader.byte()
            length -= 2
            if kind == 0:
                while length > 0 and count > 0:
                    put(reader.byte())
                    count -= 1
                    length -= 1
            elif kind == 1:
                if length >= 2:
                    b0 = reader.byte()
                    b1 = reader.byte()
                    length -= 2
                    while count > 0:
                        put(b0)
                        put(b1)
                        count -= 1
            else:
                raise FormatError(f"TeleDisk sub-encoding {kind}")
    else:
        raise FormatError(f"TeleDisk sector encoding {encoding:#04X}")
    while length > 0:
        reader.byte()
        length -= 1


def write_anadisk(disk):
    out = bytearray()
    for cyl, head, sectors in disk.ordered_tracks():
        for sec in sectors:
            out += bytes(
                (
                    cyl,
                    head,
                    sec["cylinder"],
                    sec["head"],
                    sec["sector_num"],
                    sec["size_code"],
                    len(sec["data"]) & 0xFF,
                    (len(sec["data"]) >> 8) & 0xFF,
                )
            )
            out += sec["data"]
    return bytes(out)


def write_imagedisk(disk):
    out = bytearray(IMD_BANNER)
    for cyl, head, sectors in disk.ordered_tracks():
        if not sectors:
            continue
        size_code = sectors[0]["size_code"]
        sector_size = size_by_size_code(size_code)
        out += bytes((IMD_TRANSFER_RATE, cyl, head, len(sectors), size_code))
        out += bytes(sec["sector_num"] for sec in sectors)
        for sec in sectors:
            out.append(IMD_RECORD_NORMAL)
            out += sized_buffer(size_code, sec["data"])[:sector_size]
    return bytes(out)


def write_cpcdisk(disk):
    header = bytearray(CPC_FILE_HEADER_LEN)
    header[: len(CPC_STD_BANNER)] = CPC_STD_BANNER
    header[CPC_CYL_COUNT_OFFSET] = disk.cylinders
    header[CPC_SIDE_COUNT_OFFSET] = disk.sides

    tracks = []
    track_size = 0
    for cyl, head, sectors in disk.ordered_tracks():
        track_header = bytearray(CPC_TRACK_HEADER_LEN)
        track_header[: len(CPC_TRACK_HEADER)] = CPC_TRACK_HEADER
        track_header[CPC_TRACK_CYL_OFFSET] = cyl
        track_header[CPC_TRACK_SIDE_OFFSET] = head
        size_code = sectors[0]["size_code"] if sectors else 0
        track_header[CPC_TRACK_SIZE_CODE_OFFSET] = size_code
        track_header[CPC_TRACK_SECTOR_COUNT_OFFSET] = len(sectors)
        info_pos = CPC_TRACK_SECTOR_LIST_OFFSET
        payload = bytearray()
        for sec in sectors:
            track_header[info_pos] = sec["cylinder"]
            track_header[info_pos + 1] = sec["head"]
            track_header[info_pos + 2] = sec["sector_num"]
            track_header[info_pos + 3] = sec["size_code"]
            track_header[info_pos + 6] = len(sec["data"]) & 0xFF
            track_header[info_pos + 7] = (len(sec["data"]) >> 8) & 0xFF
            info_pos += CPC_SECTOR_INFO_LEN
            payload += sec["data"]
        track_size = max(track_size, CPC_TRACK_HEADER_LEN + len(payload))
        tracks.append(bytes(track_header) + bytes(payload))

    header[CPC_STD_TRACK_SIZE_OFFSET] = track_size & 0xFF
    header[CPC_STD_TRACK_SIZE_OFFSET + 1] = (track_size >> 8) & 0xFF
    out = bytearray(header)
    for track in tracks:
        out += track
        out += bytes(track_size - len(track))
    return bytes(out)


def write_copyqm(disk, fmt):
    _cylinders, _sides, sectors_per_track, sector_size, _interleave = fmt
    image = raw_from_disk(disk)
    header = bytearray(COPYQM_HEADER_LEN)
    header[: len(COPYQM_MAGIC)] = COPYQM_MAGIC
    header[COPYQM_SECTOR_SIZE_OFFSET] = sector_size & 0xFF
    header[COPYQM_SECTOR_SIZE_OFFSET + 1] = (sector_size >> 8) & 0xFF
    header[COPYQM_SECTORS_PER_TRACK_OFFSET] = sectors_per_track & 0xFF
    header[COPYQM_SECTORS_PER_TRACK_OFFSET + 1] = (sectors_per_track >> 8) & 0xFF
    header[COPYQM_SIDES_OFFSET] = disk.sides
    header[COPYQM_USED_CYLS_OFFSET] = disk.cylinders
    header[COPYQM_CYLS_OFFSET] = disk.cylinders
    return bytes(header) + rle_encode_copyqm(image)


def rle_encode_copyqm(data):
    out = bytearray()
    pos = 0
    length = len(data)
    while pos < length:
        run = 1
        while (
            pos + run < length
            and data[pos + run] == data[pos]
            and run < COPYQM_MAX_CHUNK
        ):
            run += 1
        if run >= 2:
            encoded = (COPYQM_RUN_MODULO - run) & 0xFFFF
            out += bytes((encoded & 0xFF, (encoded >> 8) & 0xFF, data[pos]))
            pos += run
        else:
            start = pos
            while pos < length and (pos - start) < COPYQM_MAX_CHUNK:
                if pos + 1 < length and data[pos] == data[pos + 1]:
                    break
                pos += 1
            chunk = pos - start
            out += bytes((chunk & 0xFF, (chunk >> 8) & 0xFF))
            out += data[start:pos]
    return bytes(out)


def write_teledisk(disk):
    header = bytearray(TELEDISK_HEADER_LEN)
    header[: len(TELEDISK_MAGIC_PLAIN)] = TELEDISK_MAGIC_PLAIN
    header[TELEDISK_VERSION_OFFSET] = TELEDISK_SUPPORTED_VERSION
    out = bytearray(header)
    for cyl, head, sectors in disk.ordered_tracks():
        out += bytes((len(sectors), cyl, head, 0))
        for sec in sectors:
            out += bytes(
                (
                    sec["cylinder"],
                    sec["head"],
                    sec["sector_num"],
                    sec["size_code"],
                    0,
                    0,
                )
            )
            block_len = len(sec["data"]) + 1
            out += bytes(
                (block_len & 0xFF, (block_len >> 8) & 0xFF, TELEDISK_ENCODING_RAW)
            )
            out += sec["data"]
    out.append(TELEDISK_SECTOR_PHANTOM)
    return bytes(out)


class BitReader:
    def __init__(self, data):
        self.data = data
        self.byte_pos = 0
        self.bit_pos = 0

    def bit(self):
        if self.byte_pos >= len(self.data):
            return None
        value = (self.data[self.byte_pos] >> (7 - self.bit_pos)) & 1
        self.bit_pos += 1
        if self.bit_pos == 8:
            self.bit_pos = 0
            self.byte_pos += 1
        return value

    def byte(self):
        value = 0
        for _ in range(8):
            bit = self.bit()
            if bit is None:
                return None
            value = (value << 1) | bit
        return value


class AdaptiveHuffman:
    def __init__(self):
        self.freq = [0] * (LZHUF_TABLE_SIZE + 1)
        self.parent = [0] * (LZHUF_TABLE_SIZE + LZHUF_SYMBOL_COUNT)
        self.son = [0] * LZHUF_TABLE_SIZE
        for symbol in range(LZHUF_SYMBOL_COUNT):
            self.freq[symbol] = 1
            self.son[symbol] = symbol + LZHUF_TABLE_SIZE
            self.parent[symbol + LZHUF_TABLE_SIZE] = symbol
        leaf = 0
        node = LZHUF_SYMBOL_COUNT
        while node <= LZHUF_ROOT:
            self.freq[node] = self.freq[leaf] + self.freq[leaf + 1]
            self.son[node] = leaf
            self.parent[leaf] = node
            self.parent[leaf + 1] = node
            leaf += 2
            node += 1
        self.freq[LZHUF_TABLE_SIZE] = LZHUF_FREQ_SENTINEL
        self.parent[LZHUF_ROOT] = 0

    def reconstruct(self):
        packed = 0
        for node in range(LZHUF_TABLE_SIZE):
            if self.son[node] >= LZHUF_TABLE_SIZE:
                self.freq[packed] = (self.freq[node] + 1) >> 1
                self.son[packed] = self.son[node]
                packed += 1
        child = 0
        node = LZHUF_SYMBOL_COUNT
        while node < LZHUF_TABLE_SIZE:
            combined = self.freq[child] + self.freq[child + 1]
            insert = 0
            scan = node - 1
            while True:
                if self.freq[scan] <= combined:
                    insert = scan + 1
                    break
                if scan == 0:
                    break
                scan -= 1
            for slot in range(node - 1, insert - 1, -1):
                self.freq[slot + 1] = self.freq[slot]
                self.son[slot + 1] = self.son[slot]
            self.freq[insert] = combined
            self.son[insert] = child
            child += 2
            node += 1
        for node in range(LZHUF_TABLE_SIZE):
            son = self.son[node]
            self.parent[son] = node
            if son < LZHUF_TABLE_SIZE:
                self.parent[son + 1] = node

    def update(self, symbol):
        if self.freq[LZHUF_ROOT] == LZHUF_MAX_FREQ:
            self.reconstruct()
        node = self.parent[symbol + LZHUF_TABLE_SIZE]
        while True:
            self.freq[node] += 1
            raised = self.freq[node]
            nxt = node + 1
            if raised > self.freq[nxt]:
                nxt += 1
                while raised > self.freq[nxt]:
                    nxt += 1
                nxt -= 1
                self.freq[node] = self.freq[nxt]
                self.freq[nxt] = raised
                left = self.son[node]
                self.parent[left] = nxt
                if left < LZHUF_TABLE_SIZE:
                    self.parent[left + 1] = nxt
                swapped = self.son[nxt]
                self.son[nxt] = left
                self.parent[swapped] = node
                if swapped < LZHUF_TABLE_SIZE:
                    self.parent[swapped + 1] = node
                self.son[node] = swapped
                node = nxt
            node = self.parent[node]
            if node == 0:
                break

    def decode_symbol(self, reader):
        node = self.son[LZHUF_ROOT]
        while node < LZHUF_TABLE_SIZE:
            bit = reader.bit()
            if bit is None:
                return None
            node = self.son[node + bit]
        symbol = node - LZHUF_TABLE_SIZE
        self.update(symbol)
        return symbol


def build_position_table():
    value = [0] * 256
    length = [0] * 256
    index = 0
    symbol = 0
    prefix = 0
    for bit_length, symbol_count in LZHUF_POSITION_CODE_RUNS:
        for _ in range(symbol_count):
            prefix += 1 << (LZHUF_LEADING_BITS - bit_length)
            for _ in range(1 << (LZHUF_LEADING_BITS - bit_length)):
                value[index] = symbol
                length[index] = bit_length
                index += 1
            symbol += 1
    return value, length


def lzhuf_decompress(data, max_output):
    value, length = build_position_table()
    tree = AdaptiveHuffman()
    reader = BitReader(data)
    ring = bytearray([LZHUF_RING_FILL]) * LZHUF_WINDOW_SIZE
    cursor = LZHUF_WINDOW_SIZE - LZHUF_LOOKAHEAD
    output = bytearray()
    while len(output) < max_output:
        symbol = tree.decode_symbol(reader)
        if symbol is None:
            break
        if symbol < LZHUF_LITERAL_LIMIT:
            output.append(symbol)
            ring[cursor] = symbol
            cursor = (cursor + 1) & LZHUF_WINDOW_MASK
        else:
            leading = reader.byte()
            if leading is None:
                break
            high = value[leading] << LZHUF_POSITION_LOW_BITS
            extra = length[leading] - (LZHUF_LEADING_BITS - LZHUF_POSITION_HIGH_BITS)
            low = leading
            for _ in range(extra):
                bit = reader.bit()
                if bit is None:
                    return bytes(output)
                low = (low << 1) + bit
            distance = high | (low & ((1 << LZHUF_POSITION_LOW_BITS) - 1))
            source = (cursor + LZHUF_WINDOW_SIZE - distance - 1) & LZHUF_WINDOW_MASK
            run = symbol - LZHUF_MATCH_SYMBOL_BASE
            for _ in range(run):
                byte = ring[source]
                output.append(byte)
                ring[cursor] = byte
                cursor = (cursor + 1) & LZHUF_WINDOW_MASK
                source = (source + 1) & LZHUF_WINDOW_MASK
                if len(output) >= max_output:
                    break
    return bytes(output)


DECODERS = {
    "anadisk": parse_anadisk,
    "imagedisk": parse_imagedisk,
    "cpcdisk": parse_cpcdisk,
    "copyqm": parse_copyqm,
    "teledisk": parse_teledisk,
}

CONTAINER_BY_SUFFIX = {
    "dump": ("anadisk", write_anadisk),
    "anadisk": ("anadisk", write_anadisk),
    "adl": ("anadisk", write_anadisk),
    "imd": ("imagedisk", write_imagedisk),
    "dsk": ("cpcdisk", write_cpcdisk),
    "cqm": ("copyqm", write_copyqm),
    "td0": ("teledisk", write_teledisk),
}

WRITERS_WITH_FORMAT = {"copyqm"}


def suffix_of(path):
    suffixes = Path(path).suffixes
    if suffixes and suffixes[-1].lower() in (".gz", ".gzip"):
        suffixes = suffixes[:-1]
    return suffixes[-1].lstrip(".").lower() if suffixes else ""


def decode_to_raw(input_path, output_path):
    data = maybe_gunzip(Path(input_path).read_bytes())
    kind = detect_kind(data, suffix_of(input_path))
    if kind == "raw":
        raise FormatError("input is already a raw image; use --encode to pack it")
    disk = DECODERS[kind](data)
    Path(output_path).write_bytes(raw_from_disk(disk))


def encode_to_container(input_path, output_path, format_name):
    fmt = NAMED_FORMATS[format_name]
    data = maybe_gunzip(Path(input_path).read_bytes())
    disk = disk_from_raw(data, fmt)
    kind, writer = CONTAINER_BY_SUFFIX[suffix_of(output_path)]
    container = writer(disk, fmt) if kind in WRITERS_WITH_FORMAT else writer(disk)
    Path(output_path).write_bytes(container)


def default_output(input_path, suffix):
    return str(Path(input_path).with_suffix(suffix))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("input", help="source file")
    parser.add_argument("output", nargs="?", help="destination file")
    parser.add_argument(
        "-d", "--decode", action="store_true", help="unpack a container to a raw image"
    )
    parser.add_argument(
        "-e", "--encode", action="store_true", help="pack a raw image into a container"
    )
    parser.add_argument(
        "-f",
        "--format",
        choices=sorted(NAMED_FORMATS),
        default=DEFAULT_FORMAT,
        help="raw image geometry used when packing",
    )
    arguments = parser.parse_args()

    data = maybe_gunzip(Path(arguments.input).read_bytes())
    detected = detect_kind(data, suffix_of(arguments.input))
    decoding = arguments.decode or (not arguments.encode and detected != "raw")

    if decoding:
        decode_to_raw(
            arguments.input, arguments.output or default_output(arguments.input, ".img")
        )
    else:
        output = arguments.output or default_output(
            arguments.input, DEFAULT_CONTAINER_SUFFIX
        )
        if suffix_of(output) not in CONTAINER_BY_SUFFIX:
            parser.error(f"unknown container extension for '{output}'")
        encode_to_container(arguments.input, output, arguments.format)


if __name__ == "__main__":
    main()
