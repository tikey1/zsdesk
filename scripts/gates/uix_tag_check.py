#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
`.uix` 资产静态门禁（core-ui 标签 / D-15 / SFC 骨架 / id-class 保全）。

依据：
  - docs/11-COREUI-MARKUP-TAGS.md §1 支持表（权威源 = 上游 core-ui
    src/ui/page/widget_factory.cpp → ConstructByTag）
  - docs/12-UIX-TAG-FIX.md §6 自检结论

用法：
    python3 scripts/gates/uix_tag_check.py
    python3 scripts/gates/uix_tag_check.py --root <项目目录>

退出码：0 = 全过；1 = 有 FAIL；2 = 用法错误。
不做编译、不做渲染 —— 只做 L1 静态门。
"""
from __future__ import annotations

import argparse
import collections
import re
import sys
from pathlib import Path

# ---- core-ui 支持表（docs/11 §1）------------------------------------------
SUPPORTED_TAGS = {
    # §1.1 布局容器
    "div", "section", "article", "main", "header", "footer", "aside", "nav",
    "ul", "ol", "li",
    # §1.2 文本
    "span", "a", "label", "small", "strong", "em", "p",
    "h1", "h2", "h3", "h4", "h5", "h6",
    # §1.3 控件
    "button", "input", "textarea", "select", "chips-input", "ChipsInput",
    "Toggle", "toggle", "ProgressBar", "progress-bar", "progressbar",
    "Separator", "separator", "hr", "TitleBar", "title-bar",
    "ScrollView", "scroll-view", "Expander", "expander", "tabs", "TabControl",
    "custom", "Custom", "img", "svg",
    "gh_img_view", "gh-img-view", "gh_img", "gh-img",
    "ocr_img_view", "ocr-img-view", "OcrImgView", "ocr-img",
    # §1.4 特殊（不产生控件）
    "link", "meta", "head", "title",
}

# SFC 顶层块（src/ui/uix/sfc_parser.cpp 消费，不是控件）
SFC_TAGS = {"window", "template", "script", "style", "import"}

# core-ui 不识别、必须换掉的旧标签（docs/11 §2）
LEGACY_TAGS = {"text", "hbox", "vbox", "vseparator", "text-input", "panel"}

# D-15：静态布局 only —— 禁止出现的事件 / 响应式属性
FORBIDDEN_ATTRS = (
    "onClick", "onChange", "onInput", "onKeyDown", "onMouseDown",
    "v-model", "v-if", "v-for", "v-show", ":visible", "@click", "@change",
)

ASSETS = ("assets/main.uix", "assets/preview.uix", "assets/about.uix")

# 修正前后必须一致的**模板体**元素数（docs/12 §0；不含 <window>/<style>/<template>
# 三个 SFC 外壳 —— 那是文件外壳，不是界面元素）
EXPECTED_ELEMENT_COUNT = {
    "assets/main.uix": 30,
    "assets/preview.uix": 9,
    "assets/about.uix": 16,
}
# 必须存在的 id（Rust 侧靠 ui_widget_find_by_id 取句柄）
EXPECTED_IDS = {
    "assets/main.uix": {
        "titlebar_host", "global_search", "tab_host", "tab_placeholder",
        "command_host", "nav_hint", "address_host", "breadcrumb", "address",
        "search", "infobar_host", "nav_host", "list_host", "preview_host",
        "status_host", "status_item_count", "status_selected", "status_text",
        "status_progress", "status_thumb_cache", "status_image_channel",
    },
    "assets/preview.uix": {
        "preview_title", "preview_body", "preview_hint", "preview_channel",
        "preview_size", "preview_state",
    },
    "assets/about.uix": set(),
}


def strip_noise(src: str) -> str:
    """剥掉 HTML 注释、CSS 注释、<script> 块 —— 它们不该计入标签统计。"""
    src = re.sub(r"<!--.*?-->", "", src, flags=re.S)
    src = re.sub(r"/\*.*?\*/", "", src, flags=re.S)
    src = re.sub(r"<script>.*?</script>", "", src, flags=re.S)
    return src


def template_body(src: str) -> str:
    """取 <template>…</template> 内部 —— 界面元素只在这里面。"""
    m = re.search(r"<template>(.*?)</template>", src, flags=re.S)
    return m.group(1) if m else ""


def check_file(path: Path, root: Path) -> list[str]:
    rel = path.relative_to(root).as_posix()
    fails: list[str] = []
    src = path.read_text(encoding="utf-8")
    body = strip_noise(src)

    counts = collections.Counter(
        re.findall(r"<\s*(?!/)([A-Za-z][A-Za-z0-9_-]*)", body)
    )
    tpl_counts = collections.Counter(
        re.findall(r"<\s*(?!/)([A-Za-z][A-Za-z0-9_-]*)", template_body(body))
    )
    print(f"\n=== {rel} ===")
    for tag, n in sorted(counts.items()):
        if tag in SUPPORTED_TAGS:
            kind = "core-ui 支持"
        elif tag in SFC_TAGS:
            kind = "SFC 顶层块"
        else:
            kind = "!! core-ui 不识别"
            fails.append(f"{rel}: 未知标签 <{tag}> ×{n}")
        print(f"  <{tag}> ×{n:<3} {kind}")

    # 1. 旧标签零残留
    for tag in LEGACY_TAGS:
        if counts.get(tag):
            fails.append(f"{rel}: 遗留旧标签 <{tag}> ×{counts[tag]}")

    # 2. 模板体元素数不变
    total = sum(tpl_counts.values())
    want = EXPECTED_ELEMENT_COUNT.get(rel)
    if want is not None and total != want:
        fails.append(f"{rel}: 模板体元素数 {total} ≠ 期望 {want}")

    # 3. id 全在
    ids = set(re.findall(r'\bid\s*=\s*"([^"]+)"', body))
    missing = EXPECTED_IDS.get(rel, set()) - ids
    if missing:
        fails.append(f"{rel}: 丢失 id {sorted(missing)}")

    # 4. D-15：零事件 / 响应式属性
    for attr in FORBIDDEN_ATTRS:
        if re.search(rf"(?<![-\w]){re.escape(attr)}\s*=", body):
            fails.append(f"{rel}: 出现 D-15 禁止的属性 {attr}")

    # 5. text="..." 属性在 .uix 不生效（只有 <menu> 例外）
    if re.search(r'\btext\s*=\s*"', body):
        fails.append(f"{rel}: 出现 text=\"...\" 属性（.uix 里只对 <menu> 生效）")

    # 6. <window> 必须自闭合
    m = re.search(r"<window[^>]*?>", body)
    if m and not m.group(0).rstrip().endswith("/>"):
        fails.append(f"{rel}: <window> 未自闭合 → SFC parse failed，整页不加载")

    # 7. <script> 必须 export default
    sm = re.search(r"<script>(.*?)</script>", src, flags=re.S)
    if sm and "export default" not in sm.group(1):
        fails.append(f"{rel}: <script> 缺 export default → QuickJS 运行时不会挂载")

    # 8. 无 window{} 选择器（<window> 是 SFC 外壳，不产生控件）
    if re.search(r"(^|\})\s*window\s*\{", body):
        fails.append(f"{rel}: 仍有 window {{ }} 选择器（不匹配任何控件）")

    # 9. 硬编码颜色（机械门禁 G02）—— .uix 里只允许 var(--token)
    for lineno, line in enumerate(body.splitlines(), 1):
        if re.search(r"#[0-9a-fA-F]{3,8}\b", line):
            fails.append(f"{rel}:{lineno}: 硬编码色值 → {line.strip()}")

    if not fails:
        print(f"  ✔ 模板体元素 {total} 个 / id {len(ids)} 个 —— 全部检查通过")
    return fails


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", default=".", help="项目根目录（默认当前目录）")
    args = ap.parse_args()
    root = Path(args.root).resolve()

    missing = [a for a in ASSETS if not (root / a).exists()]
    if missing:
        print("找不到资产文件：" + ", ".join(missing))
        return 2

    all_fails: list[str] = []
    for rel in ASSETS:
        all_fails += check_file(root / rel, root)

    print()
    if all_fails:
        print(f"FAIL —— {len(all_fails)} 处：")
        for f in all_fails:
            print("  -", f)
        return 1
    print("PASS —— 三个 .uix 资产：标签全部在 core-ui 支持表内，"
          "零遗留旧标签，D-15 合规，id 集合与模板体元素数保全。")
    return 0


if __name__ == "__main__":
    sys.exit(main())
