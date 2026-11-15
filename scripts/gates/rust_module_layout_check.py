#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Rust 模块布局门禁 —— 静态拦截 `error[E0583]: file not found for module`。

背景：本机无 cargo / rustc，`mod X;` 与磁盘布局不一致不会被静态发现，
只会在 CI 的 `cargo test` 里以 E0583 爆出来。第 11 次 CI 实测：
`zsdesk-coreui` 的 5 个模块全部找不到文件，Test 步骤直接失败，
后续 Gate S13 / cargo build / package 全部被连带跳过。
「没有执行器的规则只是建议」—— 所以把 Rust 的模块文件解析规则机械复现，
并在 CI 的 `cargo test` **之前**执行。

Rust 2018+ 规则（简化到本项目实际用到的形态）：
  * `mod X;` —— 若紧邻 `#[path = "P"]`，文件 = dirname(F)/P；
    否则 F 为 mod-rs（`lib.rs` / `main.rs` / `mod.rs`）时找
    `dirname(F)/X.rs` 或 `dirname(F)/X/mod.rs`；
    F 为 non-mod-rs 时找 `dirname(F)/stem(F)/X.rs` 或
    `dirname(F)/stem(F)/X/mod.rs`。
  * `mod X { ... }` 是内联模块，不需要文件。
  * 找到的模块文件递归检查。

只做**文件存在性**判定，不解析 Rust 类型（那是 rustc 的职责，本机没有）。

用法：
    python3 scripts/gates/rust_module_layout_check.py
    python3 scripts/gates/rust_module_layout_check.py --root <项目目录>

退出码：0 = 全过；1 = 有 FAIL；2 = 用法错误。
"""
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

# `pub mod` / `pub(crate) mod` / `mod`，末尾 ; 表示外部模块，{ 表示内联模块。
_MOD_DECL = re.compile(
    r"^\s*(?:pub(?:\s*\([^)]*\))?\s+)?mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*([;{])"
)
_PATH_ATTR = re.compile(r'^\s*#\[path\s*=\s*"([^"]+)"\s*\]')
_MOD_RS_NAMES = frozenset({"lib.rs", "main.rs", "mod.rs"})


def strip_comments(src: str) -> str:
    """把行注释 / 块注释替换成空格，保留换行与字符串内容，行号不丢失。"""
    out = []
    in_block = False
    i = 0
    n = len(src)
    while i < n:
        c = src[i]
        nxt = src[i + 1] if i + 1 < n else ""
        if in_block:
            if c == "*" and nxt == "/":
                out.append("  ")
                i += 2
                in_block = False
                continue
            out.append("\n" if c == "\n" else " ")
            i += 1
            continue
        if c == "/" and nxt == "*":
            out.append("  ")
            i += 2
            in_block = True
            continue
        if c == "/" and nxt == "/":
            j = src.find("\n", i)
            if j == -1:
                j = n
            out.append(" " * (j - i))
            i = j
            continue
        if c == '"':
            j = i + 1
            while j < n:
                if src[j] == "\\":
                    j += 2
                    continue
                if src[j] == '"':
                    j += 1
                    break
                j += 1
            out.append(src[i:j])
            i = j
            continue
        out.append(c)
        i += 1
    return "".join(out)


def rel(root: Path, path: Path) -> str:
    try:
        return str(path.resolve().relative_to(root))
    except ValueError:
        return str(path)


def check_file(path: Path, root: Path, fails: list, seen: set) -> None:
    real = path.resolve()
    if real in seen:
        return
    seen.add(real)

    try:
        src = path.read_text(encoding="utf-8")
    except (OSError, UnicodeError) as exc:
        fails.append(f"{rel(root, path)}: 读取失败（{exc}）")
        return

    clean = strip_comments(src)
    is_mod_rs = path.name in _MOD_RS_NAMES
    pending_path = None  # type: str | None

    for lineno, line in enumerate(clean.splitlines(), 1):
        m = _PATH_ATTR.match(line)
        if m:
            pending_path = m.group(1)
            continue

        d = _MOD_DECL.match(line)
        if d:
            name, tail = d.group(1), d.group(2)
            if tail == "{":
                pending_path = None
                continue  # 内联模块，不需要文件

            if pending_path:
                candidates = [path.parent / pending_path]
            elif is_mod_rs:
                candidates = [path.parent / f"{name}.rs", path.parent / name / "mod.rs"]
            else:
                base = path.parent / path.stem
                candidates = [base / f"{name}.rs", base / name / "mod.rs"]

            found = next((c for c in candidates if c.is_file()), None)
            if found is None:
                tried = ", ".join(rel(root, c) for c in candidates)
                fails.append(
                    f"{rel(root, path)}:{lineno}: mod {name} 找不到模块文件"
                    f"（已试 {tried}）—— rustc 会报 E0583"
                )
            else:
                check_file(found, root, fails, seen)
            pending_path = None
            continue

        # 允许 #[cfg(...)] 等属性夹在 #[path] 与 mod 之间；其余非空行视为打断。
        stripped = line.strip()
        if stripped and not stripped.startswith("#[") and not stripped.startswith("#!["):
            pending_path = None


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", default=".", help="项目根目录（默认当前目录）")
    args = ap.parse_args()
    root = Path(args.root).resolve()

    crate_srcs = sorted((root / "crates").glob("*/src"))
    if not crate_srcs:
        print(f"找不到 {root / 'crates'} 下的 crate 源码目录")
        return 2

    roots = []
    for s in crate_srcs:
        for name in ("lib.rs", "main.rs"):
            p = s / name
            if p.is_file():
                roots.append(p)
    if not roots:
        print("FAIL —— 找不到任何 crate root（lib.rs / main.rs），门禁拒绝空过")
        return 1

    fails = []
    seen = set()
    for r in roots:
        check_file(r, root, fails, seen)

    print(f"检查 {len(roots)} 个 crate root，递归覆盖 {len(seen)} 个 Rust 文件")
    if fails:
        print(f"FAIL —— {len(fails)} 处：")
        for f in fails:
            print("  -", f)
        return 1
    print("PASS —— 所有 mod 声明都能解析到真实文件（不会触发 E0583）。")
    return 0


if __name__ == "__main__":
    sys.exit(main())
