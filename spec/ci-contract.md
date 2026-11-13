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
| 9 | 用上游 `scripts/build-clang-cl.ps1 -Target core-ui`（不重写上游构建逻辑） | 已实现 | ✅ |
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
| 20 | **S7' 导出面**：CI 编出的 DLL 的 `ui_*` 集合 vs `docs/05-COREUI-EXPORTS.txt`，差异 ⊆ {`ui_core_embed_binary`, `ui_dialog`} | 未实现 | ⛔ 见 §5.3 |
| 21 | **S8 绑定面**：绑定层声明 ⊆ 实测导出集合；必需∩可选=∅；必需不含 12 个「本机独有」符号 | 未实现 | ⏳ **待绑定层落地后启用**（`spec/rust-ffi-skeleton/` 合并进 `crates/zsdesk-coreui/` 之后） |

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

### 4.3 ⛔ S7' 导出面（CI 编出的 DLL vs 导出表）

**为什么必须**：`Gate G2` 比的是**版本号**；但「同版本」不等于「同 API 面」。
上游头文件**漏声明**了 12 个本机实际导出的符号（`docs/05-COREUI-ABI-NOTES.md` V04），
说明**版本号与 API 面不是单调关系**。

**判定规则**（可机械执行）：

```
CI 编出的 core-ui.dll → 导出表 → 取 ui_* 集合 C
docs/05-COREUI-EXPORTS.txt → 取 ui_* 集合 L（本机 DLL，567 个）
断言：L ⊆ C                      （本机有的，CI 产物也必须有）
断言：C \ L ⊆ {ui_core_embed_binary, ui_dialog}   （仓库 main 多出来的只能是这两个）
断言：C \ L 之外没有第三类差异
```

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

### 4.6 一条**不属于 CI 但必须写进契约**的产品条款

**Everything 引擎不入包**（已定）→ 用户的机器上**可能没有 Everything**。

→ **产品契约**：Everything 是**可选后端**；不可用时搜索必须**自动回退**
（MFT / USN 索引 → 目录扫描），且 UI 上要**让用户知道当前用的是哪个后端**
（上游 `zsdesk-core` 已有 `SearchBackendPreference`，
`zsdesk-app/src/fast_search/everything_runtime.rs` 已有运行时探测）。
**不允许**因为 Everything 缺失就静默地「搜不出东西」。

→ **CI 侧对应冒烟项（M3）**：在**没有 Everything** 的 runner 上，搜索仍能返回结果。

---

## 5. 一句话结论

> **工作流主干与「产物可信」这一层（位数 / revision 版本 / 动态 CRT / 同目录 / assets 进包 /
> manifest 版本列 / 干净构建 / llvm-rc 硬失败 / QuickJS 预置）已经到位** ——
> 这是一次高质量的实现，架构侧逐条核对无误。
> **剩下的缺口集中在「机械门禁」这一层**（S7' 导出面 / S8 绑定面 / S9·S10 依赖方向 /
> 静态门禁 / S13 token），它们**依赖绑定层与门禁脚本先落地**，因此不是 CI 的问题，
> 而是**M1 的产出物**。
> **另外两条要在 M1 前定**：clippy 的收敛期限（§4.1）、revision 戳记的产出方（§4.2）。

---

## 6. 端到端验证（CI 必须能自证）

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

## 7. 本文件不做的事

1. **不重写** CI 实现 —— 实现归运维，契约归架构；本文件只给判定与缺口。
2. **不规定** 具体用 PowerShell / Python / bash 写哪个脚本 —— 只规定**判定什么**。
3. **不改** 上游 CI 已验证可用的部分（checkout 机制、Rust toolchain、`--locked`、env 三项）。
4. **不把目标态写成现状** —— 这是 v2 的教训：v2 引用的 4 个脚本在仓库里并不存在。

---

## 8. 关联文档

- `spec/ffi-core-ui.md` —— 动态加载、版本协商、revision 戳记（§5.5）、页面加载（§4.3.2）、`.uix` 三条硬约束（§4.3.3）
- `spec/crate-layout.md` —— crate 划分、依赖方向（S9/S10 的对象）、`.uix` 资产约束
- `spec/milestones.md` —— M0~M3 与本契约的对应（哪条门禁在哪个里程碑启用）
- `docs/ci.md` —— 运维的实现说明
- `docs/05-COREUI-ABI-NOTES.md` —— U01~U12 未核实项
- `docs/05-COREUI-EXPORTS.txt` —— 绑定清单与 S7'/S8 的权威来源
- `docs/12-UIX-TAG-FIX.md` —— 17 个 token 终表（S13 的依据）
