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
**M0（CI 编出 x64 程序）只剩 core-ui 的 C++ 构建这一个未知**，已连续三次迭代修复。

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
| FFI 绑定骨架（Rust，运行时动态加载） | `spec/rust-ffi-skeleton/` |
| 13 项机械门禁（可复跑、失败即停） | `scripts/gates/` |
| CI 工作流（三 job：rust / core-ui / package） | `.github/workflows/build.yml` |
| CI 说明与实跑记录 | `docs/ci.md` |

## 进行中
| 谁 | 做到哪 | 卡在哪 |
|---|---|---|
| 高见远（架构师） | Spec v3、`ci-contract.md` v3 两栏、D10/D11/D12 定稿 | 等 `:root` 注入契约与戳记归属收口 |
| 严过关（QA） | 第二轮：B2 确认、I5 修对比度脚本、CI 契约一致性核对 | 等前端 B3 重贴输出 |
| 贾思敏（前端） | 修 B1（`:root` 注入说明）、B3（重贴输出）、I4（删字重提 14px） | 等 B1 合并口径落地 |
| 卜宕机（运维） | `spec/ci-implementation-notes.md` 独立评审 | 待回传 |
| 大湾区靓仔（交付总监） | 盯 CI；等 core-ui 编译结果 | 第四次 CI `d2964ce` 运行中 |

## 待决策
| # | 决策点 | 状态 |
|---|---|---|
| D14 | 包体积红线（10 MB）是否放宽 | **待 CI 实测后**由用户拍板（现为估算，不凭估算问） |
| D15 | 文本预览上限与编码策略 | 可后置到 M2，PM 已给建议 |

## 风险（Top 3）
1. **core-ui 的 C++ 构建在 CI 上不过** —— 已连续三次失败，根因从「可能是 FetchContent 联网」收敛到「脚本 1 秒即抛错 → vcvars 环境抓取不稳」。第四次改为 `ilammy/msvc-dev-cmd` 导入 MSVC 环境 + 直接调 cmake + `-DFETCHCONTENT_SOURCE_DIR_QUICKJS` 消除下载；失败时用 `::error::` 做成 check-run annotation（外部可经公开 API 读到）。
2. **本机无 Rust/C++ 工具链** → 一切编译验证只能靠 CI；且本机 GitHub API 未认证额度仅 60/小时，CI 查询要靠 WorkBuddy 服务端通道或等额度恢复。
3. **QA 已确认的三个 P0（`<window>` 自闭合 / `text=` 无效 / `window{}` 不匹配）会让首版界面整页加载失败** —— 前端已修，QA 已复核通过；剩余 B1（`:root` 注入位置）与 B3（输出不可复现）在修。

## 下一步
1. 第四次 CI 出结果 → 读 error annotation 定位 core-ui 编译/配置的确切问题 → 修 → 直到 core-ui job 绿，拿到便携 ZIP。
2. 前端修完 B1/B3/I4 → QA 第三轮复核 → `assets/` 入库 → M1 开工（core-ui 窗口 + 自绘文件列表）。
3. M1 的第一步按架构师的最早探测顺序做：`ui_core_version*` → `ui_init_with_theme` → `ui_page_load_string` → `ui_page_open_window` → `ui_run`，逐项最小验证 U01~U03。

## 协作纪律（已立）
- **git 操作统一由交付总监执行**；成员只写文件。已发生两次成员抢先 commit 导致 push 被 `&&` 短路。
- **任何人不得对他人正在编辑的文件做批量改写**（sed -i / 全文替换 / 格式化）；确需改动先 SendMessage 协调。
- 未经实际执行的结论一律标「未验证 / `cargo_run=false`」，禁止写成「已通过」。
