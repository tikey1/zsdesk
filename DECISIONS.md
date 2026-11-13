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
- **★ D06-1 承载方式细化（2026-11-13，第七轮）**：D06 的**实质不变**
  （图片**留在程序内**、WIC 解码、不依赖 `ghde.dll`）；
  变的只是**「谁来承载这张图」**：
  - 原写法「`ui_image_set_pixels` → core-ui 的 ImageView 负责缩放/平移/适应」——
    **实现风险已查清并改道**：`ui_image_view()` 是**运行时创建的控件** →
    **拿不到 CSS**（`ApplyCommonStyle` 只有 `compiler.cpp:920` 与 `widget_factory.cpp:1363` 两个调用点）
    → 尺寸只能 `ui_widget_set_size` **手算 px**；且它作为**运行时插入的子控件**如何参与父容器布局，
    core-ui **没有公开承诺** → **风险不可控**。
    声明式 `<img>` 也不行：它的 `src` 是**资产 key**（`ui_asset_register_*` 解析），
    **没有运行时改 `src` 的 API**。
  - **新做法**：Rust 在 **`<custom id="preview_image">`** 里 blit ——
    `ui_bitmap_create(pixels, w, h, stride)` → `ui_draw_bitmap_handle(ctx, bitmap, x, y, w, h)`
    （回调签名 `UiCustomDrawCallback(UiWidget, UiDrawCtx, UiRect, void*)` **带 draw ctx**，
    头文件该段注释即写着「Drawing API (use inside UiCustomDrawCallback)」）→ 缩放由 Rust 算 fit。
  - **像素管线不变**：仍是 **WIC → BGRA 预乘 → 位图**（`ui_draw_bitmap` 的像素格式要求与
    `ui_image_set_pixels` 相同：**BGRA 预乘**）→ `spec/ffi-core-ui.md` §7.1 的像素格式章节**继续有效**；
    上游 `image_decode.rs` **仍然直接复用**（改造点不变）。
  - **代价**：缩放 / 平移 / 滚动**要自己实现**（首版只要 fit）；
    M3+ 若要交互式缩放 → 「`<custom>` 内 `ui_widget_add_child` + `ui_widget_set_size` **显式定位**」
    或 `ui_image_gdi_*` 族（这 9 个符号**可绑定**）。

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
     → ★ **`:root` 注入的六条硬规则**（QA B1 复核 + 架构侧核对 `sfc_parser.cpp` 确认；
     **(d) 在 2026-11-13 被 QA 的 Δ-A 纠正，架构侧已独立复核并改正**——此前写的是「最后」，**方向反了**）：
       **(a) 必须包在 `<style>` 块内**，**禁止**裸放在源码顶层
       （顶层只允许 `<window>` `<template>` `<script>` `<style>` `<import>` `<link>`；
       裸放顶层 → `unexpected content at top level` → `doc.ok=false` → **整页 load 失败**，
       报错 "SFC parse failed" 极易误判成标签问题）。
       **推荐形态**：在源码**最前面插入一个完整的 `<style>` 块**（多个 `<style>` 块允许，
       `sfc_parser.cpp:321-330` 无 duplicate 检查；顶层块顺序无关 → 插最前天然保证注入是**第一个** `:root`）。
       **(b) `:root` 内只允许 CSS 变量声明**（`cascade.cpp:125` 明确
       `Skip :root — those only define CSS vars` → `background`/`color`/`font-family` 会被**静默丢弃**）。
       **(c) 加载前必须自检（四条断言）**：位置（`:root` 落在某个 `<style>` 块内）/
       **顺序**（注入的是**整张样式表**里第一个 `:root`；且源码里**没有 `<link rel="stylesheet">`**）/
       内容（`:root` 内每条声明都匹配 `--app-[a-z0-9-]+\s*:`）/
       键集（不含 `--app-accent`、不撞库 token 34 个）→ 任一不满足即 **fail-fast**
       （Rust 侧直接失败并说清**是哪一条**）——**不许**把错误留给 core-ui 报 "SFC parse failed"。
       **(d) 注入必须早于 load**，且**注入的 `:root` 必须是「整张样式表」里的第一个 `:root`**
       （整张样式表 = **`<link>` 引的 CSS（在前，`page_api.cpp:271-277`）+ inline `<style>`（在后）**；
       本项目**零 `<link>`** → 等价于「源码里第一个 `:root`」）
       ——
       ★ core-ui 的自定义属性是**先定义者赢**，不是「后者覆盖前者」：
       `CollectRootVars`（`cascade.cpp:20-33`）按文档顺序推进有序向量，
       `LookupVar`（`value.cpp:572-579`）**返回第一个命中**，
       `cascade.h:41` 注释逐字「**Returns first set encountered**」，
       且 `:root` 规则在普通级联里被**整条跳过**（`cascade.cpp:125`）→ **根本不参与 CSS 级联**。
       ⇒ 注入若排在 `.uix` 自己的 `:root` 之后 → **`.uix` 的值赢** → Rust 的注入被**静默忽略** →
       症状是「切主题颜色不变」，**没有任何报错**。
       **★ 注入位置的正式裁定（QA 第五轮提请裁，架构侧 2026-11-13 裁）：选「甲A —— 在源码最前插入」**
       （契约上一轮已改为甲A，本轮**补全前提**并正式定案）：
       ① **无条件成立**，不依赖「`.uix` 里没有 `:root`」这个巧合 —— 将来有人往 `.uix` 写 `:root` 也不会静默压掉注入值；
       ② **Rust 侧代码量相同**（prepend 与 append 都是一次字符串 concat）；
       ③ 证据 C（SFC 顶层标签**不限顺序**，`sfc_parser.cpp:252-297`）保证语法合法；
       ④ 唯一残留前提 = 证据 B（`<link>` CSS 里不能有 `:root`）→ 三个资产零 `<link>`，Rust 自检顺手断言即可。
       （被否的是「甲B —— 保留末尾追加 + 靠门禁禁止 `.uix` 出现 `:root`」：
       把正确性押在「资产里永远不会出现 `:root`」这种约定上，而违约的代价是**静默失效**，不值得。）
       （QA 实测：三个 `.uix` 当前**生效**的 `:root` 规则数 = **0** → 今天不暴露，但这是埋着的雷。）
       **(e) 键集排除 `--app-accent`**（D14：它是回退源值，不是 CSS 变量）。
       **(f) `:root` 内只允许 CSS 变量**（= (b) 的**断言版**：Rust 自检必须拦，不只是建议）。
  2. **Rust 初始化调用顺序**：
     `ui_theme_set_default_font` → `ui_theme_set_cjk_font` → `ui_theme_set_mode` →
     `ui_theme_set_accent_hex` → 拼 `:root` → `ui_page_load_string`。
     （第 2 步须在第 3 步之前或紧邻：切 mode 会用当前 accent override 重新派生。）
  3. **深浅色切换必须重建页面**：`ui_page_reload` 会**重放旧源码**（改不了 palette）
     → 必须重新 `ui_page_load_string` 一个新 page，并重走自绘控件装配流程。
- **代价**：
  1. 明暗切换时，**库自绘控件跟随库 palette 变，本项目自绘区域（`--app-*`）不会自动跟随** ——
     必须重建页面才变（这是改名避让路线的固有代价，已接受）；
  2. **重建页面 = 重建控件树** → **32 个句柄全部失效**（`main.uix` 26 + `preview.uix` 6）→
     必须**重取全部句柄** + **在新 page 上重新注册 `ui_page_on_widget_mount`**
     （钩子挂在**具体 page** 的 `PageState` 上 → **不自动转移**；catch-up 保证「注册即触发」）；
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

> 触发：前端在改 `.uix` 资产时发现「资产写法与 core-ui 实际语义不符」之处（第一轮 3 处，
> 第二轮又补 2 条 P0 + 1 条分隔线问题），架构师**逐条独立核对上游源码**后定稿。
> 详细依据见 `spec/ffi-core-ui.md` §4.3.1 / §4.3.3 / §4.6。

- **D12-1 自绘控件挂载 → 选 (b) `.uix` 里声明 `<custom id="list_host" class="list-host"/>`**
  （★ **id 以资产为准**：`list_host`，不是 `file_list`；命名规则 `<区域>_host`）
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
- **D12-6 ★ `<separator>` 不画 CSS `background`**（前端第二轮 P0，架构侧已核对
  `src/ui/controls.cpp:635-643`：`SeparatorWidget::OnDraw` **不调用** `Widget::OnDraw`，
  颜色写死 `theme::kDivider()` = **库 token**）→ 要跟项目 token `--app-divider` 同色，
  **必须用 1px `<div class="sep"/>` + `background`**。
  **修正前端的一处事实**：上游 demo 并非「`<separator>` 一次没用」——
  `demo/ui_demo.uix` 里 `class="sep"` **19 处**、`<separator/>` **3 处**（那 3 处**全在 `<menu>` 内部**，
  菜单分隔线用库色正好）。**结论不变**：布局分隔线跟项目 token 同色 → 用 `<div class="sep"/>`。
  **附带提醒**：demo 的 `.sep` 用的是**库 token** `var(--border-subtle)`；
  本项目要用 `var(--app-divider)`，否则分隔线**不随主题切换**（`--app-*` 不会自动变，见 D10 契约 3）。
- **D12-7 ★ 页面重建后句柄全部失效**（前端第二轮 P0，架构侧已核对
  `page_api.cpp`：`ui_page_load_string` → `Registry().Insert(std::make_unique<PageEntry>())`，
  旧控件被销毁；`ui_page_reload`（`:565-577`）**重放旧源码** → 改不了 palette，**且同样重新编译** →
  同样失效）。
  → **三步协议**：`ui_page_destroy(旧 page)` → `ui_page_load_string(新源码)` → **重取全部 32 个句柄**
  （`main.uix` 21 + `preview.uix` 6，架构侧已独立复核 distinct 数）→ 重挂回调 / 重投影状态。
  **旧句柄继续用 = 写野内存 → 静默崩溃 / 花屏，不会报「句柄无效」。**
  **「自动回填」只覆盖一半**：`ui_page_on_widget_mount` 会自动回填**自绘控件的绘制回调** ✅，
  但**不会**更新 Rust 侧持有的 `ui_widget_find_by_id` 句柄 ❌。
  完整协议与故障判据见 `spec/ffi-core-ui.md` **§4.6**。
- **D12-8 「加载后断言颜色」在 API 层面做不了** —— core-ui **没有**读回 CSS 变量的 C API
  （导出面只有 `ui_widget_set_bg_color` / `ui_widget_set_text_color` 这类**写**接口；
  `ui_theme_bg()` 等只读**库 token**，读不到 `--app-*`；架构侧已扫全导出表确认）。
  → 「压根没注入」这个**静默**失败项，只能用**离屏渲染取像素**验（一次性冒烟，不进启动路径）；
  启动路径只保留**加载前自检**。见 `spec/ffi-core-ui.md` §4.3.2.1 末尾。
- **D12-9 ★ 自绘宿主的判定规则与三个宿主的裁定**（第七轮，前端第三轮提问）
  **规则（一条）**：**这块矩形的内容是不是 Rust 用绘制指令画出来的？**
  是 → `<custom>`；否（core-ui 控件 / 文本投影）→ `<div>` / `<label>`；
  混合 → **`<div>` 容器 + 里面的 `<custom>` 子宿主**。
  | id | 裁定 | 理由 |
  |---|---|---|
  | `nav_host` | ✅ **改 `<custom>`** | 不是「为了统一」：`<div>` 上 `ui_custom_on_draw` 是**静默 no-op** → **树根本画不出来**（D07 已定导航树是 Rust 自绘） |
  | `breadcrumb` | ⛔ **不改 `<custom>`**，但**要改形态**：`<label id="breadcrumb">` + Rust `ui_label_set_text` | 面包屑是**文本投影**不是自绘画布；且现在是**空 `<div>` → 显示不出任何东西**（真缺口） |
  | `preview_host` | ✅ **保持 `<div>` 作容器**，里面加 `<custom id="preview_image">` + 预先声明的 `<label>` 元信息行 | 图片与元信息是「混合」；见 **D06-1**（不用 `ui_image_view()` 当子控件的理由） |
  **规则里没有「API 好不好用」**：运行时 `ui_custom()` 也能挂回调，但它**不经过 `ApplyCommonStyle`**
  （该函数全仓只有 `compiler.cpp:920` 与 `widget_factory.cpp:1363` 两个调用点）→ **没有 CSS** →
  flex 容器里**尺寸为 0** → 这是**布局问题**。
  **id 以资产为准**：列表宿主是 **`list_host`**（不是 `file_list`；本台账与各 Spec 里的旧举例已全部改正），
  命名规则 **`<区域>_host`**。
- **D12-10 ★ 修正架构侧自己上一版的措辞：「按 id 持久化 → 重建后自动回填」不成立**
  `ui_page_on_widget_mount` 的钩子挂在**具体 page** 上（`page_api.cpp:850-854` → `e->state->OnWidgetMount`
  → 存进该 `PageState` 的 `lifecycleHooks_`）：
  - **同一个 page 内**晚出现的控件（v-if / v-for 重建）→ 钩子**自动补发** ✅
    （且注册时若树上已有同 id 控件会**立刻调一次** = catch-up，`page_state.cpp:635-652`）；
  - **切主题重建出新 page** → 新 `PageState` 的钩子表**是空的** → **不会自动转移** ❌
    → Rust **必须在新 page 上重新注册**（`ui_page_on_widget_mount(new_page, id, cb, ud)`，
    catch-up 保证**注册即触发**，不需要额外的同步等待）。
  → **重建三步**（`spec/ffi-core-ui.md` §4.6）：
  `ui_page_destroy(旧)` → `ui_page_load_string(新)` → **重取全部 32 个句柄 + 在新 page 上重新注册钩子**。
  **代价**：句柄表要收敛到**一个**结构（`crates/zsdesk-ui/src/shell_page.rs`），重建走**唯一一条路径**。
- **D12-11 ★ 同一棵树内 id 必须唯一（2026-11-13，前端提问时核源码得出）**
  `Widget::FindById`（`src/ui/widget.cpp:619-626`）是**深度优先、整棵子树、首个命中**的查找
  （先比 `this->id`，再按 `children_` 顺序递归，**命中即返回**）
  → **同一棵树里出现重 id → `ui_widget_find_by_id` 取到 DFS 里靠前的那个**，另一个**永远取不到**，**且不报错**。
  → **命名规则**：`main.uix` 预览窗格内的控件用 **`preview_host_*`** 前缀，`preview.uix` 自己的用 `preview_*`；
  将来若把某页面的根**嫁接**进另一棵树，两边 id 也不撞。
  （前端主动采用 `preview_host_*` 前缀 —— **采纳**，理由与本条一致。）
- **句柄数随资产更新（2026-11-13）**：`main.uix` **21 → 26**、全项目 **27 → 32**
  （新增 `preview_image` + `preview_host_{name,size,channel,state}` 四个 `<label>`；`preview.uix` 仍 6）。
  → `spec/ffi-core-ui.md` §4.6.3 / `spec/crate-layout.md` §4.5 / M1-10 / V1.13 / `SPEC.md` 风险表第 8 条**已同步**。
  → **新增一条工程约定**：句柄数**不要硬编码** → Rust 侧维护「期望 id 清单」，
  装配时断言「期望集合 == 实际集合」，资产改 id 时由**门禁**比对，而不是靠人记得改文档。
- **代价**：
  1. `.uix` 资产必须多一次机械评审（标签自闭合 / 文字位置 / 根 class / token 命名空间 / `:root` 落点 / 分隔线写法 / **宿主标签选型** / **id 唯一性**）；
  2. 根 `<div>` 必须显式加 `class="root"`（原来靠 `window {}` 的写法要改）；
  3. 每条 `.uix` 硬约束都应有一条**可执行**的检查（否则靠人记）→ 并入门禁 S13 与前端自检脚本；
  4. Rust 侧要实现一个「`:root` 是否落在 `<style>` 内」的自检（几十行）；
  5. **切主题 / 换 accent 必须重建页面 + 重取 32 个句柄** → 句柄表要收敛到**一个**结构
     （`crates/zsdesk-ui/src/shell_page.rs`），重建走**唯一一条路径**，否则漏一个 = 野指针。

---

## D13 无障碍与窗口材质 → **接受降级并记录**（用户 2026-11-13 拍板）

**实测依据**（PM 独立核实 + QA 第五轮逐项复核 + 架构侧复核）：core-ui **没有** Mica / Acrylic / 高对比度 / UIA 入口 ——
`src/ui/ui_window.cpp` 的 `DwmSetWindowAttribute` 只用于 `TRANSITIONS_FORCEDISABLED` 与圆角偏好；`mica` / `acrylic` / `backdrop` 零命中；`src/ui/theme.h` 的 `HighContrast` / `AppsUseLightTheme` 零命中；`src/ui/widget.cpp` / `ui_api.cpp` 的 `uia` / `accessibility` / `narrator` 零命中。

**因此必然丢失的能力**
| 能力 | 状态 | 说明 |
|---|---|---|
| **高对比度模式** | ❌ 不可实现 | core-ui 自绘控件不向 UIA 暴露接口，系统高对比度不会自动接管 |
| **读屏 / Narrator** | ❌ 不可实现 | 同上 |
| **Mica / Acrylic 材质** | ❌ 丢失 | zsdesk 上游自己用 DWM 实现（`crates/zsdesk-windows/src/theme.rs:131-194`，`DWMWA_SYSTEMBACKDROP_TYPE`），换 core-ui 后不复存在 |
| 深色标题栏与窗口圆角 | ⚠️ 降级 | core-ui 只暴露圆角偏好，不暴露系统材质 |

**连带动作**
1. `docs/07-UI-DESIGN-WINUI3.md` 的视觉验收项中，Mica / Acrylic / 高对比度相关条目**按「不可实现」处理**，不再作为验收门（该文档是用户的原稿，标注写入 `docs/13` 首查清单与交付说明，不改用户原稿）。
2. 已知限制必须写进 `DELIVERY.md` 与 `README-FORK.md`，不得只留在本台账里。
3. `spec/milestones.md` 的 M1/M3 验收项中，不得出现「保留 Mica」这类不可实现的承诺。

**复评条件**：若 core-ui 上游后续补上 UIA 桥接或系统材质入口，重新评估本决策。

**落地位置（架构侧已写完）**：
- `spec/milestones.md` **§8「已知限制清单」**（L1~L5 逐条 + 复评条件 + 「为什么不违背用户设计意图」的原文引用）→ 验收项 **V3-13**；
- `spec/milestones.md` M1 验收 **V1.1 的口径补充**（视觉对照**不含** Mica / 高对比度）；
- `spec/ci-contract.md` **§4.6c**（已知限制必须随包进 `DELIVERY.md` 与 `README-FORK.md`）；
- `SPEC.md` §8 out-of-scope 第 10、11 条 + 术语表新增「已知限制」。

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

**落地位置（架构侧已写完）**：
- `spec/ffi-core-ui.md` **§5.6.2「D14 品牌色来源契约」**（读取契约逐字 + 落点 + 启动日志格式 + 连带影响）；
- `spec/ffi-core-ui.md` **§5.6.1 契约 2**（初始化顺序加第 0 步；`set_accent_hex` 在 `set_mode` **之后**；切模式后必须**重读重调**）；
- `spec/ffi-core-ui.md` §4.3.2.1 **规则 5**（拼 `:root` 时**排除 `--app-accent`**）；
- `spec/crate-layout.md` **§4.4「Rust 侧启动初始化职责落点」**（读注册表 → `crates/zsdesk-windows/src/theme.rs`（改造）；拼装 → `crates/zsdesk-ui/src/theme.rs`（上游同名文件，已有 `Mode`/`Palette`/`accent`）；依赖方向不新增环）；
- `spec/milestones.md` M1 产出物 **M1-9** + 验收 **V1.11**（启动日志打印 accent 值与来源，真机核对）；
- 门禁 **S13 细化**：`themes.json` 允许 `--app-accent`（回退源值），`.uix` 禁止定义它，任何文件禁止定义库 token 全集。

---

## D15 `.uix` 通道与深浅色的五条契约口径（架构师 FFI v3 §9.4，已定；QA 独立结论同向）

| 契约 | 口径 | 依据 |
|---|---|---|
| **F1 `:root` 注入** | 拼进 `.uix` 源码里**已有的 `<style>` 块内部**（该块最后一行 `</style>` 之前）；`:root` 内**只允许 `--app-*` 变量声明**，不得写 `background`/`color`/`font` | QA 的 B1 与架构师独立核实同向：`sfc_parser.cpp:260-266` 顶层非 `<` 内容 → `doc.ok=false`；`cascade.cpp:125` 明确 `:root` 只收 CSS 变量 |
| **F2 深浅色** | 与 F1 同因：`--app-*` **不随** `ui_theme_set_mode` 变化 → 切主题**必须重建页面**并重新注入目标主题的那一套 `:root` 值 | `page_state.cpp:688-702` 的 `RefreshThemeStyles` 只重写库的 34 个键、不重新采集 `:root` |
| **F3 句柄** | 重建页面后，32 个 `ui_widget_find_by_id` 句柄**全部失效** → 必须重新绑定；旧 page 必须销毁 | 架构师 FFI v3 §9.4 |
| **F4 字体** | 字体是**库 token**，不在 `--app-*` 内 → 切主题**不**触发重建 | 架构师 FFI v3 §9.4 |
| **F5 accent** | `ui_theme_set_accent_hex` 设的基色**跨 `ui_theme_set_mode` 保留**（`theme.h:435-440` 的 `SetMode` 里 `if (AccentOverrideActive()) ApplyAccent(AccentOverrideValue());`）—— `SetMode` 只用它**重新派生 hover / press / text 变体，不换基色**。⇒ 切主题时必须**重读系统 AccentColor 并重调** `ui_theme_set_accent_hex`，否则 accent 会停在**旧模式的基色** | 架构师 FFI v3 §9.4 + QA §12 的源码复核；修正了本表早先「不随 set_mode 保留生效」的错误表述 |

**三个 Rust 侧 P0（UNVERIFIED，禁止编造签名）**——需在 M1 第一步用真机最小探针逐项验：
1. `ui_page_load_string` 的所有权约定（返回的 page / 字符串是否需要调用方释放）
2. `ui_page_set_json` 的所有权与线程约束
3. `ui_custom()` 的精确签名（`ui_custom()` + `ui_widget_add_child()` 挂载方案的落点）

**落地顺序（M1 开工即用）**：读系统 AccentColor（失败取 `--app-accent`）→ `ui_theme_set_default_font` → `ui_theme_set_cjk_font` → `ui_theme_set_mode` → `ui_theme_set_accent_hex` → 拼 `:root` 进 `<style>` 块内 → `ui_page_load_string` → 重新绑定全部句柄。

---

## D16 戳记口径与 S13 门禁口径（QA 契约核对后的裁决，2026-11-13）

### 戳记：三方口径统一为「单行 SHA」
| 来源 | 原口径 | 问题 |
|---|---|---|
| `spec/ffi-core-ui.md` §5.5 | 甲：Rust `build.rs` 编译期固化 `EXPECTED_REVISION`（**SHA**），运行时比对 | 权威口径 |
| `spec/ci-contract.md` §3.3 | 未定 | 缺口 |
| `build.yml`（原实现） | 乙：CI 写戳记，内容 = **VERSION + 换行 + SHA**（两行） | **与 §5.5 不一致 → 运行时 `EXPECTED_REVISION`（SHA）必然对不上戳记首行 → 自证机制失效** |

**裁决**
- 戳记内容 = `packaging/core-ui-revision.txt` 的**单行 SHA**（不带 VERSION、不带换行）。
- **M0 过渡**：由 CI 代写（Rust 绑定层尚未落地），但 CI 必须**机器断言「包内戳记 == `packaging/core-ui-revision.txt`」**，断言不过即 fail。
- **M1 落地**：改为 Rust `build.rs` 编译期写，CI 只做校验（不再代写）。

### S13 门禁：改成两条**可机械判死**的规则
- QA 发现契约 §4.4 写「禁自定义 `--app-accent`」，但 `assets/themes.json` **确实定义了** `--app-accent` 作源值 → 按契约字面实现门禁会 FAIL。
**裁决（与 D14 一致）**
1. **禁止自定义与库 34 个 token 同名的属性**（真正要防的：`RebuildThemeVars` 会 wipe 同名用户值）；
2. `--app-accent` **允许存在**（它只是回退源值），但**断言它不得出现在任何 `.uix` 的 `var()` 引用里** —— `.uix` 里一律读库 token `var(--accent)`，保证库自绘控件与自绘区域同色。

### 其余「当前态」落后项
由架构师同步 `spec/ci-contract.md` 的「当前态」栏：CI 布局 / `FETCHCONTENT_SOURCE_DIR_QUICKJS` / 预置目录 / `msvc-dev-cmd` / `::error::` 诊断 / `:root` 注入位置 / S7 dumpbin 门禁 / 戳记口径 / S13 门禁。`build.yml` 里的旧方案遗留注释已由交付总监清理。

---

## D17 CI 日志截断的根因与修法（2026-11-13，交付总监实测定位）

**现象**：连续两次 CI（run `43391037201` / `43392589050`）的 core-ui job 都在
`[72/97] Building CXX object CMakeFiles\core-ui.dir\src\ui\reactive\binding.cpp.obj`
这一行**戛然而止**，之后没有任何编译器输出；check-run 的 8 条 annotation 里
**没有一条真正的 clang 报错**，全是 CMake 缓存变量（`QJS_BUILD_WERROR: OFF` /
`BUILD_SHARED_LIBS: OFF`）。

**根因**（不是编译错误）：
- GitHub 的 `shell: pwsh` 注入 `$ErrorActionPreference = 'stop'`；
- PowerShell 7.4+ 的 `$PSNativeCommandUseErrorActionPreference` 默认为 `$true`；
- 二者叠加 ⇒ native command（clang-cl）**只要往 stderr 写一行 warning**，
  就被判定为失败并抛 `NativeCommandExitException` → **管道被中断** →
  `Tee-Object` 停止 → 编译器的真实诊断**从未落盘**。

⇒ 所以「死在 binding.cpp」是假象：真正的 warning/error 一直在，只是被丢了。
而且「最后一行 warning」与「最后一行 error」都会触发中断，**位置随机**。

**修法**（`build.yml` 的 core-ui job 第一行）：
```powershell
$ErrorActionPreference = 'Continue'
$PSNativeCommandUseErrorActionPreference = $false
```
关掉之后 stderr 正常流进日志。

**连带修**：annotation 提取的正则原为 `-match`（**不区分大小写**），
会把含 `ERROR` 的 CMake 缓存变量全部命中、占满 10 条配额，把真正的错误挤掉
→ 改为 `-cmatch`（区分大小写），只认 clang 的 `file.cpp(12,5): error:` /
`error C2220:` / `fatal error:` 与 ninja 的 `FAILED:`。

**教训**：CI 里「日志看不到」不是日志不存在；**先证明管道没被截断**，再谈编译错误。

---

## D18 门禁必须在 CI 里真正执行（QA 第九轮纠正，2026-11-13）

**问题**：四个机械门禁脚本（`uix_tag_check` / `theme_contrast_check` /
`qa_uix_verify` / `s13_token_namespace_check`）此前**只存在于仓库里，CI 从不执行**。
按「**没有执行器的规则只是建议**」，S13 那一整套口径当时并不成立。

**裁决**：
1. 五个门禁全部接进 `build.yml` 的 **rust job**（新增步骤「Gate S13」），
   任一非零退出即**阻断该 job**，并以 `::error::` annotation 暴露是哪一条。
2. 新增第六个门禁 `scripts/gates/ui_id_manifest_check.py` ——
   **id 清单不再硬编码在文档里**，而是单一事实源 `assets/ui-ids.json`，
   门禁双向比对（清单有而页面缺 / 页面有而清单缺 / 计数不符 / 页内重复 id）。
3. 解析 `.uix` 的 id 时**必须先剥掉 HTML 注释** —— 注释里常写
   `→ <custom id="preview_image">` 这类说明文字，裸正则会误判成「重复 id」。

---

## D19 S7' 导出面门禁的判定基准（2026-11-13）

**问题**：`docs/05-COREUI-EXPORTS.txt` 记录的是**本机预编译 DLL（1.9.0.346）**
的实测导出面，而 CI 编的是**钉住的 revision 345**。直接比对两者会得到
**版本漂移**，而非真实缺陷。

**裁决**：S7' 的基准 = **同一 checkout 里 `src/` 的 `UI_API` 定义集**。
- 版本天然一致（同一个 checkout），不存在跨版本比对；
- **不用头文件做基准**：上游 `include/ui_core.h` **漏声明** 12 个符号
  （`ui_image_gdi_*` 与 `ui_flush_gpu`，在 `src/ui/ui_api.cpp` 里确实有
  `UI_API` 定义）→ 用头文件会把它们误判成「unexpected」；
- `ui_dialog` 在头文件里声明了但源码里**没有** `UI_API` 定义 → 库里不导出它，
  属上游已知缺口，**容忍并报告**（不静默放宽）。

**实现**：`scripts/gates/pe_exports.py`（无外部依赖的 PE 导出表解析器）
+ `scripts/gates/coreui_export_gate.py`（比对）。
**验证**：解析器用本机 `GuoheView\core-ui.dll` 实测，**逐字节复现**
`docs/05` 的 567 个 `ui_*` 基线；正例（346 DLL 对 345 源码）通过、
反例（截断 DLL / 无 `ui_*` 的 kernel32.dll）均正确失败。

---

## D20 `:root` 注入形态 = 甲（源码最前面插入完整 `<style>` 块）（2026-11-13）

**背景**：core-ui 的 `:root` 是**先定义者赢**
（`value.cpp:572-579` 的 `LookupVar` 返回第一个命中、
`cascade.h:41` 注释逐字「Returns first set encountered」、
`cascade.cpp:125` 让 `:root` 整条跳过 CSS 级联）。
⇒ Rust 注入的 `:root` 必须排在页面自己的 `:root` **之前**，否则被**静默吃掉**
（症状：切主题颜色不变，零报错）。

**候选形态**：
| 形态 | 做法 | 结论 |
|---|---|---|
| 甲 | 在**源码最前面**插入一个完整的 `<style>` 块 | ✅ **采纳** |
| 乙 | 插进页面**已有的**第一个 `<style>` 块块首 | ⚠️ 合法但脆弱（页面没有 `<style>` 时无从下手） |
| ✗ | 裸 `:root{}` 放在源码顶层 | ❌ **整页解析失败** |

**甲为什么成立**（源码级验证）：
- 顶层标签**顺序无关**（`sfc_parser.cpp` 按标签名分派到各自容器）；
- `doc.styles` 按源码中 `<style>` 出现的顺序收集（`sfc_parser.cpp:330`）；
- `page_api.cpp:333` 按该顺序**逐块 parse 压入同一 sheet**
  ⇒ 插在最前面的块是 `blocks[0]` ⇒ 它的 `:root` 排在页面自己的 `:root` 之前
  ⇒ **页面自己的定义胜出**（符合「页面可覆盖品牌默认值」的直觉）。

**✗ 为什么绝对不行**：`sfc_parser.cpp:264-270` 明确只允许
`<window> / <template> / <script> / <style> / <import> / <link>` 作为顶层标签，
任何不以 `<` 开头的顶层内容 → `"unexpected content at top level"` →
`doc.ok = false` → **整页不加载，且报错信息完全不像标签问题，极难定位**。

**落地**：`crates/zsdesk-coreui/src/lib/theme.rs` —— 形态甲 + 单元测试 9 条。

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

## 变更记录## 变更记录## 变更记录
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
- 2026-11-13 **第五轮（D13 / D14 落 Spec）**：
  - **D13（无障碍与窗口材质 → 接受降级并记录）落地**：
    `spec/milestones.md` 新增 **§8 已知限制清单**（L1 高对比度 / L2 读屏 / L3 Mica·Acrylic /
    L4 深色标题栏与圆角 / L5 系统强调色由 Rust 实现），含「用户原稿 §4 原文把 Mica 写成『可选』→
    丢掉它**不违背设计意图**」的引用与**复评条件**；
    M1 验收 **V1.1 加口径补充**（视觉对照不含 Mica / 高对比度，**不作验收门**）；
    M3 新增验收 **V3-13**（已知限制必须随包进 `DELIVERY.md` 与 `README-FORK.md`）；
    `spec/ci-contract.md` 新增 **§4.6c**（交付条款 + 可机械检查的形式）；
    `SPEC.md` out-of-scope 补第 10、11 条 + 术语表补「已知限制」。
    ★ 实测依据已由架构侧独立复核（core-ui 全仓 `GetSysColor`/`UISettings`/`AccentColor`
    `uia`/`accessibility`/`narrator` `mica`/`acrylic`/`backdrop` **全部零命中**）。
  - **D14（品牌色 → Rust 侧读系统 AccentColor）落地**：
    `spec/ffi-core-ui.md` 新增 **§5.6.2**（注册表路径 / `REG_DWORD` / **BGR 序 `0x00BBGGRR`** /
    转换公式 / 非法值当作读不到 / 回退链 / 切模式后重读重调 / 落点 / 启动日志格式）；
    契约 2 初始化顺序加**第 0 步**；新增**规则 5**（拼 `:root` 排除 `--app-accent`）；
    `spec/crate-layout.md` 新增 **§4.4**（启动初始化职责落点：读注册表归 `zsdesk-windows`，
    拼装归 `zsdesk-app`，`zsdesk-coreui ↛ zsdesk-windows` → 不新增环）；
    M1 新增产出物 **M1-9** 与验收 **V1.11**（启动日志打印 accent 值与来源）。
    ★ **连带修正前端的一处口径**：`assets/themes.json` 的 `--app-accent` 现在是
    **回退源值**（数据字段），**Rust 不得把它注入 `:root`**；
    门禁 S13 因此改为**按文件类型区分**（`themes.json` 允许、`.uix` 禁止）。
  - **戳记契约（口径「甲」）补全为可执行条款**：`spec/ffi-core-ui.md` §5.5 给出
    「谁写 / 何时 / 写什么 / 写到哪 / CI 断言」五行 + **可直接粘贴的 PowerShell 断言**，
    并点名 **CI 要删的那一步**（Gate G2 里 `Set-Content` 写两行戳记）；
    `spec/ci-contract.md` §4.2 由「产出方待定」改为 **✅ 已定：Rust 产出**。
  - **`:root` 注入的四口径定稿**（前端已按此改完 `assets/themes.json`）：
    `spec/ffi-core-ui.md` §4.3.2.1 与 `spec/crate-layout.md` §4.3.1 **逐字一致** ——
    ① 拼进 `<style>` 块内（Rust 统一**在源码最前面插入**完整 `<style>` 块 —— **必须是第一个 `:root`**，见 D10 契约 1(d)）；
    ② 只允许 CSS 变量；③ 根样式走 `.root` class；④ Rust 侧加载前自检 + fail-fast。
- 2026-11-13 **第六轮（前端第二轮 P0：`:root` 注入位置 + 切主题句柄失效 + 分隔线）**：
  - **架构侧逐条独立核对上游源码**，结论：两条 P0 **成立**；另**更正前端一处事实**。
  - **D12-5 补强**：`:root` 注入位置已由 QA B1 与前端两条消息**三方一致**，
    并在 `spec/ffi-core-ui.md` §4.3.2.1 末尾补上**「加载后断言颜色能不能做」的诚实答案**：
    core-ui **无读回 CSS 变量的 C API** → 只能靠**离屏渲染取像素**（一次性冒烟，不进启动路径）；
    启动路径只保留**加载前自检**。
  - **D12-6 新增**：`<separator>` **不画 CSS `background`**（`controls.cpp:635-643`，
    颜色写死 `theme::kDivider()`）→ 布局分隔线要跟 `--app-divider` 同色**必须**用
    1px `<div class="sep"/>`。
    **更正前端**：「demo 里 `<separator>` 一次没用」**不准确** —— 实测 `class="sep"` **19 处**、
    `<separator/>` **3 处**（那 3 处**全在 `<menu>` 内部**）；**结论不变**。
  - **D12-7 新增**：页面重建后 **句柄全部失效**（**2026-11-13 资产更新后为 32 个**：`main.uix` 26 + `preview.uix` 6，
    架构侧已复核 distinct 数）→ 三步协议（destroy → load → 重取全部句柄）；
    `ui_page_reload` 重放旧源码**帮不上忙**（`page_api.cpp:565-577`）且**同样重新编译**；
    「自动回填」只覆盖**自绘回调**，不覆盖 Rust 侧句柄表。
    → 新增 `spec/ffi-core-ui.md` **§4.6「页面重建的生命周期契约」**、
    `spec/crate-layout.md` **§4.5**（句柄表落点 `zsdesk-ui/src/shell_page.rs`）、
    M1 产出物 **M1-10**、M1 验收 **V1.13**；`SPEC.md` 风险表新增第 **8** 条（野指针）。
  - **D12-8 新增**：「加载后断言颜色」在 API 层面做不了（无读回 API）→ 改为离屏像素冒烟
    （M1 验收 **V1.15**）。
  - **M1 验收新增 V1.14**：分隔线颜色 = `--app-divider`。
  - **`.uix` 硬约束由三条增至四条**（§4.3.3 标题与 `spec/milestones.md` V0.14、`spec/ci-contract.md` 关联文档同步）。
- 2026-11-13 **第七轮（前端第三轮：id 归属 + 挂载钩子是否自动转移 + 三个宿主待裁决）**：
  - **① id 以资产为准 = `list_host`** —— 前端指出我在两处举例写了 `file_list` / `list_list`，
    而资产的既有 id 是 `list_host`（Rust 侧 `find_by_id` / `on_widget_mount` / `<menu trigger="#…">` 都用它）
    → **前端是对的**（id 是 Rust ↔ `.uix` 的契约，不能各写各的）；
    `spec/ffi-core-ui.md` / `spec/crate-layout.md` / `SPEC.md` / `DECISIONS.md` / ADR-0001
    **共 8 处举例已全部改正**；命名规则定为 **`<区域>_host`**。
  - **② ★ 架构侧自查后修正自己上一版的措辞**（「按 id 持久化 → 重建后自动回填」**不成立**）：
    `ui_page_on_widget_mount` 的钩子挂在**具体 page** 的 `PageState` 上（`page_api.cpp:850-854`）
    → 同一个 page 内晚出现的控件会自动补发 ✅（catch-up，`page_state.cpp:635-652`），
    但**新 page 的钩子表是空的** → **不自动转移** ❌ → 必须在**新 page 上重新注册**（catch-up 保证注册即触发）。
    → 重建协议由「重取句柄」扩为「**重取全部句柄 + 在新 page 上重新注册钩子**」（§4.6.2 / §4.6.4 / D12-10）。
  - **③ 三个宿主的裁决**（新增 `spec/ffi-core-ui.md` **§4.3.4**，含判定规则）：
    `nav_host` → **改 `<custom>`**（`<div>` 上 `ui_custom_on_draw` 是**静默 no-op** → 树画不出来）；
    `breadcrumb` → **不改 `<custom>`**，但**要改形态**：`<label id="breadcrumb">` + Rust `ui_label_set_text`
    （现在是**空 `<div>` → 显示不出任何东西**，是真缺口，M1 验收新增 **V1.16**）；
    `preview_host` → **保持 `<div>` 容器** + 里面加 `<custom id="preview_image">` + 预先声明的 `<label>` 元信息行。
  - **D06-1 预览承载方式细化**：不用 `ui_image_view()` 当子控件（运行时创建 → **无 CSS** → 尺寸手算 px，
    且运行时插入的子控件如何参与父容器布局 core-ui **没有公开承诺**；声明式 `<img>` 的 `src` 是**资产 key**、
    **无运行时改 src 的 API**）→ 改为 Rust 在 `<custom>` 里 `ui_bitmap_create` + `ui_draw_bitmap_handle`；
    **D06 的实质不变**（WIC 解码、留程序内、不依赖 ghde），**像素管线不变**（仍是 BGRA 预乘），
    **代价** = 缩放/平移要自己实现（首版只 fit），M3+ 走 `ui_image_gdi_*` 或显式定位子控件。
  - 新增 `spec/crate-layout.md` **§4.6 自绘宿主清单**；M1 产出物 M1-3/M1-4 与 M2-5 落点同步。
- 2026-11-13 **第八轮（QA 第五轮 Δ-A / Δ-B1 / Δ-B2 + 运维 P0 + 前端复核）**：
  - **★ D10 契约 1 的规则 (d) 被 QA 纠正，架构侧已独立复核并改正 —— 我上一版「方向反了」**：
    此前写「注入的 `:root` 必须在 source order 上**最后**（后者覆盖前者）」→ **错**。
    源码级依据（架构侧亲自复核，非转述）：`CollectRootVars`（`cascade.cpp:20-33`）按**文档顺序**推进有序向量；
    `LookupVar`（`value.cpp:572-579`）**返回第一个命中**；
    `cascade.h:41` 注释逐字「**Returns first set encountered**」；
    且 `:root` 规则在普通级联里被**整条跳过**（`cascade.cpp:125`）→ **根本不参与 CSS 级联**
    ⇒ **自定义属性是「先定义者赢」**，CSS 的「后者覆盖前者」常识**在这里不适用**。
    **后果**：注入若排在 `.uix` 自己的 `:root` 之后 → **`.uix` 的值赢** → Rust 的注入被**静默忽略**
    → 症状「切主题颜色不变」，**零报错**（QA 实测：三个 `.uix` 当前**生效**的 `:root` 数 = **0** → 今天不暴露，但是埋着的雷）。
    → **注入形态从「源码末尾追加」改为「源码最前面插入」**（顶层块顺序无关，解析器按标签名分派）；
    → **Rust 自检由一条扩为四条断言**：位置 / **顺序**（注入的是第一个 `:root`）/ **内容**（`:root` 内每条声明匹配
    `--app-[a-z0-9-]+\s*:`）/ 键集（无 `--app-accent`、不撞库 token 34 个）→ 任一不满足 fail-fast；
    → **规则由四条扩为六条**（新增「只允许 CSS 变量」的**断言版** + 键集排除）；
    → `spec/ffi-core-ui.md` §4.3.2.1 与 `spec/crate-layout.md` §4.3.1 的对外措辞**逐字同步**；
    → **M1 验收 V1.12 扩为 4 个坏源码构造**（裸放顶层 / `.uix` 自带 `:root` 抢先 / 混入普通声明 / 键集含 `--app-accent`）。
    ★ **前端需同步**：`assets/themes.json` 的 `revision.inject` 注记现在写的是「末尾追加」（形态甲的旧定义）→ **要改成「最前面插入」**。
  - **D13 证据口径修正（QA Δ-B1）**：`backdrop` **不是零命中**（架构侧复核：`src/` 全仓 38 处命中） ——
    ① **CSS `backdrop-filter` / `backdrop-blur` 在任意 widget 上受支持**（`widget_factory.cpp:606-610` → `w.backdropBlur = parseBlur(...)`；
    `widget.h:245` `float backdropBlur = 0.0f;`）→ 将来做浮层模糊**用得上**；
    ② 上下文菜单的背景模糊捕获（`context_menu.cpp`）。
    **窗口级** Mica/Acrylic 确实零命中（`DWMWA_SYSTEMBACKDROP` / `DWMWA_MICA` / `SetWindowCompositionAttribute` / `ACCENT_POLICY` 全仓 0 命中）✓
    → **结论不变**（D13 的三项限制成立），但**关键词清单要按此改口径**，否则读者会误以为「连 widget 级模糊都没有」。
  - **运维 P0 落进契约（`spec/ci-contract.md` §4.7 + §3.5 第 34 条）**：
    `package` job **只 checkout 了 `app/`**，没有 `zsui/` → 而 `crates/zsdesk-{app,ui}/Cargo.toml` 有 `zsui = { path = "../../../zsui" }`
    → cargo **装载 workspace 清单即失败** → 连炸冒烟探针与戳记断言两处；
    且戳记断言的报错「build.rs did not run」**指向错误根因**（真正原因是「zsui 没检出」）→ 最费时间的一类失败。
    → 修法：把 `rust` job 的「读 pin + checkout zsui」两步原样复制到 `package` job；
    → **通用规则写进契约**：**凡跑 cargo 的 job，都必须具备 workspace 的全部 path 依赖**。
  - **H2 措辞订正（运维复核）**：「递归会拉一个体量极大的 tc39/test262」这个理由**不成立**
    （`update = none` → git 不会去更新该子模块 → 递归也不会拉它）；
    **正确理由**：v0.14.0 **根本没有构建必需的子模块** → 递归纯属多余。**结论不变**（不带 `--recurse-submodules`）。
  - **第九轮（前端第八轮：三处宿主已落 + 句柄数 27→32）**：
    - **采纳前端的两点决策**：① 元信息行 id 用 **`preview_host_*`** 前缀（与 D12-11 的 id 唯一性规则一致）；
      ② `assets/preview.uix` 的 `preview_body` 也改 **`<custom>`**（按同一条判定规则 + D06-1 图片统一由 Rust blit）→ **架构侧确认**。
    - **架构侧复核**：按「剥注释后正则计数」实测 `main.uix` **26** 个 id、`preview.uix` **6** 个、`about.uix` **0** 个
      → **合计 32**，与前端报的一致；契约里的「27」已全部改为「32」。
    - ⚠️ **给前端留的一条**：`assets/preview.uix` 头注释里**同时**写着两段互相矛盾的话 ——
      「四、preview_body 是图片位 → Rust blit → `<custom>`」与「五、…由 core-ui 控件填充（`<gh-img-view>` 等）→ 保持 `<div>`」，
      而元素实际是 `<custom>` → **注释与实现自相矛盾**，QA 的资产复核会挑出来；
      且「`<gh-img-view>` 填充」与 `SPEC.md` §8 第 7 条（不实现瓦块金字塔喂数）冲突、**不可执行** → 请把第五段改成与实现一致的表述。
- 2026-11-13 **第十轮（QA 第五轮：口径⑤ 的四条证据 + 三条新证据 + 提请裁注入位置 + 行号漂移）**：
  - **注入位置的正式裁定（QA 提请）→ 选「甲A：在源码最前插入」**（与上一轮契约改动一致，本轮**补全前提并定案**）：
    - 理由：① 无条件成立（不依赖「`.uix` 没有 `:root`」这个巧合）；② Rust 侧代码量相同（prepend/append 都是一次 concat）；
      ③ 证据 C（SFC 顶层标签**不限顺序**，`sfc_parser.cpp:252-297`）保证语法合法；
      ④ 唯一残留前提 = 证据 B（`<link>` CSS 里不能有 `:root`）→ 本项目零 `<link>`，Rust 自检顺手断言。
    - **否掉「甲B：保留末尾追加 + 靠门禁禁止 `.uix` 出现 `:root`」**：把正确性押在约定上，而违约代价是**静默失效**。
    - ⇒ **口径⑤的完整表述**（比之前更严）：「注入的 `:root` 必须是**整张样式表**里的第一个 `:root`；
      整张样式表 = **`<link>` CSS（在前，`page_api.cpp:271-277`）+ inline `<style>`（在后）**」。
  - **三条新证据全部复核成立并写进契约**：
    **A** `page_api.cpp:333-350`（各 `<style>` 块的规则依次 push 进**同一张** `sheet.rules`）→「源码里第一个 `:root`」==「`sheet.rules` 里第一个 `:root`」→ 口径**可写成断言**；
    **B** `page_api.cpp:271-277`（`<link>` 的 CSS **排在** inline `<style>` **之前**）→ 口径的完整前提要含「`<link>` CSS 里无 `:root`」；
    **C** `sfc_parser.cpp:252-297`（顶层标签不限顺序）→ **源码最前插入**语法合法。
  - **D10 的三条源码级补强（QA 补证，架构侧复核确认 → 策略依据更硬）**：
    ① `CollectVarsWithTheme`（`compiler.cpp:1578-1590`）用户 `:root` 在前 + 库 token 带 `has()` 守卫补后 ⇒ **用户 `:root` 赢库默认**；
    ② `RebuildThemeVars`（`compiler.cpp:1647-1703`）切模式时 erase+重写 34 个库键 ⇒ 用户对库 token 的覆盖**会被清空** ⇒ **`--app-*` 前缀策略依据成立**；
    ③ `RefreshThemeStyles`（`page_state.cpp:688-702`）只 `RebuildThemeVars`、**不重新采集 `:root`** ⇒ 切主题后 `--app-*` **仍是旧模式的值** ⇒ **「切主题必须重建页面」是必需的，不是可选的**。
  - **行号漂移修正（P3）**：`LookupVar` 实测在 **`value.cpp:572-579`**（此前写 572-581）→ `spec/**` + `DECISIONS.md` **共 3 处已改**；
    `assets/themes.json:14`、`docs/12:277/289/396/532/541`、`docs/13:33` 属**前端 / QA 的文件域**，由他们自行改（QA 已在报告里列出）。
  - **前端复核**：① `assets/themes.json` 两处注记**由前端自己改**（架构侧不再动该文件）；
- 2026-11-13 **第十一轮（QA 第六轮：S13 落地 + 门禁执行契约 + 运维 P0 闭环 + id 清单门禁）**：
  - **S13 已落地**：`scripts/gates/s13_token_namespace_check.py`（QA 新建，独立脚本）——
    逐字口径实现 + **负控已证**（注入 6 类缺陷 ⇒ EXIT=1，逐条带 `文件:行`）。
    **QA 的两条实现决定，架构侧采纳**：
    ① **独立成脚本**，不塞进 `qa_uix_verify.py` —— S13 是**契约门禁**（有独立编号、会被里程碑与 CI 契约引用），
    独立脚本可被 CI **单点调用**、退出码自洽；与「资产质量门」职责不同、互不依赖
    （`.uix` 禁 `var(--app-accent)` 与 `qa_uix_verify.py` 断言 16 **有意重叠** → 两个独立执行器，一个失败不掩盖另一个）。
    ② **① 做成【34 个名字的名单】**，不做【命名空间不相交】—— 项目 token 是 `themes.json` 里**显式枚举**的一组名字，
    不是靠前缀推导的命名空间 → 「前缀 vs 库 token 名」这个判据是**空的**；
    S13 要挡的真实失败模式是「**把库 token 名误抄进项目 token 集**」→ **只有成员判定挡得住**。
    脚本每次运行都打印这 34 个名字（便于与上游 `compiler.cpp:1648-1658` 逐字 diff），
    **条数 ≠ 34 直接 FAIL**（上游升级后名单没同步 ⇒ S13 会漏判，必须让门禁自己先炸）。
    ★ **S13 的三条「附加」检查也纳入契约**（不改四条逐字口径，各自独立标注）：
    **A** token 名形状 `--app-[a-z0-9-]+`（= Rust 自检断言 3 的静态对应，两侧合起来才闭环）；
    **B** `themes.json` 的 **light/dark 键集必须一致**（= 规则 4「切主题要重注入」的前提；键集不等 ⇒ 切到缺键那侧 ⇒ 该 token 静默回落**库默认色**，不报错）；
    **C** 库 token 名单**条数 ≠ 34 ⇒ 直接 FAIL**（① 的自保）。
  - ★ **id 清单门禁落地**（= 上一轮我提的那条）：`scripts/gates/ui_id_manifest_check.py` +
    期望集文件 **`assets/ui-ids.json`**；四条检查：清单里的 id 都在 `.uix` / `.uix` 里的 id 都在清单 /
    逐页条数与总数一致 / **同页无重复 id**（= **D12-11**）。
    **架构侧本地实跑：EXIT=0，报「共 32 个唯一 id，3 页」** —— 与我独立重数（`main.uix` 26 + `preview.uix` 6 + `about.uix` 0）**一致** ✓
  - ★ **新增 `spec/ci-contract.md` §4.8「门禁执行契约」**（QA 明确请架构侧写措辞）：
    现状 = `build.yml` **零处**调用 `scripts/gates/**` → **脚本都在、CI 一步都不跑** → 按
    `mechanical-gates.md`「没有执行器的规则只是建议」→ **等于没有门禁**。
    契约内容：五个脚本（**S13 / id 清单 / S7' 导出面 / 资产质量三支**）的**逐条命令**、
    **每门禁一个独立 step**、**退出码非 0 ⇒ job 失败（fail-stop，禁止 `|| true` / `continue-on-error`）**、
    **按 job 归属落位**（`rust` job 跑资产类；S7' 需要 DLL → 放 `package` job）、
    **门禁必须零第三方依赖**（否则「装依赖」会成为新的失败点）。
    → 同步：M0 验收新增 **V0.16「门禁在 CI 里真的被执行」**；`spec/crate-layout.md` 门禁表补 S13 与 id 清单两行的脚本路径。
  - **`<link>` 残留缺口已写进契约**（QA 提出，架构侧复核成立）：
    「顶层块顺序无关」这句话**只对 inline `<style>` 成立** —— `<link>` 引的 CSS 由 `ResolveLinkBlocks` 定位，
    **恒排在 inline 之前**，与它在源码里的位置无关 → `<link>` 的 CSS 里若有 `:root`，注入值仍会输。
    → `spec/ffi-core-ui.md` §4.3.2.1 的形态甲说明已加限定 + 显式提示；本项目零 `<link>` → 前提成立。
  - **运维 P0 闭环** ✅：`package` job 已补 `zsui` 检出（`build.yml:359-380`，注释块 `:363-368` 写明推理）；
    根目录**无 `.gitmodules`** → 印证 H2 的订正（「v0.14.0 没有构建必需的子模块」）。
- 2026-11-13 **第十二轮（契约与 D15~D20 对齐 + QA 第六轮门禁落地）**：
  - **按交付总监新落的 D15~D20 同步契约**（架构侧只改 Spec，不改 `build.yml`）：
    - **D15-F1 的「注入位置」作废**（`spec/ffi-core-ui.md` §4.3.2.1 顶部已注明）—— 它写的「插进已有 `<style>` 块的**最后一行** `</style>` 之前」
      在 **first-wins** 下仍会被页面自己的 `:root` 抢先 → 位置一律按 **D20 甲（源码最前面插入）**；F1 其余部分（只允许 `--app-*`）**有效**。
    - **D16（戳记分期）** 写进 `spec/ffi-core-ui.md` §5.5 与 `spec/ci-contract.md` §4.2/§4.6b：
      **M0 过渡 CI 代写 + 必须机器断言** → **M1 改 Rust `build.rs` 产出、CI 只校验**；戳记内容 = **单行 SHA**（不带 VERSION）。
    - **D17（CI 日志截断）** 写进 `spec/ci-contract.md` **§5.0（置顶）** —— 这是「core-ui 编不出来」的最可能真因：
      `shell: pwsh` 注入 `$ErrorActionPreference='stop'` × PS7.4+ 的 `$PSNativeCommandUseErrorActionPreference=$true`
      ⇒ clang-cl **只要往 stderr 写一行 warning** 就被判失败 → `Tee-Object` 停 → **真实诊断从未落盘**；
      修法 = core-ui job 第一行 `$ErrorActionPreference='Continue'` + `$PSNativeCommandUseErrorActionPreference=$false`；
      annotation 正则 `-match` → **`-cmatch`**（原正则不区分大小写 → 把含 `ERROR` 的 CMake 缓存变量全命中、占满 10 条配额，把真错误挤掉）。
      **教训写进契约**：CI 里「日志看不到」不是日志不存在 → **先证明管道没被截断**，再谈编译错误。
    - **D18（门禁在 CI 里执行）** → 新增 `spec/ci-contract.md` **§4.8「门禁执行契约」**：
      五个脚本的**逐条命令**、**每门禁一个独立 step**、**退出码非 0 ⇒ job 失败（fail-stop）**、**`::error::` 暴露是哪一条**、
      **按 job 归属落位**（资产类进 `rust` job；**S7' 需要 DLL → 进 `package` job**）、**门禁零第三方依赖**、
      **解析 `.uix` 的 id 必须先剥 HTML 注释**（注释里的 `→ <custom id="preview_image">` 会被裸正则误判成「重复 id」）。
    - **D19（S7' 基准）** → `spec/ci-contract.md` §3.3 第 20 条改写：基准 = **同一 checkout 的 `src/` 的 `UI_API` 定义集**
      （不用 `docs/05` 的本机 346 基线 —— 跨版本只会得到「版本漂移」；不用头文件 —— 上游漏声明 12 个符号会被误判成 unexpected；
      `ui_dialog` 头文件有、源码无 → **容忍并报告**，不静默放宽）。
  - **QA 第六轮的两条实现决定，架构侧采纳**（S13 独立成脚本 + ① 做成 34 个名字的**名单**）：
    理由分别是「S13 是契约门禁、需可单点调用且退出码自洽」与「项目 token 是显式枚举的名字集、不是命名空间 → 前缀判据是空的，
    只有成员判定挡得住『把库 token 名误抄进项目 token 集』」。**S13 的三条附加检查（A 名字形状 / B light-dark 键集一致 / C 名单条数≠34 即 FAIL）也已入契约。**
  - **架构侧本地实跑（只读）**：五个门禁脚本全部 **EXIT=0**；`ui_id_manifest_check.py` 报「共 **32** 个唯一 id，3 页」
    —— 与架构侧独立重数（`main.uix` 26 + `preview.uix` 6 + `about.uix` 0）**一致** ✓
  - **`<link>` 残留缺口入契约**：「顶层块顺序无关」**只对 inline `<style>` 成立** —— `<link>` 的 CSS 恒排在 inline 之前
    （`page_api.cpp:271-277`）→ `<link>` 的 CSS 里若有 `:root`，注入值仍会输；本项目零 `<link>` → 前提成立（Rust 自检断言 2 会查）。
  - **运维 P0 闭环** ✅：`package` job 已补 `zsui` 检出；根目录无 `.gitmodules` → 印证 H2 订正。
