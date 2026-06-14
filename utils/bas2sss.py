#!/usr/bin/env python3

import argparse
from pathlib import Path

PROGRAM_START = 0x0401
END_MARKER    = 0x03
BLOCK_ALIGN   = 0x80
FIRST_TOKEN   = 0x80
TEXT_ENCODING = "latin-1"

KEYWORDS = [
    "END",
    "FOR",
    "NEXT",
    "DATA",
    "INPUT",
    "DIM",
    "READ",
    "LET",
    "GOTO",
    "RUN",
    "IF",
    "RESTORE",
    "GOSUB",
    "RETURN",
    "REM",
    "STOP",
    "OUT",
    "ON",
    "NULL",
    "WAIT",
    "DEF",
    "POKE",
    "DOKE",
    "AUTO",
    "LINES",
    "CLS",
    "WIDTH",
    "BYE",
    "!",
    "CALL",
    "PRINT",
    "CONT",
    "LIST",
    "CLEAR",
    "CLOAD",
    "CSAVE",
    "NEW",
    "TAB(",
    "TO",
    "FN",
    "SPC(",
    "THEN",
    "NOT",
    "STEP",
    "+",
    "-",
    "*",
    "/",
    "^",
    "AND",
    "OR",
    ">",
    "=",
    "<",
    "SGN",
    "INT",
    "ABS",
    "USR",
    "FRE",
    "INP",
    "POS",
    "SQR",
    "RND",
    "LN",
    "EXP",
    "COS",
    "SIN",
    "TAN",
    "ATN",
    "PEEK",
    "DEEK",
    "PI",
    "LEN",
    "STR$",
    "VAL",
    "ASC",
    "CHR$",
    "LEFT$",
    "RIGHT$",
    "MID$",
    "LOAD",
    "TRON",
    "TROFF",
    "EDIT",
    "ELSE",
    "INKEY$",
    "JOYST",
    "STRING$",
    "INSTR",
    "RENUMBER",
    "DELETE",
    "PAUSE",
    "BEEP",
    "WINDOW",
    "BORDER",
    "INK",
    "PAPER",
    "AT",
    "PSET",
    "LINE",
    "CIRCLE",
    "!",
    "PAINT",
    "LABEL",
    "SIZE",
    "ZERO",
    "HOME",
    "!",
    "GCLS",
    "SCALE",
    "SCREEN",
    "POINT",
    "XPOS",
    "!",
    "YPOS",
]

REMARK_TOKENS = {FIRST_TOKEN + KEYWORDS.index("REM"), FIRST_TOKEN + KEYWORDS.index("!")}
DATA_TOKEN = FIRST_TOKEN + KEYWORDS.index("DATA")

BYTE_TO_KEYWORD = {FIRST_TOKEN + i: kw for i, kw in enumerate(KEYWORDS)}

KEYWORD_TO_BYTE = {}
for index, keyword in enumerate(KEYWORDS):
    KEYWORD_TO_BYTE.setdefault(keyword, FIRST_TOKEN + index)

MATCH_ORDER = sorted(KEYWORD_TO_BYTE.keys(), key=len, reverse=True)


def escape_byte(b):
    if b == 0x5C:
        return "\\\\"
    elif b < 0x20 or b >= 0x7F:
        return f"\\x{b:02X}"
    else:
        return chr(b)


def decode_escapes(s):
    res = bytearray()
    i = 0
    while i < len(s):
        if s[i] == "\\":
            if i + 1 < len(s) and s[i + 1] == "\\":
                res.append(0x5C)
                i += 2
            elif i + 3 < len(s) and s[i + 1] == "x":
                try:
                    res.append(int(s[i + 2 : i + 4], 16))
                    i += 4
                except ValueError:
                    res.append(0x5C)
                    i += 1
            else:
                res.append(0x5C)
                i += 1
        else:
            res.append(ord(s[i]) & 0xFF)
            i += 1
    return bytes(res)


def encode_content(rem_bytes):
    result = bytearray()
    mode = "NORMAL"
    i = 0

    while i < len(rem_bytes):
        c = rem_bytes[i]

        if mode == "REMARK":
            result.append(c)
            i += 1

        elif mode == "DATA":
            if c == ord(":"):
                mode = "NORMAL"
                result.append(c)
                i += 1
            elif c == ord('"'):
                result.append(c)
                i += 1
                while i < len(rem_bytes) and rem_bytes[i] != ord('"'):
                    result.append(rem_bytes[i])
                    i += 1
                if i < len(rem_bytes):
                    result.append(rem_bytes[i])
                    i += 1
            else:
                result.append(c)
                i += 1

        else:
            if c == ord('"'):
                result.append(c)
                i += 1
                while i < len(rem_bytes) and rem_bytes[i] != ord('"'):
                    result.append(rem_bytes[i])
                    i += 1
                if i < len(rem_bytes):
                    result.append(rem_bytes[i])
                    i += 1
            else:
                kw = None
                content_upper = bytes(rem_bytes[i:]).upper()
                for keyword in MATCH_ORDER:
                    kw_bytes = keyword.encode("ascii")
                    if content_upper.startswith(kw_bytes):
                        kw = keyword
                        break

                if kw is not None:
                    token = KEYWORD_TO_BYTE[kw]
                    result.append(token)
                    i += len(kw)
                    if token in REMARK_TOKENS:
                        mode = "REMARK"
                    elif token == DATA_TOKEN:
                        mode = "DATA"
                else:
                    result.append(c)
                    i += 1

    return bytes(result)


def parse_basic_source(source):
    lines = {}
    for raw in source.splitlines():
        if not raw.strip():
            continue

        cursor = 0
        while cursor < len(raw) and raw[cursor] == " ":
            cursor += 1

        digits = cursor
        while digits < len(raw) and raw[digits].isdigit():
            digits += 1
        if digits == cursor:
            continue

        number = int(raw[cursor:digits])
        remainder = raw[digits:]

        if remainder.startswith(" "):
            remainder = remainder[1:]

        rem_bytes = decode_escapes(remainder)
        content = encode_content(rem_bytes)
        lines[number] = content

    return [(number, lines[number]) for number in sorted(lines)]


def encode_program(lines):
    address = PROGRAM_START
    placed = []
    for number, content in lines:
        block_length = 4 + len(content) + 1
        placed.append((address + block_length, number, content))
        address += block_length

    image = bytearray()
    for next_address, number, content in placed:
        image += bytes(
            (next_address & 0xFF, next_address >> 8, number & 0xFF, number >> 8)
        )
        image += content
        image.append(0x00)
    image += b"\x00\x00"
    return bytes(image)


def wrap_sss(image):
    body = bytes((len(image) & 0xFF, len(image) >> 8)) + image + bytes((END_MARKER,))
    padding = (-len(body)) % BLOCK_ALIGN
    return body + bytes(padding)


def read_image(blob):
    declared = blob[0] | (blob[1] << 8)
    if len(blob) >= 13 and blob[0] == blob[1] == blob[2] and 0xD3 <= blob[0] <= 0xD8:
        declared = blob[11] | (blob[12] << 8)
        return blob[13 : 13 + declared]
    return blob[2 : 2 + declared]


def detokenize_program(image):
    def at(address):
        index = address - PROGRAM_START
        return image[index] if 0 <= index < len(image) else 0

    def word(address):
        return at(address) | (at(address + 1) << 8)

    output = []
    address = PROGRAM_START
    next_address = word(address)

    while next_address >= address + 5 and at(next_address - 1) == 0:
        address += 2
        line_num = word(address)
        address += 2

        line_str = str(line_num) + " "

        while address < next_address:
            b = at(address)
            address += 1
            if b == 0:
                break

            if b == ord('"'):
                line_str += '"'
                while address < next_address:
                    b2 = at(address)
                    address += 1
                    if b2 == 0:
                        break
                    line_str += escape_byte(b2)
                    if b2 == ord('"'):
                        break
                continue

            if b >= FIRST_TOKEN:
                keyword = BYTE_TO_KEYWORD.get(b)
                if keyword:
                    line_str += keyword
                else:
                    line_str += escape_byte(b)
            else:
                line_str += escape_byte(b)

        output.append(line_str)
        address = next_address
        next_address = word(address)

    return "\n".join(output) + ("\n" if output else "")


def convert_bas_to_sss(input_path, output_path):
    source = Path(input_path).read_text(encoding=TEXT_ENCODING)
    sss = wrap_sss(encode_program(parse_basic_source(source)))
    Path(output_path).write_bytes(sss)


def convert_sss_to_bas(input_path, output_path):
    blob = Path(input_path).read_bytes()
    text = detokenize_program(read_image(blob))
    Path(output_path).write_text(text, encoding=TEXT_ENCODING)


def default_output(input_path, target_suffix):
    return str(Path(input_path).with_suffix(target_suffix))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("input", help="source file")
    parser.add_argument(
        "output", nargs="?", help="destination file"
    )
    parser.add_argument(
        "-d", "--decode", action="store_true", help="convert .sss to .bas"
    )
    parser.add_argument(
        "-e", "--encode", action="store_true", help="convert .bas to .sss"
    )
    arguments = parser.parse_args()

    decoding = arguments.decode or (
        not arguments.encode and arguments.input.lower().endswith(".sss")
    )
    if decoding:
        convert_sss_to_bas(
            arguments.input, arguments.output or default_output(arguments.input, ".bas")
        )
    else:
        convert_bas_to_sss(
            arguments.input, arguments.output or default_output(arguments.input, ".sss")
        )


if __name__ == "__main__":
    main()
