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

### job `rust`
| 步骤 | 作用 | 验证状态 |
|---|---|---|
| checkout → `path: app` | 拉本仓库 | ✅ 已实跑 |
| 读 `packaging/zsui-revision.txt` | 取钉住的 ref | ✅ 已实跑 |
| checkout `qiu7824/zsui` → `path: zsui` | 满足 path 依赖（MPL-2.0） | ✅ 已实跑 |
| `dtolnay/rust-toolchain@1.99.0` | 钉 Rust 版本 | ✅ 已实跑 |
| `cargo test --workspace --release --locked` | 全量测试 | ✅ 已实跑 |
| `cargo build --locked --release -p zsdesk-app` | 构建 full | ✅ 已实跑 |
| `cargo clippy` | 静态检查，**不阻断** | ✅ 已实跑（上游未启用 clippy，告警清零后再改强制） |
| upload artifact | 传 exe 给 package job | ✅ 已实跑 |

### job `core-ui`
| 步骤 | 作用 | 验证状态 |
|---|---|---|
| checkout → `path: app` | 仅为读 revision | ✅ 已实跑 |
| checkout `ghboke/core-ui` → `path: core-ui` | 取钉住的 revision | ✅ 已实跑 |
| **Locate clang-cl and llvm-rc** | 多候选探测 clang-cl；**llvm-rc 缺失即失败** | ✅ 已实跑 |
| Ensure Ninja | 缺失则 `python -m pip install ninja` | ✅ 已实跑 |
| **Pre-fetch QuickJS-NG** | 预置到 FetchContent 落点，消除联网不确定性 | ⬜ 本次新增 |
| Build core-ui from source | 调上游 `scripts/build-clang-cl.ps1 -Target core-ui` | ❌ **首次实跑失败** |
| Verify core-ui.dll is x86-64 | 读 PE header 断言 `Machine == 0x8664` | ⬜ 未跑到 |
| **Gate S7 — CRT 动态判定** | `dumpbin /dependents` 必须出现 `VCRUNTIME140.dll` + `MSVCP140.dll` | ⬜ 未跑到 |
| upload artifact | 传 dll 给 package job | ⬜ 未跑到 |

### job `package`
| 步骤 | 作用 | 验证状态 |
|---|---|---|
| checkout → `path: app` | 拿 LICENSE / Cargo.toml | ⬜ 未跑到 |
| download 两个 artifact | exe 与 core-ui.dll | ⬜ 未跑到 |
| **Gate S12** | `core-ui.dll` 必须与 exe 同目录（否则运行时可能命中 PATH 里的别的副本） | ⬜ 未跑到 |
| Collect VC++ x64 CRT | vswhere → `VC\Redist\MSVC\<ver>\x64\Microsoft.VC*.CRT` | ⬜ 未跑到 |
| Package ZIP | exe + `core-ui.dll` + 3 个 CRT + LICENSE + 逐文件 `bytes + SHA-256` manifest | ⬜ 未跑到 |
| upload artifact | `ZSDesk-full-x64` | ⬜ 未跑到 |

---

## 3. 第一次 CI 实跑结果（run `43387095246`，commit `d4b3bb5`）

**结果：失败 —— 12 个步骤里前 11 个全过，挂在第 12 步 `Build core-ui from source`。**

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
| `cargo test` 失败 | 上游 Rust 代码问题 | **与 core-ui 无关** |

---

## 6. 已验证 / 未验证分界

| 环节 | 状态 | 证据 |
|---|---|---|
| checkout + Rust 工具链 + `cargo test --workspace --release --locked` + 多 edition `cargo check` 矩阵 | ✅ **实跑通过** | fork 的 `Windows build #1`，commit `4a48349`，9m 58s，绿灯 |
| 收窄后的「仅 full edition」build / test / clippy | ✅ **实跑通过** | run `43387095246`，步骤 7/8/9 全绿 |
| clang-cl 探测 + Ninja 安装 | ✅ **实跑通过** | 同上，步骤 10/11 全绿 |
| **core-ui 从源码构建** | ❌ **实跑失败** | 同上，步骤 12 失败；**失败日志待取** |
| `core-ui.dll` 的 x64 断言 / Gate S7 / Gate S12 | ⬜ 未跑到 | 依赖上一步 |
| 便携 ZIP 打包 + `bytes + SHA-256` manifest | ⬜ 未跑到 | 依赖前两步 |
| QuickJS-NG 预置加固是否解决问题 | ⬜ **待验证** | 下一次 CI |

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
