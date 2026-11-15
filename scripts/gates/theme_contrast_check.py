#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
主题 token 门禁：双主题对比度（G05）+ 间距刻度（G13）。

对比度**按真实渲染关系计算**，不验 UI 里不存在的组合：
  · 先剥掉 HTML 注释 / CSS 注释 / <script> —— 注释里的说明文字不是渲染文字
  · 解析三个 .uix 的 <template>，得到元素树（标签 / class / 文本）
  · 解析 <style>，得到 class → 声明
  · 对每个**有文字**的元素：
      前景 = 该元素 class 的 color 规则；无 color 规则 → 库默认
              theme::kBtnText()（core-ui 无 CSS 属性继承，不会从父级继承）
      背景 = 最近祖先-or-self 的 background / background-color；
              没有 → 库默认窗口底
  · var(--app-*) 按 themes.json 的 light/dark 取值

**门禁有效性前提（不得删）**：资产缺失 / 0 字节 / 剥噪后无内容 / 解析不到
非空 <template> / 某主题下全项目零文字元素，一律 FAIL。绝不能用 `continue`
把缺失资产跳过——否则「没有可判对象」会被当成「全部达标」而打印 PASS。
历史上这里正是靠 `if not path.exists(): continue` 静默空过（P0）。

依据：
  references/10-standards/mechanical-gates.md G05 / G13
  assets/themes.json（token 源）、assets/*.uix（真实用色）
  上游 core-ui：src/ui/theme.h（库默认色）、src/ui/page/widget_factory.cpp:1396
  （color 覆盖）、src/ui/css/cascade.cpp:92-110（无属性继承）

用法：
    python3 scripts/gates/theme_contrast_check.py
    python3 scripts/gates/theme_contrast_check.py --root <项目目录>

退出码：0 = 全过；1 = 有 FAIL；2 = 用法错误。
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

SPACE_SCALE = {0, 4, 8, 12, 16, 24, 32, 48}
MIN_CONTRAST = 4.5

ASSETS = ("assets/main.uix", "assets/preview.uix", "assets/about.uix")

# 库默认色（上游 src/ui/theme.h：MakeLight / MakeDark）
#   btnText   = LabelWidget 无 color 规则时的默认文字色（controls.cpp:68 theme::kBtnText()）
#   windowBg  = 无任何祖先背景时的兜底窗口底
LIB_DEFAULTS = {
    "light": {"btnText": "#242424", "windowBg": "#FFFFFF"},   # grey::g14 / white
    "dark":  {"btnText": "#FFFFFF", "windowBg": "#292929"},   # white / grey::g16
}


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


def strip_noise(src: str) -> str:
    src = re.sub(r"<!--.*?-->", "", src, flags=re.S)
    src = re.sub(r"/\*.*?\*/", "", src, flags=re.S)
    src = re.sub(r"<script>.*?</script>", "", src, flags=re.S)
    return src


def load_asset_sources(root: Path, fails: list) -> dict:
    """统一读取并校验三个 .uix，返回 {rel: 剥噪后源码}。

    任何一处资产不可用都写入 fails，而不是跳过。返回 dict 可能少于 3 项，
    调用方仍会继续跑完剩余资产以便一次报全，但最终结论必为 FAIL。
    """
    sources = {}
    for rel in ASSETS:
        path = root / rel
        if not path.exists():
            fails.append(f"{rel}: 资产缺失 —— 门禁拒绝空过（缺资产无法证明对比度/间距）")
            continue
        try:
            raw = path.read_text(encoding="utf-8")
        except (OSError, UnicodeError) as exc:
            fails.append(f"{rel}: 资产读取/解码失败 —— 门禁拒绝空过（{exc}）")
            continue
        if not raw.strip():
            fails.append(f"{rel}: 资产为空（0 字节或纯空白）—— 门禁拒绝空过")
            continue
        src = strip_noise(raw)
        if not src.strip():
            fails.append(f"{rel}: 剥除注释/脚本后无有效内容 —— 门禁拒绝空过")
            continue
        tm = re.search(r"<template[^>]*>(.*?)</template>", src, flags=re.S)
        if not tm or not tm.group(1).strip():
            fails.append(f"{rel}: 解析不到非空 <template> 模板体 —— 门禁拒绝空过")
            continue
        if not parse_template(src):
            fails.append(f"{rel}: <template> 内没有任何元素节点 —— 门禁拒绝空过")
            continue
        sources[rel] = src
    return sources


def parse_style(src: str) -> dict:
    """class 选择器 → {property: value}（只解析 .foo { ... } 形式）。"""
    rules: dict = {}
    for m in re.finditer(r"\.([A-Za-z][\w-]*)\s*\{([^}]*)\}", src):
        name, body = m.group(1), m.group(2)
        decls = rules.setdefault(name, {})
        for d in re.finditer(r"([a-z-]+)\s*:\s*([^;]+);", body):
            decls[d.group(1).strip()] = d.group(2).strip()
    return rules


def parse_template(src: str) -> list:
    """返回模板体元素树：{tag, classes, text, children[]}（栈式扫描，够用即可）。"""
    m = re.search(r"<template>(.*?)</template>", src, flags=re.S)
    if not m:
        return []
    body = m.group(1)
    root = []
    stack = []
    pos = 0
    tag_re = re.compile(r"<(/?)([A-Za-z][\w-]*)([^>]*?)(/?)>")
    while True:
        tm = tag_re.search(body, pos)
        if not tm:
            break
        between = body[pos:tm.start()]
        if stack and between.strip():
            stack[-1]["text"] += between.strip()
        pos = tm.end()
        closing, tag, attrs, selfclose = tm.groups()
        if closing:
            if stack:
                stack.pop()
            continue
        classes = []
        cm = re.search(r'class\s*=\s*"([^"]*)"', attrs)
        if cm:
            classes = cm.group(1).split()
        el = {"tag": tag, "classes": classes, "text": "", "children": []}
        if stack:
            stack[-1]["children"].append(el)
        else:
            root.append(el)
        if not selfclose:
            stack.append(el)
    return root


def resolve_color(value: str, theme: dict, mode: str) -> str | None:
    """把 background/color 的值解析成具体色值（只支持 var(--app-*) 与 #hex）。"""
    value = value.strip()
    vm = re.match(r"var\(\s*(--[A-Za-z0-9_-]+)\s*(?:,\s*(.+?))?\s*\)$", value)
    if vm:
        name = vm.group(1)
        if name in theme:
            return theme[name]
        return None  # 库 token 或未定义 → 交给调用方兜底
    if value.startswith("#") and re.fullmatch(r"#[0-9a-fA-F]{3,8}", value):
        return value
    return None


def element_pairs(tree: list, rules: dict, theme: dict, mode: str):
    """产出 (元素描述, 前景色, 背景色, 文本)。"""
    out = []

    def walk(el, inherited_bg):
        classes = el["classes"]
        decls = {}
        for c in classes:
            decls.update(rules.get(c, {}))

        # 背景：自身优先，否则继承最近祖先
        bg = inherited_bg
        bg_value = decls.get("background") or decls.get("background-color")
        if bg_value:
            resolved = resolve_color(bg_value, theme, mode)
            if resolved:
                bg = resolved

        if el["text"]:
            fg_value = decls.get("color")
            fg = resolve_color(fg_value, theme, mode) if fg_value else None
            if fg is None:
                fg = LIB_DEFAULTS[mode]["btnText"]
            if bg is None:
                bg = LIB_DEFAULTS[mode]["windowBg"]
            desc = f"{el['tag']}.{'.'.join(classes) if classes else '(无 class)'}"
            out.append((desc, fg, bg, el["text"][:24]))

        for child in el["children"]:
            walk(child, bg)

    for el in tree:
        walk(el, None)
    return out


def check_spacing(sources: dict) -> list[str]:
    """G13：padding / margin / gap 的取值必须落在刻度上。

    sources 由 load_asset_sources 产出；缺资产已在那里判 FAIL，这里不再 continue。
    """
    fails: list[str] = []
    pattern = re.compile(r"\b(padding|margin|gap)(-[a-z]+)?\s*:\s*([^;}]+)")
    for rel, src in sources.items():
        for lineno, line in enumerate(src.splitlines(), 1):
            for m in pattern.finditer(line):
                for token in m.group(3).split():
                    if not token.endswith("px"):
                        continue
                    try:
                        value = int(token[:-2])
                    except ValueError:
                        continue
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

    fails: list[str] = []
    sources = load_asset_sources(root, fails)

    print("=== G05 双主题对比度（按 .uix 真实渲染关系计算，已剥注释/脚本）===")
    for mode in ("light", "dark"):
        theme = data.get(mode)
        if not isinstance(theme, dict) or not theme:
            fails.append(f"themes.json 缺 {mode} 段或该段为空 —— 门禁拒绝空过")
            continue
        all_pairs = []
        for rel, src in sources.items():
            rules = parse_style(src)
            tree = parse_template(src)
            for desc, fg, bg, text in element_pairs(tree, rules, theme, mode):
                ratio = contrast_ratio(fg, bg)
                all_pairs.append((rel, desc, fg, bg, ratio, text))
                if ratio < MIN_CONTRAST:
                    fails.append(
                        f"[{mode}] {rel} {desc}: {fg} on {bg} = {ratio:.2f}:1 "
                        f"< {MIN_CONTRAST}:1 （文本「{text}」）"
                    )
        if not all_pairs:
            # 空集不能判「全部达标」——这是本门禁最容易被绕过的地方。
            fails.append(f"[{mode}] 真实文字元素 0 处 —— 门禁拒绝空过"
                         f"（没有可判对象不等于全部达标）")
            print(f"  [{mode}] 真实文字元素 0 处（已判 FAIL）")
            continue
        all_pairs.sort(key=lambda x: x[4])
        print(f"  [{mode}] 真实文字元素 {len(all_pairs)} 处，最低 "
              f"{all_pairs[0][4]:.2f}:1（{all_pairs[0][1]} "
              f"{all_pairs[0][2]} on {all_pairs[0][3]}）")
        for rel, desc, fg, bg, ratio, text in all_pairs[:3]:
            print(f"        {rel} {desc}: {fg} on {bg} = {ratio:.2f}:1 「{text}」")

    print("\n=== G13 间距刻度 ===")
    spacing_fails = check_spacing(sources)
    fails += spacing_fails
    print(f"  已检资产 {len(sources)}/{len(ASSETS)} 个，非刻度间距：{len(spacing_fails)} 处")

    print()
    if fails:
        print(f"FAIL —— {len(fails)} 处：")
        for f in fails:
            print("  -", f)
        return 1
    print("PASS —— 真实文字元素的对比度全部 ≥ 4.5:1，间距取值全部在刻度上。")
    return 0


# Windows runner 的 stdout 默认 cp1252，直接打印中文会抛 UnicodeEncodeError，
# 让门禁以「看起来失败」的方式崩掉（第 12 次 CI 的 Gate S14 实测）。
# 强制 UTF-8，保证脚本在 CI / 本机 / 被 import 时行为一致。
for _stream in (sys.stdout, sys.stderr):
    if hasattr(_stream, "reconfigure"):
        _stream.reconfigure(encoding="utf-8", errors="replace")

if __name__ == "__main__":
    sys.exit(main())
