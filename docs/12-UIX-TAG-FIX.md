# `.uix` 标签修正对照表 + 主题 token 最终方案

> 任务：把 `assets/main.uix` / `preview.uix` / `about.uix` 修正成 core-ui 真正认的标签，并核对渲染结构。
> 依据：`docs/11-COREUI-MARKUP-TAGS.md`（标签支持表 + 修正规则 + M01~M06）。
> 权威源：上游 `ghboke/core-ui` 源码（本机只读副本 `/tmp/upstream/core-ui-main`）。
> 日期：2026-11-13　　执行：前端（贾思敏）　　**本文不含审查结论（审查由 QA 另行执行）**。

---

## 0. 摘要

| 项 | 结果 |
|---|---|
| 修正文件 | `assets/main.uix`、`assets/preview.uix`、`assets/about.uix` |
| 标签替换 | `<text>`×25、`<hbox>`×8、`<vbox>`×3、`<text-input>`×3、`<panel>`×5、`<vseparator>`×2（共 46 处） |
| 元素总数 | **不变**（main 33→33、preview 12→12、about 19→19）——只换标签，不增删元素 |
| id 保留 | main 21 / preview 6 / about 0 —— **一个没少，一个没改**（Rust 侧 `ui_widget_find_by_id` 不受影响） |
| class 保留 | 原有 class 全部保留；**新增 3 个**（`.root`、`.tab-placeholder`、`.nav-hint`）+ 补 `class="divider"` ×3（preview/about 的分隔线） |
| 结构级修正 | 5 处（见 §2）——不改设计，只让 core-ui 真正认得 |
| 主题 token | 17 个 token 全部加 `--app-` 前缀（见 §4） |
| 未验证 | **本机无 Rust/C++ 工具链，全部结论未经编译或渲染验证**；凡推断均标 `[推断]` |

> ⚠️ **docs/11 §2 的标签统计有误**：它写 `<text>`×10、`<hbox>`×7、`<vbox>`×1、`<text-input>`×2、`<panel>`×4。
> 用 `git show HEAD:assets/*.uix | grep -o '<[a-z-]*' | sort | uniq -c` 实测应为
> `<text>`×**25**、`<hbox>`×**8**、`<vbox>`×**3**、`<text-input>`×**3**、`<panel>`×**5**、`<vseparator>`×2。
> 结论不变（标签全部要换），但工作量被低估了一倍多。下文按实测值。

---

## 1. 查证方法（全部基于上游源码行号，可复核）

| 结论 | 依据（上游文件:行） |
|---|---|
| 标签是否被识别，只看 `ConstructByTag` | `src/ui/page/widget_factory.cpp:913-1194` |
| 容器横竖由 CSS `flex-direction` 决定（无 hbox/vbox） | `widget_factory.cpp:918-927`（`row`/`row-reverse`→HBox，否则 VBox） |
| 未知标签的兜底行为 | `widget_factory.cpp:1192-1193` → 落到 `VBoxWidget`（**静默变成空盒子，不报错**） |
| `span`/`label`/`a`/`small`/`strong`/`em` 同分支 | `widget_factory.cpp:942-954`（LabelWidget，wrap=true，**display=Inline**） |
| `p` / `h1`~`h6` | `widget_factory.cpp:955-972`（LabelWidget，**不设 display → 默认 Block**） |
| `<Separator>` 只有水平形态 | `widget_factory.cpp:1059-1060` + `src/ui/controls.h:198-206`（`SeparatorWidget(bool vertical=false)`，工厂从不传 true） |
| `text="..."` 属性在 `.uix` 无效 | `widget_factory.cpp:54-61`（`FirstPlainText` 只读**子文本节点**）+ `src/ui/uix/sfc_parser.cpp` / `template_parser.cpp` 内**无 `"text"` 属性处理**（`<menu text=...>` 是唯一例外，`compiler.cpp:641`） |
| `<window>` 必须自闭合 | `src/ui/uix/sfc_parser.cpp:285-297`（非自闭合 → pushErr + skipUntilClose）→ `:369` `if (!doc.errors.empty()) doc.ok = false;` → `src/ui/page/page_api.cpp:249-255` 整页 load 失败 |
| `<script>` 必须 `export default` | `src/ui/uix/script_runtime.cpp:190-192`（缺 default → `r.ok=false`）→ `src/ui/page/page_state.cpp:1948-1952`（`AttachQuickJS` **提前 return**，JS 运行时不挂载） |
| `window { }` 选择器不匹配任何控件 | `<window>` 是 SFC 外壳（`sfc_parser.cpp:285`），模板根节点才是根控件；级联只对模板节点匹配（`src/ui/css/cascade.cpp:112-145`） |
| **无 CSS 属性继承** | `src/ui/css/cascade.cpp:92-110` 定义了 `IsInheritedProperty`，但**全仓无任何调用点**；`ComputeStyle` 不接受父级样式 |
| `display` / `align-self` / `white-space` / `flex-shrink` 受支持 | `widget_factory.cpp:445-451`（display）、`:738-746`（align-self）、`:700-711`（white-space）、`:903-908`（flex-shrink） |
| `border-bottom` **不受支持** | `widget_factory.cpp:631-653` 只认 `border` / `border-color` / `border-width` / `border-radius` |
| `font-family` 对普通 LabelWidget **不生效** | `src/ui/renderer.cpp:3759 / 4907 / 5182` 均在 SVG 文本路径；普通控件文字用 `theme::kFontFamily`（`src/ui/theme.h:498` = L"Segoe UI"）或 `ui_theme_set_default_font` 设的值 |
| CSS 变量 `var()` 支持，自定义属性**必须在 `:root`** | `src/ui/css/cascade.cpp:42-54`（匹配 `var(`）、`src/ui/css/value.h:91`、`cascade.h:45`、`:134`（非 `:root` 级自定义属性被忽略） |
| 库自带 34 个 token，`ui_theme_set_mode` 会覆盖同名用户值 | `src/ui/page/compiler.cpp:1647-1660`（`kThemeKeys[]` 34 项）+ `:1672-1700`（`RebuildThemeVars` 先 erase 再重写） |

---

## 2. 结构级修正（超出 docs/11 §2.1 的 6 条规则，但**不改设计**）

docs/11 §2.1 的 6 条规则只解决「标签不被识别」。实读上游后发现 **5 处结构级问题**，不修则改完标签仍然出不来界面。全部只改写法，不改尺寸 / 间距 / 配色 / 层级。

### 2.1 `<window>` 必须自闭合，`<script>/<style>/<template>` 提到顶层

| | 原写法 | 现写法 |
|---|---|---|
| main | `<window title="tabbed_files" width="1200" height="800">` … `</window>` | `<window title="tabbed_files" width="1200" height="800" />` + 顶层 `<script>`/`<style>`/`<template>` |

**依据**：`sfc_parser.cpp:294`「`<window>` must be self-closing (use `<window .../>`)」→ `pushErr` → `skipUntilClose` 会把 `</window>` 之前的内容**整段跳过** → `doc.ok=false` → `page_api.cpp:251-255` 返回 "SFC parse failed"。
**影响**：原写法下三个 `.uix` **整页加载失败**（不是局部缺失，是全白/无窗口）。这比标签不识别更致命，必须先修。
**取法**：参照上游 demo `demo/ui_demo.uix:1-4` 的写法。`<window>` 的属性（title/width/height）原样保留。

### 2.2 文本从 `text="..."` 属性改为元素内部文本

原：`<text class="app-title" text="tabbed_files" />`
现：`<span class="app-title">tabbed_files</span>`

**依据**：`widget_factory.cpp:54-61` `FirstPlainText()` 只遍历 `node.children`，`text` 属性不参与；`sfc_parser.cpp` / `template_parser.cpp` 内无 `"text"` 属性处理。
**影响**：不改的话**所有文字都是空的**（控件建出来但没有内容）。
**注意**：`.uix` 的文本节点会做空白折叠但**保留首尾空白**（`template_parser.cpp:262-266`），所以文本一律与标签同行书写，避免引入前后空格。

### 2.3 `window { ... }` 选择器落到根节点 `.root`

原：`<style> window { background: var(--bg); color: var(--text); font-family: …; } </style>`
现：`<style> .root { background: var(--app-bg); color: var(--app-text); font-family: …; } </style>` + `<div class="root">` 作为模板根节点。

**依据**：`<window>` 是 SFC 外壳，不产生控件；级联只对模板节点做匹配（`cascade.cpp:112-145`）。
**影响**：不改的话窗口底色 / 文字色 / 字体**一条都不生效**（落到库默认色）。
**副作用（已如实标注）**：core-ui **无 CSS 属性继承**，所以 `.root` 上的 `color` 不会传给子控件 → 见 §2.4。

### 2.4 无 CSS 继承 → 每个有文字的 class 各自声明 `color`

**依据**：`cascade.cpp:92-110` 定义了 `IsInheritedProperty`（含 `color` / `font-family` / `font-size`），但**全仓无调用点**；`ComputeStyle` 也不接受父级样式。

**处理**：给每个承载文字且原本没有 `color` 规则的 class 补一条：

| 文件 | class | 补的声明 | 为什么 |
|---|---|---|---|
| main | `.tab-placeholder`（新 class，给 `#tab_placeholder`） | `color: var(--app-text); white-space: nowrap;` | 标签栏占位文字，原本无 class 无色 |
| main | `.nav-hint`（新 class，给 `#nav_hint`） | `color: var(--app-text-secondary); white-space: nowrap;` | 命令栏提示，原本无 class 无色 |
| preview | `.preview-title` | 追加 `color: var(--app-text); white-space: nowrap;` | 工具条标题原本无色 |
| about | `.heading` | 追加 `color: var(--app-text);` | 标题原本无色 |

其余有文字的元素（`.app-title` / `.status-segment` / `.preview-hint` / `.preview-state` / `.row`）原本就有 `color` 规则，只做 token 改名。

> 库默认文字色是 `contentText = #424242`（light，`theme.h` grey::g26），与项目的 `--app-text`（#1B1B1B）差一档多 —— 不补色会明显偏灰。

### 2.5 行容器补 `align-items: center`

`.titlebar`(32) / `.tabbar`(36) / `.commandbar`(40) / `.addressbar`(36) / `.statusbar`(28) / `.preview-toolbar`(32) 补 `align-items: center`。
**依据**：容器默认 `align-items: stretch`（`widget_factory.cpp:860-871`），但 `span`/`label` 是 `display:inline`、`input` 是 `display:inline-block`（`widget_factory.cpp:1344-1360`），**都不参与 stretch** → 会贴在条的上沿，下沿空出一截。docs/07 §2 规定这些条是 32/36/40/28px 固定高，控件应居中。
**影响**：不改的话标题栏文字/输入框/状态片段全部顶在上边。`.main-area` **不加**（三栏面板需要 stretch 撑满）。

### 2.6 preview / about 的分隔线补 `class="divider"`

原：`<separator />`（库默认分隔线色 = `#D1D1D1`，`theme.h` grey::g82）
现：`<separator class="divider" />` + `.divider { background: var(--app-divider); }`（`#E0E0E0`）

**依据**：docs/07 §2「分隔线 1px `--divider`」。库默认色与主窗口的 `--app-divider` 差 15 级灰度，预览窗是嵌在主窗口里的，两条线颜色不一致会被看出来。
**这是本轮唯一一处「原文件没写、但按设计文档补上」的地方**，在此显式声明，不是静默改动。

---

## 3. 逐标签对照表

> 「原:行」= `git show HEAD:assets/<file>` 的行号；「现:行」= 修正后文件的行号。
> **影响面**统一按：core-ui 侧 = 控件类型/布局行为；Rust 侧 = 是否影响 `ui_widget_find_by_id` / 绘制回调。

### 3.1 `assets/main.uix`

| # | 原标签 | 原:行 | 现标签 | 现:行 | 取舍理由 | 影响面 |
|---|---|---|---|---|---|---|
| 1 | `<vbox>` | 45 | `<div class="root">` | 116 | VBox 无对应标签；`<div>` 默认 column = 原 `<vbox>` 语义 | core-ui：同为 VBox。Rust：根节点 id 无（原本也无） |
| 2 | `<hbox id="titlebar_host">` | 46 | `<div id="titlebar_host" class="titlebar">` | 117 | 横排靠 CSS：`.titlebar` 补 `flex-direction: row` | core-ui：VBox→HBox。Rust：id 不变，句柄照常取 |
| 3 | `<text class="app-title">` | 47 | `<span class="app-title">tabbed_files</span>` | 118 | 标题栏应用名，短、单行、不占满 → `<span>`（inline 不 stretch）；加 `white-space: nowrap` | core-ui：LabelWidget(inline)。Rust：原本无 id，不影响 |
| 4 | `<text-input id="global_search">` | 48 | `<input id="global_search">` | 119 | 标签无对应控件；`<input>` → TextInputWidget | core-ui：VBox(空盒)→TextInputWidget。Rust：id 不变 |
| 5 | `<hbox id="tab_host">` | 50 | `<div id="tab_host" class="tabbar">` | 121 | 同 #2 | 同上 |
| 6 | `<label id="tab_placeholder">` | 51 | `<label id="tab_placeholder" class="tab-placeholder">…</label>` | 122 | **原就是 `<label>`，保留**；只补 class 拿颜色 + 内部文本 | core-ui：不变。Rust：id 不变 |
| 7 | `<separator class="divider">` | 53 | 不变 | 124 | 已是受支持标签 | 无 |
| 8 | `<hbox id="command_host">` | 54 | `<div id="command_host" class="commandbar">` | 125 | 同 #2 | 同上 |
| 9 | `<label id="nav_hint">` | 55 | `<label id="nav_hint" class="nav-hint">…</label>` | 126 | **原就是 `<label>`，保留**；补 class 拿颜色 | 同上 |
| 10 | `<separator class="divider">` | 57 | 不变 | 128 | 已是受支持标签 | 无 |
| 11 | `<hbox id="address_host">` | 58 | `<div id="address_host" class="addressbar">` | 129 | 同 #2 | 同上 |
| 12 | `<panel id="breadcrumb">` | 59 | `<div id="breadcrumb" class="breadcrumb" />` | 130 | `<panel>` 无对应控件 → `<div>`（column） | core-ui：VBox→VBox。Rust：id 不变 |
| 13 | `<text-input id="address">` | 60 | `<input id="address">` | 131 | 同 #4 | 同上 |
| 14 | `<text-input id="search">` | 61 | `<input id="search">` | 132 | 同 #4 | 同上 |
| 15 | `<separator class="divider">` | 63 | 不变 | 134 | 已是受支持标签 | 无 |
| 16 | `<hbox id="infobar_host">` | 64 | `<div id="infobar_host" class="infobar" />` | 135 | 同 #2 | 同上 |
| 17 | `<hbox class="main-area">` | 65 | `<div class="main-area">` | 136 | 同 #2 | 同上 |
| 18 | `<panel id="nav_host">` | 66 | `<div id="nav_host" class="nav-host" />` | 137 | 同 #12 | 同上 |
| 19 | `<vseparator class="divider">` | 67 | `<div class="vsep" />` | 138 | `<Separator>` 只有水平形态（`controls.h:200`），竖线用 1px 宽 div：`width:1px; background:var(--app-divider); align-self:stretch; flex-shrink:0` | core-ui：VBox(空盒)→1px 竖条。Rust：无 id，无影响 |
| 20 | `<panel id="list_host">` | 68 | `<div id="list_host" class="list-host" />` | 139 | 同 #12 | 同上；见 §5「Rust 自绘挂载点」 |
| 21 | `<vseparator class="divider">` | 69 | `<div class="vsep" />` | 140 | 同 #19 | 同上 |
| 22 | `<panel id="preview_host">` | 70 | `<div id="preview_host" class="preview-host" />` | 141 | 同 #12 | 同上 |
| 23 | `<separator class="divider">` | 72 | 不变 | 143 | 已是受支持标签 | 无 |
| 24 | `<hbox id="status_host">` | 73 | `<div id="status_host" class="statusbar">` | 144 | 同 #2 | 同上 |
| 25-30 | `<text id="status_*" class="status-segment">` ×6 | 74-79 | `<span id="status_*" class="status-segment" />` ×6 | 145-150 | 状态栏片段，短、单行、由 Rust 运行时填字 → `<span>`；配 `white-space: nowrap` 防长值折行 | core-ui：VBox(空盒)→LabelWidget(inline)。Rust：id 不变，`ui_label_set_text` 可用 |

### 3.2 `assets/preview.uix`

| # | 原标签 | 原:行 | 现标签 | 现:行 | 取舍理由 | 影响面 |
|---|---|---|---|---|---|---|
| 1 | `<vbox>` | 20 | `<div class="root">` | 58 | 同 main #1 | 同上 |
| 2 | `<hbox class="preview-toolbar">` | 21 | `<div class="preview-toolbar">` | 59 | 横排靠 `.preview-toolbar { flex-direction: row; align-items: center; }` | core-ui：VBox→HBox |
| 3 | `<text id="preview_title">` | 22 | `<span id="preview_title" class="preview-title">预览</span>` | 60 | 工具条标题，短、单行 → `<span>` | core-ui：VBox→LabelWidget(inline)。Rust：id 不变 |
| 4 | `<separator />` | 24 | `<separator class="divider" />` | 62 | 已是受支持标签；补 class 走项目 token（见 §2.6） | core-ui：分隔线颜色由库默认改为项目 token |
| 5 | `<panel id="preview_body">` | 25 | `<div id="preview_body" class="preview-body" />` | 63 | `<panel>` → `<div>` | core-ui：VBox→VBox。Rust：id 不变 |
| 6 | `<text id="preview_hint">` | 26 | `<label id="preview_hint" class="preview-hint">…</label>` | 64 | 提示行需要**占满宽度并可换行**（Rust 写入的通道名可能较长）→ `<label>` + `display: block` | core-ui：VBox(空盒)→LabelWidget(block/stretch)。Rust：id 不变 |
| 7 | `<text id="preview_channel">` | 27 | `<label id="preview_channel" class="preview-hint" />` | 65 | 同 #6 | 同上 |
| 8 | `<text id="preview_size">` | 28 | `<label id="preview_size" class="preview-hint" />` | 66 | 同 #6 | 同上 |
| 9 | `<text id="preview_state">` | 29 | `<label id="preview_state" class="preview-state" />` | 67 | 状态行需占满宽度 → `<label>` + `display: block` | 同上 |

### 3.3 `assets/about.uix`

| # | 原标签 | 原:行 | 现标签 | 现:行 | 取舍理由 | 影响面 |
|---|---|---|---|---|---|---|
| 1 | `<vbox>` | 14 | `<div class="root">` | 43 | 同 main #1 | core-ui：VBox→VBox |
| 2 | `<text class="heading">` | 15 | `<label class="heading">…</label>` | 44 | 标题虽短，但按「占满宽度」语义统一用 `<label>` + `display: block` | core-ui：VBox(空盒)→LabelWidget(block) |
| 3 | `<text class="row">`（首条） | 16 | `<label class="row">…</label>` | 45 | 条目是长句（最长达 40+ 汉字），**必须占满宽度才会折行**；若保持 inline，宽度=自然文字宽度 → 横向溢出窗口而不换行 | 同上 |
| 4 | `<separator />` | 17 | `<separator class="divider" />` | 46 | 已是受支持标签；补 class 走项目 token（见 §2.6） | core-ui：分隔线颜色改为项目 token |
| 5 | `<text class="row">` ×10 | 18-27 | `<label class="row">…</label>` ×10 | 47-56 | 同 #3 | 同上 |
| 6 | `<separator />` | 28 | `<separator class="divider" />` | 57 | 同 #4 | 同上 |
| 7 | `<text class="row">`（致谢） | 29 | `<label class="row">…</label>` | 58 | 同 #3 | 同上 |

### 3.4 `<text>` → `<span>` / `<label>` 的逐个取舍原则（25 处）

**先说一个关键事实（否则会误以为两者有行为差）**：
上游 `widget_factory.cpp:942-954` 把 `span` / `a` / `label` / `small` / `strong` / `em` **放在同一个分支**，产出的都是 `LabelWidget`，`wrap=true`、`display=Inline`。
→ **在 core-ui 里 `<span>` 与 `<label>` 的渲染行为完全一致**，选哪个只是语义与可读性问题。
→ 真正决定「换行 / 占满 / 不拉伸」的是 CSS：`display: block`（占满并参与 stretch）、`white-space: nowrap`（单行 + 溢出省略号）、`align-self: stretch`。
→ 所以本次一律**按布局意图选标签 + 显式写 display / white-space**，不依赖标签默认值。

| 意图 | 标签 | CSS 配合 | 本轮用到的元素 |
|---|---|---|---|
| 短、单行、不占满容器宽度的「片段」 | `<span>` | `white-space: nowrap` | main `.app-title`、main 6 个 `.status-segment`、preview `.preview-title` |
| 需要占满宽度、可折行的「段落 / 状态行 / 长条目」 | `<label>` | `display: block` | preview 4 个提示/状态行、about 13 个标题与条目 |
| 原本就是 `<label>` 且语义上就是一段说明 | `<label>`（保留） | 补 `color` / `white-space` | main `#tab_placeholder`、`#nav_hint` |

> 为什么状态栏片段用 `<span>` 而不是 `<label>`：状态栏是横向排布的一串片段，语义上是「内联片段」而非「段落」；配合 `white-space: nowrap` 后，长计数（如「12,345 项 · 3 已选」）不会折行把 28px 的状态栏撑成两行。

---

## 4. 主题 token：冲突与最终方案

### 4.1 冲突事实

1. core-ui 库自带 **34 个** CSS 变量 token（`compiler.cpp:1647-1660` 的 `kThemeKeys[]`）：
   `--accent --accent-hover --accent-press --accent-text --accent-selected --window-bg --window-border --bg --bg-2 --bg-3 --bg-4 --bg-elevated --fg --fg-2 --fg-3 --fg-4 --fg-on-accent --border --border-subtle --sidebar-bg --sidebar-text --sidebar-hover --input-bg --input-border --input-border-hover --input-border-focus --card-bg --card-border --btn-bg --btn-hover --btn-press --btn-text --disabled-bg --disabled-text`
2. 用户在 `:root` 里定义同名值时，**首次加载**用户值优先（`compiler.cpp:1579-1600` 的 `if (!has("--x"))`）。
3. 但 `ui_theme_set_mode()` 切深浅色时，`RebuildThemeVars`（`compiler.cpp:1672-1700`）会 **先 erase 掉这 34 个键、再用库当前 palette 重写** → **用户同名值被清空**。
4. 更隐蔽的是**语义撞车**：库的 `--bg` = `contentBg`（内容区底色，light = `#FFFFFF`），而本项目的 `--bg` = **窗口底色**（`#F3F3F3`）。即便侥幸没被清空，两者含义也相反。

### 4.2 运行时替换 CSS 变量的 C API —— **无**

按要求在 `include/ui_core.h`（2137 行 / 557 个 `ui_*` 声明）内查证：

| 查证项 | 结果 |
|---|---|
| 运行时设置 / 替换 CSS 变量的 C API | **无**。全头文件内不存在 `ui_*css_var*` / `*_set_var*` / `*_set_style*` / `*_stylesheet*` 一类符号 |
| 可用的主题相关 C API（只有这些） | `ui_theme_set_mode(UiThemeMode)` / `ui_theme_get_mode()` / `ui_theme_set_accent(UiColor)` / `ui_theme_set_accent_hex(const char*)` / `ui_theme_set_default_font(const wchar_t*)` / `ui_theme_set_cjk_font(const wchar_t*, const wchar_t*)` / `ui_theme_set_text_locale` / `ui_theme_set_text_render_mode`，以及只读 getter `ui_theme_bg/content_bg/sidebar_bg/toolbar_bg/accent/text/divider` |
| `:root` 是不是唯一的自定义属性入口 | 是。`cascade.h:45` 只从 `:root` 采集；`cascade.cpp:134` 明确跳过非 `:root` 级的自定义属性 |

**结论**：项目 token 只能靠 **Rust 在 `ui_page_load_string()` 之前把一段 `:root { … }` 拼进 `.uix` 源码字符串** 生效。

### 4.3 逐 token 最终表（17 个）

**决策口径**：**改名避让**（加 `--app-` 前缀）16 个 + **改用库 token** 1 个（accent）。**保留原名 0 个。**

| # | 原名 | 新名 | 决策 | 理由 |
|---|---|---|---|---|
| 1 | `--bg` | `--app-bg` | 改名避让 | 与库 `--bg` 同名，且**语义相反**（库=内容区底色，本项目=窗口底色） |
| 2 | `--content-bg` | `--app-content-bg` | 改名避让 | 统一到 `--app-*` 命名空间，避免"一半避让一半没避让"导致后续再踩 |
| 3 | `--sidebar-bg` | `--app-sidebar-bg` | 改名避让 | 与库 `--sidebar-bg` 同名 |
| 4 | `--toolbar-bg` | `--app-toolbar-bg` | 改名避让 | 统一命名空间 |
| 5 | `--text` | `--app-text` | 改名避让 | 库用 `--fg`，本项目用 `--text`；统一命名空间以免两套前景色 token 并存 |
| 6 | `--text-secondary` | `--app-text-secondary` | 改名避让 | 同上 |
| 7 | `--divider` | `--app-divider` | 改名避让 | 库用 `--border`；统一命名空间 |
| 8 | `--accent` | `--app-accent` | **改用库 token**（CSS 侧读 `var(--accent)`，不自定义） | accent 是**唯一有官方通道**的 token：`ui_theme_set_accent_hex()` 会写进库 palette 并同步到库 token `--accent`（`ui_api.cpp:2571-2587` → `theme::SetAccent` → `RefreshAllPageThemes` → `RebuildThemeVars`），且**跨 `ui_theme_set_mode` 保留**（`theme.h:435-439`）。若自定义 `--app-accent`，库自绘控件（primary 按钮 / 焦点环 / 滑块 / 进度条）仍是库默认蓝 `#3971EC`，与自绘区的 `#2563EB` 撞成两种蓝 |
| 9 | `--hover` | `--app-hover` | 改名避让 | 库有 `--btn-hover`，语义不同；统一命名空间 |
| 10 | `--pressed` | `--app-pressed` | 改名避让 | 库有 `--btn-press`；同上 |
| 11 | `--selection-bg` | `--app-selection-bg` | 改名避让 | 库有 `--accent-selected`，语义不同；同上 |
| 12 | `--info-bg` | `--app-info-bg` | 改名避让 | 库无语义色 token，必须自定义 |
| 13 | `--warning-bg` | `--app-warning-bg` | 改名避让 | 同上 |
| 14 | `--danger-bg` | `--app-danger-bg` | 改名避让 | 同上 |
| 15 | `--info-fg` | `--app-info-fg` | 改名避让 | 同上 |
| 16 | `--warning-fg` | `--app-warning-fg` | 改名避让 | 同上 |
| 17 | `--danger-fg` | `--app-danger-fg` | 改名避让 | 同上 |

**统计**：保留原名 **0** / 改名避让 **16** / 改用库 token **1**。
另：新增 `--app-*` 之外**没有**引入圆角 / 间距 token —— docs/07 §4 提到要补「圆角 / 间距」，但 core-ui 的圆角是 `border-radius: 4px` 这类**直接值**（`widget_factory.cpp:652-654`），不走 CSS 变量；本轮 `.uix` 未用到，留待后续。

### 4.4 深浅色切换行为（Rust 侧必须按此顺序）

| 阶段 | 库自绘控件（按钮 / 输入框 / 焦点环 / 勾选框…） | 本项目自绘区域（条 / 列表 / 面板 / 分隔线 / 文字） | accent |
|---|---|---|---|
| **加载时** | `ui_theme_set_mode(Light\|Dark)` 决定 | `:root` 里注入的 `--app-*` 决定 | `ui_theme_set_accent_hex(该模式的 --app-accent)` |
| **切换时（库）** | `ui_theme_set_mode()` → `RebuildThemeVars` 重写 34 个库 token → 库控件立即跟随 | **不动**（`--app-*` 不在 `kThemeKeys` 里，不会被 erase） | accent override 跨模式保留（`theme.h:435-439`） |
| **切换时（项目）** | — | ⚠️ **不会自动跟随**：`:root` 的值是静态的 | — |
| **项目 token 要跟随怎么办** | — | **必须重建页面**：Rust 用另一套 `--app-*` 重新拼 `:root` 并 `ui_page_load_string()` 新页（`ui_page_reload()` 会重放**旧源码**，改不了 palette） | — |

**推荐调用顺序（Rust）**：

```
1. ui_theme_set_default_font(L"Segoe UI Variable")      // Windows 11
   ui_theme_set_cjk_font(L"Segoe UI", L"Microsoft YaHei UI")   // 中文回退
2. ui_theme_set_mode(UiThemeMode_Light /* 或 Dark */)
3. ui_theme_set_accent_hex("#2563EB")                   // = 该模式 themes.json 的 --app-accent
4. 拼 ":root { --app-bg: …; … }" 注入 .uix 源码 → ui_page_load_string(...)
```

> 第 2 步必须在第 3 步**之前或紧邻**：切 mode 会用当前 accent override 重新派生（`theme.h:435-439`），若顺序反了会拿到上一次的 accent。
> 第 3 步不能省：省了的话 `var(--accent)` 落到库默认 `#3971EC`，与设计稿的 `#2563EB` 不符（表现为「自绘区蓝、按钮蓝」不一致）。

### 4.5 `assets/themes.json` 的改动

- `schema` 字段与 `note` 字段**原样保留**。
- 新增 `revision` 段，记录日期 / 改动原因 / 决策 / 注入方式 / 切换方式 / 详情指向本文档。
- `light` / `dark` 的键全部由原名改为 `--app-*`，**色值一个没动**（`--app-accent` 仍是 `#2563EB` / `#60A5FA`）。
- 新增 `--app-accent`（原名 `--accent`）作为 accent 的唯一源值。

---

## 5. 对 Rust 侧的影响与待办（本轮不改 `crates/**`，仅登记）

| # | 事项 | 依据 | 建议落点 |
|---|---|---|---|
| R1 | **Rust 必须在 `ui_page_load_string` 前注入 `:root` 块**（读 `themes.json` → 拼 CSS → 拼进 `.uix` 源码） | §4.2：无运行时变量 API | `crates/zsdesk-ui/` 的页面装配处 |
| R2 | **`ui_theme_set_accent_hex` 必须调用**，否则 accent 与设计稿不符 | §4.3 #8 | 同上，主题初始化处 |
| R3 | **`ui_theme_set_default_font` / `ui_theme_set_cjk_font` 必须调用** | `font-family` 对普通 LabelWidget 不生效（§1） | 同上 |
| R4 | **深浅色切换需重建页面**（不能只调 `ui_theme_set_mode`） | §4.4 | 主题切换处 |
| R5 | **列表/导航/面包屑自绘的挂载方式**：`ui_custom_on_draw` 只对 CustomWidget 生效（`ui_api.cpp:2612-2616` 的 `As<ui::CustomWidget>` 取不到就**静默 no-op**），`<div>` 宿主拿不到绘制回调 | `ui_custom(void)`（`ui_core.h:1415`）+ `ui_widget_add_child(parent, child)`（`ui_core.h:1083`）可运行时创建并挂载 | 虚拟列表 / 导航树 |
| R6 | **`.tab-active` 的 2px accent 指示条**：core-ui 不支持 `border-bottom` | `widget_factory.cpp:631-653` | 由 Rust 自绘，或在 `.uix` 里加 2px 高的 `<div>` |
| R7 | **`.infobar` 默认 `height: 0px`**（收起），`.infobar-visible` 36px；两者都无 id 切换机制 → Rust 侧需 `ui_widget_set_visible`（`<div>` 支持 `visible` 静态属性，`widget_factory.cpp:1329-1332`）或直接改高度 | — | 信息条状态机 |
| R8 | **`.preview-title` / `.heading` 的 `font-weight: 600` 是空声明**：core-ui 对 LabelWidget 只认 `bold` / `700`（`widget_factory.cpp:1392-1395`：`if (fw == "bold" \|\| fw == "700") lbl->Bold();`），`600` 落在 else，**什么都不做** → 这两个标题会按常规字重渲染，层级丢失。<br>**但本轮不擅自改**：docs/07 §4 明写「字重仅 400 / 500」，而 `500` 在 core-ui 同样是空声明；改用 `bold` 又超 docs/07 的上限。三个选项请设计师/架构师裁决：<br>(a) 保持 `600` → 标题按常规字重（现状）<br>(b) 改 `bold` → 恢复强调，但超 docs/07 字重上限<br>(c) 去掉字重、按 docs/07「标题 14px」把标题字号提到 14px → **靠字号与颜色做层级**（与 docs/07 §4 与知识库「字重不许超 500，靠字号和颜色深浅做层级」都一致，推荐此项） | `widget_factory.cpp:1392-1395` | 设计师裁决后再改（属视觉决策，不在机械修正范围） |
| R9 | **`<input>` 的 CSS `background` 会被采纳**（原先担心的「控件自绘底色盖掉 CSS」不存在）：`TextInputWidget::OnDraw` 优先用 `bgColor`（`controls.cpp:1052-1063`，`customBg = bgColor.a > 0` 时「use it as-is (no state variation)」）→ `.global-search` / `.address` / `.search` 的 `background: var(--app-content-bg)` **生效**；代价是这三个输入框**没有 hover / focus 的底色变化**（只有边框变化）。另外未设 `border-width`/`border-color` 时会走 WinUI 双色边框 + 聚焦 accent 下划线 | `controls.cpp:1045-1085` | 已知取舍，无需改动；真机确认边框观感 |

---

## 6. 标签自检结论

**执行器（可复跑，不是一次性结论）**：
```bash
python3 scripts/gates/uix_tag_check.py       # 标签 / D-15 / SFC / id / 元素总数 / 硬编码色
python3 scripts/gates/theme_contrast_check.py # G05 双主题对比度 + G13 间距刻度
```
两条都是**失败即停**（退出码非 0）的 L1 静态门，不做编译、不做渲染。

**脚本**：对三个文件做「剥注释 → 抽标签 → 比对 `docs/11` §1 支持表 → 附加 D-15 / SFC 合法性检查」。
**执行结果（实际输出，退出码 0）**：

```
=== assets/main.uix ===
  <div> ×21       core-ui 支持
  <input> ×3      core-ui 支持
  <label> ×4      core-ui 支持
  <separator> ×4  core-ui 支持
  <span> ×8       core-ui 支持
  <style> ×1      SFC 顶层块（sfc_parser）
  <template> ×1   SFC 顶层块（sfc_parser）
  <window> ×1     SFC 顶层块（sfc_parser）
=== assets/preview.uix ===
  <div> ×5 / <label> ×5 / <separator> ×1 / <span> ×1  —— 全部 core-ui 支持
  <style> ×1 / <template> ×1 / <window> ×1          —— SFC 顶层块
=== assets/about.uix ===
  <div> ×2 / <label> ×13 / <separator> ×2          —— 全部 core-ui 支持
  <style> ×1 / <template> ×1 / <window> ×1          —— SFC 顶层块

==== 结论: 全部标签均在 core-ui 支持表内，无遗留未识别标签 ====
```

**同时通过的附加检查**（脚本内断言，任一不过退出码非 0）：

| 检查项 | 结果 |
|---|---|
| 六个旧标签（`text`/`hbox`/`vbox`/`vseparator`/`text-input`/`panel`）零残留 | ✅ |
| D-15：无 `onClick` / `v-model` / `:visible` / `@click` / `v-if` / `v-for` / `v-show` | ✅ |
| 无 `text="..."` 属性 | ✅ |
| `<window>` 均自闭合 | ✅ |
| `<script>` 含 `export default` | ✅ |
| 无 `window { }` 选择器 | ✅ |
| id 数量与取值：main 21 / preview 6 / about 0，与原文件完全一致 | ✅ |
| 硬编码色值（G02）：三个 `.uix` 内零字面色值，全部走 `var(--app-*)` | ✅ |

**主题门禁实测（`scripts/gates/theme_contrast_check.py`，退出码 0）**：

```
=== G05 双主题对比度 ===
  [light] 最低对比度 4.51:1  (--app-accent on --app-selection-bg #2563EB / #E6F1FB)
  [dark]  最低对比度 4.57:1  (--app-accent on --app-selection-bg #60A5FA / #1B3A5C)
=== G13 间距刻度 ===
  非刻度间距：0 处
PASS —— 双主题对比度全部 ≥ 4.5:1，间距取值全部在刻度上。
```

**逐标签 → core-ui 支持表 对齐**（三文件并集）：

| 资产里出现的标签 | core-ui 支持表位置 | 落地控件 |
|---|---|---|
| `div` | §1.1 布局容器 | `VBoxWidget` / `HBoxWidget`（按 CSS `flex-direction`） |
| `span` | §1.2 文本 | `LabelWidget`（wrap=true，display=inline） |
| `label` | §1.2 文本 | `LabelWidget`（wrap=true，display=inline） |
| `input` | §1.3 控件 | 单行输入 `TextInputWidget` |
| `separator` | §1.3 控件 | `SeparatorWidget`（水平，1px） |
| `window` / `template` / `style` / `script` | SFC 外层（`sfc_parser.cpp`），不是控件 | — |

**结论：三个资产里出现的每一个标签都能在 core-ui 支持表里找到落点，无遗留未识别标签。**

---

## 7. 未验证声明

- 本机**无 Rust / C++ 工具链**，本轮**没有编译、没有渲染、没有真机加载**过任何东西。
- 上文所有"core-ui 会怎样"的结论**均为源码级推断**，标注为事实依据（文件:行），**不等于运行结果**。
- 标 `[推断]` 的条目：`§2.1`「原写法整页加载失败」的**后果严重程度**（确定的是解析会失败、页不会加载；具体 UI 表现需真机确认）；`§2.5` `align-items: center` 的**实际视觉收益**；`§2.6` 分隔线色差的**可见程度**。
- 真正需要真机确认的清单见 `docs/13-UIX-FIRST-RENDER-CHECKLIST.md`（M01~M06 + 布局 / 层级 / DPI / 深浅色）。
