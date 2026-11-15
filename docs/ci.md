# CI 说明（GitHub Actions / Windows x64）

> 工作流：`.github/workflows/build.yml`
> 交付形态：**唯一 `full` edition + `windows-x64`**（features `network + desktop` = default）；**不做任何 32 位 x86**。
> 本文件与工作流由交付总监在运维成员因两次 RPC 网络超时未完成时代写，**未经本机实跑验证的部分逐条标注**；CI 实跑结果见 §6。

---

## 1. 架构：三个互不依赖的 job

```
rust      ──┐
           ├──▶ package ──▶ ZSDesk-<version>-full-x64.zip
core-ui  ──┘
```

| job | 做什么 | 依赖 core-ui？ | 产出 |
|---|---|---|---|
| `rust` | 测试 + 构建 full edition + clippy（观察项） | **否** | `zsdesk-app.exe` |
| `core-ui` | 从钉住的 revision 从源码构建 core-ui | 否 | `core-ui.dll` |
| `package` | 收 CRT、打便携 ZIP、生成 manifest | 是（needs 两个） | ZIP artifact |

**为什么拆三个 job**：Rust 侧对 core-ui 采用**运行时 `LoadLibrary` 动态加载**（D09），不链接 `core-ui.lib`。所以 Rust 的编译与测试**完全不需要 core-ui 存在**。拆开后：

- core-ui 构建失败**不会**把 Rust 的绿拖红，两类失败各自独立定位；
- `package` 需要两者，用 `needs:` 显式表达依赖；
- `rust` job 只需 checkout `zsui`（`crates/<name>/Cargo.toml` 里 path 依赖是 `../../../zsui`），**不需要** checkout core-ui。

**CI 工作区布局（两个 job 各自独立）**
```
<runner 工作区根>/
├── app/        ← 本仓库（rust job 与 package job）
├── zsui/       ← qiu7824/zsui，ref 取自 app/packaging/zsui-revision.txt
└── core-ui/    ← ghboke/core-ui，ref 取自 app/packaging/core-ui-revision.txt（仅 core-ui job）
```

---

## 2. 步骤明细

> 验证状态以 **run `43666238286`（第 11 次 CI，commit `aa7e9b5`）** 为准；
> 「已实跑」= 在真实 CI 执行过，**不等同于通过**。标 ⏳ 的仍无真实结论。

### job `rust`（11 步）
| 步骤 | 作用 | 验证状态 |
|---|---|---|
| checkout → `path: app` + 读 pin + checkout `zsui` → `path: zsui` | 满足 `../../../zsui` path 依赖（MPL-2.0） | ✅ 已实跑 |
| `setup-python@v5`（3.11）+ `dtolnay/rust-toolchain@1.99.0` | 钉版本（D22） | ✅ 已实跑 |
| **Gate S14 — Rust module layout (pre-compile)** | 编译前机械复核 `mod` 声明 → 文件映射，提前拦 `error[E0583]` | 🆕 2026-11-15 新增、**未经 CI 验证**；本地正常态覆盖 6 crate root / 127 文件，两组负控精确命中 |
| `cargo test --workspace --release --locked` | 全量测试 | ❌ **第 9 / 10 / 11 次均失败**：第 9/10 次真因 = `Cargo.lock` 不同步（第 11 次已越过、`--locked` 不再报错）；**第 11 次真因 = `zsdesk-coreui` 5 个模块 `error[E0583]`**，已加 `#[path]` 修复，**待第 12 次 CI** |
| Gate S13（5 个门禁） | 标签 / token 命名空间 / id 清单 / 对比度 / 间距 | ⏳ **从未跑到**（Test 失败即跳过后续） |
| `cargo build --locked --release -p zsdesk-app` | 构建 full | ⏳ 从未跑到 |
| `cargo clippy` | 静态检查，**不阻断** | ⏳ 从未跑到 |
| upload artifact | 传 exe 给 package job | ⏳ 从未跑到 |

### job `core-ui`（16 步）
| 步骤 | 作用 | 验证状态 |
|---|---|---|
| checkout + Locate clang-cl/llvm-rc + Ensure Ninja + Pre-fetch QuickJS-NG | 工具链准备 | ✅ 第 9 / 10 次均成功 |
| Configure and build core-ui from source | 调上游 `scripts/build-clang-cl.ps1 -Target core-ui` | ✅ **连续两次 success**，日志 3057 行完整落盘 |
| Verify x86-64 / Gate G2 / Gate S7 | PE 位数 / revision 戳记 / 动态 CRT | ✅ 三次全绿 |
| **Gate S7'** | DLL 导出面 vs 同 checkout `src/` 的 `UI_API` 定义集 | ❌ 两次失败；真因 = `${{ env.GITHUB_WORKSPACE }}` 求值为空，已改用 `$env:GITHUB_WORKSPACE` + `Join-Path`，**待复验** |
| upload artifact | 传 dll 给 package job | ✅ 已实跑 |

### job `package`（13 步）
| 步骤 | 作用 | 验证状态 |
|---|---|---|
| checkout `app` + 读 pin + checkout `zsui` | 满足 workspace 整体解析（缺一不可） | ⏳ skipped（上游 job 失败连带） |
| download 两个 artifact | exe 与 `core-ui.dll` | ⏳ skipped |
| **Gate S12** | `core-ui.dll` 与 exe 同目录 | ⏳ 未跑到 |
| Collect VC++ x64 CRT | vswhere → `VC\Redist\MSVC\<ver>\x64\Microsoft.VC*.CRT` | ⏳ 未跑到 |
| **Build the core-ui smoke probe** | `cargo build -p zsdesk-coreui --example probe` | ⏳ 未跑到；**`probe.rs` 从未被 rustc 编译过**，是最大未知 |
| **Smoke-probe** | 用真实 DLL + `assets/main.uix` 跑 U01~U03 + `:root` 注入（U04） | ⏳ 未跑到 |
| Package ZIP | exe + DLL + 戳记校验 + `assets/` 四件套 + 3 个 CRT + LICENSE + `manifest.sha256` | ⏳ 未跑到 |
| Upload `ZSDesk-full-x64` | ZIP artifact | ⏳ 未跑到 |
| **Package job failure locator** | job 失败时把失败步骤名 emit 成 `::error::` | 🆕 2026-11-14 新增，**未经 CI 验证** |

---

## 3. 第一次 CI 实跑结果（run `43387095246`，commit `d4b3bb5`）

> ⚠️ **本节是 2026-11-13 的历史记录**，其中「core-ui 从源码构建失败」的结论
> **已被第 9 / 10 次 CI 推翻**（连续 success，见 §2 与 §6）。
> 保留原文仅用于追溯当时的加固决定，**不作为当前状态依据**。

**当时结果：失败 —— 12 个步骤里前 11 个全过，挂在第 12 步 `Build core-ui from source`。**

| 步骤 | 结果 |
|---|---|
| 1–6 setup / checkout / 读 pins / 拉 zsui / 拉 core-ui / Rust 工具链 | ✅ |
| 7 `cargo test --workspace --release --locked` | ✅ |
| 8 `cargo build -p zsdesk-app`（仅 full） | ✅ |
| 9 Clippy | ✅ |
| 10 Locate clang-cl | ✅ |
| 11 Ensure Ninja | ✅ |
| **12 Build core-ui from source** | ❌ |
| 13–16 x64 断言 / 收 CRT / 打包 / 上传 | skipped |

**结论：Rust 侧完全通，core-ui 的 C++ 编译本身是唯一阻塞点。**

**已知的最可能根因**：core-ui 的 `CMakeLists.txt` 用 `FetchContent` 从 GitHub 现场拉 QuickJS-NG：
```cmake
FetchContent_Declare(quickjs
    GIT_REPOSITORY https://github.com/quickjs-ng/quickjs.git
    GIT_TAG v0.14.0
    GIT_SHALLOW TRUE)
```
CI 上的雷点：① `GIT_SHALLOW` + tag 的浅克隆；② quickjs 的子模块；③ git 遇认证提示时在 CI 上**挂死到超时**。

**本次已做的加固**
1. 全局 `GIT_TERMINAL_PROMPT=0` + `GIT_ASKPASS=echo` → 认证场景**快速失败**而非挂死。
2. 新增 `Pre-fetch QuickJS-NG into the FetchContent slot`：用 `git clone --depth 1 --branch v0.14.0 --recurse-submodules` 预置到 `core-ui/clang-cl-build/_deps/quickjs-src`（CMake `FetchContent` 的默认落点），让 configure 直接复用、不再联网。**该步骤 `continue-on-error`**：预置失败也不阻断，让上游脚本自己走 FetchContent 下载路径。
3. `Locate clang-cl` 升级为 `Locate clang-cl and llvm-rc`：**llvm-rc 缺失直接失败**。理由见 §5。
4. 拆成三 job → core-ui 失败不再拖红 Rust，失败定位更快。

**仍然需要的证据**：失败步骤的原始日志。`/actions/jobs/{id}/logs` 需要仓库 admin 权限（未授权返回 403），`github.com` 网页在本机被网络策略拦截（CONNECT 502）——**只能由仓库所有者从 Actions 页面复制报错**。

---

## 4. runner 依赖

| 组件 | windows-latest 是否自带 | 缺失时的处理 |
|---|---|---|
| Visual Studio 2022（含 C++ x64 工具 + dumpbin） | ✅ | 缺失则 CRT 收集 / S7 直接 throw |
| CMake | ✅ | — |
| Ninja | 通常有 | `python -m pip install --upgrade ninja` |
| LLVM（clang-cl **且 llvm-rc**） | clang-cl 不保证在 PATH | 多候选探测；llvm-rc 缺失即失败 |
| Python | ✅ | — |
| git（能访问 github.com） | ✅ | runner 在 Azure 上，与本机的网络策略无关 |

**维护点**：`Locate clang-cl` 的候选路径写死了 `C:\Program Files\LLVM\bin\clang-cl.exe` 等。runner 镜像若变更，需同步维护这一步。

---

## 5. 常见失败点

| 症状 | 原因 | 处理 |
|---|---|---|
| `clang-cl not found` | runner 镜像没有 LLVM，或路径不在候选里 | 装 LLVM / 扩充候选路径 |
| `llvm-rc.exe not found next to ...` | 找到的 clang-cl 所在目录没有 llvm-rc | **必须装带 llvm-rc 的 LLVM**。没有它，上游脚本不会覆盖 `CMAKE_RC_COMPILER`，Windows SDK 的 `rc.exe`（10.0.26100）会在 ninja 子进程**挂起**（预处理了 `.rc` 却永远不写 `.res`） |
| `cmake configure failed` | FetchContent 拉 QuickJS-NG 失败 | 看日志首个真实报错；预置步骤的输出里能看出网络是否通 |
| `build failed` | C++ 编译错误 | 看 `Build core-ui from source` 步骤日志 |
| `core-ui.dll is not x86-64 (Machine=0x014C)` | 工具链目标成了 32 位 | 检查 `build-clang-cl.ps1` 的目标平台参数 |
| `core-ui.dll does not depend on VCRUNTIME140.dll` | CRT 不是动态 `/MD` | 两侧都必须 `/MD`；core-ui 默认即 `/MD`，**不要**启用 `-StaticCrt`/`/MT` |
| `ninja is still not on PATH` | pip 安装未生效 | 改用 `choco install ninja` |
| `Visual C++ x64 Redistributable directory not found` | runner 缺 VC++ 工具组件 | 检查 vswhere 的 `-requires` 条件 |
| `cargo test` 报 `cannot update the lock file ... --locked` | `Cargo.lock` 与 workspace manifests 不同步（本机无 cargo 时**手工追加 `[[package]]`** 最易触发：破坏排序、写错依赖名） | 不要手工拼 lock；CI 的 Test 失败分支会跑 `cargo metadata` 并把 `git diff -- Cargo.lock` 打进 annotations |
| Gate S7' 报 `core-ui.dll not found at \core-ui\...` | `${{ env.GITHUB_WORKSPACE }}` 求值为空 —— `GITHUB_WORKSPACE` 是 **shell 环境变量**，不是 Actions `env` 上下文的键 | 用 `$env:GITHUB_WORKSPACE` + `Join-Path`，不要用 `${{ env.GITHUB_WORKSPACE }}` |
| package job 失败但 annotations 为空 | 该 job 的 `actions/*` 步骤（checkout / download / upload）失败时不会 emit annotation | 查 job 末尾的 `Package job failure locator` 步骤，它会指认失败来自哪一类步骤 |
| `cargo test` 失败（其它） | 上游 Rust 代码问题 | **与 core-ui 无关** |

---

## 6. 已验证 / 未验证分界

> 以第 13 次 CI（run `43671423536`，commit `16066e6`）为止的实测为准；
> **第 14 次 CI 尚未触发**。第 13 次 cp1252 修复生效：Gate S14 与五个 S13 门禁
> **首次在 CI 执行并全部通过**；`Test` 因 `zsdesk-coreui` 首次被编译而报 5 个
> 类型/借用错误（已逐条修复，见 `docs/15` §0.5）。
>
> 第 12 次新增失败模式：Windows runner 的 Python stdout 默认 `cp1252`，
> 门禁打印中文会抛 `UnicodeEncodeError`。已在顶层 `env` 钉
> `PYTHONIOENCODING=utf-8` + `PYTHONUTF8=1`，六个门禁脚本另带自包含加固。
> **本机中文环境复验全绿不代表 CI 绿**，本地须额外用 `PYTHONIOENCODING=cp1252` 模拟。

| 环节 | 状态 | 证据 |
|---|---|---|
| fork 的基线 CI（commit `4a48349`，`Windows build #1`，9m58s） | ✅ 通过 | **上游基线**，尚未接入 core-ui；不代表当前项目 |
| checkout / 工具链 / `zsui` 同级检出 | ✅ 已实跑 | run `43398079661` / `43545192012` / `43666238286` |
| clang-cl + llvm-rc + Ninja + QuickJS-NG 预置 | ✅ 已实跑成功 | 同上 |
| **core-ui 从源码构建** | ✅ **第 9 / 10 / 11 次 CI 连续 success** | 日志 3057 行完整落盘；D21 退路已关闭 |
| `core-ui.dll` 的 x64 断言 / Gate G2 / Gate S7 | ✅ 三次全绿 | 同上 |
| **Gate S7' 对 CI 内 DLL 的导出面比对** | ✅ **第 11 次 CI = success** | `$env:GITHUB_WORKSPACE` + `Join-Path` 修复经真实 CI 证实；本机 346 DLL 的结论已在 CI 编出的 345 DLL 上实测确认 |
| `Cargo.lock` 与 workspace 一致性 | ✅ **第 11 次 CI 越过 `--locked`** | Test 未再报锁文件漂移，`cargo metadata` 自诊断分支也未触发 |
| `cargo test --workspace --release --locked` | ❌ **三次失败 + 第四次未跑到** | 第 9/10 次 = 锁文件（已关闭）；第 11 次 = E0583（已加 `#[path]` 修复）；**第 12 次被 S14 失败连带跳过**，待第 13 次 CI |
| `zsdesk-coreui` 5 个模块文件的编译 | ⏳ **从未被 rustc 编译** | 前 12 次 CI 都在更早环节失败；修完 E0583 后才轮到类型 / 借用检查 |
| 新 S14 Rust 模块布局门禁 | ❌ 第 12 次 CI 首次执行即失败 | 真因 = `cp1252` 编码崩溃（已全局修复）；本地 `PYTHONIOENCODING=cp1252` 复验通过 |
| 五个 S13 门禁 / `cargo build` / clippy | ⏳ **从未在 CI 跑到** | 第 12 次 S14 失败即跳过后续；这五个脚本同样打印中文，已一并加 UTF-8 加固 |
| Gate S12 / 收 CRT / smoke-probe / 便携 ZIP / `manifest.sha256` / 上传 | ⏳ **从未真正执行** | package job 四次均被上游失败连带 skipped |
| `probe.rs` 的类型检查 / 借用检查 / 实际运行 | ⏳ **从未编译** | 本机无 cargo/rustc，只能由 CI 回答 |
| 失败原因的公开可见性（D23） | ✅ 第 10 / 11 次 CI 已证明有效 | `::error::` annotations 先后定位「两个真因」与「5 条 E0583」；package job 的补齐仍**未经 CI 验证** |

---

## 7. 本地复现（逐步等价命令）

```powershell
# 1. 同级目录
git clone https://github.com/tikey1/zsdesk.git app
git clone https://github.com/qiu7824/zsui.git   zsui
git clone https://github.com/ghboke/core-ui.git core-ui
git -C zsui   checkout (Get-Content app/packaging/zsui-revision.txt   -Raw).Trim()
git -C core-ui checkout (Get-Content app/packaging/core-ui-revision.txt -Raw).Trim()

# 2. Rust 侧
rustup toolchain install 1.99.0
Set-Location app
$env:CARGO_PROFILE_DEV_DEBUG = '0'
$env:CARGO_PROFILE_TEST_DEBUG = '0'
cargo test --workspace --release --locked
cargo build --locked --release -p zsdesk-app

# 3. core-ui 侧（需 VS 开发者环境 + 带 llvm-rc 的 LLVM）
Set-Location ..\core-ui
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build-clang-cl.ps1 -Target core-ui -ClangClPath "C:\Program Files\LLVM\bin\clang-cl.exe"

# 4. 打包
Set-Location ..\app
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/package.ps1 -TargetDir target\release `
  -RuntimeDir "<VS Redist x64 CRT 目录>" -EverythingDir "<官方 Everything 1.4.1.1032 x64 目录>"
```

> ⚠️ **本机（开发用机器）没有 cargo / rustc / cmake / MSVC**（实测只有 git / curl / python3），因此**本地无法编译验证**。这正是 CI 必须存在的原因；任何未经实际执行的结论一律标「未验证」。

---

## 8. core-ui revision 的更新流程

1. 取目标 revision 的 40 位 SHA，写入 `packaging/core-ui-revision.txt`（只写 SHA，一行，末尾换行）。
2. 推 PR，让 CI 跑一遍 —— **不要**在本地验证过之前改这个文件。
3. CI 绿后再合并。
4. 若 core-ui 上游改了导出面（头文件与 DLL 的缺口），必须同步更新 `docs/05-COREUI-EXPORTS.txt` 与 `docs/05-COREUI-ABI-NOTES.md`，并复核 `spec/ffi-core-ui.md` 的绑定清单 —— **运行时按符号探测，缺符号降级而不是崩溃**。

---

## 9. 发布前必做项（沿用用户既有工程规范）

1. **创建/合并 CI 前，先查 UI 设计代码是否正在更新**：其它分支 / 开放中的 PR / 正在跑的 run / 未提交的 UI 改动 / 设计稿与已合入文档的哈希一致性 → 若设计代码正在更新，**等它完成并合并后再动 CI**。
   当前实测结论（2026-11-13）：仅 `main` 一个分支、无 PR。
2. 确认仓库的 **Actions 已启用**（fork 的 Actions 默认关闭；本仓库已确认启用并跑绿过）。
3. 确认所有未经实际执行的结论都标了「未验证 / `cargo_run=false`」。
4. **云盘上传需用户逐次明确确认**，不得自动上传。
5. 推送优先用 **SSH**；PAT 只用于 API 且用完即删。

---

## 10. 交接说明

`.github/workflows/build.yml` 与本文件由**交付总监**代写 —— 原定由运维成员产出，但该成员在本会话中两次因 RPC 网络超时中断且未落盘。代写依据全部来自已落盘的实测底稿（`research/facts.md` §7.4 / §7.8 / §7.9、`docs/05-COREUI-EXPORTS.txt`）与第一次 CI 的实跑结果（§3），**不是凭印象写的**。

`spec/ci-implementation-notes.md` 由运维成员产出，内含**对上述实现的独立评审**（作者不得自审），评审意见若与实现冲突，以评审为准。
