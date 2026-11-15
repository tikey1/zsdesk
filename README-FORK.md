# ZSDesk Fork — Core UI 改造分支

这是 [`qiu7824/zsdesk`](https://github.com/qiu7824/zsdesk) 的 fork（建 fork 时基于 v0.3.11，
2026-11-15 已同步上游 **v0.3.12** `cb465e8`），用于把 UI 从 ZSUI 迁移到 Core UI、
把图片预览与 GuoheView 强关联，并采用 Windows 10 / WinUI 3 / Fluent 风格。

> **当前状态（2026-11-15）**：M0 验收闭环已达成 —— **第 15 次 CI**（run `43673089804`，
> commit `63685f9`）三个 job 全部 `success`；纯文档提交 `9a3628b` 触发的**第 16 次 CI**
> （run `43674268118`）同样全绿，产物 `ZSDesk-full-x64` 4,941,625 B ≈ 4.71 MiB（**历史值**）。
>
> **上游 v0.3.12 同步收口 / M0 回归验证通过**：`--no-ff` merge 的 merge commit `aa44acc`
> 已推送，触发的**第 17 次 CI**（run `43678206351`）三个 job 逐 step 全 `success`、
> 唯一 `skipped` 是条件步骤「Package job failure locator」（job 成功时跳过，属预期），
> 4 个 artifact 齐全：`ZSDesk-full-x64` **5,594,915 B ≈ 5.34 MiB**（**当前值**，未触 10 MB 红线）、
> `zsdesk-app-full-x64` 3,006,887 B、`core-ui-x64` 1,981,859 B、`core-ui-build-log` 22,420 B。
> 这是扩大代码面上的回归验证，**不是** M1 前置达成、功能里程碑或 M0 重新达成。
> 详见 `PROJECT_STATE.md` 与 `docs/ci.md` §6。
>
> **2026-11-15 上游同步**：已用 `--no-ff` merge 合入 `qiu7824/zsdesk` v0.3.12
> （2 提交 / 108 文件，`ec7e5d2` + `cb465e8`），保留本 fork 重写的 CI、新 crate
> `crates/zsdesk-coreui/`、`.uix` 资产与全部机械门禁。合并前的全绿基线 `9a3628b`
> 由本地 tag / 备份分支 `pre-upstream-0.3.12` 固化；远端 `origin/main` 现为 `aa44acc`。
>
> **已知限制（如实披露）**：
> ① Rust 侧目前是 FFI 绑定层 + 冒烟探针，**全量业务功能（M1）尚未装配**；
>    ZIP 里的 exe 跑通的是构建链路，不是功能完整的成品。
> ② 界面**未在真机渲染验收** —— `docs/13-UIX-FIRST-RENDER-CHECKLIST.md` 清单尚未执行；
>    32 个句柄是否全部取到、`:root` 注入是否被 core-ui 先定义者吃掉，仍需人工核对。
> ③ 目前只有 Actions artifact（默认 90 天过期），**尚无 Release 永久分发**；
>    Inno Setup 安装包（`packaging/windows/zsdesk.iss`）也未接进 CI 编译。
>
> 完整现状、冲突清单与分阶段计划见 [`docs/10-FORK-ROADMAP-COREUI.md`](docs/10-FORK-ROADMAP-COREUI.md)；
> 剩余工作快照见 `docs/15-REMAINING-WORK.md`。

## 目录增补（相对上游）

| 路径 | 内容 |
|---|---|
| `docs/07-UI-DESIGN-WINUI3.md` | Win10 / WinUI 3 风格界面设计方案（与源项目同名文档 SHA256 一致） |
| `docs/10-FORK-ROADMAP-COREUI.md` | 改造路线图、硬冲突、分阶段计划、工程规范 |
| `assets/*.uix`、`assets/themes.json` | 静态布局与主题 token（遵守 D-15：只声明布局 / 样式 / token，无事件属性、无业务脚本） |
| `README-FORK.md` | 本文件 |

这些 `.uix` / 主题资产不参与 `cargo build`（未加入任何 crate 的 `include`）；
但**本 fork 已重写 `.github/workflows/build.yml`**（三 job：rust / core-ui / package），
当前 CI 实况见 `docs/ci.md`，不再沿用上游既有 CI 的行为。

## 上游基线

- 建 fork 时的上游基线：`6aec0c2 Release ZSDesk 0.3.11 fixes`
- **已同步的上游 HEAD**：`cb465e862b`（v0.3.12，2 提交 / 108 文件，2026-11-15 以 `--no-ff` merge 合入）；**merge commit `aa44acc` = 本地 `main` = 远端 `origin/main`**，第一父 `9a3628b` 提供远端回滚锚点
- 合并前全绿锚点：本地 tag / 备份分支 `pre-upstream-0.3.12` = `9a3628b`（第 15 / 16 次 CI 全绿）；合并用 merge 而非 rebase，以保留 `docs/12` 的 `6ea629f` 等历史引用可解析
- 语言：Rust 为主，少量 C++ / C / PowerShell / C#
- 许可：MIT，`Copyright (c) 2026 ZSDesk contributors`；上游 `LICENSE` 与 `THIRD-PARTY-NOTICES.txt` 原样保留
- 构建：`cargo test --workspace --release --locked`，Rust 1.89+ / CI 固定 1.99.0；Windows x64

## 交付形态（已拍板）

**只做全量 x64 版本**：唯一 edition 为 `full`（features `network + desktop`，即默认 features），平台仅 `windows-x64`。不构建 / 不测试 / 不发布 `standard` / `network` / `desktop` / `themed` / `desktop-box` 单版，也不做 32 位 x86。功能集**不裁剪**（桌面栅栏、Everything、Listary、MinHook、SQLite 全保留）。

上游 `scripts/package.ps1` 本身已是 x64-only：强制校验 Visual C++ x64 Redistributable（C++ v14 运行库）、Everything engine pin `platform=windows-x64`，并生成逐文件 `bytes + SHA-256` manifest。CI 需从「多 edition 并行」收窄为「仅 full x64」，详见路线图 §3.6 / §4。

## 注意事项

- 上游 `zsdesk-ui` 与 `zsdesk-app` 依赖仓库外的 `zsui`（`../../../zsui`，MPL-2.0）；CI 通过额外 checkout `qiu7824/zsui` 满足。本地单独 clone 本仓库无法直接编译，需要按 `.github/workflows/build.yml` 的方式同时准备 ZSUI。
- fork 的 GitHub Actions 默认未启用，需在仓库 Actions 页面手动开启。
- 本 fork 目标平台仅 Windows x64，不构建 / 不发布 32 位 x86。

## 工程规范

沿用 `dev-discipline` 与代码审查机制（Gate 0~4 / R1~R12 / 三遍法 / DoD）；每次写完代码做审查留痕；创建 CI 前先复查 UI 设计代码是否正在更新；成果上传云盘需用户明确确认。详见路线图第 5 节。
