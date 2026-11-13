# CI 说明（GitHub Actions / Windows x64）

> 工作流：`.github/workflows/build.yml`
> 交付形态：**唯一 `full` edition + `windows-x64`**（features `network + desktop` = default）；**不做任何 32 位 x86**。
> 本文件与工作流由交付总监在运维成员因两次 RPC 网络超时未完成时代写，**未经本机或 CI 实跑验证的部分已逐条标注**。

---

## 1. 步骤总览

| # | 步骤 | 作用 | 依赖 | 验证状态 |
|---|---|---|---|---|
| 1 | `checkout` → `path: app` | 拉本仓库 | — | ✅ 已验证 |
| 2 | `Read the pinned upstream revisions` | 读 `packaging/zsui-revision.txt`、`packaging/core-ui-revision.txt` | — | ✅ 已验证 |
| 3 | `checkout` `qiu7824/zsui` → `path: zsui` | 满足 `../../../zsui` path 依赖（MPL-2.0） | — | ✅ 已验证 |
| 4 | `checkout` `ghboke/core-ui` → `path: core-ui` | 满足 `../../../core-ui` path 依赖 | — | ⬜ 本次新增 |
| 5 | `dtolnay/rust-toolchain@1.99.0` | 钉住 Rust 版本 | — | ✅ 已验证 |
| 6 | `cargo test --workspace --release --locked` | 全量测试 | Rust | ✅ 已验证 |
| 7 | `cargo build --locked --release -p zsdesk-app` | 构建 full edition | Rust | ⬜ 本次收窄后首次 |
| 8 | `cargo clippy`（advisory） | 静态检查，**不阻断** | Rust | ⬜ 本次新增 |
| 9 | `Locate clang-cl` | 多候选探测 clang-cl | LLVM | ⬜ 本次新增 |
| 10 | `Ensure Ninja is available` | ninja 缺失则 `pip install ninja` | Python | ⬜ 本次新增 |
| 11 | `Build core-ui from source` | 调上游 `build-clang-cl.ps1` | VS 工具链 + CMake + Ninja + LLVM | ⬜ **本次关键未知** |
| 12 | `Verify core-ui.dll is x86-64` | 读 PE header 断言 `Machine == 0x8664` | — | ⬜ 本次新增 |
| 13 | `Collect the Visual C++ x64 Redistributable` | vswhere → `VC\Redist\MSVC\<ver>\x64\Microsoft.VC*.CRT` | VS 2022 | ⬜ 本次新增 |
| 14 | `Package the portable x64 zip` | exe + `core-ui.dll` + 3 个 CRT + LICENSE + 逐文件 `bytes + SHA-256` manifest | — | ⬜ 本次新增 |
| 15 | `Upload the portable package` | `actions/upload-artifact@v4` | — | ⬜ 本次新增 |

**为什么 Rust 测试排在 core-ui 构建之前**：Rust 侧对 core-ui 采用**运行时 `LoadLibrary` 动态加载**（见 `spec/ffi-core-ui.md` §2），不链接 `core-ui.lib`。因此 core-ui 的 C++ 构建失败**不会**让 Rust 侧连测试都跑不了 —— 两类失败分开定位。

**CI 工作区布局（关键）**
```
<runner 工作区根>/
├── app/        ← 本仓库
├── zsui/       ← qiu7824/zsui，ref 取自 app/packaging/zsui-revision.txt
└── core-ui/    ← ghboke/core-ui，ref 取自 app/packaging/core-ui-revision.txt
```
Rust crate 里的 path 依赖是 `../../../xxx`（从 `crates/<name>/` 上溯三级），所以这三个目录必须同级。

---

## 2. runner 依赖

| 组件 | windows-latest 是否自带 | 缺失时的处理 |
|---|---|---|
| Visual Studio 2022（含 C++ x64 工具） | ✅ | 缺失则 CRT 收集步骤直接 throw |
| CMake | ✅ | — |
| Ninja | 通常有 | `python -m pip install --upgrade ninja` |
| LLVM / clang-cl | **不保证在 PATH** | 多候选探测（见 §5） |
| Python | ✅ | — |

**维护点**：`Locate clang-cl` 的候选路径写死了 `C:\Program Files\LLVM\bin\clang-cl.exe` 等。runner 镜像若变更，需同步维护这一步。

---

## 3. 本地复现（逐步等价命令）

```powershell
# 1. 准备同级目录
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

# 3. core-ui 侧（需 VS 开发者环境）
Set-Location ..\core-ui
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build-clang-cl.ps1 -Target core-ui -ClangClPath "C:\Program Files\LLVM\bin\clang-cl.exe"

# 4. 打包
Set-Location ..\app
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/package.ps1 -TargetDir target\release `
  -RuntimeDir "<VS Redist x64 CRT 目录>" -EverythingDir "<官方 Everything 1.4.1.1032 x64 目录>"
```

> ⚠️ **本机（开发用机器）没有 cargo / rustc / cmake / MSVC**（实测只有 git / curl / python3），因此**本地无法编译验证**。这正是 CI 必须存在的原因；任何未经实际执行的结论一律标「未验证」。

---

## 4. core-ui revision 的更新流程

1. 取目标 revision 的 40 位 SHA，写入 `packaging/core-ui-revision.txt`（只写 SHA，一行，末尾换行）。
2. 推 PR，让 CI 跑一遍 —— **不要**在本地验证过之前改这个文件（本机没有工具链）。
3. CI 绿后再合并。
4. 若 core-ui 上游改了导出面（头文件与 DLL 的缺口），必须同步更新 `docs/05-COREUI-EXPORTS.txt` 与 `docs/05-COREUI-ABI-NOTES.md`，并复核 `spec/ffi-core-ui.md` 的绑定清单 —— **运行时按符号探测，缺符号降级而不是崩溃**。

---

## 5. 常见失败点

| 症状 | 原因 | 处理 |
|---|---|---|
| `clang-cl not found` | runner 镜像没有 LLVM，或路径不在候选里 | 装 LLVM / 扩充 `Locate clang-cl` 的候选路径 |
| 构建卡在资源编译 | Windows SDK `rc.exe`（10.0.26100）在 ninja 子进程会挂起 | 上游脚本已自动用 clang-cl 同目录的 `llvm-rc.exe` 覆盖 `CMAKE_RC_COMPILER`，**不要关掉这个替换** |
| `core-ui.dll was not produced` | CMake 配置或编译失败 | 看 `Build core-ui from source` 步骤日志里的首个真实报错 |
| `core-ui.dll is not x86-64 (Machine=0x014C)` | 工具链目标成了 32 位 | 检查 `build-clang-cl.ps1` 的目标平台参数 |
| `LNK2038 RuntimeLibrary mismatch` | Rust 侧与 core-ui 的 CRT 选项不一致 | **两侧都必须 `/MD`**（core-ui 默认即 `/MD`；**不要**启用 `-StaticCrt`/`/MT`） |
| `ninja is still not on PATH` | pip 安装未生效 | 改用 `choco install ninja` |
| `Visual C++ x64 Redistributable directory not found` | runner 缺 VC++ 工具组件 | 检查 vswhere 的 `-requires` 条件 |
| `cargo test` 失败 | 上游 Rust 代码问题 | **与 core-ui 无关**，先看是不是既有问题 |

---

## 6. 已验证 / 未验证分界（诚实标注）

| 环节 | 状态 | 证据 |
|---|---|---|
| checkout + Rust 工具链 + `cargo test --workspace --release --locked` + 多 edition `cargo check` 矩阵 | ✅ **已实跑验证** | fork 的 `Windows build #1`，commit `4a48349`，9m 58s，绿灯 |
| 本次收窄后的「仅 full edition」build / test | ⬜ **未验证** | 第一次推送后才知 |
| **core-ui 从源码构建（clang-cl + Ninja + llvm-rc + CMake）** | ⬜ **未验证 —— 本次最关键的未知** | 同上 |
| `core-ui.dll` 的 x64 断言 | ⬜ 未验证 | 同上 |
| 便携 ZIP 打包 + `bytes + SHA-256` manifest | ⬜ 未验证 | 同上 |
| `cargo clippy` | ⬜ 未验证（**且刻意设为不阻断**） | 上游 CI 未启用 clippy，可能存在既有告警；待清零后再改为强制门禁 |

---

## 7. 发布前必做项（沿用用户既有工程规范）

1. **创建/合并 CI 前，先查 UI 设计代码是否正在更新**：
   - 是否有其它分支 / 开放中的 PR / 正在跑的 run
   - 是否有未提交的 UI 改动
   - 设计稿与已合入文档的哈希是否一致
   → 若设计代码正在更新，**等它完成并合并后再动 CI**。
   当前实测结论（2026-11-13）：仅 `main` 一个分支、无 PR、最近一次 run 为 `Windows build #1`（`4a48349`，绿）。
2. 确认仓库的 **Actions 已启用**（fork 的 Actions 默认是关闭的；本仓库已确认启用并跑绿过一次）。
3. 确认所有未经实际执行的结论都标了「未验证 / `cargo_run=false`」。
4. **云盘上传需用户逐次明确确认**，不得自动上传。
5. 推送优先用 **SSH**；PAT 只用于 API 且用完即删。

---

## 8. 交接说明

`.github/workflows/build.yml` 与本文件由**交付总监**代写 —— 原定由运维成员产出，但该成员在本会话中两次因 RPC 网络超时中断且未落盘。代写依据全部来自已落盘的实测底稿（`research/facts.md` §7.4 / §7.8 / §7.9、`docs/05-COREUI-EXPORTS.txt`），**不是凭印象写的**。

`spec/ci-implementation-notes.md` 由运维成员产出，内含**对上述实现的独立评审**（作者不得自审），评审意见若与实现冲突，以评审为准。
