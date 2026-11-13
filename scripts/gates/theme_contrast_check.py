#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
主题 token 门禁：双主题对比度（G05）+ 间距刻度（G13）。

依据：
  - references/10-standards/mechanical-gates.md
      G05 双主题可读：文字色与背景色的对比度在 light/dark 下均 ≥ 4.5:1
      G13 视觉缺陷：间距取值必须落在刻度 {0,4,8,12,16,24,32,48} 上
  - docs/07-UI-DESIGN-WINUI3.md §4（颜色 token 表）
  - assets/themes.json（token 源）

用法：
    python3 scripts/gates/theme_contrast_check.py
    python3 scripts/gates/theme_contrast_check.py --root <项目目录>

退出码：0 = 全过；1 = 有 FAIL。
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

SPACE_SCALE = {0, 4, 8, 12, 16, 24, 32, 48}
MIN_CONTRAST = 4.5

# 文字色 → 允许出现的背景色（同一屏可能出现的组合）
FOREGROUND_BACKGROUND_PAIRS = [
    ("--app-text", ["--app-bg", "--app-content-bg", "--app-sidebar-bg",
                    "--app-toolbar-bg", "--app-selection-bg"]),
    ("--app-text-secondary", ["--app-bg", "--app-content-bg",
                               "--app-sidebar-bg", "--app-toolbar-bg"]),
    ("--app-info-fg", ["--app-info-bg"]),
    ("--app-warning-fg", ["--app-warning-bg"]),
    ("--app-danger-fg", ["--app-danger-bg"]),
    ("--app-accent", ["--app-bg", "--app-selection-bg"]),
]


def srgb_luminance(hex_color: str) -> float:
    h = hex_color.lstrip("#")
    if len(h) == 3:
        h = "".join(c * 2 for c in h)
    r, g, b = (int(h[i:i + 2], 16) / 255 for i in (0, 2, 4))
    f = lambda c: c / 12.92 if c <= 0.03928 else ((c + 0.055) / 1.055) ** 2.4
    r, g, b = f(r), f(g), f(b)
    return 0.2126 * r + 0.7152 * g + 0.0722 * b


def contrast_ratio(a: str, b: str) -> float:
    la, lb = srgb_luminance(a), srgb_luminance(b)
    hi, lo = max(la, lb), min(la, lb)
    return (hi + 0.05) / (lo + 0.05)


def check_contrast(theme: dict, mode: str) -> list[str]:
    fails: list[str] = []
    worst = (99.0, "", "")
    for fg, bgs in FOREGROUND_BACKGROUND_PAIRS:
        if fg not in theme:
            fails.append(f"[{mode}] 缺 token {fg}")
            continue
        for bg in bgs:
            if bg not in theme:
                fails.append(f"[{mode}] 缺 token {bg}")
                continue
            ratio = contrast_ratio(theme[fg], theme[bg])
            if ratio < worst[0]:
                worst = (ratio, f"{fg} on {bg}", f"{theme[fg]} / {theme[bg]}")
            if ratio < MIN_CONTRAST:
                fails.append(
                    f"[{mode}] {fg} ({theme[fg]}) on {bg} ({theme[bg]}) "
                    f"= {ratio:.2f}:1 < {MIN_CONTRAST}:1"
                )
    print(f"  [{mode}] 最低对比度 {worst[0]:.2f}:1  ({worst[1]} {worst[2]})")
    return fails


def check_spacing(root: Path) -> list[str]:
    """G13：padding / margin / gap 的取值必须落在刻度上。"""
    fails: list[str] = []
    pattern = re.compile(r"\b(padding|margin|gap)(-[a-z]+)?\s*:\s*([^;}]+)")
    for rel in ("assets/main.uix", "assets/preview.uix", "assets/about.uix"):
        path = root / rel
        if not path.exists():
            continue
        src = re.sub(r"<!--.*?-->", "", path.read_text(encoding="utf-8"), flags=re.S)
        src = re.sub(r"/\*.*?\*/", "", src, flags=re.S)
        for lineno, line in enumerate(src.splitlines(), 1):
            for m in pattern.finditer(line):
                for token in m.group(3).split():
                    if not token.endswith("px"):
                        continue
                    value = int(token[:-2])
                    if value not in SPACE_SCALE:
                        fails.append(
                            f"{rel}:{lineno}: 间距 {token} 不在刻度 "
                            f"{sorted(SPACE_SCALE)} 上 —— {m.group(0).strip()}"
                        )
    return fails


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", default=".", help="项目根目录（默认当前目录）")
    args = ap.parse_args()
    root = Path(args.root).resolve()

    theme_path = root / "assets" / "themes.json"
    if not theme_path.exists():
        print(f"找不到 {theme_path}")
        return 2
    data = json.loads(theme_path.read_text(encoding="utf-8"))

    print("=== G05 双主题对比度 ===")
    fails: list[str] = []
    for mode in ("light", "dark"):
        if mode not in data:
            fails.append(f"themes.json 缺 {mode} 段")
            continue
        fails += check_contrast(data[mode], mode)

    print("\n=== G13 间距刻度 ===")
    spacing_fails = check_spacing(root)
    fails += spacing_fails
    print(f"  非刻度间距：{len(spacing_fails)} 处")

    print()
    if fails:
        print(f"FAIL —— {len(fails)} 处：")
        for f in fails:
            print("  -", f)
        return 1
    print("PASS —— 双主题对比度全部 ≥ 4.5:1，间距取值全部在刻度上。")
    return 0


if __name__ == "__main__":
    sys.exit(main())
