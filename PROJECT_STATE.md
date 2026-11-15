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
**同日新任务「同步上游更新的源码」：上游 v0.3.12 已完成本地合并与静态复验，第 17 次 CI 尚未判读。**

判读依据不是只看 run 的 `conclusion`，而是逐 step 核对 `GET /repos/tikey1/zsdesk/actions/runs/43673089804/jobs`：

| job | 首次跑通的关键步骤 | 结果 |
|---|---|---|
| `Rust 编译 / 测试` | Gate S14、**`cargo test --workspace --release --locked`（含 doctest）**、Gate S13（五个门禁）、`cargo build --locked --release -p zsdesk-app`、Clippy、上传 exe | 全 `success` |
| `core-ui 源码构建` | 从源码构建 + 日志落盘 + x64 断言 + Gate G2 / **Gate S7**（动态 CRT） / **Gate S7'**（导出面 vs 源码契约） | 全 `success` |
| `便携包组装` | **本 job 首次真正执行**（前六次均被上游失败连带 skipped）：下载 exe/DLL、Gate S12、收 VC++ x64 CRT、**编译 smoke probe**、**用真实 DLL + `.uix` 跑 smoke-probe**、打 ZIP、上传 | 全 `success` |

- **第 15 次 CI 4 个有效 artifact**（`expired: false`）：`ZSDesk-full-x64` 4,941,625 B、`zsdesk-app-full-x64` 2,358,072 B、`core-ui-x64` 1,981,859 B、`core-ui-build-log` 22,423 B。
- **第 16 次 CI**（run `43674268118`，commit `9a3628b`，纯文档提交）：三个 job 全绿、所有 step success/skipped、4 个 artifact 全部有效（`ZSDesk-full-x64` 4,941,600 B）。**上表体积是第 15/16 次的历史产物，合并上游后体积必然变化，不代表当前包体积。**
- `Package job failure locator` 显示 `skipped` 是**预期行为**（只在 job 失败时执行），不是失败。
- **失败链回顾**（第 9→14 次 CI 逐轮收口，详见 `DECISIONS.md` 第十五~十八轮 / `docs/15` §0.1~§0.6）：日志管道被 PowerShell 中断 → `Cargo.lock` 漂移 / `${{ env.GITHUB_WORKSPACE }}` 空值 → `E0583` 模块布局 → Windows `cp1252` 编码崩溃 → 5 个 Rust 类型/借用错误 → `probe.rs` 的 `Cow<str>`/`String` 类型不符。

### 2026-11-15 上游同步（本轮新任务）
- **用户指令**：「同步上游更新的源码」，并拍板「完整合并 v0.3.12」。调研确认：上游 `qiu7824/zsdesk` 从 fork 基线 `6aec0c2`（v0.3.11）前进 2 提交 / **108 文件** 到 `cb465e8`（v0.3.12）；`core-ui` pin `1840d689` 已是最新；`zsui` 领先 13 提交但只改 `docs/images/star-history.svg`（**两个 pin 文件均不动**）。
- **成员互查闸门已通过**：架构师 **Go（有条件，合并前零阻断）**、QA **Go**、运维 **No-Go → 补齐回滚前置后自动转 Go**。硬约束「禁止用上游 `Cargo.toml` / `Cargo.lock` 整文件覆盖」已满足（走三方自动合并）。
- **合并动作**：`git merge --no-ff upstream/main`（**不用 rebase**，以保留 `docs/12` 的 `6ea629f` 等历史引用可解析）。`merge-base = 6aec0c2a82`；**唯一冲突是 `README.md` 的版本行**，已人工裁决：保留 fork 的「AI修改」前缀与单行全量版表、版本写 `0.3.12`、修掉「上述六个版本」既存矛盾文案；上游 0.3.12 功能描述与 `CHANGELOG.md ## 0.3.12` 段完整带入。
- **`Cargo.lock` / `Cargo.toml` 自动合并且语义正确**：tomllib 实测 127 包有序、零悬空/零歧义/零版本错误引用；5 个 `zsdesk-*` 升 0.3.12；`zsdesk-windows` 正确引用 `"roxmltree 0.20.0"`（双版本带后缀）；`zsdesk-coreui 0.1.0` 与 `libloading` 保留；`Cargo.toml` 的 `members` 仍含 `crates/zsdesk-coreui`。
- **合并后静态复验（本机）**：六个机械门禁 `EXIT=0`；`virtual-files.zip` 完整（2 entries，`testzip: None`）；`.gitattributes` 补 `*.zip binary`；README 无冲突标记、QA 五条验收点已核。**S14 覆盖 145 个 `.rs` PASS 的口径必须收紧**：只能读作「文件齐全前提下不触发 E0583」，**不能**读作「上游新增模块布局已被门禁完整验证」——上游多处用 `#[cfg(test)]#[path="..."]mod <别名>;` 同行多属性写法，模块名≠文件名，正是 S14 的已知盲区（QA 负控：删掉目标文件仍 PASS）。
- **测试面口径（架构师与 QA 对齐后）**：上游那 10 个测试类 `.rs` **会**被 `cargo test` 收集，不是孤儿文件。`#[test]` 402→512、`#[ignore]` 59→84；预期 `passed` 343→428、`ignored` 59→84、`failed=0`，且不得出现既有 `#[test]` 被改成 `#[ignore]`、断言总数不得下降（fork 1422 / upstream 1801）。
- **回滚锚点**：本地 tag + 备份分支 `pre-upstream-0.3.12` = `9a3628b`；`upstream` remote 已加（`git@github.com:qiu7824/zsdesk.git`）。刻意**不 push tag**：`build.yml` 的 `on: push` 无分支过滤，单独推 tag 会额外触发一次针对旧 commit 的 CI、干扰判读序列；远端锚点由 merge commit 的第一父 `9a3628b` 天然提供。
- **诚实边界**：上游新增 28 个 `.rs` / +13,270 行（基线 138 / 42,150 → v0.3.12 166 / 55,420，合并后 174）**首次进入 `cargo test --workspace --release --locked`**，编译/测试结论**只能由第 17 次 CI 给出，不得预设通过**。本机无 cargo/rustc。

## 已完成
| 产物 | 路径 |
|---|---|
| 事实核对底稿（10 组实测 + §7.11 复核修正） | `research/facts.md` |
| 调研简报 R01–R10（PM） | `research/brief.md` |
| 决策台账（优先级规则 + D00~D23 + T1~T8 + 第二十轮） | `DECISIONS.md` |
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
| CI 工作流（三 job：rust 11 / core-ui 16 / package 13 步） | `.github/workflows/build.yml` — **第 15/16 次 CI 三个 job 全绿** |
| CI 说明与实跑记录（§6 已验证/未验证分界） | `docs/ci.md` |
| **剩余工作流程快照** | `docs/15-REMAINING-WORK.md` |
| **上游 v0.3.12 源码同步**（`--no-ff` merge，108 文件，保留全部 fork 改造） | 本仓库 `main`，上游基准 `cb465e8` |

## 进行中
| 谁 | 做到哪 | 状态 |
|---|---|---|
| PM / 前端 / QA / 运维-2 | 4 路自查已回传：前端 **Go**，PM · QA · 运维-2 **No-Go** | 全部已停止工作（2026-11-14 11:28 被批量暂停） |
| 架构师-2 | 本轮架构契约互查 + 口径更正（作废「孤儿文件」说法） | **Go（有条件）**，已收口 |
| 运维-2 / QA | 本轮互查：运维 **No-Go → 补齐前置即 Go**、QA 重判 **Go** | 裁决已收口，闸门通过 |
| 交付总监 | 上游 v0.3.12 合并 + 冲突裁决 + 门禁加固 + 文档回写 | **本地已完成静态复验；第 17 次 CI 待判读** |

本地 `main` 正在 merge commit 阶段；当前全绿锚点为 `9a3628b`（本地 tag `pre-upstream-0.3.12`），**远端 `origin/main` 在 push 前仍是 `9a3628b`**。

## 待决策
| # | 决策点 | 状态 | 推荐项 |
|---|---|---|---|
| Q1 | 包体积红线（10 MB）是否放宽 | **第 15/16 次实测** 4.94 MB（≈4.71 MiB）未触红线；**上游 v0.3.12 合并后体积会变，需按第 17 次 CI 产物重测** | **不放宽**，维持 10 MB；若超限优先剔 `licenses/` 240 份副本（约 1.16 MiB），**不砍功能** |
| Q2 | 文本预览上限与编码策略 | 可后置到 M2 | **后置到 M2**，M0/M1 只做图片预览；M2 采用 UTF-8 优先 + GBK 兜底（`research/brief.md` `R04`） |

> **编号说明**：本表用 `Q1/Q2`（待决问题），**不占用 D 编号** —— `D14` = 品牌色（`DECISIONS.md:478`）、`D15` = `.uix` 五条契约口径（`:506`）。

**已关闭**：D21（core-ui 构建退路）、`cargo test --locked` 从未转绿、package job 从未执行、`probe.rs` 从未编译四项风险。
**D22** = 钉 Python 3.11 + 门禁脚本保持 3.9 兼容；**D23** = 可能失败的步骤必须 emit `::error::`。
**第二十轮新登记（非阻断）**：迁移矩阵重算列为 M1 前置、S14 多属性同行漏检、`forbid(unsafe_code)` 契约失真，见 `DECISIONS.md`。

## 风险（Top 3）
1. **上游 v0.3.12 新增代码首次编译**：+28 个 `.rs` / +13,270 行、新增 `roxmltree` / MediaFoundation / WinHttp / WinRT features 全部进入 `cargo test --workspace --release --locked`；`#[test]` 402→512、`#[ignore]` 59→84。本机无 cargo，只能由第 17 次 CI 回答。若红，按 `git revert -m 1 <merge>` 回滚（**禁止** `reset --hard` + force push），或按公开 annotations 修到再次转绿。
2. **真机渲染仍未验证**：CI 证明「能编译、能打包、smoke-probe 退出码 0」，**不等于**界面观感正确。Win10 / WinUI 3 / Fluent 呈现、32 个句柄是否全部取到、`:root` 注入是否被 core-ui 先定义者吃掉，仍须按 `docs/13` 真机清单核对。
3. **契约漂移与 M1 未装配**：`spec/migration-matrix.md` 的 138 / 42,150 逐档清单对 v0.3.12 已整体失效（实测 166 / 55,420，+31.5%）；Rust 侧仍是 FFI 绑定层 + 冒烟探针，面板 / 导航树 / GuoheView 预览尚未接线；Actions artifact 默认 90 天过期、无 Release、Inno Setup 未接进 CI。

## 下一步
1. **主动判读第 17 次 CI**（按用户裁定「主动上传 + 主动按 Action 结果改代码，循环到 CI 转绿」）：逐 step 核对三 job，重点看 `--locked` 是否漂移、上游新增 Rust 源码编译、`failed=0` 且 `passed 343→428` / `ignored 59→84`（QA 判读项，不得有既有测试被改成 ignore）、S14 口径（只代表文件齐全不触发 E0583）、package job 12 步。失败则定位真因 → 修 → 再主动上传。
2. **合并后首项技术债（独立提交，不与本次 merge 混在一起）**：修 `scripts/gates/rust_module_layout_check.py:39/:41` 对「`#[path]` 不在行首」的 `#[cfg(test)]#[path="..."]mod <别名>;` 同行多属性写法漏检。**派单验收标准（QA 固化，三条负控缺一不可）**：① 文件齐全 → `EXIT=0`；② 删一个同行 `#[path]` 目标（如 `context_menu_runtime_tests.rs`）→ `EXIT=1`；③ 删一个普通 `mod` 目标（如 `desktop/drag_placement.rs`）→ `EXIT=1`。**不得改坏**分行且 `#[path]` 在行首的写法（现由 `:149-152` 的 pending_path 正确覆盖）。修法（QA 收口指定）：把 `:39/:41` 改为**逐行提取本行所有 `#[path]`（不限行首）**，同时保留分行 pending_path。另需在 `spec/crate-layout.md:385-398` 门禁表**补 S14 行**——性质是「S14 在 `build.yml:68-80` 有真实 step、却未登记进契约门禁表」，属**已执行但不在契约治理范围**；该行须写明覆盖范围与三条负控。
3. **M1 开工前置**：重算 `spec/migration-matrix.md`，再进全量业务装配与真机验收；`crate-layout.md:41/:115` 的 `forbid(unsafe_code)` 与 S9/S10 执行器缺口一并入账。

## 协作纪律（已立）
- **【现行，2026-11-14 用户裁定】每天代码编写前必须先通过「成员互查」**：成员交叉复核他人产出/交付物并给出明确 Go / No-Go；未过闸门不写码、不 commit、不 push。本轮上游同步已严格过闸（架构 Go / QA Go / 运维补齐前置即 Go）。
- **【现行，2026-11-15 用户裁定】闭环自动化**：代码编写完 + 静态复验通过 ⇒ **主动 commit / push**，不再逐次请示上传；CI 结束后**主动读公开 annotations → 定位真因 → 修代码 → 主动再上传**，循环到 CI 转绿，或遇到需用户拍板的问题才停。
- **【一次性豁免的适用范围】** 第 11~15 次 CI 修复闭环按连续同一次闭环处理、跳过互查直接修；**该豁免已随闭环结束**，本轮上游同步已恢复严格互查，后续新开发日同样须重新过闸。
- **【已取消】** 「重新激活所有成员进行代码自查」不再作为持续要求。自查（作者复核自己）≠ 互查（交叉复核他人），当前只保留互查。
- **上传前置判据**：成员互查 Go + 静态可复验阻断项清零；**CI 是上传之后的最终验证，不得拿它倒堵上传**。
- **git 操作统一由交付总监执行**；成员只写文件。
- **任何人不得对他人正在编辑的文件做批量改写**；确需改动先 SendMessage 协调。
- 未经实际执行的结论一律标「未验证 / `cargo_run=false`」，禁止写成「已通过」。
