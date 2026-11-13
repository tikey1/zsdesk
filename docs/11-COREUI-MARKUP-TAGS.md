# Core UI `.uix` 标记标签支持表 + 本 fork 资产修正清单

> **权威来源**：上游 `ghboke/core-ui` 的 `src/ui/page/widget_factory.cpp` → `ConstructByTag(tag, text, style, node)`。
> 这是**唯一**决定「某个标签能不能被识别」的地方（`docs/controls.md` / `docs/layout.md` 只是散文描述，未必穷尽）。
> 本文件为实测整理，**未在真机加载验证**（本机无编译器），标注见每节末尾。

---

## 1. 完整支持表

### 1.1 布局容器
| 标签 | 落地控件 |
|---|---|
| `div` `section` `article` `main` `header` `footer` `aside` `nav` `ul` `ol` | `VBoxWidget` 或 `HBoxWidget` |
| `li` | `HBoxWidget` |

**关键机制**：容器**不按标签名**决定横竖，而是看 CSS 的 `flex-direction`——
`row` / `row-reverse` → `HBoxWidget`；否则（默认 column）→ `VBoxWidget`。
→ **没有 `<hbox>` / `<vbox>` 标签，横排用 `<div>` + `flex-direction: row`。**

### 1.2 文本
| 标签 | 说明 |
|---|---|
| `span` `a` `label` `small` `strong` `em` | `LabelWidget`，默认 `wrap=true`，`display: inline`（不参与 stretch） |
| `p` | `LabelWidget`，`wrap=true` |
| `h1`~`h6` | `LabelWidget` + 固定字号（28 / 22 / 18 / 16 / 14 / 12）+ 粗体 |

**没有 `<text>` 标签。**

### 1.3 控件
| 标签 | 落地控件 |
|---|---|
| `button` | `ButtonWidget`（`type="primary"` 走 Fluent 2 accent） |
| `input` | 单行输入（`type="checkbox"` 走 CheckBox） |
| `textarea` | 多行输入 |
| `select` | 下拉 |
| `chips-input` / `ChipsInput` | 多值输入 |
| `Toggle` / `toggle` | 开关 |
| `ProgressBar` / `progress-bar` / `progressbar` | 进度条 |
| `Separator` / `separator` / `hr` | 分隔线 |
| `TitleBar` / `title-bar` | 无边框窗口标题栏（内置最小化/最大化/关闭） |
| `ScrollView` / `scroll-view` | 垂直滚动容器（**非虚拟化**） |
| `Expander` / `expander` | 折叠面板 |
| `tabs` / `TabControl` | 选项卡 |
| `custom` / `Custom` | **自定义绘制控件**（绘制 API 仅在 draw 回调内可用） |
| `img` | 静态图像 |
| `svg` | 内联 SVG |
| `gh_img_view` / `gh-img-view` / `gh_img` / `gh-img` | **Guohe 图片视图控件** |
| `ocr_img_view` / `ocr-img-view` / `OcrImgView` / `ocr-img` | 图片划词画布 |

### 1.4 特殊（会被忽略/透传）
`link` `meta` `head` `title` —— 文档头元素，不产生控件。

---

## 2. 本 fork 现有资产的标签不匹配（**必须修，否则加载不出界面**）

`assets/main.uix`（83 行）、`assets/preview.uix`（32 行）、`assets/about.uix`（32 行）实测用到的标签：

| 用到的标签 | 出现次数 | core-ui 是否识别 | 必须改成 |
|---|---|---|---|
| `<text>` | 10 | ✗ **不识别** | `<label>`（需换行）或 `<span>`（不换行） |
| `<hbox>` | 7 | ✗ **不识别** | `<div>` + CSS `flex-direction: row` |
| `<vbox>` | 1 | ✗ **不识别** | `<div>`（默认 column） |
| `<vseparator>` | 2 | ✗ **不识别** | 竖向分隔线需用 `<div>` + 固定宽度 + 背景色（`<Separator>` 是水平线） |
| `<text-input>` | 2 | ✗ **不识别** | `<input>` |
| `<panel>` | 4 | ✗ **不识别** | `<div>` |
| `<separator>` | 4 | ✓ 识别 | 保留 |
| `<label>` | 2 | ✓ 识别 | 保留 |
| `<window>` `<script>` `<style>` `<template>` | 各 1 | ✓ 识别 | 保留 |

### 2.1 修正规则（机械，不改设计意图）
1. `<hbox>` → `<div>`，并在 `<style>` 里给该类补 `flex-direction: row;`
   - 例：`<hbox id="titlebar_host" class="titlebar">` → `<div id="titlebar_host" class="titlebar">`，且 `.titlebar { flex-direction: row; ... }`
2. `<vbox>` → `<div>`（不需额外 CSS，默认 column）
3. `<text ... />` → `<label ... />`
   - 若原意是不换行的短标签（状态栏片段、标题），优先 `<span>`；但注意 `span` 默认 `display:inline` **不参与 stretch**，在 flex 容器里若要占满需显式 `display: block` 或 `align-self: stretch`。
4. `<text-input ... />` → `<input ... />`
5. `<panel ... />` → `<div ... />`
6. `<vseparator ... />` → `<div class="vsep" />`，样式给 `width: 1px; background: var(--divider); align-self: stretch;`
7. **所有 `id` 必须保留** —— Rust 侧靠 `ui_widget_find_by_id` 取句柄。
8. **所有 `class` 与 `<style>` 规则必须保留** —— 现有 `themes.json` 的 token 变量（`--bg` / `--content-bg` / `--accent` / `--selection-bg` / `--hover` / `--pressed` / 语义色等）就是通过这些 class 生效的。

### 2.2 保留 D-15 约束
用户 `assets/main.uix` 头注释已声明：**STATIC LAYOUT ONLY** —— 只声明布局 / 样式 / token，**无事件属性、无业务脚本**；列表行由 Rust 侧 `ui_custom_on_draw` 绘制；回调在 Rust 注册；首版只读。
→ 修正标签时**不得**新增 `onClick` / `v-model` / `:visible` 之类；`<script>` 里只保留 `const themeToken = "fluent2";` 这类常量。

### 2.3 已被现有架构验证为可行
- `<custom>` **受支持** → 「列表由 Rust 侧 `ui_custom_on_draw` 绘制」这条既定架构**成立**，无需改架构。
- `<gh_img>` **受支持** → 若将来要做进程内图片预览，core-ui 有现成的 Guohe 图片视图控件（D02 选了外部 exe，此项作为后备）。
- `<TitleBar>` **受支持** → 无边框窗口 + 系统标题栏按钮可行。

---

## 3. 其它必须知道的约束（来自实测）

| 项 | 结论 |
|---|---|
| `.uix` 分页 | 不是 `<page>` 标签，而是 `<script>` 里 `data()` 返回 `page: "home"` + `<template>` 内按 `page` 条件渲染 |
| `.uix` 的 script | Vue 3 SFC，`export default { data() { return {...} } }`；脚本由内嵌 QuickJS-NG 求值 |
| **`.uix` 不能直接调 C** | 上游 demo 注释原文：`methods{} can't call C directly`；宿主只能轮询 `.uix` 暴露的 state |
| i18n | `@key` 占位 + `.lang` 文件（UTF-8 `key=value`，`#`/`;` 注释，容忍 BOM/CRLF） |
| 滚动 | `ScrollView` **非虚拟化**；文件列表必须用 `<custom>` 自绘 + Rust 侧做虚拟化 |
| 窗口几何单位 | **DIP**（1/96 英寸），物理像素 = DIP × monitor_dpi ÷ 96 |
| 样式 | CSS 子集；未知属性被忽略；`display:inline/block/inline-block` 语义与浏览器一致 |
| 调试 | `ui_debug_*` 族 71 个（控件树导出、高亮、截图、事件模拟），可在没有真机窗口时定位问题 |

---

## 4. 未验证项（UNVERIFIED，需真机确认）

| # | 项 | 验证方法 |
|---|---|---|
| M01 | **未知标签的具体行为**（报错中止 / 静默忽略 / 产生空控件）—— 决定「标签写错」是白屏还是崩溃 | 造一个含未知标签的 `.uix`，`ui_page_load_string` 加载，看返回值与错误输出 |
| M02 | `flex-direction: row` 在嵌套容器中的继承与覆盖 | 加载修正后的 `main.uix`，截图核对横排是否正确 |
| M03 | `<label>` 与 `<span>` 在 flex 容器内的 stretch 行为差异 | 同上，截图核对状态栏片段是否被拉伸 |
| M04 | `var(--token)` 自定义 CSS 变量是否被 core-ui 的 CSS 子集支持 | 加载后核对背景/文字色是否生效（`themes.json` 有 light/dark 两套） |
| M05 | `<style>` 里 class 选择器与 `id` 选择器的支持度 | 同上 |
| M06 | `<Separator>` 是否支持竖向（`orientation` 属性） | 查头文件 / 真机验证；否则按 §2.1 规则 6 用 `<div>` 代替 |

---

## 5. 主题 token 冲突（**源码级实测，最容易踩的坑**）

依据：`src/ui/page/compiler.cpp` → `CollectVarsWithTheme(sheet)`；`src/ui/page/compiler.h:30-33`。

### 5.1 core-ui 库拥有的 token 全集（34 个）
```
--accent  --accent-hover  --accent-press  --accent-text  --accent-selected
--window-bg  --window-border
--bg  --bg-2  --bg-3  --bg-4  --bg-elevated
--fg  --fg-2  --fg-3  --fg-4  --fg-on-accent
--border  --border-subtle
--sidebar-bg  --sidebar-text  --sidebar-hover
--input-bg  --input-border  --input-border-hover  --input-border-focus
--card-bg  --card-border
--btn-bg  --btn-hover  --btn-press  --btn-text
--disabled-bg  --disabled-text
```

### 5.2 三个机制
1. **首次加载：用户 `:root` 同名值优先** —— 代码用 `has("--accent")` 守卫，库发现同名即跳过自己的默认。
   原文注释：`User-defined :root vars win ... so embedders can pin a token to a specific brand value if needed.`
2. **切深浅色时，用户钉住的同名 key 会被库擦除** ——
   `compiler.h` 原文：`User-pinned :root values for the same key are wiped — semantically the library owns these keys. Called from ui_theme_set_mode to flip Light/Dark without recompiling pages.`
   → **调用 `ui_theme_set_mode` 后，界面会突然变色。**
3. **深浅色不需要两套 `.uix`** —— `ui_theme_set_mode` → `RefreshAllPageThemes` → `RebuildThemeVars`，遍历所有 live page 重建变量；**同一套 CSS 源覆盖明暗两态**，不需要 `.dark` 前缀重复规则。

### 5.3 与本 fork `assets/themes.json` 的撞名
| 本 fork token | 撞库名？ | 后果 |
|---|---|---|
| `--bg` | **是**（库 `--bg` = 内容底色） | 首次用本 fork 值；`ui_theme_set_mode` 后被库接管 |
| `--accent` | **是**（库 `--accent` = 品牌色） | 同上 |
| `--sidebar-bg` | **是**（库同名 = 侧栏底色） | 同上 |
| `--content-bg` `--toolbar-bg` `--text` `--text-secondary` `--divider` `--hover` `--pressed` `--selection-bg` `--info-bg` `--warning-bg` `--danger-bg` `--info-fg` `--warning-fg` `--danger-fg` | 否 | 安全，不会被 wipe |

### 5.4 三条可选路线（需在实现侧给出选择）
| 路线 | 做法 | 代价 |
|---|---|---|
| (a) 改名避让 | 把 `--bg` / `--accent` / `--sidebar-bg` 改为 `--app-bg` / `--app-accent` / `--app-sidebar-bg` 等不撞名变量 | 需同步改 `<style>` 引用；深浅色切换时这些值**不变** → 需自己提供两套值的切换机制（当前**未核实** core-ui 是否提供运行时替换 CSS 变量的 C API） |
| (b) 改用库 token | `--bg` → `--window-bg` / `--bg-2`；`--accent` 直接用库 accent | 值随库主题变，可能偏离 `docs/07-UI-DESIGN-WINUI3.md` 的设计值；`themes.json` 的 light/dark 两套大部分要废弃 |
| (c) 混用 | 品牌色走库 accent / 布局底色走自定义避让名 | 需逐项判断，token 表最复杂 |

> **状态**：路线未拍板。实现侧（前端）需给出逐 token 方案表与「深浅色切换行为」说明；架构侧需在 FFI 契约里写明**是否需要 Rust 侧调用 `ui_theme_set_mode`**、以及**是否存在运行时替换 CSS 变量的 API**（后者当前**未核实**，见 `docs/05-COREUI-ABI-NOTES.md`）。

---

## 6. 结论

- 现有 `assets/*.uix` 的**设计意图与 token 体系可直接沿用**，只有 **6 个标签名需要机械替换**（`<text>` `<hbox>` `<vbox>` `<vseparator>` `<text-input>` `<panel>`），不需要重做设计。
- `<custom>` 受支持 → 既定的「Rust 侧绘制列表」架构成立。
- 修正后仍需真机确认 M01~M06；其中 **M04（CSS 变量）已由源码级核实为「支持」**（`cascade.cpp:42-54` 匹配 `var(`，`value.h:91` 提供 `Resolve a var(--name) reference`），**剩余风险从「变量不支持」转为「同名 token 被库接管」**（见 §5）。
