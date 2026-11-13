# DECISIONS 决策台账

> 每次用户拍板就追加一条，写明「谁、何时、选了啥、代价是什么」。
> 状态：`已定` / `待定` / `已推翻`。

---

## 决策优先级（2026-11-13 用户明确）

> **本次对话中的选择 > 用户 fork 路线图（`docs/10-FORK-ROADMAP-COREUI.md`）> 交付总监建议。**
> 一致时不必重述；**冲突时以本次对话为准**，并在下游产出里标注「覆盖路线图 §x」。

---

## D00 用户已明确的前提（对话原话直取）

| # | 决策 | 状态 | 备注 |
|---|---|---|---|
| D00-1 | 基准项目 = `qiu7824/zsdesk` | 已定 | MIT，可 fork/改造 |
| D00-2 | UI 层改为 `ghboke/core-ui`，替换 ZSUI | 已定 | core-ui 为 MIT |
| D00-3 | 图片预览与本机 `C:\Program Files\GuoheView` 强关联 | 已定 | 关联形态见 D02 |
| D00-4 | 界面风格 Windows 10 / WinUI 3 / Fluent，符合用户习惯 | 已定 | core-ui 自绘 Fluent 2，**不引入 WinUI 3 运行时** |
| D00-5 | 代码以 Rust 为主，必要时调用用户已装的 C++ v14 运行库 | 已定 | v14 = `msvcp140`/`vcruntime140` 系列，已实测在位；`/MD` 动态 CRT 与 Rust MSVC 默认一致 |
| D00-6 | 代码编写与审查沿用两个上游库的既有规范 | 已定 | 落到 revision 固定文件 + CI 检查 + 用户既有 Gate 0~4 / R1~R12 |
| D00-7 | 只做 x86-64，不做 32 位 x86 | 已定 | CI runner 与产物均仅 x64 |

---

## D01–D04 本次对话已拍板（优先于路线图）

### D01 功能范围 → **全量移植 zsdesk**（= 路线图 C3，一致）
含多标签、收藏分类、双栏、自研 MFT/USN 搜索 + Everything 后端、桌面格子、右键菜单宿主、Listary 集成、腾讯桌面布局迁移、网络版（FTP/SFTP/UNC）、文件位置转发。**功能不裁剪**，只在交付形态上收窄（见 D05）。

**代价（已量化）**：zsdesk 约 **42k 行 Rust**（138 个 `.rs` 文件）。core-ui 自绘路线下，Shell 原生控件全部保不住：
- `zsdesk-core`(1,999 行) / `zsdesk-platform`(107 行) / `zsdesk-windows`(9,205 行) → **可复用**（Shell 操作、搜索、Everything、转发、腾讯迁移）；
- `zsdesk-ui`(1,725 行) → **重写**（ZSUI 是其直接依赖；且需补上 ZSUI 原有的「虚拟列表 + 原生窗口」能力）；
- `zsdesk-app`(29,114 行) 的呈现层（`window/`、`desktop/`、`flyout_menu.rs`、`theme.rs`、`chrome.rs`）→ **重写**；
- 新增 core-ui FFI 层 + 自绘虚拟列表 + `.uix` 界面。［估算］

### D02 GuoheView 关联形态 → **只外部调 exe**
`ShellExecute` 拉起 `C:\Program Files\GuoheView\GuoheView.exe`。
**覆盖路线图 §4 P3 的「复用 `preview_process`/`quicklook`/`image_decode` 链路」** —— 不链接 `ghde.dll`、不做进程内解码。

**好处**：`ghde.dll` 无公开 ABI 的风险整体消除（实测它有 323 个扁平 C ABI 导出但无头文件）。

**代价**：
1. 图片**进程内**预览能力归零：缩略图、EXIF、缩放、翻页都不能集成进自家界面。
2. 上游预览链路（`preview_process` / `quicklook` / `quick_preview` / `hover_preview` / `preview_handlers` / `preview_selection` / `image_decode` / `thumbnail_regression` / `paint`）的宿主是**原生窗口**，而 core-ui **不可嵌入原生 HWND 子控件**（实测）→ 这批模块必须逐个四档判定。
3. 非图片类（Office / WPS / 文本 / PDF）的预览去向尚未指定 → 已在 Spec 中要求给三个候选并推荐。

**调用方式（已实测可行）**：优先走系统文件关联 `ShellExecuteW(hwnd, L"open", <文件>, NULL, ...)`；兜底显式指定 exe（`lpFile` = exe，`lpParameters` = 文件路径）。两条路径都要做「GuoheView 未安装 / 未注册关联」的可检性检查并明确提示。

### D03 core-ui 来源 → **CI 从源码构建**
`packaging/core-ui-revision.txt` = `1840d689b51a32d392dbf06b8d594aef96696a80`；CI 用 `actions/checkout` 精确 ref 拉取，与 `zsui` 同级目录，调 `scripts/build-clang-cl.ps1 -Target core-ui` 产出 `core-ui.dll` 随包分发。
**路线图未含此项 → 本次对话新增，覆盖。**
**代价**：CI 需 VS 2022 工具链 + LLVM（clang-cl / llvm-rc）+ CMake + Ninja；clang-cl 路径需多候选探测；上游 SDK `rc.exe` 10.0.26100 在 ninja 子进程会挂起，必须用 llvm-rc 替换（上游脚本已内置）。core-ui 自身无构建工作流，这套步骤要我们自己维护。

### D04 文件列表 → **core-ui 全自绘**
**实测依据**：core-ui 全仓 `src/` 的 `SetParent` 仅出现在 `src/ui/image_view_gdi.cpp`；`include/ui_core.h:151` 明写 child window 语义不适用 → **不能嵌入 `IExplorerBrowser`**。
且 core-ui **无虚拟列表/表格控件**（控件全集无 `ListView`/`DataGrid`，`ScrollView` 非虚拟化）→ 虚拟化必须自己做。

**代价**：需自研虚拟列表（滚动/命中测试/键盘状态机/多选/框选/反向选择/就地重命名）。
**连带**：左侧原生 Shell 导航树同样不能嵌 → 需自绘或改用 core-ui 控件。
**落点选项**：(a) `<custom>` + Rust 侧虚拟化（与用户 `assets/main.uix` 的既有架构一致）；(b) 另起原生侧栏进程跑 `IExplorerBrowser`。→ 由架构师给出推荐与分阶段路线。

### D05 交付形态 → **唯一 `full` edition + `windows-x64`**（= 路线图 C3.6，一致）
- features = `network + desktop`（即 `default`）；不构建/不测试/不发布 `standard`/`network`/`desktop`/`themed`/`desktop-box`；**任何 32 位 x86 一律不做**。
- 产物名 `ZSDesk-<version>-full-x64`；逐文件 `bytes + SHA-256` manifest（沿用上游 `scripts/package.ps1`）。
- 强制校验 Visual C++ x64 Redistributable（`msvcp140`/`vcruntime140`/`vcruntime140_1`）。
- CI 的「Check all editions」「Test theme and desktop-only editions」两步删除，改为只对 full 跑 build/test/clippy。

---

## C1~C5 判定（以本次对话为准）

| 冲突项 | 判定 | 依据 |
|---|---|---|
| C1 .NET 10 | **不引入** | 对话原话「Rust 为主，必要时调用 C++ v14 运行库」+ 路线图建议一致 |
| C2 Core UI ABI 闸门 | **部分推进**：符号存在性已离线核实（2006 导出 / 567 纯 C API / 18 条 VERIFIED）；**参数语义仍需真机**，不得编造签名 | 无对话新决定，按路线图推进 |
| C3 功能保留范围 | **全量保留** | = D01，一致 |
| C4 ZSUI 过渡 | **过渡期保留双通道**（CI 继续 checkout `qiu7824/zsui`），替换完成后移除 | D02 + D04 的直接后果 |
| C5 fork 可见性 / CI | fork 为 **public**；**Actions 默认未启用，需在仓库页面手动开启** | 实测 `tikey1/zsdesk` = public |

---

## 技术后果汇总（D01–D05 组合后必须一并解决）

| # | 后果 | 归属 |
|---|---|---|
| T1 | 状态归属：zsdesk 是 Rust 侧 MVU，core-ui 是 `.uix` 内嵌 QuickJS 响应式 → **状态留 Rust**，`.uix` 只做视图 | 已由用户 `assets/main.uix` 架构注释定调，Spec 落地 |
| T2 | 内嵌预览窗格无处安放（不可嵌原生 HWND） | 架构师 D06 |
| T3 | 原生 Shell 导航树无替代 | 架构师 D07 |
| T4 | core-ui 567 个 `ui_*` 的 FFI 绑定、UTF-8/UTF-16 混用、两种内存所有权、UI 线程收敛、版本能力协商 | 架构师 D08 |
| T5 | `GuoheView.exe` 调用方式 | **已实测可行**（系统关联优先 + 显式 exe 兜底），见 D02 |
| T6 | Rust MSVC(`/MD`) × clang-cl core-ui(`/MD`) × 用户已有 v14 运行库 → 三方 CRT 一致 | 架构师 D09 |
| T7 | 本机无 Rust/C++ 工具链 → **一切编译验证只能靠 CI**；且 fork 的 Actions 默认未启用，需手动开启 | 运维 |

---

## 变更记录
- 2026-11-13 建台账；D00 录入对话原话前提。
- 2026-11-13 对话决策卡拍板 D01–D04（与用户 fork 路线图 C3/C3.6 一致；D02、D03 与路线图 P3 有出入，**以对话为准**）。
- 2026-11-13 用户明确「以本次对话中的选择为主」，写入决策优先级规则。
- 2026-11-13 落 D05（交付形态 = 路线图 C3.6），C1~C5 给出判定。
