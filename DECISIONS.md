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

**收窄（2026-11-13，架构师 D06 落地时确认）**：
D02 原文的「不做进程内解码」指的是 **GuoheView 的 `ghde.dll` 解码**（闭源、无公开 ABI）。
**本项目自研的 WIC 解码不在此列**（WIC 是 Windows 公开 API，零闭源依赖，
且上游 `window/image_decode.rs` 实测零 UI 依赖、可直接复用）→ 图片预览仍留在 core-ui 内做。
本条以 **D06** 为准。

**代价**：
1. 基于 `ghde.dll` 的解码能力（HEIC/AVIF/PSD/RAW/JXL）不集成进自家界面 → 走外部 exe。
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
| C5 fork 可见性 / CI | fork 为 **public**；**Actions 已启用并跑绿**（commit `4a48349`，`Windows build #1`，9m58s） | 实测 —— 原「需手动开启」的表述**作废** |

---

## D06–D11 架构师落定（2026-11-13，`spec/` 全套契约）

> 落定依据：`spec/ffi-core-ui.md`（ZC-FFI-001）、`spec/crate-layout.md`（ZC-LAYOUT-001）、
> `spec/migration-matrix.md`（ZC-MIG-001）、`spec/ci-contract.md`（ZC-CI-001）、
> `spec/milestones.md`（ZC-MILESTONE-001）。
> 事实依据：`research/facts.md`（10 组实测）、`docs/05-COREUI-EXPORTS.txt`（2006 导出 / 567 `ui_*`）、
> `docs/05-COREUI-ABI-NOTES.md`（18 VERIFIED / 12 UNVERIFIED）、`docs/11-COREUI-MARKUP-TAGS.md`（标签表 + token 冲突）。

### D06 内嵌预览 → **分三层**（图片留 core-ui 内 / Office·WPS 外部打开 / GuoheView 只做外部 exe）

- **做法**：
  1. **图片**：**留 core-ui 内** —— Rust 用 **WIC** 解码 → BGRA **预乘**像素 →
     `ui_image_set_pixels` → core-ui 的 ImageView 负责缩放/平移/适应窗口。
     复用上游 `window/image_decode.rs`（272 行，**实测纯 WIC、零 UI 依赖**），
     改造点：向 WIC 请求 `GUID_WICPixelFormat32bppPBGRA`（预乘）→ 零额外转换。
  2. **文本**：留 core-ui 内（Rust 读前 N KB → `.uix` 的 `TextArea`）。
  3. **Office / WPS / PDF**：**降级为「用系统程序打开」**。
  4. **GuoheView**：**只做外部 exe 入口**（D02）：优先系统文件关联 `ShellExecuteW`，
     兜底显式指定 `C://Program Files\GuoheView\GuoheView.exe`。
- **D02 的收窄（本条为对话内的最新口径，优先于 D02 的字面表述）**：
  D02 原文写「不链接 `ghde.dll`、不做进程内解码」——
  这里的「进程内解码」指 **GuoheView 的 `ghde.dll` 解码**（闭源、无公开 ABI）。
  **本项目自研的 WIC 解码不在此列**：WIC 是 Windows 公开 API，零闭源依赖。
  → 结论：**不做 ghde 解码；做 WIC 解码。**
- **符号可用性已核实（2026-11-13）**：团队质疑「`ui_image_view` / `ui_image_set_pixels`
  会不会只在 `ui_image_gdi_*` 一族里，CI 产物上就没有」。
  **核实结论：不是** —— `docs/05-COREUI-EXPORTS.txt` 行 259 / 252 / 261 均有；
  那 12 个「本机 DLL 独有」符号的真实名单是 `ui_flush_gpu`、`ui_image_evict_tile`、
  `ui_image_view_gdi`、`ui_image_gdi_{set_file,set_pixels,set_parent,set_zoom,get_zoom,width,height,fit,clear}`，
  **不含** `ui_image_view` / `ui_image_set_pixels`（标准 `ui_image_*` 族共 90 个导出），
  且二者属「头文件 ∩ DLL = 555」的交集 → **CI 产物必然导出**。**方案成立，无需降级。**
- **代码处置**：`image_decode.rs` **直接复用并接入首版预览路径**；
  `preview_process.rs` / `quicklook.rs` / `quick_preview.rs` / `hover_preview.rs` **改造**
  （宿主从原生窗口/子进程/自定义窗口消息换成 core-ui 的 ImageView）；
  `wps_preview.rs` + `wps_render_process.rs`（419 行）**删除**。
- **代价**：
  1. **不做**基于 `ghde.dll` 的任何解码 → HEIC/AVIF/PSD/RAW/JXL 只能走外部 exe；
  2. Office/WPS/PDF 只能外部打开；
  3. 进程内的 EXIF 面板、缩略图流式加载、多图胶片卷不做（GuoheView 的强项，不复刻）；
  4. **U07**（`ui_bitmap_create` 的像素格式 / stride / 生命周期）仍需真机核对 ——
     核对方法：造一张 4×4 半透明测试图灌进去，截图看边缘有没有发黑。
- **覆盖路线图 §4 P3**（原 P3 的「复用 `preview_process`/`quicklook`/`image_decode` 链路」
  并称之为「GuoheView **强关联**图片预览」→ 该「强关联」形态已被 D02 顶掉）。

### D07 原生 Shell 导航树 → **`<custom>` 自绘 + Rust 侧 `IShellItem` 枚举**

- **做法**：导航树用 `<custom id="nav_tree">`，Rust 侧枚举
  `FOLDERID_ComputerFolder / RecycleBinFolder / NetworkFolder / Desktop / Downloads` +
  快速访问 `shell:::{679F85CB-…}`，展开状态放 `.uix` 的视图局部态。
  渲染复用 `zsdesk-windows/src/shell/shell_item.rs`（385 行，**直接复用**）。
- **代价**：
  1. 丢原生树的「快速访问」自动排序与个性化细节；
  2. `namespace_tree.rs`（1,029 行）**重写** —— 其中快速访问 GUID 与已知文件夹枚举逻辑
     迁到 `crates/zsdesk-windows`，渲染部分重写到 `crates/zsdesk-ui/src/navtree/`；
  3. 自绘树的展开动画、密度感知（Win11 紧凑模式）要自己做。
- **兜底**：若 M1 排期不够 → 降级为「收藏 + 面包屑」（C 方案），M2 补。

### D08 core-ui FFI 绑定 → **手写 `extern "C"` + 运行时符号表**（不用 bindgen、不链接 `.lib`）

- **做法**：
  - 绑定清单以 **`docs/05-COREUI-EXPORTS.txt`（567 个 `ui_*`）为准**；
  - **必需符号**（约 150 个）缺失 → **加载即失败**，报出缺哪个符号；
  - **可选符号**（`ui_dialog`）缺失 → `Option` → 降级（用 `ui_msgbox` 兜底）；
  - **★ 2026-11-13 放宽**：那 12 个「**头文件漏声明、但源码有 `UI_API`**」的符号
    （`ui_flush_gpu`、`ui_image_evict_tile`、`ui_image_view_gdi`、`ui_image_gdi_*` 共 9 个）
    **可以正常绑定** —— 签名已从上游 `src/ui/ui_api.cpp` 逐个核对取得。
    （此前「只做存在性探测、绝不绑定」的判断**错在把头文件当完备契约**：
    PM 对 pinned revision `1840d689` 全 `src/` 扫 `UI_API` → **567** 个 `ui_*`，
    与本机 DLL 的 567 个**双向差集为空** → **CI 产物同样有**。）；
  - 编码**逐函数不同**（`const char*` 72 处 / `const wchar_t*` 77 处）→ 统一在绑定层
    转 UTF-8 `&str`，转换点唯一；
  - 内存所有权**三分类**：【借用】不得 free /【转移】必须用 core-ui 自己的 free 函数 /
    【拷贝】库已复制；
  - 调用**收敛 UI 线程**（`ui_window_invoke_sync`），工作线程只做 IO 与解码。
- **代价**：
  1. **没有编译期签名检查** → 用 CI 门禁 **S8**（逐符号比对导出表）补偿；
  1b. **签名有两个来源**（头文件 557 + 源码补 12）→ 两处都要对齐，靠 S8 兜；
  2. `UiWindowConfig` 结构体布局会漂移 → **首版完全不用它**（走 `.uix` 的 `<window>` 标签
     + `ui_page_open_window(page, NULL)`），要用之前必须核对 pinned revision 的头文件；
  3. 12 个 U 项语义未核实 → **不得凭导出名反推参数**；每接一组 API 先过对应的 U 项最小验证。
- **不选 bindgen 的理由**：bindgen 需要编译期存在 core-ui 头文件 → 会让
  `cargo test --workspace --release --locked` 依赖「先构建 core-ui」→ **破坏上游 CI 契约**。

### D09 core-ui 链接与 CRT → **运行时 `LoadLibrary` 动态加载；不链接 `core-ui.lib`；三方统一 `/MD`**

- **做法**：
  - Rust 侧用 `libloading` 在运行时加载 `core-ui.dll`，**显式限定搜索顺序**：
    exe 同目录 → `exe/core-ui/` → `exe/../` → `C:\Program Files\GuoheView\core-ui.dll`（**兜底**）；
  - 打包时 `core-ui.dll` **必须与 exe 同目录**（门禁 S12 强制），否则会命中 PATH 里的别的副本；
  - core-ui 用 `UI_CORE_MSVC_STATIC_CRT=OFF`（默认 `/MD`），与 Rust MSVC 默认、用户已装的
    v14 运行库三方一致；门禁 **S7** 用 `dumpbin /dependents` 机器判定（必须能看到
    `VCRUNTIME140.dll` + `MSVCP140.dll`）。
  - 内存所有权铁律：**谁分配谁释放** —— core-ui 的指针只用 core-ui 的 free 函数回收。
- **代价**：
  1. **失去编译期类型检查** → 用门禁 S8 补偿（同 D08）；
  2. 每次调用是间接调用 —— 可忽略（core-ui 内部本身就是间接调用）；
  3. 目标机必须装 VC++ x64 运行库（打包脚本强制校验，沿用上游机制）。
- **为什么不用链接期**：
  1. 链接期产生静态 import 表，若 exe 目录缺 DLL，Windows 会按 PATH 找到
     **用户自己装的** `C:\Program Files\GuoheView\core-ui.dll` —— 那份**不受我们控制**；
  2. 链接期把「API 面差 12/2 个符号」变成**加载即失败**；运行时可按符号在/不在优雅降级；
  3. 链接期需要 `core-ui.lib` 先存在 → `cargo test` 被 core-ui 的 C++ 构建绑架。

### D10 深浅色与 CSS token → **库接管；Rust 侧调 `ui_theme_set_mode`；无运行时 CSS 变量 API**

- **查证结论（两个）**：
  1. **需要** Rust 侧调 `ui_theme_set_mode(UiThemeMode)`（`UI_THEME_DARK=0` / `UI_THEME_LIGHT=1`）——
     它是 core-ui **唯一**的明暗切换入口，切模式会 `RebuildThemeVars` 重建所有 live page 的变量。
  2. **`include/ui_core.h` 里不存在运行时替换 CSS 变量的 C API** ——
     头文件全文 grep `css_var`/`var_set`/`set_var`/`stylesheet`/`token` **零命中**；
     567 个 `ui_*` 导出里只有 `ui_gh_img_view_{get,set}_shadow_style`（阴影样式，与 CSS 变量无关）。
     **前端 `docs/12-UIX-TAG-FIX.md` §4.2 独立复核，结论一致。**
- **token 终表（采纳前端 `docs/12-UIX-TAG-FIX.md` §4.3 的 17 条，架构侧只补运行时契约）**：
  - **改名避让 16 个** → 统一 `--app-*` 命名空间（`--app-bg` / `--app-content-bg` / `--app-sidebar-bg` /
    `--app-toolbar-bg` / `--app-text` / `--app-text-secondary` / `--app-divider` / `--app-hover` /
    `--app-pressed` / `--app-selection-bg` / `--app-info-bg` / `--app-warning-bg` / `--app-danger-bg` /
    `--app-info-fg` / `--app-warning-fg` / `--app-danger-fg`）。**一个都不保留原名。**
  - **改用库 token 1 个** = **accent**：CSS 侧读 `var(--accent)`、**不自定义**；
    由 Rust 侧 `ui_theme_set_accent_hex("#2563EB")` 写进库 palette 并同步到库 token
    （跨 `ui_theme_set_mode` 保留）。理由：若自定义 `--app-accent`，
    库自绘控件（primary 按钮 / 焦点环 / 滑块 / 进度条）仍是库默认蓝 `#3971EC`，
    与自绘区的 `#2563EB` **撞成两种蓝**。
  - **⚠️ 架构侧初判被前端纠正的一点**：初判建议「`--bg`/`--sidebar-bg` 改用库 token」，
    但前端源码级核对发现 **库的 `--bg` = 内容区底色（light `#FFFFFF`），
    而本项目的 `--bg` = 窗口底色（`#F3F3F3`）—— 不但同名，语义还相反**。
    → 采纳前端的改名避让方案。
- **架构侧补的三条运行时契约**：
  1. **Rust 必须在 `ui_page_load_string` 之前注入 `:root` 块**
     （读 `assets/themes.json` → 拼 CSS → 拼进 `.uix` 源码）。
     → 页面用 **`ui_page_load_string`**（`const char*` UTF-8）加载，**不是** `ui_page_load_file`；
     → **`.uix` 与 `themes.json` 必须以文件形式进包**（CI 已补，见 `spec/ci-contract.md` §3.4 第 22 条）。
     → ★ **`:root` 注入的四条硬规则**（QA B1 复核 + 架构侧核对 `sfc_parser.cpp` 确认）：
       **(a) 必须包在 `<style>` 块内**，**禁止**裸放在源码顶层
       （顶层只允许 `<window>` `<template>` `<script>` `<style>` `<import>` `<link>`；
       裸放顶层 → `unexpected content at top level` → `doc.ok=false` → **整页 load 失败**，
       报错 "SFC parse failed" 极易误判成标签问题）。
       **推荐形态**：在源码**末尾追加一个完整的 `<style>` 块**（多个 `<style>` 块允许，
       `sfc_parser.cpp:321-330` 无 duplicate 检查），天然保证注入值在 source order 最后。
       **(b) `:root` 内只允许 CSS 变量声明**（`cascade.cpp:124-126` 明确
       `Skip :root — those only define CSS vars` → `background`/`color`/`font-family` 会被**静默丢弃**）。
       **(c) 加载前必须自检**：解析注入后的源码、断言 `:root` 落在某个 `<style>` 块内，
       不满足即 **fail-fast**（Rust 侧直接失败并说清原因）——**不许**把错误留给 core-ui 报 "SFC parse failed"。
       **(d) 注入必须早于 load**，且注入值要在 source order 上**最后**（自定义属性同特异性后者覆盖前者）。
  2. **Rust 初始化调用顺序**：
     `ui_theme_set_default_font` → `ui_theme_set_cjk_font` → `ui_theme_set_mode` →
     `ui_theme_set_accent_hex` → 拼 `:root` → `ui_page_load_string`。
     （第 2 步须在第 3 步之前或紧邻：切 mode 会用当前 accent override 重新派生。）
  3. **深浅色切换必须重建页面**：`ui_page_reload` 会**重放旧源码**（改不了 palette）
     → 必须重新 `ui_page_load_string` 一个新 page，并重走自绘控件装配流程。
- **代价**：
  1. 明暗切换时，**库自绘控件跟随库 palette 变，本项目自绘区域（`--app-*`）不会自动跟随** ——
     必须重建页面才变（这是改名避让路线的固有代价，已接受）；
  2. **重建页面 = 重建控件树** → 自绘控件要重新取句柄并重挂回调
     （`ui_page_on_widget_mount` 按 id 持久化 → **自动回填**，见 D12）；
     重建期间有一次闪烁（M1/M2 要量一下）；
  3. 门禁 **S13**（最终表述见下）。
- **三个补充问答（2026-11-13 team-lead 提出，架构师定稿）**：
  1. **「深浅色切换时 `--app-*` 会不会变？」→ 不会。**
     `--app-*` 不在库的 `kThemeKeys[]`（34 个）里，`RebuildThemeVars`（`compiler.cpp:1672-1700`）
     只 erase + 重写那 34 个键 → 自定义值**原样保留**。
     **但缺口是真的**：`:root` 只有一套 → light/dark 两套值无法自动切换 →
     只切 `set_mode` 会出现「库控件变深、自绘区还是浅色」的半深半浅。
     **补法（推荐 A）**：Rust 侧**重建页面** —— 用另一套 `--app-*` 重新拼 `:root` →
     重新 `ui_page_load_string` 一个新 page。
     候选 B「加载两份 `.uix`」等价于 A 且要同步维护两份 → 否决；
     候选 C「只跟随系统 + 单套值」= 放弃深色设计 → 否决。
     **分两期**：M1 只做「**启动时按系统主题选一套**」（零闪烁、零幂等压力），
     运行时手动切换放 M2+（多标签时只重建当前可见页，其余懒重建）。
     > 重建页面在咱们这儿不贵 —— **状态在 Rust 的 EntryStore 里**，`.uix` 是无状态视图 →
     > 重建后只需重新投影一次状态。这正是 ADR-0001「状态留 Rust」的回报。
  2. **调用顺序与影响范围**：
     `set_default_font` → `set_cjk_font` → **`set_mode`** → **`set_accent_hex`** → 拼 `:root` → `load_string`。
     `set_mode` 会用**当前**的 accent override 重新派生 → **`set_accent_hex` 必须在其后或紧邻**；
     `set_accent_hex` 跨 `set_mode` 保留（`theme.h:435-439`）→ 第 3 步不能省
     （省了 `var(--accent)` 落到库默认 `#3971EC`，与设计稿 `#2563EB` 撞成两种蓝）。
  3. **门禁 S13 最终表述**：从「禁止这 3 个撞名 token」**扩展为「禁止自定义库 token 的
     34 个全集」** + 禁止自定义 `--app-accent`；**只拦「定义」（`--name:`）不拦「引用」（`var(--name)`）**。
     理由：34 个里任何一个撞名都会在 `set_mode` 时被 erase → 表现为「切主题后某处颜色突变」
     且**静默**；且统一到 `--app-*` 命名空间后，自定义属性与库 token **名字空间完全不相交** →
     门禁可以用「不相交」一句话概括，不依赖 34 个名字清单是否最新。

### D11 目录结构、迁移矩阵与里程碑 → **保留上游 5 crate 名 + 新增 `zsdesk-coreui` + 138 文件四档 + M0~M3**

- **目录结构**：`zsdesk-core`(L0) / `zsdesk-platform`(L1) / `zsdesk-windows`(L2) /
  **`zsdesk-coreui`(L2，新增)** / `zsdesk-ui`(L3) / `zsdesk-app`(L4)。**单向依赖、无环**；
  `zsdesk-coreui` **零内部依赖** → 结构上保证 FFI 层不反向依赖 UI 业务层。
  其余 crate 一律 `#![forbid(unsafe_code)]`。
  **★ 不重命名上游 crate**（初版曾提议统一改 `zc-*` 前缀，推翻：138 个文件的
  `use` 路径 / `Cargo.toml` / CI 命令 / 文档引用全要改，收益只是一致性）。
  CI 工作区 = **`app/` + `zsui/` + `core-ui/` 三级同级**（对应 `../../../xxx` path 依赖）；
  `.uix` 资源在仓库既有的 **`assets/`**。
- **迁移矩阵**（138 文件 / 42,150 行，实测）：
  **直接复用 68 文件 / 15,671 行（37.2%）**、**改造 56 / 20,193（47.9%）**、
  **重写 10 / 5,600（13.3%）**、**删除 4 / 686（1.6%）**。
  本仓新增 **5,900–10,600 行** [估算]。
- **里程碑**：M0↔P1（骨架+CI 绿+门禁）、M1↔P2（窗口与文件浏览）、
  M2↔P3（**GuoheView 外部调用 + 降级通道 + 错误提示**，**覆盖路线图 §4 P3**）、
  M3↔P4（其余功能迁移 + 完整回归 + 便携包 + Release）。
- **代价**：
  1. **13.3% 的代码要从零重写**（5,600 行）+ **5,900–10,600 行新增** → 总量约 1.2–1.6 万行 [估算]；
  2. **85% 的代码要「改造」**（不是直接复用）—— 「改造」的成本常被低估：
     一处 `use` 路径变更 + 一处 API 语义变更，工作量可能是原文件的 30%–60%；
  3. M0~M3 **不允许并行**（本机无工具链，只能靠 CI 验证，一次合并多个阶段的失败无法二分定位）。

### D12 `.uix` 资产契约与自绘挂载选型（2026-11-13 源码级核对后定稿）

> 触发：前端在改 `.uix` 资产时发现三处「资产写法与 core-ui 实际语义不符」，
> 架构师**独立核对上游源码**后定稿。详细依据见 `spec/ffi-core-ui.md` §4.3.1 / §4.3.3。

- **D12-1 自绘控件挂载 → 选 (b) `.uix` 里声明 `<custom id="file_list" class="list-host"/>`**
  （**不用** (a) 运行时 `ui_custom()` + `ui_widget_add_child()`）。
  **决定性理由是布局，不是 API 好不好用**：运行时造的控件**没有 CSS 级联结果**
  → 无 `flex-grow`/`width`/`height` → 在 flex 容器里**尺寸为 0** → **列表区空白**；
  而 `<custom>` 标签在工厂里明确「CSS width/height 走 ApplyCommonStyle 自然生效」
  （`widget_factory.cpp:1114-1118`）。
  **与 D-15 相容**：`<custom>` 是一个**空的自绘画布**，**行**仍全部由 Rust 画 ——
  用户 `assets/main.uix` 注释约束的是「**List rows** are NEVER created here」。
  **代价**：多了一个非布局标签；`.uix` 里必须给它正确的 flex 尺寸。
- **D12-2 `<window>` 必须自闭合** —— `sfc_parser.cpp:285-297` 非自闭合 → `pushErr` →
  `sfc_parser.cpp:373` `if (!doc.errors.empty()) doc.ok = false;` → **整页 load 失败**。
  **Rust 侧必须能区分两类失败**：**整页解析失败**（`ui_page_load_string` 返回 NULL，
  取 `ui_page_last_error` 的行/列）与**局部控件缺失**（加载成功但 `find_by_id` 返回 0）。
  **禁止**只判 `find_by_id`（会把整页失败误报成控件缺失）。
  **修正**：前端引用的路径 `src/ui/page/sfc_parser.cpp` **不对**，实际在 **`src/ui/uix/`**（行号 294 正确）。
- **D12-3 `text="…"` 不是文本属性** —— 只有 `<menu>` 读它作 title 别名（`compiler.cpp:641/644`）。
  真实机制是 `FirstPlainText`（`widget_factory.cpp:54-61`）取**第一个直接子文本节点**。
  → `.uix` 里写 `<label>文字</label>`；**Rust → `.uix` 推状态走 `ui_page_set_*` + 模板里的 `{{ }}` 插值**。
- **D12-4 根窗口样式用 `.root` class** —— `window { … }` 是标签选择器（`selector.cpp:59`
  `node.tag == "window"`），而 `<window>` **不产生 widget**（`sfc_parser.cpp:285-296`）
  → **匹配 0 个节点**；`:root { … }` 在普通样式计算里被**显式跳过**（`cascade.cpp:125`），
  **只定义 CSS 变量**。→ 根 `<div class="root">` + `.root { … }`。
  **修正**：前端「`window {}` 落到根节点 `.root`」的**结论对但机制要改口**
  —— 不是「window 选择器自动落到 root」，而是「window 选择器匹配不到东西，必须显式加 class」。
- **D12-5 `:root` 注入的落点** —— `:root` 块**必须包在 `<style>` 块内**，禁止裸放源码顶层
  （`sfc_parser.cpp:260-267` → `unexpected content at top level` → 整页失败）。
  推荐在源码末尾**追加一个完整的 `<style>` 块**（多个 `<style>` 块允许）；
  `:root` 内**只放 CSS 变量**（`cascade.cpp:124-126` 会跳过普通声明）；
  Rust 侧**加载前自检**，不满足即 fail-fast。完整措辞见 **D10 契约 1** 与
  `spec/ffi-core-ui.md` §4.3.2.1。
- **代价**：
  1. `.uix` 资产必须多一次机械评审（标签自闭合 / 文字位置 / 根 class / token 命名空间 / `:root` 落点）；
  2. 根 `<div>` 必须显式加 `class="root"`（原来靠 `window {}` 的写法要改）；
  3. 每条 `.uix` 硬约束都应有一条**可执行**的检查（否则靠人记）→ 并入门禁 S13 与前端自检脚本；
  4. Rust 侧要实现一个「`:root` 是否落在 `<style>` 内」的自检（几十行）。

---

## D13 无障碍与窗口材质 → **接受降级并记录**（用户 2026-11-13 拍板）

**实测依据**（PM 独立核实）：core-ui **没有** Mica / Acrylic / 高对比度 / UIA 入口 ——
`src/ui/ui_window.cpp` 的 `DwmSetWindowAttribute` 只用于 `TRANSITIONS_FORCEDISABLED` 与圆角偏好；`mica` / `acrylic` / `backdrop` 零命中；`src/ui/theme.h` 的 `HighContrast` / `AppsUseLightTheme` 零命中；`src/ui/widget.cpp` / `ui_api.cpp` 的 `uia` / `accessibility` / `narrator` 零命中。

**因此必然丢失的能力**
| 能力 | 状态 | 说明 |
|---|---|---|
| **高对比度模式** | ❌ 不可实现 | core-ui 自绘控件不向 UIA 暴露接口，系统高对比度不会自动接管 |
| **读屏 / Narrator** | ❌ 不可实现 | 同上 |
| **Mica / Acrylic 材质** | ❌ 丢失 | zsdesk 上游自己用 DWM 实现（`crates/zsdesk-app/src/theme.rs:131-194`），换 core-ui 后不复存在 |
| 深色标题栏与窗口圆角 | ⚠️ 降级 | core-ui 只暴露圆角偏好，不暴露系统材质 |

**连带动作**
1. `docs/07-UI-DESIGN-WINUI3.md` 的视觉验收项中，Mica / Acrylic / 高对比度相关条目**按「不可实现」处理**，不再作为验收门（该文档是用户的原稿，标注写入 `docs/13` 首查清单与交付说明，不改用户原稿）。
2. 已知限制必须写进 `DELIVERY.md` 与 `README-FORK.md`，不得只留在本台账里。
3. `spec/milestones.md` 的 M1/M3 验收项中，不得出现「保留 Mica」这类不可实现的承诺。

**复评条件**：若 core-ui 上游后续补上 UIA 桥接或系统材质入口，重新评估本决策。

---

## D14 品牌色 → **Rust 侧读系统 AccentColor**（用户设计文档的落地路径）

**用户原稿规定**：`docs/07-UI-DESIGN-WINUI3.md` §4 的颜色 token 表里，`--accent` 明确写「**跟随系统 `AccentColor`**」。

**实测能力边界**：core-ui 全仓 `GetSysColor` / `UISettings` / `AccentColor` **零命中** → core-ui 的品牌色是**内置调色板**，**不跟随系统**。`src/ui/theme.h:446-455` 的 `SetAccent()` 只接受外部传入的值（传零值即恢复库默认），库里没有任何读系统色的路径。

→ 所以「跟随系统强调色」**只能由 Rust 侧实现**，否则用户原稿的这条要求落不了地。

**实现路径**
1. Rust 启动时读注册表 `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\AccentColor`（DWORD，**BGR** 序）→ 转成 `#RRGGBB`；
2. 调 `ui_theme_set_accent_hex(hex)` 写进**库 token** `--accent` → 库自绘控件（按钮 / 焦点环 / 滑块 / 选中态）与 `.uix` 自绘区域用**同一个**强调色，不会出现「两种蓝」；
3. **回退**：读不到（Win10 无该键 / 组策略禁用 / 值非法）→ 用 `assets/themes.json` 的 `--app-accent` 作回退源值；
4. 切换深浅色时**重读并重调**（Win11 的深浅可能使用不同的 AccentColor），并在重建页面前完成。

**与前端方案的关系**：前端已把 `.uix` 里的 accent 统一读**库 token** `var(--accent)`、`--app-accent` 只作唯一源值 —— **与本决策完全兼容**，无需改动。`--app-accent` 的定位从「直接生效的值」变为「读不到系统色时的回退值」。

**代价**：Rust 侧多一处注册表读取（`windows` crate 的 `RegGetValueW`）；BGR→RGB 的通道顺序必须写对（写反了会变成橙色系）。

---

## 技术后果汇总（D01–D05 组合后必须一并解决）

| # | 后果 | 归属 | 状态 |
|---|---|---|---|
| T1 | 状态归属：zsdesk 是 Rust 侧 MVU，core-ui 是 `.uix` 内嵌 QuickJS 响应式 → **状态留 Rust**，`.uix` 只做视图 | 已由用户 `assets/main.uix` 架构注释定调 | ✅ Spec 落地（`spec/ffi-core-ui.md` §4） |
| T2 | 内嵌预览窗格无处安放（不可嵌原生 HWND） | 架构师 | ✅ **D06** |
| T3 | 原生 Shell 导航树无替代 | 架构师 | ✅ **D07** |
| T4 | core-ui 567 个 `ui_*` 的 FFI 绑定、UTF-8/UTF-16 混用、两种内存所有权、UI 线程收敛、版本能力协商 | 架构师 | ✅ **D08** |
| T5 | `GuoheView.exe` 调用方式 | **已实测可行**（系统关联优先 + 显式 exe 兜底） | ✅ D02 |
| T6 | Rust MSVC(`/MD`) × clang-cl core-ui(`/MD`) × 用户已有 v14 运行库 → 三方 CRT 一致 | 架构师 | ✅ **D09** |
| T7 | 本机无 Rust/C++ 工具链 → **一切编译验证只能靠 CI** | 运维 | ✅ fork 的 Actions **已启用并跑绿**（commit `4a48349`，`Windows build #1`，9m58s）→ 契约见 `spec/ci-contract.md` |
| T8 | 主题 token 撞名 → `ui_theme_set_mode` 会擦除用户同名值 | 架构师 | ✅ **D10** |

---

## 变更记录
- 2026-11-13 建台账；D00 录入对话原话前提。
- 2026-11-13 对话决策卡拍板 D01–D04（与用户 fork 路线图 C3/C3.6 一致；D02、D03 与路线图 P3 有出入，**以对话为准**）。
- 2026-11-13 用户明确「以本次对话中的选择为主」，写入决策优先级规则。
- 2026-11-13 落 D05（交付形态 = 路线图 C3.6），C1~C5 给出判定。
- 2026-11-13 架构师落 **D06–D11**，并产出 `SPEC.md` + `spec/` 五份契约 + FFI 绑定骨架。
  修正两处既有表述：① `ui_core_embed_binary` **不是 C API**（是 CMake 辅助函数），
  不在绑定清单里；② `C5` 的「Actions 需手动开启」**作废**（已实测启用且跑绿）。
- 2026-11-13 **第二轮（CI 实现评审后）**：
  - **D06 改写**为「预览分三层」（图片留 core-ui 内 / WIC 解码），
    并核实 `ui_image_view` / `ui_image_set_pixels` **确实在导出表内**（行 259 / 252）→
    **方案在 CI 产物上成立，无需降级**；
  - **D10 采纳前端 `docs/12-UIX-TAG-FIX.md` §4.3 的 17 token 终表**（16 改名 `--app-*` + accent 走库 token），
    架构侧补三条运行时契约（注入 `:root` / 初始化调用顺序 / 切明暗重建页面）；
  - **D11 改为保留上游 crate 名**，新增 `zsdesk-coreui`；
  - **D02 收窄**：其「不做进程内解码」限定为「不做 `ghde.dll` 解码」，WIC 解码仍做；
  - **发现 CI 致命缺口**：打包未含 `assets/`（`.uix` + `themes.json`）→
    交付的 exe 启动即失败 → `spec/ci-contract.md` §2.4；
  - `spec/ci-contract.md` 改为 **评审 + 缺口**（v2），列出 10 条缺口 G1~G10。
- 2026-11-13 **第三轮（CI 实现评审 + 前端三个 P0）**：
  - **D12 新增**：`.uix` 资产契约 + 自绘挂载选型（选 (b) `<custom>` 静态声明）；
    三条 P0（`<window>` 自闭合 / `text=` 不是文本属性 / `window{}` 选择器无效）
    **已独立核对上游源码**，结论与前端一致，但**修正了两处路径/机制表述**；
  - **D10 定稿**：补三问 —— ① `--app-*` 切主题**不会变**，缺口用「Rust 侧重建页面」补
    （M1 只做启动时选一套，手动切换 M2+）；② `set_mode` → `set_accent_hex` 的顺序与影响范围；
    ③ **S13 扩展为「禁止自定义库 token 的 34 个全集」**（只拦定义不拦引用）；
  - **D02/D06 同步**：`ui_image_view` / `ui_image_set_pixels` **已核实存在于导出表**
    → 方案在 CI 产物上成立，无需降级；
  - `spec/ci-contract.md` 改为 **v3「目标态 vs 当前态」两栏** —— 修正 v2 把目标态
    写成现状的问题（4 个门禁脚本**在仓库里并不存在**），并按 CI 实现的新状态重判 33 条。
- 2026-11-13 **第四轮（QA B1 合并 + D08 放宽 + 戳记归属）**：
  - **合并 QA 的 B1**：`:root` 注入的落点定为「**必须包在 `<style>` 块内**」，
    裸放源码顶层 → `unexpected content at top level` → **整页 load 失败**；
    `:root` 内只放 CSS 变量；Rust 侧**加载前自检**、fail-fast（见 D10 契约 1 / D12-5）。
  - **D08 放宽**：那 12 个「头文件漏声明、但源码有 `UI_API`」的符号**可正常绑定**
    —— 签名已从上游 `src/ui/ui_api.cpp` 逐个核对取得；
    PM 扫 pinned revision 全 `src/` → 567 个 `ui_*`，与本机 DLL 双向差集为空 → **CI 产物同样有**。
    ★ 此前「只探测不绑定」的判断**作废**，错因是**把头文件当完备契约**。
  - **戳记归属定稿（口径「甲：Rust 产出」）**：由 `crates/zsdesk-coreui/build.rs` 在编译期
    写到 `target/<profile>/core-ui.dll.revision`（与 exe 同目录）；**CI 只校验内容**，
    不自己写戳记（避免两个来源）。打包步骤不用改（它已经在拷那个路径），只需加内容断言。
  - **S7' 拆两段**：第一段（CI 产物 vs 导出表，**本版就加**）+ 第二段（绑定层必需符号，
    **M1 加**）。
  - **M3 验收项**新增 V3-10/V3-11/V3-12：搜索后端不可用时**必须自动回退**且**UI 显示当前后端**。
