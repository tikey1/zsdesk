# `.uix` 真机首次渲染检查清单

> 用途：core-ui 首次在本机跑起来时，按本清单逐条看、逐条判。
> 配套：`docs/11-COREUI-MARKUP-TAGS.md`（M01~M06 定义）、`docs/12-UIX-TAG-FIX.md`（改了什么、为什么）。
> 适用资产：`assets/main.uix`、`assets/preview.uix`、`assets/about.uix`。
> 日期：2026-11-13　　执行：前端（贾思敏）　　**本清单是待办，不是已完成的结论**。

---

## 0. 怎么用这份清单

- 每条 = **看什么 / 期望什么 / 异常怎么定位**。定位列给的是**第一个该怀疑的对象**，不是唯一原因。
- 勾选规则：**只有亲眼看到期望结果才打勾**；"应该是对的"不算。
- 每条不通过，**记一行**：现象 + 哪条编号 + 当时的窗口尺寸 / DPI / 主题模式。
- 上机前的准备（否则大部分条目无意义）：
  - [ ] Rust 侧已按 `docs/12` §5 的 R1 注入 `:root` 块（读 `assets/themes.json`）
  - [ ] 已调用 `ui_theme_set_default_font(L"Segoe UI Variable")` + `ui_theme_set_cjk_font(L"Segoe UI", L"Microsoft YaHei UI")`
  - [ ] 已调用 `ui_theme_set_accent_hex("#2563EB")`（浅色）
  - [ ] 已调用 `ui_theme_set_mode(...)` 并与系统主题一致
  - [ ] 打印 `ui_page_last_error(page)` 的值并记录（应为**空**）
  - [ ] `scripts/verify.ps1` 的 D-15 检查（零事件属性 / 零业务脚本）通过

> **重要**：第 0 项的 4 条只要少做一条，后面「颜色 / 字体 / 深浅色」的条目会**全部误报**。先确认这 4 条再做。

---

## 1. M01~M06（承自 `docs/11` §4，本轮新增的定位信息已并入）

### M01 · 未知标签的行为 —— 决定「标签写错」是白屏还是静默变形

| | |
|---|---|
| **看什么** | 页面加载后是否出现「预期的空白/空盒子」，而不是报错或崩溃 |
| **期望什么** | 本轮三个资产**不含**未知标签 → 不会出现空盒子。**基线行为**（已从源码查明）：未知标签走 `widget_factory.cpp:1192-1193` 的兜底 → 变成一个**空的 VBox**，**不报错、不崩溃** |
| **异常怎么定位** | 若某个位置是「该有文字却空白」：<br>① 先查是不是标签写错 → 未知标签 = 空盒子（无文字）<br>② 再查是不是用了 `text="..."` 属性 → `.uix` 里不生效（`FirstPlainText` 只读子文本节点）<br>③ 再查文字是不是写在了标签外面（成了顶层文本 → `sfc_parser.cpp:260` 报 "unexpected content at top level"） |
| **实测建议** | 故意在测试副本里放一个 `<bogus-tag>` 加载一次，确认「静默空盒子」而不是崩溃，把结论钉死（M01 是本轮唯一还没被源码完全回答的项） |

### M02 · `flex-direction: row` 在嵌套容器中的继承与覆盖

| | |
|---|---|
| **看什么** | ① 标题栏：应用名 + 搜索框是否**同一行**<br>② 标签栏 / 命令栏 / 地址条 / 状态栏是否横排<br>③ 工作区：导航 | 列表 | 预览 是否**三栏并排**<br>④ 预览窗格：工具条标题是否在左 |
| **期望什么** | 上述全部为**横排**。行容器 = `.titlebar` / `.tabbar` / `.commandbar` / `.addressbar` / `.statusbar` / `.main-area` / `.preview-toolbar`（各 1 条 `flex-direction: row`） |
| **异常怎么定位** | 某一行变成竖排 → ① 查该 class 是否有 `flex-direction: row`（core-ui **只看这一条**决定横竖，`widget_factory.cpp:918-927`）；② 查 CSS 解析是否失败（`var()` 未定义会整条声明作废 → 该行丢失 `flex-direction`）；③ 查 `flex-direction` 写在了哪个选择器上（选择器没匹配上就等于没写） |
| **补充** | 容器**不按标签名**分横竖：`row`/`row-reverse`→HBox，其余→VBox。所以 `<div>` + 无 `flex-direction` = 竖排，**这是最容易漏的一条** |

### M03 · `<label>` 与 `<span>` 在 flex 容器内的 stretch 行为差异

| | |
|---|---|
| **看什么** | ① 状态栏 6 个片段是否**挤在左边**、没有被拉成等宽<br>② 标题栏「tabbed_files」是否紧贴左边缘（左边距 12px）<br>③ 预览窗格的「选中单个文件以预览」是否**占满窗格宽度**（而不是只有文字宽度）<br>④ 关于面板的长条目是否**在 520px 窗宽内折行**，而不是横向溢出 |
| **期望什么** | ①② 是**不拉伸**（inline 语义）；③④ 是**占满 + 折行**（`display: block`） |
| **异常怎么定位** | 拉伸了 → 该元素缺 `display: block`（或 `align-self: stretch`）；<br>没占满宽度 → 该元素是 inline，core-ui 里 `<span>`/`<label>` **默认都不参与 stretch**；<br>该折行却溢出了 → 同样是 inline（宽度=自然文字宽度，wrap 失效） |
| **提醒** | `<span>` 与 `<label>` 在 core-ui 里**是同一条代码分支**（`widget_factory.cpp:942-954`），行为完全一样；真正控制的是 `display` / `white-space` / `align-self` |

### M04 · `var(--token)` 自定义 CSS 变量是否生效 ★最关键

| | |
|---|---|
| **看什么** | ① 窗口底色是不是 `#F3F3F3`（浅色）<br>② 列表区是不是 `#FFFFFF`、导航/预览窗格是不是 `#FBFBFB`、工具条/状态栏是不是 `#F7F7F7`<br>③ 文字是不是 `#1B1B1B`、次要文字 `#616161`<br>④ 分隔线是不是 `#E0E0E0`<br>⑤ 选中/下划线是不是 `#2563EB` |
| **期望什么** | 全部等于 `assets/themes.json` 的 `light` 值（色值一个没改，只改了 token 名） |
| **异常怎么定位** | 全是库的默认色（底色纯白 `#FFFFFF`、文字 `#424242`、分隔线 `#D1D1D1`）→ **`--app-*` 根本没进页面**，按顺序查：<br>① Rust 有没有把 `:root { … }` 拼进 `.uix` 源码再 `ui_page_load_string`（**没有运行时变量 API，这是唯一入口**）<br>② 自定义属性是不是写在了 `:root` 里（写在别的选择器下会被 `cascade.cpp:134` 直接忽略）<br>③ token 名有没有写成旧名（`--bg` / `--text` / `--divider`…）——旧名现在不存在，会回落到库 token 或解析失败<br>④ `.uix` 里引用的名字和 `themes.json` 里的键是否**逐字一致**（大小写敏感）<br>**只有 accent 是例外**：accent 应读库 token `var(--accent)`，若显示 `#3971EC`（库默认蓝）而不是 `#2563EB` → Rust 没调 `ui_theme_set_accent_hex` |
| **补充** | `var()` 支持已源码核实（`cascade.cpp:42-54`、`value.h:91`）；**但库自有的 34 个 token 会被 `ui_theme_set_mode` 清空重写**，所以项目 token 绝不能和它们同名 —— 这也是本轮全部加 `--app-` 前缀的原因 |

### M05 · `<style>` 里 class 选择器与 id 选择器的支持度

| | |
|---|---|
| **看什么** | ① 每个 class 的样式是否都生效（背景/高/色/间距逐个看）<br>② 有没有**任何** `#id` 选择器 —— 本轮三个资产**一个都没有**，本条属「回归确认」 |
| **期望什么** | class 全部生效；id 选择器数量为 0（本轮未使用，所以这条只是确认"没引入"） |
| **异常怎么定位** | 某个 class 完全没生效 → ① 该 class 在模板里是否真的写对了（`.uix` 里写 `class="titlebar"` 还是 `class="titlebar "` 会不会漏配？→ 查模板里的拼写）；② 样式表是否在 `<style>` 块里且未被注释吞掉；③ 该条声明里是否用了**未定义**的 `var()`（会让整条声明作废，而不是只作废这个值）；④ 是否有拼写错误（如 `flex-direciton` —— core-ui **未知属性被忽略，不报错**） |
| **补充** | `:root` 里的自定义属性与普通 class 规则是两条不同路径：`:root` 只认自定义属性，普通 class 规则里的自定义属性会被忽略 |

### M06 · `<Separator>` 是否支持竖向

| | |
|---|---|
| **看什么** | 导航/列表/预览之间那两条**竖线**是否可见、是否贯穿整个工作区高度 |
| **期望什么** | 两条 1px 竖线，贯穿工作区全高，颜色 `--app-divider` |
| **异常怎么定位** | 竖线不见了 → ① 查 `.vsep` 的 `width: 1px` 有没有被 `flex-shrink` 挤成 0（本轮已加 `flex-shrink: 0`）；② 查 `align-self: stretch` 有没有生效（不生效则只有 0 高）；③ 查工作区容器 `.main-area` 是不是 `flex-direction: row`（是竖排则竖线就成横线）；④ 查 `--app-divider` 是否已注入（M04） |
| **补充** | **已查实**：`<Separator>` **只有水平形态**（`controls.h:198-206` 的 `SeparatorWidget(bool vertical=false)`，工厂从不传 true），所以竖线必须用 1px 的 `<div>`——本轮就是这么做的。**不需要再验证 `<Separator orientation>`，已排除** |

---

## 2. 布局与尺寸（对照 `docs/07` §2）

窗口基准：**1200×800 @100% DPI**

| # | 看什么 | 期望 | 异常怎么定位 |
|---|---|---|---|
| L1 | 标题栏高度 | 32px，输入框（搜索）26px 且**垂直居中** | 贴上沿 → `.titlebar` 缺 `align-items: center`（inline/inline-block 不参与 stretch，默认会顶到上沿） |
| L2 | 标签栏高度 | 36px | 同上（`.tabbar`） |
| L3 | 命令栏高度 | 40px | 同上（`.commandbar`） |
| L4 | 地址与搜索条高度 | 36px；面包屑在左（自适应）、地址框在中间（`flex:1`）、搜索框在右且**固定 220px** | 搜索框被挤扁 → `.search` 的 `width: 220px` 被 `flex-shrink` 压缩（本轮未对 `.search` 加 `flex-shrink: 0`，若出现挤压需补） |
| L5 | 左侧导航窗格宽度 | 220px | 变窄 → `.nav-host` 的 `width` 被覆盖 |
| L6 | 右侧预览窗格宽度 | 320px | 同上 |
| L7 | 中间列表区宽度 | 剩余全部（`flex: 1`） | 被挤 → `.list-host` 缺 `flex: 1` |
| L8 | 竖分隔线 | 2 条，各 1px 宽，通高 | 见 M06 |
| L9 | 水平分隔线 | 4 条（标签栏下 / 命令栏下 / 地址条下 / 状态栏上），各 1px 高，通宽 | 线不见了 → `.divider` 的 `background` 未生效（M04）或分隔线被 0 高容器裁掉 |
| L10 | 信息条 | 默认**收起**（0 高，看不到）；`.infobar-visible` 时 36px | 一直显示 → `.infobar` 的 `height: 0px` 被覆盖 |
| L11 | 状态栏高度 | 28px，片段左对齐、间距 12px | 片段被拉散 → `.status-segment` 缺 `white-space: nowrap` 或被当成 block |
| L12 | 整体无滚动条 | 1200×800 下不应出现纵向滚动条 | 出现 → 某处 `flex: 1` 缺失，或固定高度总和 > 800 |

## 3. 层级与内容（对照 `docs/07` §1、§6）

| # | 看什么 | 期望 | 异常怎么定位 |
|---|---|---|---|
| H1 | 21 个 id 是否都能被 Rust 取到 | `ui_widget_find_by_id` 全部命中（main 21 / preview 6） | 取不到 → 检查该元素是否被改成了 core-ui 不支持的标签（现在全部支持）；空句柄 → Rust 侧 id 字符串写错 |
| H2 | 标题栏应用名 | 显示 `tabbed_files` | 空白 → ① 用了 `text=` 属性（不生效）② 文字写在标签外 ③ 该 label 是 inline 但被 0 宽容器裁掉 |
| H3 | 标签栏占位文字 | 显示「标签页由 Rust 侧 TabManager 提供」 | 空白 → 同 H2；这条最容易踩：`text=` 属性在 `.uix` 里不生效 |
| H4 | 命令栏提示 | 显示「后退 前进 上级 刷新 · 查看 排序 预览」 | 空白 → 同 H2 |
| H5 | 状态栏 6 个片段 | Rust 写入后各自显示（`ui_label_set_text`） | 写入后不显示 → 该元素不是 LabelWidget（本轮全部是 `<span>` → LabelWidget，可写）；或 Rust 用错了句柄 |
| H6 | 面包屑 / 导航 / 列表 / 预览 四个宿主区 | 空但**有底色**（`--app-content-bg` / `--app-sidebar-bg`） | 整个区域空白且**连底色都没有** → 该 `<div>` 的 `flex`/`width` 没生效，或被父级压成 0 |
| H7 | 关于面板 | 标题 + 13 条条目 + 2 条分隔线；长条目**在窗宽内折行** | 横向溢出 → `.row` / `.heading` 缺 `display: block` |
| H8 | 预览窗格 | 标题「预览」+ 分隔线 + 正文区 + 4 行提示 | 标题不显示 → 同 H2 |
| H9 | 首版只读 | 命令栏**没有**新建/重命名/删除/剪贴板入口 | 出现写操作 → 资产被误改 |

## 4. 字体与排版

| # | 看什么 | 期望 | 异常怎么定位 |
|---|---|---|---|
| F1 | 正文字体 | 拉丁 Segoe UI Variable（Win11）/ Segoe UI（Win10）；中文 Microsoft YaHei UI | 字体没变 → Rust 没调 `ui_theme_set_default_font` / `ui_theme_set_cjk_font`（**`.uix` 里的 `font-family` 对普通 LabelWidget 不生效**，只对 SVG 文本生效） |
| F2 | 标题栏应用名字号 | 13px | 偏大/偏小 → `.app-title { font-size: 13px }` 未生效 |
| F3 | 关于面板标题字号 | 16px、粗体 | 同上（`.heading`） |
| F4 | 长文本是否被截断成省略号 | 状态栏片段过长时显示 `abc...` 而非折行 | 折行 → `.status-segment` 缺 `white-space: nowrap` |
| F5 | 中文与英文混排 | 无豆腐块、无基线跳动 | 豆腐块 → 缺 `ui_theme_set_cjk_font` |
| F6 | 预览窗格标题「预览」、关于面板标题「tabbed_files 1.2.3」 | **当前预期是不加粗** —— `font-weight: 600` 在 core-ui 是空声明（只认 `bold`/`700`，`widget_factory.cpp:1392-1395`），见 `docs/12` §5 R8。**若真机上标题明显不粗，不是 bug，是待裁决项** | 要强调有三条路：改 `bold`（超 docs/07 字重上限）／把标题字号提到 14px 靠字号做层级（推荐）／保持现状 |
| F7 | 三个输入框（全局搜索 / 地址 / 筛选）底色 | 白色（浅色模式）＝ `--app-content-bg`，**不是**库的输入框默认态 | CSS `background` 会被采纳（`controls.cpp:1052-1063` 优先用 `bgColor`）；但注意**因此没有 hover / focus 底色变化**，只有边框变化 —— 若产品要 hover 态，需改由 Rust 设色或加 `border-width` |

## 5. DPI（100 / 125 / 150 / 200%）

| # | 看什么 | 期望 | 异常怎么定位 |
|---|---|---|---|
| D1 | 1px 分隔线 | 各档都**可见**（不消失、不变 2px 糊边） | 消失 → core-ui 用 DIP（1/96 英寸），物理像素 = DIP × monitor_dpi ÷ 96（`docs/11` §3）；1px 在 150% 下是 1.5 物理像素，需看渲染实现 |
| D2 | 文字 | 不截断、不重叠；行高随字体缩放 | 重叠 → 固定高度容器（32/36/40/28px）被系统文本缩放撑爆 |
| D3 | 布局 | 导航 220 / 预览 320 不随 DPI 变比例（是 DIP，不是物理像素） | 比例失真 → 某个宽度写成了物理像素 |
| D4 | 最小字号 | ≥11px（docs/07 §7） | 低于 → 某处 `font-size` 写得过小 |

## 6. 深浅色切换

| # | 看什么 | 期望 | 异常怎么定位 |
|---|---|---|---|
| C1 | 切深色后**自绘区**是否变深 | 底 `#202020`、内容 `#2B2B2B`、侧栏 `#252525`、文字 `#F5F5F5`、次要 `#B3B3B3`、分隔线 `#3D3D3D` | **不变** → 项目 token 不在库的 34 个 token 里，`ui_theme_set_mode` 不会改它们 → 必须按 `docs/12` §4.4 **重建页面**（重注 `:root` + `ui_page_load_string`） |
| C2 | 切深色后**库控件**（按钮/输入框/焦点环）是否变深 | 跟随系统 | 不变 → 没调 `ui_theme_set_mode` |
| C3 | accent 在两个模式下 | 浅 `#2563EB` / 深 `#60A5FA`；且**自绘区与按钮是同一个蓝** | 深色下变成 `#3971EC` → Rust 切 mode 后**没有重新调** `ui_theme_set_accent_hex`（切 mode 只保留 accent override 的**值**，不会自动换成另一套色值） |
| C4 | 切深色后是否有**半深半浅**（一半新一半旧） | 全套一起变 | 半深半浅 → 只切了库没重建页面（C1），或重建了但没先 `ui_theme_set_mode` |
| C5 | 深色下分隔线是否还可见 | `#3D3D3D` 可见但不刺眼 | 看不见 → 检查 `--app-divider` 是否注入成功 |

## 7. 加载与错误处理

| # | 看什么 | 期望 | 异常怎么定位 |
|---|---|---|---|
| E1 | `ui_page_last_error(page)` | **空字符串** | 非空 → 按内容定位：<br>· "SFC parse failed" → `<window>` 没自闭合 / 顶层有非法标签 / `<template>` 缺失<br>· "template parse failed: no root element" → `<template>` 里没有根元素<br>· "missing `export default`" → `<script>` 块不合规<br>· "<script> JS error" → 同上<br>· "unknown top-level tag" → SFC 顶层出现了非 `<window>/<template>/<script>/<style>/<import>/<link>` 的标签 |
| E2 | 窗口标题 / 尺寸 | 标题「tabbed_files」，1200×800 | 不对 → `<window>` 的属性没读到 |
| E3 | 窗口是否出现 | 出现且非白屏 | 白屏 → ① 看 E1 ② 看是否所有内容都因 `text=` 属性而空 ③ 看 `.root` 的 `background` 是否没生效 |
| E4 | D-15 | 资产里零事件属性、零业务脚本（`verify.ps1` 通过） | 不通过 → 资产被误改 |
| E5 | 内存 / 句柄 | 反复开关窗口无句柄泄漏 | 泄漏 → Rust 侧 RAII 问题（不在本轮范围） |

---

## 8. 判定与回退

- **P0（阻断）**：E1 非空 / E3 无窗口 / M04 全是库默认色 / H2~H4 任一空白 / L1~L12 任一尺寸错 → **先别往下走**，按定位列逐条查。
- **P1（必须修）**：C1~C5 深浅色问题 / D1 分隔线在 150% 消失 / F1 字体不对。
- **P2（可记账）**：细节色差（如分隔线 #D1D1D1 vs #E0E0E0）、工具条留白等。

**回退点**：本轮改动已全部落盘且**元素总数不变**（main 33 / preview 12 / about 19），id 一个没动，
出现系统性偏差时可直接 `git checkout HEAD -- assets/main.uix assets/preview.uix assets/about.uix assets/themes.json`
回到原资产，再按本文档的分项结论逐项重做，不必整体回滚。

---

## 9. 本清单里**没有**答案、必须靠真机回答的问题（汇总）

1. **未知标签到底是空盒子还是报错**（M01 的完整行为）——源码只覆盖了"落到 VBox 兜底"，未覆盖渲染期表现。
2. **`flex-direction: row` 在三层嵌套下的实际分配**（M02）——尤其 `.main-area` 里三栏 + 两条竖线的宽度分配。
3. **inline 标签在 HBox 里的实际对齐/基线**（M03）——文本会不会比输入框高/低。
4. **`:root` 注入的时序**：必须在 `ui_page_load_string` 之前；晚一步就静默用库默认色（M04）。
5. **`ui_theme_set_mode` 之后自绘区与库控件是否出现两种蓝**（C3/C4）——accent override 的实际保留行为。
6. **1px 线在 150% / 200% DPI 下的可见性**（D1）。
7. ~~`TextInputWidget` 是否会用库默认 `inputBg` 盖掉 CSS 的 `background`~~ → **已查实：不会**（`controls.cpp:1052-1063` 优先用 CSS 的 `bgColor`）；**残留问题是这三个输入框因此没有 hover / focus 底色变化**（见 F7）。
8. **`font-weight: 600` 是否真的不加粗** → 源码已答：**不加粗**（只认 `bold`/`700`）。需要设计师裁决的是「要不要改」，不是「是不是加粗」（见 F6 / `docs/12` §5 R8）。
9. **`<input>` 的默认圆角**：core-ui 未设 `border-radius` 时用 `theme::radius::medium`（`controls.cpp:1047`），与 docs/07「控件圆角 4 px」是否一致 —— 需真机看一眼。
