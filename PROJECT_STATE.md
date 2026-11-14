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
**Phase 1 调研与定案 → Phase 4 验收（进行中）** — 事实核对、Spec、契约、迁移矩阵、界面资产、QA 复核均已完成；
**M0（CI 编出 x64 程序）：core-ui 的 C++ 源码构建已由第 9 次 CI（run `43398079661`）证实成功**，
G2 / S7 全绿；剩 rust job 的 `cargo test` 与 Gate S7' 两处失败待复验，package job 尚未真正执行过。

## 已完成
| 产物 | 路径 |
|---|---|
| 事实核对底稿（10 组实测 + §7.11 复核修正） | `research/facts.md` |
| 调研简报 R01–R10（PM） | `research/brief.md` |
| 决策台账（优先级规则 + D00~D13 + T1~T7） | `DECISIONS.md` |
| **总 Spec**（总览 + D06~D12 决策索引 + 契约索引） | `SPEC.md` |
| core-ui 导出符号清单（2006 导出 / 567 纯 C API） | `docs/05-COREUI-EXPORTS.txt` |
| Core UI ABI 核对笔记（18 VERIFIED / 12 UNVERIFIED） | `docs/05-COREUI-ABI-NOTES.md` |
| `.uix` 标记标签权威支持表 + 主题 token 冲突 | `docs/11-COREUI-MARKUP-TAGS.md` |
| 界面资产修正对照表 + 真机首查清单（前端） | `docs/12-UIX-TAG-FIX.md`、`docs/13-UIX-FIRST-RENDER-CHECKLIST.md` |
| **QA 独立复核报告**（8 项裁决 + 3 阻塞 + 7 重要 + 第二轮） | `docs/14-QA-UIX-REVIEW.md` |
| 技术契约（FFI / 目录 / 迁移矩阵 / CI / 里程碑 / ADR） | `spec/` |
| FFI 绑定层（Rust，运行时动态加载） | `crates/zsdesk-coreui/` |
| `:root` 注入模块（形态甲 + 9 条单元测试） | `crates/zsdesk-coreui/src/lib/theme.rs` |
| 六个机械门禁（可复跑、失败即停） | `scripts/gates/` |
| ui id 清单（Rust ↔ `.uix` 的单一事实源） | `assets/ui-ids.json` |
| CI 工作流（三 job：rust / core-ui / package） | `.github/workflows/build.yml` |
| CI 说明与实跑记录 | `docs/ci.md` |
| **剩余工作流程快照**（本次收尾） | `docs/15-REMAINING-WORK.md` |

## 进行中
**本轮（2026-11-13 收尾）已停止代码编写**，成果全部落盘并推送（`e631cd2`）；
2026-11-14 仅补第 10 次 CI 前的收尾修复与状态回写，不新增功能代码。
| 谁 | 做到哪 | 状态 |
|---|---|---|
| 高见远（架构师） | Spec v3、契约两栏、D10~D12、D15、D17~D20 落定 | ✅ 本轮收口 |
| 严过关（QA） | 第九轮复核：抓出 BOM / `:root` 外定义 / 门禁无执行器 / 戳记覆盖 4 个 P1，全部已修 | ✅ 本轮收口 |
| 贾思敏（前端） | `nav_host` → `<custom>`；R5 宿主判定表补齐；句柄数 27→32 全文同步 | ✅ 本轮收口 |
| 卜宕机（运维） | CI 独立评审：R0（package job 缺 zsui）等 P0 已修 | ✅ 本轮收口 |
| 大湾区靓仔（交付总监） | CI 根因定位 + 门禁接线 + 第 9 次 CI 结果回写 + 三层探针（3.9 兼容 / 钉 Python 3.11 / `::error::`） | ⏸️ **代码暂停编写**，等第 10 次 CI |

## 待决策
| # | 决策点 | 状态 |
|---|---|---|
| D14 | 包体积红线（10 MB）是否放宽 | **待 CI 实测后**由用户拍板（现为估算，不凭估算问） |
| D15 | 文本预览上限与编码策略 | 可后置到 M2，PM 已给建议 |

**已关闭**：D21（core-ui 构建退路）—— 第 9 次 CI 构建成功，不需走退路，退路留 `docs/15` §2.1 备查。
**本轮新定**：D22 = 两个 job 用 `setup-python@v5` 钉 3.11、门禁脚本保持 3.9 兼容（双保险）；
D23 = 可能失败的步骤必须 emit `::error::`（job 日志需 admin，annotations 公开可读）。

## 风险（Top 3）
1. **Gate S7' 失败原因未定位** —— 第 9 次 CI 里失败且外部看不到输出。已按「3.9 兼容 + 钉 Python 3.11 + `::error::` 探针」三层处置，待第 10 次 CI 自曝真因。详见 `docs/15` §2.1c。（~~core-ui 构建不过~~ 已由第 9 次 CI 消除。）
2. **本机无 Rust/C++ 工具链** → 一切编译验证只能靠 CI。本轮已用 tree-sitter 做 Rust 语法级验证（零错误），但**类型级/借用检查器层面的验证只能靠 CI 的 `cargo test`**。
3. **多处失败模式是「静默」**（注入被吃掉 / 宿主标签错 / 句柄失效）—— 零报错、界面不对。已整理成 `docs/15` §4.2 的症状→根因对照表。

## 下一步
1. **读第 10 次 CI**（本次推送触发）：按 `docs/15` §5.1 的公开 annotations 通道判读 —— ① rust job `cargo test` 是否转绿；② Gate S7' 是否转绿（若红，annotations 里有真因）；③ package job 是否首次真正执行并产出 ZIP。
2. **M1 开工**：Rust 侧真正用 core-ui 装上面板 —— 读 `assets/*.uix` → 注入 `:root` → `ui_page_load_string` → 取 32 个句柄 → 自绘列表/导航树 → GuoheView 预览。
3. **主题切换链路**：`ui_theme_set_mode` → 重建页面 → 重取全部 32 个句柄 + 在新 page 上重新注册 `ui_page_on_widget_mount` 钩子 + 重读 AccentColor 并重调 `ui_theme_set_accent_hex`。

## 协作纪律（已立）
- **git 操作统一由交付总监执行**；成员只写文件。已发生两次成员抢先 commit 导致 push 被 `&&` 短路。
- **任何人不得对他人正在编辑的文件做批量改写**（sed -i / 全文替换 / 格式化）；确需改动先 SendMessage 协调。
- 未经实际执行的结论一律标「未验证 / `cargo_run=false`」，禁止写成「已通过」。
