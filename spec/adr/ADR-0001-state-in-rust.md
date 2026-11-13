# ADR-0001：状态留 Rust，`.uix` 只做静态布局

- 状态：已接受
- 日期：2026-11-13
- 决策者：架构师（高见远），依据用户对话 D02/D04/D05 与 `assets/main.uix` 的 D-15 注释

## 背景

上游 zsdesk 是 **Rust 侧 MVU**（消息 → 状态 → 重绘），42,150 行 Rust。
core-ui 的 `.uix` 是 **Vue 3 SFC 风格**，内嵌 QuickJS-NG，有自己的响应式状态
（`data()` / `computed` / `methods`）。

两套状态机同时存在时，「哪个是真相源」必须先定，否则后面每一处 UI 更新都会变成
「Rust 改了 → 推给 JS → JS 再推回来」的双向同步，42k 行逻辑会有一半被拖进这条同步链。

三个事实决定了答案：

1. `zsdesk-core`（1,999 行）**本来就不依赖任何 UI 框架** —— 它的 crate 文档自述
   「领域层不依赖 HWND、COM 指针或 ZSUI 控件」。状态骨架已经是平台无关的。
2. `.uix` **没有 batching**（`docs/uix-guide.md` §3.5 明示「同步触发」），
   而 `ui_page_set_json` / `ui_page_get_json` 是**全量序列化**，`ui_page_watch`
   的比较也是「整个值的 JSON」。一个 5,000 条目的目录，每次切换都是 O(n) 的
   序列化 + 解析 + 响应式 diff —— 不可接受。
3. core-ui **无虚拟列表/表格控件**（控件全集无 `ListView`/`DataGrid`，
   `ScrollView` 非虚拟化）→ 虚拟化本来就得自己做（ABI-NOTES V13）。

## 决策

> **我们将采用「状态留 Rust」：Rust 持有全部业务状态，`.uix` 只是被投影的静态布局，
> 文件列表根本不在 `.uix` 里 —— 它是 Rust 运行时创建并挂进 `<div id="list_host">` 的
> `CustomWidget`，由 Rust 在 `ui_custom_on_draw` 回调里直接画。**
>
> （2026-11-13 修正：初版写的是 `.uix` 里放 `<custom id="file_list"/>`。
> 实测 `ui_custom_on_draw` 的实现是 `As<ui::CustomWidget>(w)`，取不到就**静默 no-op**
> （`ui_api.cpp:2612-2616`），而用户的 `assets/main.uix` 用的是 `<panel id="list_host">`（div）。
> → 改为**运行时 `ui_custom()` + `ui_widget_add_child()`**，`.uix` 只提供宿主与样式。
> 另：因库无运行时替换 CSS 变量的 C API，页面改用 **`ui_page_load_string`** 加载
> （Rust 注入 `:root`），不是 `ui_page_load_file`。）

> ### 修订记录（本 ADR 的「决策」正文已被后续裁定取代，以下列时间线为准）
>
> | 版本 | 结论 | 取代它的原因 |
> |---|---|---|
> | v1 | 运行时 `ui_custom()` + `ui_widget_add_child()` | ★ **已被 D12-1 取代**：运行时造的控件**不经过 `ApplyCommonStyle`**（该函数只有 `compiler.cpp:920` 与 `widget_factory.cpp:1363` 两个调用点）→ **没有 CSS** → flex 容器里**尺寸为 0** → 列表区空白 |
> | v2（D12-1，当前生效） | **`.uix` 里静态声明 `<custom id="list_host" class="list-host"/>`**；Rust 取句柄后挂 `ui_custom_on_draw` | — |
> | v3（2026-11-13，本轮） | **id 以资产为准 = `list_host`**（不是 `file_list`；命名规则 `<区域>_host`） | id 是 Rust ↔ `.uix` 的契约（`find_by_id` / `on_widget_mount` / `<menu trigger="#…">` 三处都用）→ 不能各写各的 |
> | v3（同轮追加） | 页面重建后**必须在新 page 上重新注册 `ui_page_on_widget_mount`** | 钩子挂在**具体 page** 的 `PageState` 上（`page_api.cpp:850-854`）→ 新 page 的钩子表是空的 → **不自动转移**（catch-up 只保证「注册即触发」） |
>
> ★ **不变的部分**：**状态留 Rust**、`.uix` 只做静态布局、**行**不由 `.uix` 创建（D-15 不变）。

具体分工：

| 归属 | 内容 |
|---|---|
| **Rust（权威）** | 会话、标签、当前位置、条目列表、排序、选择集、列宽、视图记忆、搜索结果、桌面格子状态 |
| **`.uix`（视图局部）** | 侧栏树展开集合、地址栏未提交文本、过滤框文本、hover 行号、菜单/对话框开合 |
| **`.uix`（禁止）** | 文件条目数组、选择集、排序规则、任何 O(n) 数据 |

事件流向：core-ui 把事件派发成**我们注册的 C 函数指针** → 直接进 Rust 状态机
（**不经过 QuickJS**）→ Rust diff → 小载荷 `ui_page_set_*` 投影回 `.uix` → 响应式重绘。

## 后果

- **变容易的**：
  - 42,150 行 Rust 逻辑**一行不重写**（`zsdesk-core` 100% 直接复用）；
  - 交互事件不过 QuickJS，少一层序列化与延迟；
  - 状态机的测试全部是普通 Rust 单元测试，不需要起窗口。
- **变困难的**：
  - 文件列表 / 导航树的**绘制、命中测试、滚动、选择**全部要自己写；
  - 绘制回调内**禁止**调用会同步触发重绘的 `ui_*`（改状态要记 pending 队列）→ 多一层间接；
  - `.uix` 的响应式能力用得少，很多「看起来该在 JS 里做的事」要在 Rust 里做。
- **引出的新决策**：
  - D07（导航树也自绘）—— 同一个「不能嵌原生 HWND」约束的连带后果；
  - D08（FFI 绑定与线程收敛）—— 「状态留 Rust」要求 Rust 侧能安全、准时地驱动 UI 线程；
  - D10 的直接后果：页面用 `ui_page_load_string`（Rust 注入 `:root`）→
    **`.uix` 与 `themes.json` 必须以文件形式进包**（CI 打包当前漏了，见 `spec/ci-contract.md` §2.4）。

## 备选方案

| 方案 | 为什么没选 |
|---|---|
| 状态进 `.uix` 的 JS | 要重写 `ExplorerSession`/`ExplorerTab`/`ShellLocation`/`FileEntry`/`natural_cmp`/`RequestId`；搜索（Everything IPC / MFT / USN）、文件操作（`IFileOperation`）、转发、迁移要么留在 Rust 变成双份状态，要么一起重写成 JS（要引入 Node 侧文件系统与 COM 绑定）。**这不是「换 UI 库」，是重写整个领域层** |
| 双轨（Rust 持有权威 + JS 持有镜像） | 要维护同步逻辑；文件列表的 O(n) 序列化问题原样存在；两处真相源必然漂移 |
| `.uix` 声明列表数据、只画样式 | core-ui 没有虚拟列表控件，条目一多就会卡；且 `ui_page_set_json` 每次切换全量传输 |

> **不可变**：已接受后不得修改本文件；改变主意请新建 ADR 并标注「取代」。
