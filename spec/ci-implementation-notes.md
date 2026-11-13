# CI 实现评审与实现说明（`spec/ci-implementation-notes.md`）

> **本文档作者**：运维成员（**独立评审者**，不是 `build.yml` 的作者）
> **评审对象**：`.github/workflows/build.yml`
> **评审基线**：
> - 上游基线 = `qiu7824/zsdesk` 的 `.github/workflows/build.yml`（57 行，本仓库 `6822ec7` 引入，**逐字节未改**）
> - 被评审版本 = **工作区状态，2026-11-13 16:29:23，500 行，26186 字节，`sha256` 前缀 `59e1c5ca60de0f97`**
>   （含未提交改动：新增 `crates/zsdesk-coreui/` 绑定层、戳记产出方从 CI 改为 Rust `build.rs`、新增 core-ui 冒烟探针）
> - 已提交版本 = `7fbb30b`，481 行（差异见 §3 的 R5）
>
> ⚠️ **该文件在本次评审期间被并发修改过多次**（16:28:52 → 16:29:23 之间从 502 行变成 500 行，戳记策略从「缺失仅 warning」改回「缺失即阻断」）。
> 本文的行号以 **500 行 / `59e1c5ca…`** 为准。**复核方法**：
> ```bash
> python3 -c "import pathlib,hashlib;b=pathlib.Path('.github/workflows/build.yml').read_bytes();print(hashlib.sha256(b).hexdigest()[:16], len(b.decode('utf-8').splitlines()))"
> # 对不上 → 本文行号可能已漂移，但**结论与建议仍然成立**（问题清单按步骤名 + 代码特征描述，不依赖行号）
> ```
> **纪律**：本机**没有** Rust / C++ / MSVC 工具链（实测：无 `cargo` / `rustc` / `cmake` / `cl`）→
> **一切未经 CI 实跑或本机脚本可复现的结论，一律标「未验证」**。本文件不把「未验证」写成「已通过」。
> **本文件不修改 `build.yml` / `docs/ci.md`**（按交付总监要求），只给结论与建议。

---

## 0. 评审方法与本次实际执行的检查

下表是**本次评审真正跑过的检查**。凡未列入本表的内容，一律属于「读代码推断」或「未验证」。

| # | 检查 | 怎么做的 | 结果 |
|---|---|---|---|
| C1 | 16 个 `run:` 块的 PowerShell 语法 | `pyyaml` 解析 `build.yml` → 抽出每个 `run:` → 模拟 Actions 表达式插值 → `[Parser]::ParseInput` 逐块解析（UTF-8 显式解码） | **16/16 全部 PARSE-OK，0 语法错误** |
| C2 | 同上，但按 Windows PowerShell 5.1 的默认 ANSI 解码 | 同上，用 5.1 的默认文件编码 | `package` 步（唯一含中文字面量的步骤）**解析失败** → 见 R13 |
| C3 | YAML 整体可解析 | `yaml.safe_load` | 通过 |
| C4 | 上下游行数对照 | `git show 6822ec7:.github/workflows/build.yml` | 上游 57 行 → 现 502 行 |
| C5 | 表达式插值点枚举 | 脚本扫描 `${{ }}` | 4 处：`toolchain.outputs.path` / `.rc`、`quickjs.outputs.dir`、`crt.outputs.dir`、`dependency.outputs.revision` |
| C6 | `core-ui/VERSION` 与 FetchContent 声明 | 读上游 `core-ui` 副本 | `VERSION` = `1.9.0.345`；`FetchContent_Declare(quickjs … v0.14.0)` **仅此一处** |
| C7 | quickjs-ng v0.14.0 的 `.gitmodules` | 经 jsDelivr 镜像取（github.com 本机 TLS 不通） | **只有 `test262`，且 `update = none`** → 预置**不需要** `--recurse-submodules` ✅（架构师的结论成立） |
| C8 | quickjs-ng v0.14.0 根目录内容 | jsDelivr 文件树 API（159 个文件） | `cutils.h` / `libunicode.c` / `libunicode-table.h` / `gen/` **都在根目录**（vendored，非子模块）→ 浅克隆即可编译 ✅ |
| C9 | quickjs-ng v0.14.0 的 CMakeLists 是否有网络下载 | 取全文检索 `curl/wget/ExternalProject/FetchContent/add_custom` | **无任何下载步骤**；`unicode_gen` 是 `EXCLUDE_FROM_ALL` → 构建期不联网 ✅ |
| C10 | runner 镜像实际组件 | 读 `actions/runner-images` 的 `Windows2025-Readme.md` | LLVM 20.1.8、**Ninja 1.13.2**、CMake 3.31.6、VS Enterprise 2022 `17.14.37827.9`（`C:\Program Files\Microsoft Visual Studio\2022\Enterprise`）、Windows SDK 10.0.26100.0、Python 3.12.10 / pip 26.2.2、InnoSetup 6.7.1、**`Component.VisualStudio.Component.VC.Llvm.ClangToolset` 已装** |
| C11 | `packaging/licenses/` 文件扩展名 | `ls` 统计 | 240 个文件**全部是 `.txt`** → 打包步骤的 `*.txt` 过滤当前不丢文件 ✅ |
| C12 | `zsdesk.iss` 对 payload 的硬性要求 | 读 `packaging/windows/zsdesk.iss` | 第 37 行：`SourceDir\manifest.json` **必须存在** → 见 R6 |
| C13 | `crates/` 现状与 path 依赖 | 读各 `Cargo.toml` | `crates/zsdesk-ui:10` 与 `crates/zsdesk-app:21` 均有 `zsui = { path = "../../../zsui" }` → 见 **R0（P0）** |
| C14 | 新增 crate 的单元测试是否需要 DLL | 读 `crates/zsdesk-coreui/src/lib.rs:161-178` | 只有 2 个纯 UTF-16 字符串测试，**不加载 DLL** → `cargo test --workspace` 不会因缺 DLL 红 ✅ |
| C15 | `build.rs` 戳记路径推导 | 读 `crates/zsdesk-coreui/build.rs` | `OUT_DIR.ancestors().nth(3)` = `target/<profile>/` ✅ 与打包步骤读取路径一致；内容 = `revision + "\n"`（无 BOM）✅ 与契约 §4.2 字节级口径一致 |

---

## 1. 与上游 `build.yml` 的逐条差异清单

上游 57 行 → 现 502 行。**「依据」列给出可复核的文件与行号。**

| # | 改了什么 | 为什么 | 依据（文件:行） |
|---|---|---|---|
| **D1** | 触发器加 `workflow_dispatch` | core-ui 是 C++ 构建，失败时需要能手动重跑，不必等 push | `build.yml:6`；契约 ZC-CI-001 v3 |
| **D2** | **1 个 `windows` job → 3 个 job**：`rust` / `core-ui` / `package`（`package` 用 `needs: [rust, core-ui]`） | D09 决定 Rust 侧对 core-ui 是**运行时 `LoadLibrary`**、不链接 `core-ui.lib` → Rust 编译/测试完全不需要 core-ui 存在。拆开后 core-ui 的 C++ 失败**不拖红 Rust**，两类失败各自可定位 | `build.yml:26/77/339-343`；`DECISIONS.md` D09；`spec/ci-contract.md` §3.1 #2/#3 |
| **D3** | 工作区路径 `zsdesk/` → `app/`；新增 `zsui/`、`core-ui/` 两个**同级**目录 | Rust path 依赖 `../../../zsui` 从 `crates/<name>/` 出发上溯 3 级 = **仓库根的同级目录** → `app/`+`zsui/`+`core-ui/` 必须同级 | `crates/zsdesk-app/Cargo.toml:21`、`crates/zsdesk-ui/Cargo.toml:10`；`build.yml:30-44/81-94` |
| **D4** | **删除**上游 `Check all editions`（4 次 `cargo check` + 1 次 build）与 `Test theme and desktop-only editions`（2 次 `cargo test`）两步；改为只跑 `cargo build --locked --release -p zsdesk-app` | D05：唯一交付形态 = `full`（features `network + desktop` = default）。不做 32 位 x86，不构建 standard/network/desktop/themed/desktop-box | `build.yml:52-59`；`DECISIONS.md` D05；`crates/zsdesk-app/Cargo.toml` `[features] default = ["network","desktop"]`；契约 §3.1 #4 |
| **D5** | 新增 `Clippy (advisory)`，`continue-on-error: true` | 上游未启用 clippy。先当观察项看清告警量，M1 前转 `-D warnings` 阻断（契约 §4.1） | `build.yml:61-65`；契约 §4.1 |
| **D6** | 新增「上传 exe」artifact（`zsdesk-app-full-x64`） | `package` job 跨 job 取产物 | `build.yml:67-72` |
| **D7** | 新增整个 `core-ui` job：checkout `ghboke/core-ui` @ pinned revision → `ilammy/msvc-dev-cmd@v1` → clang-cl/llvm-rc 探测 → Ninja 兜底 → 清构建目录 → 预置 QuickJS → 直接调 cmake | D03：core-ui 上游**没有构建工作流**（只有 `deploy-pages.yml`），这套步骤必须我们自己维护 | `build.yml:77-337`；`research/facts.md` §7.8；契约 §3.2 |
| **D8** | QuickJS 预置机制：从「预置到 `_deps/quickjs-src`」改成「预置到 `$RUNNER_TEMP\quickjs` + `-DFETCHCONTENT_SOURCE_DIR_QUICKJS`」 | **预置到 `_deps/quickjs-src` 本身无效**：FetchContent 判「要不要下载」看的是 `_deps/quickjs-subbuild/…-populate-stamp/` 下的 stamp，不是看 `_deps/quickjs-src` 里有没有文件；CI 每次删掉整个 `clang-cl-build/`（正是为了避免 stale `CMakeCache.txt`）→ stamp 一并被删 → 重新 clone → 预置白做。官方覆盖变量会让 FetchContent **完全跳过 download step** | `build.yml:162-209`；`core-ui/CMakeLists.txt:40-52`；契约 §5 |
| **D9** | 新增 PE 头断言：`e_lfanew` → `Machine == 0x8664` | 交付物必须是 x86-64；`0x014C` = 32 位 | `build.yml:270-286`；契约 §3.3 #15 |
| **D10** | 新增 Gate G2：DLL `FileVersion` 必须 == `core-ui/VERSION` | 版本一致 ≠ API 面一致（上游头文件漏声明 12 个导出，见 `docs/05-COREUI-ABI-NOTES.md` V04），先锁版本 | `build.yml:289-305`；契约 §3.3 #16 |
| **D11** | revision 戳记：**已提交版**由 CI 写；**工作区版**改为由 Rust `build.rs` 写、CI 只校验（**缺失即阻断**） | 契约口径「甲：Rust 产出」；CI 再写一份就是两个来源，比对失去意义 | `build.yml:300-306`(HEAD) → 工作区已删；`build.yml:428-437`；`crates/zsdesk-coreui/build.rs`；契约 §4.2 / §4.6b |
| **D12** | 新增 Gate S7：`dumpbin /dependents` 必须含 `MSVCP140.dll` + `VCRUNTIME140.dll` | 机器判定 CRT 是动态 `/MD`（不能是 `/MT`），否则与 Rust 侧 `LNK2038 RuntimeLibrary mismatch` | `build.yml:307-326`；契约 §3.3 #18 |
| **D13** | 新增 Gate S12：`core-ui.dll` 必须与 exe 同目录 | 否则运行时可能命中 PATH 里的别的副本（`loader.rs` 搜索顺序第 4 档是 `C:\Program Files\GuoheView\core-ui.dll` 兜底） | `build.yml:361-368`；`crates/zsdesk-coreui/src/lib/loader.rs:389-437` |
| **D14** | 新增 `package` job：组装便携 ZIP | 交付物 | `build.yml:339-499` |
| **D15** | 逐文件 `bytes + SHA-256` manifest → `manifest.sha256`（格式 `hash␣rel␣bytes␣version`） | 沿用上游 `scripts/package.ps1` 的清单口径，但**换了文件名与格式** → 见 R6 | `build.yml:477-487`；`scripts/package.ps1:200-215` |
| **D16** | 强制收集并校验 VC++ x64 Redistributable（`msvcp140.dll` / `vcruntime140.dll` / `vcruntime140_1.dll`），找不到直接 throw | 与上游 `package.ps1:38-60` 同一套 vswhere 逻辑；缺运行库 = 目标机启动即失败 | `build.yml:370-395`；`scripts/package.ps1:38-60` |
| **D17** | **Everything 引擎不入包** | 契约 §3.4 #26；且 Rust 侧是「选择已安装的 Everything」的文件选择器（`crates/zsdesk-app/src/fast_search/everything.rs:46`），不自带不会让 full 版启动即失败 | `build.yml:417-493`；契约 §3.4 #26 |
| **D18** | 新增 `assets/` 入包 + 4 个文件逐个断言（`main.uix` / `preview.uix` / `about.uix` / `themes.json`） | 本 fork 特有需求：core-ui 没有运行时替换 CSS 变量的 C API，Rust 必须把 `:root` 拼进 `.uix` 源码 → 用 `ui_page_load_string` 运行时读文件。**缺了 `assets/`，交付的 exe 启动即失败** | `build.yml:440-452`；契约 §3.4 #22 |
| **D19** | 全局加 `GIT_TERMINAL_PROMPT=0` + `GIT_ASKPASS=echo` | 否则 git 遇认证提示会在 CI 上挂死到超时 | `build.yml:16-18` |
| **D20** | 新增「构建日志无论成败都上传」+ **逐行** `::error` annotation + step summary 尾部 | run 级日志 API 需要 admin 权限、外部取不到；多行 annotation 只显示首行（实测），必须逐行 | `build.yml:216-268` |
| **D21** | **（工作区新增）`package` job 里加 core-ui 冒烟探针**：`cargo build -p zsdesk-coreui --example probe` + `cargo run --example probe -- --dll … --uix assets/main.uix --id list_host` | 用**真实构建产物 + 仓库真实 `.uix` 资产**跑 U01~U03 最小闭环（加载 DLL → 初始化 → 解析 `.uix` → 开窗 → 按 id 取句柄） | `build.yml:400-415`；`crates/zsdesk-coreui/examples/probe.rs` |
| **D22** | **（工作区新增）戳记校验改为「缺失即阻断」** | M1 阶段 `crates/zsdesk-coreui/build.rs` 已落地 → 戳记必须由 Rust 侧产出；CI 缺失即 throw（`core-ui.dll.revision missing at … — crates/zsdesk-coreui/build.rs did not run`） | `build.yml:428-437`；`crates/zsdesk-coreui/build.rs` |

**未改动、必须保持不变的部分**（评审确认 ✅）：

| 项 | 状态 |
|---|---|
| `actions/checkout@v7` | 沿用上游 ✅（上游已在用，fork 跑绿过） |
| `dtolnay/rust-toolchain@1.99.0` | 沿用上游 ✅（版本钉死，不随镜像漂移） |
| `cargo test --workspace --release --locked` | **原样沿用**上游主测试命令 ✅ |
| `env: CARGO_INCREMENTAL / CARGO_PROFILE_DEV_DEBUG / CARGO_PROFILE_TEST_DEBUG = '0'` | 沿用上游 ✅ |
| `permissions: contents: read` | 沿用上游 ✅ |
| `cargo test --workspace` 主命令**未被改动**为 `-p zsdesk-app` | ✅ 上游是 workspace 全量，本 fork 保持全量 |

---

## 2. 未验证点清单（**已验证 vs 未验证的明确分界**）

### 2.1 ✅ 已有 CI 实跑证据

| 环节 | 证据 | 备注 |
|---|---|---|
| `actions/checkout@v7` + `qiu7824/zsui` 精确 ref checkout + `dtolnay/rust-toolchain@1.99.0` | fork 的 `Windows build #1`，commit `4a48349`，**9m 58s，绿灯** | 上游那套链路 ✅ |
| `cargo test --workspace --release --locked` | 同上 | ✅ |
| `cargo build --locked --release -p zsdesk-app`（**收窄后的仅 full**） | run `43387095246` 步骤 8 绿 | ✅ |
| `cargo clippy --locked --release -p zsdesk-app`（观察项） | run `43387095246` 步骤 9 绿 | ✅ |
| `Locate clang-cl and llvm-rc`（**含 llvm-rc 缺失即失败**） | run `43387095246` 步骤 10 绿 | ✅ **证明 `C:\Program Files\LLVM\bin\clang-cl.exe` 在当前镜像上成立** |
| `Ensure Ninja is available` | run `43387095246` 步骤 11 绿 | ✅（镜像自带 Ninja 1.13.2 → 走的是「已存在」分支，**pip 分支从未被执行过**） |
| `actions/checkout` core-ui（`ghboke/core-ui` @ pinned revision） | run `43387095246` 步骤 6 绿 | ✅ |

### 2.2 ⬜ 本次新增 / 改写，**尚未验证**

| 环节 | 状态 | 为什么重要 |
|---|---|---|
| **`core-ui` 从源码构建（新的 FETCHCONTENT 机制）** | ❌ **未验证**（上一次跑失败在旧机制上） | 这是 M0 的唯一阻塞点 |
| PE `Machine == 0x8664` 断言 | ⬜ 未跑到 | 依赖上一步 |
| Gate G2（DLL 版本 == `core-ui/VERSION` = `1.9.0.345`） | ⬜ 未跑到 | 同上 |
| Gate S7（dumpbin 判动态 CRT） | ⬜ 未跑到 | 同上 |
| Gate S12（DLL 与 exe 同目录） | ⬜ 未跑到 | 依赖 `package` job |
| 便携 ZIP 打包 + `manifest.sha256` | ⬜ 未跑到 | 同上 |
| CRT 收集（vswhere → `Microsoft.VC*.CRT`） | ⬜ 未跑到 | 同上 |
| **core-ui 冒烟探针（`cargo run --example probe`）** | ⬜ 未跑到 | **新增**，见 R0 / R12 |
| **戳记存在性断言（缺失即阻断）** | ⬜ 未跑到 | 依赖 `package` job 的 cargo 成功 → 见 R0 / R5 |
| **仅 full edition 的收敛（删掉 edition 矩阵）** | ✅ **已验证**（run `43387095246` 步骤 8 绿） | — |
| `ilammy/msvc-dev-cmd@v1` 导入 MSVC x64 环境 | ✅ 已验证（步骤 10/11 在其后执行且绿） | 但**「没有它能不能编」未单独验证** |
| `FETCHCONTENT_SOURCE_DIR_QUICKJS` 覆盖变量 | ⬜ 未验证 | CMake 官方机制，理论上完全跳过 download；**未实跑** |
| `$RUNNER_TEMP` 预置路径在删除 `clang-cl-build/` 之后仍存在 | ✅ **静态核对通过**（预置目录在构建目录之外，步骤顺序 Clean→Pre-fetch→Configure 正确） | — |

### 2.3 ⚠️ 本机静态核对已通过、但 **CI 路径未跑到**（不要当成「已通过」）

| 断言 | 本机核对方式 | CI 状态 |
|---|---|---|
| `core-ui/VERSION` 存在且为 `1.9.0.345` | 读上游 core-ui 副本（C6） | ⬜ CI 未跑到 |
| quickjs-ng v0.14.0 无构建必需子模块 | jsDelivr 镜像取 `.gitmodules`（C7/C8） | ⬜ CI 未跑到 |
| quickjs-ng v0.14.0 构建期不联网 | 读其 CMakeLists 全文（C9） | ⬜ CI 未跑到 |
| `packaging/licenses/` 240 个文件全是 `.txt` | `ls` 统计（C11） | ⬜ CI 未跑到 |
| 16 个 `run:` 块 PowerShell 语法正确 | AST 解析（C1） | ✅ 语法层面已验证（≠ 运行正确） |
| 新 crate 的单元测试不需要 DLL | 读源码（C14） | ⬜ CI 未跑到 |

### 2.4 ❗ 明确写清「不是已通过」的两件事

1. **core-ui 的 C++ 编译本身从未在 runner 上成功过。** 至今所有关于「core-ui 能在 runner 上从源码构建」的表述都只是**推断**：`CMakeLists.txt` 允许 clang-cl、`llvm-rc` 路径已确认、QuickJS 已预置 —— 但**一次都没有编出来过**。
2. **本机 `C:\Program Files\GuoheView\core-ui.dll` 是 `1.9.0.346`，而 pinned revision `1840d689` 是 `1.9.0.345`。** 因此：
   - Gate G2 的比对基准**必须**是「构建产物 vs `core-ui/VERSION`」，**不能**拿本机 DLL 的 346 当基线；
   - `docs/05-COREUI-EXPORTS.txt` 记录的 2006 个导出 / 567 个 `ui_*` 来自 **346** 的 DLL → **不可直接当作 345 构建的期望值**；契约 §3.3 #20 的 S7' 判据必须用 pinned revision 源码扫出的 567 符号集做基准。

---

## 3. 评审意见（问题清单）

> 严重度：**P0** = 会让下一次 CI 直接挂 / 产出错误交付物；**P1** = 不挂但削弱门禁或排障能力；**P2** = 健壮性 / 一致性。

### R0 【P0】`package` job 现在跑 cargo，却没有检出 `zsui/` → 工作区清单装载失败

- **位置**：`build.yml:400-415`（新增的 `Build the core-ui smoke probe` / `Smoke-probe …`）
- **事实**：
  - `crates/zsdesk-ui/Cargo.toml:10` 与 `crates/zsdesk-app/Cargo.toml:21` 都有 `zsui = { path = "../../../zsui" }` → **`zsui/` 必须与 `app/` 同级**。
  - `package` job（`build.yml:344`）**只 checkout 了 `app/`**，没有 `Read the pinned ZSUI revision` + `checkout qiu7824/zsui → path: zsui` 这两步（`rust` job 在 `build.yml:34-44` 有）。
  - `Cargo.lock` 里也含 `zsui` 这个 path package（`Cargo.lock:1096`）。
- **后果**：cargo 装载 workspace 时会解析**全部成员**的清单，`crates/zsdesk-app` / `crates/zsdesk-ui` 的 path 依赖 `../../../zsui` 找不到 →
  `error: failed to load manifest for workspace member …` / `failed to read …/zsui/Cargo.toml`
  → **冒烟探针步骤红**。（本机无 cargo，**该行为未在本机复现**；但 `zsui/` 缺失这件事是确定的事实。）
  → **且会连炸第二处**：戳记由 `build.rs` 在编译 `zsdesk-coreui` 时写出 → cargo 编不过 ⇒ 戳记不存在 ⇒ 打包步骤 throw「`core-ui.dll.revision missing … build.rs did not run`」（见 R5）。**同一根因、两处报错，第二处的报错信息会把人引向错误方向。**
- **建议**：把 `rust` job 的这两步原样复制到 `package` job（checkout `app/` 之后立刻做）：
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
  顺带：`core-ui` job 不跑 cargo，**不需要** zsui ✅（现状正确，别乱加）。
- **附注**：`zsdesk-coreui` 本身只依赖 `libloading`，**不依赖 zsui**；但 workspace 是整体装载的，所以仍然需要。

### R1 【P1】`Locate clang-cl` 的候选路径漏了 VS 自带 LLVM，失败信息不可操作

- **位置**：`build.yml:107-132`
- **事实**：
  - runner 镜像 README 明确列出 **`Component.VisualStudio.Component.VC.Llvm.ClangToolset`（17.14.36510.44）已安装** → VS 自带 LLVM 在 `C:\Program Files\Microsoft Visual Studio\2022\Enterprise\VC\Tools\Llvm\x64\bin\clang-cl.exe`（同目录有 `llvm-rc.exe`）。
  - 镜像另装独立 **LLVM 20.1.8**，但 **README 未写安装路径** → 工作流里写死的 `C:\Program Files\LLVM\bin\clang-cl.exe` 属于**未文档化约定**。它今天成立（run `43387095246` 步骤 10 绿 ✅），但镜像一改就断。
  - 失败信息 `"clang-cl not found. Install LLVM, or pass -ClangClPath to build-clang-cl.ps1."` —— **后半句已失效**：工作流现在**不再调用** `build-clang-cl.ps1`（改成直接调 cmake，`build.yml:204-212`）。操作者按提示去「传 `-ClangClPath`」根本无处可传。
- **建议**：
  1. 增加一条 vswhere 驱动的候选（与本工作流已有的 dumpbin / CRT 定位方式一致）：
     ```powershell
     $vsLlvm = Get-ChildItem -LiteralPath (Join-Path $installation 'VC\Tools\Llvm') -Recurse -Filter 'clang-cl.exe' -ErrorAction SilentlyContinue |
               Where-Object { $_.FullName -match 'x64\\bin' } | Select-Object -First 1
     ```
  2. 失败信息改成可操作版本：`"clang-cl not found on this runner image. Install LLVM (choco install llvm) or add the toolchain path to the candidate list in this step."`
  3. `llvm-rc` 的失败信息同样给动作：`"llvm-rc.exe not found next to <clang-cl>. The Windows SDK rc.exe (10.0.26100) hangs in ninja subprocesses; install an LLVM that ships llvm-rc."`

### R2 【P2】`Ensure Ninja` 的 pip 分支在当前镜像上是死代码，且缺 `choco` 退路

- **位置**：`build.yml:134-149`
- **事实**：镜像自带 **Ninja 1.13.2**（C10）→ `Get-Command ninja` 命中 → 整个 pip 分支**从未被执行过**（run `43387095246` 步骤 11 走的就是「已存在」分支）。
- **风险（镜像哪天不带 ninja 时）**：`python -m pip install --upgrade ninja` 装的是 PyPI 的 `ninja` wheel（自带二进制），落到 `C:\hostedtoolcache\windows\Python\3.12.10\Scripts` —— 该目录是否在 PATH 取决于 runner 的 PATH 组装；且 pip 26 在受限网络下可能失败。
- **建议**：pip 失败时退回 `choco install ninja -y`（`docs/ci.md` §5 已经写了这条例子，但**工作流里没有实现**），两条都失败才 throw。

### R3 【P1】CRT 收集与上游 `scripts/package.ps1:38-60` **基本一致**，有两处偏离

- **位置**：`build.yml:370-395` vs `scripts/package.ps1:38-60`
- **逐行核对结论**：

| 上游 `package.ps1` | CI 版 | 判定 |
|---|---|---|
| `:41` `vswhere` 路径 = `GetFolderPath('ProgramFilesX86')` | `:376` = `Join-Path ${env:ProgramFiles(x86)} …` | ✅ 等价（同一目录） |
| `:43` `-latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath` | `:378` **逐字相同** | ✅ 完全一致 |
| `:45` `VC\Redist\MSVC` | `:380` 同 | ✅ |
| `:47` 版本目录过滤 `^\d+\.\d+\.\d+$` + 按 `[version]` **倒序** + 拼 `x64` | `:382-385` 同 | ✅ 完全一致 |
| `:53` `Get-ChildItem -Directory -Filter 'Microsoft.VC*.CRT'` + 要求含 `msvcp140.dll` + 取第一个 | `:386-388` 同 | ✅ 完全一致 |
| `:40` **第一个候选 `$env:VCToolsRedistDir` → `x64`** | **没有** | ⚠️ **偏离 1**（CI 里该变量本就为空，影响 = 0，但口径不齐） |
| `:60` `Test-Path -LiteralPath … -PathType Leaf` 逐个校验三个 CRT | `:393` `Test-Path -LiteralPath …`（**无 `-PathType Leaf`**） | ⚠️ **偏离 2** → 一个**同名目录**会被误判为「文件存在」 |
| `:57` 找不到就 throw | `:391` 同 | ✅ 一致 |
| `:59-60` `$runtimeFiles = msvcp140 / vcruntime140 / vcruntime140_1` | `:392` 同三个 | ✅ 一致 |
| 上游允许调用方用 `-RuntimeDir` 指定 | CI **强制自己找**，找不到直接 throw | ✅ **CI 更严**，符合「缺 `-RuntimeDir` 直接 throw」的要求 |

- **建议**：补 `-PathType Leaf`；候选列表加上 `$env:VCToolsRedistDir` 以与上游逐字对齐。

### R4 【P1】`${{ }}` 表达式插值 —— **逐处核查结论**（交付总监点名要审的）

- **机制**：Actions 在 shell 启动前做**文本替换**，替换结果**原样插入脚本文本**，不经过任何转义。
- **四处插值的核查结果**：

| 位置 | 表达式 | 引号 | 值形态 | 结论 |
|---|---|---|---|---|
| `build.yml:197` | `steps.toolchain.outputs.path` | **双引号** | `C:\Program Files\LLVM\bin\clang-cl.exe` | ⚠️ 含空格但在双引号内 → 语法安全。Windows 路径不可能含 `"` → 不会破引号。**但**若路径含 `$` 或 `` ` ``，PowerShell 会二次展开（当前路径不含 → 实际安全，**非结构性安全**） |
| `build.yml:198` | `steps.toolchain.outputs.rc` | **双引号** | `C:\Program Files\LLVM\bin\llvm-rc.exe` | 同上 |
| `build.yml:199` | `steps.quickjs.outputs.dir` | **双引号** | `D:\a\_temp\quickjs` | ✅ **反斜杠安全**（特意核对，避免误报）：CMake 的反斜杠转义发生在**参数字面量解析期**，变量展开在其后 → `set(X "${FETCHCONTENT_SOURCE_DIR_QUICKJS}")` 展开进来的是原样反斜杠，**不会**触发 `Invalid escape sequence` |
| `build.yml:454` | `steps.crt.outputs.dir` | **单引号** | `C:\Program Files\Microsoft Visual Studio\2022\Enterprise\VC\Redist\MSVC\14.44.35211\x64\Microsoft.VC143.CRT` | ✅ **最稳的一处**：单引号内 `$`、`` ` `` 均不展开；含空格与 `(x86)` 之类括号也安全（括号只在未加引号时才有语法意义） |
| `build.yml:304`（HEAD） | `steps.dependency.outputs.revision` | 双引号 | 40 位 hex | ✅ 安全（工作区已删除该行） |

- **结构性建议**：改用 `env:` 传递，彻底消除「文本替换进脚本」这一整类风险：
  ```yaml
  env:
    CLANG_CL:  ${{ steps.toolchain.outputs.path }}
    LLVM_RC:   ${{ steps.toolchain.outputs.rc }}
    QUICKJS_DIR: ${{ steps.quickjs.outputs.dir }}
  ```
  脚本里改用 `$clang = $env:CLANG_CL`。值不经过脚本解析器 → 路径里任何字符都安全。
- **附带**：`GITHUB_OUTPUT` 的 `dir=$crtDir` 单行格式 ✅ 没问题（路径含空格是允许的）；但 `dir=` 后面如果路径本身含 `=` 也不影响（只有第一处 `=` 是分隔符）✅。

### R5 【P1】revision 戳记：**已提交版是「自证」，工作区版已改成 Rust 产出 + 缺失即阻断**

- **已提交版（`7fbb30b`）的问题**：CI 从 `packaging/core-ui-revision.txt` 读 SHA 写进戳记（`build.yml:300-306`），打包时再断言「戳记 == `packaging/core-ui-revision.txt`」（`build.yml:409-416`）→ **同源比同源，恒真**。它证明不了「这份 DLL 是从该 revision 构建的」。
- **契约要求**（`spec/ci-contract.md` §4.2 / §4.6b）：戳记由 Rust `crates/zsdesk-coreui/build.rs` 产出（口径甲），CI 只校验。
- **工作区已修正** ✅：CI 不再写戳记；`build.rs` 写 `target/<profile>/core-ui.dll.revision`（内容 = `revision + "\n"`，无 BOM —— 与契约 §4.2 的字节级口径**一致**）；打包步骤改为**缺失即阻断**：
  ```
  if (-not (Test-Path -LiteralPath $stampSrc)) {
    throw "core-ui.dll.revision missing at $stampSrc — crates/zsdesk-coreui/build.rs did not run"
  }
  ```
- **⚠️ 这个改动让 R0 变成双重关键**：现在 **`package` job 里有两件事依赖 cargo 成功**——① 冒烟探针；② 戳记的存在。而戳记是 `build.rs` 在**编译 `zsdesk-coreui` 时**写的 → `cargo build -p zsdesk-coreui` 失败 ⇒ 戳记不存在 ⇒ 打包步骤 throw。**同一个根因（`zsui/` 未检出）会连炸两处**，且第二处的报错信息（"build.rs did not run"）会把人引向错误方向。
- **仍需确认的三点**：
  1. `crates/zsdesk-coreui/` 目前是**未跟踪的新目录**，必须与 `Cargo.toml` / `Cargo.lock` / `build.yml` **同一次提交**推上去 —— 否则 CI checkout 到的 workspace 里没有这个 crate，`cargo build -p zsdesk-coreui` 直接失败。
  2. `build.rs` 写戳记在 `OUT_DIR` **之外**（`target/<profile>/`），cargo **不知道**这个产物 → 若 build.rs 被缓存复用而戳记文件被删，不会自动补写 → 会误报「build.rs did not run」。**加了 cargo 缓存后要专门验这一条**（见 R14b）。
  3. 戳记路径推导 `OUT_DIR.ancestors().nth(3)` = `target/<profile>/` ✅（本机核对 C15）。但它假定 `OUT_DIR` 深度为 4 段 —— cargo 改了目录布局就会写错位置。建议改成从 `CARGO_TARGET_DIR` + `PROFILE` 显式拼（更稳）。
- **建议**：把戳记升级为**带产物哈希**的格式（`sha256(core-ui.dll)␣revision`），打包时断言 `sha256(包内 core-ui.dll) == 戳记里的 sha256` → 这才是真正的完整性断言；纯 revision 戳记只能证明「版本号对得上」，不能证明「文件没被换过」。

### R6 【P1】manifest **不回读复验**、格式与上游不兼容 → CI 产物喂不进上游安装脚本

- **位置**：`build.yml:477-493`
- **上游 `scripts/package.ps1` 的三道保险，CI 版全丢**：
  1. `Assert-Payload $stage $manifest` → **逐文件回读复验**（bytes + sha256 都对一遍）
  2. `[IO.Compression.ZipFile]::CreateFromDirectory` + **校验 ZIP 条目数 == 文件数 + 1**
  3. 清单文件名与格式 = **`manifest.json`**（JSON：`{product,version,edition,files:{rel:{bytes,sha256}}}`）
- **CI 版**：只生成 `manifest.sha256`（文本），**不回读**、**不校验 ZIP 条目数**。
- **后果 1**：`packaging/windows/zsdesk.iss:37` 硬性要求 `SourceDir\manifest.json` 存在 → **CI 产出的 payload 不能喂给上游安装脚本**。CI 本身不构建安装包（不阻塞 CI），但「CI 产物 == 上游 verified payload」这句话**不成立**。
- **后果 2**：manifest 与 ZIP 内容可能不一致而无人发现。
- **建议**：① 额外输出一份上游格式的 `manifest.json`；② 打包后加 ZIP 条目数断言；③ 打包前加一次 manifest 回读复验（= 上游 `Assert-Payload` 的等价实现）。
- **附带**：`build.yml:481` 的 `if ($f.FullName -eq $manifest) { continue }` 是**死代码**，且分隔符口径不一致（`$stage` 由 `Join-Path $env:GITHUB_WORKSPACE "stage/ZSDesk-…"` 拼出 → 混合分隔符；`$f.FullName` 是反斜杠形式）→ 该比较**永远为 false**。当前无害（manifest 是最后才写盘，枚举时它还不存在），建议直接删掉或改成 `-replace '\\','/'` 后比较。

### R7 【P1】`if ($LASTEXITCODE)` 系列判断在 pwsh ≥ 7.4 上是**死代码**

- **位置**：`build.yml:58`（cargo build）、`:142`（pip）、`:180`（git clone）、`:210`/`:212`（cmake）、`:415`（cargo run）
- **事实链**：
  - GitHub 对 `pwsh` 内建 shell **前置** `$ErrorActionPreference = 'stop'`（GitHub 官方 workflow-syntax 文档明写）。
  - PowerShell **7.4 起** `$PSNativeCommandUseErrorActionPreference` **默认 `$true`**（7.4 GA release notes：该实验特性转正并置 true）。
  - 两者叠加 → **原生命令返回非 0 会先抛终止性错误** → 后面的 `if ($LASTEXITCODE)` **永远不执行**。
- **结论**：**不会误绿**（先抛错 → 步骤红 ✅），但：
  - `:210`/`:212` 精心写的 `"cmake configure failed ($LASTEXITCODE)"` 永远不会出现 → annotation 里只剩 `NativeCommandExitException`，**看不到 cmake 的退出码**；
  - `:212` 的失败分支靠 `catch` 兜住（`$failed = $true` → 最后 `throw`）✅ 仍然 fail-closed；
  - 但 `*>&1 | Tee-Object` 在抛错时**可能截断日志** —— 而日志是这次排障的**唯一证据来源**（run 级日志 API 需要 admin）。
- **建议**：在每个含原生命令的步骤开头显式写
  ```powershell
  $PSNativeCommandUseErrorActionPreference = $false
  ```
  让 `if ($LASTEXITCODE)` 按作者预期生效 → 退出码进 annotation、日志不截断、错误文案可控。

### R8 【P2】`ilammy/msvc-dev-cmd@v1` 未钉 commit SHA

- **位置**：`build.yml:99-101`
- **理由**：上游用的是自带的 `cmd.exe /c "set … && vcvars64.bat >nul && set"` 环境块抓取（`core-ui/scripts/build-clang-cl.ps1`）。改用 action 的方向是合理的（脚本不透传额外 `-D`，且自带环境块抓取在 CI 上不稳），但**未钉 SHA** → 供应链面，且该 action 的行为变化会直接决定 core-ui 能不能编。
- **建议**：钉到具体 commit SHA，并在注释里写明「为什么不直接用上游 `build-clang-cl.ps1`」（不透传 `-D` 是主因）。

### R9 【P1】打包出来的 exe **从未被验证身份**（上游 `Get-BuildInfo` 整个丢了）

- **位置**：`build.yml:417-493`
- **上游有**（`scripts/package.ps1:92-111`）：`Get-BuildInfo` ① 校验 exe 的**版本资源** == workspace version（`0.3.11`）；② **实际运行** exe 的 `--build-info`，解析 JSON 校验 `product=ZSDesk` / `edition` / `network` / `desktop` / `themed`。
- **CI 版没有** → 打错 edition、打错产物、或 exe 启动即崩，CI 都不会发现。
- **建议**：在 `Package the portable x64 zip` 里加一段（照抄上游逻辑）：
  ```powershell
  $exe = Join-Path $env:GITHUB_WORKSPACE 'app\target\release\zsdesk-app.exe'
  $bi = & $exe --build-info | ConvertFrom-Json
  if ($bi.product -ne 'ZSDesk' -or $bi.edition -ne 'full') { throw "unexpected build identity: $($bi | ConvertTo-Json -Compress)" }
  ```
  **注意**：exe 是 `windows_subsystem="windows"` 的 GUI 子系统程序，`--build-info` 走 stdout → 上游用 `ProcessStartInfo` + `RedirectStandardOutput` 读取；**在 PowerShell 里直接 `& $exe --build-info` 可能拿不到 stdout**（GUI 子系统无控制台）→ 必须照抄上游的 `ProcessStartInfo` 写法。

### R10 【P2】打包内容与上游 `package.ps1` 有四处差异，未在交付说明里声明

- **位置**：`build.yml:417-493`
- **差异清单**：
  1. **Everything 引擎不入包**（上游对非 `desktop-box` 版**强制要求** `-EverythingDir` 并入包 + 复制 pin 的 `manifest.json`）
  2. **Listary 辅助脚本不入包**（上游会 stage `native/listary/Prepare-Listary7.ps1` + `README.md`）
  3. **`manifest.json` → `manifest.sha256`**（见 R6）
  4. **`LICENSE` 文件名**：上游拷成 `LICENSE.txt`，CI 用 `LICENSE`
- **事实核对**：Rust 侧 `crates/zsdesk-app/src/fast_search/everything.rs:46` 是「选择已安装的 Everything」的文件选择器 → **不自带 Everything 不会让 full 版启动即失败**，但用户要自己装 Everything 才能用网络版搜索。契约 §3.4 #26 明确「不入包」→ **与契约一致**，只是交付说明里没写。
- **建议**：在 `docs/ci.md` 与交付说明书里逐条写出这四条差异，别让「CI 产物 == 上游 verified payload」的印象继续存在。

### R11 【P2】`licenses/*.txt` 的静默失败

- **位置**：`build.yml:465`
- **事实**：本机核对 `packaging/licenses/` **240 个文件全部是 `.txt`** ✅ → 当前不丢文件。但 `-ErrorAction SilentlyContinue` 会把将来出现的非 `.txt` 文件**静默丢掉**且不报错。
- **建议**：去掉 `-ErrorAction SilentlyContinue`，或改成整目录 `-Recurse` 复制（与上游 `package.ps1` 的 `Copy-Item -LiteralPath (Join-Path $root 'packaging\licenses') … -Recurse` 一致）。

### R12 【P2】冒烟探针把 **GUI 依赖**引进了 CI（新增步骤的风险）

- **位置**：`build.yml:406-415`；`crates/zsdesk-coreui/examples/probe.rs`
- **事实**：探针会 `ui_page_open_window(page, NULL)` → 在 runner 上真的创建一个窗口（D2D 渲染）。Windows hosted runner 跑在 session 0（有 window station 与默认 desktop）→ 窗口创建通常可行，D2D 有 WARP 软件回退。探针本身写得很好（用 `ui_page_root`/`ui_page_last_error` 判失败，而不是不可靠的「句柄 == 0」；结束时 `page_destroy` + `shutdown` → 不会挂住）✅
- **风险**：窗口创建在 CI 上失败 → `package` job 红，但**失败原因与 core-ui 本身无关**。未验证。
- **建议**：若探针在 CI 上不稳，把「加载 DLL + 解析 `.uix`」设为**阻断**，「开窗 + 取 id」降为 **warning**（`::warning::`），避免 GUI 环境问题卡住交付流水线。

### R13 【P2】`shell: pwsh` 是**承重**的 —— 换成 `powershell` 会直接语法错

- **位置**：`build.yml:417-493`（唯一含中文字面量的步骤）
- **事实**：C1/C2 实测 —— 同一份脚本，按 UTF-8 显式解码 → 解析通过；按 Windows PowerShell 5.1 的默认（无 BOM → ANSI）解码 → **解析失败**（`package` 步，第 54 行）。
  原因：`说明.txt` 的中文写在 `run:` 块里，脚本文件由 runner 以 UTF-8 落盘；pwsh 7 默认按 UTF-8 读无 BOM 脚本 ✅，而 `powershell.exe` 5.1 按 ANSI 读 → 乱码 → 语法错。
- **建议**：保持 `shell: pwsh`；更稳的做法是**把中文文案移出 `run:` 块**（放到仓库里的一个文本文件，CI 只 `Copy-Item`），这样将来换 shell 或换 runner 也不会炸。

### R14 【P2】其它低优先项

| # | 位置 | 问题 | 建议 |
|---|---|---|---|
| R14a | `build.yml:11-18` | 未设 `CARGO_PROFILE_RELEASE_DEBUG: '0'`（上游 `package.ps1:85` 设了）→ release 产物带调试符号，包更大 | 对齐上游设上；契约 §3.1 #7 已标 🟡 低优先 |
| R14b | 全局 | **无 cargo 缓存** → 每次从零编译，9m58s 的基线基本都花在这里 | 按知识库 `ci-cd-pipeline.md`（缓存 key 必须含锁文件哈希）加 `actions/cache`，key 含 `hashFiles('**/Cargo.lock')`。不改变门禁语义，只省时间 |
| R14c | `build.yml:327-335` | core-ui artifact 的 `path` 仍列着 `core-ui/clang-cl-build/core-ui.dll.revision`，但**戳记已改由 Rust `build.rs` 产出**，这个路径**在 `core-ui` job 里不再存在** → upload-artifact v4 只会匹配到 `core-ui.dll`（**不会**因此失败，因为「至少匹配到一个文件」），但会留一条令人困惑的「路径不存在」警告 | 把 `path` 里的 `core-ui.dll.revision` 一行删掉（戳记由 `package` job 自己的 cargo 构建产出，不走这个 artifact）；或保留但加注释说明「保留以兼容 M0 旧产物」 |

---

## 4. 三处探测逻辑的专项核对（交付总监点名）

### (a) `Locate clang-cl`（`build.yml:103-132`）

| 核查项 | 结论 |
|---|---|
| 候选是否覆盖 `windows-latest` 实际镜像 | **部分覆盖**。独立 LLVM 20.1.8 存在但**安装路径未在镜像 README 中写明** → 工作流写死的 `C:\Program Files\LLVM\bin\clang-cl.exe` 今天成立（run `43387095246` 步骤 10 绿 ✅）但属未文档化约定。**漏了 VS 自带 LLVM**（`Component.VisualStudio.Component.VC.Llvm.ClangToolset` 明确已装 → `…\VC\Tools\Llvm\x64\bin\clang-cl.exe`） |
| `Get-Command clang-cl` 分支 | 有效但**不应作为唯一兜底** —— 镜像是否把 LLVM\bin 加进 PATH 随镜像版本变 |
| 失败信息可操作性 | ❌ **不可操作**：提示「pass -ClangClPath to build-clang-cl.ps1」，但工作流**已不再调用该脚本** |
| `llvm-rc` 缺失即失败 | ✅ **正确且必要**（Windows SDK `rc.exe` 10.0.26100 在 ninja 子进程会挂起）。但失败信息也该给动作 |
| 修法 | 见 R1：加 vswhere 驱动的 VS-LLVM 候选 + 改写失败文案 |

### (b) `Ensure Ninja`（`build.yml:134-149`）

| 核查项 | 结论 |
|---|---|
| `pip install ninja` 在 runner 上可靠吗 | **在本镜像上不可靠也用不上** —— 镜像自带 Ninja 1.13.2，步骤会在第一行 `exit 0`，pip 分支**从未被执行**（run `43387095246` 步骤 11 绿 = 走的是「已存在」分支） |
| 需要退回 `choco` 吗 | **需要**。pip 装的是 PyPI wheel（自带二进制），落到 `…\Python\3.12.10\Scripts`，是否在 PATH 取决于 runner 的 PATH 组装；pip 26 在受限网络下也可能失败。`docs/ci.md` §5 已经写了「改用 `choco install ninja`」这条建议，但**工作流里没有实现** → 文档与实现不一致 |
| 修法 | 见 R2：pip 失败 → `choco install ninja -y` → 仍失败才 throw |

### (c) `Collect the Visual C++ x64 Redistributable`（`build.yml:370-395`）vs 上游 `scripts/package.ps1:39-59`

**逐行核对结果见 R3 的表格**，总结：

- **一致**：vswhere 路径与参数、`VC\Redist\MSVC`、版本目录正则 `^\d+\.\d+\.\d+$`、按 `[version]` 倒序、`x64`、`Microsoft.VC*.CRT` 过滤、要求含 `msvcp140.dll`、三个 CRT 文件名清单、找不到即 throw —— **全部逐字对齐** ✅
- **偏离 1**（`:40`）：上游的**第一个候选 `$env:VCToolsRedistDir`** 在 CI 版里没有 → 口径不齐（CI 里该变量为空，实际影响 0）
- **偏离 2**（`:60`）：上游用 `Test-Path -PathType Leaf`，CI 版（`:393`）**漏了 `-PathType Leaf`** → 同名目录会被误判为文件存在
- **CI 更严一处**：上游允许 `-RuntimeDir` 由调用方指定；CI 强制自己找，找不到直接 throw ✅ 符合「缺 `-RuntimeDir` 直接 throw」的要求
- **修法**：补 `-PathType Leaf`；候选加 `VCToolsRedistDir`

---

## 5. 文档漂移清单（不影响 CI，但会误导下一次排障）

| 文件:行 | 漂移 |
|---|---|
| `docs/ci.md:61` | 仍写「调上游 `scripts/build-clang-cl.ps1 -Target core-ui`」→ 工作区已改为**直接调 cmake**（`build.yml:204-212`） |
| `docs/ci.md:106` | 仍描述旧预置机制（`--recurse-submodules` → `_deps/quickjs-src`，且「该步骤 `continue-on-error`」）→ 现版本是 `$RUNNER_TEMP` + `FETCHCONTENT_SOURCE_DIR_QUICKJS`，**无** `continue-on-error` |
| `docs/ci.md:52-64` | core-ui 步骤表缺 `Clean the build directory` / `Gate G2` / 戳记三步；且「Build core-ui from source」标 ❌「首次实跑失败」—— 失败的是**旧机制**，新机制尚未验证，文案需要区分 |
| `docs/ci.md:120` | 「Ninja 通常有」→ 实测**确定有**（1.13.2，C10） |
| `spec/ci-contract.md:2` | 「当前 402 行 / 3 个 job」→ 评审时点为 500 行 |
| `spec/ci-contract.md` §2 当前态快照 | 仍写「Pre-fetch QuickJS-NG into the FetchContent slot ← 预置 `_deps/quickjs-src`」与「Build core-ui from source → 调上游脚本」→ **两处都已过时** |
| `spec/ci-contract.md` §3.2 #12 | 判「当前态 = 预置 `_deps/quickjs-src`」→ 已被 `FETCHCONTENT_SOURCE_DIR_QUICKJS` 取代（结论「CI 需预置源码」仍成立） |
| `spec/ci-contract.md` §3.2 #17 / §4.2 / §4.6b | 仍写「CI 现在还在自己写一份戳记 → 删掉它」→ **工作区已删** ✅ 契约可改为「已达成」 |
| `spec/ci-contract.md` §4.6b | 契约要求戳记内容 = SHA + **一个 `\n`**、无 BOM、只有一行；`build.rs` 写的正是 `revision + "\n"` ✅ **已对齐**（本机核对 C15） |
| `spec/ci-contract.md` §4.6b「⚠️ 前置条件」 | 契约写「CI 必须真的编到 `zsdesk-coreui`」→ 现工作区**已编到**（`build.yml:402`）✅，但**必须同时补上 `zsui/` 检出**（R0），否则编不到 |

---

## 6. 最可能让 CI 第一次跑挂的 1 个点

### 6.1 概率最高的那个：**core-ui 的 C++ 编译本身**（未验证的全新机制）

- 依据：core-ui 从未在 runner 上成功编出来过；新机制（`FETCHCONTENT_SOURCE_DIR_QUICKJS` + 直接调 cmake + `ilammy/msvc-dev-cmd` 导入环境）**一次都没跑过**。
- 已排除的雷（都有依据）：QuickJS 子模块（无构建必需子模块，C7/C8）、QuickJS 构建期联网（无，C9）、llvm-rc 缺失（已实跑验证存在）、Ninja 缺失（镜像自带）、FetchContent 覆盖变量语义（CMake 官方机制，但未实跑）。
- 仍未排除的雷：clang-cl 20.1.8 编译 quickjs-ng v0.14.0 + core-ui 源码的兼容性；`-G Ninja` 下 `MSVC` 变量是否为 1（`core-ui/CMakeLists.txt:22` 的 `if(NOT MSVC) FATAL_ERROR` —— clang-cl 会让 `MSVC=1` ✅ 但未实跑）；`link.exe` 能否被 `msvc-dev-cmd` 正确带入 PATH。
- **读日志的顺序**（沿用架构师的诊断表）：只看**第一条** `error:`；`CMake Error at …FetchContent` / `fatal:` → 下载路径；`LNK2038` → CRT 模式错（确认没传 `-StaticCrt`）；步骤长时间无输出 → `rc.exe` 挂起（确认 `CMAKE_RC_COMPILER` 真传进去了）；`ninja: build stopped` 前的 `FAILED:` → 记下 target。

### 6.2 唯一「确定性」的挂点（不依赖运气）：**R0 —— `package` job 缺 `zsui/` 检出**

- 如果 core-ui 这次编过了，**下一个必然红的就是它**：`package` job 新加的 `cargo build -p zsdesk-coreui` 会因为工作区里 `crates/zsdesk-app` / `crates/zsdesk-ui` 的 path 依赖 `../../../zsui` 找不到而失败。
- **而且它会连炸两处**：① 冒烟探针步骤（`build.yml:402`）；② 戳记存在性断言（`build.yml:428-431`，报错「`core-ui.dll.revision missing … build.rs did not run`」—— **这条报错会把人引向「build.rs 没跑」而不是「zsui 没检出」**）。
- 修法只需在 `package` job 补 2 步（见 R0 建议的 YAML 片段），**10 秒的工作量**。
- 顺带提醒：`crates/zsdesk-coreui/` 目前是**未跟踪**目录，必须与 `Cargo.toml` / `Cargo.lock` / `build.yml` **同一次提交**推上去，否则 CI checkout 到的 workspace 里根本没有这个 crate（那会是另一个 100% 必挂点）。

---

## 7. 放行建议

1. **必须先修（会让 CI 挂或产出错交付物）**：**R0**（package job 补 `zsui` 检出）、确认 `crates/zsdesk-coreui/` 与 `Cargo.toml`/`Cargo.lock` **同一次提交**。
2. **强烈建议先修（削弱门禁 / 影响排障）**：**R7**（`$PSNativeCommandUseErrorActionPreference = $false`，让退出码与日志都可用）、**R6**（manifest 回读复验 + ZIP 条目数断言 + 补一份上游格式 `manifest.json`）、**R9**（`Get-BuildInfo` 身份校验）。
3. **建议修（低成本）**：**R1**（clang-cl 候选 + 可操作失败文案）、**R3**（`-PathType Leaf` + `VCToolsRedistDir`）、**R8**（钉 action SHA）、**R2**（`choco` 退路）。
4. **与文档同步**：§5 的 8 处漂移。
5. **纪律提醒**：本次评审里所有关于 core-ui 编译结果的表述都是**未验证**。下一次 CI 跑完，请把「core-ui 编译成功 / 失败」这条**单独**回填到 §2.2 的表格里，并同步更新 `docs/ci.md` §6 的已验证/未验证分界表。**不要**把「workflow 跑完了」写成「core-ui 构建已通过」。

---

## 附：本评审未触碰的边界

- **没有**修改 `.github/workflows/build.yml`、`docs/ci.md`、`scripts/package.ps1`、`Cargo.toml`、`Cargo.lock`；
- **没有**修改 `crates/**` 下的任何 `.rs` 文件（只读了 `crates/zsdesk-coreui/` 的现有内容用于核对）；
- **没有**执行任何 Rust / C++ 构建（本机无工具链），§0 的 15 项检查全部是**静态解析 / 文件读取 / 远端镜像取源**，已在表中逐条标注做法。
