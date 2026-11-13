#!/usr/bin/env python3
"""Dump the PE export table of a Windows DLL — no external dependencies.

Used by the CI gate S7' to compare the export surface of the freshly built
core-ui.dll against the API contract declared by the upstream header.

Implementation notes:
  * PE32 (0x10b) and PE32+ (0x20b) are both handled — the data-directory
    offset differs between them.
  * Only the export directory is parsed; no dependency on the Windows SDK,
    so the script runs on any runner that has Python 3.

Usage:
    python pe_exports.py <path-to.dll> [--prefix ui_] [--out <file>]

With --prefix, only exports whose name starts with that prefix are printed.
Without it, every export name is printed.
"""

from __future__ import annotations

import struct
import sys


class PeError(RuntimeError):
    """Raised when the file is not a PE image we can parse."""


def _read_u16(buf: bytes, off: int) -> int:
    try:
        return struct.unpack_from("<H", buf, off)[0]
    except struct.error as exc:
        raise PeError(f"file is truncated: cannot read u16 at offset 0x{off:X}") from exc


def _read_u32(buf: bytes, off: int) -> int:
    try:
        return struct.unpack_from("<I", buf, off)[0]
    except struct.error as exc:
        raise PeError(f"file is truncated: cannot read u32 at offset 0x{off:X}") from exc


def _rva_to_offset(rva: int, sections: list[tuple[int, int, int, int]]) -> int:
    """Map a relative virtual address to a file offset.

    sections holds (virtual_address, virtual_size, raw_pointer, raw_size)
    tuples as described by the PE section headers.
    """
    for vaddr, vsize, raw_ptr, raw_size in sections:
        if vaddr <= rva < vaddr + max(vsize, raw_size):
            return raw_ptr + (rva - vaddr)
    raise PeError(f"RVA 0x{rva:X} does not fall inside any section")


def parse_exports(data: bytes) -> list[str]:
    """Return the sorted list of exported symbol names."""
    if len(data) < 0x40:
        raise PeError("file is too small to be a PE image")
    if data[:2] != b"MZ":
        raise PeError("missing MZ signature — not a PE image")

    pe_off = _read_u32(data, 0x3C)
    if data[pe_off : pe_off + 4] != b"PE\0\0":
        raise PeError("missing PE signature")

    coff = pe_off + 4
    number_of_sections = _read_u16(data, coff + 2)
    size_of_optional_header = _read_u16(data, coff + 16)

    opt = coff + 20
    magic = _read_u16(data, opt)
    if magic == 0x20B:
        data_dir_off = opt + 112  # PE32+
    elif magic == 0x10B:
        data_dir_off = opt + 96  # PE32
    else:
        raise PeError(f"unknown optional header magic 0x{magic:X}")

    # Data directory 0 is the export table.
    export_rva = _read_u32(data, data_dir_off)
    export_size = _read_u32(data, data_dir_off + 4)
    if export_rva == 0:
        return []

    # Section headers follow the optional header.
    sec_off = opt + size_of_optional_header
    sections: list[tuple[int, int, int, int]] = []
    for i in range(number_of_sections):
        base = sec_off + i * 40
        virtual_size = _read_u32(data, base + 8)
        virtual_address = _read_u32(data, base + 12)
        raw_size = _read_u32(data, base + 16)
        raw_pointer = _read_u32(data, base + 20)
        sections.append((virtual_address, virtual_size, raw_pointer, raw_size))

    exp_off = _rva_to_offset(export_rva, sections)

    number_of_names = _read_u32(data, exp_off + 24)
    address_of_names = _read_u32(data, exp_off + 32)
    address_of_name_ordinals = _read_u32(data, exp_off + 36)
    address_of_functions = _read_u32(data, exp_off + 28)

    names_off = _rva_to_offset(address_of_names, sections)
    ordinals_off = _rva_to_offset(address_of_name_ordinals, sections)
    funcs_off = _rva_to_offset(address_of_functions, sections)

    names: list[str] = []
    for i in range(number_of_names):
        name_rva = _read_u32(data, names_off + i * 4)
        name_file_off = _rva_to_offset(name_rva, sections)

        end = data.index(b"\0", name_file_off)
        name = data[name_file_off:end].decode("ascii", errors="replace")
        ordinal = _read_u16(data, ordinals_off + i * 2)
        func_rva = _read_u32(data, funcs_off + ordinal * 4)
        # A zero RVA means the symbol is a forwarder reference to another
        # module (declared in the forwarder string table) rather than a
        # local address. Forwarded exports are still part of the export
        # surface, so keep them.
        del func_rva
        names.append(name)

    del export_size
    return sorted(set(names))


def main(argv: list[str]) -> int:
    args = [a for a in argv[1:] if not a.startswith("--")]
    prefix: str | None = None
    out_path: str | None = None

    i = 1
    while i < len(argv):
        arg = argv[i]
        if arg == "--prefix" and i + 1 < len(argv):
            prefix = argv[i + 1]
            i += 2
        elif arg == "--out" and i + 1 < len(argv):
            out_path = argv[i + 1]
            i += 2
        else:
            i += 1

    if not args:
        print("usage: pe_exports.py <dll> [--prefix ui_] [--out <file>]", file=sys.stderr)
        return 2

    path = args[0]
    with open(path, "rb") as fh:
        data = fh.read()

    try:
        names = parse_exports(data)
    except PeError as exc:
        print(f"pe_exports: {exc}", file=sys.stderr)
        return 1

    if prefix is not None:
        names = [n for n in names if n.startswith(prefix)]

    text = "\n".join(names)
    if out_path:
        with open(out_path, "w", encoding="utf-8", newline="\n") as fh:
            fh.write(text + ("\n" if names else ""))
    else:
        print(text)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
