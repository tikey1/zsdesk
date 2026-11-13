# 调研简报 R01–R10：ZSDesk × core-ui 改造（zsdesk-coreui）

> 契约编号：`ZC-BRIEF-001` ｜ 日期：2026-11-13 ｜ 作者：许清楚（PM）
> 决策优先级（用户明确）：**本次对话中的选择 > 用户 fork 路线图（`docs/10-FORK-ROADMAP-COREUI.md`）> 交付总监建议**。
> 证据基线：`research/facts.md`（10 组实测）、`docs/05-COREUI-EXPORTS.txt`、`docs/05-COREUI-ABI-NOTES.md`、
> `docs/11-COREUI-MARKUP-TAGS.md`、`docs/12-UIX-TAG-FIX.md`、`DECISIONS.md`（D00–D11）、
> `SPEC.md`、`spec/`（五份契约）、`spec/migration-matrix.md`（逐文件矩阵）。
> 标签约定：`[FACT]` 已实测复核 ｜ `[INFER]` 基于实测的推断 ｜ `[MISSING]` 查不到、需真机/用户补。

---

## 研究问题

- RQ1：全量移植 zsdesk 到 core-ui 的真实工作量落在哪里，哪些能力保不住？
- RQ2：GuoheView 只做外部 exe 入口后，用户实际会失去什么、还能得到什么？
- RQ3：这套「Rust 主导 + core-ui 自绘 + 闭源 DLL 外部依赖」的组合，哪几条最早会翻车？

---

## 结论

1. `[FACT]` **工作量不在「移植」而在「呈现层重写」**：138 个 `.rs` 文件 / 42,150 行中，直接复用 68 文件 / 15,671 行（37.2%），
   需改造 56 文件 / 20,193 行（47.9%），重写 10 文件 / 5,600 行（13.3%），删除 4 文件 / 686 行（1.6%）
   （`spec/migration-matrix.md` §2，`spec/tools/matrix_calc.py` 可复算）。
2. `[FACT]` **四类上游能力在 core-ui 下整体落空**（D04 / D06 / D07 已拍板）：Shell 原生文件列表、原生导航树、
   原生预览宿主、ZSUI 虚拟列表 —— 因为 core-ui 不支持原生 HWND 子控件嵌入，也无虚拟列表控件
   （`research/facts.md` §7.2 / §7.10）。
3. `[FACT]` **GuoheView 走外部 exe 后，「进程内能预览哪些格式」被 WIC 的解码能力划界**：
   本机注册表实测 `GuoheView.*` ProgID 共 **66 个**（JPG/PNG/WEBP/AVIF/HEIC/JXL/JXR/PSD/RAW/SVG/TIFF…），
   `\shell\open\command` = `"…\GuoheView.exe" "%1"`，
   `.jpg/.png/.webp/.avif/.heic/.jxl/.svg` 的默认处理器**就是** GuoheView（`UserChoice` 实测）。
   → 其中 **WIC 能解的**（JPG/PNG/BMP/TIFF 等）仍可进程内预览；**WIC 解不了的**（HEIC/AVIF/PSD/RAW/JXL/JXR）
   按 D06 只能走外部 exe。`[MISSING]` WIC 在 Win10 19045 上实际支持哪些扩展名未逐一实测。
4. `[FACT]` **本机 core-ui.dll 的真实版本是 1.9.0.346，不是 345**。PE 版本资源 + DLL 字节内嵌字符串双重确认
   （`1.9.0.346` UTF-16LE 存在；`1.9.0.345` 不存在）；上游 `VERSION`/`version.json`/pinned CMakeLists 均为 `345`。
   → **既有底稿 `research/facts.md` §7.9 与 `docs/05-COREUI-ABI-NOTES.md` 的「本机 DLL = 345，与仓库一致」这条结论是错的。**
5. `[FACT]` **「12 个符号是 DLL 私有扩展」这条结论也是错的**：对 pinned revision `1840d689…` 的全部
   `src/` 源文件做 `UI_API` 定义扫描，得 567 个 `ui_*`，与本机 DLL 的 567 个**逐名完全一致（双向差集为空）**。
   那 12 个符号是**上游头文件漏声明**，源码里有定义（`src/ui/ui_api.cpp`）。反向差集只有 1 个：
   `ui_dialog` 在头文件里声明、但上游源码自 build 158 起已删除（`ui_api.cpp:582` 注释「MsgBox 取代 in-window ui_dialog_*」）。
   → **D08 里「12 个符号签名未知 → 只探测不绑定」的处置可以放宽为「可正常绑定」**（签名需从 pinned 头文件取）。
6. `[FACT]` **core-ui 上游没有 Mica/Acrylic、没有高对比度、没有 UI Automation**：
   `src/ui/ui_window.cpp` 里 `DwmSetWindowAttribute` 只用于 `DWMWA_TRANSITIONS_FORCEDISABLED` 与圆角偏好，
   `mica|acrylic|backdrop` 零命中；`theme.h` 里 `HighContrast` / `AppsUseLightTheme` 零命中；
   `widget.cpp` / `ui_api.cpp` 里 `uia|accessibility|narrator` 零命中。
   → **zsdesk 上游自己用 DWM 实现了 Mica**（`zsdesk-windows/src/theme.rs:131-194`，`DWMWA_SYSTEMBACKDROP_TYPE`）
   —— 迁移矩阵把 `theme.rs` 判为「改造」，但 **core-ui 不提供 Mica 入口 → Win11 22H2+ 的 Mica 能力会丢**。
7. `[MISSING]` **U01~U12 的参数语义、M01~M06 的标记渲染行为全部未在真机验证过**（本机无 Rust/C++ 工具链）。
8. `[MISSING]` **core-ui 从源码构建的耗时、产物体积、clang-cl 路径在本机的实际值**均未测过（CI 侧首次绿灯 9m58s 是
   `Windows build #1`，那是**未接 core-ui** 的基线）。

---

## 竞品与替代方案对照

| 方案 | 文件列表 | 图片预览 | 呈现层工作量 | 闭源依赖 | 本项目判定 |
|---|---|---|---|---|---|
| A. 保留 ZSUI + Shell 原生视图（上游现状） | Shell `IExplorerBrowser` 原生 | 上游 WIC + 外部进程 | 0 | 无 | 不满足 D00-2/D04 |
| B. core-ui 全自绘 + WIC 解码（本项目选定） | `<custom>` + Rust 虚拟化 | 进程内 WIC；HEIC 等外部 exe | 5,600 行重写 + 5,900–10,600 行新增 [估算] | 运行时不依赖闭源 DLL | **D01/D04/D06 已定** |
| C. core-ui 只做外壳 + 保留原生视图 | 不可行 | — | — | — | 源码级证伪（无子控件嵌入） |
| D. 真接 WinUI 3 / Windows App SDK | XAML | 同 B | 更高 | 引入 App SDK 运行时 | 与 D00-4「不引 WinUI 3 运行时」冲突 |

---

## 反证检索

针对「core-ui 自绘 + 迁移矩阵四档」这一方案，检索了三条可能的推翻路径：

1. **「core-ui 其实能嵌原生 HWND，那就不用重写文件列表」** —— 证伪成立。全仓 `src/` 中 `SetParent` 仅出现在
   `src/ui/image_view_gdi.cpp`（挂自己的 GDI 子窗口）；`include/ui_core.h:151` 注释明写 child window 语义不适用。
   C 方案（保留原生列表）不可行。
2. **「12 个 `ui_image_gdi_*` 是 GuoheView 私有扩展，CI 自建的 core-ui 不会有」** —— 证伪成立。
   pinned revision 的 `src/ui/ui_api.cpp:1691-1741` 逐个定义了这些符号，**上游源码就有**，不是私有扩展。
   → 该结论推翻后，D06「图片预览留在 core-ui 内」在 CI 产物上成立，**且 `ui_image_view`/`ui_image_set_pixels`
   本就在 567 个交集里**（DECISIONS.md D06 第二轮已独立复核，结论一致）。
3. **「本机 core-ui.dll 是 1.9.0.345，与上游一致，所以能力协商只需认 345」** —— **部分推翻**。
   版本资源显示本机是 **346**（详见结论 4）。但 ABI 层面反而更简单：**本机 DLL 与 pinned 源码的 `ui_*` 集合逐名一致**，
   → **能力协商应以「符号存在性」为准，而不是以版本号为准**（与 D08 已有口径一致，此处只是把依据从「版本」升级为「实测符号集」）。

---

## 来源

| # | 类型 | 链接/位置 | 抓取日期 |
|---|---|---|---|
| 1 | 一手 | 本机 `C:\Program Files\GuoheView\core-ui.dll` PE 版本资源（`FileVersionRaw=1.9.0.346`） | 2026-11-13 |
| 2 | 一手 | 同上，DLL 字节内嵌 UTF-16LE 字符串 `1.9.0.346`（`1.9.0.345` 不存在） | 2026-11-13 |
| 3 | 一手 | 上游 `ghboke/core-ui@1840d689…`：`VERSION`=1.9.0.345、`version.json`、`CMakeLists.txt:3-4` | 2026-11-13 |
| 4 | 一手 | 上游 pinned revision 全 `src/` 源文件 `UI_API` 扫描（547+20 = 567 个 `ui_*`） | 2026-11-13 |
| 5 | 一手 | 本机注册表 `HKCU\Software\Classes\GuoheView.*`（66 个 ProgID）+ `shell\open\command` + `FileExts` `UserChoice` | 2026-11-13 |
| 6 | 一手 | 本机 `zsdesk-windows/src/theme.rs:131-194`（DWM Mica/Acrylic）、`zsdesk-ui/src/theme.rs:208-210`（高对比度走 ZSUI） | 2026-11-13 |
| 7 | 一手 | 上游 `src/ui/ui_window.cpp`（`DwmSetWindowAttribute` 仅 2 类用途）、`src/ui/theme.h`、`src/ui/widget.cpp` | 2026-11-13 |
| 8 | 一手 | 本仓 `spec/tools/matrix_calc.py` 复算 138 文件 / 42,150 行四档 | 2026-11-13 |
| 9 | 二手 | 本仓 `docs/05-COREUI-ABI-NOTES.md`（18 VERIFIED / 12 UNVERIFIED）、`docs/11-COREUI-MARKUP-TAGS.md` | 2026-11-13 |
| 10 | 二手 | 本仓 `spec/migration-matrix.md`、`spec/ci-contract.md`、`spec/milestones.md`、`SPEC.md` | 2026-11-13 |
| 11 | 二手 | 用户 fork `docs/10-FORK-ROADMAP-COREUI.md`（C1–C5 与 P1–P4） | 2026-11-13 |
| 12 | 行为 | 本机 `Uninstall` 项 `GuoheView 3.4.18.127`，`InstallLocation=C:\Program Files\GuoheView` | 2026-11-13 |

---

## R01 · 功能范围与「全量移植」的工作量切分

**结论**：D01「全量移植」的真实新增成本 = **5,600 行重写（上游既有代码）+ 5,900–10,600 行新增（上游没有的代码）**
≈ **1.2–1.6 万行** [估算]。可直接复用的只有 37.2%。

**依据**：
- `spec/migration-matrix.md` §2（138 文件 / 42,150 行 → 复用 68/15,671、改造 56/20,193、重写 10/5,600、删除 4/686）。
- 重写集中在 4 个点：`zsdesk-ui/src/chrome.rs`(788)、`zsdesk-app/src/window/mod.rs`(1,511)、
  `zsdesk-windows/src/shell/namespace_tree.rs`(1,029)、`zsdesk-app/src/window/paint.rs`(534)。
- 新增集中在：`zsdesk-coreui` FFI 层 1,200–1,800、自绘文件列表 2,000–4,000、自绘导航树 600–1,000、
  `.uix` 页面 800–1,500、预览窗格 500–900、Shell 缩略图/属性 300–500、状态投影胶水 500–900（§6，均标 [估算]）。
- **D11 变更记录里的关键提示**：`改造` 档（47.9%）的成本常被低估 —— 一处 `use` 路径 + 一处 API 语义变更，
  工作量可能是原文件的 30%–60%。

**「全量移植 ≈ 重写 ZSDesk」这个判断，成立但需精确化**：不是 42k 行全部重写，而是
**呈现层 ≈ 13.3% 重写 + 47.9% 改造 + 上游没有的 1.2–1.6 万行新增**。
换算下来，**约 1/3 的上游代码要动、另加一个同等体量的新子系统** —— 这才是「接近重写」的准确含义。

**这决定后续 Phase 做什么**：M0~M3 **不允许并行**（本机无工具链，只能靠 CI 验证；一次合并多个阶段的失败无法二分定位）。
`desktop/host.rs`（2,681 行）**M3 才碰**，M0–M2 不动它。

---

## R02 · 交付形态与验收标准

**结论**：唯一交付形态 = **`full` edition + `windows-x64` 便携 ZIP**，逐文件 `bytes + SHA-256` manifest，
强制校验 VC++ x64 Redistributable。**但「≤10 MB」这条验收线在新依赖下已不成立**。

**依据**：
- D05：features = `network + desktop`；不构建/不测试/不发布其余 5 个 edition；任何 32 位 x86 一律不做。
- `scripts/package.ps1` 已内置：CRT 自动收集（`vswhere` → `VC\Redist\MSVC\<ver>\x64` → `Microsoft.VC*.CRT`）、
  逐文件 `bytes+SHA256` manifest、`Assert-Payload` 复校、`Compress-Archive` 出 ZIP、**已存在则拒绝覆盖**。
- **体积实测**（本仓当前文件 + 钉住版本的固定文件）：

  | 组成 | 字节 | MiB |
  |---|---:|---:|
  | `core-ui.dll`（本机 1.9.0.346） | 4,782,536 | 4.56 |
  | Everything 三件（`everything.json` 钉 1.4.1.1032） | 3,308,230 | 3.15 |
  | `packaging/licenses/`（240 个 crate 许可副本） | 1,213,417 | 1.16 |
  | CRT ×3（msvcp140 / vcruntime140 / vcruntime140_1） | 871,824 | 0.83 |
  | **小计（不含 ZSDesk.exe 与 routing/）** | **10,176,007** | **9.70** |

  → **`docs/07-UI-DESIGN-WINUI3.md` §10「发布包仍 ≤10MB」在加入 core-ui.dll 后已不可能满足**，
  且这还没算 exe 本体和 `native/routing`（ShellRoutingControl.exe + ZSDeskRevealRouter.dll）。
  `[MISSING]` CI 自建 core-ui.dll 的实际体积未测（4.56 MiB 是本机那份的数，不是 CI 产物的数）。

- 其它固定项：`packaging/engines/everything.json` 已钉版本 + SHA-256 + `manifest.json` 哈希，
  打包时逐文件复校 → **Everything 可原样沿用，无需改动**。

**这要求用户拍板什么**：发布包体积红线 —— ① 维持 10 MB → 必须砍掉 Everything 随包（改为「检测到本机 Everything 才启用搜索」），
或砍掉 240 个 crate 许可副本（改为单文件 NOTICE + 指向 Cargo.lock）；② 放宽到 ~20 MB → 现状可接受。
**这是本简报里唯一一条「既有验收标准已经被新依赖打破」的问题，需要明确裁定。**

---

## R03 · GuoheView 外部调用的 UX 契约

**结论**：D02 的「只外部调 exe」在**调用形式**上已完全可落地（注册表实测证明），但**触发时机、失败提示、降级路径**
尚未定义成契约。这三条必须写死，否则用户遇到「点了没反应」时无法自处。

**依据**：
- **调用形式已实测**：`HKCU\Software\Classes\GuoheView.<EXT>\shell\open\command` = `"C:\Program Files\GuoheView\GuoheView.exe" "%1"`。
  → **D02 里「exe 的打开文件参数形式未在字符串中出现」这条未核实项，现已由注册表坐实**：`%1` = 文件路径，标准参数约定。
- **覆盖率实测**：注册了 **66 个 ProgID**（3FR…X3F，含 JPG/PNG/WEBP/AVIF/HEIC/JXL/JXR/PSD/RAW/SVG/TIFF…），
  **不含 PDF / DOCX / XLSX** → 非图片类预览的落点（D06）与 GuoheView 无关。
- **默认处理器实测**：`.jpg / .png / .webp / .avif / .heic / .jxl / .svg` 的 `UserChoice` **当前就是** `GuoheView.*`
  → 「走系统关联」这条路径在本机**会真的打开 GuoheView**，不是落到别的看图器。
- **可检性**：`Uninstall` 项 `GuoheView 3.4.18.127` + `InstallLocation=C:\Program Files\GuoheView` → 装没装可查。
- **未覆盖**：`.jpg` 在 `HKLM\SOFTWARE\Classes` 下的默认值仍是 `jpegfile`，只是 `UserChoice` 覆盖了它
  → 「系统关联优先」这条路径**依赖用户当前选择**，不是稳定契约 → **必须保留显式 exe 兜底**。

**这决定后续 Phase 做什么**（M2 的契约草案，需实现侧与设计师共同确认）：

| 场景 | 触发 | 用户看到 | 降级 |
|---|---|---|---|
| 图片已装 GuoheView | 用户在列表里对图片触发「用 GuoheView 打开」 | GuoheView 独立窗口打开该图 | 无需降级 |
| 图片未装 GuoheView | 同上 | **明确提示「未检测到 GuoheView」+ 给出安装位置指引**，不静默失败 | 提示 + 可改用系统默认看图器 |
| 非图片（Office/WPS/PDF） | 同上 | 按 D06 走「用系统程序打开」 | — |

**硬约束**：调用**必须参数数组化**（`ShellExecuteW` 的 `lpFile` / `lpParameters` 分开传，不拼字符串），
路径先规范化、日志脱敏（`spec/milestones.md` M2 已列 E02/E16）；**不得阻塞 UI 线程**。

**这要求用户拍板什么**：GuoheView 触发的入口放在哪 —— 命令栏独立按钮 / 右键菜单项 / 双击图片直接交给它？
**双击交给它会改变「双击=在 ZSDesk 内打开」的既有习惯**，建议明确排除。

---

## R04 · 非图片类预览（Office / WPS / 文本 / PDF）的用户预期与验收口径

**结论**：D06 已定「Office/WPS/PDF 降级为『用系统程序打开』」，**但用户预期与验收口径还没定义**，
且该决策**删除了上游 419 行已能用的能力**（`wps_preview.rs` 269 + `wps_render_process.rs` 150）。

**依据**：
- D06 处置：`wps_preview.rs` + `wps_render_process.rs` = **删除**（迁移矩阵 §5 的 4 个删除文件之二，686 行删除量中占 419）。
- 上游 README 记载的能力现状：Office/WPS **进程内预览**是可用的（`window/wps_preview.rs` 走 WPS 渲染子进程），
  `preview_process.rs`(351) / `quicklook.rs`(108) / `quick_preview.rs`(791) / `hover_preview.rs`(374) 走宿主窗口或子进程。
- **能力落差**：进程内 EXIF 面板、缩略图流式加载、多图胶片卷 **均不复刻**（D06 代价 3）。
- 文本预览**保留在 core-ui 内**（Rust 读前 N KB → `.uix` 的 `TextArea`）。
- **「哪些格式进程内能看」的边界 = WIC 的解码能力**（D06 走 WIC）。WIC 能解的（常见位图）进程内看；
  WIC 解不了的（HEIC/AVIF/PSD/RAW/JXL/JXR 等）按 D06 走 GuoheView 外部 exe。
  `[MISSING]` WIC 在 Win10 19045 上实际支持的扩展名清单未逐一实测 → M1 首屏渲染前需列一次实测表。

**这决定后续 Phase 做什么**：M2 必须把「预览能力矩阵」写成用户可见的表，而不是一句「外部打开」。
建议验收口径按「类型 × 落点 × 失败提示」三列固化：

| 文件类型 | 落点 | 失败提示 | 验收 |
|---|---|---|---|
| 图片（JPG/PNG/WEBP…） | 进程内 WIC → `ui_image_set_pixels` | 解码失败：显示占位 + 文件名 + 「用 GuoheView 打开」 | 预览窗格显示像素，非占位 |
| HEIC/AVIF/PSD/RAW/JXL | **外部 exe**（GuoheView 支持） | 未装时给安装指引 | 双击能开 |
| 纯文本 / 代码 / 源码 | 进程内（TextArea） | 编码识别失败：明示「按 GBK 解码失败」 | 打开显示前 N KB，中文不乱码 |
| Office / WPS / PDF | **外部系统程序** | 未装对应程序：明示「未检测到可打开此格式的程序」 | 系统程序能打开 |

**这要求用户拍板什么**：文本预览的**行数/字节上限**与**编码探测**策略（UTF-8 优先、GBK 兜底、还是只按 UTF-8 读并标「可能乱码」）。

---

## R05 · 界面风格验收点（Win10 / WinUI 3 / Fluent）

**结论**：core-ui 自绘的 Fluent 2 **能覆盖「令牌、圆角、阴影、动效」这四层，但覆盖不了「材质」与「无障碍」两层**。
且**上游 zsdesk 自己实现的 Mica 会在迁移中丢失**。因此「界面风格符合用户习惯」这条目标，
**验收口径必须从「看起来像 WinUI 3」改成「令牌与布局像 WinUI 3，且在 Win10 上完整可用」**。

**依据**：
- **core-ui 没有 Mica / Acrylic**：上游 `src/ui/ui_window.cpp` 中 `DwmSetWindowAttribute` 只用于
  `DWMWA_TRANSITIONS_FORCEDISABLED`（3 处）与 `kDwmWindowCornerPreference`（1 处）；
  `mica|acrylic|backdrop` 在该文件**零命中**；`SetWindowCompositionAttribute` 零命中。
  core-ui 只自绘圆角（DWM 圆角偏好）与阴影（`D2D1GaussianBlur`）。
- **zsdesk 上游有 Mica**：`zsdesk-windows/src/theme.rs:131-194` 用 `DWMWA_SYSTEMBACKDROP_TYPE`
  设 `Solid=1 / System|Mica=2 / Acrylic=3`，并有 `set_blur_behind`（Win10 accent 策略）。
  迁移矩阵把该文件判为「改造」且备注「DWM 效果部分保留」—— **但 core-ui 不提供 backdrop 入口**，
  自绘窗口也无法让出背景 → **Win11 22H2+ 的 Mica 与 Win10 的 accent 模糊，实测上会落空**。
  备注：`docs/07-UI-DESIGN-WINUI3.md` §4 本就写明「Win11 22H2+ 可选 Mica，Win10 一律纯色回退；不依赖 Acrylic/模糊」
  —— **设计文档的预期与实现能力方向一致**，只是要明确写成验收项。
- **没有高对比度**：core-ui `theme.h` 中 `HighContrast` / `AppsUseLightTheme` / `Personalize` **零命中**；
  上游 zsdesk 的高对比度走 `zsui::PlatformPalette::current(mode)`（`zsdesk-ui/src/theme.rs:208-210`）→ **随 ZSUI 一起消失**。
  设计文档 §7 要求「高对比度模式：忽略 --accent 与材质，全部改用系统色」→ **这一条当前无法实现**。
- **没有系统主题跟随**：core-ui 只有 `UI_THEME_DARK=0` / `UI_THEME_LIGHT=1` 二值枚举，**无 System 档**；
  设计文档 §7 要求「默认跟随系统主题」→ 需 Rust 侧读注册表 `AppsUseLightTheme` 后自行决定，**能力存在但需自己实现**。
- **没有 UI Automation**：`widget.cpp` / `ui_api.cpp` / CHANGELOG 中 `uia|accessibility|narrator` **零命中**
  → 设计文档 §5 的「为每个控件提供 narrator 名称与状态」**当前无法实现**。
- **可用的部分**：core-ui 令牌严格对齐 Fluent 2（`docs/design-system.md` 基于 `@fluentui/tokens`，控件对齐 WinUI 3 `microsoft-ui-xaml` 源码）；
  有 `ui_window_dpi` / `ui_window_dpi_scale`（DIP 语义，1/96 英寸）；有 `ui_custom_set_focusable` + `ui_widget_on_focus/blur` + `ui_window_on_key`
  → **键盘可达性可做**；有 `ui_theme_set_cjk_font`（CJK 字体可设）。

**可验证的验收条目**（建议写进 `spec/milestones.md` M1）：

| # | 验收项 | 判定方式 | 当前可达成 |
|---|---|---|---|
| A1 | 1200×800 @100% 与 150% DPI 下无截断/无重叠、1 px 线可见 | 双 DPI 截图逐项比对 | 可（core-ui 用 DIP） |
| A2 | 浅色 / 深色两套均可读、焦点环清晰 | 切 `ui_theme_set_mode` 前后截图 | 可（D10 已定库接管） |
| A3 | **高对比度模式** | — | **当前不可**（无 API，需自建或砍需求） |
| A4 | **跟随系统主题** | 注册表 `AppsUseLightTheme` 切换后重启 | 需 Rust 侧实现（能力在，入口缺） |
| A5 | **Mica / 背景模糊** | Win11 22H2+ 截图看窗口底 | **当前不可**（core-ui 无 backdrop 入口） |
| A6 | 键盘可达：Tab 顺序 标题栏→标签→命令栏→地址→搜索→导航→列表→预览→状态栏 | 逐控件 Tab 实录 | 可（core-ui 有 focus/blur/key 回调） |
| A7 | **屏幕阅读器（narrator）可读** | — | **当前不可**（无 UIA） |
| A8 | 首版只读：无写操作入口，误触有明确提示 | 界面走查 | 可 |

**这要求用户拍板什么**：A3（高对比度）/ A5（Mica）/ A7（narrator）三条**当前无法实现** ——
是**接受降级**（写进 README 与验收清单），还是**为它们单独立项**（自建高对比度调色板 / 向 core-ui 上游提 backdrop 与 UIA 需求）？
**这三项直接决定「符合用户习惯」这句话能不能兑现**，必须由用户裁定，不能由实现侧默认放弃。

---

## R06 · 规范与审查机制

**结论**：D00-6「沿用两个上游库的规范」在**代码层可继承**、在**配置层几乎为零** ——
两个上游库**都没有** lint / format / 贡献指南配置文件。可继承的规范要靠**从代码本体反推 + 把「可机器判定的部分」写成新仓库的配置文件**。

**依据**（实测）：

| 项 | zsdesk | core-ui | 落点 |
|---|---|---|---|
| `.editorconfig` / `rustfmt.toml` / `clippy.toml` | **无** | **无**（`.clang-format` / `.clang-tidy` 亦无） | 新仓库需新建 |
| 贡献指南 | 无 `CONTRIBUTING.md` | 无 | 新仓库需新建 |
| 代码风格载体 | 中文注释 + `//!` crate 文档 + `crates/` 分层 + edition 2021 | `/utf-8` `/Zc:__cplusplus` `/permissive-` `/EHsc` C++17 + `docs/` | 编译选项进 `CMakeLists.txt`，注释规范进规范文件 |
| 唯一 CI 工作流 | `.github/workflows/build.yml`（`dtolnay/rust-toolchain@1.99.0` / `cargo test --workspace --release --locked` / 多 edition `cargo check` 矩阵） | **无构建工作流**（只有 `deploy-pages.yml` 部署文档站） | **core-ui 的 CI 构建要我们自己写**（D03） |
| 依赖固定法 | `packaging/zsui-revision.txt` → 精确 ref checkout 到同级目录 | — | 换成 `packaging/core-ui-revision.txt`（D03 已定，`1840d689…`） |
| 换行/二进制规则 | `.gitattributes`（`*.rs` lf、`*.ps1` crlf、`*.rc` crlf、`packaging/licenses/** -text`） | — | 直接沿用 |
| 用户既有审查机制 | Gate 0~4 / R1~R12 / 三遍法 / DoD / **作者不得自审** / 意见带 🔴🟡💭 + 文件:行号 + 原因 + 建议 / 无工具链时 `cargo_run=false` 必须如实标注 | — | **不打折沿用**（用户 fork 路线图 §5） |

**新仓库该写哪几份规范文件**（建议，与 `spec/ci-contract.md` 的门禁 S1–S13 对齐）：

| 文件 | 内容来源 | CI 检查 |
|---|---|---|
| `CONTRIBUTING.md` | 上游路线图 §5 的 dev-discipline + Gate 0~4/R1~R12 | PR 模板必填「自检清单」 |
| `docs/CODE-REVIEW-STD.md` | 参照同族项目已有的 `06-CODE-REVIEW-STD.md` 体例（Gate/DoD/意见格式/三遍法） | — |
| `rustfmt.toml` + `clippy.toml` | **新建**（上游无 → 用 rustfmt 默认 + `-D warnings`） | `cargo fmt --check` / `cargo clippy --all-targets -- -D warnings` |
| `docs/CCXX-STYLE.md` | core-ui 的 `/utf-8` `/permissive-` + 命名风格 + 注释语言 | 编译期保证，无需额外门禁 |
| `docs/UIX-STYLE.md` | `.uix` 的 D-15 约束（只声明布局/样式/token，无事件属性、无业务脚本） | 门禁 S13（已存在） |

**这决定后续 Phase 做什么**：M0 必须落这 5 份文件 + 2 条 lint 门禁，否则「沿用既有规范」只停留在口号。

---

## R07 · 风险与早期探测手段（按概率 × 影响排序）

**结论**：Top 3 风险与 `SPEC.md` §7 一致，但**本简报新增 2 条**（R07-A / R07-B），并给全部 8 条配**可执行的早期探测**。

| # | 风险 | 概率 | 影响 | 早期探测手段（可执行） | 探测时机 |
|---|---|---|---|---|---|
| **1** | 自绘文件列表交互细节（虚拟滚动/命中/框选/反向选择/就地重命名）—— 全是新代码，上游 `IExplorerBrowser` 白拿的 | 高 | 高 | M1 第 1 周只做「滚动 + 画行 + 单击选中」；draw 回调里断言「画出行数 ≤ 可见行数 + 4」 | M1 W1 |
| **2** | 声明式右键菜单时序（`ui_page_set_json` 推数据 vs 菜单弹出先后无文档承诺） | 中高 | 中高 | 最小探针：`set_json` → 立即 `ui_menu_show` → `ui_debug_screenshot_menu` 截图，核对菜单项是否已是最新值 | M1 W2 |
| **3** | OLE 拖出（`DoDragDrop` 只在 core-ui 注释里被提及，非承诺） | 中高 | 中 | 最小探针：`<custom>` 内按住鼠标拖出窗口，看是否返回 `DRAGDROP_S_DROP`，观察是否崩溃/卡住 | M1 W2 |
| **R07-A** | **高对比度 / Mica / narrator 三项无法实现**（R05 A3/A5/A7）→ 与「符合用户习惯」的目标直接冲突，且已到验收期才发现 | **高** | **高** | 立刻做**不需要编译**的判定：查上游 `src/ui/theme.h` 是否有 `HighContrast`、`ui_window.cpp` 是否有 backdrop 入口、`widget.cpp` 是否有 UIA —— **本轮已查完，三项均无**（R05 依据）→ **现在就能定，不用等 M1** | **本轮（已闭环）** |
| **R07-B** | **core-ui 自建产物与本机那份能力面不一致** —— 若只按版本号协商（345 vs 346）会误判 | 中 | 中高 | 用 `spec/tools` 的导出表比对脚本，对 **CI 产物** 跑一次「导出集 ⊆ 钉住清单」校验；**判定依据改为符号存在性，不改版本号** | M0 首次 core-ui 绿灯后 |
| 4 | 12 个 U 项语义未核实（U01~U12） | 中 | 高 | 按 ABI-NOTES 顺序做最小闭环：`ui_core_version*` → `ui_init_with_theme` → `ui_page_load_string` → `ui_page_open_window` → `ui_run` | M0 |
| 5 | 深浅色切换突变（token 撞名） | 中 | 中 | M1 的 V1.9：切一次主题，截图对比前后两帧 | M1 |
| 6 | GuoheView 调用形式（**已由注册表实测解决**） | 低 | 中 | `HKCU\Software\Classes\GuoheView.<EXT>\shell\open\command` 读 `%1` → 已确认为标准参数；真机只需验一次「打开的文件是不是目标文件」 | M2 |
| 7 | `desktop/host.rs`（2,681 行）单文件体量 | 中 | 中 | M3 才碰；M0–M2 不动它 | M3 |

**这决定后续 Phase 做什么**：R07-A **不排进里程碑，等就晚了** —— 它是一条**需求裁剪决策**，必须由用户在 M0 之前拍板（见 R05 结尾）。

---

## R08 · 里程碑验收标准（P1~P4 各阶段「做到什么算完成」）

**结论**：M0~M3 ↔ P1~P4 的映射已由 D11 定下，但**每阶段的「完成」判据必须写成可执行命令**，
且 M0 有一个**已知红项**（打包未含 `assets/` → 交付的 exe 启动即失败，DECISIONS.md 已记为 🔴 缺口）。

**依据**（D11 + `spec/milestones.md` + `spec/ci-contract.md`）：

| 里程碑 | ↔ 路线图 | 主题 | 「做到什么算完成」的判据 |
|---|---|---|---|
| **M0** | P1 | 仓库骨架 + CI 绿 + 门禁 + 绑定骨架 | ① Actions 跑出 **full x64** 绿灯；② `cargo test --workspace --release --locked` 绿**且不碰 core-ui.dll**；③ 门禁 S7~S13 全绿；④ **打包含 `assets/`（`.uix` + `themes.json`）** ← 🔴 当前缺口；⑤ 5 份规范文件 + 2 条 lint 门禁落地（R06）；⑥ **core-ui 自建成功**且「导出集 ⊆ 钉住清单」校验通过（R07-B） |
| **M1** | P2 | 窗口与文件浏览 | ① 10,000 文件目录首屏 ≤ 120 ms、滚动 P95 ≤ 8 ms；② 右键菜单时序实测通过；③ OLE 拖出探针通过；④ 深浅色切换不突变；⑤ **R05 的 A1/A2/A6 三条通过** |
| **M2** | P3 | GuoheView 外部调用 + 降级 + 错误提示 | ① GuoheView 能打开指定文件；② 未安装 / 未注册关联**各有可区分提示与替代动作**；③ 调用参数数组化；④ 不阻塞 UI 线程；⑤ **R04 预览能力矩阵落地** |
| **M3** | P4 | 其余功能迁移 + 完整回归 + 发布 | ① 搜索 / 桌面格子 / 右键菜单宿主 / 网络版 / Listary / 腾讯迁移**全通**；② `zsui` 依赖移除；③ 便携包 + 逐文件 SHA-256 manifest；④ **R02 体积红线裁定后达成** |

**这要求用户拍板什么**：M2 的验收口径已写死，但 **M0 的 ④ 是已知红项** —— 是否允许「M0 交付一个启动即失败的 exe」？
建议**不允许**：M0 的判据应是「exe 能启动并显示一个空窗口」，不是「CI 绿」。

---

## R09 · 回归范围（功能全量保留下必须覆盖的业务链路清单）

**结论**：D01 全量保留功能 → 回归面 = **20 个上游回归/契约测试文件 + 10 条业务链路**。
其中 **20 个测试文件全部无 `cfg(feature)` 门控** → **D05 收窄到单 edition 不会漏跑测试**，这是好消息。

**依据**：
- 上游回归测试文件 **20 个**（实测）：`context_menu_process_tests` / `desktop/{context_menu,layer,window_message}_regression` /
  `fast_search/runtime_regression` / `window/{context_menu,everything,geometry,integration_navigation,interaction,preview_lifecycle,quicklook_process,search,selection,shell_reveal,thumbnail}_regression` /
  `shell/{file_transfer,icons_alpha,menu_runtime,shell_item_contract}_tests`。
  **逐文件查 `cfg(feature` 计数全为 0** → 收窄 edition 后不会因 feature 门控而漏测。
- 业务链路清单（按 D01 全量保留，覆盖上游 README 的功能域）：

| # | 链路 | 主要落点 | 回归来源 |
|---|---|---|---|
| L1 | 多标签 + 收藏分类 + 双栏 | `zsdesk-app/src/window/` | `interaction_regression` / `selection_regression` |
| L2 | 自研 MFT/USN 搜索 | `fast_search/{ntfs,native,query,snapshot}` | `runtime_regression` / `search_regression` |
| L3 | Everything 后端 | `fast_search/everything*` | `everything_regression` |
| L4 | Shell 右键菜单宿主 | `zsdesk-windows/src/shell/menu.rs` | `menu_runtime_tests` / `context_menu_regression` |
| L5 | 桌面格子 + 腾讯布局迁移 | `zsdesk-app/src/desktop/` | `desktop/*_regression` |
| L6 | Listary 集成 | `window/listary.rs` + `native/listary/` | — |
| L7 | 网络版（FTP/SFTP/UNC） | `zsdesk-windows` + `native/routing/` | `shell_reveal_regression` |
| L8 | 文件位置转发 | `explorer_forwarder` + `win_e_bridge` | `shell_reveal_regression` |
| L9 | 预览（图片/文本/外部程序） | `zsdesk-ui/src/preview/` | `preview_lifecycle_regression` / `quicklook_process_tests` |
| L10 | 缩略图 | `zc-windows/src/shell/thumb.rs` | `thumbnail_regression` / `icons_alpha_tests` |

- **新增回归面**（上游没有的，本项目引入）：**自绘列表交互**（虚拟滚动 / 框选 / 反向选择 / 就地重命名）、
  **自绘导航树**、**core-ui 能力协商与降级**、**深浅色切换** —— 这四条**必须新建回归测试**，
  否则 M1/M2 的新能力没有回归网。

**这决定后续 Phase 做什么**：M1 必须先立「自绘列表交互」的回归测试骨架（哪怕先只有 3 条：滚动窗口正确 / 选中集正确 / 键盘焦点正确），
否则 M3 的「完整回归」会漏掉本项目最核心的新代码。

---

## R10 · 未核实项清单与真机核对计划（U01~U12 + M01~M06）

**结论**：**32 个未核实项**分三类，且**本轮已把其中 3 个从未核实降级为已核实**。
剩余的**全部依赖真机**（本机无 Rust/C++ 工具链，`facts.md` §7.6）→ **只能在 CI 上做，且必须在 M0 完成最小闭环**。

**本轮已解决（3 项）**：

| 原未核实项 | 现状 | 依据 |
|---|---|---|
| GuoheView.exe 打开文件的命令行参数形式 | **已核实**：`"…\GuoheView.exe" "%1"`，标准 `%1` 参数约定 | 注册表 `HKCU\Software\Classes\GuoheView.*\shell\open\command`（66 个 ProgID） |
| 本机 core-ui.dll 与仓库 `1.9.0 build 345` 是否同版本 | **已核实：不同**。本机 = **1.9.0.346**，上游 = 345 | PE 版本资源 + DLL 内嵌字符串 `1.9.0.346`（`1.9.0.345` 不存在） |
| 12 个 `ui_image_gdi_*` 是否为闭源私有扩展 | **已核实：不是**。pinned revision 源码里有定义，与本机 DLL 导出集**双向差集为空** | 上游 `src/ui/ui_api.cpp:117,841,1691-1741` + 全 `src/` 扫描 567/567 一致 |

**仍需真机核对（29 项）**：

| 类 | 项 | 核对方法 | 何时 |
|---|---|---|---|
| **ABI 语义**（U01–U12） | `ui_init_with_theme` 参数；`ui_page_load_file/string` 路径编码与失败返回；`ui_page_open_window` 配置布局；`set_json/get_json` 类型映射与释放方；`ui_custom_on_draw` 回调签名与线程；`ui_widget_on_*` 注册/注销语义；`ui_bitmap_create` 像素格式/stride/生命周期；`ui_image_gdi_set_pixels` 的 GDI 语义与所有权；`ui_theme_*` 颜色取值域；`ui_debug_free` 是否唯一释放方；`ui_scroll_set_content` 语义；`ui_ocr_img_view_*` 可否复用 | ABI-NOTES §2 的最小探针；**顺序**：`ui_core_version*` → `ui_init_with_theme` → `ui_page_load_string` → `ui_page_open_window` → `ui_run`（U01–U03 闭环）→ 再接 `ui_custom_on_draw`（U05） | **M0**（U01–U03 必须先过） |
| **标记渲染**（M01–M06） | 未知标签行为（白屏 or 崩溃）；`flex-direction: row` 嵌套继承；`<label>` vs `<span>` 的 stretch 差异；`var(--token)` 自定义变量支持（**源码级已确认支持**，仅需真机复核）；class/id 选择器支持度；`<Separator>` 竖向 | 加载修正后的 `assets/main.uix` 截图核对 | **M1** |
| **环境** | ① core-ui 从源码构建的**耗时与产物体积**；② clang-cl 在 CI runner 上的**实际路径**；③ CI 自建 core-ui.dll 的**导出集**（是否 ⊆ 钉住清单）；④ Rust 侧 `cargo fmt --check` / `clippy -D warnings` 是否零告警；⑤ 发布包最终体积（见 R02）；⑥ Win10 19045 上的 Mica/高对比度实际表现（若 R05 决定保留） | CI job 里输出并写进 `spec/ci-contract.md` 的实测基线 | **M0 / M3** |

**这决定后续 Phase 做什么**：**M0 的验收必须包含「U01–U03 最小闭环跑通」**（否则 M1 写的每行 FFI 都是猜的）。
**R05 的 A3/A5/A7 三条**（高对比度 / Mica / narrator）**不在 U/M 清单里**，因为它们**不是「未核实」而是「已核实为不可实现」** ——
需单列一条用户裁剪决策（R05 结尾 / R07-A）。

---

## 三条决策

- 做：按 D00–D11 已定的路线推进，**并把本简报的 R07-A 提到 M0 之前**裁剪 ——
  高对比度 / Mica / narrator 三项要么立项、要么显式记为「已知不可实现」写进 README 与验收清单，
  不允许在 M1 验收期才发现。
- 不做：不引入 WinUI 3 / Windows App SDK 运行时（C1 + D00-4）；不链接 `ghde.dll`、不随包分发任何闭源二进制（D02/C4）；
  不做 32 位 x86、不做其余 5 个 edition（D05）；**不按「版本号 = 345」做能力协商**（本机是 346）——
  改为按**符号存在性**协商（D08 口径不变，依据升级）。
- 待验证：**发布包体积红线**（10 MB 在加进 core-ui.dll + Everything + 240 个许可副本后已不可能，需裁定砍哪些）
  与 **文本预览上限/编码策略** —— 两条都需用户拍板，见 R02 与 R04 结尾。

---

## 关联文档

| 文件 | 内容 |
|---|---|
| `research/facts.md` | 事实核对底稿（10 组实测；**§7.9 的版本结论被本简报结论 4 修正**） |
| `DECISIONS.md` | 决策台账 D00–D11 + C1–C5 + T1–T8 |
| `SPEC.md` | 技术总览与索引 |
| `spec/migration-matrix.md` | 逐文件四档矩阵（R01/R09 的数据源） |
| `spec/ffi-core-ui.md` | FFI 与呈现层契约（D08–D10） |
| `spec/ci-contract.md` | CI 与打包契约 + G1–G10 缺口 |
| `spec/milestones.md` | M0–M3 验收条目（R08 的落点） |
| `docs/05-COREUI-ABI-NOTES.md` | 18 VERIFIED / 12 UNVERIFIED（**V05 的「头文件多 2 个」应为 1 个**，见结论 5） |
| `docs/11-COREUI-MARKUP-TAGS.md` | 标记标签权威表 + token 冲突 |
| `docs/12-UIX-TAG-FIX.md` | 前端 tag 修正与 17 token 终表 |
| `docs/10-FORK-ROADMAP-COREUI.md` | 用户 fork 路线图（被 D02/D03/D06 覆盖处已标注） |
