# ADR-0003：文件列表用 `<custom>` 全自绘（不嵌 `IExplorerBrowser`）

- 状态：已接受
- 日期：2026-11-13
- 决策者：架构师（高见远），依据 D04 / D06 / D07

## 背景

上游 zsdesk 的文件列表是 **Shell 原生视图**：`IExplorerBrowser`（`zsdesk-windows/src/shell/explorer_browser.rs`，883 行）
+ 左侧原生 `INameSpaceTreeControl`（`namespace_tree.rs`，1,029 行）+ 原生 Shell 右键菜单宿主。

三条实测事实让「保留原生控件」这条路走不通：

1. **core-ui 不提供原生 HWND 子控件嵌入**：全仓 `src/` 中 `SetParent` 仅出现在
   `src/ui/image_view_gdi.cpp`（挂自己的 GDI 子窗口）；`include/ui_core.h:151` 明写
   child window 语义不适用（ABI-NOTES V12）。
2. **core-ui 无虚拟列表/表格控件**：控件全集为 Label/Button/CheckBox/RadioButton/Toggle/
   Slider/ProgressBar/TextInput/TextArea/ComboBox/NavItem/Separator/TitleBar/IconButton/
   TabControl/SVG/Image/ImageView/Dialog/Toast/ContextMenu/CustomWidget；
   `ScrollView` **非虚拟化**（ABI-NOTES V13）。→ 虚拟化本来就得自己做。
3. **`<custom>` 标签受支持**（`src/ui/page/widget_factory.cpp` 的 `ConstructByTag`），
   可用 `ui_widget_find_by_id` 取句柄后挂 `ui_custom_on_draw` / `ui_custom_on_mouse_*`
   （`docs/11-COREUI-MARKUP-TAGS.md` §2.3）→ 用户 `assets/main.uix` 既定的
   「Rust 侧绘制列表」架构**成立，无需改架构**。

保留原生控件还有第二个问题：**它与 core-ui 的 Fluent 观感必然割裂**
（原生控件跟随系统主题，core-ui 走自己的 Fluent 2 令牌）。

## 决策

> **我们将采用 core-ui 全自绘：文件列表与导航树都用 `<custom>`，
> 数据与交互全部在 Rust 侧，Shell 只提供语义（枚举 / 图标 / 文件操作）。**

落点（详见 `spec/ffi-core-ui.md` §7.3）：

> **挂载方式（2026-11-13 修正）**：`.uix` 只提供宿主 `<div id="list_host">` 与样式；
> 自绘控件由 Rust **运行时**创建并挂载：
> `ui_widget_find_by_id(page_root, "list_host")` → `ui_custom()` →
> `ui_widget_add_child(host, custom)` → 挂 `ui_custom_on_draw` / `ui_custom_on_mouse_*`。
> 依据：`ui_custom_on_draw` 的实现是 `As<ui::CustomWidget>(w)`，取不到就**静默 no-op**
> （`ui_api.cpp:2612-2616`；前端 `docs/12-UIX-TAG-FIX.md` R5）。

| 能力 | 接口 | 落点 |
|---|---|---|
| 虚拟滚动 / 复用 | 自绘 + `ui_draw_push_clip` / `ui_draw_pop_clip` | `crates/zsdesk-ui/src/filelist/{viewport,draw}.rs` |
| 图标 | `IShellItemImageFactory::GetImage` → HBITMAP → BGRA 预乘 | `crates/zsdesk-windows/src/shell/icons.rs`（**直接复用**） |
| 详细信息列 | `IShellItem::GetDisplayInfo` / `IShellPropertyStore` | `crates/zsdesk-windows/src/shell/properties.rs`（新增） |
| 排序 | **Rust 侧排**（复用 `natural_cmp`） | `crates/zsdesk-app/src/sort.rs` |
| 多选 / 框选 / 反选 | 纯 Rust（`HashSet<identity_key>`） | `crates/zsdesk-app/src/selection.rs` |
| 拖放（进） | `ui_window_on_drop`（core-ui 内部已用 `DragQueryFileW` 遍历全部） | `crates/zsdesk-app/src/window/drop.rs` |
| 拖放（出） | `OleInitialize` + `SHCreateDataObject` + `DoDragDrop` | `crates/zsdesk-windows/src/ole/drop.rs`（**改造**） |
| 就地重命名 | `ui_custom_on_key_down`(F2) → 自绘编辑框 → `IFileOperation` rename | `crates/zsdesk-ui/src/filelist/rename.rs` |
| 右键菜单 | **`.uix` 声明式 `<menu>`**（core-ui build 75+ 删了 imperative 菜单构造）+ `ui_window_on_menu` 路由 | `crates/zsdesk-ui/src/filelist/context_menu.rs` |
| Shell 虚拟位置 | `SHGetKnownFolderItem` / `SHParseDisplayName` | `crates/zsdesk-windows/src/shell/shell_item.rs`（**直接复用**） |
| 视图 / 列宽 / 排序记忆 | 已有存储层 | `crates/zsdesk-app/src/storage.rs` |

## 后果

- **变容易的**：
  - 外观与 core-ui 的 Fluent 2 令牌**完全一致**，没有原生控件割裂；
  - 虚拟化完全可控 → 10,000 条目目录也能流畅滚动（目标：首屏 ≤ 120 ms、滚动 P95 ≤ 8 ms）；
  - 交互（框选、反选、就地重命名）可以按文件管理器的习惯定制，而不是受 `IExplorerBrowser` 限制。
- **变困难的**：
  - 自研虚拟列表全套交互 —— 这是 M1 的最大工作量项，也是**最大的翻车点**；
  - Shell 右键菜单的**一致性**要自己补：`IExplorerBrowser` 自带的那套菜单不再免费；
  - `explorer_browser.rs`（883 行）要拆：枚举与视图状态保留，宿主删除；
  - `namespace_tree.rs`（1,029 行）重写（数据源迁到 `crates/zsdesk-windows`，渲染迁到 `crates/zsdesk-ui/src/navtree/`）；
  - 拖出到资源管理器必须自己实现 OLE（core-ui 只在源码注释里提过支持，**无文档承诺**）。
- **引出的新决策**：
  - D07（导航树自绘）；
  - D06（预览窗格降级 —— 既然列表都自绘了，预览也顺势降级为「元信息 + 外部打开」）；
  - 门禁 **S7**（CRT 与导出面）、**S8**（绑定面）。

## 备选方案

| 方案 | 为什么没选 |
|---|---|
| 保留 `IExplorerBrowser` | 不能嵌进 core-ui 窗口（V12）；且原生控件与 Fluent 2 观感割裂 |
| 另起原生侧栏/列表进程跑 `IExplorerBrowser` | 两窗口的焦点、置顶、DPI、拖放协同全要自己做；「假一体化」；复杂度不低于自绘 |
| 用 core-ui 的 `ScrollView` + `v-for` 画列表 | `ScrollView` 非虚拟化；条目一多就卡；且数据要进 QuickJS（ADR-0001 已否决） |

> **不可变**：已接受后不得修改本文件；改变主意请新建 ADR 并标注「取代」。
