# 里程碑 M0~M3

> 契约编号：`ZC-MILESTONE-001`
> 状态：已定（随 D11 生效）
> 路线图映射：`docs/10-FORK-ROADMAP-COREUI.md` §4 的 P1~P4
> **决策优先级**：本次对话中的选择 > 用户 fork 路线图 > 本 Spec 建议。冲突处以对话为准并标注「覆盖路线图 §x」。

---

## 0. 映射总表

| 里程碑 | 路线图阶段 | 主题 | 核心可交付物 |
|---|---|---|---|
| **M0** | **P1** | 仓库骨架 + CI 绿灯 + 门禁 + 绑定骨架 | 能编译、能测试、门禁全绿的空壳仓库；core-ui.dll 由 CI 从源码构建并通过 S7 校验 |
| **M1** | **P2** | 窗口与文件浏览 | core-ui 窗口 + `.uix` 静态布局 + Rust 自绘文件列表 + 自绘导航树 + 标签/地址栏/状态栏 |
| **M2** | **P3** | **GuoheView 外部调用 + 降级通道 + 错误提示** | 「用 GuoheView 打开」入口 + 未安装/未注册关联的降级提示 + 预览窗格降级内容 —— **覆盖路线图 §4 P3** |
| **M3** | **P4** | 其余功能迁移 + 完整回归 + 发布 | 搜索/Everything/桌面格子/右键菜单宿主/网络版/Listary/腾讯布局迁移 + 完整回归 + 便携包 + Release |

> **M0 的存在理由**：本机**无 cargo / rustc / cmake / MSVC**（实测）。
> 一切编译与运行验证只能靠 CI → **CI 必须第一版就绿**，否则后面每一步都在盲写。

---

## 1. M0 —— 仓库骨架 + CI 绿灯（↔ P1）

### 1.1 产出物

| # | 产出物 | 落点 |
|---|---|---|
| M0-1 | workspace 骨架：6 个 crate 的 `Cargo.toml` + 空 `lib.rs` | `crates/zc-{core,platform,windows,ffi-coreui,ui,app}/` |
| M0-2 | **core-ui 动态绑定骨架**（手写 extern + 必需/可选符号表 + 版本协商 + UI 线程收敛） | `crates/zc-ffi-coreui/`（原型见 `spec/rust-ffi-skeleton/`） |
| M0-3 | `packaging/core-ui-revision.txt`（= `1840d689b51a32d392dbf06b8d594aef96696a80`） | `packaging/` |
| M0-4 | `.github/workflows/build.yml`（3 job：rust / core-ui / package）+ `scripts/check-coreui-dll.ps1` + `scripts/check-coreui-api.py` + `scripts/gates/check_crate_dag.py` | `.github/workflows/`、`scripts/` |
| M0-5 | 机械门禁脚本与**首次基线**（`scripts/gates/verify.py --phase 3` 的输出落盘） | `scripts/gates/` |
| M0-6 | `assets/*.uix` 的**标签机械修正**（`<text>`→`<label>`、`<hbox>`/`<vbox>`/`<panel>`→`<div>`、`<text-input>`→`<input>`、`<vseparator>`→`<div>`），**保持 D-15 静态布局 only 不变** | `assets/` |
| M0-7 | 最小可运行 exe：`zc-app.exe` 能启动 → 加载 core-ui.dll → 打开 `.uix` 页面 → 显示一个空的自绘矩形 | `crates/zc-app/src/main.rs` |

### 1.2 验收标准

| # | 标准 | 判定方式 |
|---|---|---|
| V0.1 | `cargo test --workspace --release --locked` 全绿，**且测试过程中不访问 `core-ui.dll`** | CI job `rust` |
| V0.2 | `cargo build --locked --release -p zc-app --features network,desktop` 成功 | CI job `rust` |
| V0.3 | `cargo clippy -- -D warnings` 零告警 | CI job `rust` |
| V0.4 | CI 能从 `packaging/core-ui-revision.txt` 精确 checkout core-ui 并构建出 `core-ui.dll` | CI job `core-ui` |
| V0.5 | **S7 通过**：PE machine = `0x8664`；导入表含 `VCRUNTIME140.dll` + `MSVCP140.dll`（动态 CRT）；导出面差异只落在两张已知清单内 | `scripts/check-coreui-dll.ps1` |
| V0.6 | **S8 通过**：必需符号 ⊆ 567 个 `ui_*`；必需 ∩ 可选 = ∅；必需符号不含 12 个「本机独有」符号 | `scripts/check-coreui-api.py` |
| V0.7 | **S9/S10 通过**：crate 依赖图无环；`zc-ffi-coreui` 无任何 `zc-*` 路径依赖 | `scripts/gates/check_crate_dag.py` |
| V0.8 | 便携包内 `core-ui.dll` 与 exe 同目录，revision 戳记一致 | `scripts/check-coreui-dll.ps1 -RequireBesideExe` |
| V0.9 | **关键错误流**：移走 `core-ui.dll` 后启动 → **非零码退出** + stderr 打印完整候选列表（不静默回退到 `C:\Program Files\GuoheView\core-ui.dll`） | 人工 + CI 脚本 |
| V0.10 | 静态门禁 G01/G03/G04/G08 + 工程门禁 E 段 P0 全绿 | `scripts/gates/verify.py --phase 3` |

### 1.3 可演示的动作

1. 打开 CI 的 Actions 页面 → `Windows build` 全绿（3 个 job）。
2. 下载 `ZSDesk-full-x64` 便携包 → 解压 → 双击 `zc-app.exe` → **出现一个带标题栏的空窗口**，
   窗口里有一个 Rust 侧 `ui_custom_on_draw` 画出来的灰色矩形。
3. 把 `core-ui.dll` 改名 → 再启动 → 弹出明确的错误提示（列出 tried 过的路径）。

### 1.4 依赖前置

| 前置 | 状态 |
|---|---|
| fork 的 Actions 已启用 | ✅ **已实测**（commit `4a48349`，`Windows build #1` 跑绿 9m58s） |
| core-ui revision 固定 | ✅ M0-3 |
| core-ui 的 C++ 构建链（VS 2022 + LLVM + CMake + Ninja） | ⚠️ CI runner 自带 VS 2022 / CMake / LLVM，**Ninja 需 `choco install ninja`** |
| Rust 工具链 | ⚠️ 本机无，**CI 提供** |

### 1.5 M0 阶段要解决的 **U 项**（ABI-NOTES §2）

M0 的最小闭环**必须**按 ABI-NOTES 建议的顺序，逐个用真机最小探针验证：

```
ui_core_version*  →  ui_init_with_theme  →  ui_page_load_string  →  ui_page_open_window  →  ui_run
   （U01）              （U01）              （U02）                 （U03）
```

> **不得凭导出名反推参数**（ABI-NOTES 硬规则 3）。
> **每接入一组 API，必须先过对应的 U 项最小验证**。
> U05（`ui_custom_on_draw` 回调签名/绘制子集/回调线程）必须在 M1 开始前过。

---

## 2. M1 —— 窗口与文件浏览（↔ P2）

### 2.1 产出物

| # | 产出物 | 落点 |
|---|---|---|
| M1-1 | `.uix` 静态布局落地（修正后的 `main.uix` 标签）+ **Rust 侧 `ui_widget_find_by_id` 取句柄** | `assets/main.uix`、`crates/zc-ui/src/shell_page.rs` |
| M1-2 | **Rust 自绘文件列表**：虚拟滚动 + 命中测试 + 单击/多选/框选/反向选择 + 图标 + 详细信息列 + 排序 | `crates/zc-ui/src/filelist/` |
| M1-3 | **Rust 自绘导航树**（已知文件夹 + 快速访问 + 展开收起） | `crates/zc-ui/src/navtree/` |
| M1-4 | 标签条 / 地址栏 / 状态栏（Rust 侧状态 → `.uix` 小载荷投影） | `crates/zc-app/src/view_model.rs` |
| M1-5 | `IShellItemArray` 枚举 + `IShellItemImageFactory` 取图标（BGRA 预乘）+ LRU 缓存 | `crates/zc-windows/src/shell/{shell_item,icons}.rs` |
| M1-6 | `IShellPropertyStore` 详细信息列 | `crates/zc-windows/src/shell/properties.rs` |
| M1-7 | 视图记忆 / 列宽 / 排序记忆 | `crates/zc-app/src/storage.rs`（沿用上游 `window/view_memory.rs`） |
| M1-8 | 核心 UI 迁移的第一批 `zsdesk-app` 重写（`window/mod.rs` / `paint.rs` / `input.rs` / `mouse_gestures.rs` / `wheel_input.rs`） | `crates/zc-app/src/window/` |

> **首版只读**（D-15）：命令条不含新建 / 重命名 / 删除 / 剪贴板命令。

### 2.2 验收标准

| # | 标准 | 判定方式 |
|---|---|---|
| V1.1 | 双击 exe 出现主窗口，标题栏 / 标签条 / 地址栏 / 状态栏 / 导航树 / 列表区布局与 `docs/07-UI-DESIGN-WINUI3.md` 一致 | 截图（`ui_debug_screenshot`） |
| V1.2 | 打开一个含 10,000 个文件的目录 → 首屏可见行 ≤ 120 ms；滚动 P95 ≤ 8 ms | 埋点计时 |
| V1.3 | 列表滚动流畅，**只画可见行**（draw 回调里的行数 ≤ 可见行数 + 4） | draw 回调计数断言 |
| V1.4 | 单击选中 / Ctrl 多选 / 框选 / Ctrl+单击反向选择，行为与资源管理器一致 | 人工 + `ui_debug_click` 脚本 |
| V1.5 | 详细信息列（名称 / 大小 / 类型 / 修改时间）取值正确 | 与资源管理器对照 |
| V1.6 | **右键菜单**：`.uix` 声明式 `<menu>` + Rust `ui_page_set_json` 推菜单项 + `ui_window_on_menu` 路由，**弹出前读到最新值** | **实测时序**（本项是最大的时序风险） |
| V1.7 | **OLE 拖出探针**：在文件列表里按住一个文件拖到桌面 → 桌面能放下（`DoDragDrop` 成功） | **最小探针**，core-ui 未文档承诺 `WM_CAPTURECHANGED` 行为 |
| V1.8 | 导航树能展开「此电脑 / 回收站 / 快速访问 / 网络」，点击能导航 | 人工 |
| V1.9 | **深浅色切换不突变**（D10 的 token 路线生效） | 截图对比 |
| V1.10 | 全部门禁（含新增的 S8/S9/S10）全绿 | CI |

### 2.3 可演示的动作

1. 启动 → 主窗口出现 → 左侧导航树可展开 → 点「此电脑」→ 中间列表显示磁盘。
2. 双击某个目录 → 列表滚动流畅 → 点一个文件 → 状态栏显示「已选 1 项 / 12.3 MB」。
3. 在列表空白处右键 → 弹出上下文菜单（**禁用项是灰的**）→ 菜单项随选中状态变化。
4. Ctrl 选 3 个文件 → 框选一片 → Ctrl+单击其中一个 → 它被取消选中。
5. 从列表里把一个文件拖到桌面 → 桌面出现该文件。

### 2.4 依赖前置

| 前置 | 状态 |
|---|---|
| M0 全部完成（CI 绿、绑定可加载） | 依赖 M0 |
| U02（`ui_page_load_file` 的路径编码与失败返回值）已核对 | M0 完成 |
| U05（`ui_custom_on_draw` 签名 / 绘制子集 / 回调线程）已核对 | M0 完成 |
| U06（`ui_widget_on_mouse_*` / `on_key` 的注册注销语义、回调内可否再调 `ui_*`）已核对 | M0 完成 |
| U07（`ui_bitmap_create` 的像素格式 BGRA/RGBA、stride、生命周期）已核对 | M0 完成 |
| M04（`var(--token)` 自定义 CSS 变量支持） | ✅ 已由源码级核实为**支持**（`cascade.cpp:42-54` / `value.h:91`） |

### 2.5 M1 覆盖路线图 §4 P2 的说明

路线图 P2 要求「在**全量功能基线上**按 crate 分阶段拆 `zsui` path 依赖……保持可编译」。
→ 本阶段**不删** `zsui` 依赖（C4：过渡期保留双通道），只是**新增** core-ui 通道；
删除动作在 M3 收口。**覆盖路线图 §4 P2 的「保持可编译」为「保持可编译且不删 zsui 依赖」**。

---

## 3. M2 —— GuoheView 外部调用 + 降级通道 + 错误提示（↔ P3）

> ### ⚠️ **覆盖路线图 §4 P3**
>
> 路线图 §4 P3 原文：**「GuoheView 强关联图片预览（复用 `preview_process` / `quicklook` /
> `image_decode` 链路），补降级通道与错误提示」**。
>
> **本次对话 D02 已拍板「只外部调 exe」** → 原 P3 的「复用进程内预览链路」**作废**。
> 本阶段的 P3 内容改为：**GuoheView 外部调用 + 降级通道 + 错误提示**，
> **不移植进程内解码链路**（`preview_process` / `quicklook` / `quick_preview` / `hover_preview`
> 的**宿主部分不接入**）。
>
> 保留：`image_decode.rs` 的 **WIC 解码实现**（实测零 UI 依赖，直接复用）**留在仓库**，
> 作为 M3+ 可选能力，但**不接入**本阶段的预览路径。

### 3.1 产出物

| # | 产出物 | 落点 |
|---|---|---|
| M2-1 | **GuoheView 外部调用入口**：优先走系统文件关联 `ShellExecuteW(hwnd, L"open", <文件>, …)`；兜底显式指定 `C:\Program Files\GuoheView\GuoheView.exe` + 参数 | `crates/zc-ui/src/external.rs` |
| M2-2 | **可检性检查**：`GuoheView` 未安装 / 未注册文件关联 / exe 存在但启动失败 → 三种可区分的结论 | `crates/zc-ui/src/external.rs` |
| M2-3 | **降级通道**：未安装 → 提示 + 提供「用系统默认程序打开」/「在资源管理器中显示」两个替代动作；不静默失败 | 同上 + `crates/zc-app/src/window/` |
| M2-4 | **错误提示文案**（人话 + 下一步动作，不用「错误 0x80070002」） | `crates/zc-ui/src/external.rs` + `.lang` |
| M2-5 | **预览窗格降级内容**：图标 + 元信息（名称/大小/类型/时间）+ 纯文本预览（`.txt/.md/.log/.json/.rs` 等）+ 「用 GuoheView 打开」按钮 | `crates/zc-ui/src/preview/` |
| M2-6 | 外部调用的**超时与失败归一化**（不阻塞 UI 线程） | `crates/zc-windows/src/` |
| M2-7 | 安全基线落地：调用参数**数组化**（不拼命令行）、路径规范化、失败日志脱敏 | `crates/zc-ui/src/external.rs` |

### 3.2 验收标准

| # | 标准 | 判定方式 |
|---|---|---|
| V2.1 | 选中一张图片 → 「用 GuoheView 打开」→ GuoheView 启动并打开该文件 | 人工（真机） |
| V2.2 | **未安装 GuoheView** → 明确的提示 + 两个替代动作；**不静默失败、不白屏** | 人工（改注册表或用干净 VM） |
| V2.3 | **已安装但未注册文件关联** → 走 exe 兜底路径；仍失败 → 明确报错「已找到 exe 但启动失败，错误码 0x…」 | 人工 |
| V2.4 | 预览窗格显示图标 + 元信息；文本文件显示前 N KB 纯文本 | 截图 |
| V2.5 | 外部调用**不阻塞 UI 线程**：GuoheView 启动慢时列表仍可滚动 | 埋点 |
| V2.6 | 调用参数**未拼接**为命令行字符串（`lpParameters` 用数组/独立参数） | 代码审查 + E02 门禁 |
| V2.7 | 失败日志不含用户目录完整路径（脱敏） | E16 门禁 |
| V2.8 | 全部门禁全绿 | CI |

### 3.3 可演示的动作

1. 选中一张 JPG → 点「用 GuoheView 打开」→ GuoheView 弹出并显示该图。
2. 在一台**没装 GuoheView** 的机器上做同一操作 → 弹出降级提示 → 点「用系统默认程序打开」→ 照片查看器打开。
3. 预览窗格右侧显示选中文件的图标 + 「1.2 MB / JPEG / 2026-11-10 15:04」。
4. 选中一个 `.md` 文件 → 预览窗格显示文件前几 KB 的文本。

### 3.4 依赖前置

| 前置 | 状态 |
|---|---|
| M1 完成 | 依赖 M1 |
| `GuoheView.exe` 的命令行参数形式 | ⚠️ **仍未核实**（本机无文档）→ M2 的第一步就是**实测**：用 `ShellExecuteW` 传路径，观察 GuoheView 是否正确打开；若不认，则改走「exe + 参数」兜底并记录实际参数形式 |
| 文件关联注册状态的可检性 | M2-2 自行实现（读注册表 `HKCR\.<ext>` + `HKCR\<progid>\shell\open\command`） |

---

## 4. M3 —— 其余功能迁移 + 完整回归 + 发布（↔ P4）

### 4.1 产出物

| # | 产出物 | 对应上游链路 |
|---|---|---|
| M3-1 | **搜索**：自研 MFT / USN 索引 + 目录扫描 + Everything 后端 | `fast_search/**`、`window/global_search.rs`、`window/search_panel.rs` |
| M3-2 | **快速搜索常驻**（托盘 / 全局快捷键） | `quick_search/**` |
| M3-3 | **桌面格子**（desktop fences） | `desktop/**`（10 文件，最大 `host.rs` 2,681 行） |
| M3-4 | **右键菜单宿主** | `context_menu_process.rs`、`window/context_menu.rs`、`flyout_menu.rs` |
| M3-5 | **网络版**（FTP / SFTP / UNC） | `zsdesk-windows/src/integration.rs` + `network` feature |
| M3-6 | **Listary 集成**（C# 插件，`native/listary`） | `window/listary.rs` |
| M3-7 | **腾讯桌面布局迁移** | `desktop/migration.rs`、`desktop/tencent_import.rs` |
| M3-8 | **文件位置转发** + Win+E 桥接 | `explorer_forwarder.rs`、`win_e_bridge.rs`、`routing_control.rs` |
| M3-9 | **收口**：移除 `zsui` path 依赖（C4 过渡期结束） | `crates/zc-app/Cargo.toml` |
| M3-10 | 完整回归 + 便携包 + Release + SHA-256 manifest | `scripts/package-portable.ps1`、`.github/workflows/release.yml` |

### 4.2 验收标准

| # | 标准 | 判定方式 |
|---|---|---|
| V3.1 | `full` edition 的 **build / test / clippy 全绿**，且**已无 `zsui` 依赖** | CI + `Cargo.toml` 检查 |
| V3.2 | 搜索：Everything 后端可用时走 Everything，不可用时自动回退 MFT/USN → 目录扫描 | 人工 + `window/everything_regression.rs` 回归 |
| V3.3 | 桌面格子：可创建 / 移动 / 恢复；退出时图标状态还原 | 人工（上游有 `desktop/layer_regression.rs`） |
| V3.4 | 右键菜单宿主：资源管理器右键 → 本程序打开并定位选中项 | 人工（上游有 `window/shell_reveal_regression.rs`） |
| V3.5 | Listary：能搜到本程序并跳转 | 人工（需装 Listary） |
| V3.6 | 腾讯布局导入：导入后桌面格子布局与源一致 | 人工 |
| V3.7 | 便携包 `ZSDesk-<version>-full-x64.zip` 生成，manifest 逐文件 `bytes + SHA-256` 可校验 | CI |
| V3.8 | 强制校验 VC++ x64 Redistributable 通过 | `scripts/package-portable.ps1` |
| V3-9 | 真机冒烟：全新 Windows 10/11 x64 机器上解压即用（只需已装 GuoheView） | 人工（≥2 台机器） |

### 4.3 可演示的动作

1. 按全局快捷键 → 快速搜索窗口弹出 → 输入关键字 → 命中结果 → 回车在文件管理器中打开。
2. 右键桌面 → 新建桌面格子 → 把几个文件拖进去 → 退出程序 → 图标状态还原。
3. 在资源管理器里右键一个文件夹 → 「在 ZSDesk 中打开」→ 程序打开并定位。
4. 导入腾讯桌面布局 → 桌面格子按源布局生成。

### 4.4 依赖前置

| 前置 | 状态 |
|---|---|
| M2 完成 | 依赖 M2 |
| Everything 未安装时的回退路径已验证 | M3-1 |
| Listary / 腾讯布局的**外部依赖**（用户是否装了 Listary、是否有腾讯布局样本） | ⚠️ 需用户确认；**降级路径**：功能不可用时给出明确提示，不静默失败 |

---

## 5. 里程碑之间的硬依赖（不可并行）

```
M0 ──► M1 ──► M2 ──► M3
 │      │      │      │
 └ 绑定骨架         └ 收口：删 zsui 依赖（C4 结束）
   + CI 绿
```

**不允许并行开工的理由**：M1 的自绘列表依赖 M0 的绑定层；M2 的外部调用依赖 M1 的预览宿主；
M3 的「删 zsui」依赖 M1+M2 全部迁完。**并行会产出「改坏了也不知道是哪一步」的中间态**，
而本机无法编译验证（只能靠 CI），一次合并多个阶段的失败无法二分定位。

---

## 6. 每个里程碑的「不做什么」（防镀金）

| 里程碑 | 明确不做 |
|---|---|
| M0 | 不做任何业务功能；不改 `crates/**` 下的上游 `.rs`（只新建 `zc-*`）；不删 `zsui` 依赖 |
| M1 | 不做写操作（新建/重命名/删除/剪贴板）；不做右键菜单的动作项（只验证弹出与置灰）；不做预览；不做搜索；不做桌面格子 |
| M2 | 不做进程内图片解码；不做 Office/WPS/PDF 进程内渲染；不做 EXIF 面板 |
| M3 | 不做 32 位 x86；不做 `standard`/`network`/`desktop`/`themed`/`desktop-box` 单版 |

---

## 7. 端到端验证（每个里程碑收尾时跑一遍）

```powershell
# 通用
cargo test --workspace --release --locked
python3 scripts/gates/verify.py --phase 3
python3 scripts/gates/gates_engineering.py --min-sev P0
python3 scripts/gates/check_crate_dag.py
python3 scripts/check-coreui-api.py --exports docs/05-COREUI-EXPORTS.txt

# M0 追加
pwsh scripts/check-coreui-dll.ps1 -DllPath core-ui/clang-cl-build/core-ui.dll `
    -ExpectedRevision (Get-Content packaging/core-ui-revision.txt -Raw).Trim() `
    -ExpectedExports docs/05-COREUI-EXPORTS.txt

# M2 追加（关键错误流）
#   未装 GuoheView → 必须给出降级提示，不得静默失败
#   （用一台干净 VM 或临时改 PATH / 注册表关联）

# M3 追加
pwsh scripts/package-portable.ps1 -StageDir stage -OutFile ZSDesk-<ver>-full-x64.zip
#   → 校验 manifest 的每个 SHA-256
```

---

## 8. 关联文档

- `SPEC.md` —— 总览与索引
- `spec/crate-layout.md` —— crate 划分与目录树
- `spec/ffi-core-ui.md` —— FFI 契约
- `spec/migration-matrix.md` —— 文件级复用矩阵（M1~M3 按本表切分）
- `spec/ci-contract.md` —— CI 步骤级契约
- `docs/10-FORK-ROADMAP-COREUI.md` —— 用户 fork 路线图（被覆盖处已标注）
