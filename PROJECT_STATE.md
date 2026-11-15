# PROJECT_STATE

> 本文件是**唯一**的跨会话接续点。每次 Phase 切换、成员回传、需求变更时更新。
> 「已完成」只写路径；「待决策」每条必须带推荐项；「下一步」≤3 条。
> **决策优先级：本次对话中的选择 > 用户 fork 路线图 > 交付总监建议。**

## 目标
以 `qiu7824/zsdesk` 为基础，用 `ghboke/core-ui` 替换 ZSUI，做成**可由 GitHub Actions 编译出 x86-64 可运行程序**的 Windows 桌面文件管理器（**全量功能移植**）；
图片预览经 `C:\Program Files\GuoheView\GuoheView.exe` 外部调用；Rust 为主，CRT 用用户已装的 v14 运行库；
代码规范沿用 zsdesk 与 core-ui 两个上游库 + 用户既有 Gate 0~4 / R1~R12。
**工作仓库 = 用户自己的 fork `tikey1/zsdesk`（public，默认分支 main）。**

## 当前阶段
**Phase 1 调研与定案 → Phase 4 验收**：事实核对、Spec、契约、迁移矩阵、界面资产、QA 复核均已完成。
**2026-11-15 第 15 次 CI（run `43673089804`，commit `63685f9`）三个 job 全部 `success` —— M0「可由 GitHub Actions 编译并打包出可运行程序」的验收闭环达成**；第 16 次 CI 同样全绿。
**同日新任务「同步上游更新的源码」已收口**：v0.3.12 以 `--no-ff` merge 合入（merge commit `aa44acc`），
**第 17 次 CI（run `43678206351`）三个 job 逐 step 全 `success`、4 个 artifact 齐全** ⇒
记为「**上游 v0.3.12 同步收口 / M0 回归验证通过**」（扩大代码面上的回归验证），
**不是** M1 前置达成、不是功能里程碑、也不是 M0 重新达成。**CI 全绿 ≠ 功能完整或界面正确**。

判读依据不是只看 run 的 `conclusion`，而是逐 step 核对 `GET /repos/tikey1/zsdesk/actions/runs/43673089804/jobs`：

| job | 首次跑通的关键步骤 | 结果 |
|---|---|---|
| `Rust 编译 / 测试` | Gate S14、**`cargo test --workspace --release --locked`（含 doctest）**、Gate S13（五个门禁）、`cargo build --locked --release -p zsdesk-app`、Clippy（advisory / `continue-on-error`，step 记为 success）、上传 exe | 全 `success` |
| `core-ui 源码构建` | 从源码构建 + 日志落盘 + x64 断言 + Gate G2 / **Gate S7**（动态 CRT） / **Gate S7'**（导出面 vs 源码契约） | 全 `success` |
| `便携包组装` | **本 job 首次真正执行**（前六次均被上游失败连带 skipped）：下载 exe/DLL、Gate S12、收 VC++ x64 CRT、**编译 smoke probe**、**用真实 DLL + `.uix` 跑 smoke-probe**、打 ZIP、上传 | 全 `success` |

- **第 15 次 CI 4 个有效 artifact**（`expired: false`，**历史值**）：`ZSDesk-full-x64` 4,941,625 B、`zsdesk-app-full-x64` 2,358,072 B、`core-ui-x64` 1,981,859 B、`core-ui-build-log` 22,423 B。
- **第 16 次 CI**（run `43674268118`，commit `9a3628b`，纯文档提交）：三个 job 全绿、所有 step success/skipped、4 个 artifact 全部有效（`ZSDesk-full-x64` 4,941,600 B，**历史值**）。
- **第 17 次 CI 4 个有效 artifact**（`expired: false`，**当前值**）：`ZSDesk-full-x64` **5,594,915 B ≈ 5.34 MiB**、`zsdesk-app-full-x64` 3,006,887 B、`core-ui-x64` 1,981,859 B、`core-ui-build-log` 22,420 B。便携包未触 10 MB 红线（Q1 维持「不放宽」）。
- `Package job failure locator` 显示 `skipped` 是**预期行为**（只在 job 失败时执行），不是失败；**其失败分支仍属「未实测」**。
- **annotations 判读纪律（第 17 次重点）**：第 15 / 16 / 17 次公开 check-run annotations **逐条完全一致**。core-ui job 的 10 条 `failure` 级文本来自 `build.yml:361/:372/:384` 的**无条件日志探针**（`:370` 正则显式含 `warning`，`errorHits=534` 是 error+warning 总命中，不是失败计数）；Rust job 的 `Process completed with exit code 1.` 对应 `continue-on-error: true` 的 `Clippy (advisory)` 步骤；三个 job 各 1 条 Node.js 20 deprecated `warning`。`::error::` 不改变 step 退出码，`build.yml:395` 的 `if ($failed) { throw ... }` 保证构建真失败会让 step 失败。⇒ **判读必须以 run / job / step 的 `conclusion` 为准，不得凭 `annotation_level` 判红**；同时也不得把「clippy step success」写成「clippy 零告警」。
- **失败链回顾**（第 9→14 次 CI 逐轮收口，详见 `DECISIONS.md` 第十五~十八轮 / `docs/15` §0.1~§0.6）：日志管道被 PowerShell 中断 → `Cargo.lock` 漂移 / `${{ env.GITHUB_WORKSPACE }}` 空值 → `E0583` 模块布局 → Windows `cp1252` 编码崩溃 → 5 个 Rust 类型/借用错误 → `probe.rs` 的 `Cow<str>`/`String` 类型不符。

### 2026-11-15 上游同步（本轮新任务）
- **用户指令**：「同步上游更新的源码」，并拍板「完整合并 v0.3.12」。调研确认：上游 `qiu7824/zsdesk` 从 fork 基线 `6aec0c2`（v0.3.11）前进 2 提交 / **108 文件** 到 `cb465e8`（v0.3.12）；`core-ui` pin `1840d689` 已是最新；`zsui` 领先 13 提交但只改 `docs/images/star-history.svg`（**两个 pin 文件均不动**）。
- **成员互查闸门已通过**：架构师 **Go（有条件，合并前零阻断）**、QA **Go**、运维 **No-Go → 补齐回滚前置后自动转 Go**。硬约束「禁止用上游 `Cargo.toml` / `Cargo.lock` 整文件覆盖」已满足（走三方自动合并）。
- **合并动作**：`git merge --no-ff upstream/main`（**不用 rebase**，以保留 `docs/12` 的 `6ea629f` 等历史引用可解析）。`merge-base = 6aec0c2a82`；**唯一冲突是 `README.md` 的版本行**，已人工裁决：保留 fork 的「AI修改」前缀与单行全量版表、版本写 `0.3.12`、修掉「上述六个版本」既存矛盾文案；上游 0.3.12 功能描述与 `CHANGELOG.md ## 0.3.12` 段完整带入。
- **`Cargo.lock` / `Cargo.toml` 自动合并且语义正确**：tomllib 实测 127 包有序、零悬空/零歧义/零版本错误引用；5 个 `zsdesk-*` 升 0.3.12；`zsdesk-windows` 正确引用 `"roxmltree 0.20.0"`（双版本带后缀）；`zsdesk-coreui 0.1.0` 与 `libloading` 保留；`Cargo.toml` 的 `members` 仍含 `crates/zsdesk-coreui`。
- **合并后静态复验（本机）**：六个机械门禁 `EXIT=0`；`virtual-files.zip` 完整（2 entries，`testzip: None`）；`.gitattributes` 补 `*.zip binary`；README 无冲突标记、QA 五条验收点已核。**S14 覆盖 145 个 `.rs` PASS 的口径必须收紧**：`145` 是**门禁递归可达文件数**（`:181` 打印 `len(seen)`），**不是全仓 `.rs` 数（174）**；差额 29 = **18 个同行 `#[path]` 目标（5 个生产模块 + 13 个测试文件）** + 其余不在 `mod` 树上的文件（`examples/probe.rs`、`build.rs` 等），不能笼统写成「≈ 上游 28 个新增测试文件」。只能读作「文件齐全前提下不触发 E0583」，**不能**读作「上游新增模块布局已被门禁完整验证」。
  - **盲区是既存债务被放大，不是本次 merge 从 0 引入**：逐 commit 实测同行声明 `9a3628b`（合并前 fork）**8 个**（1 生产 + 7 测试）→ `cb465e8` / `aa44acc` **18 个**（5 生产 + 13 测试），**+10 / 2.25 倍**；分行声明 14 → 13 → 18；注释形态 2 / 0 / 2；`#[path]` 总命中 24 → 31 → 38。5 个生产模块是 `context_menu_process.rs:13/:14`、`utility_panel.rs:8`、`window/folder_sizes.rs:14`、`window/integrated_search.rs:5`。第 17 次 CI 全绿只证明这 18 个目标当下存在且被编译，**S14 本身没干活**
- **测试面口径（2026-11-15 重算，权威口径 = `.rs` 属性总数）**：合并前 fork `9a3628b` → 合并后 `aa44acc`：`#[test]` **402→527**、`#[ignore` 属性 **59→84**（全仓文本口径 405→534 / 59→88 仅作参考，含台账自指文本、不稳定）。上游 `cb465e8` 单独为 512 / 84。旧写法「402→512 / 59→84 / `passed` 343→428」是**跨对象 + 混口径**（402 属 fork、512 属上游；428 = 上游 512−84），**已作废**。四条独立对象链须分清：上游版本间 `6aec0c2`→`cb465e8` 为 387→512 / 59→84 / 328→428；fork 合并前后 `9a3628b`→`aa44acc` 为 402→527 / 59→84 / 343→443（`aa44acc` 的 84 与上游 84 数值相同纯属巧合）。
  - `#[ignore` 全部带 reason，其中 **14 个写成 `#[test]#[ignore="..."]` 同行**，按行首正则会漏数；静态非 ignored 估算 **343→443**（`#[test]` 减 `#[ignore`），**只是属性计数，不是运行结果**。
  - **runner 的 `passed` / `ignored` / `failed` 具体数字无公开通道可读**：`test result: ok. N passed; M ignored` 这些行**确实由 `build.yml:94` 的 `Tee-Object` 透传进 runner stdout 的 job log**（不是 CI 没产生数字）；但 `test-output.txt` 未收录进任何 artifact、成功分支（`:126`）不 emit annotation、job logs 下载端点需 admin（实测 403）⇒ **对外部读者与本次复核不可得**。一律标「**静态估算、无公开通道、未观测**」，不得写成「CI 实测」「CI 没产生数字」或「全部测试通过」。
  - `build.yml:94` 不带 `-- --ignored`，**84 个 ignored 测试按设计从未执行**（既存质量债，见 `DECISIONS.md:1116`）。
  - 上游经 `#[cfg(test)] #[path="..."] mod <别名>;` 引入的测试模块（模块名≠文件名），因 `cargo test --workspace --release --locked` 退出码 0 ⇒ 目标文件存在且被编译收集（缺失必致 E0583）；但退出码 0 **不能证明** 527 个静态 `#[test]` 全部被收集（cfg 屏蔽 / 宏展开不影响退出码），也不能拿 S14 PASS 当证据。
- **回滚锚点**：本地 tag + 备份分支 `pre-upstream-0.3.12` = `9a3628b`；`upstream` remote 已加（`git@github.com:qiu7824/zsdesk.git`）。刻意**不 push tag**：`build.yml` 的 `on: push` 无分支过滤，单独推 tag 会额外触发一次针对旧 commit 的 CI、干扰判读序列；远端锚点由 merge commit 的第一父 `9a3628b` 天然提供。
- **诚实边界**：上游新增 28 个 `.rs` / +13,270 行（基线 138 / 42,150 → v0.3.12 166 / 55,420，合并后 174）首次进入 `cargo test --workspace --release --locked`；本机无 cargo/rustc，结论**已由第 17 次 CI 给出：逐 step 全 `success`、退出码 0 ⇒ workspace 编译通过，被收集且未被 ignore 的测试与 doctest 均未使进程失败**。但 runner 具体计数未公开；本机静态复核不得写成「已通过编译/测试」。

## 已完成
| 产物 | 路径 |
|---|---|
| 事实核对底稿（10 组实测 + §7.11 复核修正） | `research/facts.md` |
| 调研简报 R01–R10（PM） | `research/brief.md` |
| 决策台账（优先级规则 + D00~D23 + T1~T8 + 第二十/二十一轮） | `DECISIONS.md` |
| **总 Spec**（总览 + D06~D12 决策索引 + 契约索引） | `SPEC.md` |
| core-ui 导出符号清单（2006 导出 / 567 纯 C API） | `docs/05-COREUI-EXPORTS.txt` |
| Core UI ABI 核对笔记（18 VERIFIED / 12 UNVERIFIED） | `docs/05-COREUI-ABI-NOTES.md` |
| `.uix` 标记标签权威支持表 + 主题 token 冲突 | `docs/11-COREUI-MARKUP-TAGS.md` |
| 界面资产修正对照表 + 真机首查清单（前端） | `docs/12-UIX-TAG-FIX.md`、`docs/13-UIX-FIRST-RENDER-CHECKLIST.md` |
| **QA 独立复核报告**（8 项裁决 + 3 阻塞 + 7 重要 + 第二轮） | `docs/14-QA-UIX-REVIEW.md` |
| 技术契约（FFI / 目录 / 迁移矩阵 / CI / 里程碑 / ADR） | `spec/` |
| FFI 绑定层（Rust，运行时动态加载） | `crates/zsdesk-coreui/` |
| `:root` 注入模块（形态甲 + 13 个 `#[test]`） | `crates/zsdesk-coreui/src/lib/theme.rs` — **13 个单元测试已由第 15 次 CI 执行并通过** |
| 八个门禁脚本（7 个独立可执行门禁含 S14 + 1 个 PE 解析辅助库） | `scripts/gates/` — **Gate S13 / S14 已在 CI 执行并通过** |
| ui id 清单（Rust ↔ `.uix` 的单一事实源，32 个 id） | `assets/ui-ids.json` |
| CI 工作流（三 job：rust 11 / core-ui 16 / package 13 步） | `.github/workflows/build.yml` — **第 15/16/17 次 CI 三个 job 全绿** |
| CI 说明与实跑记录（§6 已验证/未验证分界） | `docs/ci.md` |
| **剩余工作流程快照** | `docs/15-REMAINING-WORK.md` |
| **上游 v0.3.12 源码同步**（`--no-ff` merge，108 文件，保留全部 fork 改造） | 本仓库 `main`，上游基准 `cb465e8`，merge commit `aa44acc` |

## 进行中
| 谁 | 做到哪 | 状态 |
|---|---|---|
| PM / 前端 / QA / 运维-2 | 4 路自查已回传：前端 **Go**，PM · QA · 运维-2 **No-Go** | 全部已停止工作（2026-11-14 11:28 被批量暂停） |
| 架构师-2 | 本轮架构契约互查 + 口径更正（作废「孤儿文件」说法） | **Go（有条件）**，已收口 |
| 运维-2 / QA | 本轮互查：运维 **No-Go → 补齐前置即 Go**、QA 重判 **Go** | 裁决已收口，闸门通过 |
| 交付总监 | 第 17 次 CI 判读 + 两路成员互查收口 + 状态台账回写 | **两路互查均 Go（QA Go 无条件 / 架构师 Go 有条件），已 commit + push 触发第 18 次 CI** |

本地 `main` 与远端 `origin/main` **均已指向 merge commit `aa44acc`**（第 17 次 CI 为 `event=push, head_sha=aa44acc, head_branch=main, run_attempt=1`，无重跑）。
合并前全绿锚点 `9a3628b` 仍由本地 tag / 备份分支 `pre-upstream-0.3.12` 固化，远端则由 merge commit 第一父天然提供。

## 待决策
| # | 决策点 | 状态 | 推荐项 |
|---|---|---|---|
| Q1 | 包体积红线（10 MB）是否放宽 | **第 17 次 CI 实测** `ZSDesk-full-x64` 5,594,915 B ≈ 5.34 MiB，未触红线（第 15/16 次 4.94 MB 为历史值） | **不放宽**，维持 10 MB；若超限优先剔 `licenses/` 240 份副本（约 1.16 MiB），**不砍功能** |
| Q2 | 文本预览上限与编码策略 | 可后置到 M2 | **后置到 M2**，M0/M1 只做图片预览；M2 采用 UTF-8 优先 + GBK 兜底（`research/brief.md` `R04`） |

> **编号说明**：本表用 `Q1/Q2`（待决问题），**不占用 D 编号** —— `D14` = 品牌色（`DECISIONS.md:478`）、`D15` = `.uix` 五条契约口径（`:506`）。

**已关闭**：D21（core-ui 构建退路）、`cargo test --locked` 从未转绿、package job 从未执行、`probe.rs` 从未编译四项风险。
**D22** = 钉 Python 3.11 + 门禁脚本保持 3.9 兼容；**D23** = 可能失败的步骤必须 emit `::error::`。
**第二十轮新登记（非阻断）**：迁移矩阵重算列为 M1 前置、S14 多属性同行漏检、`forbid(unsafe_code)` 契约失真，见 `DECISIONS.md`。

## 风险（Top 3）
1. ~~上游 v0.3.12 新增代码首次编译~~ **已由第 17 次 CI 关闭**：+28 个 `.rs` / +13,270 行、新增 `roxmltree` / MediaFoundation / WinHttp / WinRT features 全部通过 `cargo test --workspace --release --locked`（`.rs` `#[test]` 402→527、`#[ignore` 59→84）。**残余风险**：runner 具体 passed/ignored 无公开通道；84 个 ignored 未执行；S14 对 18 个同行 `#[path]` 声明漏检（防御纵深缺口，见「下一步 2」；**该漏检为既存债务**：合并前 `9a3628b` 已有 8 个同行声明在盲区，v0.3.12 扩大至 18，+10 / 2.25 倍，不是本次 merge 从 0 引入）。若日后 CI 红，按 `git revert -m 1 <merge>` 回滚（**禁止** `reset --hard` + force push）。
2. **真机渲染仍未验证**：CI 证明「能编译、能打包、smoke-probe 退出码 0」，**不等于**界面观感正确。Win10 / WinUI 3 / Fluent 呈现、32 个句柄是否全部取到、`:root` 注入是否被 core-ui 先定义者吃掉，仍须按 `docs/13` 真机清单核对。
3. **契约漂移与 M1 未装配**：`spec/migration-matrix.md` 的 138 / 42,150 逐档清单对 v0.3.12 已整体失效（实测 166 / 55,420，+31.5%）；Rust 侧仍是 FFI 绑定层 + 冒烟探针，面板 / 导航树 / GuoheView 预览尚未接线；Actions artifact 默认 90 天过期、无 Release、Inno Setup 未接进 CI。

## 下一步
1. **第 17 次 CI 已判读全绿**（按用户裁定「主动上传 + 主动按 Action 结果改代码，循环到 CI 转绿」）：run `43678206351` 三 job 逐 step 全 `success`、唯一 `skipped` 为条件步骤、4 个 artifact 齐全、`--locked` 未漂移。**本次状态台账回写是纯文档提交，push 后必然触发第 18 次 CI，须单独判读**，不得混入第 17 次结论。CI 判读只认 run/job/step `conclusion`，不认 `annotation_level`。
2. **合并后首项技术债（独立提交，不与本次 merge 混在一起）**：修 `scripts/gates/rust_module_layout_check.py` 对 `#[cfg(test)]#[path="..."]mod <别名>;` 同行多属性写法的漏检。**派单验收标准（QA + 架构师共同固化）**：
   - ① 根因是 `:38-39` `_MOD_DECL` 与 `:41` `_PATH_ATTR` **两个正则都带 `^` 行首锚定**，且 `:117/:122` 都用 `.match(line)`；同行写法整行既认不出 path 也认不出 mod。**必须两个正则同时去掉 `^`**，并在**同一行内同时提取 `#[path]` 与 `mod` 就地配对、配对后立刻清空 `pending_path`**；分行且 `#[path]` 在行首的写法继续走 `:149-152` 的 pending_path，不得改坏。
   - ② **必须同改两个正则**：只把 `_PATH_ATTR` 换成 `.search()` 而 `_MOD_DECL` 仍带 `^`，同行行会设置 `pending_path` 却识别不到 `mod`，扩大泄漏面。**泄漏机制在当前脚本下已存在**（不是修复才引入）：实测 `#[path="a.rs"]mod x;` 紧跟 `mod y;` 时，[A] a.rs 存在 / y.rs 缺失 → `fails=0` 假阴性；[B] a.rs 缺失 / y.rs 存在 → 误报「`mod y` 找不到模块文件（已试 a.rs）」；[C] 真实上游形态 `#[cfg(test)]#[path="a.rs"]mod x;` 因整行被跳过反而不泄漏。当前仓库 5 个生产模块中，`context_menu_process.rs:13` 的下一行 `:14` 是**另一条行首 `#[path]` 声明**（会覆盖 `pending_path`，不是普通代码）、`window/integrated_search.rs:5` 的下一行 `:6` 是**空行**（`:151` 的 `if stripped` 对空行不重置，须到 `:7` 的普通代码才重置），其余三个下一行是普通代码 ⇒ 正确理由是「下一行或再下一行遇到普通代码 / 另一条行首 `#[path]` 覆盖」。**目前仍未实际误配，属潜在风险而非已发生缺陷**；但修复方案不得把这个既存缺陷带进新路径。
   - ③ 三条负控缺一不可：ⓐ 文件齐全 → `EXIT=0`；ⓑ 删一个同行 `#[path]` 目标（如 `context_menu_runtime_tests.rs`）→ `EXIT=1` 且报错点名该文件与行；ⓒ 删一个普通 `mod` 目标（如 `desktop/drag_placement.rs`）→ `EXIT=1`。**须在真实仓库工作副本上跑**（覆盖 31 个真实声明），不是临时最小复现。
   - ④ **新增负控（防误配）**：构造 `#[cfg(test)]#[path="a.rs"]mod x;` 紧跟一行 `mod y;`，断言 `y` 按默认规则解析、不被指向 `a.rs`。
   - ⑤ **新增量化锚点（防「改了但没生效」）**：现场 `python scripts/gates/rust_module_layout_check.py --root .` 输出「检查 6 个 crate root，递归覆盖 **145** 个 Rust 文件」（`:181` = `len(seen)`），全仓 `.rs` = 174，差 29 = 18 个同行目标（5 生产 + 13 测试）+ 其余非 mod 树文件；修复后该数须明显上升，**且 18 个同行 `#[path]` 的目标文件必须逐一进入 `seen`**（逐名核对比单看「≥163」更可机械判定）。注意同行声明在 fork 合并前已有 8 个、v0.3.12 后 18 个，属**既存债务扩大**而非新引入。
   - ⑥ `pending_path` 保持单值 `str`：实测 18 个同行声明里 0 个含多个 `#[path]`，按「每行一个 `#[path]`」处理即可，不必改成列表。
   - ⑦ 同时在 `spec/crate-layout.md:385-398` 门禁表**补 S14 行**（性质：S14 在 `build.yml:68-83` 有真实 step、却未登记进契约门禁表）。该行须写明覆盖范围与三条负控，**且不得写成「覆盖全部 mod 声明」**——18 个同行写法在修复前是盲区；须注明「当前覆盖行首 `#[path]` 与分行写法，同行多属性写法在 P1 修复前不覆盖」，修复后再更新。
   - ⑧ `145` 与 `174` 并存时须注明「145 = 门禁递归可达文件数，非全仓 `.rs` 数」，不得据以推断「门禁已覆盖新代码」。
   - ⑨ CI 复验：修复提交后第 18 次（或后续）CI 的 Gate S14 须 `success` 且不报 E0583。
3. **M1 开工硬前置**：重算 `spec/migration-matrix.md`。**注意 `spec/tools/matrix_calc.py` 当前不支持对任意 ref 重跑**：`:3` 的 `BASE` 硬编码 `C:/Users/tx/AppData/Local/Temp/upstream/zsdesk-main`、无 `--ref` 参数、`CLS` 是 v0.3.11 的硬编码逐文件分类表。⇒ 重算前须先给脚本加 `--ref` / 可配置源码根，或改用 `git ls-tree -r <ref>` + 行数统计，再**重建四档分类**；目标总计为上游 `cb465e8` **166 个 `.rs` / 55,420 行**，并区分「上游 166」与「含 fork `zsdesk-coreui` 共 174」。顶部警示改为「已重算 / 生效」、修 `:6/:13` 的旧工具口径。**重算完成前 M1 不得开工**；`crate-layout.md:41/:115` 的 `forbid(unsafe_code)` 与 S9/S10 执行器缺口一并入账。

## 协作纪律（已立）
- **【现行，2026-11-14 用户裁定】每天代码编写前必须先通过「成员互查」**：成员交叉复核他人产出/交付物并给出明确 Go / No-Go；未过闸门不写码、不 commit、不 push。本轮上游同步已严格过闸（架构 Go / QA Go / 运维补齐前置即 Go）；第 17 次 CI 判读与台账回写另经两路交叉复核，**最终口径：两路均曾 No-Go（有条件），在 6 条条件补齐 + 复核最终 diff 后同转 Go（有条件）**，方才 commit / push（见 `DECISIONS.md` 第二十一轮）。
- **【现行，2026-11-15 用户裁定】闭环自动化**：代码编写完 + 静态复验通过 ⇒ **主动 commit / push**，不再逐次请示上传；CI 结束后**主动读公开 annotations → 定位真因 → 修代码 → 主动再上传**，循环到 CI 转绿，或遇到需用户拍板的问题才停。
- **【一次性豁免的适用范围】** 第 11~15 次 CI 修复闭环按连续同一次闭环处理、跳过互查直接修；**该豁免已随闭环结束**，本轮上游同步已恢复严格互查，后续新开发日同样须重新过闸。
- **【已取消】** 「重新激活所有成员进行代码自查」不再作为持续要求。自查（作者复核自己）≠ 互查（交叉复核他人），当前只保留互查。
- **上传前置判据**：成员互查 Go + 静态可复验阻断项清零；**CI 是上传之后的最终验证，不得拿它倒堵上传**。
- **git 操作统一由交付总监执行**；成员只写文件。
- **任何人不得对他人正在编辑的文件做批量改写**；确需改动先 SendMessage 协调。
- 未经实际执行的结论一律标「未验证 / `cargo_run=false`」，禁止写成「已通过」。
