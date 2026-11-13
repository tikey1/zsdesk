# spec —— 技术契约目录

> 本目录只放**契约与设计**，**不放实现**。
> core-ui 绑定层的唯一实体在 `crates/zsdesk-coreui/`（原 `spec/rust-ffi-skeleton/` 已并入，2026-11-13）。

| 文件 | 内容 |
|---|---|
| `ffi-core-ui.md` | core-ui C API 的绑定契约：编码 / 所有权 / 线程 / 状态归属 / 页面重建生命周期 |
| `crate-layout.md` | crate 分层、依赖方向、`.uix` 资产的五条硬约束、句柄表落点、自绘宿主裁定 |
| `migration-matrix.md` | zsdesk 五个 crate 的文件级四档判定（复用 / 改造 / 重写 / 删除） |
| `ci-contract.md` | GitHub Actions 与打包契约（目标态 vs 当前态两栏） |
| `milestones.md` | M0~M3 里程碑与验收项（映射用户路线图 P1~P4） |
| `adr/` | 架构决策记录 |
| `tools/` | 契约与矩阵的辅助脚本 |
