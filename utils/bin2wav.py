#!/usr/bin/env python3

import argparse
import struct
import sys
from pathlib import Path

KCC_HEADER_LEN = 128
KCC_NAME_LEN = 8
KCC_TYPE_LEN = 3
KCC_NUM_ADDR_OFF = 16
KCC_LOAD_ADDR_OFF = 17
KCC_END_ADDR_OFF = 19
KCC_EXEC_ADDR_OFF = 21

KCTAP_MAGIC = b"\xc3KC-TAPE by AF. "
KCTAP_BLOCK_LEN = 128

KCBASIC_MAGIC = b"\xd3\xd3\xd3"
KCBASIC_NAME_LEN = 8
KCBASIC_LEN_OFF = 11
KCBASIC_DATA_OFF = 13
KCBASIC_END_MARKER = 0x03

SSS_BLOCK_ALIGN = 0x80

FRAME_RATE = 22050
WAV_LOW = 0x00
WAV_HIGH = 0xC8
TAP_PAD = 0x00
WAV_PAD = 0xFF
HALF_BIT_ZERO = 5
HALF_BIT_ONE = 10
HALF_PRETONE = 10
HALF_SEPARATOR = 20
PRETONE_FIRST = 8000
PRETONE_NEXT = 320
PAUSE_BETWEEN = 240
TRAILING_PAUSE_MS = 500

BLK_NUM_MACHINE = 0
BLK_NUM_BASIC = 1

FORMATS = ("bin", "kcc", "tap", "sss", "wav")


class ConvertError(Exception):
    pass


def parse_address(text):
    try:
        return int(text, 0) & 0xFFFF
    except ValueError:
        raise ConvertError(f"invalid address: {text}") from None


def detect_format(path, blob):
    if blob[:16] == KCTAP_MAGIC:
        return "tap"
    if blob[:12] == b"RIFF" and blob[8:12] == b"WAVE":
        return "wav"
    if blob[:3] == KCBASIC_MAGIC:
        return "sss"

    suffix = Path(path).suffix.lower().lstrip(".")
    if suffix in FORMATS:
        return suffix

    if is_kcc_header(blob):
        return "kcc"
    return "bin"


def is_kcc_header(blob):
    if len(blob) < KCC_HEADER_LEN:
        return False
    if any(b >= 0x80 for b in blob[:KCC_NAME_LEN]):
        return False
    num_addr = blob[KCC_NUM_ADDR_OFF]
    if num_addr < 2 or num_addr > 3:
        return False
    load_addr = blob[KCC_LOAD_ADDR_OFF] | (blob[KCC_LOAD_ADDR_OFF + 1] << 8)
    end_addr = blob[KCC_END_ADDR_OFF] | (blob[KCC_END_ADDR_OFF + 1] << 8)
    return end_addr > load_addr


def trim_name(raw):
    name = []
    spaces = 0
    for b in raw:
        if b == 0x00:
            break
        if b == 0x20:
            spaces += 1
        elif b > 0x20:
            name.append(" " * spaces)
            spaces = 0
            name.append(chr(b))
    return "".join(name)


def make_kc_header(load_addr, end_addr, exec_addr, name, file_type):
    header = bytearray(KCC_HEADER_LEN)
    encoded_name = name.encode("latin-1")[:KCC_NAME_LEN]
    header[: len(encoded_name)] = encoded_name
    encoded_type = file_type.encode("latin-1")[:KCC_TYPE_LEN]
    header[KCC_NAME_LEN : KCC_NAME_LEN + len(encoded_type)] = encoded_type
    header[KCC_NUM_ADDR_OFF] = 2 if exec_addr is None else 3
    header[KCC_LOAD_ADDR_OFF : KCC_LOAD_ADDR_OFF + 2] = struct.pack("<H", load_addr)
    header[KCC_END_ADDR_OFF : KCC_END_ADDR_OFF + 2] = struct.pack(
        "<H", end_addr & 0xFFFF
    )
    if exec_addr is not None:
        header[KCC_EXEC_ADDR_OFF : KCC_EXEC_ADDR_OFF + 2] = struct.pack("<H", exec_addr)
    return header


def make_kcc_tape(data, load_addr, exec_addr, name, file_type):
    end_addr = (load_addr + len(data) - 1) & 0xFFFF
    header = make_kc_header(load_addr, end_addr, exec_addr, name, file_type)
    return bytes(header) + data


def split_kcc_tape(payload):
    if len(payload) < KCC_HEADER_LEN or not is_kcc_header(payload):
        raise ConvertError("not a valid KCC image")
    load_addr = payload[KCC_LOAD_ADDR_OFF] | (payload[KCC_LOAD_ADDR_OFF + 1] << 8)
    end_addr = payload[KCC_END_ADDR_OFF] | (payload[KCC_END_ADDR_OFF + 1] << 8)
    length = (end_addr - load_addr + 1) & 0xFFFF
    data = payload[KCC_HEADER_LEN : KCC_HEADER_LEN + length]
    return data, load_addr


def kcc_to_tape(blob, options):
    load_addr = blob[KCC_LOAD_ADDR_OFF] | (blob[KCC_LOAD_ADDR_OFF + 1] << 8)
    file_end = blob[KCC_END_ADDR_OFF] | (blob[KCC_END_ADDR_OFF + 1] << 8)
    length = (file_end - load_addr) & 0xFFFF
    num_addr = blob[KCC_NUM_ADDR_OFF]
    if num_addr >= 3:
        exec_addr = blob[KCC_EXEC_ADDR_OFF] | (blob[KCC_EXEC_ADDR_OFF + 1] << 8)
    else:
        exec_addr = None
    data = blob[KCC_HEADER_LEN : KCC_HEADER_LEN + length]
    name = options.name if options.name is not None else trim_name(blob[:KCC_NAME_LEN])
    if options.type is not None:
        file_type = options.type
    else:
        file_type = "COM" if exec_addr is not None else ""
    return make_kcc_tape(data, load_addr, exec_addr, name, file_type)


def make_kcbasic(image, name):
    header = bytearray(KCBASIC_MAGIC)
    encoded_name = name.encode("latin-1")[:KCBASIC_NAME_LEN]
    header += encoded_name
    header += b"\x20" * (KCBASIC_NAME_LEN - len(encoded_name))
    header += struct.pack("<H", len(image))
    return bytes(header) + image + bytes((KCBASIC_END_MARKER,))


def is_kcbasic(payload):
    return payload[:3] == KCBASIC_MAGIC


def split_kcbasic(payload):
    if not is_kcbasic(payload):
        raise ConvertError("not a KC-BASIC payload")
    length = payload[KCBASIC_LEN_OFF] | (payload[KCBASIC_LEN_OFF + 1] << 8)
    return payload[KCBASIC_DATA_OFF : KCBASIC_DATA_OFF + length]


def make_sss(image):
    body = struct.pack("<H", len(image)) + image + bytes((KCBASIC_END_MARKER,))
    padding = (-len(body)) % SSS_BLOCK_ALIGN
    return body + bytes(padding)


def split_sss(blob):
    length = blob[0] | (blob[1] << 8)
    return blob[2 : 2 + length]


def payload_to_blocks(payload, blk_num_start):
    blocks = []
    number = blk_num_start
    offset = 0
    total = len(payload)
    first = True
    while offset < total:
        remaining = total - offset
        if first or remaining > KCTAP_BLOCK_LEN:
            block_number = number & 0xFF
            number += 1
        else:
            block_number = 0xFF
        blocks.append((block_number, payload[offset : offset + KCTAP_BLOCK_LEN]))
        offset += KCTAP_BLOCK_LEN
        first = False
    return blocks


def blocks_to_payload(blocks):
    payload = bytearray()
    for _, data in blocks:
        payload += data
    return bytes(payload)


def pad_block(data, filler):
    if len(data) >= KCTAP_BLOCK_LEN:
        return data
    return data + bytes((filler,)) * (KCTAP_BLOCK_LEN - len(data))


def blocks_to_tap(blocks):
    out = bytearray(KCTAP_MAGIC)
    for number, data in blocks:
        out.append(number)
        out += pad_block(data, TAP_PAD)
    return bytes(out)


def tap_to_blocks(blob):
    if blob[:16] != KCTAP_MAGIC:
        raise ConvertError("not a KC-TAP file")
    body = blob[16:]
    stride = 1 + KCTAP_BLOCK_LEN
    if len(body) % stride != 0:
        raise ConvertError("truncated KC-TAP file")
    blocks = []
    for offset in range(0, len(body), stride):
        number = body[offset]
        data = body[offset + 1 : offset + stride]
        blocks.append((number, data))
    return blocks


def encode_wav(blocks):
    samples = bytearray()
    phase = [False]

    def add(count, level):
        samples.extend((WAV_HIGH if level else WAV_LOW,) * count)

    def add_phase(count):
        phase[0] = not phase[0]
        add(count, phase[0])

    def add_byte(value):
        for _ in range(8):
            if value & 0x01:
                add_phase(HALF_BIT_ONE)
                add_phase(HALF_BIT_ONE)
            else:
                add_phase(HALF_BIT_ZERO)
                add_phase(HALF_BIT_ZERO)
            value >>= 1
        add_phase(HALF_SEPARATOR)
        add_phase(HALF_SEPARATOR)

    for index, (number, data) in enumerate(blocks):
        if index == 0:
            pretone = PRETONE_FIRST
        else:
            add(PAUSE_BETWEEN, phase[0])
            pretone = PRETONE_NEXT
        for _ in range(pretone):
            add_phase(HALF_PRETONE)
        add_phase(HALF_SEPARATOR)
        add_phase(HALF_SEPARATOR)

        padded = pad_block(data, WAV_PAD)
        checksum = 0
        add_byte(number)
        for byte in padded:
            add_byte(byte)
            checksum = (checksum + byte) & 0xFF
        add_byte(checksum)

    if samples:
        add_phase(HALF_SEPARATOR)
    add(FRAME_RATE * TRAILING_PAUSE_MS // 1000, False)

    return wrap_wav(bytes(samples))


def wrap_wav(samples):
    header = b"RIFF"
    header += struct.pack("<I", 36 + len(samples))
    header += b"WAVEfmt "
    header += struct.pack("<IHHIIHH", 16, 1, 1, FRAME_RATE, FRAME_RATE, 1, 8)
    header += b"data"
    header += struct.pack("<I", len(samples))
    return header + samples


def decode_wav(blob):
    samples, offset = read_wav_samples(blob)
    reader = WaveReader(samples, offset)
    blocks = []
    while True:
        block = reader.read_block()
        if block is None:
            break
        blocks.append(block)
    if not blocks:
        raise ConvertError("no KC blocks found in WAV")
    return blocks


def read_wav_samples(blob):
    if blob[:4] != b"RIFF" or blob[8:12] != b"WAVE":
        raise ConvertError("not a WAV file")
    offset = 12
    channels = 1
    bits = 8
    while offset + 8 <= len(blob):
        chunk_id = blob[offset : offset + 4]
        chunk_len = struct.unpack_from("<I", blob, offset + 4)[0]
        body = offset + 8
        if chunk_id == b"fmt ":
            channels = struct.unpack_from("<H", blob, body + 2)[0]
            bits = struct.unpack_from("<H", blob, body + 14)[0]
        elif chunk_id == b"data":
            data = blob[body : body + chunk_len]
            return normalise_samples(data, channels, bits), 0
        offset = body + chunk_len + (chunk_len & 1)
    raise ConvertError("WAV data chunk not found")


def normalise_samples(data, channels, bits):
    step = (bits // 8) * channels
    if step == 0:
        raise ConvertError("unsupported WAV format")
    raw = []
    if bits == 8:
        for i in range(0, len(data) - step + 1, step):
            raw.append(data[i])
    elif bits == 16:
        for i in range(0, len(data) - step + 1, step):
            raw.append(struct.unpack_from("<h", data, i)[0])
    else:
        raise ConvertError(f"unsupported WAV bit depth: {bits}")
    if not raw:
        return raw
    midpoint = (min(raw) + max(raw)) // 2
    return [value - midpoint for value in raw]


class WaveReader:
    def __init__(self, samples, offset):
        self.samples = samples
        self.pos = offset

    def read_sample(self):
        if self.pos >= len(self.samples):
            return None
        value = self.samples[self.pos]
        self.pos += 1
        return value

    def read_half(self):
        first = self.read_sample()
        if first is None:
            return None
        length = 1
        rising = first >= 0
        while True:
            value = self.read_sample()
            if value is None:
                return None
            length += 1
            if rising and value < 0:
                break
            if not rising and value >= 0:
                break
        self.pos -= 1
        return length

    def read_full(self):
        first = self.read_half()
        if first is None:
            return None
        second = self.read_half()
        if second is None:
            return None
        return first + second

    def read_byte(self):
        value = 0
        for _ in range(8):
            full = self.read_full()
            if full is None:
                return None
            value >>= 1
            if full > 16:
                value |= 0x80
        self.read_full()
        return value

    def read_block(self):
        pretone = 0
        while True:
            full = self.read_full()
            if full is None:
                return None
            if 18 <= full <= 30:
                pretone += 1
            elif pretone >= 16:
                break
            else:
                pretone = 0

        number = self.read_byte()
        if number is None:
            return None
        data = bytearray()
        checksum = 0
        for _ in range(KCTAP_BLOCK_LEN):
            byte = self.read_byte()
            if byte is None:
                return None
            data.append(byte)
            checksum = (checksum + byte) & 0xFF
        stored = self.read_byte()
        if stored is None or stored != checksum:
            raise ConvertError(f"block {number:#04x} checksum mismatch")
        return number, bytes(data)


def load_blocks(fmt, blob, options):
    if fmt == "tap":
        return tap_to_blocks(blob)
    if fmt == "wav":
        return decode_wav(blob)
    if fmt == "kcc":
        if not is_kcc_header(blob):
            raise ConvertError("not a valid KCC image")
        return payload_to_blocks(kcc_to_tape(blob, options), BLK_NUM_MACHINE)
    if fmt == "bin":
        if options.addr is None:
            raise ConvertError("converting from .bin requires --addr")
        name = options.name if options.name is not None else ""
        if options.type is not None:
            file_type = options.type
        else:
            file_type = "COM" if options.entry is not None else ""
        payload = make_kcc_tape(blob, options.addr, options.entry, name, file_type)
        return payload_to_blocks(payload, BLK_NUM_MACHINE)
    if fmt == "sss":
        image = split_sss(blob)
        name = options.name if options.name is not None else ""
        payload = make_kcbasic(image, name)
        return payload_to_blocks(payload, BLK_NUM_BASIC)
    raise ConvertError(f"unsupported input format: {fmt}")


def write_blocks(fmt, blocks, options):
    if fmt == "tap":
        return blocks_to_tap(blocks)
    if fmt == "wav":
        return encode_wav(blocks)

    payload = blocks_to_payload(blocks)
    if fmt in ("kcc", "bin"):
        if not is_kcc_header(payload):
            raise ConvertError("source does not contain a KCC image")
        data, load_addr = split_kcc_tape(payload)
        if fmt == "bin":
            return data
        file_end = (load_addr + len(data)) & 0xFFFF
        header = bytearray(payload[:KCC_HEADER_LEN])
        header[KCC_END_ADDR_OFF : KCC_END_ADDR_OFF + 2] = struct.pack("<H", file_end)
        return bytes(header) + data
    if fmt == "sss":
        if not is_kcbasic(payload):
            raise ConvertError("source does not contain a KC-BASIC program")
        return make_sss(split_kcbasic(payload))
    raise ConvertError(f"unsupported output format: {fmt}")


def default_target(source):
    return "tap" if source == "wav" else "wav"


def convert(input_path, output_path, target, options):
    blob = Path(input_path).read_bytes()
    source = detect_format(input_path, blob)

    if target is None and output_path is not None:
        target = Path(output_path).suffix.lower().lstrip(".")
    if target is None:
        target = default_target(source)
    if target not in FORMATS:
        raise ConvertError(f"unknown target format: {target}")
    if target == source:
        raise ConvertError(f"source and target are both .{target}")

    blocks = load_blocks(source, blob, options)
    result = write_blocks(target, blocks, options)

    if output_path is None:
        output_path = str(Path(input_path).with_suffix("." + target))
    Path(output_path).write_bytes(result)
    return source, target, output_path


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("input", help="source file")
    parser.add_argument("output", nargs="?", help="destination file")
    parser.add_argument("-t", "--to", choices=FORMATS, help="target format")
    parser.add_argument("-a", "--addr", help="load address for .bin input")
    parser.add_argument("-e", "--entry", help="execution address")
    parser.add_argument("-n", "--name", help="program name stored in the header")
    parser.add_argument("--type", help="KCC file type")
    arguments = parser.parse_args()

    try:
        options = argparse.Namespace(
            addr=parse_address(arguments.addr) if arguments.addr else None,
            entry=parse_address(arguments.entry) if arguments.entry else None,
            name=arguments.name,
            type=arguments.type,
        )
        convert(arguments.input, arguments.output, arguments.to, options)
    except (ConvertError, OSError) as error:
        print(f"error: {error}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
