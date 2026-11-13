#!/usr/bin/env python3
"""Gate — the Rust-side ui_widget_find_by_id id manifest must match the .uix files.

Why this gate exists
--------------------
The Rust binding layer resolves widgets by id (`ui_widget_find_by_id`).
Every id it looks up must exist in the corresponding `.uix`, or the lookup
returns an empty handle and the panel silently stays blank — no error, no
warning, nothing in the log.

Hardcoding the id count in the docs is not a check: the count drifts the
moment someone adds an id. So the expected set lives in one machine-readable
file (`assets/ui-ids.json`) and this gate compares it against what the `.uix`
files actually declare.

Checks
------
1. Every id in the manifest exists in its page's `.uix`.
2. Every id declared in a `.uix` is listed in the manifest
   (catches an id added to the page but not to the manifest).
3. The manifest's per-page counts and the total match the actual id lists
   (catches a hand-edited manifest that lists the right ids but a wrong count).
4. No duplicate ids within a page (a duplicate makes the second lookup
   shadow the first).

Usage:
    python ui_id_manifest_check.py --root <project directory>
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

_ID_RE = re.compile(r"""\bid\s*=\s*["']([^"']+)["']""")

# HTML 注释：<!-- ... -->。.uix 里大量注释在说明设计意图，其中也会写出
# id 名（例：'→ <custom id="preview_image">'）。注释里的 id 不是元素，
# 必须排除，否则会把「说明文字」误判成「重复 id」。
_COMMENT_RE = re.compile(r"<!--.*?-->", re.DOTALL)


def extract_ids(path: Path) -> list[str]:
    """Return the declared ids in document order (duplicates preserved).

    Comments are stripped first — an id mentioned inside a comment is
    documentation, not an element.
    """
    text = path.read_text(encoding="utf-8", errors="replace")
    text = _COMMENT_RE.sub("", text)
    return _ID_RE.findall(text)


def main() -> int:
    parser = argparse.ArgumentParser(description="check the ui id manifest against the .uix files")
    parser.add_argument("--root", required=True, help="project directory")
    args = parser.parse_args()

    root = Path(args.root)
    assets = root / "assets"
    manifest_path = assets / "ui-ids.json"

    if not manifest_path.is_file():
        print(f"FAIL: manifest not found at {manifest_path}", file=sys.stderr)
        return 1

    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        print(f"FAIL: {manifest_path} is not valid JSON: {exc}", file=sys.stderr)
        return 1

    pages = manifest.get("pages")
    if not isinstance(pages, dict) or not pages:
        print(f"FAIL: {manifest_path} has no 'pages' object", file=sys.stderr)
        return 1

    problems: list[str] = []
    total = 0

    for page_name, spec in pages.items():
        file_name = spec.get("file", "")
        uix_path = assets / file_name
        if not uix_path.is_file():
            problems.append(f"{page_name}: 清单指向的 {file_name} 不存在")
            continue

        actual = extract_ids(uix_path)
        actual_set = set(actual)
        expected = list(spec.get("ids", []))

        # 重复 id：第二个会遮蔽第一个。
        seen: set[str] = set()
        for i in actual:
            if i in seen:
                problems.append(f"{page_name}: {file_name} 里 id 重复：{i}")
            seen.add(i)

        # 清单里有、页面里没有 → Rust 取不到 → 面板空白且无报错。
        for i in expected:
            if i not in actual_set:
                problems.append(f"{page_name}: 清单列了 {i}，但 {file_name} 里没有这个 id")

        # 页面里有、清单里没有 → 清单漏了，Rust 侧可能没接上。
        for i in actual:
            if i not in set(expected):
                problems.append(f"{page_name}: {file_name} 里有 id {i}，但清单没列")

        # 计数自洽。
        declared_count = spec.get("count")
        if declared_count != len(expected):
            problems.append(
                f"{page_name}: 清单声明 count={declared_count}，但 ids 实际有 {len(expected)} 个"
            )

        total += len(actual_set)

    declared_total = manifest.get("total")
    if declared_total != total:
        problems.append(f"清单声明 total={declared_total}，但三页实际共有 {total} 个唯一 id")

    if problems:
        print("FAIL: ui id 清单与 .uix 不一致：", file=sys.stderr)
        for p in problems:
            print(f"  - {p}", file=sys.stderr)
        return 1

    print(f"OK: ui id 清单与 .uix 一致（共 {total} 个唯一 id，{len(pages)} 页）")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
