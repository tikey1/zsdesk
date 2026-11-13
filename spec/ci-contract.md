# CI 与打包契约

> 契约编号：`ZC-CI-001`
> 状态：已定（随 D10 生效）
> 依据：上游 `qiu7824/zsdesk` 的 `.github/workflows/build.yml`（实测规范）+ `research/facts.md` §7.4 / §7.6
> **CI 现状（已实测）**：fork 的 Actions **早已启用**，并在 commit `4a48349` 跑绿过 `Windows build #1`（9m58s）→ **上游那套「checkout + Rust 1.99 + `cargo test --workspace --release --locked`」链路已验证可用**，本契约在其上增量扩展，**不重写**。
> **本机无 cargo / rustc / cmake / MSVC**（实测）→ **一切编译与运行验证只能靠 CI**。

---

## 1. 三条设计前提

1. **Rust 侧不能依赖 core-ui**（D09 的直接推论）。
   `cargo test --workspace --release --locked` 必须在**没有 core-ui.dll、没有 core-ui 源码**的环境里跑通。
   → CI 拆成**独立 job**：`rust` job 与 `core-ui` job **互不依赖**。
2. **core-ui 自己没有构建工作流**（实测：core-ui 只有 `deploy-pages.yml`）。
   → core-ui 的构建步骤由**我们**维护，落在 `.github/workflows/build.yml` 里。
3. **产物必须可复现、可校验**：便携 ZIP 里要能证明「这份 `core-ui.dll` 就是 pinned revision 构建的那份」。

---

## 2. 工作流文件清单

| 文件 | 触发 | 职责 |
|---|---|---|
| `.github/workflows/build.yml` | `push` / `pull_request` / `workflow_dispatch` | **主工作流**：3 个 job（`rust` / `core-ui` / `package`）+ 全部机械门禁 |
| `.github/workflows/release.yml` | `push: tags: v*` | 便携 ZIP 发布（`workflow_call` 复用 `build.yml` 的 job，不重复写） |
| `scripts/build-coreui.ps1` | — | 调 core-ui 上游的 `scripts/build-clang-cl.ps1`，并做产物校验 |
| `scripts/check-coreui-dll.ps1` | — | **门禁 S7**：PE 位数 / CRT 模式 / 导出面 / 版本号 / revision 戳记 |
| `scripts/check-coreui-api.py` | — | **门禁 S8**：Rust 绑定声明 vs `docs/05-COREUI-EXPORTS.txt` 的 `ui_*` 集合 |
| `scripts/check-crate-dag.py` | — | **门禁 S9 / S10**：crate 依赖无环 + FFI 层无业务依赖 |
| `scripts/package-portable.ps1` | — | 便携 ZIP 组装 + 逐文件 `bytes + SHA-256` manifest（沿用上游 `scripts/package.ps1` 的清单机制） |

> **门禁不放独立 workflow**（`gates.yml` 不新建）：上游只有一个 `build.yml`，
> 门禁分散到两个文件会让「哪条失败对应哪次改动」变难追溯。全部并进 `build.yml` 的
> `rust` job 末尾，失败即停。

---

## 3. `build.yml` —— 步骤级契约

### 3.0 顶层（**沿用上游，逐字保留前三项**）

```yaml
name: Windows build

on:
  push:
  pull_request:
  workflow_dispatch:

permissions:
  contents: read

env:
  # —— 沿用上游 zsdesk build.yml ——
  CARGO_INCREMENTAL: '0'
  CARGO_PROFILE_DEV_DEBUG: '0'
  CARGO_PROFILE_TEST_DEBUG: '0'
  # —— 本仓新增 ——
  CARGO_PROFILE_RELEASE_DEBUG: '0'
```

> **新增第四项的理由**：上游只有前三项。`CARGO_PROFILE_RELEASE_DEBUG=0` 让 release
> 产物与本地 `cargo build --release` 完全一致，避免「CI 绿、本地行为不同」。
> 代价：release 崩溃栈不带行号 —— 但本仓 release 崩溃靠 `tracing` 日志定位，不依赖调试符号。

### 3.1 Job `rust` —— Rust 编译 / 测试 / 门禁（**不依赖 core-ui**）

```yaml
  rust:
    name: Rust 编译 / 测试 / 门禁
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v7
        with:
          path: zsdesk-coreui

      - uses: dtolnay/rust-toolchain@1.99.0

      - name: 读取 core-ui revision（仅供门禁比对，不参与构建）
        id: dependency
        shell: pwsh
        run: '"revision=$((Get-Content -LiteralPath zsdesk-coreui/packaging/core-ui-revision.txt -Raw).Trim())" >> $env:GITHUB_OUTPUT'

      - name: 测试
        working-directory: zsdesk-coreui
        run: cargo test --workspace --release --locked

      - name: 构建 full edition（D05：唯一交付形态）
        working-directory: zsdesk-coreui
        shell: pwsh
        run: |
          cargo build --locked --release -p zc-app --features network,desktop
          if ($LASTEXITCODE) { exit $LASTEXITCODE }
          cargo clippy --locked --release -p zc-app --features network,desktop -- -D warnings
          exit $LASTEXITCODE

      - name: 门禁 S9 / S10 —— crate 依赖方向
        working-directory: zsdesk-coreui
        run: python3 scripts/gates/check_crate_dag.py

      - name: 门禁 S11 —— core-ui 绑定面 vs 导出表
        working-directory: zsdesk-coreui
        run: python3 scripts/check-coreui-api.py --exports docs/05-COREUI-EXPORTS.txt

      - name: 门禁 G01/G03/G04/G08 —— 静态门禁
        working-directory: zsdesk-coreui
        run: python3 scripts/gates/verify.py --phase 3

      - name: 门禁 E 段 —— 工程门禁 P0
        working-directory: zsdesk-coreui
        run: python3 scripts/gates/gates_engineering.py --min-sev P0

      - name: 上传可执行文件
        uses: actions/upload-artifact@v4
        with:
          name: zc-app-full-x64
          path: zsdesk-coreui/target/release/zc-app.exe
```

> **D05 收窄说明**：上游 `build.yml` 的「Check all editions」与
> 「Test theme and desktop-only editions」两步**删除**，改为只对 `full`
> （`--features network,desktop`）跑 build / test / clippy。
> `--locked` 保留不动。

### 3.2 Job `core-ui` —— 从源码构建 core-ui（**不依赖 `rust` job**）

```yaml
  core-ui:
    name: core-ui 源码构建
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v7
        with:
          path: zsdesk-coreui

      - name: 读取 core-ui revision
        id: dependency
        shell: pwsh
        run: '"revision=$((Get-Content -LiteralPath zsdesk-coreui/packaging/core-ui-revision.txt -Raw).Trim())" >> $env:GITHUB_OUTPUT'

      - uses: actions/checkout@v7
        with:
          repository: ghboke/core-ui
          ref: ${{ steps.dependency.outputs.revision }}
          path: core-ui

      - name: 准备 Ninja
        run: choco install ninja -y --no-progress

      - name: 准备 LLVM（clang-cl + llvm-rc）
        shell: pwsh
        run: |
          $clang = Get-Command clang-cl.exe -ErrorAction SilentlyContinue
          $rc    = Get-Command llvm-rc.exe  -ErrorAction SilentlyContinue
          if (-not $clang -or -not $rc) {
            choco install llvm -y --no-progress
            $env:PATH = "C:\Program Files\LLVM\bin;$env:PATH"
            Add-Content -Path $env:GITHUB_PATH -Value "C:\Program Files\LLVM\bin"
            $clang = Get-Command clang-cl.exe -ErrorAction SilentlyContinue
            $rc    = Get-Command llvm-rc.exe  -ErrorAction SilentlyContinue
          }
          if (-not $clang) { throw "clang-cl.exe 缺失（core-ui 只支持 MSVC / clang-cl）" }
          if (-not $rc)    { throw "llvm-rc.exe 缺失（Windows SDK 的 rc.exe 10.0.26100 在 ninja 子进程会挂起，必须用 llvm-rc）" }
          "clang-cl: $($clang.Source)"  | Out-File -FilePath $env:GITHUB_ENV -Append -Encoding utf8
          "llvm-rc:  $($rc.Source)"     | Out-File -FilePath $env:GITHUB_ENV -Append -Encoding utf8

      - name: 构建 core-ui（上游脚本，-Target core-ui，动态 CRT /MD）
        working-directory: core-ui
        shell: pwsh
        run: |
          & ./scripts/build-clang-cl.ps1 -Target core-ui
          if ($LASTEXITCODE) { exit $LASTEXITCODE }

      - name: 门禁 S7 —— 产物校验（位数 / CRT / 导出面 / 版本）
        working-directory: zsdesk-coreui
        shell: pwsh
        run: |
          & ./scripts/check-coreui-dll.ps1 `
            -DllPath "core-ui/clang-cl-build/core-ui.dll" `
            -ExpectedRevision "${{ steps.dependency.outputs.revision }}" `
            -ExpectedExports "docs/05-COREUI-EXPORTS.txt"
          exit $LASTEXITCODE

      - name: 写 revision 戳记
        shell: pwsh
        run: |
          "${{ steps.dependency.outputs.revision }}" | Out-File `
            -FilePath "core-ui/clang-cl-build/core-ui.dll.revision" `
            -Encoding utf8 -NoNewline

      - uses: actions/upload-artifact@v4
        with:
          name: core-ui-dll
          path: |
            core-ui/clang-cl-build/core-ui.dll
            core-ui/clang-cl-build/core-ui.dll.revision
```

**关键点**

| 项 | 决定 | 理由 |
|---|---|---|
| 构建入口 | 上游 `scripts/build-clang-cl.ps1 -Target core-ui` | 不重写上游构建逻辑（D03）；上游脚本已内置 `llvm-rc` 替换 |
| CRT | **不加** `-StaticCrt`（保持 `UI_CORE_MSVC_STATIC_CRT=OFF` → `/MD`） | D00-5；与 Rust MSVC 默认一致，与用户已装的 v14 运行库一致 |
| 输出目录 | `core-ui/clang-cl-build/core-ui.dll` | 上游脚本固定输出到 `<repo>/clang-cl-build/`；Ninja 单配置 → DLL 直接在 build 根 |
| `llvm-rc` | **必须** | Windows SDK 的 `rc.exe` 10.0.26100 在 ninja 子进程会挂起（上游脚本注释原文） |
| 钉 revision | `actions/checkout` + `ref: <revision 文件内容>` | **沿用上游 zsui-revision.txt 的同一套机制**，只是换成 `core-ui-revision.txt` |
| 与 `rust` job 的关系 | **无 `needs:`** | 两个 job 互不依赖 —— 这正是 D09「Rust 侧不依赖 core-ui」的 CI 表达 |

### 3.3 Job `package` —— 便携 ZIP 组装（依赖前两个 job）

```yaml
  package:
    name: 便携包组装
    needs: [rust, core-ui]
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v7
        with:
          path: zsdesk-coreui

      - uses: actions/download-artifact@v4
        with:
          name: zc-app-full-x64
          path: stage/zc-app.exe
      - uses: actions/download-artifact@v4
        with:
          name: core-ui-dll
          path: stage/

      - name: 组装目录
        shell: pwsh
        run: |
          $stage = "stage"
          New-Item -ItemType Directory -Force -Path "$stage/ui"          | Out-Null
          Copy-Item "zsdesk-coreui/assets/*.uix" "$stage/ui/" -Force
          if (Test-Path "zsdesk-coreui/assets/assets") {
            Copy-Item "zsdesk-coreui/assets/assets" "$stage/ui/assets" -Recurse -Force
          }
          Copy-Item "zsdesk-coreui/assets/themes.json" "$stage/ui/" -Force
          Copy-Item "zsdesk-coreui/LICENSE" "$stage/" -Force
          Copy-Item "zsdesk-coreui/THIRD-PARTY-NOTICES.txt" "$stage/" -Force

      - name: 门禁 S12 —— 组装后自检（core-ui.dll 必须在 exe 同目录）
        shell: pwsh
        run: |
          & ./scripts/check-coreui-dll.ps1 `
            -DllPath "stage/core-ui.dll" `
            -ExpectedRevision "$(Get-Content zsdesk-coreui/packaging/core-ui-revision.txt -Raw).Trim()" `
            -ExpectedExports "zsdesk-coreui/docs/05-COREUI-EXPORTS.txt" `
            -RequireBesideExe "stage/zc-app.exe"
          exit $LASTEXITCODE

      - name: 生成 manifest + ZIP
        shell: pwsh
        run: |
          & ./scripts/package-portable.ps1 -StageDir "stage" -OutFile "ZSDesk-${{ github.sha }}-full-x64.zip"
          exit $LASTEXITCODE

      - uses: actions/upload-artifact@v4
        with:
          name: ZSDesk-full-x64
          path: ZSDesk-*-full-x64.zip
```

---

## 4. 门禁清单（每条都有执行器）

> 依据 `10-standards/mechanical-gates.md`：「没有执行器的规则，只是建议。」
> **失败即停**：门禁失败只有两条路 —— 修，或显式登记豁免（写清谁豁免 / 为什么 / 何时复审）。
> **禁止**调低阈值、加 ignore、用 `it.skip` 混过去。

| # | 门禁 | 检查什么 | 失败处置 | 执行命令 |
|---|---|---|---|---|
| G01 | emoji 扫描 | 源码 / `.uix` / 文档出现 emoji | 换语义图标 | `python3 scripts/gates/gates.py G01` |
| G03 | 单文件行数 | 任一 `.rs` > 300 行 | 拆分 | `python3 scripts/gates/gates.py G03` |
| G04 | 分层方向 | 底层 crate 反向依赖上层 | 修正 | `python3 scripts/gates/gates.py G04` |
| G08 | 图标库唯一 | 同时出现多套图标库 | 统一到 `ui/assets/icons/*.svg` | `python3 scripts/gates/gates.py G08` |
| E07 | 硬编码密钥 | 字面量 token / 密钥 | 走环境变量 | `python3 scripts/gates/gates_engineering.py E07` |
| E11 | 外部调用无超时 | 调外部进程 / COM 无超时 | 加超时 | `python3 scripts/gates/gates_engineering.py E11` |
| **S7** | **core-ui 产物校验** | PE machine = `0x8664`；导入表含 `VCRUNTIME140` / `MSVCP140`（= `/MD` 动态 CRT）；导出面与 `docs/05-COREUI-EXPORTS.txt` 一致；版本号 = `1.9.0.345` | 修正构建 | `pwsh scripts/check-coreui-dll.ps1 …` |
| **S8** | **绑定面 vs 导出表** | Rust 绑定层声明的 `ui_*` 集合 ⊆ `docs/05-COREUI-EXPORTS.txt` 的 567 个；且**必需符号**全部在集合内 | 改绑定 / 改 revision | `python3 scripts/check-coreui-api.py --exports docs/05-COREUI-EXPORTS.txt` |
| **S9** | **crate 依赖无环** | `cargo metadata` 解析的依赖图无环 | 修正 | `python3 scripts/gates/check_crate_dag.py` |
| **S10** | **FFI 层无业务依赖** | `zc-ffi-coreui/Cargo.toml` 无任何 `zc-*` 路径依赖 | 修正 | 同上 |
| **S11** | **可选符号声明正确** | 绑定层 `optional` 表里的符号必须**确实不在**必需表里；标为「本机 DLL 独有」或「仓库独有」的符号各有归属 | 修正 | 同 `check-coreui-api.py` |
| **S12** | **产物布局** | `core-ui.dll` 必须与 exe 同目录（否则会命中系统 PATH 里的别的 core-ui.dll） | 修正打包脚本 | `pwsh scripts/check-coreui-dll.ps1 -RequireBesideExe …` |

### 4.1 S7 的 CRT 检查怎么落地（为什么这条重要）

D00-5 要求三方 `/MD` 一致：Rust MSVC（默认 `/MD`）、clang-cl core-ui（`UI_CORE_MSVC_STATIC_CRT=OFF` → `/MD`）、用户本机已装的 v14 运行库。

**可执行的判据**：用 `dumpbin /dependents core-ui.dll` 看导入表。

| 导入表含 | 判定 |
|---|---|
| `VCRUNTIME140.dll` + `MSVCP140.dll` | ✅ **动态 CRT（`/MD`）** —— 与 Rust MSVC 和用户 v14 运行库一致 |
| **不含** 上述两者 | ❌ **静态 CRT（`/MT`）** —— 会与 Rust 侧的 CRT 分配器不匹配（跨模块 `malloc`/`free` 配对风险）→ **判失败** |

> 这是本仓唯一能**机器判定** CRT 一致性的手段，也是「C2 不引入 .NET 10、只依赖已装 v14 运行库」
> 这条约束的守门人。

### 4.2 S7 的导出面检查为什么要比对 `docs/05-COREUI-EXPORTS.txt`

**头文件不是完备契约**（ABI-NOTES 硬规则 1）。实测：

| 项 | 数量 |
|---|---|
| DLL 总导出 | 2006 |
| DLL 的 `ui_*` 纯 C API | **567** |
| 头文件声明的 `ui_*` | 557 |
| **DLL 比头文件多** | **12**（`ui_flush_gpu`、`ui_image_evict_tile`、`ui_image_view_gdi`、`ui_image_gdi_{set_file,set_pixels,set_parent,set_zoom,get_zoom,width,height,fit,clear}`） |
| **头文件比 DLL 多** | **2**（`ui_core_embed_binary`、`ui_dialog`） |

**但 CI 构建出来的 DLL 是从仓库 main 编的**，所以它必然：
- **有** `ui_core_embed_binary` 与 `ui_dialog`（仓库有）；
- **没有**那 12 个「本机独有」的符号（仓库的 `ui_api.cpp` 里有定义但头文件漏声明 —— V04 说「上游头文件漏同步声明，不是本机版本领先」）。

→ 由此得出一条**新的能力结论**（覆盖 ABI-NOTES 的 V04 表述）：

> **那 12 个符号在「仓库 main 编出来的 DLL」里是没有的**（虽然 `src/ui/ui_api.cpp` 有 `UI_API` 定义，
> 但 `UI_API` 在该构建配置下不导出，或导出名不同）。
> 因此 **CI 产物上 `image_gdi` 能力族必然为 `false`** → 我们的绑定层必须把它当**可选**能力，
> 且**不能**把它作为任何功能的必要路径。

S7 就是把这条结论变成机器检查：CI 编出的 DLL 的 `ui_*` 导出集合必须与
`docs/05-COREUI-EXPORTS.txt`（本机 DLL 的集合）**逐名比对**，差异项必须落在
「仓库有 / 本机无」或「本机有 / 仓库无」两张已知清单里，**出现第三类差异即失败**。

### 4.3 S8 的判定规则

```
必需符号（Api 表）⊆ 567 个 ui_* 集合      → 否则 FAIL（加载会失败）
可选符号（OptionalApi 表）⊆ 567 个 ui_* 集合 → 否则 FAIL（声明了一个不存在的符号）
「必需 ∩ 可选」= ∅                        → 否则 FAIL（同一符号两张表都写 = 意图不明）
必需符号里不得出现 12 个「本机独有」符号    → 否则 FAIL（CI 产物上不存在）
```

---

## 5. 打包契约

### 5.1 目录布局（便携包）

```
ZSDesk-<version>-full-x64/
├── zc-app.exe                       # 主程序
├── core-ui.dll                      # ★ 必须与 exe 同目录（D09 / 门禁 S12）
├── core-ui.dll.revision             # ★ revision 戳记（运行时比对，见 ffi-core-ui.md §5.5）
├── ui/
│   ├── main.uix
│   ├── preview.uix
│   ├── about.uix
│   ├── themes.json
│   └── assets/{icons/*.svg, theme.css, images/*}
├── LICENSE
├── THIRD-PARTY-NOTICES.txt
└── manifest.json                    # 逐文件 {path, bytes, sha256}
```

### 5.2 打包规则

| 规则 | 依据 |
|---|---|
| 产物名 `ZSDesk-<version>-full-x64.zip` | D05 |
| **只构建 / 只测试 / 只发布 `full`**（`--features network,desktop`） | D05 |
| **只 x64**，无 32 位产物 | D00-7 |
| 逐文件 `bytes + SHA-256` manifest | 沿用上游 `scripts/package.ps1` |
| 强制校验 Visual C++ x64 Redistributable（`msvcp140` / `vcruntime140` / `vcruntime140_1`） | 沿用上游 `package.ps1` |
| **不打包** `ghde.dll` / `guohe-view-thumb.dll` | D02：GuoheView 是外部程序，只做运行时调用 |
| **过渡期同时打包** `zsui` 相关内容（C4） | C4：过渡期保留双通道；替换完成后移除 |

### 5.3 运行时查找顺序（D09 的落地）

`zc-ffi-coreui` 的 `default_candidates()` 顺序：

1. `<exe 同目录>\core-ui.dll` —— **CI 产物的权威副本**
2. `<exe 同目录>\core-ui\core-ui.dll`
3. `<exe 同目录>\..\core-ui.dll`
4. `C:\Program Files\GuoheView\core-ui.dll` —— **兜底**

> **为什么不能靠 DLL 搜索路径**：链接期会产生静态 import 表 `core-ui.dll`，
> Windows 加载器按「exe 目录 → 系统目录 → PATH」找。若 exe 目录缺 DLL，
> 就会命中 PATH 里**用户自己装的**那份（`C:\Program Files\GuoheView\core-ui.dll`）——
> 那份**不受我们控制**，用户升级 GuoheView 就可能换掉 API 面。
> 动态加载让我们能**显式限定搜索路径**并在兜底时打警告 + 比对 revision 戳记。

---

## 6. 本地开发与 CI 的关系

| 场景 | 能不能本地做 | 怎么办 |
|---|---|---|
| 改 Rust 代码 + 跑单元测试 | ❌ 本机无 cargo | 只能推 CI（**本机 `cargo_run=false` 必须如实标注**，不得把「未验证」写成「已通过」） |
| 看 core-ui 的 API | ✅ | `docs/05-COREUI-EXPORTS.txt` + `docs/05-COREUI-ABI-NOTES.md` |
| 验证 `.uix` 语法 | ⚠️ 部分 | 标签表 `docs/11-COREUI-MARKUP-TAGS.md`；但 M01~M06 仍需真机 |
| 跑真机冒烟 | ❌ | 需要一台装了 core-ui + GuoheView 的 Windows 机（人工） |

---

## 7. 端到端验证（CI 必须能自证）

```powershell
# 1) Rust 侧（不需要 core-ui）
cargo test --workspace --release --locked
#    期望：全部 PASS；且测试过程中**没有**任何对 core-ui.dll 的文件访问

# 2) core-ui 侧
pwsh scripts/check-coreui-dll.ps1 -DllPath core-ui/clang-cl-build/core-ui.dll `
    -ExpectedRevision (Get-Content packaging/core-ui-revision.txt -Raw).Trim() `
    -ExpectedExports docs/05-COREUI-EXPORTS.txt
#    期望：位数 x64 / CRT 动态 / 导出面差异只落在两张已知清单内 / 版本 1.9.0.345

# 3) 产物侧
pwsh scripts/check-coreui-dll.ps1 -DllPath stage/core-ui.dll -RequireBesideExe stage/zc-app.exe
#    期望：core-ui.dll 与 exe 同目录，且 revision 戳记一致

# 4) 关键错误流（必须能判失败，不是静默降级）
#    把 core-ui.dll 移走 → 启动 zc-app.exe → 必须以非零码退出，
#    且 stderr 打印完整候选列表（不是白屏、不是静默回退到 GuoheView 那份）
```

---

## 8. out-of-scope

1. **不建** `gates.yml` 独立工作流（见 §2 的理由）。
2. **不做** macOS / Linux 交叉编译（core-ui 只支持 Windows）。
3. **不做** 多版本矩阵（`matrix.version`）—— 只跟 pinned revision。
4. **不做** 自动发布到 Release（`release.yml` 只在 tag 触发，且需要人工确认）。
5. **不引入** `cargo-dist` / `cross` 等第三方发布工具（沿用上游 `package.ps1`）。

---

## 9. 关联文档

- `spec/ffi-core-ui.md` —— 动态加载、版本协商、revision 戳记
- `spec/crate-layout.md` —— crate 划分、依赖方向、门禁清单
- `spec/milestones.md` —— M0~M3 与本契约的对应
- `docs/05-COREUI-ABI-NOTES.md` —— U01~U12 未核实项
