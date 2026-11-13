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
| 17 | **G2b 戳记**：戳记内容 == pinned revision（`packaging/core-ui-revision.txt`） | **产出方已定 = Rust 侧**（`crates/zsdesk-coreui/build.rs`，口径甲）→ CI **只校验**；**CI 现在还在自己写一份**（Gate G2 里 `Set-Content`，写到 `clang-cl-build/`，两行）→ **删掉它** | 🟡 **断言待加**（§4.2 / §4.6b） |
| 18 | **S7 CRT 动态性**：导入表必须含 `MSVCP140.dll` + `VCRUNTIME140.dll` | `Gate S7 — core-ui.dll must link the dynamic VC++ runtime`（dumpbin 判定） | ✅ |
| 19 | **S12 同目录**：`core-ui.dll` 与 exe 同目录 | `Gate S12` | ✅ |
| 20 | **S7' 导出面门禁**：CI 编出的 DLL 的 `ui_*` 导出集 vs **同一 checkout 里 `src/` 的 `UI_API` 定义集**（★ **基准按 D19 定**：`docs/05-COREUI-EXPORTS.txt` 记的是**本机 1.9.0.346** 的实测面，与 CI 编的 **345** 跨版本 → 只能得到「版本漂移」而非真缺陷；**也不用头文件**（上游 `include/ui_core.h` **漏声明 12 个符号** → 会把它们误判成 unexpected）；`ui_dialog` 头文件有、源码无 → **容忍并报告**，不静默放宽） | ✅ **脚本已落地**：`scripts/gates/coreui_export_gate.py`（+ `pe_exports.py` 解析器，无外部依赖；解析器已用本机 `GuoheView\core-ui.dll` 实测**逐字节复现** docs/05 的 567 基线，正例/反例均正确） | ✅ **已接进 `build.yml`**：S13 五个门禁在 `rust` job 的「Gate S13」步骤；S7' 在 `core-ui` job 的「Gate S7'」步骤（🟡 **尚未经真实 CI 验证**） |
| 21 | **S8 绑定面**：绑定层**必需符号集** ⊆ S7' 实测导出集；必需 ∩ 可选 = ∅；可选里的 12 个「头文件漏声明」符号签名必须与 `src/ui/ui_api.cpp` 一致 | 未实现（`scripts/check-coreui-api.py` 仍不存在） | ⏳ **M1 加**（需绑定层先落地） |

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
| 28 | **S9 crate 依赖无环** | `scripts/gates/check_crate_dag.py` —— **脚本不存在**，CI 未调用 | ⛔ 见 §4.8 |
| 29 | **S10 FFI 层无业务依赖** | 同上 | ⛔ |
| 30 | **G01/G03/G04/G08 + 工程门禁 P0** | `scripts/gates/verify.py` / `gates_engineering.py` —— **脚本不存在**，CI 未调用 | ⛔ 见 §4.8 |
| 31 | **S13 token 不撞名**（禁止自定义库 token 名，见 `spec/ffi-core-ui.md` §5.6.1） | ✅ **脚本已落地**：`scripts/gates/s13_token_namespace_check.py --root .`（逐字口径实现 + 负控已证：注入 6 类缺陷 ⇒ EXIT=1，逐条带 `文件:行`）；⭐ **但 CI 仍未调用** | ✅ **已接进 `build.yml`**：S13 五个门禁在 `rust` job 的「Gate S13」步骤；S7' 在 `core-ui` job 的「Gate S7'」步骤（🟡 **尚未经真实 CI 验证**） |
| 31b | ★ **id 清单 == 实际 id 集合**（防止资产加 id 后 Rust 漏取句柄 → 静默不刷新；并查同页重 id） | ✅ **脚本已落地**：`scripts/gates/ui_id_manifest_check.py --root .`（期望集在 `assets/ui-ids.json`；实测「共 32 个唯一 id，3 页」EXIT=0） | ✅ **已接进 `build.yml`**：S13 五个门禁在 `rust` job 的「Gate S13」步骤；S7' 在 `core-ui` job 的「Gate S7'」步骤（🟡 **尚未经真实 CI 验证**） |
| 31c | **S7' 导出面**（CI 编出的 DLL vs 上游 `UI_API` 源码面） | ✅ **脚本已落地**：`scripts/gates/coreui_export_gate.py --dll <dll> --source-dir <core-ui/src> --include-dir <core-ui/include>`（+ 辅助 `pe_exports.py`） | ✅ **已接进 `build.yml`**：S13 五个门禁在 `rust` job 的「Gate S13」步骤；S7' 在 `core-ui` job 的「Gate S7'」步骤（🟡 **尚未经真实 CI 验证**） |
| 31d | 资产质量门（G05 对比度 / G13 间距 / 标签 / D-15 / SFC 骨架） | ✅ **脚本已落地**：`scripts/gates/qa_uix_verify.py`、`theme_contrast_check.py`、`uix_tag_check.py`（各自 `--root .`） | ✅ **已接进 `build.yml`**：S13 五个门禁在 `rust` job 的「Gate S13」步骤；S7' 在 `core-ui` job 的「Gate S7'」步骤（🟡 **尚未经真实 CI 验证**） |
| 32 | **G7 错误流**：移走 `core-ui.dll` → 启动 → 非零码退出 + 打印候选路径列表 | 未实现 | ⛔ 见 §5.6（M0 之后） |
| 33 | **G10 `zsui` 收口门禁**（M3：workspace 不再有 zsui path 依赖、CI 不再 checkout zsui） | 未实现 | ⏳ M3 |
| 34 | ★ **凡跑 cargo 的 job 都必须具备 workspace 的全部 path 依赖**：`zsui` 与 `app/` **同级**（`crates/zsdesk-{app,ui}/Cargo.toml` 里是 `path = "../../../zsui"`）→ 凡以 `app/` 为工作目录的 job 都要检出 `zsui/` | `rust` job ✅ 有（读 pin + `actions/checkout` → `path: zsui`）；**`package` job ❌ 只 checkout 了 `app`**（`build.yml` 第 344 行） | ⛔ **P0**（见 §4.7） |

---

## 4. 仍然缺的（按落地顺序）

### 4.1 🟡 `clippy` 收敛期限（M1 前）

**现状**：`continue-on-error: true`（advisory）。
**目标**：M1 结束前转成阻断（`cargo clippy --locked --release -p zsdesk-app -- -D warnings`）。
**理由**：`clippy -D warnings` 是 Rust 侧唯一能机器判死的「unsafe 用错 / 错误转换」门禁；
一直观察等于这道门禁不存在。**建议在 `build.yml` 里写一行注释标明收敛期限**，避免变成永久豁免。

### 4.2 ✅ revision 戳记的产出方 —— **已定：口径「甲：Rust 产出」（2026-11-13）**

**现状**：打包步骤拷 `app/target/release/core-ui.dll.revision`（**若存在**）——
即戳记由 **Rust 构建**产出，而不是由 CI 从 `packaging/core-ui-revision.txt` 直接写。

**两种口径（已选「甲」）**：

| 口径 | 产出方 | 优点 | 缺点 |
|---|---|---|---|
| **甲：Rust 产出 ✅ 已定** | `crates/zsdesk-coreui/build.rs` 把 `packaging/core-ui-revision.txt` 的 SHA 固化成 `ZSCOREUI_CORE_UI_REVISION`，并**同时**在 `target/release/` 旁写一份 `core-ui.dll.revision` | 运行时比对的是**编译期固化的那个值**（`spec/ffi-core-ui.md` §5.5 机制），编译期与运行期天然一致 | build.rs 要写文件（跨平台注意） |
| ~~乙：CI 产出~~ | ~~CI 从 `packaging/core-ui-revision.txt` 读 SHA，打包时写~~ | 简单直接 | 运行时要比对的必须是**同一个**值 → Rust 侧得从 exe 同目录读文件，而不是用编译期常量 |

**最终契约（逐字，运维照此改 `build.yml`）**：

| 项 | 约定 |
|---|---|
| **谁写** | **Rust 侧** `crates/zsdesk-coreui/build.rs`；CI **不写** |
| **何时** | build.rs 运行时（每次 `cargo build` / `cargo test`）→ 与 exe 天然同版本 |
| **写什么** | revision SHA（40 位 hex）+ **一个 `\n`**；**无 BOM**；**只有一行**（★ **D16：单行 SHA，不带 VERSION、不带第二行** —— 权威口径见 `spec/ffi-core-ui.md` §5.5；版本号不进戳记） |
| **写到哪** | `target/<profile>/core-ui.dll.revision`（**与 exe 同目录**）→ CI 打包拷的就是这个路径 |
| **CI 做什么** | **只校验**：包内戳记（trim）== `packaging/core-ui-revision.txt`（trim）；不等 → **打包失败** |
| **CI 要删的一步** | 现 Gate G2 里 `Set-Content ... "$expected`n<revision>"`（写 `core-ui/clang-cl-build/core-ui.dll.revision`，**两行**）→ **删掉**。两个来源 = 比对失去意义；且它不在 exe 同目录 |
| **分期（★ D16）** | **M0 过渡**：由 **CI 代写**戳记（Rust 绑定层尚未落地），但 CI **必须机器断言**「包内戳记 == `packaging/core-ui-revision.txt`」，不过即 fail；**M1 落地**：改为 Rust `build.rs` 编译期写，CI **只校验、不再代写** |

**为什么是 Rust 产出**：编译期常量 `EXPECTED_REVISION` 是**唯一权威来源**。
若戳记由 CI 写，两者可能来自不同来源 → 比对失去意义。
→ **契约要求**：戳记文件的内容必须 **== 编译期 `EXPECTED_REVISION`** == `packaging/core-ui-revision.txt`。

> **⚠️ 前置条件**：CI 必须**真的编到 `zsdesk-coreui`**（`zsdesk-app` 依赖它，或 CI 额外跑
> `cargo build --locked --release -p zsdesk-coreui`）—— 否则 build.rs 不跑 → 戳记不存在 →
> 打包断言失败（**预期行为**：失败即停）。

完整可粘贴的断言片段见 `spec/ffi-core-ui.md` §5.5。

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

### 4.6b ✅ 戳记的**断言**（口径「甲：Rust 产出」）

**契约**：戳记由 **Rust 侧 `crates/zsdesk-coreui/build.rs`** 在编译期写出
（内容 = `packaging/core-ui-revision.txt` 的 SHA + 一个换行，无 BOM，**单行**；
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
> —— build.rs 写到那里正好被拷进 ZIP；只需**再加一条内容断言**
> （可粘贴片段见 `spec/ffi-core-ui.md` §5.5）。
>
> **CI 要删的一步**：Gate G2 里现在那段 `Set-Content ... "$expected`n<revision>"`
> （写到 `core-ui/clang-cl-build/core-ui.dll.revision`，**两行**）→ 按本契约删掉。
>
> **⚠️ 前置条件**：CI 必须编到 `zsdesk-coreui` 这个 crate，否则 build.rs 不跑 → 戳记不存在 →
> 断言失败（**预期行为**：失败即停，不是「跳过」）。

### 4.6c ★ 已知限制清单必须随包交付（D13）

**这不是 CI 门禁，是交付物条款**（但它有一个可机械检查的形式）：

| 项 | 约定 |
|---|---|
| **内容来源** | `spec/milestones.md` §8 的限制清单（L1~L5）—— **唯一权威**，本节只是引用 |
| **必须出现在** | ① `DELIVERY.md`（交付说明）② `README-FORK.md`（fork 说明） |
| **CI 侧可机械检查** | 打包时断言这两个文件**存在于包内**（若包内带文档）；或断言仓库里存在这两份文件且**包含关键词** `高对比度` 与 `Mica` |
| **为什么** | L1~L3 是**用户会直接感知到的缺失**（开了高对比度没反应、读屏读不出、没有 Mica）→ 不写清楚就是「交付后才发现」 |

> **验收项**：`spec/milestones.md` **V3-13**（M3 收口时人工 + 打包内容检查）。

### 4.7 ✅ **已修**（2026-11-13）：`package` job 补上 `zsui/` 检出 —— 原 P0：`package` job 缺 `zsui/` → cargo 装载 workspace 失败（运维 2026-11-13 报，架构侧已独立复核）

**事实**（架构侧已核对 `build.yml` 与各 crate 的 `Cargo.toml`）：

| job | 检出 | 跑 cargo？ |
|---|---|---|
| `rust` | `app/`（第 30 行）+ `zsui/`（第 40–44 行：读 `app/packaging/zsui-revision.txt` → `actions/checkout` → `path: zsui`） | ✅ |
| `core-ui` | `app/` + core-ui 源码 | ❌（不跑 cargo） |
| **`package`** | **只有 `app/`（第 344 行）** | ✅（冒烟探针第 404 行 + 戳记断言第 426 行） |

**为什么致命**：workspace 成员 `crates/zsdesk-app`、`crates/zsdesk-ui` 都有
`zsui = { path = "../../../zsui" }` → 从 `app/crates/<name>/` 解析 = **`<workspace>/../zsui`** →
而 `package` job **没检出它** → cargo 装载 workspace 清单时**必然**失败：
`failed to load manifest for workspace member …/zsdesk-app`。

**会连炸两处，且第二处会误导人**：
1. 冒烟探针 `cargo build --release --locked -p zsdesk-coreui --example probe`（第 404 行）→ 失败；
2. 戳记断言（第 426–430 行）报 **`core-ui.dll.revision missing … build.rs did not run`**
   → ★ **把人引向「build.rs 没跑」**，而真正原因是「`zsui` 没检出」。
   （这类「错误信息指向错误根因」的失败最费时间 —— 要写进 `DELIVERY.md` 的排障提示。）

**修法（把 `rust` job 的两步原样复制到 `package` job，10 秒）**：

```yaml
      - name: Read the pinned ZSUI revision
        id: dependency
        shell: pwsh
        run: '"revision=$((Get-Content -LiteralPath app/packaging/zsui-revision.txt -Raw).Trim())" >> $env:GITHUB_OUTPUT'
      - uses: actions/checkout@v7
        with:
          repository: qiu7824/zsui
          ref: ${{ steps.dependency.outputs.revision }}
          path: zsui
```

**通用规则（写进契约）**：**任何会跑 cargo 的 job，都必须具备 workspace 的全部 path 依赖**；
`zsui` 与 `app/` 同级 → 凡以 `app/` 为工作目录的 job 都要检出 `zsui/`。
→ 将来再加 job（发布 job / 基准 job）时**照这条抄**，别再踩。

**✅ 修复已落地**（运维 2026-11-13 回报，QA 复核确认）：`build.yml` 的 `rust` job（`:30-44`）与
**`package` job（`:359-380`，注释块 `:363-368` 逐字写明了推理）现在都 checkout 了 `zsui`**，
用 `app/packaging/zsui-revision.txt` 的钉版 ✓。→ **本条闭环**。

> **顺带**：这条也解释了「戳记断言为什么在 CI 上可能失败」——
> 戳记由 `crates/zsdesk-coreui/build.rs` 产出，**前提是 cargo 真的跑起来了**。
> M3 收口掉 `zsui` 依赖后，这个前提才会消失（见 §3.5 第 33 条 G10）。

### 4.8 ★ 门禁执行契约 —— **脚本已存在，CI 一步都没调用**（QA 2026-11-13 报，架构侧复核确认）

> **按 `mechanical-gates.md` 的铁律：「没有执行器的规则只是建议。」**
> 现状：`build.yml` 里 **零** 处调用 `scripts/gates/**`（`grep 'scripts/gates|qa_uix_verify|uix_tag_check|theme_contrast|s13_token' build.yml` → 0 命中）
> → **包括 S13 在内，这些门禁现在都只在本地手工跑，CI 不执行** → 等于没有门禁。

**契约（逐条可执行）**：

| # | 门禁 | 脚本（**已存在**） | 调用命令 | 作用 |
|---|---|---|---|---|
| G-S13 | **S13 token 不撞名** | `scripts/gates/s13_token_namespace_check.py` | `python3 scripts/gates/s13_token_namespace_check.py --root .` | 库 34 个 token 名单判定 + `.uix` 禁定义 `--app-accent` + `themes.json` 允许 + 只拦定义不拦 `var()` 引用 |
| G-ID | **id 清单 == 实际 id 集合** | `scripts/gates/ui_id_manifest_check.py` | `python3 scripts/gates/ui_id_manifest_check.py --root .` | 期望集（`assets/ui-ids.json`）与 `.uix` 实际 id 双向比对 + 逐页条数 + **同页重 id** |
| G-S7' | **core-ui 导出面** | `scripts/gates/coreui_export_gate.py` | `python3 scripts/gates/coreui_export_gate.py --dll core-ui/clang-cl-build/core-ui.dll --source-dir core-ui/src --include-dir core-ui/include` | CI 编出的 DLL 的导出面 vs 上游 `UI_API` 源码面 |
| G-ASSET | **资产质量** | `scripts/gates/qa_uix_verify.py`、`theme_contrast_check.py`、`uix_tag_check.py` | 各自 `--root .` | G05 对比度 / G13 间距 / 标签 / D-15 / SFC 骨架 |

**执行规则**（★ **D18 已裁决**，本节把它落成可执行条款）：

1. **每个门禁一个独立 step**（不合并成一个脚本）—— 失败时**能一眼看出是哪一道**；
2. **退出码非 0 ⇒ 该 step 失败 ⇒ job 失败**（**fail-stop**）：
   禁止 `|| true`、禁止 `continue-on-error`（`clippy` 是**唯一的例外**，且有 M1 前的收敛期限，见 §4.1）；
   失败时用 **`::error::` annotation** 暴露**是哪一条**；
3. **工作目录 = 仓库根**（`app/`，因为资产与脚本都在仓库里）→ 一律用 `--root .`；
4. **按 job 归属落位**（D18：资产类五道门进 **`rust` job**）：
   `rust` job → G-S13 / G-ID / G-ASSET（三支）；
   **S7' 导出面门禁需要 core-ui 的 DLL** → 只能在 `core-ui` / `package` job 里跑（`rust` job 没有 DLL）→ 落 **`package` job**；
5. **门禁跑在 `cargo test` 之后、打包之前**（资产类门禁不需要编译产物）；
6. **脚本必须零第三方依赖**（如 `pe_exports.py` 是纯标准库实现）→ 否则「CI 里装依赖」会成为新的失败点；
7. ★ **解析 `.uix` 的 id 时必须先剥掉 HTML 注释**（D18 第 3 条）：
   注释里常写 `→ <custom id="preview_image">` 这类说明文字，**裸正则会误判成「重复 id」** → 门禁必须先剥注释再计数
   （`ui_id_manifest_check.py` 已按此实现；架构侧独立重数时也是先剥注释 → 得 32，与门禁一致 ✓）。

> **架构侧已本地实跑（2026-11-13，只读）**：五个脚本全部 **EXIT=0**；
> `ui_id_manifest_check.py` 报「共 **32** 个唯一 id，3 页」—— 与架构侧独立重数的结果一致 ✓
> （`main.uix` 26 + `preview.uix` 6 + `about.uix` 0）。
> ⇒ **脚本本身是可用的，缺的只是 CI 里的一步调用。**

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

> ★ **最高优先级：先排除「日志被截断」**（**D17，交付总监实测定位，2026-11-13**）：
> 连续两次 CI（run `43391037201` / `43392589050`）的 core-ui job 都「死在」
> `[72/97] Building CXX object … src/ui/reactive/binding.cpp.obj`，**戛然而止、零编译器报错**，
> 而 8 条 annotation 里**没有一条真 clang 报错**（全是 CMake 缓存变量）。
> **根因不是编译错误**：
> GitHub 的 `shell: pwsh` 注入 `$ErrorActionPreference = 'stop'`；
> PowerShell 7.4+ 的 `$PSNativeCommandUseErrorActionPreference` 默认 `$true`；
> 两者叠加 ⇒ native command（clang-cl）**只要往 stderr 写一行 warning** 就被判失败并抛
> `NativeCommandExitException` → **管道被中断** → `Tee-Object` 停止 → **编译器的真实诊断从未落盘**。
> ⇒ **修法（core-ui job 第一行）**：
> ```powershell
> $ErrorActionPreference = 'Continue'
> $PSNativeCommandUseErrorActionPreference = $false
> ```
> ⇒ **连带修**：annotation 提取的正则由 `-match`（**不区分大小写**）改为 **`-cmatch`** ——
> 原正则会把含 `ERROR` 的 CMake 缓存变量（`QJS_BUILD_WERROR: OFF` 等）全部命中、**占满 10 条配额**，
> 把真正的错误挤掉；`-cmatch` 只认 clang 的 `file.cpp(12,5): error:` / `error C2220:` / `fatal error:` 与 ninja 的 `FAILED:`。
> **教训（写进契约）**：CI 里「日志看不到」**不是日志不存在**；**先证明管道没被截断**，再谈编译错误。


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

### 5.2 H2 —— 预置用了 `--recurse-submodules`（**结论不变，理由按运维复核改正**）

**依据**：quickjs-ng v0.14.0 的 `.gitmodules` 内容（已核实）：

```
[submodule "test262"]
	path = test262
	url = https://github.com/tc39/test262.git
	shallow = true
	update = none
```

**只有一个 submodule `test262`，且 `update = none`** → **构建根本不需要它**
（它是 JS 引擎的测试套件；`cutils.h` / `libunicode.c` / `libunicode-table.h` / `gen/` 都已 vendored 在主仓里）。
其 CMakeLists 全文**无任何下载步骤**（无 `curl`/`wget`/`ExternalProject`/`FetchContent`），
`unicode_gen` 是 `EXCLUDE_FROM_ALL` → **构建期不联网** ✓

→ **预置 clone 里的 `--recurse-submodules` 应当去掉**，浅克隆即可
（`--branch v0.14.0` + `--depth 1`）→ 拿到的就是**完整可编译**的树。

> ⚠️ **措辞订正（2026-11-13，运维复核）**：此前本节写的理由是
> 「递归会去克隆一个体量极大的 tc39/test262」—— **这个理由不成立**：
> `update = none` 的含义是 git **不会去更新**该子模块，所以 `--recurse-submodules` **也不会**拉它。
> **正确理由是**：v0.14.0 **根本没有构建必需的子模块** → 递归纯属多余。
> （结论不变：不带 `--recurse-submodules`。）

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
> **另外两条要在 M1 前定**：clippy 的收敛期限（§4.1）、revision 戳记的**断言**（§4.2 —— 产出方已定：Rust 侧）。

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
#   - core-ui.dll.revision 戳记 == revision 文件     ✅ 产出方已定（Rust 侧，§4.2）｜断言待加
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

- `spec/ffi-core-ui.md` —— 动态加载、版本协商、revision 戳记（§5.5）、页面加载（§4.3.2）、`.uix` 五条硬约束（§4.3.3）、页面重建生命周期（§4.6）
- `spec/crate-layout.md` —— crate 划分、依赖方向（S9/S10 的对象）、`.uix` 资产约束
- `spec/milestones.md` —— M0~M3 与本契约的对应（哪条门禁在哪个里程碑启用）
- `docs/ci.md` —— 运维的实现说明
- `docs/05-COREUI-ABI-NOTES.md` —— U01~U12 未核实项
- `docs/05-COREUI-EXPORTS.txt` —— 绑定清单与 S7'/S8 的权威来源
- `docs/12-UIX-TAG-FIX.md` —— 17 个 token 终表（S13 的依据）
