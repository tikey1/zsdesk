# 事实核对底稿（编排者实测，未含推测）

> 只记录**已核实**的事实。来源标注在每节末尾。
> 本文件不是 PRD，也不是技术 Spec——那两份由 PM / 架构师产出。

---

## 1. 基准项目 qiu7824/zsdesk（已核实）

| 项 | 实测值 | 来源 |
|---|---|---|
| 定位 | Windows 原生多标签文件管理器 + 桌面格子 | 仓库 README |
| 技术栈 | Rust + Win32/COM + Direct2D + DirectWrite | 仓库 README |
| 当前版本 | 0.3.11 | 仓库 README / Cargo.toml |
| workspace 成员 | `crates/zsdesk-core`、`zsdesk-platform`、`zsdesk-app`、`zsdesk-ui`、`zsdesk-windows` | `Cargo.toml`（raw 实测） |
| Rust 基线 | `edition 2021`，`rust-version = "1.89"`，CI 用 1.99 | `Cargo.toml`（raw 实测） |
| 共享依赖 | serde / serde_json / tracing / tracing-subscriber / anyhow / dirs | `Cargo.toml`（raw 实测） |
| release profile | `opt-level = 3`，`lto = "thin"` | `Cargo.toml`（raw 实测） |
| UI 库 | **ZSUI**（MPL-2.0），源码固定版本记录在 `packaging/zsui-revision.txt`，需与本仓库同级放置 `zsui/` 目录 | 仓库 README |
| ZSUI 承担的职责 | 原生窗口、虚拟列表、输入、快捷键 | 仓库 README |
| Shell 依赖 | 文件列表用 `IExplorerBrowser`；左侧用原生 Shell 导航树；菜单走原生 Shell 菜单宿主 | 仓库 README |
| 目标平台 | Windows x64（无 32 位产物） | 仓库 README「构建」 |
| 许可证 | 项目 MIT；ZSUI MPL-2.0；MinHook BSD-2-Clause；Everything 自带许可 | `LICENSE` / `THIRD-PARTY-NOTICES.txt` |

**关键结论**：zsdesk 的界面 = ZSUI（自绘/原生混合）+ Windows Shell 原生控件，二者深度咬合。替换 ZSUI **不是换一个依赖**，而是换掉整套控件与窗口承载。

---

## 2. UI 库 ghboke/core-ui（已核实）

| 项 | 实测值 | 来源 |
|---|---|---|
| 定位 | Windows 原生桌面 UI 框架，Direct2D/D3D11 硬件加速 | 仓库 README |
| 设计体系 | 严格对齐 Microsoft **Fluent 2** 设计令牌（颜色/圆角/阴影/动效） | 仓库 README |
| 控件数 | 25+ / 29+ 内置控件（button、input、toggle、select、progressbar、menu、Dialog、Toast、TitleBar…） | 仓库 README |
| 对外接口 | **纯 C API，400+ 导出函数**，句柄全是 `uint64_t`，无 C++ 类型外泄 | 仓库 README |
| 语言绑定 | 官方明示 Rust / Go / Python / C# / Delphi / Lua 可直接绑定，**无需 C++ shim** | 仓库 README |
| 声明式 UI | `.uix`（Vue 3 SFC 风格：`<window>` + `<script>` + `<style>` + `<template>`）与 `.ui` 标记文件；脚本由内嵌 **QuickJS-NG** 进程内求值 | 仓库 README / `UI_CORE_API.md` |
| 版本 | `1.9.0`，内部 build `345`，发布日 2026-11-01 | `CMakeLists.txt`（raw 实测）/ 仓库首页 |
| 尺寸 | 单 DLL 约 4.1 MB，无 Chromium / 无 .NET / 无 Qt | 仓库 README |
| 工具链 | CMake ≥ 3.20、C++17、**仅支持 MSVC / clang-cl（MinGW 明确不支持）**，需 `vcvars64` + `llvm-rc` 替换 `rc.exe` | `CMakeLists.txt`（raw 实测）/ 仓库 README |
| 构建脚本 | `scripts/build-clang-cl.ps1 -Target core-ui`，产物 `core-ui.dll` + `core-ui.lib`；`-Static` 出单 exe | 仓库 README |
| CRT 开关 | `UI_CORE_MSVC_STATIC_CRT=OFF` 默认 `/MD`（动态 CRT，需 vcredist）；开则 `/MT` 静态链接 | `CMakeLists.txt`（raw 实测） |
| 资源内嵌 | `ui_core_embed_text(FILE app.uix OUT app.embed.h VAR k_app)` 可把 `.uix`/`.lang` 烤进 exe | `CMakeLists.txt`（raw 实测）/ 仓库 README |
| 许可证 | MIT | 仓库 README / `LICENSE` |

**关键结论**：core-ui 天然满足「Fluent 观感 + Win10 兼容 + Rust 主导」，且**不引入 WinUI 3 / Windows App SDK 运行时**，这比真接 WinUI 3 更轻、更贴 Win10 用户习惯（WinUI 3 需 Win10 1809+ 且带 App SDK 运行时）。

---

## 3. 本机 GuoheView（C:\Program Files\GuoheView，已实测）

### 3.1 文件清单
```
GuoheView.exe            4,287,944   主程序
core-ui.dll              4,782,536   ← 同名 UI 框架
ghde.dll                 6,949,320   ← 图像解码引擎
guohe-view-thumb.dll       363,976   ← COM 缩略图扩展
GuoheViewCollect.exe       636,872
GuoheView-Updater.exe      361,416
news.exe / pdfdecode.exe
plugins/decoder/jxl.dll            ← 插件式解码器
plugins/ocr/gv-ocr.dll + onnxruntime.dll + PP-OCR v3 模型（det/rec/cls）
resources/icc/sRGB2014.icc, JapanColor2001Coated.icc
resources/icons/*.ico              ← JPG/PNG/HEIC/AVIF/PSD/RAW/JXL/JXR/SVG/TIF…
lang/{zh-CN,zh-TW,en,ja,ru,template}.lang
```

### 3.2 C++ v14 运行库（用户已安装，随包自带）
```
msvcp140.dll              642,720
msvcp140_atomic_wait.dll   57,936
vcruntime140.dll          178,848
vcruntime140_1.dll        213,072
vcomp140.dll              642,720(原文 213,072 除外) → 642,720
```
系统32目录另有 `msvcp140_1.dll` / `msvcp140_2.dll` / `msvcp140_clr0400.dll` 等。

### 3.3 PE 导出表实测（自写解析器读取，结论可信）
| DLL | 架构 | 导出数 | 导出风格 |
|---|---|---|---|
| `core-ui.dll` | x86-64 (0x8664) | **2006** | C++ mangled 符号为主（`??0ButtonWidget@ui@@...`），**同时存在 `ui_*` 纯 C API** |
| `ghde.dll` | x86-64 | **323** | **纯 C ABI，扁平 `ghde_*` 命名** |
| `guohe-view-thumb.dll` | x86-64 | 4 | 仅 COM：`DllGetClassObject` / `DllRegisterServer` / `DllUnregisterServer` / `DllCanUnloadNow` |

### 3.4 ghde.dll 关键导出（节选，已核实存在）
```
ghde_init / ghde_open / ghde_close / ghde_free
ghde_decode_preview / ghde_decode_preview_with_job
ghde_decode_thumbnail / ghde_decode_thumbnail_ex / ghde_decode_thumbnail_from_reader
ghde_get_info / ghde_get_frame_info / ghde_get_exif_metadata / ghde_get_exif_tags
ghde_get_icc_profile / ghde_get_hdr_info / ghde_get_color_info / ghde_get_level_info
ghde_acquire / ghde_acquire_multiframe / ghde_acquire_ultra_hdr
ghde_job_create / ghde_job_cancel / ghde_job_is_cancelled
ghde_get_tile_rgba8 / ghde_get_tile_fp16_rgb / ghde_get_tile_cmyk8 / ghde_get_tile_cmyka8
ghde_build_bc6h_cache / ghde_cache_is_fresh / ghde_open_cache_file / ghde_default_cache_path_for
ghde_encode_webp_to_buffer / ghde_encode_webp_to_file
ghde_extensions_with_capability / ghde_get_version
```

### 3.5 core-ui.dll 内部已含 Guohe 图片控件（符号实测）
```
GhImgViewWidget          ← Guohe 图片视图控件
ImageViewWidget / ImageViewGDIWidget / ImageViewPlusWidget
```
→ **core-ui 与 GuoheView 同源**，GuoheView 的图片浏览能力已经下沉到 core-ui 控件层。

---

## 4. 由事实推出的三条硬结论

1. **「Rust 为主 + 调用本机 C++ v14 运行库」有落地路径，且不需要 C++ shim。**
   `ghde.dll` 是扁平 C ABI（`ghde_*`），Rust 可用 `libloading` 动态加载 + 手写/生成 FFI 声明直接调用；`core-ui.dll` 官方提供纯 C API（`ui_*`），同样可直接绑定。

2. **「图片预览与 GuoheView 强关联」有两个层次，必须分清：**
   - ① **UI 层**：`core-ui` 的 `GhImgViewWidget` 承担图片显示与缩放交互；
   - ② **解码层**：`ghde.dll` 提供解码/缩略图/EXIF/ICC/HDR/缓存能力。
   若走「进程内动态加载」，二者同进程、无 IPC；若走「外部调 exe」，只能拿到最终画面，控制力弱。

3. **CI 与本机存在天然边界，必须在架构上显式处理：**
   `ghde.dll` / 本机 `core-ui.dll` 属于闭源商业产物，**GitHub Actions runner 上不存在**。因此二者只能**运行时动态加载**，不能在编译期链接；CI 负责产出「不含这两个 DLL 的可执行文件」，用户本机需已安装 GuoheView。

---

## 5. 已识别的冲突与风险（尚未定案，待决策）

| # | 冲突 | 影响 | 候选方向 |
|---|---|---|---|
| C1 | zsdesk 用 **Windows Shell 原生视图**（`IExplorerBrowser`、原生导航树、原生菜单）做文件列表；core-ui 是 **Direct2D 自绘** | 决定「文件列表要不要重写」——全自绘=工作量剧增；保留 Shell=core-ui 只当外壳 | 见决策卡 Q4 |
| C2 | core-ui 需 MSVC/clang-cl 构建；zsdesk 需 Rust MSVC 构建；两者 CRT 选项必须一致 | `/MD` vs `/MT` 不匹配会出 `LNK2038 RuntimeLibrary mismatch` | 两边统一 `/MD`（对齐 Rust MSVC 默认 + 用户已有 v14 运行库） |
| C3 | core-ui 的 `GhImgViewWidget` 是 C++ 内部类，非 C API 导出 | 图片视图能力要通过 `.uix` 标记声明使用，或走 `custom` 控件回调 | 需设计师 + 前端确认调用路径 |
| C4 | 闭源 DLL 再分发问题 | `ghde.dll` 不可随包分发 | 运行时加载 + README 声明「需已安装 GuoheView」 |
| C5 | zsdesk 功能面极大（搜索/Everything/桌面格子/右键菜单宿主/网络版/Listary/腾讯布局迁移） | 全量移植≈重写 ZSDesk | 见决策卡 Q1（范围） |
| C6 | `guohe-view-thumb.dll` 是 32/64 位 COM 缩略图扩展（仅 4 导出） | Explorer 缩略图对接与进程内解码是两条路 | 优先进程内 `ghde_decode_thumbnail` |

---

## 7. 二轮实测补充（源码级与二进制级）

### 7.1 本机 core-ui.dll 与仓库 main 的 API 差异（PE 导出表逐名比对）

| 项 | 数量 |
|---|---|
| 仓库 `include/ui_core.h` 声明 `ui_*` | 557 |
| 本机 `core-ui.dll` 导出 `ui_*` | 567 |
| 双方都有 | 555 |
| 仓库有、本机无 | 2（`ui_core_embed_binary`、`ui_dialog`） |
| 本机有、仓库无 | 12（`ui_image_gdi_*` 9 个、`ui_image_view_gdi`、`ui_image_evict_tile`、`ui_flush_gpu`） |

- 仓库 `src/ui/image_view_gdi.cpp` 存在，但 `include/ui_core.h` 尚未导出对应 API → **本机 DLL 与仓库 main 不是同一 API 面，本机更新**。
- **铁律**：运行时必须用 `ui_core_version()` / `ui_core_version_build()` 做能力协商，不能假设任一 API 必然存在。

### 7.2 core-ui 不提供通用原生 HWND 子控件嵌入（源码级实测）→ 推翻原假设

- 全仓 `src/` 中 `SetParent` 仅出现在 `src/ui/image_view_gdi.cpp`，只用于把 core-ui 自己创建的 GDI 子窗口挂到 core-ui 窗口下；
- `include/ui_core.h:151` 注释明确：child window 会限制在父窗矩形里，"不是这里的语义"。
- **结论：不能把 `IExplorerBrowser`（原生 Shell 列表 HWND）塞进 core-ui 窗口。**
- zsdesk 的「Shell 原生列表 + 原生导航树」在 core-ui 方案下**不可保留**，文件列表必须自绘。
- 可保留的是 Shell 的**语义**：`IShellItemArray` 枚举、`IShellItemImageFactory` 取缩略图位图、`IFileOperation` 执行复制/剪切/粘贴/重命名/冲突处理与撤销 —— 位图经 `ui_bitmap_create` / `ui_image_gdi_*` 喂给 core-ui 自绘列表。

### 7.3 core-ui C API 的字符串与内存约定（实测，直接决定 FFI 契约）

- 头文件中 `const char*` 出现 **72** 次、`const wchar_t*` 出现 **77** 次 → **UTF-8 与 UTF-16 混用且逐函数不同**，不能统一假设；
- 资源/路径类走 UTF-8（`ui_asset_register_dir(const char* dir_utf8)`、`.lang` 规定 UTF-8）；
- 文本内容类多走 UTF-16（`ui_text_input_get_text` → `const wchar_t*`；拖放回调 `const wchar_t* path`）；
- 内存归属两类：返回**内部指针**（不可 free，如 `ui_text_input_get_text`）vs 返回**需显式释放**（如 `ui_debug_toast_info` 需 `ui_debug_free`）；
- 线程模型：`docs/c-api.md` 明确「工作线程访问 widget 前先 marshal 到 UI 线程」→ Rust 侧所有 core-ui 调用必须收敛到 UI 线程。

### 7.4 上游 CI 与规范的可继承物（实测）

**zsdesk** 无 `.editorconfig` / `rustfmt.toml` / `clippy.toml` / `CONTRIBUTING.md`；规范体现在代码本体（中文注释、`//!` crate 文档、`crates/` 分层、edition 2021）与唯一 CI 工作流 `.github/workflows/build.yml`：

- `runs-on: windows-latest`；`dtolnay/rust-toolchain@1.99.0`；
- `cargo test --workspace --release --locked`；
- 多 feature 矩阵逐个 `cargo check --locked --release -p zsdesk-app --no-default-features [--features X]`；
- **依赖固定法**：从 `packaging/zsui-revision.txt` 读 revision，再 `actions/checkout` 精确 ref 到同级目录（`path: zsui`）。
  → 这套「revision 文件 + 精确 checkout」就是「规范沿用」最直接的落点：换成 `packaging/core-ui-revision.txt`。

**core-ui** 无 `.editorconfig` / `.clang-format` / `.clang-tidy` / `CONTRIBUTING.md`；规范在 `CMakeLists.txt` 编译选项（`/utf-8`、`/Zc:__cplusplus`、`/permissive-`、`/EHsc`、C++17、`_CRT_SECURE_NO_WARNINGS`）+ `docs/`（`design-system.md`、`ai-guide.md`、`c-api.md`）；CI 只有 `deploy-pages.yml`，**没有构建工作流** → core-ui 的 CI 构建要我们自己写。
构建入口 `scripts/build-clang-cl.ps1 -Target core-ui`；需 vcvars64 + `llvm-rc` 绕开 Windows SDK `rc.exe` 挂起。

### 7.5 zsdesk 代码分层（替换 ZSUI 的影响面）

| crate | 文件/行数 | 职责 | 受替换影响 |
|---|---|---|---|
| `zsdesk-core` | 8 / 1999 | 领域模型 | 低 |
| `zsdesk-platform` | 2 / 107 | 平台抽象 trait | 低 |
| `zsdesk-ui` | 4 / 1725 | **平台无关设计层**：窗口几何、Fluent 令牌、菜单定义；依赖 ZSUI（feature `windows-win32`） | **高（需重写）** |
| `zsdesk-windows` | 32 / 9205 | Windows/Shell 实现 | 中 |
| `zsdesk-app` | 92 / 29114 | MVU 状态、命令、窗口宿主 | **高** |

合计约 42k 行 Rust。`zsdesk-ui` 只有 4 个文件，是替换 ZSUI 的天然切口。

### 7.6 本机工具链状态（实测，决定验证方式）

本机**无** cargo / rustc / cmake / MSVC / gcc，仅有 git、curl、python3。
→ **本机无法编译验证**；所有编译与测试验证必须走 GitHub Actions。交付物必须自带 CI 验证路径，首版就把 CI 当「编译器」用起来。

### 7.7 GuoheView.exe 命令行与界面形态（实测字符串分析，2026-11-13）

对 `GuoheView.exe` / `GuoheViewCollect.exe` / `pdfdecode.exe` 做 UTF-16LE + ASCII 字符串提取，命中：

**已注册的文件关联**：`\shell\open\command`、`\OpenWithProgids` → 它是注册过的图片打开器。

**实测到的命令行开关**：
```
--register-thumbnails / --unregister-thumbnails
--register-file-assoc   / --unregister-file-assoc
--install-plugin
--server / --auto / --manual        （GuoheView-Updater.exe --manual）
--from-build 0 --to-build %d        （更新器）
```

**关键结论（把 T5 从「未核实」降级为「可落地」）**：
- D02 定了「只外部调 exe」，但 exe 的**「打开某个文件」参数形式未在字符串中出现**；
- 因此最稳的调用是**走系统文件关联**：`ShellExecuteW(hwnd, L"open", <文件路径>, NULL, workdir, SW_SHOWNORMAL)` → 由系统按 `shell\open\command` 拉起 GuoheView；
- 兜底为**显式指定 exe**：`ShellExecuteW(hwnd, L"open", L"\"C:\\Program Files\\GuoheView\\GuoheView.exe\"", L"\"<文件路径>\"", ...)`（lpFile=exe，lpParameters=文件路径）—— 这是图片查看器的通行做法，参数形式几乎无争议；
- 两条路径都要做**「GuoheView 未安装 / 未注册关联」的可检性检查**并给出明确提示（不能静默失败）。
- 资源根解析：exe 以**自身所在目录**为根（字符串 `GuoheView.exe; lang\`、`\resources\icc`、`\plugins\ocr`、`\plugin.zip`）→ 若从别处启动需传工作目录。

**重大旁证：GuoheView 本体就是 core-ui 写的**，exe 内嵌了 core-ui 的 `.uix`/`.ui` 模板源码，命中片段：
```
<window title="GuoheView"
<TitleBar id="titlebar" class="titlebar" :class="titlebarExtraClass" title="GuoheView"/>
<button id="qr_open_url" visible="false">@qr.btn.open_url</button>
<label class="row-label">@settings.menu_open_anim</label>
<toggle id="opt_menu_open_anim" v-model="menuOpenAnim"/>
<custom id="sc_open" class="shortcut-box"/>
```
→ 证实：core-ui 的 `.uix` = Vue 3 SFC（`<script>` + `<template>` + `<style>`），**i18n 用 `@key` 占位 + `.lang` 键值对**，**自定义控件用 `<custom>` + class**，**条件渲染用 `:visible` / `v-if` / `v-model`**。
→ **GuoheView 的 .uix 组织方式就是我们新程序的现成参考样例**（标题栏 / 工具栏 / 设置页 / 缩略图样式页 / 画中画）。

**另注**：exe 内出现 `ghde_iniH`、`ghde_supported_extensions`、`ghde_set_cache_caps`、`ghde_query_motion_photo` 等 → ghde 的 C API 在产品中被直接使用（D02 选外部 exe 后我们不需要这条路径，但可留作未来升级项）。

### 7.8 CI 从源码构建 core-ui 的可行性（脚本级实测，2026-11-13）

`/tmp/upstream/core-ui-main/scripts/build-clang-cl.ps1` 共 **136 行**，实测其硬依赖与行为：

| 依赖 | 获取方式 | CI 影响 |
|---|---|---|
| `vcvars64.bat` | `vswhere.exe` 定位 VS → `VC\Auxiliary\Build\vcvars64.bat`；脚本用 `cmd.exe /c "set ... && vcvars64.bat >nul && set"` 抓环境块 | windows-latest 自带 VS 2022，**可用** |
| `clang-cl.exe` | 优先 `Get-Command clang-cl`（PATH），否则要求 `-ClangClPath` | windows-latest 的 LLVM 不保证在 PATH，**CI 需显式传路径** |
| `llvm-rc.exe` | 取 `clang-cl` 同目录；存在则自动 `-DCMAKE_RC_COMPILER=` 覆盖 | 脚本已内置，**不要关** |
| CMake + Ninja | `-G Ninja`，`CMAKE_BUILD_TYPE=Release` | windows-latest 自带 CMake；Ninja 亦自带 |
| 输出 | `<repo>/clang-cl-build/` | M0 的 CI 步骤照抄即可 |

- 脚本注释原文说明替换原因：Windows SDK `rc.exe`（10.0.26100）在部分 ninja 子进程会**挂起**。
- `-StaticCrt` 开关 = `-DUI_CORE_MSVC_STATIC_CRT=ON`（`/MT`）。**我们不启用**，保持 `/MD`，与 Rust MSVC 默认及用户已有 v14 运行库一致（见 D00-5）。
- 该脚本 `cmake configure` 时固定传 `-DCMAKE_C_COMPILER` / `-DCMAKE_CXX_COMPILER` 指向 clang-cl → **core-ui 在 CI 里是 clang-cl 产物，Rust 侧链接的是 clang-cl 生成的 `.lib`/`.dll`**，两者 ABI 兼容（同 MSVC ABI），但需保证 `/MD` 一致。

**上游精确 revision（2026-11-13 实测，用于 `packaging/core-ui-revision.txt`）**：
```
core-ui  main = 1840d689b51a32d392dbf06b8d594aef96696a80
zsdesk  main = 6aec0c2a82acabdc80251c9ccb0e7637bbab6f4f
```
（沿用 zsdesk 的 `packaging/zsui-revision.txt` 机制，把 ZSUI 的 SHA 换成 core-ui 的 SHA。）

### 7.9 上游可继承的打包/构建资产（实测清单）

**zsdesk 打包脚本 `scripts/package.ps1` 参数**：`WorkDir` / `TargetDir` / `OutputDir` / `RoutingDir` / `RuntimeDir` / `EverythingDir` / `InnoCompiler` / `SkipBuild` / `BinaryPath`。
其中已内置**CRT 自动收集**逻辑：用 `vswhere` 定位 VS → `VC\Redist\MSVC\<ver>\x64` → 取 `Microsoft.VC*.CRT` 目录内的 `msvcp140.dll` / `vcruntime140.dll` / `vcruntime140_1.dll`。
→ **正好对应 D00-5（用户已装 v14 运行库）**：打包时把这三个 CRT 一起装进目标机（或声明目标机需已装 v14 Redist）。可整套沿用。

**其它可继承资产**：
| 资产 | 位置 | 复用性 |
|---|---|---|
| Everything 版本钉死 + SHA256 清单 | `packaging/engines/everything.json` | 直接沿用 |
| 逐 crate 许可证副本 | `packaging/licenses/*.txt`（上百个 crate 的 LICENSE） | 需用 cargo-about 之类重新生成 |
| Listary 插件（C#） | `native/listary/`（含 `Prepare-Listary7.ps1`、`packages.lock.json`） | 与 UI 无关，可直接沿用 |
| 路由转发 DLL（C++） | `native/routing/`（`CMakeLists.txt` + `.cpp` + `.rc` + `build.ps1`） | 与 UI 无关，可直接沿用；**注意这是我们仓库里第一处需要 MSVC 构建的 C++** |
| 换行/二进制属性规则 | `.gitattributes`（`*.rs` lf、`*.ps1` crlf、`*.rc` crlf、`packaging/licenses/** -text`） | 直接沿用 |
| 忽略规则 | `.gitignore`（`/target/`、`/data/`、`/releases/**/*.exe|dll|zip` 等） | 直接沿用 |

**core-ui 侧可继承**：
| 资产 | 位置 | 说明 |
|---|---|---|
| 现成 `.uix` 样例 | `demo/ui_demo.uix`（多页演示）、`demo/ocr_uix_demo.uix` | M1 界面的起点 |
| i18n 样例 | `demo/lang/ui_demo_zh.lang`、`ui_demo_en.lang` | `.lang` = UTF-8 `key=value` |
| Fluent 2 令牌权威文档 | `docs/design-system.md`（色彩/字体/间距/圆角/描边/阴影/动画） | 设计师的令牌来源 |
| 版本单一事实源 | `VERSION` = `1.9.0.345`、`version.json` = `{"version":"1.9.0.345"}` | 与本机 DLL 的 build 345 **同版本号** |

**新发现（修正 7.1 的推断）**：本机 DLL 的 build 号（345）与仓库 `VERSION`/`version.json` **一致**，说明本机 DLL 就是 1.9.0.345；而 `ui_image_gdi_*` 在仓库 `include/ui_core.h` 中查无声明、在 DLL 中却有导出 → 是**头文件未同步导出声明**（导出符号在 `src/` 内），不是版本领先。
→ 但结论不变：**运行时必须按 `ui_core_version_build()` 做能力协商**，且**绑定清单要以「实际 DLL 导出表」为准，头文件与 DLL 允许存在缺口**。

### 7.10 core-ui **没有**虚拟列表 / 表格控件（实测，直接改变 D04 的实现量）

`docs/controls.md` 实测控件全集：
`Label / Button / CheckBox / RadioButton / Toggle / Slider / ProgressBar / TextInput / TextArea / ComboBox / NavItem / Separator / TitleBar / IconButton / TabControl / SVG / Image / ImageView / Dialog / Toast / ContextMenu / CustomWidget`
布局：`VBox / HBox / Grid / Stack / ScrollView / Splitter / SplitView / Panel / Spacer`。

- **不存在** `ListView` / `DataGrid` / 表格 / 虚拟化列表 / 回收器。
- 与「列表」相关的导出仅 `ui_scroll_set_content` / `ui_scroll_view` / `ui_page_set_text_list` / `ui_page_bind_combobox_items`，语义都是「一次性塞全量」。
- `ScrollView` 是**普通滚动容器，非虚拟化**（`docs/layout.md`：`<ScrollView><VBox>…全部子项…</VBox></ScrollView>`）。
- `CustomWidget` 是唯一自定义绘制出口，绘制 API **仅在 draw 回调内可用**。

**推论（已同步架构师）**：
1. 文件列表的虚拟化只能自己做：`<custom>` + Rust 侧持数据与可见区窗口 + draw 回调只画可见条目 + 自写滚动/命中测试/键盘/多选/就地重命名。
2. ZSUI 原本提供的「虚拟列表 + 原生窗口」能力**不是换个控件，而是要在 Rust 侧重写** → `zsdesk-ui` 的迁移代价上调。
3. **draw 回调在 UI 线程** → 回调里禁止解码/读盘；图标与缩略图取位图（`IShellItemImageFactory`）必须放工作线程再回传。
4. 因此 D04 的「全自绘」实为两选项：(a) `<custom>` + Rust 虚拟化；(b) 另起原生侧栏进程跑 `IExplorerBrowser`。需要分阶段取舍。

---

## 7.11 复核修正（2026-11-13 晚，由 PM 与 QA 独立复核发现）

> 本节**修正**前面 §7.1 / §7.9 的表述错误。**冲突时以本节为准。**

### 修正 1：本机 core-ui.dll 是 **1.9.0.346**，不是 345
- PE 版本资源 `FileVersionRaw = 1.9.0.346`；DLL 内嵌 UTF-16LE 字符串 `1.9.0.346`（`1.9.0.345` 不存在）→ 双重确认。
- 上游 `VERSION` / `version.json` / pinned CMakeLists 均为 **345**。
- **影响**：本机 DLL（346）比 pinned revision（345）**新一个 build**；CI 从源码构建的产物是 345。§7.1 与 §7.9 里「与仓库一致」的表述作废。
- **对 G2 门禁无影响**：门禁比对的是「构建产物 ↔ 同一次 checkout 的 `core-ui/VERSION`」，两者同源，必然一致。

### 修正 2：「12 个符号是 DLL 私有扩展」**不成立**
- 对 pinned revision `1840d689` 全 `src/` 做 `UI_API` 扫描得 **567 个 `ui_*`**，与本机 DLL 的 567 个**双向差集为空**。
- 那 12 个（`ui_image_gdi_*` 等）是**上游头文件漏声明、源码里有定义**（`src/ui/ui_api.cpp:1691-1741`），**不是私有扩展**。
- 反向差集只有 **1 个**：`ui_dialog` —— 头文件有声明，但源码自 build 158 已删除。
- **影响**：D08「12 个只探测不绑定」可**放宽为可正常绑定**；能力协商的判据应从「按版本号」升级为「**按实测符号集**」。

### 修正 3：core-ui **没有** Mica / Acrylic / 高对比度 / UIA 入口
- `src/ui/ui_window.cpp` 的 `DwmSetWindowAttribute` 只用于 `TRANSITIONS_FORCEDISABLED` 与圆角偏好；`mica` / `acrylic` / `backdrop` **零命中**。
- `src/ui/theme.h` 的 `HighContrast` / `AppsUseLightTheme` **零命中**。
- `src/ui/widget.cpp` / `ui_api.cpp` 的 `uia` / `accessibility` / `narrator` **零命中**。
- **影响**：迁移矩阵把 `theme.rs` 判「改造」并备注「DWM 效果部分保留」——**该备注不成立**。zsdesk 上游自己用 DWM 实现的 Mica（`theme.rs:131-194`）**在 core-ui 方案下会丢失**，需显式记为已知降级。

### 修正 4：标签统计数字（QA 实测）
原版 `assets/main.uix` 的真实开标签数：`<text>`×**7**、`<text-input>`×**3**、`<hbox>`×7、`<panel>`×4、`<separator>`×4、`<vseparator>`×2、`<label>`×2、`<vbox>`×1。
先前 `docs/11` §2 记的 `<text>`×10 / `<text-input>`×2 是**把闭合标签也计入**所致，已回填。

### 修正 5：`ui_debug_*` 计数
本机 `core-ui.dll` 的 `ui_*` 导出 **567** 个，其中 `ui_debug_*` 为 **72**（先前记为 71）。

---

## 6. 未核实项（不猜，需用户或实测补齐）

- ghde.dll 的**具体 ABI 签名**（本机无头文件，仓库亦未公开）——需实机导出表反推或用户方提供 SDK。
- core-ui.dll 本机版本与仓库 `1.9.0 build 345` 是否同源同版本。
- GuoheView 是否提供公开的开发者文档/SDK。
