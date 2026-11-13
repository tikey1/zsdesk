# zsdesk → zsdesk-coreui 文件级迁移矩阵

> 契约编号：`ZC-MIG-001`
> 状态：已定（随 D08 生效）
> 基线：`qiu7824/zsdesk` @ main，**138 个 `.rs` 文件 / 42,150 行**（实测 `find | xargs cat | wc -l`）
> 校验工具：`spec/tools/matrix_calc.py`（重跑即可复现本文所有数字）

---

## 1. 四档定义

| 档 | 含义 | 判据 |
|---|---|---|
| **直接复用** | 原文件搬过去就能用，只需改 crate 名 / import 路径 | 文件内**不出现** ZSUI / `IExplorerBrowser` / `INameSpaceTreeControl` / ZSUI 窗口消息 |
| **改造** | 业务逻辑保留，**呈现层或依赖**要换 | 逻辑可留，但调用了 ZSUI 控件 / ExplorerBrowser / 原生树 / ZSUI 消息 |
| **重写** | 保留**意图**，代码基本重写 | 整个实现绑定在 ZSUI 的控件模型或原生 HWND 宿主上 |
| **删除** | 能力被 D02/D04/D06 明确放弃，或功能被替代 | 有明确的替代物或明确的「不做」 |

---

## 2. 总计

| 档 | 文件数 | 行数 | 占行数比 |
|---|---:|---:|---:|
| 直接复用 | 68 | 15,671 | 37.2% |
| 改造 | 56 | 20,193 | 47.9% |
| 重写 | 10 | 5,600 | 13.3% |
| 删除 | 4 | 686 | 1.6% |
| **合计** | **138** | **42,150** | 100% |

**读法**：约 **85% 的代码行不需要从零写**（直接复用 + 改造）。
需要从零写的是 **5,600 行（13.3%）** —— 这就是 D01「全量移植」的**真实新增成本**
（不含 core-ui FFI 绑定层 ~1.5k 行、不含自绘文件列表 ~2–4k 行、不含 `.uix` 页面）。

> **注意**：D01 的估算里说「新增 core-ui FFI 层 1.5–3k 行 + 自绘文件列表 2–4k 行」，
> 与本表的 5,600 行重写是**两笔不同的账**：本表算的是「上游代码的处置」，
> 新增的两笔算的是「上游没有的新代码」。两者相加才是总工程量。

---

## 3. 分 crate 汇总

| 上游 crate | 文件 | 行数 | 直接复用 | 改造 | 重写 | 删除 |
|---|---:|---:|---|---|---|---|
| `zsdesk-core` | 8 | 1,999 | 8 / 1,999 | — | — | — |
| `zsdesk-platform` | 2 | 107 | 1 / 3 | 1 / 104 | — | — |
| `zsdesk-ui` | 4 | 1,725 | — | 2 / 931 | 2 / 794 | — |
| `zsdesk-windows` | 32 | 9,205 | 18 / 3,561 | 12 / 4,429 | 1 / 1,029 | 1 / 186 |
| `zsdesk-app` | 92 | 29,114 | 41 / 10,108 | 41 / 14,729 | 7 / 3,777 | 3 / 500 |

---

## 4. 可勾选清单

> 勾选框 = 「这一项已处置完毕」。每项的「新落点」列指向 `spec/crate-layout.md` 的目录树。

### 4.1 `zsdesk-core` → `zc-core`（8 文件 / 1,999 行，**100% 直接复用**）

| ✓ | 上游文件 | 行 | 档 | 新落点 | 备注 |
|---|---|---:|---|---|---|
| ☐ | `crates/zsdesk-core/src/lib.rs` | 7 | 直接复用 | `crates/zc-core/src/lib.rs` | 只改 crate 名 |
| ☐ | `crates/zsdesk-core/src/error.rs` | 34 | 直接复用 | `crates/zc-core/src/error.rs` | |
| ☐ | `crates/zsdesk-core/src/model.rs` | 6 | 直接复用 | `crates/zc-core/src/model.rs` | |
| ☐ | `crates/zsdesk-core/src/model/explorer.rs` | 804 | 直接复用 | `crates/zc-core/src/model/explorer.rs` | `ExplorerSession` / `ExplorerTab` / `ShellLocation` / `ExplorerUi` —— **状态归属的权威模型** |
| ☐ | `crates/zsdesk-core/src/model/desktop.rs` | 794 | 直接复用 | `crates/zc-core/src/model/desktop.rs` | 桌面格子模型 |
| ☐ | `crates/zsdesk-core/src/model/tab.rs` | 131 | 直接复用 | `crates/zc-core/src/model/tab.rs` | |
| ☐ | `crates/zsdesk-core/src/model/view.rs` | 163 | 直接复用 | `crates/zc-core/src/model/view.rs` | `FileEntry` / `natural_cmp` / `RequestId` / `VisibleRange` |
| ☐ | `crates/zsdesk-core/src/path.rs` | 60 | 直接复用 | `crates/zc-core/src/path.rs` | |

**新增（上游没有）**：`crates/zc-core/src/model/preview.rs` —— 预览请求模型
（当前路径、滚动位置、缩放、瓦块级别、取消令牌）。约 150 行 [估算]。

---

### 4.2 `zsdesk-platform` → `zc-platform`（2 文件 / 107 行）

| ✓ | 上游文件 | 行 | 档 | 新落点 | 备注 |
|---|---|---:|---|---|---|
| ☐ | `crates/zsdesk-platform/src/lib.rs` | 3 | 直接复用 | `crates/zc-platform/src/lib.rs` | |
| ☐ | `crates/zsdesk-platform/src/shell.rs` | 104 | 改造 | `crates/zc-platform/src/shell.rs` | 加 `ShellItemSource` trait（供自绘树枚举虚拟位置）；`IconProvider::file_icon` 返回值从 RGBA 改 **BGRA 预乘**（core-ui 位图格式） |

---

### 4.3 `zsdesk-ui` → `zc-ui`（4 文件 / 1,725 行，**影响最高**）

| ✓ | 上游文件 | 行 | 档 | 新落点 | 备注 |
|---|---|---:|---|---|---|
| ☐ | `crates/zsdesk-ui/src/lib.rs` | 6 | 重写 | `crates/zc-ui/src/lib.rs` | 模块清单全变（加 `filelist` / `navtree` / `preview` / `shell_page` / `external`） |
| ☐ | `crates/zsdesk-ui/src/chrome.rs` | 788 | **重写** | 拆到 `crates/zc-ui/src/chrome.rs` + `ui/main.uix` | 窗口几何 / 标题栏 / 命令栏布局：ZSUI 控件模型 → core-ui `<TitleBar>` + `.uix` flex 布局 |
| ☐ | `crates/zsdesk-ui/src/menus.rs` | 644 | 改造 | `crates/zc-ui/src/menus.rs` + `ui/main.uix` 的 `<menu>` | 菜单**内容与启用规则**保留；**渲染**从 ZSUI 菜单改为 `.uix` 声明式 `<menu>`（core-ui build 75+ 删除了 imperative 菜单构造） |
| ☐ | `crates/zsdesk-ui/src/theme.rs` | 287 | 改造 | `crates/zc-ui/src/theme.rs` | 令牌**语义角色**保留；**取值**改从 core-ui `ui_theme_*()` 取（DWM 模糊等 core-ui 没有的能力降级为纯色） |

**新增（上游没有）**：`crates/zc-ui/src/filelist/**`（8 文件，约 2–4k 行 [估算]）、
`crates/zc-ui/src/navtree/**`（3 文件，约 800 行 [估算]）。

---

### 4.4 `zsdesk-windows` → `zc-windows`（32 文件 / 9,205 行）

| ✓ | 上游文件 | 行 | 档 | 新落点 | 备注 |
|---|---|---:|---|---|---|
| ☐ | `crates/zsdesk-windows/build.rs` | 23 | 直接复用 | `crates/zc-windows/build.rs` | `embed-resource` |
| ☐ | `crates/zsdesk-windows/src/lib.rs` | 23 | 直接复用 | `crates/zc-windows/src/lib.rs` | |
| ☐ | `crates/zsdesk-windows/src/desktop.rs` | 422 | 直接复用 | `crates/zc-windows/src/desktop.rs` | 桌面 Shell 集成 |
| ☐ | `crates/zsdesk-windows/src/enumerate.rs` | 167 | 直接复用 | `crates/zc-windows/src/enumerate.rs` | |
| ☐ | `crates/zsdesk-windows/src/integration.rs` | 703 | 直接复用 | `crates/zc-windows/src/integration.rs` | Shell 集成注册 |
| ☐ | `crates/zsdesk-windows/src/taskbar.rs` | 85 | 直接复用 | `crates/zc-windows/src/taskbar.rs` | |
| ☐ | `crates/zsdesk-windows/src/ole/mod.rs` | 3 | 直接复用 | `crates/zc-windows/src/ole/mod.rs` | |
| ☐ | `crates/zsdesk-windows/src/ole/drop.rs` | 115 | 改造 | `crates/zc-windows/src/ole/drop.rs` | **加 OLE 拖出**：`OleInitialize` + `SHCreateDataObject` + `IDropSource` + `DoDragDrop`（core-ui 明确支持宿主在鼠标回调里起拖） |
| ☐ | `crates/zsdesk-windows/src/shell/mod.rs` | 28 | 直接复用 | `crates/zc-windows/src/shell/mod.rs` | |
| ☐ | `crates/zsdesk-windows/src/shell/commands.rs` | 302 | 直接复用 | `crates/zc-windows/src/shell/commands.rs` | |
| ☐ | `crates/zsdesk-windows/src/shell/file_transfer.rs` | 344 | 直接复用 | `crates/zc-windows/src/shell/file_transfer.rs` | `IFileOperation` —— **D04 的核心复用点** |
| ☐ | `crates/zsdesk-windows/src/shell/file_transfer_tests.rs` | 200 | 直接复用 | `crates/zc-windows/src/shell/file_transfer_tests.rs` | |
| ☐ | `crates/zsdesk-windows/src/shell/icons.rs` | 467 | 直接复用 | `crates/zc-windows/src/shell/icons.rs` | `IShellItemImageFactory` → BGRA 预乘（**改一处转换，不改结构**） |
| ☐ | `crates/zsdesk-windows/src/shell/icons_alpha_tests.rs` | 99 | 直接复用 | `crates/zc-windows/src/shell/icons_alpha_tests.rs` | |
| ☐ | `crates/zsdesk-windows/src/shell/notify.rs` | 72 | 直接复用 | `crates/zc-windows/src/shell/notify.rs` | |
| ☐ | `crates/zsdesk-windows/src/shell/pidl.rs` | 67 | 直接复用 | `crates/zc-windows/src/shell/pidl.rs` | |
| ☐ | `crates/zsdesk-windows/src/shell/search.rs` | 52 | 直接复用 | `crates/zc-windows/src/shell/search.rs` | |
| ☐ | `crates/zsdesk-windows/src/shell/shell_item.rs` | 385 | 直接复用 | `crates/zc-windows/src/shell/shell_item.rs` | **+ 扩展**：作为自绘导航树的数据源（已知文件夹枚举） |
| ☐ | `crates/zsdesk-windows/src/shell/shell_item_contract_tests.rs` | 119 | 直接复用 | `crates/zc-windows/src/shell/shell_item_contract_tests.rs` | |
| ☐ | `crates/zsdesk-windows/src/shell/events.rs` | 24 | 改造 | `crates/zc-windows/src/shell/events.rs` | ZSUI 窗口消息 → core-ui 回调；桌面格子仍用 Win32 消息 |
| ☐ | `crates/zsdesk-windows/src/shell/explorer_browser.rs` | 883 | 改造 | 拆：枚举部分 → `zc-windows/src/shell/shell_item.rs`；宿主部分 → **删除** | `IExplorerBrowser` 不能嵌（D04）；但里面的 `IShellItemArray` 枚举与视图状态逻辑要保留 |
| ☐ | `crates/zsdesk-windows/src/shell/menu.rs` | 1,022 | 改造 | `crates/zc-windows/src/shell/menu.rs` | Shell 右键菜单宿主：保留 `IContextMenu` / `IExplorerCommand` 逻辑，弹出改由 `.uix` `<menu>` 或原生 `TrackPopupMenu` |
| ☐ | `crates/zsdesk-windows/src/shell/menu_runtime_tests.rs` | 254 | 改造 | `crates/zc-windows/src/shell/menu_runtime_tests.rs` | |
| ☐ | `crates/zsdesk-windows/src/shell/new_menu.rs` | 73 | 改造 | `crates/zc-windows/src/shell/new_menu.rs` | |
| ☐ | `crates/zsdesk-windows/src/shell/quick_menu.rs` | 61 | 改造 | `crates/zc-windows/src/shell/quick_menu.rs` | |
| ☐ | `crates/zsdesk-windows/src/shell/view_state.rs` | 312 | 改造 | `crates/zc-windows/src/shell/view_state.rs` | 保留视图状态持久化，去掉 ExplorerBrowser 专属部分 |
| ☐ | `crates/zsdesk-windows/src/flyout.rs` | 731 | 改造 | `crates/zc-windows/src/flyout.rs` | 弹出菜单宿主：ZSUI → 原生 `HWND` 或 core-ui 菜单 |
| ☐ | `crates/zsdesk-windows/src/flyout_layout.rs` | 248 | 改造 | `crates/zc-windows/src/flyout_layout.rs` | |
| ☐ | `crates/zsdesk-windows/src/gfx.rs` | 485 | 改造 | `crates/zc-windows/src/gfx.rs` | Direct2D/DirectWrite 绘制 → core-ui 自绘；**保留** GDI 位图转换（HBITMAP → BGRA） |
| ☐ | `crates/zsdesk-windows/src/theme.rs` | 221 | 改造 | `crates/zc-windows/src/theme.rs` | DWM 效果部分保留；颜色取值改 core-ui 主题 |
| ☐ | `crates/zsdesk-windows/src/shell/namespace_tree.rs` | 1,029 | **重写** | 数据源 → `zc-windows/src/shell/shell_item.rs` 扩展；渲染 → `zc-ui/src/navtree/` | `INameSpaceTreeControl` 是子窗口控件，不能嵌（D07 方案 A） |
| ☐ | `crates/zsdesk-windows/src/shell/view_background.rs` | 186 | **删除** | —— | `IExplorerBrowser` 的背景绘制，随 D04 一并作废 |

**新增（上游没有）**：`zc-windows/src/shell/thumb.rs`（缩略图，约 200 行 [估算]）、
`zc-windows/src/shell/properties.rs`（`IShellPropertyStore`，约 200 行 [估算]）。

---

### 4.5 `zsdesk-app` → `zc-app`（92 文件 / 29,114 行）

#### 4.5.1 入口与运行时（13 文件）

| ✓ | 上游文件 | 行 | 档 | 新落点 | 备注 |
|---|---|---:|---|---|---|
| ☐ | `src/main.rs` | 64 | **重写** | `crates/zc-app/src/main.rs` | 入口：`CoreUi::load_checked()` → `ui_init_with_theme` → `ui_page_open_window` → `ui_run` |
| ☐ | `src/app.rs` | 436 | 改造 | `crates/zc-app/src/app.rs` | 参数解析**原样保留**；模式分发（Explorer / Desktop / Guard / ImportTencent / QuickSearch）保留，窗口创建改 core-ui |
| ☐ | `src/commands.rs` | 119 | 改造 | `crates/zc-app/src/commands.rs` | 命令枚举保留；新增「投影到 `.uix`」的映射 |
| ☐ | `src/runtime.rs` | 257 | 改造 | `crates/zc-app/src/runtime.rs` | |
| ☐ | `src/instance.rs` | 132 | 改造 | `crates/zc-app/src/instance.rs` | 单实例：改用 core-ui 窗口唯一性判定 |
| ☐ | `src/edition.rs` | 42 | 直接复用 | `crates/zc-app/src/edition.rs` | |
| ☐ | `src/storage.rs` | 245 | 直接复用 | `crates/zc-app/src/storage.rs` | 配置 / 视图记忆持久化 |
| ☐ | `src/build.rs` | 12 | 直接复用 | `crates/zc-app/build.rs` | |
| ☐ | `src/framework_windows.rs` | 81 | **删除** | —— | ZSUI 窗口框架桥接 |
| ☐ | `src/shell_window_registry.rs` | 1,073 | 改造 | `crates/zc-app/src/shell_window_registry.rs` | |
| ☐ | `src/win_e_bridge.rs` | 380 | 改造 | `crates/zc-app/src/win_e_bridge.rs` | |
| ☐ | `examples/desktop_probe.rs` | 46 | 直接复用 | `crates/zc-app/examples/desktop_probe.rs` | |
| ☐ | `examples/enum_probe.rs` | 30 | 直接复用 | `crates/zc-app/examples/enum_probe.rs` | |

#### 4.5.2 搜索（11 文件）

| ✓ | 上游文件 | 行 | 档 | 备注 |
|---|---|---:|---|---|
| ☐ | `src/fast_search.rs` | 182 | 直接复用 | Everything / MFT / 目录扫描的统一入口 |
| ☐ | `src/fast_search/everything.rs` | 326 | 直接复用 | Everything IPC —— **与 UI 无关，全部保留** |
| ☐ | `src/fast_search/everything_runtime.rs` | 415 | 直接复用 | |
| ☐ | `src/fast_search/metadata.rs` | 227 | 直接复用 | |
| ☐ | `src/fast_search/native.rs` | 244 | 直接复用 | |
| ☐ | `src/fast_search/native_scan.rs` | 54 | 直接复用 | |
| ☐ | `src/fast_search/ntfs.rs` | 263 | 直接复用 | MFT / USN 索引 |
| ☐ | `src/fast_search/query.rs` | 87 | 直接复用 | |
| ☐ | `src/fast_search/runtime_regression.rs` | 244 | 直接复用 | |
| ☐ | `src/fast_search/snapshot.rs` | 102 | 直接复用 | |
| ☐ | `src/window/global_search.rs` | 468 | 改造 | 后端不动；搜索框与结果呈现改 core-ui |
| ☐ | `src/window/integrated_search.rs` | 216 | 直接复用 | |
| ☐ | `src/window/search_panel.rs` | 325 | 改造 | 呈现部分改 `.uix`；面板状态归 Rust |
| ☐ | `src/window/search_regression.rs` | 462 | 改造 | |
| ☐ | `src/window/everything_regression.rs` | 349 | 直接复用 | |

#### 4.5.3 快速搜索 / Listary / 转发（8 文件）

| ✓ | 上游文件 | 行 | 档 | 备注 |
|---|---|---:|---|---|
| ☐ | `src/quick_search/mod.rs` | 68 | 直接复用 | |
| ☐ | `src/quick_search/control.rs` | 313 | 改造 | 常驻窗口宿主改 core-ui |
| ☐ | `src/quick_search/resident.rs` | 95 | 直接复用 | |
| ☐ | `src/window/listary.rs` | 60 | 直接复用 | Listary 集成 |
| ☐ | `src/explorer_forwarder.rs` | 302 | 直接复用 | 文件位置转发 |
| ☐ | `src/explorer_forwarder/visibility_guard.rs` | 378 | 直接复用 | |
| ☐ | `src/reveal.rs` | 50 | 直接复用 | |
| ☐ | `src/routing_control.rs` | 1,306 | 直接复用 | 路由转发 |
| ☐ | `src/window/routing_settings.rs` | 202 | 直接复用 | |

#### 4.5.4 桌面格子（10 文件）

| ✓ | 上游文件 | 行 | 档 | 备注 |
|---|---|---:|---|---|
| ☐ | `src/desktop/mod.rs` | 1,126 | 改造 | 桌面格子**用原生窗口**（不是 core-ui），逻辑保留 |
| ☐ | `src/desktop/host.rs` | 2,681 | 改造 | 最大的文件；拆分为 `desktop/host.rs` + `desktop/grid.rs` |
| ☐ | `src/desktop/fence.rs` | 713 | 直接复用 | 围栏逻辑 |
| ☐ | `src/desktop/migration.rs` | 586 | 直接复用 | 布局迁移 |
| ☐ | `src/desktop/tencent_import.rs` | 667 | 直接复用 | 腾讯布局导入 |
| ☐ | `src/desktop/return_to_desktop.rs` | 249 | 直接复用 | |
| ☐ | `src/desktop/menu_worker.rs` | 396 | 直接复用 | |
| ☐ | `src/desktop/context_menu_regression.rs` | 133 | 改造 | |
| ☐ | `src/desktop/layer_regression.rs` | 83 | 改造 | |
| ☐ | `src/desktop/window_message_regression.rs` | 152 | 改造 | |

#### 4.5.5 右键菜单宿主（8 文件）

| ✓ | 上游文件 | 行 | 档 | 备注 |
|---|---|---:|---|---|
| ☐ | `src/context_menu_process.rs` | 905 | 改造 | 右键菜单宿主进程；弹出改 core-ui 菜单 / 原生菜单 |
| ☐ | `src/context_menu_process_tests.rs` | 285 | 改造 | |
| ☐ | `src/context_menu_background.rs` | 27 | 改造 | |
| ☐ | `src/context_preferences.rs` | 21 | 直接复用 | |
| ☐ | `src/flyout_menu.rs` | 136 | 改造 | |
| ☐ | `src/window/context_menu.rs` | 140 | 改造 | |
| ☐ | `src/window/context_menu_regression.rs` | 171 | 改造 | |

#### 4.5.6 窗口与面板（`src/window/**`，39 文件）

| ✓ | 上游文件 | 行 | 档 | 备注 |
|---|---|---:|---|---|
| ☐ | `src/window/mod.rs` | 1,511 | **重写** | 窗口宿主核心：ZSUI 窗口 → core-ui 页面 |
| ☐ | `src/window/paint.rs` | 534 | **重写** | Direct2D 绘制 → core-ui 自绘 |
| ☐ | `src/window/input.rs` | 895 | **重写** | 输入分发：ZSUI → core-ui 回调 |
| ☐ | `src/window/dual_pane.rs` | 639 | **重写** | 双栏宿主 → `.uix` flex 布局；导航/选择逻辑拆出保留 |
| ☐ | `src/window/mouse_gestures.rs` | 93 | **重写** | 手势判定 → core-ui 鼠标回调 |
| ☐ | `src/window/wheel_input.rs` | 41 | **重写** | 滚动 → 自绘列表的虚拟滚动 |
| ☐ | `src/window/address_input.rs` | 45 | 改造 | 地址栏输入 → `.uix` `TextInput` |
| ☐ | `src/window/bookmarks.rs` | 233 | 直接复用 | 收藏夹 |
| ☐ | `src/window/deferred_selection.rs` | 41 | 改造 | |
| ☐ | `src/window/diagnostics.rs` | 377 | 改造 | 加 core-ui 启动自检报告（版本 / 能力 / revision 戳记） |
| ☐ | `src/window/folder_sizes.rs` | 387 | 直接复用 | |
| ☐ | `src/window/geometry.rs` | 77 | 改造 | 窗口几何 → core-ui `<window>` 属性 + `ui_window_set_rect` |
| ☐ | `src/window/geometry_regression.rs` | 337 | 改造 | |
| ☐ | `src/window/hover_preview.rs` | 374 | 改造 | 触发逻辑保留；渲染 → `zc-ui/src/preview/` |
| ☐ | `src/window/image_decode.rs` | 272 | 直接复用 | **WIC 解码**（实测 zsdesk 不用 ghde.dll） |
| ☐ | `src/window/interaction_regression.rs` | 390 | 改造 | |
| ☐ | `src/window/keyboard.rs` | 150 | 改造 | 快捷键 → `ui_window_on_key` |
| ☐ | `src/window/mouse_navigation.rs` | 67 | 直接复用 | |
| ☐ | `src/window/navigation.rs` | 459 | 改造 | |
| ☐ | `src/window/pane_input.rs` | 148 | 改造 | |
| ☐ | `src/window/preview_handlers.rs` | 133 | 改造 | |
| ☐ | `src/window/preview_lifecycle_regression.rs` | 107 | 改造 | |
| ☐ | `src/window/preview_process.rs` | 351 | 改造 | |
| ☐ | `src/window/preview_selection.rs` | 101 | 直接复用 | |
| ☐ | `src/window/quick_preview.rs` | 791 | 改造 | |
| ☐ | `src/window/quicklook.rs` | 108 | 改造 | |
| ☐ | `src/window/quicklook_process_tests.rs` | 113 | 改造 | |
| ☐ | `src/window/selection_regression.rs` | 451 | 改造 | |
| ☐ | `src/window/shell_registration.rs` | 133 | 直接复用 | |
| ☐ | `src/window/shell_reveal_regression.rs` | 155 | 直接复用 | |
| ☐ | `src/window/startup_settings.rs` | 151 | 直接复用 | |
| ☐ | `src/window/terminal.rs` | 151 | 改造 | |
| ☐ | `src/window/thumbnail_regression.rs` | 152 | 改造 | |
| ☐ | `src/window/view_memory.rs` | 250 | 直接复用 | 视图记忆 / 列宽 / 排序记忆 |
| ☐ | `src/window/window_actions.rs` | 170 | 改造 | |
| ☐ | `src/window/wps_preview.rs` | 269 | **删除** | D06：Office/WPS 进程内预览降级为「用系统程序打开」 |
| ☐ | `src/window/wps_render_process.rs` | 150 | **删除** | 同上 |

---

## 5. 删掉的 4 个文件（686 行）—— 明确的能力放弃

| 文件 | 行 | 放弃的能力 | 替代物 | 决策 |
|---|---:|---|---|---|
| `zsdesk-windows/src/shell/view_background.rs` | 186 | ExplorerBrowser 列表背景绘制 | 自绘列表背景 | D04 |
| `zsdesk-app/src/framework_windows.rs` | 81 | ZSUI 窗口框架桥接 | core-ui 窗口 | D04 |
| `zsdesk-app/src/window/wps_preview.rs` | 269 | Office/WPS 进程内预览 | 「用系统程序打开」/ GuoheView.exe | D06 |
| `zsdesk-app/src/window/wps_render_process.rs` | 150 | WPS 渲染子进程 | 同上 | D06 |

> 这 686 行是**有意放弃**的，不是遗漏。每一条都在 `DECISIONS.md` 里有对应的决策记录。

---

## 6. 新增代码清单（上游没有的）

| 新增 | 落点 | 规模 [估算] | 里程碑 |
|---|---|---:|---|
| core-ui 动态绑定层 | `crates/zc-ffi-coreui/` | 1,200–1,800 行 | M0 |
| 自绘文件列表 | `crates/zc-ui/src/filelist/` | 2,000–4,000 行 | M1 |
| 自绘导航树 | `crates/zc-ui/src/navtree/` | 600–1,000 行 | M1 |
| `.uix` 页面 | `ui/*.uix` | 800–1,500 行 | M1–M2 |
| 预览窗格 | `crates/zc-ui/src/preview/` | 500–900 行 | M2 |
| Shell 缩略图 / 属性 | `crates/zc-windows/src/shell/{thumb,properties}.rs` | 300–500 行 | M1 |
| 核心 UI 胶水（状态投影） | `crates/zc-ui/src/shell_page.rs` + `crates/zc-app/src/view_model.rs` | 500–900 行 | M1 |

**合计新增**：约 5,900–10,600 行 [估算]。

---

## 7. 风险：哪几项最可能超工期

| # | 项 | 风险 | 早期探测手段 |
|---|---|---|---|
| 1 | `zsdesk-app/src/window/mod.rs`（1,511 行）重写 | 窗口宿主耦合 ZSUI 最深 | M0 先做一个**最小窗口**（空白页 + 一个按钮 + 一个 `ui_custom` 画矩形），证明 core-ui 能承载 Rust 回调 |
| 2 | 自绘文件列表 | 交互细节多（框选/拖放/重命名） | M1 第一周只做「滚动 + 画行 + 单击选中」，量出每项成本再排后续 |
| 3 | `zsdesk-app/src/desktop/host.rs`（2,681 行） | 桌面格子是独立子系统 | M3 才碰；M0–M2 不动它 |
| 4 | 声明式右键菜单的时序 | `ui_page_set_json` 与菜单弹出的先后 | M1 必须实测（见 `spec/milestones.md` V1.6） |
| 5 | OLE 拖出 | core-ui 的 `WM_CAPTURECHANGED` 处理未见文档承诺 | M1 做一个最小 DoDragDrop 探针（见 `spec/milestones.md` V1.7） |

---

## 8. 关联文档

- `spec/crate-layout.md` —— 新落点的目录树与依赖方向
- `spec/ffi-core-ui.md` —— 自绘列表 / 导航树 / 菜单的具体落点
- `spec/milestones.md` —— 按里程碑切分本矩阵
- `spec/tools/matrix_calc.py` —— 重跑本矩阵统计
