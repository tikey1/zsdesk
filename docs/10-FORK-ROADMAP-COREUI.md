# Fork 改造路线图：ZSDesk → Core UI 版本

- 日期：2026-11-13
- Fork：`tikey1/zsdesk`（public，默认分支 `main`，上游基线 `6aec0c2 Release ZSDesk 0.3.11 fixes`；fork 当前 HEAD 以 `git rev-parse origin/main` 为准）
- 上游：`qiu7824/zsdesk` v0.3.11，MIT，`Copyright (c) 2026 ZSDesk contributors`
- 状态：**P0 已完成；C3（全量 x64）已拍板；C1 / C2 / C4 / C5 待确认，P1~P4 待执行**

## 1. 上游现状（只读调研结论）

| 项 | 事实 |
|---|---|
| 定位 | Native Windows tabbed file manager and desktop fences, built with Rust and Win32 |
| 语言构成 | Rust 2,499,609 B；C++ 243,350 B；C 77,615 B；PowerShell 36,023 B；C# 9,046 B；CMake / Inno Setup 少量 |
| Workspace | 5 个 crate：`zsdesk-core` / `zsdesk-platform` / `zsdesk-app` / `zsdesk-ui` / `zsdesk-windows` |
| UI 依赖 | **`zsui = { path = "../../../zsui" }`（仓库外本地目录，MPL-2.0）**；CI 通过额外 checkout `qiu7824/zsui` + `packaging/zsui-revision.txt` 满足该 path 依赖 |
| Windows 依赖 | `windows 0.62.2` / `windows-sys 0.59` + Direct2D / DirectWrite / WIC / DWM / Shell / Search |
| 工具链 | `rust-version = 1.89`，CI 固定 `dtolnay/rust-toolchain@1.99.0`，`--locked` |
| 图片预览 | 链路完整：`window/{quicklook, quick_preview, hover_preview, preview_process, preview_handlers, preview_selection, image_decode, thumbnail_regression, paint}.rs` |
| 其他组件 | 桌面栅栏（desktop fences）、Everything 快速搜索、Listary C# 插件、MinHook(BSD)、SQLite、5 个 edition（network / desktop / full / themed / desktop-box） |
| 目标相关代码 | 仓库内 **无** Core UI、GuoheView、.NET / dotnet 相关代码 |

## 2. 需求理解（请确认或纠正）

按此前要求，本 fork 的目标理解为：

1. UI 改用 **Core UI**（替换 ZSUI）；
2. 图片预览与 **GuoheView** 强关联，并作为运行前提调用；
3. 界面风格 **Windows 10 / WinUI 3 / Fluent**，符合用户习惯；
4. 代码以 **Rust 为主**，必要时调用 C++ v14 运行库；
5. 必要时通过 **.NET 10** 提供跨平台应用运行环境；
6. 代码编写与审查要求沿用既有规范；
7. 只做 x86-64，不考虑 32 位 x86；
8. 成果随时上传云盘，做好 UI 设计代码同步与合并。
9. **（已拍板）只做全量 x64 版本**：唯一交付形态为 `full` edition + `windows-x64`，不构建、不测试、不发布其余 edition 与 32 位 x86。详见 §3.6。

## 3. 硬冲突（C3 已拍板，C1 / C2 / C4 / C5 待确认）

### C1 .NET 10「跨平台」与 Windows 原生栈冲突（阻塞）

Core UI 是 Windows DLL，GuoheView 是 Windows 可执行程序，WinUI 3 / Fluent 是 Windows 设计语言，上游又是纯 Win32 + Direct2D + WIC。**引入 .NET 10 运行时无处可用，且会打破「单进程 / 原生 / ≤10 MB / 唯一第三方二进制」约束**，还会带来运行时分发与体积问题。

- 建议（默认）：**不引入 .NET 10**；Rust 主程序 + 原生 Win32，C++ v14 运行库按需（VC++ 2015-2022 Redistributable x64，`core-ui.dll` 本身就依赖 `MSVCP140` / `VCRUNTIME140` / `VCRUNTIME140_1`）。
- 若确有托管组件需求，请指明具体组件与用途，再单独评估（C# 目前只有 `native/listary` 插件，9 KB，非主链路）。

### C2 Core UI ABI 闸门 K-01~K-18 尚未真机核对（阻塞）

已归档的 `tikey1/ceshi` 里 `src/coreui/symbols.rs` 仍为 `ABI_VERIFIED = false`，程序走 `PureCApi` 自检降级模式、**不显示窗口**。把 Core UI 接进本 fork 会撞上同一堵墙：不核对函数签名就只能继续降级，编造签名违反 R1 红线。

- 建议：P1 先在具备 Core UI 1.9.0.346 的真机核对 K-01~K-18，产出 `verified.rs`；否则 Core UI 接入只能停在「可编译的骨架」。
- 可复用资产：`vendor/core-ui.dll`（4,782,536 B）、`docs/05-COREUI-EXPORTS.txt`（2006 个导出符号）、`docs/05-ABI-NOTES.md`、现有 `coreui/*` 模块。

### C3 上游功能保留范围 ~~未定~~ → **已拍板：全量保留**

用户已明确「只做全量 x64 版本的」。即 **不裁剪上游功能**：桌面栅栏、Everything 快速搜索、Listary 插件、MinHook、SQLite、网络 / 桌面能力全部保留，改造在完整代码基上做（跨约 138 个 Rust 文件），不做「只留文件管理器核心」的收窄。

- 影响：P2 的 ZSUI → Core UI 替换范围扩大，必须按 crate 分阶段推进并保留每条业务链路的回归测试；不能靠删功能来规避替换工作量。
- 唯一收窄的是**交付形态**（见 §3.6），不是功能集。

### C3.6 平台与交付形态（已拍板）

| 项 | 结论 | 依据 |
|---|---|---|
| 唯一 edition | **`full`** | `crates/zsdesk-app/src/edition.rs` 的 `key()`：`NETWORK && DESKTOP && !THEMED && !DESKTOP_ONLY` → `"full"`（名称「全量版」） |
| features | `network + desktop`（即 `default`） | `crates/zsdesk-app/Cargo.toml` `[features]` 与 `scripts/package.ps1` 的 `$definitions.full = @{ Features=@('network','desktop') }` |
| 目标平台 | **仅 `windows-x64`** | `package.ps1` 头部即写明 "Builds verified Windows x64 editions"；`VCToolsRedistDir x64`、Everything engine pin `platform=windows-x64`、输出名 `ZSDesk-$version-$edition-x64`、manifest `platform='windows-x64'` |
| 不构建 / 不发布 | `standard` / `network` / `desktop` / `themed` / `desktop-box` 单版；**任何 32 位 x86** | 用户明确要求 |
| C++ v14 运行库 | 沿用上游既有机制 | `package.ps1` 已强制校验 `Visual C++ x64 Redistributable`（`msvcp140.dll` 等），缺 `-RuntimeDir` 直接 throw；`core-ui.dll` 同样依赖 `MSVCP140` / `VCRUNTIME140` / `VCRUNTIME140_1` |
| CI 调整 | 收窄为只跑 `full` x64 的 build / test / clippy + 打包 | 上游 `build.yml` 现有「Check all editions」「Test theme and desktop-only editions」两步需删除或改为仅 full |
| 打包产物 | 单一 `ZSDesk-<version>-full-x64` 便携包 / 安装包，逐文件 `bytes + SHA-256` manifest | 沿用上游 `package.ps1` 已有清单机制，与本项目的 `stage-portable.ps1` 实践一致 |

> 结论：x64-only 与 C++ v14 运行库**上游已经天然满足**，这部分不需要额外设计；需要改的是「多 edition 并行构建」收窄为「仅 full」。

### C4 ZSUI 是仓库外 MPL-2.0 依赖

`zsdesk-ui` 与 `zsdesk-app` 都直接依赖 `zsui`，且 path 指向仓库外。移除它既是技术替换，也能降低 MPL-2.0 的源码公开义务；但在替换完成前，fork 的 CI 仍依赖 `qiu7824/zsui` checkout。需要决定过渡期是否保留双通道。

### C5 fork 可见性与 CI

GitHub fork 公开仓库默认为 **public**；fork 的 Actions 默认**未启用**，需在 Actions 页面手动开启。若需要 private 或需要 CI 立即生效，请明确。

## 4. 分阶段计划（P1 起均需先确认方案）

| 阶段 | 内容 | 前置 |
|---|---|---|
| **P0（本轮已完成）** | 创建 fork；SSH 克隆；只读调研；同步合并 UI 设计文档与静态资源；产出本路线图 | 无 |
| **P1** | 拍板 C1 / C2 / C4 / C5；建立 Gate 0~4 / R1~R12 门禁与审查模板；**把 `build.yml` 从多 edition 收窄为仅 `full` x64**（删「Check all editions」「Test theme and desktop-only editions」），保留 `--locked` / 逐文件 SHA-256 / VC++ x64 Redist 校验；确认 UI 更新状态后启用并创建 CI | C1 / C2 / C4 / C5 |
| **P2** | 在**全量功能基线上**按 crate 分阶段拆 `zsui` path 依赖，建立 Core UI 接入骨架；WinUI 3 静态布局 + 主题 token 落地；每条业务链路配回归测试，保持可编译 | K-01~K-18 核对结论 |
| **P3** | GuoheView 强关联图片预览（复用 `preview_process` / `quicklook` / `image_decode` 链路），补降级通道与错误提示 | P2 |
| **P4** | 完整回归（full x64 的 build / test / clippy / Gate 1 / 真机冒烟）→ 单一 `ZSDesk-<ver>-full-x64` 便携包 → Release → 审查留痕 → 云盘归档 | P3 |

## 5. 沿用工程规范（不打折）

- `dev-discipline`：ANALYZE → PLAN → CONFIRM → BACKUP → IMPLEMENT → REVIEW → DELIVER；只改该改的，找到真原因再改。
- 代码审查：Gate 0~4、R1~R12、三遍法、DoD；意见带 🔴 / 🟡 / 💭 + 文件:行号 + 原因 + 建议；**作者不得自审**、本机无 Rust 工具链时 `cargo_run=false` 必须如实标注，不得把「未验证」写成「已通过」。
- **创建 CI 前先查 UI 设计代码是否正在更新**：分支 / PR / 活跃 run / 他人提交 / 未提交 UI 改动 / 设计稿与已合入文档哈希一致性；若在更新则等完成 → 合并 → 再建 CI。
- 每次写完代码都跑机械门禁并出审查留痕；推送优先 SSH，PAT 只用于 API 且用完即删。
- 云盘上传按项目规则需用户逐次明确确认。

## 6. 本轮已落地（P0）

- Fork：`tikey1/zsdesk`（public，`main`），SSH 克隆到本地 `zsdesk-fork/`。
- UI 设计代码同步：`docs/07-UI-DESIGN-WINUI3.md`、`assets/{main,preview,about}.uix`、`assets/themes.json`，逐文件 SHA256 与源项目一致。
- 本路线图 `docs/10-FORK-ROADMAP-COREUI.md` 与 `README-FORK.md`。
- **未改动任何 Rust 业务代码**，未引入新依赖，未添加 CI 工作流，未上传云盘。

## 7. 许可与致谢

上游 ZSDesk 为 MIT（`LICENSE` / `Copyright (c) 2026 ZSDesk contributors`）。本 fork 保留上游许可全文与 `THIRD-PARTY-NOTICES.txt`。后续引入 Core UI（MIT）需按其分发义务保留许可全文与构建差异说明；GuoheView 为外部可选程序，不随包分发，仅做运行时调用与降级声明。ZSUI 为 MPL-2.0，若替换完成则不再随包分发。
