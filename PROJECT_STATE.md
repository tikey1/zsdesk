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

- **core-ui 的 C++ 源码构建**：第 9 / 10 / **11 次 CI 连续 success**。第 11 次连 **Gate S7' 也转绿** ⇒ `${{ env.GITHUB_WORKSPACE }}` 空值的修复生效，M0 最大未知已消除。
- **第 10 次 CI**（`1a00121`）定位的两真因（`Cargo.lock` 不同步 / `env.GITHUB_WORKSPACE` 空值）已在 `aa7e9b5` 修复并推送，第 11 次 CI 证实修复有效。
- **第 11 次 CI**（run `43666238286`，commit `aa7e9b5`）：core-ui job **全绿**；rust job 在 **Test** 失败，真因是 `zsdesk-coreui` 的 5 个 `mod` 声明找不到文件（`error[E0583]`）；package job 再次 skipped。
- **2026-11-15 本轮修复（尚未推送）**：给 5 个模块加 `#[path = "lib/xxx.rs"]`；新增机械门禁 `rust_module_layout_check.py` 并接进 `build.yml`，位置在 `cargo test` **之前**（rust job 10 → 11 步）。本地六个门禁 / 负控 / YAML / pwsh 语法全绿。
- `package` job 至今**从未真正执行过**；`probe.rs` 从未被 rustc 编译过 —— 这两项只能由第 12 次 CI 回答。
- 互查：沿用 2026-11-14 确立的**本轮一次性豁免**（用户「不用了」「继续上传」「根据 action 返回的问题重新修改代码」），长期规则见「协作纪律」。

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
| 八个门禁脚本（7 个独立可执行门禁含 S14 + 1 个 PE 解析辅助库） | `scripts/gates/` |
| ui id 清单（Rust ↔ `.uix` 的单一事实源，32 个 id） | `assets/ui-ids.json` |
| CI 工作流（三 job：rust 11 / core-ui 16 / package 13 步） | `.github/workflows/build.yml` |
| CI 说明与实跑记录 | `docs/ci.md` |
| **剩余工作流程快照** | `docs/15-REMAINING-WORK.md` |

## 进行中
| 谁 | 做到哪 | 状态 |
|---|---|---|
| PM / 前端 / QA / 运维-2 | 4 路自查已回传：前端 **Go**，PM · QA · 运维-2 **No-Go** | 全部已停止工作（2026-11-14 11:28 被批量暂停） |
| 架构师-2 | 架构契约自查被中断，**未回传裁决** | 已停止；按用户新规不续跑自查，遗留 A–E 范围并入后续互查 |
| 交付总监 | 第 11 次 CI 真因（E0583）已修 + 新增 S14 门禁 + 状态回写 | 本地静态复验全绿；**未 commit / 未 push**，待第 12 次 CI |

本地 HEAD 与远端 `main` 均为 `aa7e9b5`；工作区有未提交改动（见 `git status`）。

## 待决策
| # | 决策点 | 状态 | 推荐项 |
|---|---|---|---|
| Q1 | 包体积红线（10 MB）是否放宽 | 待首次产出真实 ZIP 体积后拍板 | **暂不放宽**：PM `R02` 实测 9.70 MiB 尚未含 exe 本体与 `routing/`；若超限，优先剔 `licenses/` 240 份副本（约 1.16 MiB），**不砍功能** |
| Q2 | 文本预览上限与编码策略 | 可后置到 M2 | **后置到 M2**，M0/M1 只做图片预览；M2 采用 UTF-8 优先 + GBK 兜底，行数/字节上限届时按真机体验定（`research/brief.md` `R04`） |

> **编号说明**：本表用 `Q1/Q2`（待决问题），**不占用 D 编号** —— 决策台账里
> 正式 `D14` = 品牌色（`DECISIONS.md:478`）、`D15` = `.uix` 五条契约口径（`:506`）。

**已关闭**：D21（core-ui 构建退路）—— 第 9 / 10 / 11 次 CI 构建成功，退路留 `docs/15` §2.1 备查。
**第十三轮新定**：D22 = 钉 Python 3.11 + 门禁脚本保持 3.9 兼容；
D23 = 可能失败的步骤必须 emit `::error::`（job 日志需 admin，annotations 公开可读）。
**第十四 / 十五轮**：无新增用户决策，仅流程裁定与 bug 修复，见 `DECISIONS.md` 变更记录。

## 风险（Top 3）
1. **`cargo test --locked` 从未在 CI 转绿**：第 11 次暴露的 E0583 只是第一层——5 个模块文件此前**从未被 rustc 编译过**，修完路径后才轮到类型 / 借用检查。`zsdesk-coreui` 里仍可能有下一批编译错误，只能由 CI 逐轮回答。
2. **package job 的 smoke-probe 是最大未知**：`crates/zsdesk-coreui/examples/probe.rs` **从未被 rustc 编译过**；已修 C 字符串 NUL 结尾，但 DLL 加载、`.uix` 真解析只能由 CI 回答。该 job 已补 5 步 `::error::` + job 级兜底定位。
3. **多处失败模式是「静默」**（注入被吃掉 / 宿主标签错 / 句柄失效）—— 零报错、界面不对。已整理成 `docs/15` §4.2 的症状→根因对照表。

## 下一步
1. **上传并触发第 12 次 CI**（对外动作，需用户点头）：按 `docs/15` §5.1 的公开 annotations 通道判读 —— ① `cargo test` 是否转绿（E0583 已修，可能暴露下一批编译错误）；② 五个 S13 门禁 + 新 S14 门禁首次执行；③ `cargo build`；④ package job 是否首次产出 ZIP。
2. **M1 开工**：Rust 侧真正用 core-ui 装上面板 —— 读 `assets/*.uix` → 注入 `:root` → `ui_page_load_string` → 取 32 个句柄 → 自绘列表/导航树 → GuoheView 预览。
3. **主题切换链路**：`ui_theme_set_mode` → 重建页面 → 重取全部 32 个句柄 + 在新 page 上重新注册 `ui_page_on_widget_mount` 钩子 + 重读 AccentColor 并重调 `ui_theme_set_accent_hex`。

## 协作纪律（已立）
- **【现行，2026-11-14 用户裁定】每天代码编写前必须先通过「成员互查」**：成员交叉复核他人产出/交付物并给出明确 Go / No-Go；未过闸门不写码、不 commit、不 push。
- **【一次性豁免的适用范围】** 用户 2026-11-14「不用了，继续修复 bug」、11-15「继续上传」「根据 action 返回的问题重新修改代码」⇒ 这几轮按连续的同一次 CI 修复闭环处理，跳过互查直接修；**长期规则不变**，不视为废止。后续开启新的开发日须重新过互查。
- **【已取消】** 「重新激活所有成员进行代码自查」不再作为持续要求，仅当轮一次性生效。自查（作者复核自己）≠ 互查（交叉复核他人），当前只保留互查。
- **上传前置判据**：成员互查 Go + 静态可复验阻断项清零；**CI 是上传之后的最终验证，不得拿它倒堵上传**（否则 CI 永远触发不了）。
- **git 操作统一由交付总监执行**；成员只写文件。已发生两次成员抢先 commit 导致 push 被 `&&` 短路。
- **任何人不得对他人正在编辑的文件做批量改写**（sed -i / 全文替换 / 格式化）；确需改动先 SendMessage 协调。
- 未经实际执行的结论一律标「未验证 / `cargo_run=false`」，禁止写成「已通过」。
