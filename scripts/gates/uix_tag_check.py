#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
`.uix` 资产静态门禁（core-ui 标签 / D-15 / SFC 骨架 / id-class 保全）。

依据：
  - docs/11-COREUI-MARKUP-TAGS.md §1 支持表（权威源 = 上游 core-ui
    src/ui/page/widget_factory.cpp → ConstructByTag）
  - docs/12-UIX-TAG-FIX.md §6 自检结论
  - 门禁 S13（token 命名空间）—— 口径由架构师给定，**正式门禁**是
    scripts/gates/s13_token_namespace_check.py；本脚本里的是**预检**，
    与正式门禁**有意重叠**（两个独立执行器，一个门禁失败不应掩盖另一个）。
    库 token 名单**不在本文件重复抄写**，改为从正式门禁 import（单一事实源）。

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

# --------------------------------------------------------------------------
# S13：库 token 名单从正式门禁 import，避免两份名单各自漂移
# --------------------------------------------------------------------------
_S13_MODULE = "s13_token_namespace_check"


def _load_lib_tokens() -> tuple[set[str], str | None]:
    """从 scripts/gates/s13_token_namespace_check.py 取库的 34 个 token。

    取不到时返回空集 + 说明（调用方据此 FAIL，而不是静默跳过 —— 静默跳过
    等于把一条契约门禁变成「建议」）。
    """
    here = Path(__file__).resolve().parent
    if str(here) not in sys.path:
        sys.path.insert(0, str(here))
    try:
        mod = __import__(_S13_MODULE)
    except Exception as exc:  # noqa: BLE001 —— 任何取不到都要报出来
        return set(), f"无法从 {_S13_MODULE}.py 取库 token 名单：{exc}"
    tokens = set(getattr(mod, "LIB_TOKENS", ()))
    if len(tokens) != getattr(mod, "LIB_TOKEN_COUNT", len(tokens)):
        return set(), f"{_S13_MODULE}.py 的 LIB_TOKENS 条数与 LIB_TOKEN_COUNT 不一致"
    return tokens, None

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
THEMES_FILE = "assets/themes.json"

# S13 口径 ③：accent 的回退源值（D14）—— 数据字段，不是 CSS 变量。
#   themes.json 里允许定义它；.uix 里既不允许定义，也不允许 var() 引用它。
ACCENT_FALLBACK = "--app-accent"

# `--name:` 形式是【定义】；`var(--name)` 是【引用】。口径 ④：引用不拦。
_DEF_RE = re.compile(r"(?<![\w-])(--[A-Za-z0-9_-]+)\s*:")
_REF_RE = re.compile(r"var\(\s*(--[A-Za-z0-9_-]+)")

# 修正前后必须一致的**模板体**元素数（docs/12 §0；不含 <window>/<style>/<template>
# 三个 SFC 外壳 —— 那是文件外壳，不是界面元素）
EXPECTED_ELEMENT_COUNT = {
    "assets/main.uix": 35,
    "assets/preview.uix": 9,
    "assets/about.uix": 16,
}
# 必须存在的 id（Rust 侧靠 ui_widget_find_by_id 取句柄）
EXPECTED_IDS = {
    "assets/main.uix": {
        "titlebar_host", "global_search", "tab_host", "tab_placeholder",
        "command_host", "nav_hint", "address_host", "breadcrumb", "address",
        "search", "infobar_host", "nav_host", "list_host", "preview_host",
        "preview_image", "preview_host_name", "preview_host_size",
        "preview_host_channel", "preview_host_state",
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


def _defs_and_refs(src: str) -> tuple[list[tuple[str, int]], list[tuple[str, int]]]:
    """返回 (定义列表, 引用列表)，每项是 (token 名, 行号)。行号相对传入文本。"""
    defs: list[tuple[str, int]] = []
    for m in _DEF_RE.finditer(src):
        if src[max(0, m.start() - 4):m.start()].endswith("var("):
            continue
        defs.append((m.group(1), src.count("\n", 0, m.start()) + 1))
    refs: list[tuple[str, int]] = [
        (m.group(1), src.count("\n", 0, m.start()) + 1) for m in _REF_RE.finditer(src)
    ]
    return defs, refs


def check_token_namespace(root: Path, lib_tokens: set[str]) -> list[str]:
    """门禁 S13 预检（口径由架构师给定，正式门禁见 s13_token_namespace_check.py）。

    ① 不得定义 core-ui 库的 34 个 token 任一；
    ② `.uix` 内不得定义 `--app-accent`；
    ③ `themes.json` 内**允许**定义 `--app-accent`（D14 的回退源值）；
    ④ `var(--accent)` 等引用**不拦**；
    另：`.uix` 内不得出现 `var(--app-accent)`。
    """
    fails: list[str] = []
    print("\n=== S13 预检：token 命名空间 ===")

    if not lib_tokens:
        fails.append("S13: 取不到 core-ui 库的 token 名单 —— 门禁不可用（不得静默跳过）")
        return fails

    for rel in ASSETS:
        path = root / rel
        raw = path.read_text(encoding="utf-8")
        src = strip_noise(raw)
        # 行号要算在**原文**上（剥注释后行数会变），故按原文逐行判定
        defs: list[tuple[str, int]] = []
        refs: list[tuple[str, int]] = []
        for lineno, line in enumerate(raw.splitlines(), 1):
            bare = re.sub(r"<!--.*?-->", "", line)
            bare = re.sub(r"/\*.*?\*/", "", bare)
            for m in _DEF_RE.finditer(bare):
                if bare[max(0, m.start() - 4):m.start()].endswith("var("):
                    continue
                defs.append((m.group(1), lineno))
            for m in _REF_RE.finditer(bare):
                refs.append((m.group(1), lineno))

        for name, lineno in defs:
            if name in lib_tokens:
                fails.append(f"{rel}:{lineno}: 定义了库的 token `{name}` → 撞名，"
                             f"会被 RebuildThemeVars 擦除重写（compiler.cpp:1647-1703）")
            if name == ACCENT_FALLBACK:
                fails.append(f"{rel}:{lineno}: 定义了 `{ACCENT_FALLBACK}` —— "
                             f"它是 Rust 的【回退源值（数据字段）】，不进 :root")
        for name, lineno in refs:
            if name == ACCENT_FALLBACK:
                fails.append(f"{rel}:{lineno}: 出现 `var({ACCENT_FALLBACK})` —— "
                             f".uix 里 accent 一律读库 token var(--accent)")
        # ④ 引用不拦（显式留痕，证明「不拦」是有意为之）
        lib_refs = sorted({n for n, _ in refs if n in lib_tokens})
        print(f"  {rel}: 定义 {len(defs)} 个 / 引用 {len(refs)} 处"
              + (f"（引用了库 token {lib_refs} —— 合法，不拦）" if lib_refs else ""))

    # themes.json：① 不撞库 token；③ 允许 --app-accent
    themes_path = root / THEMES_FILE
    if themes_path.exists():
        import json

        try:
            data = json.loads(themes_path.read_text(encoding="utf-8"))
        except Exception as exc:  # noqa: BLE001
            fails.append(f"{THEMES_FILE}: 不是合法 JSON —— {exc}")
            data = None
        if data is not None:
            def walk(node, path):
                if isinstance(node, dict):
                    for k, v in node.items():
                        if isinstance(k, str) and k.startswith("--"):
                            yield k, path + [k]
                        yield from walk(v, path + [str(k)])
                elif isinstance(node, list):
                    for i, v in enumerate(node):
                        yield from walk(v, path + [str(i)])

            for name, path_parts in walk(data, []):
                if name in lib_tokens:
                    fails.append(f"{THEMES_FILE}: 定义了库的 token `{name}`"
                                 f"（{'/'.join(path_parts)}）→ 撞名")
            print(f"  {THEMES_FILE}: 扫描完毕"
                  f"（`{ACCENT_FALLBACK}` 在此文件里**允许** —— D14 回退源值）")
    else:
        fails.append(f"找不到 {THEMES_FILE}")

    return fails


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

    # 3. id 全在（期望清单 = assets/ui-ids.json 的单一事实源；本脚本内
    #    另有一份硬编码副本，两者不一致时以 ui-ids.json 为准并报错）
    ids = set(re.findall(r'\bid\s*=\s*"([^"]+)"', body))
    expected = EXPECTED_IDS.get(rel, set())
    missing = expected - ids
    if missing:
        fails.append(f"{rel}: 丢失 id {sorted(missing)}")
    extra = ids - expected
    if extra:
        fails.append(f"{rel}: 出现清单外的 id {sorted(extra)} —— "
                     f"新增 id 必须同步 assets/ui-ids.json 与 docs/12 的句柄数")

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


def check_id_manifest(root: Path) -> list[str]:
    """本脚本内硬编码的期望 id 清单 vs assets/ui-ids.json（单一事实源）。

    两处不一致 = 一处会漏取句柄（Rust 侧 ui_widget_find_by_id 取不到 → 静默不刷新）。
    正式门禁是 scripts/gates/ui_id_manifest_check.py；这里是预检。
    """
    fails: list[str] = []
    manifest_path = root / "assets/ui-ids.json"
    if not manifest_path.exists():
        return fails  # 清单文件不存在时由正式门禁负责报，这里不重复失败
    import json

    try:
        data = json.loads(manifest_path.read_text(encoding="utf-8"))
    except Exception as exc:  # noqa: BLE001
        return [f"assets/ui-ids.json: 不是合法 JSON —— {exc}"]

    pages = data.get("pages", {})
    page_of = {"assets/main.uix": "main", "assets/preview.uix": "preview",
               "assets/about.uix": "about"}
    total = 0
    for rel, page in page_of.items():
        entry = pages.get(page)
        if entry is None:
            fails.append(f"assets/ui-ids.json: 缺 pages.{page}")
            continue
        manifest_ids = set(entry.get("ids", []))
        total += len(manifest_ids)
        mine = EXPECTED_IDS.get(rel, set())
        if manifest_ids != mine:
            only_mine = sorted(mine - manifest_ids)
            only_manifest = sorted(manifest_ids - mine)
            fails.append(f"assets/ui-ids.json 的 pages.{page} 与本脚本硬编码清单不一致"
                         f"（只在清单里：{only_manifest}；只在脚本里：{only_mine}）")
    declared = data.get("total")
    if isinstance(declared, int) and declared != total:
        fails.append(f"assets/ui-ids.json 的 total={declared} 与实际 id 总数 {total} 不符")
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

    # 门禁 S13 预检（正式门禁 = scripts/gates/s13_token_namespace_check.py）
    lib_tokens, lib_err = _load_lib_tokens()
    if lib_err:
        all_fails.append(f"S13 预检：{lib_err}")
    else:
        all_fails += check_token_namespace(root, lib_tokens)

    # id 清单一致性（正式门禁 = scripts/gates/ui_id_manifest_check.py）
    all_fails += check_id_manifest(root)

    print()
    if all_fails:
        print(f"FAIL —— {len(all_fails)} 处：")
        for f in all_fails:
            print("  -", f)
        return 1
    print("PASS —— 三个 .uix 资产：标签全部在 core-ui 支持表内，"
          "零遗留旧标签，D-15 合规，id 集合与模板体元素数保全，"
          "token 命名空间不撞库 token。")
    return 0


if __name__ == "__main__":
    sys.exit(main())
