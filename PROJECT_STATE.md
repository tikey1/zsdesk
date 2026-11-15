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
**2026-11-15 第 15 次 CI（run `43673089804`，commit `63685f9`）三个 job 全部 `success` —— M0「可由 GitHub Actions 编译并打包出可运行程序」的验收闭环达成。**

判读依据不是只看 run 的 `conclusion`，而是逐 step 核对 `GET /repos/tikey1/zsdesk/actions/runs/43673089804/jobs`：

| job | 首次跑通的关键步骤 | 结果 |
|---|---|---|
| `Rust 编译 / 测试` | Gate S14、**`cargo test --workspace --release --locked`（含 doctest）**、Gate S13（五个门禁）、`cargo build --locked --release -p zsdesk-app`、Clippy、上传 exe | 全 `success` |
| `core-ui 源码构建` | 从源码构建 + 日志落盘 + x64 断言 + Gate G2 / **Gate S7**（动态 CRT） / **Gate S7'**（导出面 vs 源码契约） | 全 `success` |
| `便携包组装` | **本 job 首次真正执行**（前六次均被上游失败连带 skipped）：下载 exe/DLL、Gate S12、收 VC++ x64 CRT、**编译 smoke probe**、**用真实 DLL + `.uix` 跑 smoke-probe**、打 ZIP、上传 | 全 `success` |

- **4 个有效 artifact**（`expired: false`）：`ZSDesk-full-x64` 4,941,625 B、`zsdesk-app-full-x64` 2,358,072 B、`core-ui-x64` 1,981,859 B、`core-ui-build-log` 22,423 B。
- `Package job failure locator` 显示 `skipped` 是**预期行为**（该步骤只在 job 失败时执行），不是失败。
- **失败链回顾**（第 9→14 次 CI 逐轮收口，详见 `DECISIONS.md` 第十五~十八轮 / `docs/15` §0.1~§0.6）：日志管道被 PowerShell 中断 → `Cargo.lock` 漂移 / `${{ env.GITHUB_WORKSPACE }}` 空值 → `E0583` 模块布局 → Windows `cp1252` 编码崩溃 → 5 个 Rust 类型/借用错误 → `probe.rs` 的 `Cow<str>`/`String` 类型不符。每一轮都由公开 check-run annotations 定位真因后修代码、主动推送。
- **闭环完成**：按用户 2026-11-15 裁定的「主动上传 + 主动按 Action 结果改代码，循环到 CI 转绿」，本轮无需继续修 bug。

## 已完成
| 产物 | 路径 |
|---|---|
| 事实核对底稿（10 组实测 + §7.11 复核修正） | `research/facts.md` |
| 调研简报 R01–R10（PM） | `research/brief.md` |
| 决策台账（优先级规则 + D00~D23 + T1~T8） | `DECISIONS.md` |
| **总 Spec**（总览 + D06~D12 决策索引 + 契约索引） | `SPEC.md` |
| core-ui 导出符号清单（2006 导出 / 567 纯 C API） | `docs/05-COREUI-EXPORTS.txt` |
| Core UI ABI 核对笔记（18 VERIFIED / 12 UNVERIFIED） | `docs/05-COREUI-ABI-NOTES.md` |
| `.uix` 标记标签权威支持表 + 主题 token 冲突 | `docs/11-COREUI-MARKUP-TAGS.md` |
| 界面资产修正对照表 + 真机首查清单（前端） | `docs/12-UIX-TAG-FIX.md`、`docs/13-UIX-FIRST-RENDER-CHECKLIST.md` |
| **QA 独立复核报告**（8 项裁决 + 3 阻塞 + 7 重要 + 第二轮） | `docs/14-QA-UIX-REVIEW.md` |
| 技术契约（FFI / 目录 / 迁移矩阵 / CI / 里程碑 / ADR） | `spec/` |
| FFI 绑定层（Rust，运行时动态加载） | `crates/zsdesk-coreui/` |
| `:root` 注入模块（形态甲 + 13 个 `#[test]`） | `crates/zsdesk-coreui/src/lib/theme.rs` — **13 个单元测试已由第 15 次 CI 的 `cargo test` 执行并通过** |
| 八个门禁脚本（7 个独立可执行门禁含 S14 + 1 个 PE 解析辅助库） | `scripts/gates/` — **Gate S13 / S14 已在 CI 执行并通过** |
| ui id 清单（Rust ↔ `.uix` 的单一事实源，32 个 id） | `assets/ui-ids.json` |
| CI 工作流（三 job：rust 11 / core-ui 16 / package 13 步） | `.github/workflows/build.yml` — **三个 job 全部 success** |
| CI 说明与实跑记录（§6 已验证/未验证分界已对齐第 15 次 CI） | `docs/ci.md` |
| **剩余工作流程快照**（含 §0.7 第 15 次 CI 结果） | `docs/15-REMAINING-WORK.md` |

## 进行中
| 谁 | 做到哪 | 状态 |
|---|---|---|
| PM / 前端 / QA / 运维-2 | 4 路自查已回传：前端 **Go**，PM · QA · 运维-2 **No-Go** | 全部已停止工作（2026-11-14 11:28 被批量暂停） |
| 架构师-2 | 架构契约自查被中断，**未回传裁决** | 已停止；按用户新规不续跑自查，遗留 A–E 范围并入后续互查 |
| 交付总监 | 第 11~15 次 CI 的主动「定位 → 修复 → 上传 → 判读」闭环 | **第 15 次 CI 全绿，闭环完成**；本轮按用户一次性豁免互查执行 |

本地与远端 `main` 保持同步；**第 15 次 CI 对应的 commit 是 `63685f9`**，本次状态回写会再产生一个新 commit（纯文档，不改构建逻辑）。

## 待决策
| # | 决策点 | 状态 | 推荐项 |
|---|---|---|---|
| Q1 | 包体积红线（10 MB）是否放宽 | **已有实测**：`ZSDesk-full-x64` = 4,941,625 B ≈ **4.71 MiB**（压缩后 ZIP），未触红线 | **不放宽**，维持 10 MB；PM `R02` 早期担心的 `licenses/` 240 份副本（约 1.16 MiB）在实测体积下也无需剔除，**不砍功能** |
| Q2 | 文本预览上限与编码策略 | 可后置到 M2 | **后置到 M2**，M0/M1 只做图片预览；M2 采用 UTF-8 优先 + GBK 兜底，行数/字节上限届时按真机体验定（`research/brief.md` `R04`） |

> **编号说明**：本表用 `Q1/Q2`（待决问题），**不占用 D 编号** —— 决策台账里
> 正式 `D14` = 品牌色（`DECISIONS.md:478`）、`D15` = `.uix` 五条契约口径（`:506`）。

**已关闭**：D21（core-ui 构建退路）—— 第 9 / 10 / 11 次 CI 构建成功，退路留 `docs/15` §2.1 备查；
**第 15 次 CI 另关闭**：`cargo test --locked` 从未转绿、package job 从未执行、`probe.rs` 从未编译三项风险。
**第十三轮新定**：D22 = 钉 Python 3.11 + 门禁脚本保持 3.9 兼容；
D23 = 可能失败的步骤必须 emit `::error::`（job 日志需 admin，annotations 公开可读）。
**第十四~十九轮**：无新增用户决策，仅流程裁定与 bug 修复，见 `DECISIONS.md` 变更记录。

## 风险（Top 3）
1. **真机渲染仍未验证**：CI 证明的是「能编译、能打包、smoke-probe 退出码 0」，**不等于**界面观感正确。Win10 / WinUI 3 / Fluent 的实际呈现、32 个句柄是否全部取到、`:root` 注入是否被 core-ui 的先定义者吃掉，仍须按 `docs/13-UIX-FIRST-RENDER-CHECKLIST.md` 在真机逐项核对。
2. **M1 业务功能尚未装配**：当前 Rust 侧是 FFI 绑定层 + 冒烟探针，全量移植的面板 / 导航树 / 自绘列表 / GuoheView 预览尚未接线；ZIP 里的 exe 能跑通构建链路，但不是功能完整的成品。
3. **分发链路未建立**：Actions artifact 默认 90 天过期，尚无 Release / 永久下载；Inno Setup 安装包（`packaging/windows/zsdesk.iss`）也未接进 CI 编译。

## 下一步
1. **开启新开发日须先过成员互查**（长期规则，见「协作纪律」）；本轮 CI 修复闭环用的一次性豁免到此为止，不自动延续。
2. **M1 开工**：Rust 侧真正用 core-ui 装上面板 —— 读 `assets/*.uix` → 注入 `:root` → `ui_page_load_string` → 取 32 个句柄 → 自绘列表/导航树 → GuoheView 预览。
3. **真机验收 + 分发**：跑 `docs/13` 首渲染清单；决定 artifact 是否转为 Release，并把 Inno Setup 安装包纳入 CI。

## 协作纪律（已立）
- **【现行，2026-11-14 用户裁定】每天代码编写前必须先通过「成员互查」**：成员交叉复核他人产出/交付物并给出明确 Go / No-Go；未过闸门不写码、不 commit、不 push。
- **【现行，2026-11-15 用户裁定】闭环自动化**：代码编写完 + 静态复验通过 ⇒ **主动 commit / push**，不再逐次请示上传；CI 结束后**主动读公开 annotations → 定位真因 → 修代码 → 主动再上传**，循环到 CI 转绿，或遇到需用户拍板的问题才停。旧规则「上传属对外动作需逐次确认」被本条覆盖。
- **【一次性豁免的适用范围】** 用户 2026-11-14「不用了，继续修复 bug」、11-15「继续上传」「根据 action 返回的问题重新修改代码」⇒ 第 11~15 次 CI 按连续的同一次修复闭环处理，跳过互查直接修；**长期互查规则不变**，不视为废止。后续开启新的开发日须重新过互查。
- **【已取消】** 「重新激活所有成员进行代码自查」不再作为持续要求，仅当轮一次性生效。自查（作者复核自己）≠ 互查（交叉复核他人），当前只保留互查。
- **上传前置判据**：成员互查 Go + 静态可复验阻断项清零；**CI 是上传之后的最终验证，不得拿它倒堵上传**（否则 CI 永远触发不了）。
- **git 操作统一由交付总监执行**；成员只写文件。已发生两次成员抢先 commit 导致 push 被 `&&` 短路。
- **任何人不得对他人正在编辑的文件做批量改写**（sed -i / 全文替换 / 格式化）；确需改动先 SendMessage 协调。
- 未经实际执行的结论一律标「未验证 / `cargo_run=false`」，禁止写成「已通过」。
