# 剩余工作流程（2026-11-13 收尾快照 / 2026-11-15 闭环更新）

> 本文件原记录**代码停止编写时**的未完成事项。
> 已推送状态见 §1；CI 复验事项（**现已全部关闭**）见 §2；
> 后续里程碑见 §3；已知的判断与风险见 §4。
> **当前状态以 §0.7 与 `PROJECT_STATE.md` / `docs/ci.md` §6 为准**；§0.1~§0.6 是逐轮历史快照。

---

## 0. 一句话现状

**2026-11-15 第 15 次 CI（run `43673089804`，commit `63685f9`）三个 job 全部 `success`
—— M0「可由 GitHub Actions 编译并打包出可运行程序」的验收闭环达成，
本文件原先记录的「剩余 CI 验证事项」已全部关闭。**

- Rust 侧 core-ui 绑定层（`crates/zsdesk-coreui/`）、CI 契约、界面资产与
  `scripts/gates/` 共 8 个脚本（7 个独立门禁含 S14 + 1 个 PE 解析辅助库）均已落地，并经真实 CI 验证。
- **第 15 次 CI 首次跑通**：`cargo test --workspace --release --locked`
  （含 doctest 与 `theme.rs` 的 13 个单元测试）、`cargo build`、clippy，
  以及 **package job 全链路**（Gate S12 / 收 VC++ x64 CRT / smoke probe 首次编译 /
  真实 DLL + 真实 `.uix` 冒烟 / 便携 ZIP 打包与上传）。该 job 此前六次均被上游失败连带 skipped。
- **4 个有效 artifact**（`expired: false`）：`ZSDesk-full-x64` 4,941,625 B、
  `zsdesk-app-full-x64` 2,358,072 B、`core-ui-x64` 1,981,859 B、`core-ui-build-log` 22,423 B。
- 第 9~14 次的逐轮失败链与修法见 §0.1~§0.6；第 15 次收口见 §0.7。
- §1 的表格是**历史静态复核记录**，其中「未推送 / 未编译」是写入当时的标签，
  **不代表当前状态**：这些改动已随 `aa7e9b5` / `6f96662` / `16066e6` / `33aca13` / `63685f9`
  依次推送，并由第 11~15 次 CI 逐层验证。
- **仍未开始的是 M1 功能装配与真机验收**（见 §3），不是 CI 修复；按用户裁定的
  「主动上传 → 主动按 Action 结果改代码 → 循环到 CI 转绿」，本轮闭环到此结束。
- **2026-11-15 后续新增任务：同步上游 v0.3.12**。已按用户拍板「完整合并」执行：
  三路成员互查闸门通过（架构 Go / QA Go / 运维补齐前置即 Go）→ `git merge --no-ff upstream/main`
  （2 提交 / 108 文件，`ec7e5d2` + `cb465e8`）→ 唯一冲突 `README.md` 人工裁决 →
  `Cargo.lock` / `Cargo.toml` 三方自动合并且 tomllib 校验通过 → 六门禁 + S14（145 个 `.rs`）全过。
  **第 17 次 CI 尚未判读**，上游新增 28 个 `.rs` / +13,270 行首次进入 `cargo test`，
  编译结论只能由 CI 给出。完整决策与判读口径见 `PROJECT_STATE.md` 与 `DECISIONS.md` 第二十轮。

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
万一本地修得不全，后续 CI 会直接回传 cargo 期望的准确改动。
（**第 11 次 CI 未触发该分支** ⇒ 锁文件已修全，见 §2.1b。）

---

## 0.3 第 11 次 CI（run `43666238286`，commit `aa7e9b5`）实测结果

> 步骤一律用**步骤名**，不写编号 —— §0.1 / §0.2 的编号是各轮自己的口径，
> 插入 `setup-python` 后已漂移，继续沿用只会制造新的不一致。

| job | 步骤名 | 结果 | 结论 |
|---|---|---|---|
| core-ui | 全部（Configure and build / PE / G2 / S7 / S7' / upload） | **success** | C++ 源码构建第三次成功 |
| core-ui | Gate S7' — the DLL export surface must match the source contract | **success** | ★ `${{ env.GITHUB_WORKSPACE }}` → `$env:GITHUB_WORKSPACE` + `Join-Path` 的修复**已由真实 CI 证实** |
| Rust | Test | **failure** | 公开 annotations 给出 5 条 `error[E0583]: file not found for module` |
| Rust | Gate S13 / Build the full edition / Clippy / Upload the executable | skipped | Test 失败即跳过后续 |
| package | 全部 | skipped | `needs: [rust, core-ui]` 未满足 |

**真因 C —— Rust 模块布局**（不是 `Cargo.lock`，也不是 Python）：
`crates/zsdesk-coreui/src/lib.rs` 是 crate root，里面写的是裸 `pub mod api;` 等 5 条；
而模块文件全部放在 `crates/zsdesk-coreui/src/lib/*.rs`。Rust 2018+ 默认只为
`mod X;` 查找 `src/X.rs` 或 `src/X/mod.rs`，**不会进 `src/lib/`** ⇒
`api` / `loader` / `theme` / `ui_thread` / `version` 全部 E0583 ⇒
`error: could not compile zsdesk-coreui (lib) due to 5 previous errors`。
本机无 cargo/rustc，且此前 10 次 CI 都在更早的环节失败，所以这批 `.rs` 文件
**一次都没被编译过**，静态复核（含 tree-sitter 语法级）抓不到布局错。

**修法**：不搬文件（全仓 7 份文档按 `src/lib/*.rs` 引用），改在 5 条 `mod` 声明前
加 `#[path = "lib/xxx.rs"]`，显式钉住布局。另加机械门禁
`scripts/gates/rust_module_layout_check.py` 复现 Rust 的模块解析规则，
并接进 `build.yml` 的 `cargo test` **之前**（rust job 10 → 11 步）——
「编译前就能发现的问题不该浪费一整轮 CI」。

**仍未验证**：修完 E0583 后才轮到 rustc 真正编译这 5 个文件，可能还有类型 /
借用检查错误；`probe.rs` 也从未编译过。只能由第 12 次 CI 逐层回答。

---

## 0.4 第 12 次 CI（run `43670088427`，commit `6f96662`）实测结果

| job | 步骤名 | 结果 | 结论 |
|---|---|---|---|
| core-ui | 全部（含 Gate S7'） | **success** | C++ 源码构建第四次成功，S7' 保持转绿 |
| Rust | Gate S14 — Rust module layout (pre-compile) | **failure** | 新门禁自己崩了，见下方真因 E |
| Rust | Test / Gate S13 / Build / Clippy / Upload | skipped | S14 失败即跳过后续 |
| package | 全部 | skipped | `needs` 未满足，第四次跳过 |

**真因 E —— Windows runner 的 Python stdout 编码**：公开 annotations 给出完整 traceback：
```
UnicodeEncodeError: 'charmap' codec can't encode characters
File "...\rust_module_layout_check.py", line 181, in main
  print(f"检查 {len(roots)} 个 crate root，递归覆盖 ...")
File "...\Python\3.11.9\x64\Lib\encodings\cp1252.py", line 19, in encode
```
runner 的 Python 3.11 stdout 默认 `cp1252`，打印中文判定理由就抛异常 ⇒ 门禁以
「看起来失败」的方式崩溃。**本机是中文环境，同一批脚本本地全绿 ⇒ 本地绿不代表 CI 绿。**
更关键的是：Gate S13 的 5 个门禁脚本同样打印中文（实测 2~9 行不等），
此前从未跑到；只修 S14 的话，第 13 次 CI 会在 Gate S13 原地再崩一次。

**修法（全局根因，不是绕过）**：
1. `build.yml` 顶层 `env` 钉 `PYTHONIOENCODING: 'utf-8'` + `PYTHONUTF8: '1'`，
   一次覆盖所有 `python` 调用、六个门禁脚本与未来新增门禁；
2. 六个含中文输出的门禁脚本各加一段自包含的
   `sys.stdout/stderr.reconfigure(encoding="utf-8", errors="replace")`，
   保证脱离 workflow 单独运行也不崩。

**本地验证**：用 `PYTHONIOENCODING=cp1252` 复现了 S14 与 `theme_contrast_check` 的崩溃；
修复后六个门禁在 cp1252 与正常环境下**全部 `EXIT=0`**；S14 去掉 `#[path]` 的负控
仍精确报 5 条 E0583；YAML / 22 个 pwsh 块语法 / 3.9 兼容 / `git diff --check` 全绿。

**仍未验证**：`cargo test` 及其后所有步骤从未在 CI 跑到；`zsdesk-coreui` 5 个模块
首次被 rustc 编译可能暴露下一批类型 / 借用错误。只能由第 13 次 CI 逐层回答。

---

## 0.5 第 13 次 CI（run `43671423536`，commit `16066e6`）实测结果

| job | 步骤名 | 结果 | 结论 |
|---|---|---|---|
| core-ui | 全部（含 Gate S7'） | **success** | C++ 源码构建第五次成功 |
| Rust | Gate S14 — Rust module layout | **success** | cp1252 编码修复生效，门禁首次跑过 |
| Rust | Gate S13 — the mechanical .uix / theme gates | **success** | 五个 S13 门禁**首次在 CI 执行并通过**（六脚本 UTF-8 加固生效） |
| Rust | Test | **failure** | `zsdesk-coreui` 5 个模块首次被 rustc 编译，报 5 个错误 |
| Rust | Build / Clippy / Upload | skipped | Test 失败即跳过 |
| package | 全部 | skipped | 第五次跳过 |

**真因 G —— 从未编译过的 Rust 代码里的 5 个类型/借用错误**（公开 annotations 给出错误码，
本机无 cargo/rustc，靠静态逐条定位）：

| 错误 | 位置 | 根因与修法 |
|---|---|---|
| `E0277` | `loader.rs:366` | `libloading::Library::new(c_path)` 传了 `CString`，0.8 要求 `AsRef<OsStr>` → 改传 `path.as_os_str()`，删掉 `CString` |
| `E0631` / `E0308`（其一） | `loader.rs:366` | `LoadError::Load(std::io::Error)` 与 `libloading::Error` 不符 → 变体类型改 `libloading::Error` |
| `E0308` / `E0061` | `ui_thread.rs:76` + `:128` | `UiHandle.invoke_sync` 字段误声明为 `UiInvokeCallback`（回调，1 参数），但 `handle()` 赋的是 `Api::window_invoke_sync`（封送函数，3 参数），128 行又按 3 实参调用 → 字段类型改 3 参数函数指针，两个错误一起消失 |
| `E0382` | `lib.rs:169` | `StartupReport` 里 `capabilities: core.capabilities` 移动后又 `Ok((core, report))` → 改 `.clone()` |

**可观测性改进**：Test 步骤的 annotation 探针原先只打错误码，看不到出错位置（本轮就是
靠静态推断定位的）。现改为 `Select-String -Pattern '^error' -Context 0,5`，把 rustc 的
`--> 文件:行:列` 与代码片段一并打进公开 annotations（D23）。

**仍未验证**：`cargo test` 及其后所有步骤从未跑通；修完这 5 个错误后仍可能暴露下一批
编译错误。只能由第 14 次 CI 逐层回答。

---

## 0.6 第 14 次 CI（run `43672643005`，commit `33aca13`）实测结果

| job | 结果 | 结论 |
|---|---|---|
| core-ui | **success** | 含 Gate S7'，第六次成功 |
| Rust — Gate S14 / Gate S13 | **success** | 编码与门禁保持通过 |
| Rust — Test | **failure** | 第 13 次的 5 个编译错误**全部消失**；只剩 1 个新错误 |
| package | skipped | 第六次跳过 |

**真因 H（唯一剩余）**：改进后的 annotation 探针首次生效，直接给出
`error[E0308]: if and else have incompatible types` 与
`--> crates/zsdesk-coreui/examples/probe.rs:187:13`。
`err_text` 的 `if` 分支是 `String::new()`（`String`），`else` 分支是
`String::from_utf8_lossy(...)`（`Cow<str>`）⇒ 类型不符。
修法：`from_utf8_lossy(...).into_owned()`。

**探针改进已验证有效**：第 13 次只有错误码、靠静态推断定位；第 14 次直接拿到
文件:行:列，一轮定位完成（D23）。

**仍未验证**：`cargo test` 全量（含 doctest）、`cargo build`、package job、
`probe.rs` 实际运行。待第 15 次 CI。（→ **已由 §0.7 全部关闭**）

---

## 0.7 第 15 次 CI（run `43673089804`，commit `63685f9`）实测结果 —— 全绿 / 闭环达成

> 判读通道：`GET /repos/tikey1/zsdesk/actions/runs/43673089804/jobs?per_page=100`
> **逐 step 核对** + `/actions/runs/43673089804/artifacts` 核实产物；
> `GET /repos/tikey1/zsdesk/commits/63685f9/check-runs` 显示 3 个 check-run 全 `success`。

| job | 结果 | 结论 |
|---|---|---|
| Rust 编译 / 测试 | **success** | Gate S14、**`cargo test --locked` 首次通过（含 doctest 与 13 个单元测试）**、Gate S13、`cargo build`、clippy、上传 exe 全绿 |
| core-ui 源码构建 | **success** | 从源码构建 + x64 断言 + Gate G2 / S7 / **S7'** 连续第七次全绿 |
| 便携包组装 | **success** | **本 job 首次真正执行**：Gate S12 / 收 VC++ x64 CRT / smoke probe 首次编译 / **真实 DLL + 真实 `.uix` 冒烟通过** / ZIP 打包与上传 |

**第 14 次遗留的 4 项未知全部关闭**：

| 第 14 次时的未知项 | 第 15 次裁决 |
|---|---|
| `cargo test` 全量（含 doctest） | ✅ rust job step 8 `success` |
| `cargo build --locked --release -p zsdesk-app` | ✅ step 10 `success` |
| package job 首次真正执行 | ✅ 13 步中 12 步 `success`；`Package job failure locator` 按条件 `skipped`（**预期**） |
| `probe.rs` 编译 + 实际运行 | ✅ step 10 首次编译成功、step 11 冒烟退出码 0 |

**真因 I（第 14 次唯一剩余）已修复并证实**：`probe.rs` 的
`String::from_utf8_lossy(...)` 加 `.into_owned()`，与 `if` 分支的 `String::new()`
统一为 `String`（commit `63685f9`）。第 15 次 CI 不再报 `E0308`。

**产物**：`ZSDesk-full-x64` 4,941,625 B（≈ 4.71 MiB，未触 10 MB 红线）、
`zsdesk-app-full-x64` 2,358,072 B、`core-ui-x64` 1,981,859 B、
`core-ui-build-log` 22,423 B；四个 artifact 均 `expired: false`。

**闭环结论**：按用户 2026-11-15「代码编写完主动上传、主动根据 Action 结果修改代码、
循环到 CI 转绿」的裁定，本轮无需继续修 bug。**剩余工作转入 M1 功能装配与真机验收**（§3），
与 CI 修复闭环无关。

> 诚实边界：`Package job failure locator` 的**失败分支**在第 15 次未被触发
> （job 成功 ⇒ 条件步骤 skipped），因此该 `::error::` 捕获逻辑仍属**未实测**，
> 不得写成「已验证」。CI 全绿也**不等于**真机渲染 / 32 个句柄 / `:root` 视觉正确。

---

## 1. 已写入磁盘的改动与静态复核记录

> **口径（2026-11-15 第 15 次 CI 后更新）**：本表是**历史静态复核记录**，
> 逐行标签（含「未推送」「未编译」「待 CI 编译验证」）描述的是**写入当时**的状态，
> **不代表当前状态**，请勿据此判断项目进度。所有条目均已随
> `aa7e9b5` / `6f96662` / `16066e6` / `33aca13` / `63685f9` 依次推送，
> 并由第 11~15 次 CI 逐层验证；收口结论见 **§0.7** 与 `docs/ci.md` §6。
> 本机**无 cargo / rustc** 且缺 `zsui` path 依赖 ⇒ 本表所有「验证」都只是
> **静态 / 本机脚本级复核**，一律不代表「已通过编译或测试」；
> 编译结论以第 15 次 CI 为准。旧标题「已通过本地验证」措辞过强，已收敛。

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
| `Cargo.lock` 规范化 | `Cargo.lock` | **【已于 aa7e9b5 推送】** `libloading` 依赖改 `cfg-if` + `windows-link`、127 包按 `(name,version)` 重排、块间空行规范；tomllib 复验：有序 / 零歧义 / 零悬空引用。**第 11 次 CI 的 Test 已越过 `--locked`**（不再报锁文件漂移） |
| Gate S7' 路径修复 | `.github/workflows/build.yml` | **【已于 aa7e9b5 推送】** 4 个参数由 `${{ env.GITHUB_WORKSPACE }}`（求值为空）改用 `$env:GITHUB_WORKSPACE` + `Join-Path`；**第 11 次 CI 的 Gate S7' = success，已由真实 CI 证实** |
| lock 漂移自诊断（双保险） | `.github/workflows/build.yml` | **【已于 aa7e9b5 推送】** Test 失败且报错含 `--locked` 时，跑 `cargo metadata` 并把 `git diff -- Cargo.lock` 打进 annotations（第 11 次未触发该分支） |
| 门禁空过修复（P0） | `scripts/gates/theme_contrast_check.py` | **【已于 aa7e9b5 推送】** 缺/空资产不再 `continue`，缺资产 / 0 字节 / 无 `<template>` / 零文字元素一律 FAIL；本地 4 组负控 EXIT 符合预期。顺带统一 G05 口径：先剥注释与 `<script>`，不再把注释里的说明文字当成渲染文字 |
| smoke-probe 字符串传参修复（P0） | `crates/zsdesk-coreui/examples/probe.rs` | **【已于 aa7e9b5 推送】** `ui_page_load_string` 改传 NUL 结尾缓冲（含内嵌 NUL 守卫）；本机无 cargo，**未编译验证** |
| package job 失败可见性 | `.github/workflows/build.yml` | **【已于 aa7e9b5 推送】** 5 个可失败步骤各加 `::error::` 捕获 + job 级兜底定位；YAML 解析通过、21 个 pwsh 块语法解析 0 错误 |
| **Rust 模块布局修复（第 11 次 CI 真因）** | `crates/zsdesk-coreui/src/lib.rs` | **【2026-11-15 本轮，未推送】** 5 条 `mod` 声明各加 `#[path = "lib/xxx.rs"]`，消除 E0583；不搬文件以保留全仓 7 份文档的 `src/lib/*.rs` 引用。已静态核对所有 `pub use` 名字、`StartupReport` 字段与 `RevisionStamp` 变体均对得上；**本机无 cargo，类型检查只能由第 12 次 CI 回答** |
| **S14 Rust 模块布局门禁** | `scripts/gates/rust_module_layout_check.py` + `.github/workflows/build.yml` | **【2026-11-15 本轮，未推送】** 机械复现 Rust 模块文件解析规则，接在 `cargo test` **之前**（rust job 10 → 11 步），失败原因 emit `::error::`。正常态覆盖 6 个 crate root / 127 个 `.rs` 文件 EXIT=0；负控①去掉 `#[path]` → 精确报出第 11 次 CI 的那 5 个 E0583；负控②保留 `#[path]` 抽掉 `theme.rs` → 精确点名 theme |
| **Windows cp1252 编码修复（第 12 次 CI 真因）** | `.github/workflows/build.yml` + `scripts/gates/{qa_uix_verify,rust_module_layout_check,s13_token_namespace_check,theme_contrast_check,ui_id_manifest_check,uix_tag_check}.py` | **【2026-11-15 本轮，按新规矩主动推送】** 顶层 `env` 钉 `PYTHONIOENCODING=utf-8` + `PYTHONUTF8=1`；六脚本各加 stdout/stderr UTF-8 自包含加固。`PYTHONIOENCODING=cp1252` 下修复前后对照：S14 与 `theme_contrast_check` 由 `EXIT=1` 变 `EXIT=0`，六个门禁全部通过；负控仍报 5 条 E0583 |
| 决策台账 D17–D20、第十三 / 十四 / 十五 / 十六轮 | `DECISIONS.md` | — |
| **【收口，2026-11-15】上表所有「未推送 / 未编译 / 未验证」条目** | 全仓 | **已随 `aa7e9b5`~`63685f9` 依次全部推送；第 15 次 CI（run `43673089804`）三个 job 全绿 —— `cargo test`（含 13 个单元测试与 doctest）、`cargo build`、Gate S13 / S14、`probe.rs` 首次编译与实际运行、package job 全链路全部通过。逐项证据见 §0.7 与 `docs/ci.md` §6** |

---

## 2. CI 复验事项（**全部已关闭**）

> 本节原题为「待第 13 次 CI 复验的事项」。截至**第 15 次 CI**（run `43673089804`）
> 三个 job 全绿，表内所有 ⏳ 项均已由真实 CI 逐层关闭。
> 下面各小节（§2.1~§2.4）保留当时的分析过程，**仅作历史追溯**；
> **最新状态以本表、`docs/ci.md` §6 与 `PROJECT_STATE.md` 为准**。

| 原待验事项 | 关闭于 | 证据 |
|---|---|---|
| §2.1 core-ui C++ 构建 | 第 9 次 CI | 第 9~15 次连续 `success`，D21 退路已关闭 |
| §2.1b `Cargo.lock` 漂移 | 第 11 次 CI | 不再报 `--locked` 漂移 |
| §2.1c Gate S7' 路径 | 第 11 次 CI | `$env:GITHUB_WORKSPACE` + `Join-Path` 修复经真实 CI 证实 |
| §2.4 五个 S13 门禁在 CI 首次执行 | 第 13 次 CI | 第 13 / 15 次均通过 |
| **Gate S14 在 CI 首次执行** | 第 13 次 CI | 第 12 次 `cp1252` 崩溃已由全局 UTF-8 修复；第 13 / 15 次通过 |
| **Rust 5 个模块文件首次被 rustc 编译** | 第 13 次（首次）/ 第 15 次（全通过） | 暴露的 5 个类型/借用错误已在 `33aca13` 修复 |
| **`cargo test --workspace --release --locked`** | **第 15 次 CI** | rust job step 8 `success`，含 doctest 与 13 个单元测试 |
| `cargo build` / clippy / upload exe | 第 15 次 CI | rust job step 10 / 11 / 12 全 `success` |
| **package job 全链路**（Gate S12 / CRT / probe 编译与运行 / ZIP / 上传） | **第 15 次 CI 首次真正执行** | package job 12 步 `success`，第 13 步按条件 `skipped` |
| `probe.rs` 的 `E0308` | 第 15 次 CI | `63685f9` 的 `.into_owned()` 修复生效，不再报错 |

> 本机**无 cargo / rustc** 且缺 `zsui` path 依赖 ⇒ 「静态复核不能替代编译」
> 这条纪律仍然有效；只是本节的未知项已全部由 CI 逐轮回答完毕。
> 唯一仍属**未实测**的是 `Package job failure locator` 的失败分支（job 成功时条件步骤 skipped）。

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

### 2.1b Rust job 的 `cargo test` —— `Cargo.lock` 真因 ✅ 已由第 11 次 CI 关闭

第 10 次 CI 的真因是 **`Cargo.lock` 与 workspace 不同步**（详见 §0.2 真因 B），
**不是 doctest** —— doctest 的旧 crate 名只是同一轮里顺手修掉的确定性缺陷。
本地已把锁文件改成 cargo 规范形式：`libloading` 依赖 `cfg-if` + `windows-link`、
127 个包按 `(name, version)` 重排、块间空行规范化，tomllib 复验通过。

**第 11 次 CI 裁决**：Test 没有再报 `--locked` 漂移，`cargo metadata` 自诊断分支
也未触发 ⇒ 锁文件与 workspace manifests 已一致，**本真因关闭**。
⚠️ 但同一个 Test 步骤随后报出 5 条 E0583（§0.3），仍以 exit 1 结束；
本机无 Rust 工具链且缺 `zsui` path 依赖 ⇒ 模块内容的编译只能由第 12 次 CI 回答。
Test 的失败分支已加 lock 漂移自诊断（`cargo metadata` + `git diff -- Cargo.lock` → annotations）。
**第 9 / 10 / 11 次 CI 都在 Test 这一步失败，后面的 S13 门禁与 `cargo build` 一步都没跑到**
（第 9/10 次真因 = 锁文件，第 11 次真因 = E0583 模块布局）⇒ §2.4 仍待验。

### 2.1c Gate S7' 导出面门禁 —— ✅ 已由第 11 次 CI 关闭

第 10 次 CI 的真因是 **`${{ env.GITHUB_WORKSPACE }}` 求值为空**（详见 §0.2 真因 A），
**不是** Python 3.9 兼容问题 —— 昨天写在 §0.1 里的推定**已被证伪**，此处更正。
已改用 PowerShell 的 `$env:GITHUB_WORKSPACE` + `Join-Path` 拼四个参数。
昨天的三层处置里，第 1、2 层（3.9 兼容 / 钉 Python 3.11）作为**防御性加固保留**，
但它们不是本次失败的原因；**真正让问题浮出水面的是第 3 层 `::error::` 探针** ——
没有它，这个错误至今仍不可见。

**第 11 次 CI 裁决：Gate S7' = success** —— 四个 `Join-Path $ws ...` 参数都拿到了
CI 里的真实路径，导出面比对完成并通过。本机此前用 346 DLL 对 345 源码验证通过的
结论，现已在 CI 编出的 345 DLL 上**实测确认**。本项关闭。

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
   §0.7 = 当前已验证结论，§1 = 历史静态复核记录，§2 = CI 复验事项（**已全部关闭**），§3 = 未开始。
   第 15 次 CI 之后，**M1 功能装配、真机首渲染清单、Inno Setup 安装包编译、Release 分发**
   仍是明确标注的未验证 / 未开始项，见 `PROJECT_STATE.md`「风险」与「下一步」。
