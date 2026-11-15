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
**Phase 1 调研与定案 → Phase 4 验收（进行中）** — 事实核对、Spec、契约、迁移矩阵、界面资产、QA 复核均已完成。

- **core-ui 的 C++ 源码构建**：第 9 次（run `43398079661`）/ 第 10 次（run `43545192012`）CI **连续 success**，PE / G2 / S7 全绿 ⇒ M0 最大未知已消除。
- **第 10 次 CI**（commit `1a00121`）靠 `::error::` 探针定位到两个真因：`cargo test --locked` = `Cargo.lock` 手工拼接不合规；Gate S7' = `${{ env.GITHUB_WORKSPACE }}` 求值为空。两处**已在本地修好，尚未推送**。
- **2026-11-14 本轮**又修了三处成员复核阻断项（`theme_contrast_check.py` 空过、`probe.rs` C 字符串缺 NUL、package job 零 annotation）+ 文档台账一致性，**同样尚未推送**。
- **第 11 次 CI 尚未触发**；`package` job 至今仍被上游失败连带跳过，**从未真正执行过**。
- 本轮用户裁定「不用了，继续修复 bug」⇒ **本轮一次性豁免互查闸门**，长期规则见「协作纪律」。

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
| `:root` 注入模块（形态甲 + 13 个 `#[test]`，**均未执行**：本机无 cargo） | `crates/zsdesk-coreui/src/lib/theme.rs` |
| 七个机械门禁（可复跑、失败即停） | `scripts/gates/` |
| ui id 清单（Rust ↔ `.uix` 的单一事实源，32 个 id） | `assets/ui-ids.json` |
| CI 工作流（三 job：rust 10 / core-ui 16 / package 13 步） | `.github/workflows/build.yml` |
| CI 说明与实跑记录 | `docs/ci.md` |
| **剩余工作流程快照** | `docs/15-REMAINING-WORK.md` |

## 进行中
| 谁 | 做到哪 | 状态 |
|---|---|---|
| PM / 前端 / QA / 运维-2 | 4 路自查已回传：前端 **Go**，PM · QA · 运维-2 **No-Go** | 全部已停止工作（2026-11-14 11:28 被批量暂停） |
| 架构师-2 | 架构契约自查被中断，**未回传裁决** | 已停止；按用户新规不续跑自查，遗留 A–E 范围并入后续互查 |
| 交付总监 | 本地修复 6 项静态可复验阻断项 + 台账回写 | 待第 11 次 CI 验证；**未 commit / 未 push** |

本地 HEAD 与远端 `main` 均为 `1a00121`；工作区有未提交改动（见 `git status`）。

## 待决策
| # | 决策点 | 状态 | 推荐项 |
|---|---|---|---|
| Q1 | 包体积红线（10 MB）是否放宽 | 待第 11 次 CI 产出真实 ZIP 体积后拍板 | **暂不放宽**：PM `R02` 实测 9.70 MiB 尚未含 exe 本体与 `routing/`；若超限，优先剔 `licenses/` 240 份副本（约 1.16 MiB），**不砍功能** |
| Q2 | 文本预览上限与编码策略 | 可后置到 M2 | **后置到 M2**，M0/M1 只做图片预览；M2 采用 UTF-8 优先 + GBK 兜底，行数/字节上限届时按真机体验定（`research/brief.md` `R04`） |

> **编号说明**：本表改用 `Q1/Q2`（待决问题），**不占用 D 编号** —— 决策台账里
> 正式 `D14` = 品牌色（`DECISIONS.md:478`）、`D15` = `.uix` 五条契约口径（`:506`）。
> 旧版本表用 D14/D15 指包体积与文本预览，属编号冲突，已更正。

**已关闭**：D21（core-ui 构建退路）—— 第 9 / 10 次 CI 构建成功，退路留 `docs/15` §2.1 备查。
**第十三轮新定**：D22 = 两个 job 用 `setup-python@v5` 钉 3.11、门禁脚本保持 3.9 兼容；
D23 = 可能失败的步骤必须 emit `::error::`（job 日志需 admin，annotations 公开可读）。
**第十四轮（2026-11-14）**：无新增用户决策；仅流程裁定（一次性豁免互查，规则本身不变）+ bug 修复。

## 风险（Top 3）
1. **第 11 次 CI 从未触发**：`cargo test --locked`、五个 S13 门禁、`cargo build`、Gate S7' 对 CI 内 DLL 的比对、package job —— **全部一次都没在真实 CI 跑通**。本机无 cargo/rustc 且缺 `zsui` path 依赖，静态复核不能替代。
2. **package job 的 smoke-probe 是最大未知**：`crates/zsdesk-coreui/examples/probe.rs` **从未被 rustc 编译过**；已修 C 字符串 NUL 结尾，但类型/借用检查、DLL 加载、`.uix` 真解析只能由 CI 回答。该 job 原 12 步零 `::error::`，本轮已补 5 步 + job 级兜底。
3. **多处失败模式是「静默」**（注入被吃掉 / 宿主标签错 / 句柄失效）—— 零报错、界面不对。已整理成 `docs/15` §4.2 的症状→根因对照表。

## 下一步
1. **上传并触发第 11 次 CI**（对外动作，需用户点头）：按 `docs/15` §5.1 的公开 annotations 通道判读 —— ① `cargo test` 是否转绿；② 五个 S13 门禁首次执行结果；③ Gate S7' 对 CI 内 DLL 是否通过；④ package job 是否首次产出 ZIP。
2. **M1 开工**：Rust 侧真正用 core-ui 装上面板 —— 读 `assets/*.uix` → 注入 `:root` → `ui_page_load_string` → 取 32 个句柄 → 自绘列表/导航树 → GuoheView 预览。
3. **主题切换链路**：`ui_theme_set_mode` → 重建页面 → 重取全部 32 个句柄 + 在新 page 上重新注册 `ui_page_on_widget_mount` 钩子 + 重读 AccentColor 并重调 `ui_theme_set_accent_hex`。

## 协作纪律（已立）
- **【现行，2026-11-14 用户裁定】每天代码编写前必须先通过「成员互查」**：成员交叉复核他人产出/交付物并给出明确 Go / No-Go；未过闸门不写码、不 commit、不 push。
- **【本轮一次性豁免】** 用户 2026-11-14 明确「不用了，继续修复 bug」⇒ 本轮跳过互查直接修；**长期规则不变**，不视为废止。
- **【已取消】** 「重新激活所有成员进行代码自查」不再作为持续要求，仅当轮一次性生效。自查（作者复核自己）≠ 互查（交叉复核他人），当前只保留互查。
- **上传前置判据**：成员互查 Go + 静态可复验阻断项清零；**第 11 次 CI 是上传之后的最终验证，不得拿它倒堵上传**（否则 CI 永远触发不了）。
- **git 操作统一由交付总监执行**；成员只写文件。已发生两次成员抢先 commit 导致 push 被 `&&` 短路。
- **任何人不得对他人正在编辑的文件做批量改写**（sed -i / 全文替换 / 格式化）；确需改动先 SendMessage 协调。
- 未经实际执行的结论一律标「未验证 / `cargo_run=false`」，禁止写成「已通过」。
