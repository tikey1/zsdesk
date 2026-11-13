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
| 标签替换 | `<text>`×25、`<hbox>`×8、`<vbox>`×3、`<text-input>`×3、`<panel>`×5、`<vseparator>`×2、`<separator>`×7（**共 53 处**） |
| 元素总数 | **模板体元素数不变**（main 30→30、preview 9→9、about 16→16）——只换标签，不增删元素。连 `<window>`/`<style>`/`<template>` 三个 SFC 外壳一起算是 33/12/19，同样不变 |
| id 保留 | main 21 / preview 6 / about 0 —— **一个没少，一个没改**（Rust 侧 `ui_widget_find_by_id` 不受影响） |
| class 保留 | 原有 class 全部保留；**新增 3 个**（`.root`、`.tab-placeholder`、`.nav-hint`）；`.divider` 规则从「只设 background」改为「设 height + background + flex-shrink」（见 §2.6） |
| 结构级修正 | 6 处（见 §2）——不改设计，只让 core-ui 真正认得 |
| 主题 token | 17 个 token 全部加 `--app-` 前缀（见 §4） |
| 复核 | 经 QA 第一轮复核（`docs/14-QA-UIX-REVIEW.md`）：3 阻塞 + 7 重要 + 5 建议，**已全部处理**，逐条对应见 §2.6 / §4.2 / §4.4 / §5 R10 / §6 / §8 |
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

### 2.6 `<separator>` → `<div class="divider">`（第一轮遗漏，第二轮由 QA 查出并修正）

**第一轮我写错了**：当时把 `<separator class="divider" />` 当作「受支持标签，保留」，
并补了 `.divider { background: var(--app-divider); }`。
**QA 复核推翻了这个结论**（`docs/14-QA-UIX-REVIEW.md` B2），我独立复核源码后确认 QA 正确：

| | |
|---|---|
| **依据** | `src/ui/controls.cpp:635-643` `SeparatorWidget::OnDraw` **不调用** `Widget::OnDraw(r)`；而画 `bgColor` 的正是 `Widget::OnDraw`（`src/ui/widget.cpp:328-344`）。对照 `SliderWidget::OnDraw`（`controls.cpp:665-666`）**有**调用 |
| **后果** | CSS `background` 被写进 `bgColor` 但**永远不会被画出来**。线条颜色写死 `theme::kDivider()`：light = `grey::g82` = **#D1D1D1**，dark = `grey::g26` = **#424242** |
| **受影响** | 7 处：main 4 处、preview 1 处、about 2 处 —— 它们的 `class="divider"` + `background` **零效果** |
| **上游正解** | 上游 demo 的水平分隔线**不是** `<separator>`，而是 `<div class="sep"/>` + `.sep { height: 1; background: var(--border-subtle); }`（`demo/ui_demo.uix:426` 等 10 处）—— 与竖线 `.vsep` 同款做法 |

**现写法**（三个文件统一）：

```css
/* 水平分隔线 1px：必须用 <div> 而不是 <separator> —— ... */
.divider { height: 1px; background: var(--app-divider); flex-shrink: 0; }
```
```html
<div class="divider" />
```

`flex-shrink: 0` 是为了在列方向空间不足时这条 1px 线不被压成 0（core-ui 的
`flexShrink` 默认是 1，`src/ui/widget.h:115`）。

> 与竖线 `.vsep` 现在是同一套机制、同一个 token（`--app-divider`），水平与竖向
> 分隔线颜色一致。
> 代价：分隔线不再「跟随主题自动切」（`<separator>` 本来是跟的）—— 但本项目所有
> `--app-*` 都不跟随主题（见 §4.4），统一口径反而少一处例外。

---

## 3. 逐标签对照表

> 「原:行」= `git show HEAD:assets/<file>` 的行号；「现:行」= 修正后文件的行号。
> **影响面**统一按：core-ui 侧 = 控件类型/布局行为；Rust 侧 = 是否影响 `ui_widget_find_by_id` / 绘制回调。

### 3.1 `assets/main.uix`

| # | 原标签 | 原:行 | 现标签 | 现:行 | 取舍理由 | 影响面 |
|---|---|---|---|---|---|---|
| 1 | `<vbox>` | 45 | `<div class="root">` | 140 | VBox 无对应标签；`<div>` 默认 column = 原 `<vbox>` 语义 | core-ui：同为 VBox。Rust：根节点 id 无（原本也无） |
| 2 | `<hbox id="titlebar_host">` | 46 | `<div id="titlebar_host" class="titlebar">` | 141 | 横排靠 CSS：`.titlebar` 补 `flex-direction: row` | core-ui：VBox→HBox。Rust：id 不变，句柄照常取 |
| 3 | `<text class="app-title">` | 47 | `<span class="app-title">tabbed_files</span>` | 142 | 标题栏应用名，短、单行、不占满 → `<span>`（inline 不 stretch）；加 `white-space: nowrap` | core-ui：LabelWidget(inline)。Rust：原本无 id，不影响 |
| 4 | `<text-input id="global_search">` | 48 | `<input id="global_search">` | 143 | 标签无对应控件；`<input>` → TextInputWidget | core-ui：VBox(空盒)→TextInputWidget。Rust：id 不变 |
| 5 | `<hbox id="tab_host">` | 50 | `<div id="tab_host" class="tabbar">` | 145 | 同 #2 | 同上 |
| 6 | `<label id="tab_placeholder">` | 51 | `<label id="tab_placeholder" class="tab-placeholder">…</label>` | 146 | **原就是 `<label>`，保留**；只补 class 拿颜色 + 内部文本 | core-ui：不变。Rust：id 不变 |
| 7 | `<separator class="divider">` | 53 | `<div class="divider" />` | 148 | **第二轮修正**：`<separator>` 不画 CSS background（§2.6）→ 改用 1px 的 div | core-ui：SeparatorWidget → 1px 实心 div，颜色走项目 token。Rust：无 id |
| 8 | `<hbox id="command_host">` | 54 | `<div id="command_host" class="commandbar">` | 149 | 同 #2 | 同上 |
| 9 | `<label id="nav_hint">` | 55 | `<label id="nav_hint" class="nav-hint">…</label>` | 150 | **原就是 `<label>`，保留**；补 class 拿颜色 | 同上 |
| 10 | `<separator class="divider">` | 57 | `<div class="divider" />` | 152 | 同 #7 | 同上 |
| 11 | `<hbox id="address_host">` | 58 | `<div id="address_host" class="addressbar">` | 153 | 同 #2 | 同上 |
| 12 | `<panel id="breadcrumb">` | 59 | `<div id="breadcrumb" class="breadcrumb" />` | 154 | `<panel>` 无对应控件 → `<div>`（column） | core-ui：VBox→VBox。Rust：id 不变 |
| 13 | `<text-input id="address">` | 60 | `<input id="address">` | 155 | 同 #4 | 同上 |
| 14 | `<text-input id="search">` | 61 | `<input id="search">` | 156 | 同 #4 | 同上 |
| 15 | `<separator class="divider">` | 63 | `<div class="divider" />` | 158 | 同 #7 | 同上 |
| 16 | `<hbox id="infobar_host">` | 64 | `<div id="infobar_host" class="infobar" />` | 159 | 同 #2 | 同上 |
| 17 | `<hbox class="main-area">` | 65 | `<div class="main-area">` | 160 | 同 #2 | 同上 |
| 18 | `<panel id="nav_host">` | 66 | `<div id="nav_host" class="nav-host" />` | 161 | 同 #12 | 同上；见 §5 R5（Rust 自绘挂载点） |
| 19 | `<vseparator class="divider">` | 67 | `<div class="vsep" />` | 162 | `<Separator>` 只有水平形态（`controls.h:200`），竖线用 1px 宽 div：`width:1px; background:var(--app-divider); align-self:stretch; flex-shrink:0` | core-ui：VBox(空盒)→1px 竖条。Rust：无 id，无影响 |
| 20 | `<panel id="list_host">` | 68 | `<div id="list_host" class="list-host" />` | 163 | 同 #12 | 同上；见 §5 R5 |
| 21 | `<vseparator class="divider">` | 69 | `<div class="vsep" />` | 164 | 同 #19 | 同上 |
| 22 | `<panel id="preview_host">` | 70 | `<div id="preview_host" class="preview-host" />` | 165 | 同 #12 | 同上 |
| 23 | `<separator class="divider">` | 72 | `<div class="divider" />` | 167 | 同 #7 | 同上 |
| 24 | `<hbox id="status_host">` | 73 | `<div id="status_host" class="statusbar">` | 168 | 同 #2 | 同上 |
| 25-30 | `<text id="status_*" class="status-segment">` ×6 | 74-79 | `<span id="status_*" class="status-segment" />` ×6 | 169-174 | 状态栏片段，短、单行、由 Rust 运行时填字 → `<span>`；配 `white-space: nowrap` 防长值折行 | core-ui：VBox(空盒)→LabelWidget(inline)。Rust：id 不变，`ui_label_set_text` 可用 |

### 3.2 `assets/preview.uix`

| # | 原标签 | 原:行 | 现标签 | 现:行 | 取舍理由 | 影响面 |
|---|---|---|---|---|---|---|
| 1 | `<vbox>` | 20 | `<div class="root">` | 69 | 同 main #1 | 同上 |
| 2 | `<hbox class="preview-toolbar">` | 21 | `<div class="preview-toolbar">` | 70 | 横排靠 `.preview-toolbar { flex-direction: row; align-items: center; }` | core-ui：VBox→HBox |
| 3 | `<text id="preview_title">` | 22 | `<span id="preview_title" class="preview-title">预览</span>` | 71 | 工具条标题，短、单行 → `<span>` | core-ui：VBox→LabelWidget(inline)。Rust：id 不变 |
| 4 | `<separator />` | 24 | `<div class="divider" />` | 73 | **第二轮修正**：`<separator>` 不画 CSS background（§2.6）→ 改用 1px 的 div | core-ui：SeparatorWidget → 1px 实心 div，颜色走项目 token |
| 5 | `<panel id="preview_body">` | 25 | `<div id="preview_body" class="preview-body" />` | 74 | `<panel>` → `<div>` | core-ui：VBox→VBox。Rust：id 不变 |
| 6 | `<text id="preview_hint">` | 26 | `<label id="preview_hint" class="preview-hint">…</label>` | 75 | 提示行需要**占满宽度并可换行**（Rust 写入的通道名可能较长）→ `<label>` + `display: block` | core-ui：VBox(空盒)→LabelWidget(block/stretch)。Rust：id 不变 |
| 7 | `<text id="preview_channel">` | 27 | `<label id="preview_channel" class="preview-hint" />` | 76 | 同 #6 | 同上 |
| 8 | `<text id="preview_size">` | 28 | `<label id="preview_size" class="preview-hint" />` | 77 | 同 #6 | 同上 |
| 9 | `<text id="preview_state">` | 29 | `<label id="preview_state" class="preview-state" />` | 78 | 状态行需占满宽度 → `<label>` + `display: block` | 同上 |

### 3.3 `assets/about.uix`

| # | 原标签 | 原:行 | 现标签 | 现:行 | 取舍理由 | 影响面 |
|---|---|---|---|---|---|---|
| 1 | `<vbox>` | 14 | `<div class="root">` | 54 | 同 main #1 | core-ui：VBox→VBox |
| 2 | `<text class="heading">` | 15 | `<label class="heading">…</label>` | 55 | 标题虽短，但按「占满宽度」语义统一用 `<label>` + `display: block` | core-ui：VBox(空盒)→LabelWidget(block) |
| 3 | `<text class="row">`（首条） | 16 | `<label class="row">…</label>` | 56 | 条目是长句（最长达 40+ 汉字），**必须占满宽度才会折行**；若保持 inline，宽度=自然文字宽度 → 横向溢出窗口而不换行 | 同上 |
| 4 | `<separator />` | 17 | `<div class="divider" />` | 57 | **第二轮修正**：`<separator>` 不画 CSS background（§2.6）→ 改用 1px 的 div | core-ui：SeparatorWidget → 1px 实心 div，颜色走项目 token |
| 5 | `<text class="row">` ×10 | 18-27 | `<label class="row">…</label>` ×10 | 58-67 | 同 #3 | 同上 |
| 6 | `<separator />` | 28 | `<div class="divider" />` | 68 | 同 #4 | 同上 |
| 7 | `<text class="row">`（致谢） | 29 | `<label class="row">…</label>` | 69 | 同 #3 | 同上 |

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

#### ⚠️ 注入位置（第二轮由 QA 查出，第一轮我漏了 —— 这是会导致「一个窗口都出不来」的坑）

`:root { … }` **必须拼进 `<style>…</style>` 块内部**（紧跟 `<style>` 开头），
**绝不能放在源码顶层**：

| 位置 | 后果 | 依据 |
|---|---|---|
| **`<style>` 块内部**（正确） | 自定义属性被 `CollectRootVars` 采集 → `var(--app-*)` 正常解析 | `src/ui/css/cascade.h:41-43`（只从 `:root` 采集）、`cascade.cpp:18-33` |
| **源码顶层**（错误） | `sfc_parser.cpp:260-266` 判 "unexpected content at top level" → `doc.ok=false` → `page_api.cpp:249-255` 返回 "SFC parse failed" → **整页不加载，三个窗口一个都出不来** | `sfc_parser.cpp:260-266` |

> 报错信息是 "SFC parse failed"，**不像标签问题**，极容易被误判成"改完标签还是出不来"，
> 然后去反复调标签 —— 所以这条单列。

**三种失败模式（都是静默或误报型的）**：

| 失败模式 | 现象 | 定位 |
|---|---|---|
| 顶层注入 | "SFC parse failed" → 一个窗口都出不来 | 看 `ui_page_last_error()` |
| 完全没注入 | **静默**：界面能出来，但所有 `var(--app-*)` 解析成空串（`cascade.cpp:79-81` → `ParseColor("")` 失败）→ 全是库默认色（纯白底 / `#424242` 文字 / `#D1D1D1` 分隔线） | 比对颜色与 `themes.json` |
| 注入晚于 `ui_page_load_string` | 同上，静默回落到库默认色 | 比对时序 |

→ 三条都已写进 `assets/themes.json` 的 `revision.inject_failure_modes` 与 `docs/13` §0 上机前准备清单。

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

#### ⚠️ 重建页面的副作用：27 个句柄全部失效（第二轮由 QA 查出）

`ui_page_load_string()` 返回的是**一个全新的 page**。旧 page 里的
**27 个 `ui_widget_find_by_id` 句柄（main 21 + preview 6）全部失效**，
且旧 page 必须显式 `ui_page_destroy`。

→ **切主题后 Rust 侧必须做三件事**（已登记为 §5 R4）：

1. `ui_page_destroy(旧 page)` —— 不销毁就是句柄 + 内存双泄漏；
2. **重新 `ui_widget_find_by_id` 取全部 27 个句柄** —— 旧句柄指向已销毁的控件，
   继续用会写到野内存（静默崩溃 / 花屏，不会报"句柄无效"）；
3. 重新挂载回调与自绘状态（列表、导航、面包屑、预览）。

→ 判据：切主题后如果状态栏不再更新、列表空白、预览不刷新，**第一嫌疑就是旧句柄没重取**。

> `ui_page_reload()` 帮不上忙：它重放**旧源码**（`page_api.cpp:565-577`），palette 改不了；
> 而且它同样会重新编译，旧句柄一样失效。

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
| R4 | **深浅色切换需重建页面**（不能只调 `ui_theme_set_mode`）；且重建后**必须 `ui_page_destroy` 旧 page + 重取全部 27 个 `ui_widget_find_by_id` 句柄 + 重挂回调**（新 page ⇒ 旧句柄全部失效） | §4.4 | 主题切换处 |
| R10 | **`.tab-active` / `.cmd-button` / `.cmd-group` / `.infobar-visible` / `.preview-collapsed` 五条 class 规则当前无任何元素引用**（标签条与命令栏由 Rust 自绘），且 core-ui **没有运行时设置 class 的 C API**（`include/ui_core.h` 内无相关符号；`dynamicClasses` 只来自 `:class` 绑定）→ 这五条是**绘制 / 状态规范参照**，不是生效的 CSS（`.tab-active` `.cmd-button` `.cmd-group` 是标签条与命令栏的绘制规范；`.infobar-visible` `.preview-collapsed` 是两个状态的尺寸规范，Rust 侧需直接改高度 / 宽度，见 R7）。已在三个 `.uix` 里逐条加注释标明。另：`.tab-active` 里的 `var(--accent)` 是**库 token**（随 `ui_theme_set_mode` 切换），与其余 `--app-*`（不跟随）刻意分属两个命名空间 —— 指示条本就该跟随系统主题 | 第二轮由 QA I7 查出；`src/ui/page/compiler.cpp:859`（dynamicClasses 来源） | Rust 侧 `src/ui/tabbar.rs` / `toolbar.rs` 实现时按此对齐 |
| R5 | **列表/导航/面包屑自绘的挂载方式**：`ui_custom_on_draw` 只对 CustomWidget 生效（`ui_api.cpp:2612-2616` 的 `As<ui::CustomWidget>` 取不到就**静默 no-op**），`<div>` 宿主拿不到绘制回调 | `ui_custom(void)`（`ui_core.h:1415`）+ `ui_widget_add_child(parent, child)`（`ui_core.h:1083`）可运行时创建并挂载 | 虚拟列表 / 导航树 |
| R6 | **`.tab-active` 的 2px accent 指示条**：core-ui 不支持 `border-bottom` | `widget_factory.cpp:631-653` | 由 Rust 自绘，或在 `.uix` 里加 2px 高的 `<div>` |
| R7 | **`.infobar` 默认 `height: 0px`**（收起），`.infobar-visible` 36px；两者都无 id 切换机制 → Rust 侧需 `ui_widget_set_visible`（`<div>` 支持 `visible` 静态属性，`widget_factory.cpp:1329-1332`）或直接改高度 | — | 信息条状态机 |
| R8 | **`.preview-title` / `.heading` 的 `font-weight: 600` 是空声明**：core-ui 对 LabelWidget 只认 `bold` / `700`（`widget_factory.cpp:1392-1395`：`if (fw == "bold" \|\| fw == "700") lbl->Bold();`），`600` 落在 else，**什么都不做** → 这两个标题会按常规字重渲染，层级丢失。<br>**但本轮不擅自改**：docs/07 §4 明写「字重仅 400 / 500」，而 `500` 在 core-ui 同样是空声明；改用 `bold` 又超 docs/07 的上限。三个选项请设计师/架构师裁决：<br>(a) 保持 `600` → 标题按常规字重（现状）<br>(b) 改 `bold` → 恢复强调，但超 docs/07 字重上限<br>(c) 去掉字重、按 docs/07「标题 14px」把标题字号提到 14px → **靠字号与颜色做层级**（与 docs/07 §4 与知识库「字重不许超 500，靠字号和颜色深浅做层级」都一致，推荐此项） | `widget_factory.cpp:1392-1395` | 设计师裁决后再改（属视觉决策，不在机械修正范围） |
| R9 | **`<input>` 的 CSS `background` 会被采纳**（原先担心的「控件自绘底色盖掉 CSS」不存在）：`TextInputWidget::OnDraw` 优先用 `bgColor`（`controls.cpp:1052-1063`，`customBg = bgColor.a > 0` 时「use it as-is (no state variation)」）→ `.global-search` / `.address` / `.search` 的 `background: var(--app-content-bg)` **生效**；代价是这三个输入框**没有 hover / focus 的底色变化**（只有边框变化）。另外未设 `border-width`/`border-color` 时会走 WinUI 双色边框 + 聚焦 accent 下划线 | `controls.cpp:1045-1085` | 已知取舍，无需改动；真机确认边框观感 |

---

## 6. 标签自检结论

**执行器（可复跑，不是一次性结论）**：

```bash
python3 scripts/gates/uix_tag_check.py        # 标签 / D-15 / SFC / id / 模板体元素数 / 硬编码色
python3 scripts/gates/theme_contrast_check.py  # G05 真实渲染关系对比度 + G13 间距刻度
```

两条都是**失败即停**（退出码非 0）的 L1 静态门，不做编译、不做渲染。

> ⚠️ **第一轮我贴的输出是错的**（QA B3）：贴的是临时脚本的输出，与仓库里的
> `scripts/gates/uix_tag_check.py` 对不上（开标签+闭标签混算）。本节现贴的是
> **当前仓库脚本的真实输出**，任何人都能按上面的命令复现。

**`uix_tag_check.py` 真实输出**（退出码 0）：

```

=== assets/main.uix ===
  <div> ×18  core-ui 支持
  <input> ×3   core-ui 支持
  <label> ×2   core-ui 支持
  <span> ×7   core-ui 支持
  <style> ×1   SFC 顶层块
  <template> ×1   SFC 顶层块
  <window> ×1   SFC 顶层块
  ✔ 模板体元素 30 个 / id 21 个 —— 全部检查通过

=== assets/preview.uix ===
  <div> ×4   core-ui 支持
  <label> ×4   core-ui 支持
  <span> ×1   core-ui 支持
  <style> ×1   SFC 顶层块
  <template> ×1   SFC 顶层块
  <window> ×1   SFC 顶层块
  ✔ 模板体元素 9 个 / id 6 个 —— 全部检查通过

=== assets/about.uix ===
  <div> ×3   core-ui 支持
  <label> ×13  core-ui 支持
  <style> ×1   SFC 顶层块
  <template> ×1   SFC 顶层块
  <window> ×1   SFC 顶层块
  ✔ 模板体元素 16 个 / id 0 个 —— 全部检查通过

PASS —— 三个 .uix 资产：标签全部在 core-ui 支持表内，零遗留旧标签，D-15 合规，id / 元素总数保全。
```

**断言清单**（脚本内断言，任一不过退出码非 0）：

| 检查项 | 结果 |
|---|---|
| 六个旧标签（`text`/`hbox`/`vbox`/`vseparator`/`text-input`/`panel`）零残留 | ✅ |
| 未知标签零出现（每个标签都能在 docs/11 §1 支持表找到落点） | ✅ |
| D-15：无 `onClick` / `v-model` / `:visible` / `@click` / `v-if` / `v-for` / `v-show` | ✅ |
| 无 `text="..."` 属性 | ✅ |
| `<window>` 均自闭合 | ✅ |
| `<script>` 含 `export default` | ✅ |
| 无 `window { }` 选择器 | ✅ |
| id 集合与取值：main 21 / preview 6 / about 0，与原文件完全一致 | ✅ |
| **模板体**元素数：main 30 / preview 9 / about 16，与原文件一致（含 SFC 外壳则 33/12/19） | ✅ |
| 硬编码色值（G02）：三个 `.uix` 内零字面色值，全部走 `var(--app-*)` | ✅ |

**逐标签 → core-ui 支持表 对齐**（三文件并集）：

| 资产里出现的标签 | core-ui 支持表位置 | 落地控件 |
|---|---|---|
| `div` | §1.1 布局容器 | `VBoxWidget` / `HBoxWidget`（按 CSS `flex-direction`） |
| `span` | §1.2 文本 | `LabelWidget`（wrap=true，display=inline） |
| `label` | §1.2 文本 | `LabelWidget`（wrap=true，display=inline） |
| `input` | §1.3 控件 | 单行输入 `TextInputWidget` |
| `window` / `template` / `style` / `script` | SFC 外层（`sfc_parser.cpp`），不是控件 | — |

> 注：`<separator>` 在第二轮已全部换成 `<div>`（§2.6），资产里不再出现。

**结论：三个资产里出现的每一个标签都能在 core-ui 支持表里找到落点，无遗留未识别标签。**

**主题门禁真实输出**（`theme_contrast_check.py`，退出码 0）：

```
=== G05 双主题对比度（按 .uix 真实渲染关系计算）===
  [light] 真实文字元素 55 处，最低 5.78:1（span.app-title #616161 on #F7F7F7）
        assets/main.uix span.app-title: #616161 on #F7F7F7 = 5.78:1 「tabbed_files」
        assets/main.uix label.nav-hint: #616161 on #F7F7F7 = 5.78:1 「后退 前进 上级 刷新 · 查看 排序 预览」
        assets/preview.uix label.preview-hint: #616161 on #FBFBFB = 5.98:1 「选中单个文件以预览」
  [dark] 真实文字元素 55 处，最低 6.75:1（span.app-title #B3B3B3 on #2B2B2B）
        assets/main.uix span.app-title: #B3B3B3 on #2B2B2B = 6.75:1 「tabbed_files」
        assets/main.uix label.nav-hint: #B3B3B3 on #2B2B2B = 6.75:1 「后退 前进 上级 刷新 · 查看 排序 预览」
        assets/about.uix label.row: #B3B3B3 on #2B2B2B = 6.75:1 「单进程 · Core UI · Rust 原生标」

=== G13 间距刻度 ===
  非刻度间距：0 处

PASS —— 真实文字元素的对比度全部 ≥ 4.5:1，间距取值全部在刻度上。
```

> 第二轮重写了这个脚本（QA I5）：第一版验的是**UI 里不存在的组合**
> （如 `--app-accent` on `--app-selection-bg`，而 `--app-selection-bg` 从未被任何
> `.uix` 用作背景），最差对是 4.51:1 —— 结论虽对，但验错了对象。
> 现按**真实渲染关系**计算：解析 `.uix` 的 `<template>` 元素树 → 取每个有文字元素的
> `color`（无 color 规则则用库默认 `theme::kBtnText()`）与**最近祖先-or-self** 的
> `background` → 逐对算对比度。
> 实测 **55 个真实文字元素 × 2 套主题 = 110 对**，最低 **5.78:1**（light）/ **6.75:1**（dark），
> 与 QA 独立复算结果一致。

---

## 7. 未验证声明

- 本机**无 Rust / C++ 工具链**，本轮**没有编译、没有渲染、没有真机加载**过任何东西。
- 上文所有"core-ui 会怎样"的结论**均为源码级推断**，标注为事实依据（文件:行），**不等于运行结果**。
- 标 `[推断]` 的条目：`§2.1`「原写法整页加载失败」的**后果严重程度**（确定的是解析会失败、页不会加载；具体 UI 表现需真机确认）；`§2.5` `align-items: center` 的**实际视觉收益**；`§2.6` 换成 `<div>` 后分隔线的**实际观感**（含 1px 线在 150% DPI 下是否可见）。
- 第二轮把 `<separator>` 换成 `<div>` 后，**分隔线在真机上的颜色与粗细未经任何验证** —— 这是 QA 要求真机优先看的两条之一。
- 真正需要真机确认的清单见 `docs/13-UIX-FIRST-RENDER-CHECKLIST.md`（M01~M06 + 布局 / 层级 / 字体 / DPI / 深浅色）。

---

## 8. 对 QA 第一轮复核的逐条回应

> 复核报告：`docs/14-QA-UIX-REVIEW.md`。**3 阻塞 + 7 重要 + 5 建议，全部已处理。**
> 每一条我都独立复核了源码后才改，不按建议直接照抄。

| QA 编号 | 我的结论 | 处理 |
|---|---|---|
| **B1** `:root` 注入位置没规定 | **QA 正确**。我第一轮只说了"拼进源码字符串"，没写位置 | `themes.json` 的 `revision.inject` 改写为「必须拼进 `<style>` 块**内部**，绝不能放源码顶层」，并新增 `revision.inject_failure_modes`（三种失败模式）；docs/12 §4.2 加专节 + 位置对照表；docs/13 §0 加 3 条上机前自查 |
| **B2** `<separator>` 的 CSS `background` 无效 | **QA 正确，我第一轮写错了**。`SeparatorWidget::OnDraw` 不调 `Widget::OnDraw`，背景永远不画 | 7 处 `<separator>` 全改 `<div class="divider">` + `.divider { height:1px; background:var(--app-divider); flex-shrink:0 }`；docs/12 §2.6 重写（含我错在哪）；docs/13 L9/C5/§8 同步修正；三个 `.uix` 头部记录该变更 |
| **B3** docs/12 §6 输出复现不出来 | **QA 正确**。我贴的是临时脚本输出，与仓库脚本对不上 | §6 重写为**当前仓库脚本的真实输出**（可用文首命令复现）+ 断言清单 + 判定标准；并写明第一轮错在哪 |
| **I1** docs/11 §2 计数表有误 | **QA 正确**。实测 text×25 / hbox×8 / vbox×3 / text-input×3 / panel×5 / vseparator×2（+ 第二轮的 separator×7） | docs/11 §2 已回填真实数字，并标注与 docs/12 的对应关系 |
| **I2** docs/11 §4 M01/M06 已被源码结案 | **QA 正确** | docs/11 §4 M01/M06 已改为「源码级已结案」，并保留真机复验项 |
| **I3** 切主题重建页面 → 27 个句柄失效 | **QA 正确，补漏了** | docs/12 §4.4 加「重建页面的副作用」专节（destroy + 重取 + 重挂三步）；§5 R4 改写；themes.json `revision.switch` 同步 |
| **I4** docs/13 F3 与 F6 自相矛盾 | **QA 正确** | F3 改为「16px、**不加粗**（见 F6）」；R8 的三选一保持**未裁决**（等设计师） |
| **I5** 对比度门禁验了不存在的组合 | **QA 正确** | 脚本重写为按 **DOM 真实渲染关系**计算（元素 color × 最近祖先-or-self background），实测 55 个文字元素 × 2 主题 = 110 对，最低 5.78:1 / 6.75:1，与 QA 独立复算一致 |
| **I6** `scripts/gates/` 未纳入版本控制 | **QA 正确** | 已 `git add`（连同 docs/12、docs/13） |
| **I7** `.tab-active` 三重失效 | **QA 正确，前两点我漏了**（`border-bottom` 不支持我已知，命名空间与「无元素引用」是我漏的） | 三个 `.uix` 里给 `.tab-active` / `.cmd-button` / `.cmd-group` 加注释标明「Rust 自绘参照规范，当前不生效」+ 说明 `var(--accent)` 是库 token 且刻意分属两个命名空间；docs/12 §5 新增 **R10** 登记 |
| **S1** 元素总数口径把 SFC 外壳算进去了 | **QA 正确**，且我第一轮 §0 与 §3 表自相矛盾 | 口径统一为**模板体元素数 30 / 9 / 16**（另注明含 SFC 外壳是 33/12/19）；门禁脚本同步改为只数模板体 |
| **S2** docs/11 §3「ui_debug_* 71 个」/ §6「value.h:91」 | **部分正确**。71 是**导出的 `UI_API ui_debug_*` 声明数**（`grep -c "UI_API.*ui_debug_"` = 71 ✓），头文件内含该前缀的共 73 处（2 处在注释里）；`value.h:91` 是注释行、声明在 `:93` | docs/11 已改为写明计数口径（71 个导出声明 / 全文 73 处提及）与 `value.h:91-93` |
| **S3** docs/11 §1.3 `input` 行漏了子类型 | **QA 正确** | docs/11 §1.3 已补 `type="radio"/"range"/"number"/"password"`（`widget_factory.cpp:1207-1283`） |
| **S4** `.root` 的 `font-family` 对 LabelWidget 无效 | **QA 正确** | 已在三个 `.uix` 的 `.root` 上方加注释「设计记录，字体由 `ui_theme_set_default_font` 决定」 |
| **S5** 根 `<div class="root">` 铺满窗口可结案 | **接受** | 已核实 `ui_window.cpp:1533` `root_->rect = {0,0,size.width,size.height}`；docs/13 §9 该条改为**已结案**（QA 提供的行号，与我的独立查证一致） |
