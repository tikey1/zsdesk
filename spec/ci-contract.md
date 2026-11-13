# CI 与打包契约 —— 目标态 vs 当前态

> 契约编号：`ZC-CI-001`（v3）
> 状态：已定（随 D03 / D05 / D09 / D11 生效）
> 评审对象：`.github/workflows/build.yml`（**当前 402 行 / 3 个 job，team-lead 实现并已推送**）+ `docs/ci.md`
> 依据：上游 `qiu7824/zsdesk` 的 `.github/workflows/build.yml`（实测规范）+ `research/facts.md` §7.4 / §7.6
> **CI 现状（已实测）**：fork 的 Actions **早已启用**，并在 commit `4a48349` 跑绿过
> `Windows build #1`（9m58s）。
> **第一次新 CI 已实跑**：run `43387095246` —— 步骤 1–11 全绿（cargo test / cargo build / clippy /
> clang-cl 探测 / ninja 安装），**挂在第 12 步 `Build core-ui from source`**；
> 根因定位为 core-ui 用 `FetchContent` 现场拉 QuickJS-NG（`GIT_TAG v0.14.0` + `GIT_SHALLOW TRUE`），
> 已加固（预置 `_deps/quickjs-src` + `GIT_TERMINAL_PROMPT=0`），第二次 CI 已推送。

> **v3 的定位变更**：v1 是「从零写 CI」，v2 是「评审 + 缺口」。
> **v3 改成两栏对照** —— 左栏「目标态」是契约，右栏「当前态」是**已实现**的事实。
> **本文件不再引用尚不存在的脚本名而不加标注**（v2 的问题：把目标态写成了现状）。

---

## 1. 两栏读法

| 栏 | 含义 | 权威性 |
|---|---|---|
| **目标态** | 契约要求「最终必须成立」的性质 | **权威**。这一栏是验收依据 |
| **当前态** | 2026-11-13 `.github/workflows/build.yml` 的**实际内容** | **事实**，但可能落后于目标态 |

**判定列的三种取值**：
`✅ 已达成` · `🟡 部分达成` · `⛔ 未达成` · `⏳ 待绑定层落地后启用`

> **一条纪律**：目标态里的每一条，要么已达成，要么有明确的**落点**与**里程碑**。
> 不写「建议」这种无法判死的条目。

---

## 2. 当前态快照（实测 `build.yml`，2026-11-13）

```
job rust       （无 needs → 与 core-ui 互不依赖）
  Read the pinned ZSUI revision → checkout qiu7824/zsui
  checkout ghboke/core-ui（pinned revision）
  rust-toolchain 1.99.0
  Test              cargo test --workspace --release --locked
  Build the full edition (x64)   cargo build --locked --release -p zsdesk-app
  Clippy (advisory) continue-on-error
  Upload the executable → artifact zsdesk-app-full-x64

job core-ui   （无 needs）
  Read the pinned core-ui revision → checkout ghboke/core-ui
  Locate clang-cl and llvm-rc        ← llvm-rc 缺失即 throw
  Ensure Ninja is available           ← pip install ninja 兜底
  Clean the build directory           ← 删 clang-cl-build/（防复用旧 CMakeCache）
  Pre-fetch QuickJS-NG into the FetchContent slot   ← 预置 _deps/quickjs-src
  Build core-ui from source
  Upload the core-ui build log        ← 失败时可取日志
  Verify core-ui.dll is a Windows x86-64 image      ← PE Machine == 0x8664
  Gate G2  — DLL 版本必须与 pinned revision 对应（core-ui/VERSION ↔ DLL 版本资源）
  Gate S7  — core-ui.dll 必须链动态 VC++ 运行库（dumpbin 判定 /MD）
  Upload core-ui.dll → artifact core-ui-x64

job package  needs: [rust, core-ui]
  Download the executable / Download core-ui.dll
  Gate S12 — core-ui.dll 必须与 exe 同目录
  Collect the Visual C++ x64 Redistributable    ← vswhere 定位
  Package the portable x64 zip
      exe→ZSDesk.exe / core-ui.dll / core-ui.dll.revision / assets/（含断言）
      / 3 个 CRT DLL / LICENSE / notices / licenses/ / 说明.txt / manifest.sha256
  Upload the portable package → ZSDesk-full-x64
```

---

## 3. 逐条对照

### 3.1 工作区与步骤顺序

| # | 目标态 | 当前态 | 判定 |
|---|---|---|---|
| 1 | CI 工作区 = `app/` + `zsui/` + `core-ui/` 三级同级（对应 `../../../xxx` path 依赖） | 已实现 | ✅ |
| 2 | **Rust 侧在 core-ui 之前**（D09：动态加载 → Rust 不依赖 core-ui） | `rust` job 无 `needs`；`core-ui` job 无 `needs` → **互不依赖** | ✅ |
| 3 | `package` job 依赖前两者 | `needs: [rust, core-ui]` | ✅ |
| 4 | 只 full edition（`network+desktop`） | `cargo build -p zsdesk-app`（default features） | ✅（建议后续显式写 features） |
| 5 | `cargo test --workspace --release --locked` | 已实现 | ✅ |
| 6 | env：`CARGO_INCREMENTAL` / `CARGO_PROFILE_DEV_DEBUG` / `CARGO_PROFILE_TEST_DEBUG` = '0' | 已实现 | ✅ |
| 7 | 可补 `CARGO_PROFILE_RELEASE_DEBUG: '0'` | 未设 | 🟡 低优先 |
| 8 | `clippy` 最终必须转阻断 | `continue-on-error: true`（advisory） | 🟡 **M1 前收敛**（见 §5.1） |

### 3.2 core-ui 构建

| # | 目标态 | 当前态 | 判定 |
|---|---|---|---|
| 9 | **契约约束的是「构建结果」，不是「调用哪个脚本」** —— D03 已明确「core-ui 自身无构建工作流，这套步骤要我们自己维护」。当前实现调用上游 `scripts/build-clang-cl.ps1 -Target core-ui`；**若 H1 成立（见 §5）则改为 CI 直接调 cmake** | 已实现（用上游脚本） | ✅ / ⚠️ 见 §6 |
| 10 | `llvm-rc` 缺失即失败（Windows SDK 的 `rc.exe` 10.0.26100 在 ninja 子进程会挂起） | `Locate clang-cl and llvm-rc`，缺 llvm-rc 直接 throw，并注释说明「不能静默退回 rc.exe」 | ✅ |
| 11 | 构建前清掉 `clang-cl-build/`（上游脚本**目录存在就跳过 configure** → 会用旧配置产物） | `Clean the build directory` | ✅ |
| 12 | core-ui 用 `FetchContent` 现场拉 QuickJS-NG → CI 需预置源码 | `Pre-fetch QuickJS-NG into the FetchContent slot`（`git clone --depth 1 --branch v0.14.0 --recurse-submodules` → `_deps/quickjs-src`） | ✅ **这是第 12 步失败的加固** |
| 13 | 构建失败要能取到日志 | `Upload the core-ui build log` | ✅ |
| 14 | 不加 `-StaticCrt`（保持 `/MD`） | 未加 | ✅ |

### 3.3 产物校验（Gate）

| # | 目标态 | 当前态 | 判定 |
|---|---|---|---|
| 15 | **位数**：PE Machine == `0x8664` | `Verify core-ui.dll is a Windows x86-64 image` | ✅ |
| 16 | **G2 revision 一致性**：DLL 版本资源 == pinned revision 的 `VERSION` | `Gate G2 — the DLL version must match the pinned revision` | ✅ |
| 17 | **G2b 戳记**：打包写出 `core-ui.dll.revision`，运行时比对 | 打包拷 `app/target/release/core-ui.dll.revision`（**若存在**）→ 戳记由 Rust 侧 build.rs/构建产出 | 🟡 **戳记的产出方要定**（见 §5.2） |
| 18 | **S7 CRT 动态性**：导入表必须含 `MSVCP140.dll` + `VCRUNTIME140.dll` | `Gate S7 — core-ui.dll must link the dynamic VC++ runtime`（dumpbin 判定） | ✅ |
| 19 | **S12 同目录**：`core-ui.dll` 与 exe 同目录 | `Gate S12` | ✅ |
| 20 | **S7'-第一段**：CI 编出的 DLL 的 `ui_*` 导出集 vs `docs/05-COREUI-EXPORTS.txt`（567 个）；**判据按实测符号集**（PM 扫 pinned revision 全 `src/` 的 `UI_API` → 567，与本机 DLL 双向差集为空，仅 `ui_dialog` 例外）→ 断言「CI 产物的 `ui_*` 集 ⊇ 本机 567（允许 `ui_dialog` 缺）」且「差集 ⊆ {`ui_core_embed_binary`}」 | 未实现 | 🟡 **建议本版就加**（不需要绑定层落地） |
| 21 | **S7'-第二段 / S8 绑定面**：绑定层**必需符号集** ⊆ S7' 实测导出集；必需 ∩ 可选 = ∅；可选里的 12 个「头文件漏声明」符号签名必须与 `src/ui/ui_api.cpp` 一致 | 未实现 | ⏳ **M1 加**（需绑定层先落地） |

### 3.4 打包

| # | 目标态 | 当前态 | 判定 |
|---|---|---|---|
| 22 | **G1 `assets/` 必须进包**（`main.uix` / `preview.uix` / `about.uix` / `themes.json`）—— 缺了交付的 exe **启动即失败** | 已实现，且**逐个断言**四个文件存在 | ✅ |
| 23 | 逐文件 `bytes + SHA-256` manifest（沿用上游口径） | `manifest.sha256`（`hash  rel  bytes  version` —— **多了文件版本列**） | ✅ |
| 24 | **G6 CRT/资源来源记录** + 已知风险提示 | `说明.txt` 写明「同目录运行库优先于系统目录；遇异常请装最新 VC++ x64 运行库」；manifest 末列是文件版本 | ✅ |
| 25 | 产物名 `ZSDesk-<version>-full-x64.zip`，只 x64 | 已实现 | ✅ |
| 26 | **Everything 引擎不入包**（可选后端） | 未入包 | ✅（但见 §5.4 的产品条款） |
| 27 | `licenses/` + LICENSE + notices | 已实现 | ✅ |

### 3.5 门禁脚本（机械门禁）

| # | 目标态 | 当前态 | 判定 |
|---|---|---|---|
| 28 | **S9 crate 依赖无环** | `scripts/gates/check_crate_dag.py` —— **脚本不存在**，CI 未调用 | ⛔ 见 §5.5 |
| 29 | **S10 FFI 层无业务依赖** | 同上 | ⛔ |
| 30 | **G01/G03/G04/G08 + 工程门禁 P0** | `scripts/gates/verify.py` / `gates_engineering.py` —— **脚本不存在**，CI 未调用 | ⛔ 见 §5.5 |
| 31 | **S13 token 不撞名**（禁止自定义库 token 名，见 `spec/ffi-core-ui.md` §5.6.1） | 未实现 | ⛔ 见 §5.5 |
| 32 | **G7 错误流**：移走 `core-ui.dll` → 启动 → 非零码退出 + 打印候选路径列表 | 未实现 | ⛔ 见 §5.6（M0 之后） |
| 33 | **G10 `zsui` 收口门禁**（M3：workspace 不再有 zsui path 依赖、CI 不再 checkout zsui） | 未实现 | ⏳ M3 |

---

## 4. 仍然缺的（按落地顺序）

### 4.1 🟡 `clippy` 收敛期限（M1 前）

**现状**：`continue-on-error: true`（advisory）。
**目标**：M1 结束前转成阻断（`cargo clippy --locked --release -p zsdesk-app -- -D warnings`）。
**理由**：`clippy -D warnings` 是 Rust 侧唯一能机器判死的「unsafe 用错 / 错误转换」门禁；
一直观察等于这道门禁不存在。**建议在 `build.yml` 里写一行注释标明收敛期限**，避免变成永久豁免。

### 4.2 🟡 revision 戳记的**产出方**要定（G2b）

**现状**：打包步骤拷 `app/target/release/core-ui.dll.revision`（**若存在**）——
即戳记由 **Rust 构建**产出，而不是由 CI 从 `packaging/core-ui-revision.txt` 直接写。

**两种口径，二选一（必须一致）**：

| 口径 | 产出方 | 优点 | 缺点 |
|---|---|---|---|
| **甲：Rust 产出** | `crates/zsdesk-coreui/build.rs` 把 `packaging/core-ui-revision.txt` 的 SHA 固化成 `ZSCOREUI_CORE_UI_REVISION`，并**同时**在 `target/release/` 旁写一份 `core-ui.dll.revision` | 运行时比对的是**编译期固化的那个值**（`spec/ffi-core-ui.md` §5.5 机制），编译期与运行期天然一致 | build.rs 要写文件，跨平台要注意 |
| **乙：CI 产出** | CI 从 `packaging/core-ui-revision.txt` 读 SHA，打包时写 `core-ui.dll.revision` | 简单直接 | 运行时要比对的必须是**同一个**值 —— Rust 侧得从 exe 同目录读戳记文件，而不是用编译期常量 |

**架构侧推荐：甲**（Rust 产出），CI 只负责**把戳记拷进包**。
理由：`spec/ffi-core-ui.md` §5.5 的比对机制是「编译期固化的 `EXPECTED_REVISION` vs 旁置戳记」，
**编译期常量才是权威**；若戳记由 CI 写，两者可能来自不同来源 → 比对失去意义。
→ **契约要求**：戳记文件的内容必须 **== 编译期 `EXPECTED_REVISION`**；
CI 侧加一条断言「打包进去的戳记 == `packaging/core-ui-revision.txt`」。

### 4.3 🟡 S7' 导出面（CI 编出的 DLL vs 导出表）—— **两段，建议本版就加第一段**

**为什么必须**：`Gate G2` 比的是**版本号**；但「同版本」不等于「同 API 面」。
上游头文件**漏声明**了 12 个本机实际导出的符号（`docs/05-COREUI-ABI-NOTES.md` V04），
说明**版本号与 API 面不是单调关系**。

**★ 判据已按实测符号集更新（2026-11-13）**：PM 对 pinned revision `1840d689` 全 `src/`
做 `UI_API` 扫描 → **567** 个 `ui_*`，与本机 DLL 的 567 个**双向差集为空**，
仅 `ui_dialog` 例外（头文件有声明、源码自 build 158 已删）。
→ 那 12 个「头文件漏声明」的符号**在仓库 main 的源码里也有** → **CI 产物同样有**。

**第一段（🟡 建议本版就加 —— 不需要绑定层落地）**：

```
CI 编出的 core-ui.dll → 导出表 → 取 ui_* 集合 C
docs/05-COREUI-EXPORTS.txt → 取 ui_* 集合 L（本机 DLL，567 个）
断言：L 去掉 ui_dialog 后 ⊆ C              （本机有的，CI 产物也必须有）
断言：C 去掉 L 后 ⊆ {ui_core_embed_binary}  （仓库 main 多出来的只能是这一个）
```

> `ui_core_embed_binary` 其实是 **CMake 辅助函数**（`cmake/embed_binary.cmake`），
> 不是 C API，从来不在任何 DLL 的导出表里 → 它不该出现在任何导出集合里。

**为什么建议现在加**：① 纯 PE 导出目录解析，PowerShell 可独立完成；
② **不需要绑定层落地** → 今天就能加；
③ 它能**当场证伪**「CI 产物缺符号」这类问题 —— 而这正是当前 CI 卡住的那类问题。

**第二段（⏳ M1 加 —— 需要绑定层先落地）**：
绑定层的**必需符号集** ⊆ S7' 实测导出集；必需 ∩ 可选 = ∅；
可选里那 12 个符号的签名必须与上游 `src/ui/ui_api.cpp` 一致。

### 4.4 ⛔ 静态门禁与 crate 依赖方向门禁

**注意**：v2 列出的 4 个脚本路径 **在仓库里都不存在**（实测确认）：

| 脚本 | 状态 | 落点 | 内容约定 |
|---|---|---|---|
| `scripts/gates/check_crate_dag.py` | ❌ 不存在 | M1 前创建 | 解析各 crate `Cargo.toml` 的 path 依赖 → 建图 → 判环（**S9**）；并断言 `crates/zsdesk-coreui/Cargo.toml` 不含任何指向 `zsdesk-core`/`-platform`/`-ui`/`-app`/`-windows` 的 path 依赖（**S10**） |
| `scripts/check-coreui-api.py` | ❌ 不存在 | 绑定层落地后创建 | 解析绑定层声明的符号名 → 必须 ⊆ S7' 实测集合；必需∩可选=∅；必需不含 12 个「本机独有」符号（**S8**） |
| `scripts/gates/verify.py` / `gates_engineering.py` | ❌ 不存在 | 从专家包 `templates/gates/` 复制 | G01/G03/G04/G08 + 工程门禁 P0 |
| S13 token 门禁 | ❌ 不存在 | M1 前创建 | 扫描 `assets/*.uix` + `assets/themes.json` 的自定义属性**定义**（`--name:`，不含 `var(--name)` 引用）→ 命中库 34 个 token 全集即失败；另禁自定义 `--app-accent` |

> **纪律**（`mechanical-gates` §4）：**失败即停**。禁止调低阈值、加 ignore、`it.skip` 混过去。
> 例外必须**显式登记豁免**（谁豁免 / 为什么 / 何时复审）。

### 4.5 ⏳ G7 错误流（M0 之后）

**契约**：**移走 `core-ui.dll` 后启动，必须非零码退出，并在 stderr 打印完整候选路径列表**；
**不得**静默回退到 `C:\Program Files\GuoheView\core-ui.dll`（兜底，不是分发渠道），**不得**白屏。

**为什么是契约**：这是 D09「显式限定搜索路径」的全部价值所在。
启动时悄悄用了 GuoheView 那份 → 用户会遇到「API 面不匹配」的玄学问题，且**永远不会知道**。

**加法（M0 落地后）**：CI 里加一步 —— 把 exe 与 `core-ui.dll` 复制到临时目录 →
删掉 `core-ui.dll` → 运行 exe → **断言退出码非 0 且 stderr 含 `core-ui.dll`**。

### 4.6b 🟡 戳记的**断言**（口径「甲：Rust 产出」）

**契约**：戳记由 **Rust 侧 `crates/zsdesk-coreui/build.rs`** 在编译期写出
（内容 = `packaging/core-ui-revision.txt` 的 SHA + 一个换行，无 BOM；
路径 = `target/<profile>/core-ui.dll.revision`，**与 exe 同目录**）。
**CI 不自己写戳记**，只做**校验**：

```
打包后断言：
  包内 core-ui.dll.revision 存在
  且 其内容（去首尾空白）== packaging/core-ui-revision.txt 的内容（去首尾空白）
  不等 → 打包失败
```

> **为什么不让 CI 写**：编译期常量 `EXPECTED_REVISION` 是唯一权威来源；
> 若 CI 也写一次戳记，就有两个来源 → 运行时比对失去意义。
>
> 打包步骤**不用改**：现在的实现已经拷 `app/target/release/core-ui.dll.revision`
> —— build.rs 写到那里正好被拷进 ZIP；只需**再加一条内容断言**。

### 4.6 一条**不属于 CI 但必须写进契约**的产品条款

**Everything 引擎不入包**（已定）→ 用户的机器上**可能没有 Everything**。

→ **产品契约**：Everything 是**可选后端**；不可用时搜索必须**自动回退**
（MFT / USN 索引 → 目录扫描），且 UI 上要**让用户知道当前用的是哪个后端**
（上游 `zsdesk-core` 已有 `SearchBackendPreference`，
`zsdesk-app/src/fast_search/everything_runtime.rs` 已有运行时探测）。
**不允许**因为 Everything 缺失就静默地「搜不出东西」。

→ **CI 侧对应冒烟项（M3）**：在**没有 Everything** 的 runner 上，搜索仍能返回结果。
---

## 5. ★ core-ui 构建失败的诊断（2026-11-13，基于上游源码的独立分析）

> 背景：CI 已实跑两次，`rust` job 成功，**core-ui job 都挂在 `Build core-ui from source`**。
> 第一次 run `43387095246`；第二次 run `43388501675`（**QuickJS 预置步骤成功** → 网络不是唯一原因）。
> 第三次已推送（commit `0c20df7`，新增日志 artifact 与 step summary）。
> 下面是架构侧基于**上游源码 + CMake 语义**给出的诊断，**按可能性排序**。

### 5.0 先说结论

> **预置 QuickJS 到 `_deps/quickjs-src` 这个做法本身是无效的** ——
> 它既不能让 CMake 跳过下载，又额外拖慢一次克隆。
> **CMake 的官方开关是 `-DFETCHCONTENT_SOURCE_DIR_QUICKJS=<路径>`。**

### 5.1 H1 —— 预置路径无效，CMake 仍在自己下载（**最可能**）

**依据（三条都可复核）**：

| 事实 | 出处 |
|---|---|
| core-ui 的 CMakeLists 用 `FetchContent_Declare(quickjs GIT_REPOSITORY … GIT_TAG v0.14.0 GIT_SHALLOW TRUE)` + `FetchContent_MakeAvailable(quickjs)` | `CMakeLists.txt:41-52` |
| 上游脚本 `build-clang-cl.ps1` 的 `$needsConfigure = -not (Test-Path (Join-Path $buildDir 'CMakeCache.txt'))` → **只要 build 目录存在就跳过 configure** | `scripts/build-clang-cl.ps1:103` |
| FetchContent 是否需要下载，**由 `_deps/<name>-subbuild/<name>-populate-prefix/src/<name>-populate-stamp/` 下的 stamp 决定**，**不是**由 `_deps/<name>-src` 里有没有文件决定 | CMake `ExternalProject` 语义 |

**因果链**：CI 每次删掉整个 `clang-cl-build/`（团队为了避免 stale `CMakeCache.txt`）→
**stamp 全被删掉** → FetchContent 判定「未下载过」→ **重新 clone** →
预置到 `_deps/quickjs-src` 的那份**被覆盖或白做**。

> ★ 这解释了「预置步骤成功但构建仍失败」：
> **预置成功 ≠ CMake 复用了它。**

**正解（两种，推荐第二种）**：

| 方案 | 做法 | 评价 |
|---|---|---|
| 甲 | 预置到 `_deps/quickjs-src`，并**保留 stamp 文件** | ✗ 依赖 CMake 内部 stamp 路径与文件名，**脆弱**；CMake 版本一变就失效 |
| **乙（推荐）** | **改用 CMake 的官方覆盖变量**：FetchContent 的名字是 `quickjs` → 变量名大写 → **`-DFETCHCONTENT_SOURCE_DIR_QUICKJS=<绝对路径>`**。设了它，FetchContent **完全跳过 download step**（不 clone、不建 subbuild、不拷 `_deps`），直接就地编译该目录 | ✅ **官方机制**，跨 CMake 版本稳定 |

> **障碍与绕法**：上游 `build-clang-cl.ps1` **不透传额外的 `-D`**（参数只有
> `Target` / `Clean` / `StaticCrt` / `VsPath` / `ClangClPath`）。
> → **绕法**：CI 里**不调用上游脚本**，直接调 cmake，参数照抄脚本（见 §5.4），
> 追加 `-DFETCHCONTENT_SOURCE_DIR_QUICKJS=…`。vcvars 环境用 `ilammy/msvc-dev-cmd@v1`
> 或复刻脚本里的 `cmd.exe /c "set … && vcvars64.bat >nul && set"` env 导入。
>
> **这不违反任何既有决策**：D03 原文就是「core-ui 自身无构建工作流，**这套步骤要我们自己维护**」。
> 契约约束的是**构建结果**（x64 / `/MD` / 版本匹配 / 导出面一致），**不是调用哪个脚本**。

### 5.2 H2 —— 预置用了 `--recurse-submodules`，会去克隆一个巨大的无用仓库

**依据**：quickjs-ng v0.14.0 的 `.gitmodules` 内容（已核实）：

```
[submodule "test262"]
	path = test262
	url = https://github.com/tc39/test262.git
	shallow = true
	update = none
```

**只有一个 submodule `test262`，且 `update = none`** → **构建根本不需要它**
（它是 JS 引擎的测试套件）。

→ **预置 clone 里的 `--recurse-submodules` 应当去掉**。
否则会额外浅克隆 tc39/test262（体量极大）→ **慢、超时、或把 runner 的时间/带宽吃光**。

→ 预置应当用不带 `--recurse-submodules` 的浅克隆（指定 `--branch v0.14.0` + `--depth 1`）。

### 5.3 H3 —— 真正的 C++ 编译 / 配置错误（若 H1/H2 都不是）

**诊断方法**（run 3 的 artifact 里有完整日志）：

| 日志特征 | 含义 | 处置 |
|---|---|---|
| `error:` 开头的**第一条**（不是 warning） | 真实的编译错误 | 按错误信息定位；quickjs-ng 的 C 代码与 clang-cl 的兼容问题最常见 |
| `CMake Error at …FetchContent` / `git clone` / `fatal:` | 下载路径失败 | 回到 H1（用 `-DFETCHCONTENT_SOURCE_DIR_QUICKJS` 彻底不下载） |
| `LNK2038` / `RuntimeLibrary mismatch` | CRT 不一致 | 确认没传 `-StaticCrt`，且 core-ui 与 quickjs 的 CRT 选项一致 |
| `rc.exe` 挂起（步骤无输出直到超时） | 资源编译 | 已被 llvm-rc 硬失败检查覆盖；若仍出现，确认 `CMAKE_RC_COMPILER` 真的传进去了 |
| `ninja: build stopped` 前的 `FAILED:` 行 | 哪个 target 失败 | 记下 target 名 |

> **诊断纪律**：只看**第一条** error。不要被后续上百条连锁错误带偏。

### 5.4 若改为「CI 直接调 cmake」，参数照抄上游脚本

```text
cmake -S <core-ui 目录> -B <core-ui>/clang-cl-build -G Ninja
      -DCMAKE_BUILD_TYPE=Release
      -DCMAKE_C_COMPILER=<clang-cl 前向斜杠路径>
      -DCMAKE_CXX_COMPILER=<clang-cl 前向斜杠路径>
      -DCMAKE_RC_COMPILER=<llvm-rc 前向斜杠路径>
      -DFETCHCONTENT_SOURCE_DIR_QUICKJS=<预置的 quickjs 源码绝对路径>
cmake --build <core-ui>/clang-cl-build --target core-ui
```

**要点**：

1. `-DCMAKE_RC_COMPILER` **必须显式给 llvm-rc**（上游脚本只在 `llvm-rc.exe` 与 `clang-cl.exe`
   同目录时才加这一项 —— 团队已有硬失败检查，✅）；
2. **不加** `-DUI_CORE_MSVC_STATIC_CRT=ON`（保持 `/MD`，D00-5）；
3. `FETCHCONTENT_SOURCE_DIR_QUICKJS` 的值应是**预置目录**，且该目录**必须在 `clang-cl-build/` 之外**
   （否则被 clean 删掉）；
4. 预置目录里要有 `CMakeLists.txt`（quickjs-ng 有）；
5. `lunasvg` / `plutovg` 是 **vendored** 在 `third_party/lunasvg/`（含 plutovg 子目录）
   → **不联网**，不需要额外处理（已核实 `CMakeLists.txt` 里只有 quickjs 一个 FetchContent）。

### 5.5 与本契约的关系

| 契约条目 | 状态 |
|---|---|
| 目标是「core-ui 从 pinned revision 构建出 x64 / `/MD` / 版本匹配的 DLL」 | **不变**（§3.2 第 9 行已改为约束结果而非调用方式） |
| 「用上游脚本」 | **降级为「当前实现」**，不再是契约要求 |
| 门禁 G2 / S7 / S12 / G1 | **不变**，且与构建方式无关 |


---

## 6. 一句话结论

> **工作流主干与「产物可信」这一层（位数 / revision 版本 / 动态 CRT / 同目录 / assets 进包 /
> manifest 版本列 / 干净构建 / llvm-rc 硬失败 / QuickJS 预置）已经到位** ——
> 这是一次高质量的实现，架构侧逐条核对无误。
> **剩下的缺口集中在「机械门禁」这一层**（S7' 导出面 / S8 绑定面 / S9·S10 依赖方向 /
> 静态门禁 / S13 token），它们**依赖绑定层与门禁脚本先落地**，因此不是 CI 的问题，
> 而是**M1 的产出物**。
> **另外两条要在 M1 前定**：clippy 的收敛期限（§4.1）、revision 戳记的产出方（§4.2）。

---

## 7. 端到端验证（CI 必须能自证）

```powershell
# ① Rust 侧（不需要 core-ui）
cargo test --workspace --release --locked

# ② core-ui 侧（当前态已覆盖）
#   - PE machine == 0x8664                        ✅ 已实现
#   - DLL 版本 == pinned revision 的 VERSION       ✅ 已实现（Gate G2）
#   - 导入表含 VCRUNTIME140.dll + MSVCP140.dll      ✅ 已实现（Gate S7）
#   - 导出面 L ⊆ C 且 C\L ⊆ {embed_binary, dialog} ⛔ 待实现（S7'）
#   - 绑定面 ⊆ 实测导出集合                        ⏳ 待绑定层落地（S8）

# ③ 产物侧（当前态已覆盖）
#   - core-ui.dll 与 exe 同目录                    ✅ 已实现（Gate S12）
#   - assets/ 四件套齐全                           ✅ 已实现
#   - core-ui.dll.revision 戳记 == revision 文件     🟡 产出方待定（§4.2）
#   - manifest 每个文件的 sha256 可复算             ✅ 已实现

# ④ 关键错误流
#   移走 core-ui.dll → 启动 → 非零码退出 + stderr 打印候选路径列表   ⛔ 待实现（G7）

# ⑤ 机械门禁
python3 scripts/gates/check_crate_dag.py         ⛔ 脚本不存在
python3 scripts/check-coreui-api.py              ⛔ 脚本不存在
python3 scripts/gates/verify.py --phase 3        ⛔ 脚本不存在
```

---

## 8. 本文件不做的事

1. **不重写** CI 实现 —— 实现归运维，契约归架构；本文件只给判定与缺口。
2. **不规定** 具体用 PowerShell / Python / bash 写哪个脚本 —— 只规定**判定什么**。
3. **不改** 上游 CI 已验证可用的部分（checkout 机制、Rust toolchain、`--locked`、env 三项）。
4. **不把目标态写成现状** —— 这是 v2 的教训：v2 引用的 4 个脚本在仓库里并不存在。

---

## 9. 关联文档

- `spec/ffi-core-ui.md` —— 动态加载、版本协商、revision 戳记（§5.5）、页面加载（§4.3.2）、`.uix` 三条硬约束（§4.3.3）
- `spec/crate-layout.md` —— crate 划分、依赖方向（S9/S10 的对象）、`.uix` 资产约束
- `spec/milestones.md` —— M0~M3 与本契约的对应（哪条门禁在哪个里程碑启用）
- `docs/ci.md` —— 运维的实现说明
- `docs/05-COREUI-ABI-NOTES.md` —— U01~U12 未核实项
- `docs/05-COREUI-EXPORTS.txt` —— 绑定清单与 S7'/S8 的权威来源
- `docs/12-UIX-TAG-FIX.md` —— 17 个 token 终表（S13 的依据）
