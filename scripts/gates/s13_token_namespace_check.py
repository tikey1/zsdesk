#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
门禁 S13 —— 项目 token 命名空间不得与 core-ui 库的 34 个 token 撞名

用法：
    python3 scripts/gates/s13_token_namespace_check.py                 # 用当前目录当项目根
    python3 scripts/gates/s13_token_namespace_check.py --root <项目目录>

退出码：0 = 通过（可含 WARN）；1 = 有 FAIL；2 = 用法/环境错误。

--------------------------------------------------------------------------
逐字口径（架构师给定，QA 只做实现，不改口径）
--------------------------------------------------------------------------
  ① name ∈ 库的 34 个 token 全集 → 判失败；
  ② .uix 内定义 --app-accent → 判失败；
  ③ themes.json 内的 --app-accent 允许（D14 的回退源值，不是 CSS 变量）；
  ④ var(--accent) 等引用不拦。
  另：.uix 内不得出现 var(--app-accent)。

--------------------------------------------------------------------------
QA 的两条实现决定（架构师授权：「放哪个脚本」「名单 vs 命名空间」由 QA 定）
--------------------------------------------------------------------------
  · 决定一：**独立成脚本**（本文件），不塞进 qa_uix_verify.py。
    理由：S13 是**契约门禁**（有独立编号，会被 spec/milestones.md 与
    spec/ci-contract.md 引用），独立脚本可被 CI 单点调用、退出码自洽；
    qa_uix_verify.py 是「资产质量门」，两者职责不同、不互相依赖。
    注：本脚本的「另：.uix 内不得出现 var(--app-accent)」与
    qa_uix_verify.py 的断言 16 **有意重叠** —— 两个独立执行器，
    一个门禁失败不应掩盖另一个（mechanical-gates.md：没有执行器的规则只是建议）。

  · 决定二：① 做成【**名单**】而不是【命名空间不相交】。
    理由：项目 token 集是 themes.json 里**显式枚举**的一组名字，不是靠前缀推导的
    命名空间 ⇒ 「--app- 前缀 vs 库 token 名」这个判据是**空的**（--app-bg 天然 ≠ --bg）。
    S13 要挡的真实失败模式是「**把库 token 名误抄进项目 token 集**」
    （例如在 themes.json 里写 "--bg": "#FFF"、或在 .uix 的 :root 里写 --bg:）。
    前缀规则挡不住这种错，**只有 34 个名字的成员判定挡得住**。

--------------------------------------------------------------------------
名单来源与自检
--------------------------------------------------------------------------
  LIB_TOKENS 逐字抄自上游 core-ui：
      src/ui/page/compiler.cpp:1648-1658  static const char* kThemeKeys[] = { ... }
  （这 34 个键会被 RebuildThemeVars 擦除重写 —— src/ui/page/compiler.cpp:1647-1703）

  **名单漂移自检**：升级 core-ui 后若库新增了 token，本名单会过时。
  一条命令即可复核（需上游只读副本）：
      sed -n '1648,1658p' <上游>/src/ui/page/compiler.cpp
  本脚本每次运行都会把名单**排序打印**出来，便于与上面一行输出逐字 diff。
  名单条数不是 34 时**直接 FAIL**（不静默放过）。

本脚本**只读**：不改任何资产、不写任何文件。
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

# --------------------------------------------------------------------------
# 库的 34 个主题 token —— 逐字抄自 src/ui/page/compiler.cpp:1648-1658
# --------------------------------------------------------------------------
LIB_TOKENS = {
    "--accent", "--accent-hover", "--accent-press", "--accent-text", "--accent-selected",
    "--window-bg", "--window-border",
    "--bg", "--bg-2", "--bg-3", "--bg-4", "--bg-elevated",
    "--fg", "--fg-2", "--fg-3", "--fg-4", "--fg-on-accent",
    "--border", "--border-subtle",
    "--sidebar-bg", "--sidebar-text", "--sidebar-hover",
    "--input-bg", "--input-border", "--input-border-hover", "--input-border-focus",
    "--card-bg", "--card-border",
    "--btn-bg", "--btn-hover", "--btn-press", "--btn-text",
    "--disabled-bg", "--disabled-text",
}
LIB_TOKEN_COUNT = 34

ACCENT_FALLBACK = "--app-accent"   # D14：Rust 的回退源值（数据字段），不是 CSS 变量

ASSETS = ("assets/main.uix", "assets/preview.uix", "assets/about.uix")
THEMES = "assets/themes.json"

# `--name:` 形式的【定义】；`var(--name)` 是【引用】，不算定义（口径 ④）
DEF_RE = re.compile(r"(?<![\w-])(--[A-Za-z0-9_-]+)\s*:")
REF_RE = re.compile(r"var\(\s*(--[A-Za-z0-9_-]+)")


class Report:
    def __init__(self):
        self.fails: list[str] = []
        self.warns: list[str] = []

    def ok(self, msg):
        print(f"  PASS  {msg}")

    def fail(self, msg):
        self.fails.append(msg)
        print(f"  FAIL  {msg}")

    def warn(self, msg):
        self.warns.append(msg)
        print(f"  WARN  {msg}")


def strip_comments(src: str) -> str:
    """剥掉 HTML 注释与 CSS 注释 —— 注释里的 token 名不是定义。"""
    src = re.sub(r"<!--.*?-->", "", src, flags=re.S)
    src = re.sub(r"/\*.*?\*/", "", src, flags=re.S)
    return src


def scan_uix(rel: str, root: Path, rep: Report):
    path = root / rel
    raw = path.read_text(encoding="utf-8")
    src = strip_comments(raw)

    defs: list[tuple[str, int]] = []   # (token, 行号)
    for m in DEF_RE.finditer(src):
        name = m.group(1)
        # 排除 var(--x) 形态（理论上不会同时满足 `--x:`，但显式排除更稳）
        if src[max(0, m.start() - 4):m.start()].endswith("var("):
            continue
        defs.append((name, src.count("\n", 0, m.start()) + 1))

    refs: list[tuple[str, int]] = []
    for m in REF_RE.finditer(src):
        refs.append((m.group(1), src.count("\n", 0, m.start()) + 1))

    print(f"\n=== {rel} ===")

    # ① 定义了库 token → FAIL
    hit_lib = [(n, ln) for n, ln in defs if n in LIB_TOKENS]
    for n, ln in hit_lib:
        rep.fail(f"{rel}:{ln}: 定义了库的 token `{n}` → 与库 token 撞名。"
                 f"该键会被 RebuildThemeVars 擦除重写（compiler.cpp:1647-1703），"
                 f"值被静默接管；项目 token 一律加 --app- 前缀")
    if not hit_lib:
        rep.ok(f"① 定义侧：{len(defs)} 个 token 定义，零个撞库 token（库 34 个全集）")

    # ② .uix 内定义 --app-accent → FAIL
    hit_acc_def = [(n, ln) for n, ln in defs if n == ACCENT_FALLBACK]
    for n, ln in hit_acc_def:
        rep.fail(f"{rel}:{ln}: 定义了 `{ACCENT_FALLBACK}` —— 它是 Rust 的【回退源值"
                 f"（数据字段，不是 CSS 变量）】，不进 :root；.uix 一律读库 token var(--accent)")
    if not hit_acc_def:
        rep.ok(f"② 定义侧：未定义 `{ACCENT_FALLBACK}`")

    # 另：.uix 内不得出现 var(--app-accent) → FAIL
    hit_acc_ref = [(n, ln) for n, ln in refs if n == ACCENT_FALLBACK]
    for n, ln in hit_acc_ref:
        rep.fail(f"{rel}:{ln}: 出现 `var({ACCENT_FALLBACK})` —— accent 在 .uix 里"
                 f"一律读库 token var(--accent)，否则库自绘控件与自绘区会撞成两种蓝")
    if not hit_acc_ref:
        rep.ok(f"另：未出现 `var({ACCENT_FALLBACK})`")

    # ④ 引用不拦 —— 但要显式区分出来，证明「不拦」是有意为之
    lib_refs = sorted({n for n, _ in refs if n in LIB_TOKENS})
    rep.ok(f"④ 引用侧：{len(refs)} 处 var() 引用**不拦**"
           + (f"（其中引用了库 token {lib_refs} —— 合法，accent 必须读 var(--accent)）"
              if lib_refs else "（未引用任何库 token）"))

    # S13 附加（QA 增补，不改变上面四条逐字口径）：
    # 静态侧对应 Rust 自检断言 3「注入的 :root 内每条声明匹配 --app-[a-z0-9-]+:」
    bad_shape = [(n, ln) for n, ln in defs
                 if not re.fullmatch(r"--app-[a-z0-9-]+", n)]
    for n, ln in bad_shape:
        rep.fail(f"{rel}:{ln}: token 名 `{n}` 不符合 `--app-[a-z0-9-]+` —— "
                 f"静态侧对应 Rust 自检断言 3（spec/ffi-core-ui.md §4.3.2.1）")
    if not bad_shape:
        rep.ok("附加：全部 token 名符合 `--app-[a-z0-9-]+`（Rust 自检断言 3 的静态对应）")


def scan_themes(root: Path, rep: Report):
    path = root / THEMES
    print(f"\n=== {THEMES} ===")
    if not path.exists():
        rep.fail(f"{THEMES}: 文件不存在 —— token 源缺失，Rust 注入无从谈起")
        return

    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as e:
        rep.fail(f"{THEMES}: JSON 解析失败 → {e}")
        return

    # 定义 = 所有以 `--` 开头的 JSON **键**（递归）。
    # prose（revision.* 里的说明文字）不是定义，不扫。
    defs: list[tuple[str, str]] = []   # (token, 路径)
    refs: list[tuple[str, str]] = []

    def walk(node: str, obj):
        if isinstance(obj, dict):
            for k, v in obj.items():
                if k.startswith("--"):
                    defs.append((k, node + "/" + k))
                walk(node + "/" + k, v)
        elif isinstance(obj, list):
            for i, v in enumerate(obj):
                walk(f"{node}[{i}]", v)
        elif isinstance(obj, str):
            for m in REF_RE.finditer(obj):
                refs.append((m.group(1), node))

    walk("", data)

    # ① 定义了库 token → FAIL
    hit_lib = [(n, p) for n, p in defs if n in LIB_TOKENS]
    for n, p in hit_lib:
        rep.fail(f"{THEMES}: 定义了库的 token `{n}`（{p}）→ 与库 token 撞名，"
                 f"该键会被 RebuildThemeVars 擦除重写（compiler.cpp:1647-1703）")
    if not hit_lib:
        rep.ok(f"① 定义侧：{len(defs)} 个 token 定义，零个撞库 token（库 34 个全集）")

    # ③ themes.json 内的 --app-accent 允许
    hit_acc = [(n, p) for n, p in defs if n == ACCENT_FALLBACK]
    if hit_acc:
        rep.ok(f"③ `{ACCENT_FALLBACK}` 在 {THEMES} 内**允许**"
               f"（{', '.join(p for _, p in hit_acc)}）—— D14 的回退源值，不是 CSS 变量；"
               f"由 Rust 读出后直接喂 ui_theme_set_accent_hex()，不进 :root")
    else:
        rep.warn(f"{THEMES}: 未定义 `{ACCENT_FALLBACK}` —— "
                 f"读不到系统强调色时就没有回退源值（D14 要求保留该键）")

    # ④ 引用不拦 —— 但要显式区分「真引用」与「revision.* 说明文字里的提及」
    real_refs = [(n, p) for n, p in refs if p.startswith("/light") or p.startswith("/dark")]
    prose_refs = [(n, p) for n, p in refs if not (p.startswith("/light") or p.startswith("/dark"))]
    lib_refs = sorted({n for n, _ in real_refs if n in LIB_TOKENS})
    rep.ok(f"④ 引用侧：{len(real_refs)} 处真 var() 引用**不拦**"
           + (f"（其中引用了库 token {lib_refs} —— 合法）" if lib_refs else "")
           + (f"；另有 {len(prose_refs)} 处出现在 revision.* 说明文字里 —— 那是文档提及，不是引用"
              if prose_refs else ""))

    # S13 附加：token 名形状
    bad_shape = [(n, p) for n, p in defs if not re.fullmatch(r"--app-[a-z0-9-]+", n)]
    for n, p in bad_shape:
        rep.fail(f"{THEMES}: token 名 `{n}`（{p}）不符合 `--app-[a-z0-9-]+`")
    if not bad_shape:
        rep.ok("附加：全部 token 名符合 `--app-[a-z0-9-]+`（Rust 自检断言 3 的静态对应）")

    # 附加：light / dark 两套的键集必须一致（否则切主题时少一个 token → 静默回落库色）
    lt = set(data.get("light", {}))
    dk = set(data.get("dark", {}))
    only_lt, only_dk = sorted(lt - dk), sorted(dk - lt)
    if only_lt or only_dk:
        rep.fail(f"{THEMES}: light/dark 键集不一致 —— "
                 f"只在 light: {only_lt} / 只在 dark: {only_dk}"
                 f"（切到缺键的那一侧会静默回落到库默认色）")
    else:
        rep.ok(f"附加：light / dark 键集一致（各 {len(lt)} 个）")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", default=".", help="项目根目录（默认当前目录）")
    args = ap.parse_args()
    root = Path(args.root).resolve()

    print("门禁 S13 —— 项目 token 命名空间 vs core-ui 库 34 个 token（静态，不编译不渲染）")
    print(f"项目根：{root}")

    rep = Report()

    # 名单自检：条数不对直接 FAIL，不静默放过
    if len(LIB_TOKENS) != LIB_TOKEN_COUNT:
        rep.fail(f"库 token 名单条数 {len(LIB_TOKENS)} ≠ {LIB_TOKEN_COUNT}"
                 f"（名单抄自 compiler.cpp:1648-1658，升级 core-ui 后须同步）")
    else:
        rep.ok(f"库 token 名单 {LIB_TOKEN_COUNT} 个（compiler.cpp:1648-1658）")
    print("  名单（排序，便于与上游逐字 diff）：")
    for i in range(0, len(sorted(LIB_TOKENS)), 6):
        print("    " + "  ".join(sorted(LIB_TOKENS)[i:i + 6]))

    missing = [a for a in ASSETS if not (root / a).exists()]
    if missing:
        print("找不到资产文件：" + ", ".join(missing))
        return 2

    for rel in ASSETS:
        scan_uix(rel, root, rep)
    scan_themes(root, rep)

    print()
    if rep.fails:
        print(f"FAIL —— {len(rep.fails)} 处阻塞 / {len(rep.warns)} 处提示")
        for f in rep.fails:
            print("  - " + f)
        return 1
    print(f"PASS —— S13 通过（{len(rep.warns)} 处提示）")
    for w in rep.warns:
        print("  ! " + w)
    return 0


if __name__ == "__main__":
    sys.exit(main())
