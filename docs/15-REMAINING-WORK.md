# 剩余工作流程（2026-11-13 收尾快照）

> 本文件记录**代码停止编写时**的未完成事项。
> 上一次已推送到 GitHub 的状态见 §1；本轮新增但**尚未验证**的改动见 §2；
> 后续里程碑见 §3；已知的判断与风险见 §4。

---

## 0. 一句话现状

Rust 侧的 core-ui 绑定层（`crates/zsdesk-coreui/`）与 CI 契约已经落地，
**六个机械门禁全部接进 CI**；
**core-ui 的 C++ 源码构建连续两次（第 9 / 10 次 CI）成功** —— M0 的最大未知已消除。
第 10 次 CI 靠本次装的 `::error::` 探针，把此前两处「看不到原因」的失败**全部定位到确切根因**：
`cargo test` 是 `Cargo.lock` 与 workspace 不同步，Gate S7' 是
`${{ env.GITHUB_WORKSPACE }}` 求值为空。两处均已在本地修好，
**待第 11 次 CI 复验**；package job 至今仍被上游失败连带跳过，尚未真正执行。

2026-11-14 又按成员复核结论修了三处（均**尚未推送、待第 11 次 CI**）：
`theme_contrast_check.py` 的缺/空资产静默 PASS（P0）、`probe.rs` 传给
`ui_page_load_string` 的字符串缺 NUL 结尾（P0）、package job 全 12 步零 `::error::`
（失败原因对外不可见）。详见 §1 表格末尾三行。

---

## 0.1 第 9 次 CI（run `43398079661`）实测结果

| job | 步骤 | 结果 | 结论 |
|---|---|---|---|
| core-ui | 10. Configure and build core-ui from source | **success** | ★ **clang-cl 路径成立**，前 4 次失败确实是日志管道被中断，不是编译错误 |
| core-ui | 12. Verify core-ui.dll is a Windows x86-64 image | success | 产物架构正确 |
| core-ui | 13. Gate G2 — the DLL version must match the pinned revision | success | 戳记链路正确 |
| core-ui | 14. Gate S7 — core-ui.dll must link the dynamic VC++ runtime | success | `/MD` 动态 CRT 正确 |
| core-ui | 15. Gate S7' — the DLL export surface must match the source contract | **failure** | 当时原因未定位 → **真因见 §0.2**（路径表达式空值） |
| Rust | 6. Test | **failure** | 当时误判为 doctest → **真因见 §0.2**（`Cargo.lock` 不同步） |
| package | 全部 | skipped | 上游 job 失败 ⇒ 未执行 |

**关键限制**：GitHub 的 job 日志下载端点
`GET /repos/{owner}/{repo}/actions/jobs/{job_id}/logs` 要求 admin（实测 403
`Must have admin rights to Repository.`），
**外部唯一可读的通道是 check-run annotations**（公开可读，已实测可取）。
⇒ **凡是会失败的步骤，都必须把失败原因打成 `::error::` annotation**，
否则失败原因对外完全不可见（本轮已对 `cargo test` / S13 门禁 / Gate S7' 三处补上）。

## 0.2 第 10 次 CI（run `43545192012`）—— 两个真因确认

| job | 步骤 | 结果 | annotation 探针给出的真实报错 |
|---|---|---|---|
| core-ui | 11. Configure and build core-ui from source | **success** | 日志 3057 行完整落盘 —— D17 修复**再次**证实有效 |
| core-ui | 12–15. PE / G2 / S7 | success | 产物架构、戳记、动态 CRT 全对 |
| core-ui | 16. Gate S7' | **failure** | `S7' FAILED: core-ui.dll not found at \core-ui\clang-cl-build\core-ui.dll` |
| Rust | 7. Test | **failure** | `error: cannot update the lock file ...\app\Cargo.lock because --locked was passed to prevent this` |
| package | 全部 | skipped | 上游失败连带跳过 |

**真因 A —— S7' 路径**（不是 Python 3.9，昨天那条推定**已被证伪**）：
`GITHUB_WORKSPACE` 是 **shell 环境变量**，不是 Actions 表达式里 `env` 上下文的键
⇒ `${{ env.GITHUB_WORKSPACE }}` 求值为 **空** ⇒ 四个参数全变成 `\core-ui\...` 这种
没有盘符的路径。同一文件里其余 20 多处都用 `$env:GITHUB_WORKSPACE`（PowerShell），一直是对的，
只有这一处用了表达式。修法：改用 `$ws = $env:GITHUB_WORKSPACE` + `Join-Path`。

**真因 B —— Cargo.lock**（不是 doctest；doctest 只是同一轮顺手修掉的确定性缺陷）：
上一轮把 `spec/rust-ffi-skeleton/` 升格为 `crates/zsdesk-coreui/` 时本机没有 cargo，
于是**手工**往 `Cargo.lock` 末尾追加了两个 `[[package]]` 条目：
① 破坏字母序；② 把 `libloading 0.8.9` 的依赖写成裸 `windows-sys`
（仓里有两个版本 ⇒ 歧义），而它的真实运行时依赖是 `cfg-if` + **`windows-link`**
（`windows-sys 0.61` 只是 dev-dependency，不进锁文件）。
⇒ cargo 判定锁文件需要重写，`--locked` 直接拒绝。
修法：依赖名改 `windows-link`、按 `(name, version)` 重排全部 127 个包、块间空行规范化；
tomllib 复验：127 包 / 有序 / 零歧义引用 / 零悬空引用。

**双保险**：本机无 Rust 工具链且缺 `zsui` path 依赖，无法本地跑 `cargo test` 验证锁文件。
故在 Test 的失败分支加了一条自诊断：若报错含 `--locked`，就让 CI 跑
`cargo metadata` 并把 `git diff -- Cargo.lock` 打进 annotations ⇒
万一本地修得不全，第 11 次 CI 会直接回传 cargo 期望的准确改动。

---

## 1. 已写入磁盘的改动与静态复核记录

> **口径（2026-11-14 更正）**：本表混合两类条目 —— ① 已随 `1a00121` 推送的历史项；
> ② 第 10 次 CI 之后在**本地**修好、`git status` 仍为 `M`、**尚未推送**的项
> （表格末尾三行已逐行标注）。
> 本机**无 cargo / rustc** 且缺 `zsui` path 依赖 ⇒ 本表所有「验证」都只是
> **静态 / 本机脚本级复核**，一律不代表「已通过编译或测试」；编译结论只能由第 11 次 CI 回答。
> 旧标题「已通过本地验证」措辞过强，已收敛。

| 项 | 文件 | 验证方式 |
|---|---|---|
| `:root` 注入模块（形态甲） | `crates/zsdesk-coreui/src/lib/theme.rs` | tree-sitter 语法零错误；文件内有 **13 个 `#[test]`**，但本机无 cargo/rustc，**这些单元测试从未执行过**（无「5 条等价逻辑用例」的落盘证据） |
| 注入路径进 CI 探针 | `crates/zsdesk-coreui/examples/probe.rs` | tree-sitter 语法零错误；**未编译**（`page_load_string` 的 NUL 结尾缺陷已于 2026-11-14 静态修正，待 CI 编译验证） |
| PE 导出表解析器 | `scripts/gates/pe_exports.py` | 用本机 `GuoheView\core-ui.dll` 实测 567 个 `ui_*`，与 `docs/05` **集合一致**；`diff` 因 Python stdout CRLF vs grep/sort LF **返回 1**，故**不是逐字节一致** |
| 导出面门禁 S7' | `scripts/gates/coreui_export_gate.py` | 正例通过；反例（截断 DLL / 无 `ui_*` 的 kernel32）均正确失败 |
| id 清单门禁 | `scripts/gates/ui_id_manifest_check.py` + `assets/ui-ids.json` | 本地通过；注入 typo 后正确失败 |
| CI 日志中断修复 | `.github/workflows/build.yml` | **第 9 次 CI 已证实有效**：core-ui 源码构建 success，日志完整落盘 3057 行，真实 clang warning 可见（不再是「恰好截断」） |
| CI 门禁接线（5 个 S13 门禁 + S7'） | `.github/workflows/build.yml` | 本地逐个跑通；**S7' 已在第 9 次 CI 里真正执行并失败** ⇒ 门禁确实有执行器，不是建议 |
| package job 补 zsui 检出 | `.github/workflows/build.yml` | 路径推导已核对；第 9 次 CI 因上游 job 失败被 skipped，**仍未实测** |
| 戳记来源收口 | `.github/workflows/build.yml` | core-ui job 不再上传 `.revision`，只由 Rust `build.rs` 产出 |
| doctest crate 名修正 | `crates/zsdesk-coreui/src/lib.rs` | `zc_ffi_coreui` → `zsdesk_coreui`（2 处）；`lib.rs` 内已清除，但**全仓并非零残留** —— 本行与 `DECISIONS.md` 第十三轮仍保留该串作为历史说明 |
| Python 版本钉死 + 3.9 兼容修正 | `.github/workflows/build.yml` + `scripts/gates/{pe_exports,coreui_export_gate,ui_id_manifest_check}.py` | 两个 job 各加 `setup-python@v5`（3.11）；ast 扫描确认全部门禁脚本 3.9 兼容 |
| 失败原因可视化（annotation 探针） | `.github/workflows/build.yml` | `cargo test` / S13 门禁 / Gate S7' 三处补 `::error::`，失败原因可经**公开** check-run annotations 读取；**第 10 次 CI 靠它一次定位两个真因** |
| `Cargo.lock` 规范化 | `Cargo.lock` | **【本轮本地修复，未推送】** `libloading` 依赖改 `cfg-if` + `windows-link`、127 包按 `(name,version)` 重排、块间空行规范；tomllib 复验：有序 / 零歧义 / 零悬空引用 |
| Gate S7' 路径修复 | `.github/workflows/build.yml` | **【本轮本地修复，未推送】** 4 个参数由 `${{ env.GITHUB_WORKSPACE }}`（求值为空）改用 `$env:GITHUB_WORKSPACE` + `Join-Path` |
| lock 漂移自诊断（双保险） | `.github/workflows/build.yml` | **【本轮本地修复，未推送】** Test 失败且报错含 `--locked` 时，跑 `cargo metadata` 并把 `git diff -- Cargo.lock` 打进 annotations |
| 门禁空过修复（P0） | `scripts/gates/theme_contrast_check.py` | **【2026-11-14 本轮，未推送】** 缺/空资产不再 `continue`，缺资产 / 0 字节 / 无 `<template>` / 零文字元素一律 FAIL；本地 4 组负控 EXIT 符合预期。顺带统一 G05 口径：先剥注释与 `<script>`，不再把注释里的说明文字当成渲染文字 |
| smoke-probe 字符串传参修复（P0） | `crates/zsdesk-coreui/examples/probe.rs` | **【2026-11-14 本轮，未推送】** `ui_page_load_string` 改传 NUL 结尾缓冲（含内嵌 NUL 守卫）；本机无 cargo，**未编译验证** |
| package job 失败可见性 | `.github/workflows/build.yml` | **【2026-11-14 本轮，未推送】** 5 个可失败步骤各加 `::error::` 捕获 + job 级兜底定位；YAML 解析通过、21 个 pwsh 块语法解析 0 错误 |
| 决策台账 D17–D20、第十三轮 | `DECISIONS.md` | — |

---

## 2. 待第 11 次 CI 复验的三件事

> 第 9 / 10 次 CI 已把 core-ui 构建、G2、S7 全部跑绿（§2.1 关闭）；
> 第 10 次 CI 又把另两处失败的确切根因暴露出来（§0.2）并在本地修复。
> 本节只列**仍未验证**的项；已证实的见 §1。

### 2.1 ✅ 已关闭：core-ui 的 C++ 构建

**run `43398079661` 的 core-ui job 第 10 步 = success**，随后的
「DLL 是 Windows x86-64 映像」/「Gate G2 戳记一致」/「Gate S7 动态 VC++ 运行时」全绿。
至此 **clang-cl + Ninja + llvm-rc 路径在我们 pin 的 revision 上成立**，M0 的最大未知已消除。

前 4 次失败的真实成因（留档，避免以后重踩）：

- 现象：日志都**恰好截断**在 `[72/97] ... binding.cpp.obj` 之后零输出；
  check-run 的 annotation 里全是 CMake 缓存变量，没有一条真正的 clang 报错。
- 根因：GitHub `shell: pwsh` 注入 `$ErrorActionPreference='stop'`，叠加 PowerShell 7.4+
  的 `$PSNativeCommandUseErrorActionPreference` 默认 `$true` ⇒ clang-cl 只要往 stderr
  写一行 warning 就抛 `NativeCommandExitException` ⇒ 管道中断 ⇒ `Tee-Object` 停止
  ⇒ **编译器诊断从未落盘**。
- 修法（已进 `build.yml`，已由第 9 次 CI 证实）：步骤开头设
  `$ErrorActionPreference='Continue'` + `$PSNativeCommandUseErrorActionPreference=$false`；
  annotation 提取正则 `-match` → **`-cmatch`**（前者不区分大小写，会把含 `ERROR` 的
  CMake 缓存变量全命中、占满 10 条配额）。

**保留的退路**：上游 core-ui 仓库**没有任何 Windows 构建 CI**（只有 docs 站点的
`deploy-pages.yml`）—— 「clang-cl 能编过」上游从未验证，现在只是**我们**验证过了。
若将来 pin 的 revision 变更导致该路径反复失败，可评估改用 VS 的 `cl.exe`，
或过渡期直接用本机 GuoheView 的定制 DLL。**当前不需要走退路。**

### 2.1b Rust job 的 `cargo test` —— 真因已定位、已修、待复验

第 10 次 CI 的真因是 **`Cargo.lock` 与 workspace 不同步**（详见 §0.2 真因 B），
**不是 doctest** —— doctest 的旧 crate 名只是同一轮里顺手修掉的确定性缺陷。
本地已把锁文件改成 cargo 规范形式：`libloading` 依赖 `cfg-if` + `windows-link`、
127 个包按 `(name, version)` 重排、块间空行规范化，tomllib 复验通过。
⚠️ **本机无 Rust 工具链且缺 `zsui` path 依赖 ⇒ 仍无法本地复跑 `cargo test`**：
「锁文件是否与 cargo 的解析结果完全一致」只能由第 11 次 CI 回答。
Test 的失败分支已加 lock 漂移自诊断（`cargo metadata` + `git diff -- Cargo.lock` → annotations）。
**第 9 / 10 次 CI 都在这一步失败，后面的 S13 门禁与 `cargo build` 一步都没跑到** ⇒ §2.4 仍待验。

### 2.1c Gate S7' 导出面门禁 —— 真因已定位、已修、待复验

第 10 次 CI 的真因是 **`${{ env.GITHUB_WORKSPACE }}` 求值为空**（详见 §0.2 真因 A），
**不是** Python 3.9 兼容问题 —— 昨天写在 §0.1 里的推定**已被证伪**，此处更正。
已改用 PowerShell 的 `$env:GITHUB_WORKSPACE` + `Join-Path` 拼四个参数。
昨天的三层处置里，第 1、2 层（3.9 兼容 / 钉 Python 3.11）作为**防御性加固保留**，
但它们不是本次失败的原因；**真正让问题浮出水面的是第 3 层 `::error::` 探针** ——
没有它，这个错误至今仍不可见。

**待验**：第 11 次 CI 里 S7' 能否拿到真实 DLL 并完成导出面比对。
本机用 346 DLL 对 345 源码验证通过；CI 编出的 345 DLL 是否与源码 `UI_API` 定义集
完全一致，**仍未实测**。若报缺符号/多符号，那是需要判断的真实差异，不是误报。

### 2.2 package job 的 zsui 检出（仍未实测）

`crates/zsdesk-app` 与 `crates/zsdesk-ui` 的 path 依赖是 `../../../zsui`
⇒ 解析到 `GITHUB_WORKSPACE/zsui`（与 `app` 同级）。
Cargo 解析**整个 workspace** 的 manifest，缺一个 path 依赖就直接失败，
即使本步骤只想构建 `zsdesk-coreui` 一个 crate。
已补 `path: zsui` 的 checkout，ref 取自 `app/packaging/zsui-revision.txt`。
**第 9 次 CI 的 package job 因上游两 job 失败被 skipped** ⇒ 这条修复仍未经实测。
第 10 次 CI 若 rust / core-ui 都转绿，package job 将**首次真正执行**，
那时才能确认 zsui 检出、戳记比对、ZIP 组装三条链路。

### 2.3 S7' 的判定基准（口径留档，不是未验证项）

门禁基准 = **同一 checkout 里 `src/` 的 `UI_API` 定义集**，不是 `docs/05`
（那是本机 **346** 预编译 DLL 的实测面，CI 编的是 **345**，跨版本比对只得版本漂移），
也不是 `include/ui_core.h`（上游漏声明 12 个符号：`ui_image_gdi_*`×10 +
`ui_flush_gpu` + `ui_image_view_gdi` + `ui_image_evict_tile`）。
`ui_dialog` 头文件有、源码无 ⇒ **双向容忍并报告**。
本机用 346 DLL 对 345 源码验证通过；第 9 次 CI 里 S7' 已真正执行（失败原因见 §2.1c）。

### 2.4 五个 S13 门禁在 CI 里的首次执行（仍未发生）

`uix_tag_check` / `theme_contrast_check` / `qa_uix_verify` /
`s13_token_namespace_check` / `ui_id_manifest_check`
本地全绿，但**第 9 次 CI 的 rust job 在第 6 步 `Test` 就失败，后续步骤全部跳过**
⇒ 门禁步骤一次都没跑到。CI 的工作目录（`$GITHUB_WORKSPACE\app`）与本地不同，
路径解析是否一致，**仍未实测**。本次已给门禁步骤补 `::error::` 探针，
失败时会把判定理由打进 annotations。

---

## 3. 后续里程碑（未开始）

| 里程碑 | 内容 | 依赖 |
|---|---|---|
| **M1** | Rust 侧真正用 core-ui 装上面板：读 `assets/*.uix` → 注入 `:root` → `ui_page_load_string` → 取 32 个句柄 → 自绘列表/导航树 → GuoheView 预览 | 依赖 §2.1 的 core-ui 构建成功 |
| **M2** | 文本预览上限与编码策略；GuoheView 深度集成 | M1 |
| **M3** | 去掉 zsui path 依赖（workspace 收口）；包体积红线实测 | M1 |
| 主题切换 | `ui_theme_set_mode` → **重建页面** → 重取全部 32 个句柄 + 在新 page 上重新注册 `ui_page_on_widget_mount` 钩子 + 重读系统 AccentColor 并重调 `ui_theme_set_accent_hex` | M1 |

---

## 4. 已知的判断与风险（留给下一轮）

### 4.1 已在源码级核实、但**只存在于文档/注释**、尚无机器门禁保护

- **`:root` 的「先定义者赢」** —— 靠 `value.cpp:572-579` 的 `LookupVar`
  首个命中语义；`:root` 跳过 CSS 级联（`cascade.cpp:125`）是「它只是变量容器」
  的原因，**不是**先定义者赢的原因（因果不要弄反）。
- **注入位置合法性** —— 顶层只允许 `<window>/<template>/<script>/<style>/<import>/<link>`
  （`sfc_parser.cpp:260-266`）；任何非 `<` 开头的顶层内容 ⇒ 整页不加载。
- **`<link>` 的优先级** —— `page_api.cpp:271` 把 link 排在所有 inline `<style>` 之前
  ⇒ 页面若有 `<link>` 且其 CSS 定义了 `:root`，注入值仍会被吃掉。
  **当前三个资产零 `<link>`**，但没有门禁守着这条。
- **BOM** —— 注入是前置的，BOM 会被挤到文件中部 ⇒ 整页不加载；
  `std::fs::read_to_string` 不剥 BOM。

### 4.2 已知的「静默失败」清单（无报错、界面不对）

| 症状 | 根因 |
|---|---|
| 界面全是库默认色，零报错 | 注入缺失 / 注入排在页面 `:root` 之后 / BOM / 变量名不在 `:root` 规则内 |
| 某块区域空白，零报错 | 该宿主是 `<div>` 而非 `<custom>` ⇒ `ui_custom_on_draw` 静默 no-op |
| 切主题颜色不变，零报错 | 同「注入被吃掉」 |
| 改文字没反应 | 句柄已失效（页面重建后）⇒ 往脱离控件树的 widget 写属性是**静默无效** |

### 4.3 已知缺口（有意留白，非本轮范围）

- **UIA / 无障碍**：core-ui **没有** UIA 支持 → 接受降级（用户 2026-11-13 拍板，D13）。
- **窗口材质**：core-ui **没有** Mica/Acrylic → 接受降级（D13）。
- **Rust 侧「注入自检」**：架构要求「位置/顺序/内容/键集」四条断言，
  `theme.rs` 目前在**构造上**保证了这些（只允许 `--app-*`、形态甲保证第一个 `:root`），
  但**没有**再写一个独立的「读回校验」——core-ui 无读回 CSS 变量的 C API，
  只能靠离屏渲染取像素（架构建议一次性冒烟，不进启动路径）。**未做。**
- **`about.uix` 的版本号**仍是写死的文本（已从 346 改成 345），
  理想做法是运行时从 DLL 读；about 页**尚未被 Rust 代码装配**，故不急。

---

## 5. 复现本次收尾的自检命令

```bash
# 五个 S13 门禁
for f in uix_tag_check theme_contrast_check qa_uix_verify \
         s13_token_namespace_check ui_id_manifest_check; do
  python scripts/gates/$f.py --root .
done

# S7' 导出面门禁（用本机 DLL + 上游源码作参照）
python scripts/gates/coreui_export_gate.py \
  --dll "C:/Program Files/GuoheView/core-ui.dll" \
  --source-dir <core-ui 源码>/src --include-dir <core-ui 源码>/include

# PE 导出表解析器的自校验（应输出与 docs/05 完全一致的 567 行）
python scripts/gates/pe_exports.py "C:/Program Files/GuoheView/core-ui.dll" \
  --prefix ui_ | diff - <(grep -oE "^ui_[a-z0-9_]+" docs/05-COREUI-EXPORTS.txt | sort -u)
```

### 5.1 读 CI 结果（不需要 admin 权限的通道）

job 日志下载端点要 admin（实测 403），但**对 public repo，check-run annotations 公开可读**。
本次已把 `cargo test` / S13 门禁 / Gate S7' 的失败原因打进 `::error::`，按下面读：

```bash
# 1) 最近几次 run
curl -s "https://api.github.com/repos/tikey1/zsdesk/actions/runs?per_page=5" \
  | python -c "import json,sys;[print(r['id'],r['conclusion'],r['created_at']) for r in json.load(sys.stdin)['workflow_runs']]"

# 2) 该 run 的三个 job（Actions 的 job id == check-run id）
curl -s "https://api.github.com/repos/tikey1/zsdesk/actions/runs/<RUN_ID>/jobs" \
  | python -c "import json,sys;[print(j['id'],j['name'],j['conclusion']) for j in json.load(sys.stdin)['jobs']]"

# 3) 失败原因
curl -s "https://api.github.com/repos/tikey1/zsdesk/check-runs/<JOB_ID>/annotations?per_page=100" \
  | python -c "import json,sys;[print('-',a['message']) for a in json.load(sys.stdin)]"
```

---

## 6. 协作纪律（沿用至今）

1. **git 操作统一由交付总监执行** —— 成员只写文件，不 commit / push。
2. **任何人不得对他人正在编辑的文件做批量改写**。
3. **作者不得自审** —— 产出需另一方独立复核（本轮 QA 与架构师的双向复核
   实际抓出了 6 个 P1 级问题，全部已修）。
4. **未验证的东西必须标注** —— 本文件即为该纪律的载体：
   §1 = 已验证，§2 = 未验证，§3 = 未开始。
