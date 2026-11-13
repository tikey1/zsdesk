#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
QA 复核执行器 —— `.uix` 资产（core-ui 静态审查第二轮可复跑）

依据：上游 core-ui 源码（只读副本）逐条核对得出，权威行号见 docs/14-QA-UIX-REVIEW.md §7。
本脚本只读、不改任何资产；是 L1 静态门，不做编译、不做渲染。

用法：
    python3 scripts/gates/qa_uix_verify.py                 # 用当前目录当项目根
    python3 scripts/gates/qa_uix_verify.py --root <项目目录>

退出码：0 = 无 FAIL（可含 WARN）；1 = 有 FAIL；2 = 用法/环境错误。

输出格式：每条给 PASS / FAIL / WARN + 文件:行（`mechanical-gates.md` §四.3 只上报真正跑过的命令）。
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

# --------------------------------------------------------------------------
# 1. core-ui 标签支持表
#    源：src/ui/page/widget_factory.cpp → ConstructByTag (913-1194) + ConstructInput (1197-1284)
# --------------------------------------------------------------------------
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
    # §1.4 特殊（返回 nullptr → 不产生控件，但会产生一条非致命编译告警）
    "link", "meta", "head", "title",
}
SFC_TAGS = {"window", "template", "script", "style", "import", "link"}
LEGACY_TAGS = {"text", "hbox", "vbox", "vseparator", "text-input", "panel"}

# --------------------------------------------------------------------------
# 2. core-ui 支持的 CSS 属性全集（66 项）
#    源：src/ui/page/widget_factory.cpp 中全部 Has("…") 取值
#    不在其中 → 声明被静默丢弃
# --------------------------------------------------------------------------
SUPPORTED_PROPS = {
    "accent-color", "align-items", "align-self", "backdrop-blur", "backdrop-filter",
    "background", "background-color", "background-image", "background-position",
    "background-size", "border", "border-color", "border-radius", "border-width",
    "box-shadow", "caret-color", "color", "column-gap", "cursor", "display",
    "flex", "flex-direction", "flex-grow", "flex-shrink", "flex-wrap", "font-size",
    "font-weight", "gap", "height", "item-background", "item-bg", "item-border-color",
    "item-hover-background", "item-hover-bg", "justify-content", "line-height",
    "margin", "margin-bottom", "margin-left", "margin-right", "margin-top",
    "opacity", "overflow", "padding", "padding-bottom", "padding-left",
    "padding-right", "padding-top", "placeholder-color", "position", "row-gap",
    "selected-background", "selected-bg", "selected-color", "selected-radius",
    "selection-background-color", "selection-color", "selection-inactive-background-color",
    "selection-inactive-color", "text-align", "transform", "transition",
    "user-select", "visibility", "white-space", "width",
}
# 已知「被支持但语义与直觉不同」或「压根不支持」的属性 —— 单独给说明，不当 FAIL
PROP_NOTES = {
    "border-bottom": "不受支持（只认 border/border-color/border-width/border-radius，widget_factory.cpp:631-653）",
    "font-family":  "对普通 LabelWidget 不生效（仅 SVG 文本读取）；字体由 ui_theme_set_default_font 决定",
}

# --------------------------------------------------------------------------
# 3. core-ui 库自有的 34 个主题 token
#    源：src/ui/page/compiler.cpp:1648-1658 (kThemeKeys[])
#    这些键会被 RebuildThemeVars 擦除重写；同名用户 :root 值在切模式后被库接管
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

# --------------------------------------------------------------------------
# 4. 必须保全的 id（Rust 侧靠 ui_widget_find_by_id 取句柄）
#    实测自 git show HEAD:assets/*.uix，逐个核对拼写与大小写
# --------------------------------------------------------------------------
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

# 模板体元素数（不含 window/style/template 三个 SFC 外壳标签）
EXPECTED_TEMPLATE_ELEMENTS = {
    "assets/main.uix": 30,
    "assets/preview.uix": 9,
    "assets/about.uix": 16,
}

ASSETS = ("assets/main.uix", "assets/preview.uix", "assets/about.uix")
FORBIDDEN_ATTRS = (
    "onClick", "onChange", "onInput", "onKeyDown", "onMouseDown",
    "v-model", "v-if", "v-for", "v-show", ":visible", "@click", "@change",
)
TEXT_TAGS = {"span", "label", "p", "h1", "h2", "h3", "h4", "h5", "h6",
             "small", "strong", "em", "a"}


# --------------------------------------------------------------------------
# 工具
# --------------------------------------------------------------------------
def strip_noise(src: str) -> str:
    """剥掉 HTML 注释、CSS 注释、<script> 块 —— 它们不该计入标签统计。"""
    src = re.sub(r"<!--.*?-->", "", src, flags=re.S)
    src = re.sub(r"/\*.*?\*/", "", src, flags=re.S)
    src = re.sub(r"<script\b.*?</script>", "", src, flags=re.S)
    return src


def style_block(src: str) -> str:
    """取 <style> 块内容（先剥 HTML 注释，避免匹配到注释里提到的 <style>）。"""
    src = re.sub(r"<!--.*?-->", "", src, flags=re.S)
    m = re.search(r"<style\b[^>]*>(.*?)</style>", src, flags=re.S)
    if not m:
        return ""
    return re.sub(r"/\*.*?\*/", "", m.group(1), flags=re.S)


def template_block(src: str) -> str:
    src = re.sub(r"<!--.*?-->", "", src, flags=re.S)
    m = re.search(r"<template>(.*?)</template>", src, flags=re.S)
    return m.group(1) if m else ""


def rel_lum(hex_color: str) -> float:
    h = hex_color.lstrip("#")
    if len(h) == 3:
        h = "".join(c * 2 for c in h)
    if len(h) < 6:
        return 0.0
    r, g, b = (int(h[i:i + 2], 16) / 255 for i in (0, 2, 4))
    f = lambda c: c / 12.92 if c <= 0.03928 else ((c + 0.055) / 1.055) ** 2.4
    r, g, b = f(r), f(g), f(b)
    return 0.2126 * r + 0.7152 * g + 0.0722 * b


def contrast(a: str, b: str) -> float:
    l1, l2 = rel_lum(a), rel_lum(b)
    if l1 < l2:
        l1, l2 = l2, l1
    return (l1 + 0.05) / (l2 + 0.05)


def token_of(value: str):
    m = re.search(r"var\(\s*(--[a-zA-Z0-9_-]+)", value or "")
    return m.group(1) if m else None


def parse_dom(tmpl: str):
    """极简 DOM 解析：返回根元素（children 列表）。只处理标签/属性/自闭合。"""
    toks = re.findall(
        r"<([A-Za-z][\w-]*)((?:\"[^\"]*\"|[^>\"])*?)(/?)>|</([A-Za-z][\w-]*)>", tmpl)
    stack, tree = [], []
    for name, attrs, selfc, close in toks:
        if close:
            if stack:
                stack.pop()
            continue
        cm = re.search(r"class=\"([^\"]*)\"", attrs or "")
        classes = cm.group(1).split() if cm else []
        el = {"tag": name, "cls": classes, "children": []}
        (stack[-1]["children"] if stack else tree).append(el)
        if not selfc:
            stack.append(el)
    return tree


# --------------------------------------------------------------------------
# 检查
# --------------------------------------------------------------------------
class Report:
    def __init__(self):
        self.fails, self.warns = [], []

    def ok(self, msg):
        print(f"  PASS  {msg}")

    def fail(self, msg):
        self.fails.append(msg)
        print(f"  FAIL  {msg}")

    def warn(self, msg):
        self.warns.append(msg)
        print(f"  WARN  {msg}")


def check_file(rel: str, root: Path, rep: Report, themes: dict):
    path = root / rel
    src = path.read_text(encoding="utf-8")
    body = strip_noise(src)
    css = style_block(src)
    tmpl = template_block(src)
    print(f"\n=== {rel} ===")

    # --- 1. 标签白名单 ---
    counts = {}
    for t in re.findall(r"<\s*(?!/)([A-Za-z][A-Za-z0-9_-]*)", body):
        counts[t] = counts.get(t, 0) + 1
    unknown = {t: n for t, n in counts.items()
               if t not in SUPPORTED_TAGS and t not in SFC_TAGS}
    if unknown:
        for t, n in sorted(unknown.items()):
            rep.fail(f"{rel}: core-ui 不识别的标签 <{t}> ×{n} "
                     f"→ 静默变成空 VBox（widget_factory.cpp:1192-1193），不是报错")
    else:
        rep.ok(f"标签白名单：{len(counts)} 种，全部在 core-ui 支持表内")

    # --- 2. 旧标签零残留 ---
    legacy = {t: counts[t] for t in LEGACY_TAGS if counts.get(t)}
    if legacy:
        rep.fail(f"{rel}: 遗留旧标签 {legacy}")
    else:
        rep.ok("旧标签（text/hbox/vbox/vseparator/text-input/panel）零残留")

    # --- 3. 模板体元素数 ---
    tmpl_tags = re.findall(r"<([A-Za-z][\w-]*)", tmpl)
    want = EXPECTED_TEMPLATE_ELEMENTS.get(rel)
    if want is not None and len(tmpl_tags) != want:
        rep.fail(f"{rel}: 模板体元素数 {len(tmpl_tags)} ≠ 期望 {want}")
    else:
        rep.ok(f"模板体元素数 {len(tmpl_tags)}（不含 3 个 SFC 外壳标签）")

    # --- 4. id 保全 ---
    ids = set(re.findall(r"\bid\s*=\s*\"([^\"]+)\"", body))
    expect = EXPECTED_IDS.get(rel, set())
    lost = expect - ids
    extra = ids - expect
    if lost:
        rep.fail(f"{rel}: 丢失 id {sorted(lost)}（Rust 侧 ui_widget_find_by_id 会取不到）")
    elif extra:
        rep.warn(f"{rel}: 新增 id {sorted(extra)}（确认 Rust 侧有对应实现）")
    else:
        rep.ok(f"id 保全：{len(ids)} 个，与改前完全一致")

    # --- 5. D-15 ---
    d15 = []
    for attr in FORBIDDEN_ATTRS:
        if re.search(rf"(?<![-\w]){re.escape(attr)}\s*=", body):
            d15.append(attr)
    if d15:
        rep.fail(f"{rel}: D-15 违规属性 {d15}")
    else:
        rep.ok("D-15：零事件属性 / 零绑定 / 零指令")
    if re.search(r"\{\{.*?\}\}", body, flags=re.S):
        rep.fail(f"{rel}: 存在 {{{{ }}}} 插值（D-15 静态布局 only）")
    if re.search(r"\btext\s*=\s*\"", body):
        rep.fail(f"{rel}: 存在 text=\"...\" 属性（.uix 里只对 <menuitem> 生效）")

    # --- 6. SFC 骨架 ---
    m = re.search(r"<window[^>]*?>", body)
    if m and not m.group(0).rstrip().endswith("/>"):
        rep.fail(f"{rel}: <window> 未自闭合 → SFC parse failed，整页不加载")
    else:
        rep.ok("<window> 自闭合")
    sm = re.search(r"<script>(.*?)</script>", src, flags=re.S)
    if sm and "export default" not in sm.group(1):
        rep.fail(f"{rel}: <script> 缺 export default → QuickJS 运行时不会挂载")
    if re.search(r"(^|\})\s*window\s*\{", body):
        rep.fail(f"{rel}: 仍有 window {{ }} 选择器（不匹配任何控件）")

    # --- 7. 硬编码色值（G02）---
    hard = [(i, l.strip()) for i, l in enumerate(body.splitlines(), 1)
            if re.search(r"#[0-9a-fA-F]{3,8}\b", l)]
    for i, l in hard:
        rep.fail(f"{rel}:{i}: 硬编码色值 → {l}")
    if not hard:
        rep.ok("G02：零字面色值，全部走 var(--token)")

    # --- 8. CSS 属性支持性 ---
    used = set(re.findall(r"([a-zA-Z-]+)\s*:", css))
    unsupported = sorted(used - SUPPORTED_PROPS)
    for p in unsupported:
        note = PROP_NOTES.get(p, "声明被静默丢弃")
        rep.warn(f"{rel}: CSS 属性 `{p}` 不受支持 → {note}")
    if not unsupported:
        rep.ok(f"CSS 属性：{len(used)} 种，全部受支持")

    # --- 9. font-weight 取值 ---
    for mm in re.finditer(r"font-weight\s*:\s*([^;}]+)", css):
        val = mm.group(1).strip()
        if val not in ("bold", "700"):
            rep.warn(f"{rel}: font-weight: {val} 是空声明"
                     f"（只认 bold/700，widget_factory.cpp:1392-1394）→ 该文字不加粗")

    # --- 10. <separator> + CSS background（已知无效）---
    seps = len(re.findall(r"<separator\b", body))
    if seps and re.search(r"\.divider\s*\{[^}]*background", css):
        rep.warn(f"{rel}: <separator> ×{seps} 上的 CSS background 无效"
                 f"（SeparatorWidget::OnDraw 不调 Widget::OnDraw，controls.cpp:635-643）"
                 f"→ 线条色写死 theme::kDivider()")

    # --- 11. class 覆盖双向核对 ---
    css_classes = set(re.findall(r"\.([a-zA-Z][a-zA-Z0-9_-]*)", css))
    elem_classes = set()
    for c in re.findall(r"\bclass\s*=\s*\"([^\"]*)\"", tmpl):
        elem_classes |= set(c.split())
    orphan = elem_classes - css_classes
    if orphan:
        rep.warn(f"{rel}: 元素带 class 但无 CSS 规则 {sorted(orphan)}")
    unused = css_classes - elem_classes
    if unused:
        rep.warn(f"{rel}: CSS 规则存在但无元素使用 {sorted(unused)}（死规则）")

    # --- 12. var() 引用可解析 ---
    refs = set(re.findall(r"var\(\s*(--[a-zA-Z0-9_-]+)", css))
    defined = set(themes.get("light", {})) if themes else set()
    unresolved = {r for r in refs if r not in defined and r not in LIB_TOKENS}
    if unresolved:
        rep.fail(f"{rel}: var() 引用了未定义的 token {sorted(unresolved)}"
                 f"（themes.json 无、也不是库 token）")
    else:
        rep.ok(f"var() 引用 {len(refs)} 个，全部可解析")

    # --- 13. G05 双主题对比度（沿真实 DOM 取有效前景 × 最近祖先背景）---
    if themes:
        cls_rules = {}
        for sel, decl in re.findall(r"([^{}]+)\{([^{}]*)\}", css):
            props = {}
            for d in decl.split(";"):
                if ":" in d:
                    k, v = d.split(":", 1)
                    props[k.strip()] = v.strip()
            for c in sel.split(","):
                c = c.strip()
                if c.startswith("."):
                    cls_rules.setdefault(c[1:], {}).update(props)
        tree = parse_dom(tmpl)
        worst = {"light": (99.0, ""), "dark": (99.0, "")}

        def collect(node, anc_bg):
            own = token_of(node["_decls"].get("background", ""))
            bg = own or anc_bg
            if node["tag"] in TEXT_TAGS:
                fg = token_of(node["_decls"].get("color", ""))
                if fg and bg:
                    yield node, fg, bg
            for c in node["children"]:
                yield from collect(c, bg)

        # 给节点补合并后的声明
        def annotate(node):
            props = {}
            for c in node["cls"]:
                props.update(cls_rules.get(c, {}))
            node["_decls"] = props
            for c in node["children"]:
                annotate(c)
        for n in tree:
            annotate(n)

        pairs = set()
        for root in tree:
            for el, fg, bg in collect(root, None):
                for mode in ("light", "dark"):
                    tc = themes[mode].get(fg)
                    bc = themes[mode].get(bg)
                    if tc and bc:
                        r = contrast(tc, bc)
                        desc = f"{fg} on {bg}"
                        if r < worst[mode][0]:
                            worst[mode] = (r, desc)
                        pairs.add((mode, r, desc))
        bad = [(m, r, d) for m, r, d in sorted(pairs) if r < 4.5]
        for m, r, d in bad:
            rep.fail(f"{rel}: {m} 模式对比度 {r:.2f}:1 < 4.5:1（{d}）")
        if not bad:
            wl, dl = worst["light"]
            wd, dd = worst["dark"]
            rep.ok(f"G05 双主题对比度：真实配对全部 ≥4.5:1"
                   f"（light 最低 {wl:.2f}:1 {dl} / dark 最低 {wd:.2f}:1 {dd}）")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", default=".", help="项目根目录（默认当前目录）")
    args = ap.parse_args()
    root = Path(args.root).resolve()

    missing = [a for a in ASSETS if not (root / a).exists()]
    if missing:
        print("找不到资产文件：" + ", ".join(missing))
        return 2

    themes_path = root / "assets/themes.json"
    themes = {}
    if themes_path.exists():
        themes = json.loads(themes_path.read_text(encoding="utf-8"))

    print("QA 复核执行器 —— .uix 资产（静态审查，不编译不渲染）")
    print(f"项目根：{root}")
    rep = Report()
    for rel in ASSETS:
        check_file(rel, root, rep, themes)

    print()
    if rep.fails:
        print(f"FAIL —— {len(rep.fails)} 处阻塞 / {len(rep.warns)} 处提示")
        for f in rep.fails:
            print("  - " + f)
        return 1
    print(f"PASS —— 无阻塞项（{len(rep.warns)} 处提示需人工裁决，不阻塞）")
    for w in rep.warns:
        print("  ! " + w)
    return 0


if __name__ == "__main__":
    sys.exit(main())
