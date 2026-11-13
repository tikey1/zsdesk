# ZSDesk Fork — Core UI 改造分支

这是 [`qiu7824/zsdesk`](https://github.com/qiu7824/zsdesk) v0.3.11 的 fork，用于把 UI 从 ZSUI 迁移到 Core UI、把图片预览与 GuoheView 强关联，并采用 Windows 10 / WinUI 3 / Fluent 风格。

> 当前处于 **P0**：已完成 fork、只读调研、UI 设计代码同步与路线图，**尚未改动任何 Rust 业务代码**。
> 完整现状、冲突清单与分阶段计划见 [`docs/10-FORK-ROADMAP-COREUI.md`](docs/10-FORK-ROADMAP-COREUI.md)。

## 目录增补（相对上游）

| 路径 | 内容 |
|---|---|
| `docs/07-UI-DESIGN-WINUI3.md` | Win10 / WinUI 3 风格界面设计方案（与源项目同名文档 SHA256 一致） |
| `docs/10-FORK-ROADMAP-COREUI.md` | 改造路线图、硬冲突、分阶段计划、工程规范 |
| `assets/*.uix`、`assets/themes.json` | 静态布局与主题 token（遵守 D-15：只声明布局 / 样式 / token，无事件属性、无业务脚本） |
| `README-FORK.md` | 本文件 |

这些文件不参与 `cargo build`（未加入任何 crate 的 `include`），不影响上游既有 CI。

## 上游基线

- 提交：`6aec0c2 Release ZSDesk 0.3.11 fixes`
- 语言：Rust 为主，少量 C++ / C / PowerShell / C#
- 许可：MIT，`Copyright (c) 2026 ZSDesk contributors`；上游 `LICENSE` 与 `THIRD-PARTY-NOTICES.txt` 原样保留
- 构建：`cargo test --workspace --release --locked`，Rust 1.89+ / CI 固定 1.99.0；Windows x64

## 注意事项

- 上游 `zsdesk-ui` 与 `zsdesk-app` 依赖仓库外的 `zsui`（`../../../zsui`，MPL-2.0）；CI 通过额外 checkout `qiu7824/zsui` 满足。本地单独 clone 本仓库无法直接编译，需要按 `.github/workflows/build.yml` 的方式同时准备 ZSUI。
- fork 的 GitHub Actions 默认未启用，需在仓库 Actions 页面手动开启。
- 本 fork 目标平台仅 Windows x64，不构建 / 不发布 32 位 x86。

## 工程规范

沿用 `dev-discipline` 与代码审查机制（Gate 0~4 / R1~R12 / 三遍法 / DoD）；每次写完代码做审查留痕；创建 CI 前先复查 UI 设计代码是否正在更新；成果上传云盘需用户明确确认。详见路线图第 5 节。
