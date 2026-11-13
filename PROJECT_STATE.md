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
**Phase 1 调研与定案 → Phase 2 设计（进行中）** — 事实核对完成、对话决策已拍板、ABI 闸门已推进到「符号级核实」；架构师 Spec 与运维 CI 工作流并行产出中。

## 已完成
| 产物 | 路径 |
|---|---|
| 事实核对底稿（10 组实测：仓库结构、core-ui 构建约束、本机 GuoheView PE 导出表、版本差异、冲突清单、zsdesk 分层、CI 模板） | `research/facts.md` |
| 决策台账（决策优先级规则 + D00~D05 + C1~C5 判定 + T1~T7） | `DECISIONS.md` |
| **Core UI 导出符号清单（本机实测 2006 导出 / 567 纯 C API）** | `docs/05-COREUI-EXPORTS.txt` |
| **Core UI ABI 核对笔记（18 条 VERIFIED / 12 条 UNVERIFIED）** | `docs/05-COREUI-ABI-NOTES.md` |
| 用户 fork 既有资产同步（路线图 / UI 设计文档 / 3 个 .uix / themes.json / README-FORK） | `docs/10-FORK-ROADMAP-COREUI.md`、`docs/07-UI-DESIGN-WINUI3.md`、`assets/` |
| core-ui revision 钉扎 | `packaging/core-ui-revision.txt` = `1840d689b51a32d392dbf06b8d594aef96696a80` |
| git 基线（138 个 Rust 文件 / 42,150 行，commit `32cfe51`） | 本仓库 |

## 进行中
| 谁 | 做到哪 | 卡在哪 |
|---|---|---|
| 高见远（首席架构师） | 技术 Spec / crate-layout / FFI 契约 / 迁移矩阵 / CI 契约 / 里程碑（对齐 P1~P4） | 待回传 |
| 卜宕机（运维） | GitHub Actions 工作流（仅 full x64 收窄 + core-ui 从源码构建 + x64 断言 + 便携包） | 待回传 |
| 许清楚（PM） | 调研简报 R01–R10 | 待回传 |

## 待决策
| # | 决策点 | 候选 | 推荐 | 状态 |
|---|---|---|---|---|
| D06 | 上游非图片类预览（Office/WPS/文本/PDF）在 core-ui 下的落点 | 自绘预览 / 独立顶层原生窗口 / 交给外部程序 | 待架构师出方案 | 待定 |
| D07 | 左侧 Shell 导航树替代 | 自绘导航树 / core-ui 控件 / 去掉 | 待架构师出方案 | 待定 |
| D08 | core-ui 链接方式 | 链接期 `core-ui.lib` / 运行时 `LoadLibrary` | 待架构师出方案 | 待定 |
| D09 | ABI 真机核对清单（U01~U12）执行时机 | M0 内做 / M1 内做 | **M1 前必须完成 U01~U03 最小闭环** | 待定 |

## 风险（Top 3）
1. **本机无 Rust/C++ 工具链**（实测仅 git/curl/python3）→ 编译验证只能靠 GitHub Actions；且 **fork 的 Actions 默认未启用，需用户在仓库页面手动开启**，否则 M0 无法闭环。
2. **core-ui 是自绘、不可嵌原生 HWND、无虚拟列表控件** → 上游「Shell 原生列表 + 原生导航树 + 原生预览宿主 + ZSUI 虚拟列表」四处能力全部落空，呈现层重写量 ≈ 12–18k 行［估算］。
3. **ABI 参数语义未核对**（U01~U12）→ 编造签名违反用户 R1 红线。缓解：已把符号存在性离线核实（18 条 VERIFIED），剩余语义按「最小探针逐项验」推进，Spec 中标 UNVERIFIED。

## 下一步
1. 收架构师 Spec + 运维 CI 工作流 → 汇编后交用户确认一次 → 锁定。
2. 推送到 `tikey1/zsdesk` fork，**由用户手动开启 Actions**，跑出第一次 full x64 绿灯。
3. 进 P2：按 crate 分阶段拆 `zsui` path 依赖，落 Core UI 接入骨架 + WinUI3 静态布局与 token。
