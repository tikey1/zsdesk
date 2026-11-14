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
| 元素总数 | **模板体元素数**：main **35**、preview **9**、about **16**。<br>**口径统一**：一律指**模板体**元素数，**不含 4 个 SFC 外壳标签**（`<window>` `<script>` `<style>` `<template>`）。<br>⚠️ **第七轮起 main 从 30 变成 35**：架构裁定把 `preview_host` 拆成「容器 + `<custom id="preview_image">` + 4 个元信息 `<label>`」⇒ **主动新增 5 个元素**（不是标签替换）。改之前的 30 与原文件一致；改之后 35 = 30 + 5。<br>（含 SFC 外壳则是 39/12/19） |
| id | main **26** / preview 6 / about 0 —— 原有 **27 个一个没少、一个没改**，第七轮**新增 5 个**（`preview_image` + `preview_host_name/size/channel/state`）⇒ 现共 **32 个**句柄（Rust 侧 `ui_widget_find_by_id` 的目标集合见 §5 R4） |
| class 保留 | 原有 class 全部保留；**新增 3 个**（`.root`、`.tab-placeholder`、`.nav-hint`）；`.divider` 规则从「只设 background」改为「设 height + background + flex-shrink」（见 §2.6） |
| 结构级修正 | 7 处（见 §2）——不改设计，只让 core-ui 真正认得 |
| 宿主标签 | **Rust 自绘内容的宿主 → `<custom>`；core-ui 控件/文本投影 → `<div>` / `<label>`；混合 → `<div>` 容器 + 里面的 `<custom>`**（§5 R5 的通用规则）。**六处**已落：`list_host` / `nav_host` / `preview_image` / `preview_body` = `<custom>`；`breadcrumb` = `<label>`；`preview_host` = `<div>` 容器 |
| 主题 token | 17 个 token 全部加 `--app-` 前缀（见 §4） |
| 复核 | QA 第一轮（3 阻塞 + 7 重要 + 5 建议）**已全部处理**；QA 第二轮（B2 / I5 / M06 复核通过）**通过**；交付总监裁决 I4（标题字重）**已落**；架构裁定自绘宿主用 `<custom>` **已落**。<br>**第十一轮**：QA 第七轮的 5 条开放项（O-1~O-5）**全部处理**（其中 O-2 是**我自己写错了机制**，已改正）；架构第九轮的**资产头注释自相矛盾**已改（三个文件）；**基线从 `git show HEAD:` 钉到 `6ea629f`**（HEAD 已含修正，照原写法会得出错误结论）。逐条对应见 §2.6 / §4.2 / §4.4 / §5 R4/R5/R5b/R6/R10/R11/R12 / §6 / §8 / §10 |
| 未验证 | **本机无 Rust/C++ 工具链，全部结论未经编译或渲染验证**；凡推断均标 `[推断]` |

> ⚠️ **docs/11 §2 的标签统计有误**：它写 `<text>`×10、`<hbox>`×7、`<vbox>`×1、`<text-input>`×2、`<panel>`×4。
> 用 **`git show 6ea629f:assets/*.uix`**（`6ea629f` = 本次修正**之前**的最后一个原始提交）
> 剥注释后数模板体标签，实测应为
> `<text>`×**25**、`<hbox>`×**8**、`<vbox>`×**3**、`<text-input>`×**3**、`<panel>`×**5**、`<vseparator>`×2。
>
> ⚠️ **不要用 `git show HEAD:` 取原始资产**：本次修正**本身已被提交**
> （`9a6234b`「界面资产按 QA 复核修完」→ `3cd3a08`/`b336ae4`「落地 I4 裁决」）
> ⇒ **`HEAD` 现在返回的就是修正后的文件**，用它统计只会得到「一个旧标签都不剩」的错误结论。
> **原始基线固定为 `6ea629f`**（`docs(fork): sync WinUI3 UI design assets`）。
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
| CSS 变量 `var()` 支持，自定义属性**必须在 `:root`** | `src/ui/css/cascade.cpp:42-54`（匹配 `var(`）、`src/ui/css/value.h:91-93`、`cascade.h:41-43`、`:134`（非 `:root` 级自定义属性被忽略） |
| **`:root` 块里只能放变量**，放普通样式会被丢掉 | `cascade.cpp:125`「Skip :root — those only define CSS vars, handled separately」→ `:root` 规则整体被跳过（`CollectRootVars` 单独处理其中的 `--*`） |
| `<custom>` 的 CSS 宽高**会**生效（与 `<separator>` 相反） | `widget_factory.cpp:1114-1118` 注释原文「CSS width/height 走 ApplyCommonStyle 自然生效」 |
| 库自带 34 个 token，`ui_theme_set_mode` 会覆盖同名用户值 | `src/ui/page/compiler.cpp:1647-1660`（`kThemeKeys[]` 34 项）+ `:1672-1700`（`RebuildThemeVars` 先 erase 再重写） |

---

## 2. 结构级修正（超出 docs/11 §2.1 的 6 条规则，但**不改设计**）

docs/11 §2.1 的 6 条规则只解决「标签不被识别」。实读上游后发现 **5 处结构级问题**，不修则改完标签仍然出不来界面。全部只改写法，不改尺寸 / 间距 / 配色 / 层级。

### 2.1 `<window>` 必须自闭合，`<script>/<style>/<template>` 提到顶层

| | 原写法 | 现写法 |
|---|---|---|
| main | `<window title="tabbed_files" width="1200" height="800">` … `</window>` | `<window title="tabbed_files" width="1200" height="800" />` + 顶层 `<script>`/`<style>`/`<template>` |

**依据**（路径是 `src/ui/uix/sfc_parser.cpp`，**不是** `src/ui/page/`）：
`sfc_parser.cpp:294`「`<window>` must be self-closing (use `<window .../>`)」→ `pushErr` → `skipUntilClose` 会把 `</window>` 之前的内容**整段跳过** → `sfc_parser.cpp:373`「`if (!doc.errors.empty()) doc.ok = false;`」→ `page_api.cpp:249-255` 返回 "SFC parse failed"。
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

**依据**：`<window>` 是 SFC 层面的窗口配置标签，**不产生任何 widget**（`sfc_parser.cpp:285-296` 只把 attrs 存进 `doc.windowAttrs`）→ **没有任何节点的 tag 是 `window`** → 整条规则命中 **0 个节点**（级联只对模板节点匹配，`cascade.cpp:112-145`）。
**改口说明**：不是「`window` 规则会落到根节点」，而是「**`window` 选择器匹配不到任何东西**」—— 必须**显式给根 `<div>` 加 `class="root"` 再用 `.root { … }`**。写法结果一样，但按前者理解会以为 `window` 选择器有效，将来 core-ui 若加了同名节点就会踩坑。
**另**：`:root` 是伪类，在普通样式计算里被**显式跳过**（`cascade.cpp:125`）→ `:root` 块里**只能放 CSS 变量**，放 `background`/`color`/`font` 会被丢掉。
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

> 库默认文字色是 `theme::kBtnText()` = **`#242424`**（light，grey::g14；`controls.cpp:61-68` 取值路径 `css.hasFg → customColor_ → theme::kBtnText()`），与项目的 `--app-text`（#1B1B1B）差一档多 —— 不补色会明显偏灰。
> ⚠️ 我第一轮这里写的是 `contentText = #424242`（grey::g26），**数字错了**（QA 第二轮指出）：那是 `theme::Current().contentText`（dark = `#D6D6D6` / grey::g84），但 LabelWidget 实际走的是 `kBtnText()`。结论方向不变，数字已更正。

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
| **上游正解** | 上游 demo 的水平分隔线**主力是** `<div class="sep"/>` + `.sep { height: 1; background: var(--border-subtle); }`（`demo/ui_demo.uix:203` 定义规则，**19 处**用 `<div class="sep"/>`）—— 与竖线 `.vsep` 同款做法。<br>⚠️ **更正**：我第一轮写「demo 里 `<separator>` 一次都没出现」**不实** —— demo 另有 **3 处真实 `<separator/>`**（`demo/ui_demo.uix:663, 672, 687`），**那 3 处只能渲染成库色**（印证了本条结论，不是反例）。<br>**进一步澄清**（架构侧交叉核对）：那 3 处**全在 `<menu>` 内部** —— 是菜单项之间的分隔线，用库色正好；**布局里的分隔线 demo 一律用 `<div class="sep"/>`**。⇒ 结论不变：**布局里的分隔线要跟项目 token 同色 → 用 `<div>` + `background`**。<br>另：本项目的 7 处用的是**项目 token** `--app-divider`，而不是 demo 的库 token `var(--border-subtle)`。 |

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

### 2.7 `.uix` 里**不允许出现深浅色分支**（QA 第四轮查出的硬约束，本轮资产已符合）

`.uix` 的 CSS 选择器**没有主题态位** —— 上游 `src/ui/css/selector.cpp:60-64` 的伪类只认
**hover / pressed / focus / disabled / checked / root / drag-over**，**没有「深色 / 浅色」这类主题伪类**。

⇒ 以下写法**永远不命中**（静默失效，和 `text=` 属性、`border-bottom`、`<separator>` 的 `background` 同一类坑）：

```css
.dark { … }                      /* ✘ 不命中 */
[data-theme="dark"] { … }       /* ✘ 不命中 */
```

⇒ **深浅色只能靠 Rust 侧重注 `:root` + 重建页面**（§4.4），CSS 侧无能为力。
本轮三个资产**没有任何主题分支选择器**（只有 `.root` 挂 `--app-*`），符合该约束。

---

## 3. 逐标签对照表

> 「原:行」= **`git show 6ea629f:assets/<file>`**（原始资产）的行号；「现:行」= 修正后文件的行号。
> ⚠️ **基线是 `6ea629f` 而不是 `HEAD`** —— 本次修正已提交，`HEAD` 返回的已是修正后的文件（见 §0 警示）。
> **影响面**统一按：core-ui 侧 = 控件类型/布局行为；Rust 侧 = 是否影响 `ui_widget_find_by_id` / 绘制回调。

### 3.1 `assets/main.uix`

| # | 原标签 | 原:行 | 现标签 | 现:行 | 取舍理由 | 影响面 |
|---|---|---|---|---|---|---|
|  1 | `<vbox>` | 45 | `<div class="root">` | 186 | VBox 无对应标签；`<div>` 默认 column = 原 `<vbox>` 语义 | core-ui：同为 VBox。Rust：根节点 id 无（原本也无） |
|  2 | `<hbox id="titlebar_host">` | 46 | `<div id="titlebar_host" class="titlebar">` | 187 | 横排靠 CSS：`.titlebar` 补 `flex-direction: row` | core-ui：VBox→HBox。Rust：id 不变，句柄照常取 |
|  3 | `<text class="app-title">` | 47 | `<span class="app-title">tabbed_files</span>` | 188 | 标题栏应用名，短、单行、不占满 → `<span>`（inline 不 stretch）；加 `white-space: nowrap` | core-ui：LabelWidget(inline)。Rust：原本无 id，不影响 |
|  4 | `<text-input id="global_search">` | 48 | `<input id="global_search">` | 189 | 标签无对应控件；`<input>` → TextInputWidget | core-ui：VBox(空盒)→TextInputWidget。Rust：id 不变 |
|  5 | `<hbox id="tab_host">` | 50 | `<div id="tab_host" class="tabbar">` | 191 | 同 #2 | 同上 |
|  6 | `<label id="tab_placeholder">` | 51 | `<label id="tab_placeholder" class="tab-placeholder">…</label>` | 192 | **原就是 `<label>`，保留**；只补 class 拿颜色 + 内部文本 | core-ui：不变。Rust：id 不变 |
|  7 | `<separator class="divider">` | 53 | `<div class="divider" />` | 194 | **第二轮修正**：`<separator>` 不画 CSS background（§2.6）→ 改用 1px 的 div | core-ui：SeparatorWidget → 1px 实心 div，颜色走项目 token。Rust：无 id |
|  8 | `<hbox id="command_host">` | 54 | `<div id="command_host" class="commandbar">` | 195 | 同 #2 | 同上 |
|  9 | `<label id="nav_hint">` | 55 | `<label id="nav_hint" class="nav-hint">…</label>` | 196 | **原就是 `<label>`，保留**；补 class 拿颜色 | 同上 |
|  10 | `<separator class="divider">` | 57 | `<div class="divider" />` | 194 | 同 #7 | 同上 |
|  11 | `<hbox id="address_host">` | 58 | `<div id="address_host" class="addressbar">` | 199 | 同 #2 | 同上 |
|  12 | `<panel id="breadcrumb">` | 59 | `<label id="breadcrumb" class="breadcrumb" />` | 200 | **第七轮（架构裁定）**：面包屑是**文本投影**不是自绘画布 → `<label>`；且原来是**空 `<div>`**（显示不出任何东西，是真缺口）→ Rust 用 `ui_label_set_text(w, L"…")`（UTF-16）填 | core-ui：VBox→LabelWidget（inline）。Rust：id 不变 |
|  13 | `<text-input id="address">` | 60 | `<input id="address">` | 201 | 同 #4 | 同上 |
|  14 | `<text-input id="search">` | 61 | `<input id="search">` | 202 | 同 #4 | 同上 |
|  15 | `<separator class="divider">` | 63 | `<div class="divider" />` | 194 | 同 #7 | 同上 |
|  16 | `<hbox id="infobar_host">` | 64 | `<div id="infobar_host" class="infobar" />` | 205 | 同 #2 | 同上 |
|  17 | `<hbox class="main-area">` | 65 | `<div class="main-area">` | 206 | 同 #2 | 同上 |
|  18 | `<panel id="nav_host">` | 66 | `<custom id="nav_host" class="nav-host" />` | 207 | **宿主标签通用规则（交付总监第四轮裁定）**：导航树由 Rust 自绘（D07）→ 必须 `<custom>`，与列表宿主同源；`<div>` 会尺寸 0 → 左侧导航区空白**且无任何报错** | core-ui：VBox→CustomWidget（空的自绘画布）。Rust：id 不变，`ui_page_on_widget_mount(page,"nav_host",…)` 取句柄 |
|  19 | `<vseparator class="divider">` | 67 | `<div class="vsep" />` | 208 | `<Separator>` 只有水平形态（`controls.h:200`），竖线用 1px 宽 div：`width:1px; background:var(--app-divider); align-self:stretch; flex-shrink:0` | core-ui：VBox(空盒)→1px 竖条。Rust：无 id，无影响 |
|  20 | `<panel id="list_host">` | 68 | `<custom id="list_host" class="list-host" />` | 209 | **架构裁定（第三轮）**：列表由 Rust 自绘，宿主必须是 `<custom>` —— `ui_custom_on_draw` 内部是 `As<ui::CustomWidget>(w)`，对 `<div>` 会**静默 no-op**；且运行时 `ui_custom()` 造的控件没有 CSS 级联结果 → 无 flex/width/height → 尺寸 0 → 列表区空白。`<custom>` 则「CSS width/height 走 ApplyCommonStyle 自然生效」（`widget_factory.cpp:1114-1118`）→ `.list-host { flex: 1 }` 生效 | core-ui：VBox→CustomWidget（空的自绘画布）。Rust：id 不变，`ui_page_on_widget_mount(page,"list_host",…)` 取句柄 |
|  21 | `<vseparator class="divider">` | 69 | `<div class="vsep" />` | 208 | 同 #19 | 同上 |
|  22 | `<panel id="preview_host">` | 70 | `<div id="preview_host" class="preview-host">`（**容器**） | 211 | **第七轮（架构裁定）**：预览是「图片（Rust blit）+ 元信息（文本）」的**混合** → 按规则拆成 `<div>` 容器 + 里面的 `<custom>` 子宿主 + 预先声明的 `<label>` 元信息行 | core-ui：VBox 容器。Rust：id 不变 |
|  22a | （新增） | — | `<custom id="preview_image" class="preview-image" />` | 212 | 图片位 —— **Rust 用绘制指令画**（`ui_bitmap_create` → `ui_draw_bitmap_handle`）→ 必须 `<custom>`；`height: 128px` 对应 docs/07 §3 的「图片 ≤128px」上限，fit 缩放由 Rust 算 | core-ui：CustomWidget（空的自绘画布）。Rust：`ui_page_on_widget_mount(page,"preview_image",…)` 取句柄后挂 draw 回调 |
|  22b | （新增） | — | `<label id="preview_host_name" class="preview-meta" />` | 213 | 元信息行（文件名）—— 文本投影 → `<label>`，Rust 用 `ui_label_set_text` 填 | core-ui：LabelWidget。Rust：id 见 §5 R4 的句柄清单 |
|  22c | （新增） | — | `<label id="preview_host_size" class="preview-meta" />` | 214 | 元信息行（大小）—— 同上 | 同上 |
|  22d | （新增） | — | `<label id="preview_host_channel" class="preview-meta" />` | 215 | 元信息行（通道状态）—— 同上 | 同上 |
|  22e | （新增） | — | `<label id="preview_host_state" class="preview-meta" />` | 216 | 元信息行（状态）—— 同上 | 同上 |
|  23 | `<separator class="divider">` | 72 | `<div class="divider" />` | 194 | 同 #7 | 同上 |
|  24 | `<hbox id="status_host">` | 73 | `<div id="status_host" class="statusbar">` | 220 | 同 #2 | 同上 |
|  25-30 | `<text id="status_*" class="status-segment">` ×6 | 74-79 | `<span id="status_*" class="status-segment" />` ×6 | 221-226 | 状态栏片段，短、单行、由 Rust 运行时填字 → `<span>`；配 `white-space: nowrap` 防长值折行 | core-ui：VBox(空盒)→LabelWidget(inline)。Rust：id 不变，`ui_label_set_text` 可用 |

### 3.2 `assets/preview.uix`

| # | 原标签 | 原:行 | 现标签 | 现:行 | 取舍理由 | 影响面 |
|---|---|---|---|---|---|---|
|  1 | `<vbox>` | 20 | `<div class="root">` | 97 | 同 main #1 | 同上 |
|  2 | `<hbox class="preview-toolbar">` | 21 | `<div class="preview-toolbar">` | 98 | 横排靠 `.preview-toolbar { flex-direction: row; align-items: center; }` | core-ui：VBox→HBox |
|  3 | `<text id="preview_title">` | 22 | `<span id="preview_title" class="preview-title">预览</span>` | 99 | 工具条标题，短、单行 → `<span>` | core-ui：VBox→LabelWidget(inline)。Rust：id 不变 |
|  4 | `<separator />` | 24 | `<div class="divider" />` | 101 | **第二轮修正**：`<separator>` 不画 CSS background（§2.6）→ 改用 1px 的 div | core-ui：SeparatorWidget → 1px 实心 div，颜色走项目 token |
|  5 | `<panel id="preview_body">` | 25 | `<custom id="preview_body" class="preview-body" />` | 102 | **第七轮**：图片位由 Rust blit（同 main 的 `preview_image`）→ 按同一条规则应是 `<custom>`。**架构未点名本文件，按同一条规则推出；改回 `<div>` 只是一行** | core-ui：VBox→CustomWidget。Rust：id 不变 |
|  6 | `<text id="preview_hint">` | 26 | `<label id="preview_hint" class="preview-hint">…</label>` | 103 | 提示行需要**占满宽度并可换行**（Rust 写入的通道名可能较长）→ `<label>` + `display: block` | core-ui：VBox(空盒)→LabelWidget(block/stretch)。Rust：id 不变 |
|  7 | `<text id="preview_channel">` | 27 | `<label id="preview_channel" class="preview-hint" />` | 104 | 同 #6 | 同上 |
|  8 | `<text id="preview_size">` | 28 | `<label id="preview_size" class="preview-hint" />` | 105 | 同 #6 | 同上 |
|  9 | `<text id="preview_state">` | 29 | `<label id="preview_state" class="preview-state" />` | 106 | 状态行需占满宽度 → `<label>` + `display: block` | 同上 |

### 3.3 `assets/about.uix`

| # | 原标签 | 原:行 | 现标签 | 现:行 | 取舍理由 | 影响面 |
|---|---|---|---|---|---|---|
|  1 | `<vbox>` | 14 | `<div class="root">` | 59 | 同 main #1 | core-ui：VBox→VBox |
|  2 | `<text class="heading">` | 15 | `<label class="heading">…</label>` | 60 | 标题虽短，但按「占满宽度」语义统一用 `<label>` + `display: block` | core-ui：VBox(空盒)→LabelWidget(block) |
|  3 | `<text class="row">`（首条） | 16 | `<label class="row">…</label>` | 61 | 条目是长句（最长达 40+ 汉字），**必须占满宽度才会折行**；若保持 inline，宽度=自然文字宽度 → 横向溢出窗口而不换行 | 同上 |
|  4 | `<separator />` | 17 | `<div class="divider" />` | 62 | **第二轮修正**：`<separator>` 不画 CSS background（§2.6）→ 改用 1px 的 div | core-ui：SeparatorWidget → 1px 实心 div，颜色走项目 token |
|  5 | `<text class="row">` ×10 | 18-27 | `<label class="row">…</label>` ×10 | 63-72 | 同 #3 | 同上 |
|  6 | `<separator />` | 28 | `<div class="divider" />` | 62 | 同 #4 | 同上 |
|  7 | `<text class="row">`（致谢） | 29 | `<label class="row">…</label>` | 74 | 同 #3 | 同上 |

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

`:root { … }` **必须拼进 `<style>…</style>` 块内部**，**绝不能放在源码顶层**。

> **第八轮更正注入形态**：Rust 实现走**形态甲** —— 在 `.uix` 源码**最前面插入一个完整的 `<style>` 块**（**不是**「末尾追加」）。
> **顶层块顺序无关** —— 解析器按标签名分派到各自容器（`<window>` / `<template>` / `<style>` 谁先都行）→ 插最前合法，且**天然保证「注入的是第一个 `:root`」**。
> 形态乙（插进**第一个** `<style>` 块的**块首**）同样满足口径 ⑤ —— 两者都要求「**此前不能有 `:root`**」。
>
> ⚠️ **方向修正**：我第三轮写的「末尾追加」和架构侧上一版契约写的「必须在 source order 上**最后**（后者覆盖前者）」**都是反的**。实际是**先定义者赢**：
> `LookupVar`（`value.cpp:572-579`）**返回第一个命中**；`cascade.h:41` 注释逐字「**Returns first set encountered.**」；而 `:root` 规则在 `cascade.cpp:124-125` 被**整条跳过**（根本不进 CSS 级联）。
> ⇒ **注入排在 `.uix` 自己的 `:root` 之后会被静默吃掉** —— 症状是「切主题颜色不变」，**零报错**。

**六条硬口径**（与 `spec/ffi-core-ui.md` §4.3.2.1 / `spec/crate-layout.md` §4.3.1 逐字一致）：

| # | 口径 | 违反后果 | 依据 |
|---|---|---|---|
| 1 | **拼进 `<style>` 块内** —— Rust 统一**在源码最前面插入**一个完整的 `<style>` 块 | 裸放源码顶层 → `sfc_parser.cpp:260-266` 判 "unexpected content at top level" → `doc.ok=false` → `page_api.cpp:249-255` 返回 "SFC parse failed" → **整页不加载，三个窗口一个都出不来** | `sfc_parser.cpp:260-266` |
| 2 | **只允许 CSS 变量** —— `:root` 里只写 `--app-*` | 普通声明（`background`/`color`/`font-family`）被**静默丢弃**（`:root` 是伪类，在普通样式计算里被整条跳过） | `cascade.cpp:124-126` |
| 3 | **根样式走 `.root` class** | `window { … }` 匹配 **0 个节点**（`<window>` 不产生 widget）→ 不报错，只是**不生效** | `sfc_parser.cpp:285-296`、`cascade.cpp:112-145` |
| 4 | **Rust 侧加载前自检（四条断言）**：**位置**（落在 `<style>` 内）/ **顺序**（注入的是**第一个** `:root`）/ **内容**（只含 `--app-*`）/ **键集**（无 `--app-accent`） | 任一不满足 → fail-fast；否则表现为「颜色不对 / 切主题不变 / 库色混入」，且**零报错** | 同 1 / 2 / 5 / 6 |
| 5 | **注入必须是「第一个」`:root`** | core-ui 是**先定义者赢** → 注入排在 `.uix` 的 `:root` 之后会被**静默吃掉**（症状：切主题颜色不变，零报错） | `value.cpp:572-579`、`cascade.h:41`、`cascade.cpp:124-125` |
| 6 | **键集排除 `--app-accent`** | 它是 D14 的**回退源值**（数据字段，不是 CSS 变量），由 Rust 读出后直接喂 `ui_theme_set_accent_hex()`；混进 `:root` 会让「库 token `--accent`」与「项目 `--app-accent`」两套值并存 → 库控件与自绘区颜色分裂 | 架构 D14；`assets/themes.json` `revision.decision` |

> **当前前提成立**：实测三个 `.uix` 剥注释后**生效 `:root` 规则数 = 0**（`main.uix` 那 3 处命中全在 HTML 注释里）→ 现在不会出问题。
> 这是把一个**将来会静默咬人**的缺口提前钉住（Rust 侧自检四条断言见口径 ④）。

**三种失败模式（都是静默或误报型的）**：

| 失败模式 | 现象 | 定位 |
|---|---|---|
| 顶层注入 | "SFC parse failed" → 一个窗口都出不来 | 看 `ui_page_last_error()` |
| 完全没注入 | **静默**：界面能出来，但所有 `var(--app-*)` 解析成空串（`cascade.cpp:79-81` → `ParseColor("")` 失败）→ 全是库默认色：窗口底 = 库 `windowBg` 纯白 `#FFFFFF` / 文字 = 库 `kBtnText()` **`#242424`**（`controls.cpp:61-68`）/ **分隔线直接消失**（分隔线已是 `<div class="divider">`，`--app-divider` 未定义 → `background` 解析失败 → 画不出来；**不会再出现库色 `#D1D1D1`**） | 比对颜色与 `themes.json`；看不到分隔线本身就是信号 |
| 注入晚于 `ui_page_load_string` | 同上，静默回落到库默认色 | 比对时序 |

→ 三条都已写进 `assets/themes.json` 的 `revision.inject_failure_modes` 与 `docs/13` §0 上机前准备清单。

### 4.3 逐 token 最终表（17 个）

**决策口径**：**改名避让**（加 `--app-` 前缀）16 个 + **改用库 token** 1 个（accent）。**保留原名 0 个。**

| # | 原名 | 新名 | 决策 | 理由 |
|---|---|---|---|---|
|  1 | `--bg` | `--app-bg` | 改名避让 | 59 |
|  2 | `--content-bg` | `--app-content-bg` | 改名避让 | 60 |
|  3 | `--sidebar-bg` | `--app-sidebar-bg` | 改名避让 | 61 |
|  4 | `--toolbar-bg` | `--app-toolbar-bg` | 改名避让 | 62 |
|  5 | `--text` | `--app-text` | 改名避让 | 63-72 |
|  6 | `--text-secondary` | `--app-text-secondary` | 改名避让 | 62 |
|  7 | `--divider` | `--app-divider` | 改名避让 | 74 |
| 8 | `--accent` | `--app-accent`（**仅回退源值**） | **改用库 token**（CSS 侧读 `var(--accent)`，**不读** `var(--app-accent)`） | accent 是**唯一有官方通道**的 token：`ui_theme_set_accent_hex()` 会写进库 palette 并同步到库 token `--accent`（`ui_api.cpp:2571-2587` → `theme::SetAccent` → `RefreshAllPageThemes` → `RebuildThemeVars`），且**跨 `ui_theme_set_mode` 保留**（`theme.h:435-439`）。若自定义 `--app-accent`，库自绘控件（primary 按钮 / 焦点环 / 滑块 / 进度条）仍是库默认蓝 `#3971EC`，与自绘区的 `#2563EB` 撞成两种蓝。<br>**第三轮补充（交付总监 D14）**：core-ui **根本不读系统强调色** —— 全仓无 `GetSysColor` / `UISettings` / `AccentColor`，`theme.h:446-455` 的 `SetAccent` 只接受外部传入值 → docs/07 §4「跟随系统 `AccentColor`」**只能由 Rust 实现**：读注册表 `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\AccentColor`（DWORD，**BGR 序**）→ 转 hex → `ui_theme_set_accent_hex()`；读不到才回退到 `--app-accent`。**因此 `--app-accent` 的定位是「读不到系统色时的回退源值」，不是主源值。**<br>**并且它是【数据字段，不是 CSS 变量】** —— Rust 读出后直接喂 `ui_theme_set_accent_hex()`，**不进 `:root`**（见 §4.2 口径 ③）；任何 `.uix` 里都不得出现 `var(--app-accent)`。 |
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

#### ⚠️ 重建页面的副作用：32 个句柄全部失效（第二轮由 QA 查出；第七轮由 27 更新为 32）

`ui_page_load_string()` 返回的是**一个全新的 page**。旧 page 里的
**32 个 `ui_widget_find_by_id` 句柄（main 26 + preview 6）全部失效**，
且旧 page 必须显式 `ui_page_destroy`。

→ **切主题后 Rust 侧必须做三件事**（已登记为 §5 R4）：

1. `ui_page_destroy(旧 page)` —— 不销毁就是句柄 + 内存双泄漏；
2. **重新 `ui_widget_find_by_id` 取全部 32 个句柄**；
3. 重新挂载回调与自绘状态（列表、导航、面包屑、预览）。

→ 判据：切主题后如果状态栏不再更新、列表空白、预览不刷新，**第一嫌疑就是旧句柄没重取**。

#### ⚠️ 旧句柄的**确切失效形态**：不是「写野内存崩溃」，而是「静默无效」

> **第十一轮更正（QA O-2 指出，我独立复核上游源码后成立）**：
> 我原先写的「旧句柄 = 写野内存 = **静默崩溃 / 花屏**」**机制是错的** —— 结论对、机制错。
> 实际形态是**静默无效**（API 返回成功、界面毫无变化），**不是崩溃**。

| 事实 | 依据 |
|---|---|
| 句柄表存的是 **`shared_ptr<Widget>`**（不是裸指针） | `handle_table.h:31`（`unordered_map<uint64_t, shared_ptr<Widget>> handles_`）、`handle_table.cpp:5-10` |
| `ui_page_destroy` **只**把 `PageEntry` 从注册表摘掉，**不碰句柄表** | `page_api.cpp:561-563`（`Registry().Erase(p)` → `entries_.erase(id)`） |
| 句柄表**只在 `Context::Shutdown` 时**才 `Clear()` | `ui_context.cpp:125`（`handles.Clear()`） |
| 旧句柄查表时**仍能查到**那个 widget（`LookupRaw` 命中） | `ui_api.cpp:42`（`W(h)` = `Ctx().handles.LookupRaw(h)`） |

⇒ **旧句柄指向的是一个「还活着、但已脱树」的孤儿控件**：
句柄表里的 `shared_ptr` 让它**不被析构**，但它**不在任何 page 的控件树里**了。
往它写属性 = **写成功、API 不报错、界面毫无变化**（不是崩溃、不是花屏）。

⚠️ **代价是泄漏**：句柄表不随 page 销毁而清理 ⇒ **每次切主题，旧页的整棵控件子树
都被句柄表吊着不放**（32 个句柄 = 32 棵子树）。

→ **补一条处置**（Rust 侧）：切主题销毁旧 page 前，先对 32 个旧句柄逐个调
`ui_widget_destroy(old_handle)`（`ui_api.cpp:1886-1888` → `handles.Remove(handle)`）
把句柄表条目摘掉，控件才能真正释放。**顺序：先 `ui_widget_destroy` 摘句柄，再 `ui_page_destroy` 销毁页。**

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

> 第 2 步必须在第 3 步**之前或紧邻**：切 mode 会用**同一个 accent 基色**重新派生 hover/press/text 变体，**基色本身不变**（`theme.h:435-439` 的 `SetMode` → `ApplyAccent(AccentOverrideValue())`，用的是同一个 override 值）⇒ 顺序反了会拿到**上一次的基色**。
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
| R4 | **深浅色切换需重建页面**（不能只调 `ui_theme_set_mode`）；且重建后**必须 `ui_page_destroy` 旧 page + 重取全部 32 个 `ui_widget_find_by_id` 句柄（main 26 + preview 6）+ 在新 page 上重新注册 `ui_page_on_widget_mount` + 重挂回调**（新 page ⇒ 旧句柄全部失效；mount 钩子也不转移）<br>⚠️ **补一步防泄漏（第十一轮，QA O-2）**：销毁旧 page **之前**，先对 32 个旧句柄逐个调 `ui_widget_destroy(old_handle)`（`ui_api.cpp:1886-1888` → `handles.Remove(handle)`）把句柄表条目摘掉 —— 否则句柄表里的 `shared_ptr` 会把旧页整棵控件子树吊着不放（句柄表只在 `Context::Shutdown` 时 `Clear()`，`ui_context.cpp:125`）。**顺序：先摘句柄 → 再销毁页。** | §4.4（机制更正） | 主题切换处 |
| R10 | **剩余四条无元素引用的 class 规则**：`.cmd-button` / `.cmd-group`（标签条与命令栏的**绘制规范**，Rust 自绘时按此对齐）+ `.infobar-visible` / `.preview-collapsed`（两个**状态**的尺寸规范，Rust 需直接改高度 / 宽度，见 R7）。core-ui **没有运行时设置 class 的 C API**（`include/ui_core.h` 内无相关符号；`dynamicClasses` 只来自 `:class` 绑定）→ 这四条**不会由 `.uix` 渲染**。已在三个 `.uix` 里逐条加注释标明。<br>（`.tab-active` 已在第三轮删除，理由见 R6 —— 它额外踩了「不受支持的 `border-bottom`」和「命名空间混用」两个坑。） | 第二轮由 QA I7 查出；`src/ui/page/compiler.cpp:859`（dynamicClasses 来源） | Rust 侧 `src/ui/tabbar.rs` / `toolbar.rs` / `preview_pane.rs` |
| R11 | **Rust 侧加载前自检 = 四条断言**（架构第八轮扩为四条；口径与 `spec/milestones.md` **V1.12** 的四个坏源码构造一一对应）：<br>**① 位置** —— 每个 `:root` 必须落在某个 `<style>…</style>` 区间内。违反 → 裸放源码顶层 → `sfc_parser.cpp:260-266` 判 "unexpected content at top level" → `doc.ok=false` → `page_api.cpp:249-255` 返回 "SFC parse failed" → **三个窗口一个都出不来**。<br>**② 顺序** —— 注入的 `:root` 必须是**整张样式表里的第一个 `:root`**（整张表 = `<link>` CSS（在前）+ inline `<style>`（在后），`page_api.cpp:333-350`；本项目零 `<link>` → 等价于源码里第一个）。理由：自定义属性**先定义者赢**（`LookupVar` 返回第一个命中，`value.cpp:572-579`；`cascade.h:41`「Returns first set encountered.」；`:root` 规则不进 CSS 级联，`cascade.cpp:124-125`）⇒ 若 `.uix` 自带 `:root` 且排在注入块之前，**注入的值被静默忽略**（不报错、不告警，症状「切主题颜色不变」）。<br>**③ 内容** —— `:root` 内**每条声明**必须匹配 `--app-[a-z0-9-]+:`（即只允许 CSS 变量，且是项目命名空间）。违反 → `background`/`color`/`font-family` 被 `cascade.cpp:124-126` **静默丢弃**。<br>**④ 键集** —— `:root` 内**不得有 `--app-accent`**（它是 D14 的回退源值、数据字段，不进 `:root`），且**不得撞 core-ui 库的 34 个 token**（`compiler.cpp:1648-1658` 的 `kThemeKeys[]`；撞名会被 `RebuildThemeVars` erase+重写，`compiler.cpp:1647-1703`）。<br>**当前前提成立**：三个 `.uix` 剥注释后生效 `:root` 规则数 = 0 ✓（`main.uix` 那 3 处命中全在 HTML 注释里）<br>**静态侧已有门禁**：`scripts/gates/s13_token_namespace_check.py`（正式）+ `uix_tag_check.py`（预检）；V1.12 的四条在 `spec/milestones.md` | `sfc_parser.cpp:260-266`、`value.cpp:572-579`、`cascade.h:41`、`cascade.cpp:124-125`、`compiler.cpp:1647-1703` | Rust 侧加载前自检（`crates/zsdesk-coreui/src/lib/theme.rs`，并入 V1.12） |
| R5 | **宿主标签通用规则（交付总监第四轮裁定 + 架构第七轮细化）**：**这块矩形的内容，是不是 Rust 用绘制指令画出来的？**<br>· **是** → 标签必须是 **`<custom>`**（Rust 自绘画布；工厂保证 CSS 宽高生效）<br>· **否，是 core-ui 控件或文本投影** → 用 **`<div>` / `<label>`**<br>· **混合（既要 Rust 画图，又要有 core-ui 控件/文本）** → 拆成「**`<div>` 容器 + 里面的 `<custom>` 子宿主**」<br>理由：`<div>` 拿不到绘制回调（`ui_custom_on_draw` 内部是 `As<ui::CustomWidget>`，取不到就**静默 no-op**），且运行时挂载的控件**没有 CSS 级联结果** → flex 规则失效 → 尺寸 0 → **区域空白且无任何报错**。<br>**逐个判定（六处，判定来源见架构 D06 / D06-1 / D07 / D12）**：<br>· `list_host`（Rust 自绘文件列表，D12）→ **`<custom>`** ✔ 已改<br>· `nav_host`（Rust 自绘导航树，D07）→ **`<custom>`** ✔ 已改<br>· `preview_image`（main.uix 内，Rust blit 图片位，D06-1）→ **`<custom>`** ✔ 第七轮新增<br>· `preview_body`（preview.uix 内，Rust blit 图片位，D06-1）→ **`<custom>`** ✔ 第七轮改（架构第九轮确认）<br>· `breadcrumb`（**文本投影**：Rust `ui_label_set_text` 写路径文字）→ **`<label>`** ✔ 已改（原来是**空 `<div>`**，显示不出任何东西）<br>· `preview_host`（**混合容器**：图片 + 4 行元信息）→ **`<div>`** 保持，内含 `<custom id="preview_image">` + 4 个 `<label class="preview-meta">`<br>`<custom>` 的 CSS 宽高**会**生效（`widget_factory.cpp:1114-1118`「CSS width/height 走 ApplyCommonStyle 自然生效」）→ `.list-host { flex: 1 }` / `.nav-host { width: 220px }` / `.preview-image { height: 128px }` 均生效。<br>⚠️ **`preview_host` 不能整体改成 `<custom>`**：`<custom>` 是**空的自绘画布**，改成它就**丢掉那 4 个 `<label>` 元信息行**。<br>⚠️ **「自动回填」只覆盖一半**（架构第四轮澄清）：`ui_page_on_widget_mount` 的钩子在**新 page 上会重新触发** → **自绘控件的绘制回调自动回填 ✅**；但 Rust 自己持有的 `ui_widget_find_by_id` 句柄**不会自动更新 ❌** → 必须**主动重取全部 32 个句柄**（旧句柄 = 指向「还活着但已脱树」的孤儿控件 → 往它写属性**静默无效**，**不报错也不崩**，见 §4.4 的机制更正）。<br>⚠️ **做不到的一条**：core-ui **没有读回 CSS 变量的 C API**（导出面只有 `ui_widget_set_bg_color` / `set_text_color` 这类写接口，无 `get_style`/`get_var`/`computed`）→ 「压根没注入」这个静默项只能靠**离屏渲染取一个已知像素**与 `themes.json` 比对（一次性冒烟，**不进启动路径**）；**不要为此造读回 hack**。 | 架构裁定（`spec/ffi-core-ui.md` §4.3.4）+ 交付总监第四轮裁定 | 虚拟列表 / 导航树 / 预览 |
| R6 | **`.tab-active` 规则已从 `.uix` 删除**（第三轮，交付总监裁定）：它同时踩三个坑 —— ① core-ui 不支持 `border-bottom`（`widget_factory.cpp:631-653` 只认 border / border-color / border-width / border-radius）② `var(--accent)` 是库 token，与其余 `--app-*` 混用两个命名空间 ③ **没有任何元素使用这个 class**（标签条由 Rust 自绘）。删除后不丢设计意图：docs/07 §2 的「选中标签底部 2px accent 指示条」由 **Rust 自绘**实现，或在 `.uix` 里用 2px 高的 `<div>` | `widget_factory.cpp:631-653`；`docs/07` §2 | `src/ui/tabbar.rs` 自绘 |
| R7 | **`.infobar` 默认 `height: 0px`**（收起），`.infobar-visible` 36px；两者都无 id 切换机制 → Rust 侧需 `ui_widget_set_visible`（`<div>` 支持 `visible` 静态属性，`widget_factory.cpp:1329-1332`）或直接改高度 | — | 信息条状态机 |
| R8 | **✅ 已裁决并已落地（交付总监 I4，采纳选项 (c)）**：`.preview-title` / `.heading` 的 `font-weight: 600` 是空声明（core-ui 对 LabelWidget 只认 `bold` / `700`，`widget_factory.cpp:1392-1395`）→ 标题本会按常规字重渲染、层级丢失；而 docs/07 §4 明写「字重仅 400 / 500」，**500 同样是空声明** → 改 `bold` 超上限、保持 `600` 也超上限 ⇒ 采纳 (c)：**删掉字重、把标题字号提到 14px，层级靠【字号 + 颜色】**（14px + `--app-text` vs 12px + `--app-text-secondary` vs 13px + `--app-text`）。**已落**：`.heading` 16px→14px 且删 `font-weight`；`.preview-title` 补 `font-size: 14px` 且删 `font-weight`。`docs/13` F3 / F6 / §9-8 同步 | `widget_factory.cpp:1392-1395`；`docs/07` §4 | 已完成（`assets/about.uix` / `assets/preview.uix`） |
| R12 | **「面包屑可见」是 Rust 侧契约，不是 `.uix` 能单独保证的事** —— `#breadcrumb` 在 `.uix` 里是**空的 `<label>`**（元素只有壳、没有文字），**Rust 必须用 `ui_label_set_text(w, L"…")` 投影当前路径**，否则地址栏那一行**永远空白**（`<label>` 无默认文本，`LabelWidget` 只画 `SetText` 进去的内容）。<br>**两种等价写法**（二选一，Rust 侧二选一即可，**不要同时用**）：<br>· **A（推荐）直接写控件**：`ui_label_set_text(find_by_id(page,"breadcrumb"), L"C:////Users////…////报告.docx")` —— 语义直白，路径格式完全由 Rust 控制。<br>· **B 走页面状态 + 插值**：`<label id="breadcrumb">{{ path }}</label>` + Rust 侧 `ui_page_set_text(page, "path", L"…")` —— 文本由 core-ui 的绑定机制刷新（`ui_page_set_text` 在 `include/ui_core.h:1937`）。⚠️ B 依赖 `<script>` 的 `data()` 与插值绑定都正确装配，比 A 多一层耦合，**首版建议走 A**。<br>**验收对应 M1 的 V1.16**（「进入目录后地址栏显示当前路径」）⇒ **Rust 侧漏了这一步，V1.16 必挂**，且现象是「整行空白、零报错」（与 §4.4 的旧句柄失效同属静默类）。 | `include/ui_core.h:1134`（`ui_label_set_text`）、`:1937`（`ui_page_set_text`）；`spec/milestones.md` V1.16 | `crates/zsdesk-ui/src/shell_page.rs`（写控件）+ `crates/zsdesk-app/src/app.rs`（状态投影） |
| R5b | **「`preview_host` 里那张图是谁画的」= Rust 在 `<custom id="preview_image">` 里 blit**（架构 **D06-1**）—— 不是 core-ui 的 `<gh-img-view>` 控件：<br>· **D06-1 的机制**：`ui_bitmap_create(pixels, w, h, stride)` → `ui_draw_bitmap_handle(ctx, bitmap, x, y, w, h)`（回调签名 `UiCustomDrawCallback(UiWidget, UiDrawCtx, UiRect, void*)` **带 draw ctx**，头文件该段注释即 "Drawing API (use inside UiCustomDrawCallback)"）→ **缩放由 Rust 自己算 fit**。像素管线不变：**WIC 解码 → BGRA 预乘 → `ui_bitmap_create`**（`spec/ffi-core-ui.md` §7.1 的像素格式章节继续有效）。<br>· **为什么不用 `<gh-img-view>`**：它的喂数 API（`ui_gh_img_view_begin` / `set_tile` 等瓦块金字塔喂数）在 **`SPEC.md` §8 第 7 条是明确的 out-of-scope**。<br>· **为什么不用 `ui_image_view()` 当子控件**：它是**运行时创建的控件** → **拿不到 CSS**（`ApplyCommonStyle` 全仓只有 `compiler.cpp:920` 与 `widget_factory.cpp:1363` 两个调用点）→ 尺寸只能 `ui_widget_set_size` 手算 px；且它作为**运行时插入的子控件**如何参与父容器布局，core-ui **没有公开承诺** → 风险不可控。声明式 `<img>` 也不行（`src` 是**资产 key**，**无运行时改 src 的 API**）。<br>· **代价（必须承认）**：缩放 / 平移 / 滚动要自己实现（**首版只要 fit**）；M3+ 要交互式缩放再走 `ui_image_gdi_*` 族（9 个符号可绑定）。 | 架构 D06-1（`spec/ffi-core-ui.md` §4.3.4）；`include/ui_core.h:1472`/`:1476`；`SPEC.md` §8 第 7 条 | `crates/zsdesk-ui/src/preview/` |
| R9 | **`<input>` 的 CSS `background` 会被采纳**（原先担心的「控件自绘底色盖掉 CSS」不存在）：`TextInputWidget::OnDraw` 优先用 `bgColor`（`controls.cpp:1052-1063`，`customBg = bgColor.a > 0` 时「use it as-is (no state variation)」）→ `.global-search` / `.address` / `.search` 的 `background: var(--app-content-bg)` **生效**；代价是这三个输入框**没有 hover / focus 的底色变化**（只有边框变化）。另外未设 `border-width`/`border-color` 时会走 WinUI 双色边框 + 聚焦 accent 下划线 | `controls.cpp:1045-1085` | 已知取舍，无需改动；真机确认边框观感 |

---

## 6. 标签自检结论

**执行器（可复跑，不是一次性结论）**：

```bash
python3 scripts/gates/uix_tag_check.py        # 标签 / D-15 / SFC / id / 模板体元素数 / 硬编码色 / S13 预检 / id 清单一致性
python3 scripts/gates/theme_contrast_check.py  # G05 真实渲染关系对比度 + G13 间距刻度
python3 scripts/gates/s13_token_namespace_check.py  # S13 正式门禁（QA 域）
python3 scripts/gates/ui_id_manifest_check.py       # id 清单 == 实际 id 集合（QA 域）
```

两条都是**失败即停**（退出码非 0）的 L1 静态门，不做编译、不做渲染。

> ⚠️ **第一轮我贴的输出是错的**（QA B3）：贴的是临时脚本的输出，与仓库里的
> `scripts/gates/uix_tag_check.py` 对不上（开标签+闭标签混算）。
> **第三轮按交付总监要求改成「命令 + 判定 + 退出码」三行式**（`contract-prose.md`
> §二.4：给指针不给倾倒）—— 详细输出一律交 CI 存档，避免"文档里贴的数和实跑对不上"
> 这类问题再次发生。判定条件见下方断言清单。

**`uix_tag_check.py`**（输出交 CI 存档；本节只给判定条件）：

```
$ python3 scripts/gates/uix_tag_check.py
判定：见下方断言清单，全部 PASS 时退出码 0
退出码：0
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
| id 集合：原有 **27 个（main 21 / preview 6 / about 0）一个没少没改**；第七轮新增 5 个（`preview_image` + `preview_host_*`）⇒ 现 32 个 | ✅ |
| **模板体**元素数：main **35**（原 30 + 第七轮新增 5）/ preview 9 / about 16（含 SFC 外壳则 39/12/19） | ✅ |
| 硬编码色值（G02）：三个 `.uix` 内零字面色值，全部走 `var(--app-*)` | ✅ |
| **S13 预检**（架构授权加在本文档脚本，正式门禁是 `s13_token_namespace_check.py`）：① 不定义库 34 token 任一；② `.uix` 内不定义 `--app-accent`；③ `themes.json` 内**允许** `--app-accent`；④ `var(--accent)` 等引用**不拦**；另 `.uix` 内不出现 `var(--app-accent)` | ✅ |
| **id 清单一致性**：本脚本的硬编码清单 == `assets/ui-ids.json`（单一事实源），且 `.uix` 内**无清单外的 id** | ✅ |

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

**主题门禁**（`theme_contrast_check.py`，输出交 CI 存档）：

```
$ python3 scripts/gates/theme_contrast_check.py
判定：110 对真实渲染配对全部 ≥4.5:1，且 spacing 越界数为 0 时退出码 0
退出码：0
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
| **I3** 切主题重建页面 → 27 个句柄失效（**当轮值；现为 32 个句柄**） | **QA 正确，补漏了** | docs/12 §4.4 加「重建页面的副作用」专节（destroy + 重取 + 重挂三步）；§5 R4 改写；themes.json `revision.switch` 同步 |
| **I4** docs/13 F3 与 F6 自相矛盾 | **QA 正确** | **已全部落地**：`.heading` / `.preview-title` 删 `font-weight: 600`、字号按 docs/07 §4 提到 **14px**（选项 (c)，交付总监 I4 裁决）；`docs/13` F3 = 「**14px、不加粗**，靠字号 + 颜色做层级」、F6 = 「**已裁决**」、§9 第 8 条 = 「**已结案**」—— 三处口径一致，`qa_uix_verify.py` 的两条 `font-weight: 600` WARN 随之消失 |
| **I5** 对比度门禁验了不存在的组合 | **QA 正确** | 脚本重写为按 **DOM 真实渲染关系**计算（元素 color × 最近祖先-or-self background），实测 55 个文字元素 × 2 主题 = 110 对，最低 5.78:1 / 6.75:1，与 QA 独立复算一致 |
| **I6** `scripts/gates/` 未纳入版本控制 | **QA 正确** | 已 `git add`（连同 docs/12、docs/13） |
| **I7** `.tab-active` 三重失效 | **QA 正确，前两点我漏了**（`border-bottom` 不支持我已知，命名空间与「无元素引用」是我漏的） | 三个 `.uix` 里给 `.tab-active` / `.cmd-button` / `.cmd-group` 加注释标明「Rust 自绘参照规范，当前不生效」+ 说明 `var(--accent)` 是库 token 且刻意分属两个命名空间；docs/12 §5 新增 **R10** 登记 |
| **S1** 元素总数口径把 SFC 外壳算进去了 | **QA 正确**，且我第一轮 §0 与 §3 表自相矛盾 | 口径统一为**模板体元素数 30 / 9 / 16**（**当轮值；现为 35 / 9 / 16**，含 SFC 外壳是 39/12/19）；门禁脚本同步改为只数模板体 |
| **S2** docs/11 §3「ui_debug_* 71 个」/ §6「value.h:91」 | **部分正确**。71 是**导出的 `UI_API ui_debug_*` 声明数**（`grep -c "UI_API.*ui_debug_"` = 71 ✓），头文件内含该前缀的共 73 处（2 处在注释里）；`value.h:91` 是注释行、声明在 `:93` | docs/11 已改为写明计数口径（71 个导出声明 / 全文 73 处提及）与 `value.h:91-93` |
| **S3** docs/11 §1.3 `input` 行漏了子类型 | **QA 正确** | docs/11 §1.3 已补 `type="radio"/"range"/"number"/"password"`（`widget_factory.cpp:1207-1283`） |
| **S4** `.root` 的 `font-family` 对 LabelWidget 无效 | **QA 正确** | 已在三个 `.uix` 的 `.root` 上方加注释「设计记录，字体由 `ui_theme_set_default_font` 决定」 |
| **S5** 根 `<div class="root">` 铺满窗口可结案 | **接受** | 已核实 `ui_window.cpp:1533` `root_->rect = {0,0,size.width,size.height}`；docs/13 §9 该条改为**已结案**（QA 提供的行号，与我的独立查证一致） |

---

## 9. 第三轮（I4 裁决 + 架构裁定 + QA 第二轮）

| 来源 | 事项 | 我的处理 |
|---|---|---|
| **交付总监 I4 裁决**（采纳选项 (c)） | 标题字重：`font-weight: 600` 是空声明（core-ui 只认 `bold`/`700`）→ 两个标题不加粗；而 docs/07 §4 写「字重仅 400/500」，**500 同样是空声明** → 两条路都违反文档 | **两行精确改法已落**：<br>· `assets/about.uix` `.heading`：删 `font-weight: 600`，`font-size: 16px` → **14px**<br>· `assets/preview.uix` `.preview-title`：删 `font-weight: 600`，补 `font-size: **14px**`<br>层级改由**字号 + 颜色**承担（14px + `--app-text` vs 12px + `--app-text-secondary` vs 13px + `--app-text`） |
| **交付总监 accent 定位**（D14） | core-ui 全仓无 `GetSysColor`/`UISettings`/`AccentColor` → **根本不读系统强调色**；docs/07 §4「跟随系统 AccentColor」只能由 Rust 实现 | `assets/themes.json` 的 `revision.decision` 改写为「accent 由 Rust 读系统 `AccentColor` 后 `ui_theme_set_accent_hex()` 写入**库 token** `--accent`；`.uix` 一律读 `var(--accent)`；**`--app-accent` 仅作读不到系统色时的回退源值**」，并新增 `revision.accent_source`（注册表路径 + BGR 序 + 调用顺序）。§4.3 终表的 `--app-accent` 行同步标注「仅回退源值」 |
| **交付总监 `.tab-active`** | 死规则（不受支持的 `border-bottom` + 命名空间混用 + 无元素引用），要求删掉或改成有元素真正使用 | **已删**（R6）。设计意图不丢：docs/07 §2 的「选中标签底部 2px accent 指示条」由 Rust 自绘，或在 `.uix` 里用 2px 高的 `<div>` |
| **交付总监 B3** | §6 粘贴的输出复现不出来 | §6 改成**「命令 + 判定 + 退出码」三行式**（`contract-prose.md` §二.4：给指针不给倾倒），详细输出交 CI 存档 |
| **架构裁定 (b)** | 自绘宿主用 `.uix` 声明 `<custom>`，理由是**布局**（运行时 `ui_custom()` 造的控件没有 CSS 级联结果 → 无 flex/width/height → 尺寸 0 → 列表区空白），而不是"`<div>` 拿不到回调" | **已落**：`assets/main.uix` 的 `<div id="list_host">` → `<custom id="list_host" class="list-host" />`。R5 改写并补两条 Rust 侧注意：① `ui_page_on_widget_mount` 的钩子**挂在具体 page 上** → 切主题重建出新 page 后必须**在新 page 上重新注册**；② 其余三个 Rust 自绘宿主（`nav_host`/`breadcrumb`/`preview_host`）按同一推理应改 `<custom>`，但架构只裁定了列表宿主 → **我未擅自改，已上报待裁决** |
| **架构改口 ①** | `sfc_parser.cpp` 的路径是 `src/ui/uix/`，**不是** `src/ui/page/` | 我原文档写的就是 `src/ui/uix/sfc_parser.cpp`（无误），但 §2.1 现在补全了完整链条：`sfc_parser.cpp:285-297` 非自闭合 → `pushErr` → **`:373` `if (!doc.errors.empty()) doc.ok = false;`** → `page_api.cpp:249-255` 返回 "SFC parse failed" |
| **架构改口 ②** | `FirstPlainText` 的精确机制 | 已补进 §1：`FirstPlainText` 取元素的**第一个直接子节点** —— Text → 取内容；Interpolation（`{{ }}`）→ 返回 `""`（由 binding 处理）；Element → 返回 `""`。`text=` 只在 **`<menu>`** 上被读（`title` 别名，`compiler.cpp:641/644`） |
| **架构改口 ③** | `window { }` 的机制不是"落到根节点"，而是"**选择器匹配 0 个节点**"；另 `:root` 是伪类、普通样式计算里被**显式跳过**（`cascade.cpp:125`） | §2.3 已改口：`window` 选择器命中 0 个节点 → 必须显式给根 `<div>` 加 `class="root"` 再用 `.root { … }`（写法一样，但避免误以为 `window` 选择器有效）；并补「`:root` 块里**只能放 CSS 变量**，放 background/color/font 会被丢掉」 |
| **QA 第二轮** | B2 / I5 / M06 复核通过；剩 3 处（I4 落地、I3 补 C6、B3） | I4 与 B3 已落；I3 在 `docs/13` §6 补了 **C6**（切深色重建后 27 个句柄是否重取 —— **当轮值；现为 32 个句柄**） |
| **QA 第二轮（数字纠正）** | `docs/12` §2.4 的库默认文字色 `#424242` 有误 | **已改**：实为 `theme::kBtnText()` = **`#242424`**（light，grey::g14；`controls.cpp:61-68`），`#424242` 是 `theme::Current().contentText`（dark 是 `#D6D6D6`），LabelWidget 实际走 `kBtnText()` |
| **QA 第二轮（口径）** | 「元素总数」有三套值（docs/11 的 34/12/19、`uix_tag_check.py` 的 33/12/19、QA 的 30/9/16） | §0 已统一并写明口径：**一律指模板体元素数 30/9/16**（**当轮值；现为 35/9/16**），**不含 4 个 SFC 外壳标签**；三个数各自的口径也一并注明 |
| **QA 第三轮 X-2** | 我引的上游 demo 证据不实：写「`<separator>` 在 demo 里一次都没出现」，实际 demo 有 **3 处真实 `<separator/>`**（`demo/ui_demo.uix:663, 672, 687`）；且 `<div class="sep"/>` 是 **19 处**不是 10 处 | **已更正**（§2.6）：改成「demo 主力是 19 处 `<div class="sep"/>`（规则在 `demo/ui_demo.uix:203`），另有 3 处 `<separator/>` —— 那 3 处只能是库色，**印证而非反证**本条结论」。**不影响 B2 的结论**（那 3 处仍然拿不到项目 token） |
| **QA 第三轮 X-3** | `docs/11` 的 `ui_debug_*` 口径第二个数字不对（我写「全文 73 处含 2 处注释」） | **已更正**为可复现的三口径：**71 个导出声明**（`grep -c "UI_API.*ui_debug_"` = 71）/ 全文 **83 行**提到该前缀（`grep -c "ui_debug_"` = 83）/ 按唯一符号名会得 **73** 个 —— 其中 **2 个是注释里的前缀占位**（`ui_debug_*` 与 `ui_debug_post_*` 的 `*` 被正则截断），**真符号仍是 71**。<br>（QA 给的「72 个 / 1 个仅见于注释」两种口径我都复现不出来，故按上面这组可复现数字落笔） |
| **QA 第三轮 X-4** | `themes.json` 的 `inject_failure_modes` 第 2 条复制了错数字（`#424242` 文字 / `#D1D1D1` 分隔线） | **已更正**：`#424242` → **`#242424`**（库 `theme::kBtnText()`）；分隔线那半句改成「**分隔线直接消失**」—— 分隔线已是 `<div class="divider">`，`--app-divider` 未定义时 `background` 解析失败 → **画不出来**，**不会再出现库色 `#D1D1D1`**。`docs/13` M04 的同一处也同步 |
| **架构第三轮 ①** | `--app-accent` 的定位变了：**是 Rust 的回退源值（数据字段），不是 CSS 变量** → Rust 拼 `:root` 时**必须排除** | `themes.json` 的 `revision.decision` 与 `revision.inject` 口径 ③ 已写明；`docs/12` §4.2 加「四条硬口径」表、§4.3 accent 行补「数据字段，不进 `:root`」 |
| **架构第三轮 ③** | 注入形态：我的「形态乙（紧跟 `<style>` 开头）」vs Rust 的「形态甲（**末尾追加**完整 `<style>` 块）」（**当轮口径；第八轮已改为「最前面插入」**） | **已统一为形态甲**（两者都合法）。⚠️ **第五轮更正**：我原写的「乙要求该 `<style>` 是最后一个 `<style>` 块」**依据是反的** —— 真正的约束是「**此前不能有 `:root`**」（自定义属性先定义者赢：`value.cpp:572-579` + `cascade.h:41` + `cascade.cpp:124-125`）。已改，见 §4.2 口径 ⑤。⚠️ **第八轮再更正**：形态甲已由「末尾追加」改为「**最前面插入**」（架构裁定 QA 的甲A）—— 因为「末尾」与口径 ⑤「必须是第一个 `:root`」**在同一句里自相矛盾** |
| **交付总监第四轮裁定（宿主标签）** | 四个宿主逐个定「最终由谁画」：`list_host` Rust 自绘 → `<custom>`；`nav_host` Rust 自绘（D07）→ **`<custom>`**；`preview_host` core-ui 控件填充（D06）→ `<div>`；`breadcrumb` core-ui 控件填充 → `<div>` | **已落**：`assets/main.uix` 的 `<div id="nav_host">` → `<custom id="nav_host" class="nav-host" />`（`list_host` 上轮已改）。并在三个 `.uix` 头部与本节 R5 写下**通用规则**：「凡是由 Rust 自绘内容的宿主容器 → `<custom>`；由 core-ui 内置控件填充的宿主 → `<div>`」+ 四个宿主各自的判定来源（D06 / D07 / D12），以后新增宿主照此判定 |
| **交付总监第四轮裁定（id）** | `list_host` 这个 id **保持不变**（架构师举例写的 `file_list`/`list_list` 是占位名，不是对资产的要求；只要 `.uix` 的 id 与 Rust 侧查的 id 一致即可） | **不改**。`.uix` / `docs/12` §3.1 / §5 R5 三处口径一致 |
| **QA 第四轮 · docs/12 旧表述** | 同文件内三处口径打架：`:21` 复核栏写「I4 已落」，而 `:360`（R8）与 `:461`（§8 I4 行）还停在「等设计师」 | **已统一**：R8 行改为「**✅ 已裁决并已落地（交付总监 I4，采纳选项 (c)）**」并写明改了哪两个 class；§8 的 I4 行改为「已全部落地 + F3/F6/§9-8 三处口径一致 + 两条 WARN 随之消失」 |
| **QA 第四轮 · Δ3** | §4.4 的「切 mode 会用当前 accent override 重新派生」不准确 —— 重新派生只对**变体**成立，**基色不变** | **已改**：「切 mode 会用**同一个 accent 基色**重新派生 hover/press/text 变体，**基色本身不变**（`theme.h:435-439` 的 `SetMode` → `ApplyAccent(AccentOverrideValue())`，用的是同一个 override 值）⇒ 顺序反了会拿到**上一次的基色**」 |
| **QA 第四轮 · Δ4** | `themes.json` 的 `revision.switch` 缺「切主题时**重读系统 AccentColor 并重调 `ui_theme_set_accent_hex()`**」 | **已补**（同段还补了「自动回填只覆盖一半」的精确区分） |
| **QA 第四轮 · 硬约束** | `.uix` 里**不允许出现深浅色分支**（`.dark { }` / `[data-theme="dark"] { }`）：`selector.cpp:60-64` 的伪类只认 hover/pressed/focus/disabled/checked/root/drag-over，**没有主题态位** → 这类选择器**永远不命中**（静默失效） | **已记为 §2.7**（新增小节）。本轮三个资产**没有任何主题分支选择器** ✓ 符合 |
| **架构第四轮 · 两条 P0 核对** | `:root` 必须在 `<style>` 内 / `:root` 是唯一采集点 / 「不注入是静默的」/ 27 个句柄失效（**当轮值；现为 32**）/ `ui_page_reload` 帮不上忙 / `<separator>` 不画 background —— 六条逐条独立核对，**全部成立** | 无需改动；已在本轮各处引用其结论 |
| **架构第四轮 · demo 更正** | demo 的 3 处 `<separator/>` **全在 `<menu>` 内部**（菜单项之间的分隔线，用库色正好）；布局里的分隔线 demo 一律用 `<div class="sep"/>` | §2.6 已补这句澄清；并注明**本项目 7 处用的是项目 token `--app-divider`**，不是 demo 的库 token `var(--border-subtle)` |
| **QA 第六轮 · 形态乙的依据我写反了** | 我写「形态乙要求该 `<style>` 是**最后一个** `<style>` 块」→ **依据是反的**。正确依据是「**此前不能有 `:root`**」：自定义属性**先定义者赢** —— `LookupVar`（`value.cpp:572-579`）**返回第一个命中**；`cascade.h:41` 注释逐字「**(Returns first set encountered.)**」；而 `:root` 规则在 `cascade.cpp:124-125` 被**整条跳过**（根本不进 CSS 级联）⇒ 多个 `:root` 谁先出现谁赢，**后者不会覆盖前者**。<br>**策略不动**（形态甲仍是对的，Rust 不用解析已有 `<style>` 块） | **只改措辞**：§4.2 的形态甲/乙说明 + 四口径表 → **五条硬口径**（新增口径 ⑤「注入的 `:root` 必须是源码中第一个 `:root`」）；`themes.json` 的 `revision.inject` 同步加口径 ⑤；§9 第三轮记录里那句旧依据也标了更正 |
| **QA 第六轮 · Rust 自检第 3 条断言** | 建议 Rust 侧加载前自检加「注入的 `:root` 必须是源码中第一个 `:root`」，并入 V1.12 | **已登记为 §5 R11**；`themes.json` 的 `revision.inject` 口径 ⑤ 与 `docs/12` §4.2 口径 ⑤ 都写明了「并入 V1.12」。**当前前提成立**：实测三个 `.uix` 剥注释后生效 `:root` 规则数 = **0/0/0**（`main.uix` 那 3 处命中全在 HTML 注释里）→ 现在不会出问题，这是提前钉住将来会静默咬人的缺口 |
| **架构第七轮 · 宿主形态的三处裁定** | ① `nav_host` → `<custom>`（**第四轮已改**，本轮复核）；② `breadcrumb` → **`<label>`**（原来是**空 `<div>`** → **显示不出任何东西**，是真缺口 → Rust 用 `ui_label_set_text(w, L"…")` 投影文本）；③ `preview_host` → 保持 `<div>` 当**容器**，里面加 `<custom id="preview_image" class="preview-image"/>` + 若干**预先声明**的 `<label>` 元信息行 | **已落**：<br>· `assets/main.uix:185` `<div id="breadcrumb">` → `<label id="breadcrumb" class="breadcrumb" />`<br>· `assets/main.uix:196-201` `preview_host` 拆成「容器 + `<custom id="preview_image">` + 4 行 `<label class="preview-meta">`（`preview_host_name` / `_size` / `_channel` / `_state`）」，新增 CSS `.preview-image { height: 128px }`（对齐 docs/07 §3「图片 ≤128px」上限，fit 由 Rust 算）与 `.preview-meta { display:block; padding:4px 8px; color:var(--app-text-secondary); white-space:nowrap }`<br>· **元信息行 id 用 `preview_host_*` 前缀**（架构只给了 `preview_image`）：避免与 `assets/preview.uix` 的 `preview_channel` / `preview_size` / `preview_state` **同 id** —— 若将来 `preview.uix` 的根被 `ui_widget_add_child` 嵌进 `preview_host`，同树内重 id 会让 `FindById` 命中错的那个 |
| **架构第七轮 · preview.uix 的 `preview_body`** | 架构点名的是 main.uix 的三个宿主，未点名 `assets/preview.uix` 的 `preview_body`（图片位）。但按**同一条规则**（「这块矩形的内容是不是 Rust 用绘制指令画出来的」→ 是 → `<custom>`）+ D06-1（图片统一由 Rust 在 `<custom>` 里 blit）⇒ 它也应是 `<custom>` | **已改**（`assets/preview.uix:94` `<div>` → `<custom>`）。**这是我按规则推出的，架构未点名** —— 若裁决不同，改回 `<div>` 只是一行 |
| **架构第七轮 · 元素数与句柄数变了** | 这次是**主动新增元素**（不是标签替换）：`preview_host` 拆开 ⇒ main 模板体 30 → **35**，main 的 id 21 → **26**，全项目句柄 **27 → 32** | 已全量同步：`docs/12` §0 / §3.1 表（现:行 列已按资产实际行号**脚本重建**并逐行复核）/ §4.4 / §5 R4 / §6 断言清单；`docs/13` §0 / H1 / H6 / C6；`assets/themes.json` 的 `revision.switch`；`scripts/gates/uix_tag_check.py` 的期望值（35 / 9 / 16 与 id 清单） |
| **架构第四轮 · 做不到的一条** | 「加载后断言颜色是否等于 `themes.json`」在 API 层做不到 —— core-ui **没有读回 CSS 变量的 C API**（导出面只有 `ui_widget_set_bg_color` / `set_text_color` 这类写接口，无 `get_style`/`get_var`/`computed`） | **接受，不造 hack**：写进 §5 R5 —— 该静默项只能靠**离屏渲染取一个已知像素**与 `themes.json` 比对（一次性冒烟，不进启动路径） |

---

## 10. 第十一轮（QA 第七轮复核 + 架构第九/十轮裁定 + 基线修正）

| 来源 | 事项 | 我的处理 |
|---|---|---|
| **★ 架构第九轮 · 资产头部注释自相矛盾** | 架构查出 `assets/preview.uix` 的头部注释同时写着「四、preview_body 是图片位 → Rust blit → `<custom>`」与「五、preview_body 由 core-ui 控件填充（`<gh-img-view>` 等）→ 保持 `<div>`」，**而元素实际是 `<custom>`**；且「`<gh-img-view>` 填充」**不可执行**（喂数 API 在 `SPEC.md` §8 第 7 条是 out-of-scope）。架构要求前端把第五段改成与实现一致的表述，并指出「`.uix` 头注释是给下一个人的说明书，自相矛盾比写错更贵」 | **已改**（三个文件全查了一遍，不只 preview.uix）：<br>· `assets/preview.uix` 第五段重写为「宿主标签通用规则」+ 明确 `preview_body` 是 Rust 自绘图片位 → `<custom>`；文件头的「优先通道是 `ui_gh_img_view_*` 族」那句**与 D06-1 冲突**，改为指向 §四/§五 并注明 gh-img-view 喂数是 out-of-scope<br>· `assets/main.uix` 第八段（宿主标签通用规则）的判定表**与实际实现矛盾**（还写着 `breadcrumb` → `<div>` 保持、`preview_host` → core-ui 控件填充）→ 改为六处逐个判定（与 §5 R5 同一张表）<br>· 三个文件头部的**元素数**同步更新（main 30→30 / 33→33 的旧数字改为「标签替换不改元素数（30→30，外壳 34→34）；第七轮起是 35，外壳 39」） |
| **★ 基线修正：`git show HEAD:` 已失效** | 本次修正**本身已被提交**（`9a6234b` → `3cd3a08` / `b336ae4`）⇒ **`HEAD` 现在返回的就是修正后的文件**。docs/12 §0 的标签统计与 §3 表的「原:行」都写着 `git show HEAD:` ⇒ **照着做会得到「一个旧标签都不剩」的错误结论** | **基线钉到 `6ea629f`**（本次修正**之前**的最后一个原始提交，`docs(fork): sync WinUI3 UI design assets`）：§0 加了警示段；§3 表头改为「原:行 = `git show 6ea629f:assets/<file>`」。**并已用脚本逐行复核 46 行**（原:行 对 `6ea629f`、现:行 对当前资产），异常 0 处 |
| **QA O-1 · 「末尾追加」与口径 ⑤ 自相矛盾** | QA 指出 `themes.json` 与 docs/12 同处并存两句互斥的话 | **本轮已由架构裁定甲A**（注入形态 = 「源码**最前面**插入」），`themes.json` 的 `revision.inject` 与 docs/12 §4.2 六口径表**已在上一轮改毕**；本轮补的是 §9 历史行的更正标注（把「末尾追加」标成**当轮口径**并注明第八轮已改） |
| **QA O-2 · 旧句柄的失效机制写错（结论对、机制错）** | 我原先写「旧句柄 = 写野内存 = **静默崩溃 / 花屏**」。实际：句柄表存 `shared_ptr`（`handle_table.h:31`）、`ui_page_destroy` 只 erase `PageEntry` 不碰句柄表（`page_api.cpp:561-563`）、句柄表只在 `Context::Shutdown` 时 `Clear()`（`ui_context.cpp:125`）⇒ 旧句柄指向**还活着但已脱树**的孤儿控件 ⇒ 写它是**静默无效**（API 成功、界面毫无变化），**不是崩溃**；且每次切主题**泄漏旧页整棵控件子树** | **已独立复核上游源码后成立并改正**：§4.4 新增「旧句柄的确切失效形态」小节（四条事实表 + 结论 + 泄漏说明）；§5 R4 补一步处置（**销毁旧 page 前先对 32 个旧句柄逐个 `ui_widget_destroy(old_handle)`**，`ui_api.cpp:1886-1888`）；§5 R5 里的「静默崩溃/花屏」措辞同步改为「静默无效」；§5 R5 的句柄数 **27 → 32** 一并改掉 |
| **QA O-3 · R5 的宿主表只有 4 行** | 新增的 `preview_body`（已改 `<custom>`）与 `preview_image`（新增 `<custom>`）**两行没有**；且若有人把 `preview_host` 整体改成 `<custom>`，**会丢掉 4 个 `<label>`**（`<custom>` 是空的自绘画布） | **已补**：§5 R5 的判定表改为**六处**逐个判定，并加一条「⚠️ `preview_host` 不能整体改成 `<custom>`」；三个 `.uix` 的头部注释同步 |
| **QA O-4 · `value.cpp` 行号漂移** | 实测 `LookupVar` 在 **`572-579`**（我原写 `573-581`），共 7 处 | **已改 8 处**：`docs/12` 6 处 + `docs/13` 1 处 + `assets/themes.json` 1 处（`docs/15` 那 1 处是交付总监写的，已是正确值） |
| **QA O-5 · §9 历史行仍写当轮旧值** | 会被读成现值 | **已逐行加「（当轮值；现为 …）」**：句柄数 27→32（3 处）、元素数 30/9/16→35/9/16（2 处） |
| **QA · 主题分支约束归谁** | QA 裁定断言 14/15/16（主题分支选择器 / `:root` 里不得定义 `--app-*` / `.uix` 里不得出现 `var(--app-accent)`）由 **QA 加在 `qa_uix_verify.py`**，不去改前端的 `uix_tag_check.py`（文件域：那个脚本是我写的） | **接受**：本轮**未**把这三条加进 `uix_tag_check.py`。架构另授权的 **S13 两条**（token 命名空间）**是**加进我的脚本的（见下行） |
| **架构授权 · S13 两条断言进 `uix_tag_check.py`** | 架构明确授权前端把 S13 等价断言加进自己的 `uix_tag_check.py`（**预检**），正式门禁仍归 QA 的 `scripts/gates/` | **已加**（`scripts/gates/uix_tag_check.py`）：① 不得定义库 34 token 任一；② `.uix` 内不得定义 `--app-accent`；③ `themes.json` 内允许 `--app-accent`；④ `var(--accent)` 等引用不拦；另 `.uix` 内不得出现 `var(--app-accent)`。<br>**库的 34 个 token 名单不在本脚本重复抄写**，而是从正式门禁 `s13_token_namespace_check.py` **import**（单一事实源，避免两份名单各自漂移）；取不到名单时**判 FAIL 而不是静默跳过**。<br>**负控已做**：注入 4 类缺陷（`.uix` 定义 `--bg` / 定义 `--app-accent` / 出现 `var(--app-accent)` / `themes.json` 定义 `--bg`）⇒ **EXIT=1，4 处 FAIL，逐条带文件:行** |
| **架构第九/十轮 · id 清单与门禁** | 架构要求「期望 id 清单 == 实际 id 集合」的门禁（归属 QA），并把句柄数从硬编码改为**单一事实源** | 交付总监已落 `assets/ui-ids.json`（32 个 id：main 26 / preview 6 / about 0）+ `scripts/gates/ui_id_manifest_check.py`。**本轮我把自己的 `uix_tag_check.py` 与它对齐**：新增「清单外的 id」检测（防止只查丢失、不查多出）+ 清单一致性交叉校验（硬编码副本 vs `ui-ids.json` 不一致即 FAIL）—— 负控已做（从 `ui-ids.json` 删一个 id ⇒ 正确 FAIL） |
| **架构第八轮 · R11 扩为四条断言** | 架构把 Rust 侧加载前自检由一条扩为四条（位置 / 顺序 / 内容 / 键集），与 `spec/milestones.md` V1.12 的四个坏源码构造一一对应 | **§5 R11 已改写为四条断言**（每条给出违反后果与源码依据），并写明静态侧已有门禁（`s13_token_namespace_check.py` + `uix_tag_check.py`） |
| **架构第七/八轮 · 两条必记** | ① `breadcrumb` 的**空 `<div>` 显示不出任何东西** → 必须补内容，否则 M1 的 **V1.16** 必挂；② `preview_host` 的「谁画这张图」请记成「Rust 在 `<custom id="preview_image">` 里画」（**D06-1**） | **已登记为两条新的待办**：**R12**（面包屑契约：Rust 必须 `ui_label_set_text` 投影路径；给出 A（直接写控件，推荐）/ B（页面状态 + `{{ }}` 插值）两种等价写法，并注明 B 多一层耦合、首版建议走 A；对应 V1.16）与 **R5b**（谁画这张图 = Rust 在 `<custom>` 里 blit；含 D06-1 的机制、为什么不用 `<gh-img-view>`（out-of-scope）/ 不用 `ui_image_view()`（运行时控件无 CSS）/ 不用 `<img>`（src 是资产 key），以及「缩放/平移/滚动自己实现、首版只要 fit」的代价） |
