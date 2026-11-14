#!/usr/bin/env python3
"""Gate S7' — the built core-ui.dll must expose exactly the API the source defines.

Why this gate exists
--------------------
`docs/05-COREUI-EXPORTS.txt` records the export surface measured on a
*pre-built* DLL (v1.9.0.346). The CI builds core-ui from a pinned source
revision, so comparing the freshly built DLL against that document would mix
two versions and report drift that is not a defect.

Instead the reference is the *same revision's* own source: every symbol
marked `UI_API` in `src/` is exported by the DLL. That makes the gate
version-consistent by construction — no cross-version comparison at all.

Why the source and not the public header
----------------------------------------
The upstream header is known to under-declare the export surface: twelve
`ui_image_gdi_*` / `ui_flush_gpu` symbols are defined `UI_API` in
`src/ui/ui_api.cpp` but never declared in `include/ui_core.h`. Using the
header as the reference would flag those twelve as "unexpected" even though
they are perfectly legitimate exports. The source is the authority on what
the library actually provides.

Known upstream gaps
-------------------
`ui_dialog` is declared in the header but has no `UI_API` definition in the
source, so the library never exports it. That is an upstream inconsistency,
not a build defect — it is tolerated and reported, so the gate stays honest
instead of silently widening its tolerance.

Rules
-----
  required  = UI_API definitions found in the pinned revision's source
  actual    = ui_* exports of the freshly built DLL

  * required - actual  must be empty.
  * actual - required  must be empty (an undescribed export cannot be
    relied upon by the binding layer).

Usage:
    python coreui_export_gate.py --dll <core-ui.dll> --source-dir <core-ui/src>
                                  [--include-dir <core-ui/include>]
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from pe_exports import PeError, parse_exports  # noqa: E402

# The upstream header declares `ui_dialog`, but the library never exports it
# (no UI_API definition in the source) — an upstream inconsistency, not a
# build defect. It is tolerated in BOTH directions:
#   * missing      → the DLL does not export it (today's reality)
#   * unexpected   → upstream later implemented it (the DLL exports it)
# Either way it must not fail the gate, otherwise the gate would break the
# day upstream fixes the inconsistency.
# ★ 不用 `frozenset[str]` 注解：模块级变量注解同样不受
# `from __future__ import annotations` 影响，在 Python 3.9 上会抛
# "TypeError: 'type' object is not subscriptable"。
KNOWN_NOT_IMPLEMENTED = frozenset({"ui_dialog"})

# A UI_API definition: `UI_API <return type> ui_name(args)`.
#
# The `UI_API` marker is NOT required to be at the start of a line: upstream
# uses `extern "C" { ... }` blocks (src/ui/asset.cpp:137, src/ui/page/page_api.cpp:508)
# in which the definitions are indented. Requiring line-start would make the
# gate's reference set depend on formatting.
_DEF_RE = re.compile(r"\bUI_API\s+[^;{()]*?\b(ui_[A-Za-z0-9_]+)\s*\(", re.MULTILINE)


def collect_definitions(roots: list[Path]) -> set[str]:
    """Gather every ui_* symbol marked UI_API in the given source trees."""
    found = set()  # type: set[str]
    for root in roots:
        if not root.is_dir():
            continue
        for path in sorted(root.rglob("*")):
            if path.suffix not in (".c", ".cpp", ".h", ".hpp", ".cc"):
                continue
            try:
                text = path.read_text(encoding="utf-8", errors="replace")
            except OSError:
                continue
            found.update(_DEF_RE.findall(text))
    return found


def main() -> int:
    parser = argparse.ArgumentParser(description="Gate S7' — core-ui.dll export surface")
    parser.add_argument("--dll", required=True, help="path to the built core-ui.dll")
    parser.add_argument(
        "--source-dir",
        required=True,
        action="append",
        help="a source tree to scan for UI_API definitions (repeatable)",
    )
    parser.add_argument(
        "--include-dir",
        action="append",
        help="an include tree to scan for UI_API definitions (repeatable)",
    )
    parser.add_argument(
        "--doc",
        default=None,
        help="optional docs/05-COREUI-EXPORTS.txt baseline, reported for context only",
    )
    args = parser.parse_args()

    dll_path = Path(args.dll)
    if not dll_path.is_file():
        print(f"S7' FAILED: core-ui.dll not found at {dll_path}", file=sys.stderr)
        return 1

    try:
        all_exports = set(parse_exports(dll_path.read_bytes()))
    except PeError as exc:
        print(f"S7' FAILED: cannot parse exports from {dll_path}: {exc}", file=sys.stderr)
        return 1

    exported = {name for name in all_exports if name.startswith("ui_")}

    roots = [Path(p) for p in (args.source_dir or []) + (args.include_dir or [])]
    required = collect_definitions(roots)

    if not required:
        print(
            "S7' FAILED: no UI_API definitions were found — the scan pattern is "
            "stale, so the gate would pass vacuously.",
            file=sys.stderr,
        )
        return 1

    missing = required - exported
    unexpected = exported - required

    # ui_dialog is tolerated in both directions (see KNOWN_NOT_IMPLEMENTED).
    tolerated_missing = missing & KNOWN_NOT_IMPLEMENTED
    hard_missing = missing - KNOWN_NOT_IMPLEMENTED
    tolerated_unexpected = unexpected & KNOWN_NOT_IMPLEMENTED
    hard_unexpected = unexpected - KNOWN_NOT_IMPLEMENTED

    print(f"S7' reference : {', '.join(str(r) for r in roots)}")
    print(f"S7' required  (UI_API definitions): {len(required)}")
    print(f"S7' exported  (ui_* in built DLL): {len(exported)}")
    print(f"S7' missing   (defined, not exported): {len(missing)}")
    print(f"S7' unexpected(exported, not defined): {len(unexpected)}")
    if tolerated_missing or tolerated_unexpected:
        print(
            "S7' tolerated upstream gaps: "
            + ", ".join(sorted(tolerated_missing | tolerated_unexpected))
        )

    if args.doc:
        doc_path = Path(args.doc)
        if doc_path.is_file():
            doc_symbols = {
                line.strip()
                for line in doc_path.read_text(encoding="utf-8", errors="replace").splitlines()
                if line.startswith("ui_")
            }
            print(f"S7' context: docs/05 baseline lists {len(doc_symbols)} ui_* symbols")

    if hard_missing:
        print("", file=sys.stderr)
        print(
            "S7' FAILED: the built DLL does not export symbols the source defines. "
            "The binding layer would fail at LoadLibrary time.",
            file=sys.stderr,
        )
        for name in sorted(hard_missing):
            print(f"  missing: {name}", file=sys.stderr)
        return 1

    if hard_unexpected:
        print("", file=sys.stderr)
        print(
            "S7' FAILED: the built DLL exports ui_* symbols that no source defines. "
            "An undescribed export cannot be relied upon.",
            file=sys.stderr,
        )
        for name in sorted(hard_unexpected):
            print(f"  unexpected: {name}", file=sys.stderr)
        return 1

    print("S7' passed: the export surface matches the source contract exactly.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
