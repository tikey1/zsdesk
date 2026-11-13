# 仓库目录与 crate 分层

> 契约编号：`ZC-LAYOUT-001`
> 状态：已定（随 D08 生效）
> 仓库名：`zsdesk-coreui`（工作区名沿用团队代号 `zsdesk-coreui`）

---

## 1. 目录树

```
zsdesk-coreui/
├── Cargo.toml                       # workspace 根（沿用上游 edition 2021 / rust-version 1.89）
├── Cargo.lock                       # 提交
├── README.md
├── LICENSE                          # MIT
├── THIRD-PARTY-NOTICES.txt          # core-ui(MIT) / ZSUI(MPL-2.0)/ MinHook(BSD-2) / Everything
│
├── crates/
│   ├── zc-core/                     # L0 领域模型（源自 zsdesk-core）
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs
│   │       ├── path.rs
│   │       └── model/
│   │           ├── mod.rs
│   │           ├── explorer.rs     # ExplorerSession / ExplorerTab / ShellLocation / ExplorerUi
│   │           ├── desktop.rs      # 桌面格子模型
│   │           ├── tab.rs
│   │           ├── view.rs         # FileEntry / natural_cmp / RequestId / VisibleRange
│   │           └── preview.rs      # 预览请求模型（新增）
│   │
│   ├── zc-platform/                 # L1 平台抽象 trait（源自 zsdesk-platform）
│   │   └── src/
│   │       ├── lib.rs
│   │       └── shell.rs             # DirectoryLister / IconProvider / FileOperator /
│   │                               #   ShellItemSource / CancellationToken
│   │
│   ├── zc-windows/                  # L2 Win32/COM/Shell 实现（源自 zsdesk-windows）
│   │   ├── build.rs
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── enumerate.rs
│   │       ├── gfx.rs
│   │       ├── integration.rs
│   │       ├── taskbar.rs
│   │       ├── theme.rs
│   │       ├── desktop.rs
│   │       ├── flyout.rs
│   │       ├── flyout_layout.rs
│   │       ├── ole/
│   │       │   ├── mod.rs
│   │       │   └── drop.rs         # OLE 拖放源（拖出到资源管理器）
│   │       └── shell/
│   │           ├── mod.rs
│   │           ├── shell_item.rs   # IShellItem <-> ShellLocation
│   │           ├── icons.rs        # IShellItemImageFactory -> BGRA
│   │           ├── thumb.rs        # 缩略图（新增）
│   │           ├── properties.rs   # IShellPropertyStore（新增）
│   │           ├── file_transfer.rs # IFileOperation
│   │           ├── commands.rs
│   │           ├── events.rs
│   │           ├── notify.rs
│   │           ├── pidl.rs
│   │           └── ...             # menu.rs / explorer_browser.rs 等按迁移矩阵处置
│   │
│   ├── zc-ffi-coreui/               # L2 core-ui 动态绑定（**全仓唯一允许 unsafe**）
│   │   ├── build.rs                 # 固化 core-ui revision
│   │   └── src/
│   │       ├── lib.rs
│   │       └── lib/
│   │           ├── api.rs           # 手写 extern "C" 声明
│   │           ├── loader.rs        # LoadLibrary + 必需/可选符号表
│   │           ├── version.rs       # 版本协商 + 能力表
│   │           └── ui_thread.rs     # UI 线程收敛
│   │
│   ├── zc-ui/                       # L3 呈现层（源自 zsdesk-ui + 新增自绘控件）
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── theme.rs             # Fluent 令牌 -> core-ui 主题
│   │       ├── chrome.rs            # 窗口几何 / 标题栏 / 命令栏布局
│   │       ├── menus.rs             # 菜单定义（数据驱动，供 .uix 投影）
│   │       ├── shell_page.rs        # .uix 页面装配 + 状态投影
│   │       ├── filelist/            # ★ 自绘文件列表
│   │       │   ├── mod.rs
│   │       │   ├── model.rs         # FileListModel（Rust 侧视图模型）
│   │       │   ├── viewport.rs      # 虚拟滚动 / 行高 / 可见区间
│   │       │   ├── draw.rs          # ui_custom_on_draw 的绘制
│   │       │   ├── input.rs         # 鼠标 / 键盘 / 选择 / 框选
│   │       │   ├── rename.rs        # 内联重命名
│   │       │   ├── context_menu.rs  # 右键菜单装配
│   │       │   └── icon_cache.rs    # 图标 / 缩略图 LRU
│   │       ├── navtree/             # ★ 自绘导航树（D07 方案 A）
│   │       │   ├── mod.rs
│   │       │   ├── model.rs
│   │       │   └── draw.rs
│   │       ├── preview/             # M2 预览窗格
│   │       │   ├── mod.rs
│   │       │   ├── image.rs
│   │       │   └── text.rs
│   │       └── external.rs          # GuoheView / 系统程序外部打开（D02）
│   │
│   └── zc-app/                      # L4 应用与状态机（源自 zsdesk-app）
│       ├── build.rs
│       └── src/
│           ├── main.rs
│           ├── app.rs               # 启动参数解析 / 模式分发
│           ├── commands.rs          # UiCommand 枚举 + 归约
│           ├── runtime.rs
│           ├── instance.rs
│           ├── storage.rs           # 配置 / 视图记忆持久化
│           ├── edition.rs
│           ├── selection.rs         # 选择集 / 框选 / 反选
│           ├── sort.rs              # 排序（natural_cmp）
│           ├── fast_search/         # Everything / MFT / USN / 目录扫描
│           ├── quick_search/        # 快速搜索常驻
│           ├── window/              # 窗口宿主 / 标签 / 面板
│           ├── desktop/             # 桌面格子
│           ├── context_menu_*.rs    # 右键菜单宿主
│           ├── explorer_forwarder.rs
│           ├── routing_control.rs
│           ├── win_e_bridge.rs
│           └── shell_window_registry.rs
│
├── ui/                              # ★ .uix 资源（core-ui 页面）
│   ├── main.uix                     # 主窗口页面（标签 / 地址栏 / 工具栏 / 树 / 列表 / 状态栏）
│   ├── settings.uix                 # 设置窗口
│   ├── preview.uix                  # 预览窗格页面
│   ├── lang/
│   │   ├── zh-CN.lang
│   │   └── en.lang
│   └── assets/
│       ├── icons/*.svg              # ★ 锁定的 SVG 图标库（G08：全项目只用这一套）
│       ├── theme.css
│       └── images/*.png
│
├── packaging/
│   ├── core-ui-revision.txt         # ★ pinned revision（CI 按它 checkout）
│   ├── zsui-revision.txt            # 保留（M3 前若仍有 ZSUI 残留）
│   ├── licenses/
│   ├── windows/                     # .ico / manifest
│   └── third-party-notices.txt
│
├── native/                          # 原生扩展（M3）
│   ├── listary/                     # Listary 插件（沿用上游）
│   └── routing/                     # 路由转发 DLL（沿用上游）
│
├── scripts/
│   ├── build-coreui.ps1             # 调 core-ui 的 build-clang-cl.ps1
│   ├── package.ps1                  # 便携 ZIP 打包
│   ├── gates/                       # ★ 机械门禁脚本（见 spec/ci-contract.md）
│   │   ├── verify.py
│   │   ├── gates.py
│   │   ├── gates_engineering.py
│   │   └── ...                     # 从专家包 templates/gates 复制
│   └── check-coreui-api.py         # ★ 门禁 S8：绑定符号 vs 上游头文件
│
├── spec/                            # ★ 本 Spec 目录
│   ├── crate-layout.md
│   ├── ffi-core-ui.md
│   ├── migration-matrix.md
│   ├── ci-contract.md
│   ├── milestones.md
│   ├── adr/
│   │   ├── ADR-0001-state-in-rust.md
│   │   ├── ADR-0002-dynamic-loading.md
│   │   └── ADR-0003-custom-file-list.md
│   └── rust-ffi-skeleton/           # FFI 骨架（合并进 crates/zc-ffi-coreui）
│
├── .github/workflows/
│   ├── build.yml                    # ★ 主工作流（沿用上游规范 + core-ui 构建）
│   ├── gates.yml                    # 机械门禁（可并入 build.yml）
│   └── release.yml                  # 便携 ZIP 发布
│
└── docs/
    └── ...                          # 开发文档（不与 spec/ 混）
```

---

## 2. crate 职责与依赖方向

### 2.1 分层与依赖图

```
        L4            zc-app ──────────────────────────────┐
                      │                                    │
        L3            zc-ui ────────────┐                 │
                      │                   │                 │
        L2      zc-windows        zc-ffi-coreui ◄───────────┘
               │        │            （无内部依赖）
        L1   zc-platform │
               │        │
        L0      zc-core ◄┘
```

**依赖边（箭头 = 「依赖于」）：**

| crate | 依赖 | 理由 |
|---|---|---|
| `zc-core` | （无内部依赖） | 领域模型，零 UI 框架依赖（沿用上游设计原则） |
| `zc-platform` | `zc-core` | trait 的返回类型来自领域模型 |
| `zc-windows` | `zc-platform`, `zc-core` | 实现平台 trait |
| `zc-ffi-coreui` | （**无内部依赖**） | **FFI 层不得反向依赖 UI 业务层** |
| `zc-ui` | `zc-core`, `zc-platform`, `zc-ffi-coreui` | 呈现层消费领域模型 + 平台 trait + core-ui 绑定 |
| `zc-app` | `zc-core`, `zc-platform`, `zc-windows`, `zc-ui`, `zc-ffi-coreui` | 应用层组装一切 |

### 2.2 三条不可违反的规则

1. **无环**。上表是一张 DAG，按 L0→L4 单向。CI 门禁 S9 会用 `cargo metadata` 解析
   workspace 依赖图，**发现反向边即失败**。
2. **FFI 层不反向依赖 UI 业务层**。`zc-ffi-coreui` 的 `Cargo.toml` 里
   **不允许出现任何 `zc-*` 路径依赖**。门禁 S10 扫描该 crate 的 `Cargo.toml` 判失败。
3. **除 `zc-ffi-coreui` 外，其余 crate 一律 `#![forbid(unsafe_code)]`**。
   `zc-windows` 需要 `unsafe`（COM / Win32）—— 它是第二个例外，
   但它的 unsafe 集中在 `src/shell/` 与 `src/ole/`，并在每个文件头注明理由。

### 2.3 每个 crate 的一句话职责

| crate | 一句话 |
|---|---|
| `zc-core` | 「文件管理器有哪些状态」—— 领域模型，不认识 Windows |
| `zc-platform` | 「平台要提供哪些能力」—— trait 定义 |
| `zc-windows` | 「Windows 上这些能力怎么实现」—— COM / Win32 |
| `zc-ffi-coreui` | 「怎么安全地跟 core-ui 说话」—— 动态加载 + 线程收敛 |
| `zc-ui` | 「界面长什么样、怎么画、怎么响应」—— 自绘控件 + 页面装配 |
| `zc-app` | 「应用怎么运转」—— 状态机、命令、窗口宿主、生命周期 |

---

## 3. 与上游 zsdesk 的 crate 对应

| 上游 | 新仓 | 处置 |
|---|---|---|
| `zsdesk-core` | `zc-core` | 改名 + 加 `model/preview.rs` |
| `zsdesk-platform` | `zc-platform` | 改名 + 加 `ShellItemSource` trait |
| `zsdesk-windows` | `zc-windows` | 改名 + 按迁移矩阵增删 |
| `zsdesk-ui` | `zc-ui` | 改名 + **大幅扩写**（自绘文件列表 / 导航树 / 页面装配） |
| `zsdesk-app` | `zc-app` | 改名 + 按迁移矩阵拆分/删除呈现层代码 |
| （无） | `zc-ffi-coreui` | **新增**，全仓唯一 unsafe crate |

**改名理由**：上游 `zsdesk-*` 前缀带着「ZSDesk」这个产品名。新仓一旦有第二个形态
（core-ui 版）就会重名冲突。统一 `zc-`（zsdesk-coreui）前缀。

---

## 4. `.uix` 资源放在哪、怎么被找到

### 4.1 布局

```
ui/
├── main.uix
├── settings.uix
├── preview.uix
├── lang/{zh-CN,en}.lang
└── assets/{icons/*.svg, theme.css, images/*.png}
```

### 4.2 运行时路径解析

core-ui 的资源系统是**按名字**解析的（`ui_asset_register_dir` / `register_blob`），
不假设任何路径。所以：

| 形态 | 做法 | 何时用 |
|---|---|---|
| **开发** | `ui_asset_register_dir("<exe目录>/../../ui/assets")` | 本地跑 |
| **发布** | `ui_asset_register_blob("main.uix", bytes, len)` 逐个注册 | 便携 ZIP |

> `ui_core_embed_binary`（把二进制烤进 exe）在本机 DLL 上**不存在**（实测：仓库有、本机无），
> 所以发布形态**不走「单文件 exe」**，走**便携 ZIP + 旁置资源目录**。
> 见 `spec/ffi-core-ui.md` §5.3 的 `embed_binary` 能力降级。

### 4.3 `.uix` 的组织约束

| 约束 | 理由 |
|---|---|
| **一个 `.uix` = 一个页面 = 一个窗口** | core-ui 1.5.0 起 `<import>` 子组件系统已删除；多页面要么在 C 端组合多个 page，要么复制模板 |
| **`<template>` 恰好一个根元素** | core-ui 强制 |
| **所有标签显式闭合**（`<br/>`） | core-ui L3 严格模式 |
| **模板里不写 `this.` 前缀** | 编译器自动改写 |
| **不用 `watch` / `ref` / `reactive` / 生命周期 hook** | `.uix` 不支持（`docs/uix-guide.md` §3.5） |
| **列表数据整体替换，不原地 mutate** | 原地 mutate 不触发响应式 |
| **`<custom id="file_list"/>` 与 `<custom id="nav_tree"/>` 是 Rust 自绘的挂点** | 见 `spec/ffi-core-ui.md` §4.3 |

---

## 5. `packaging/` 内容

| 文件 | 内容 | 谁维护 |
|---|---|---|
| `core-ui-revision.txt` | pinned revision（当前：`1840d689b51a32d392dbf06b8d594aef96696a80`） | 架构师，升级 core-ui 时改 |
| `zsui-revision.txt` | 保留；M3 前若仍有 ZSUI 残留则仍需要 | 沿用上游 |
| `licenses/` | 第三方许可原文（core-ui MIT / ZSUI MPL-2.0 / MinHook BSD-2 / Everything） | 合规 |
| `third-party-notices.txt` | 汇总声明 | 合规 |
| `windows/` | exe 图标（`.ico`）、manifest（UAC asInvoker、DPI awareness） | 运维 |

**`core-ui-revision.txt` 的消费点（唯一事实来源）：**

1. `zc-ffi-coreui/build.rs` → 固化为 `ZSCOREUI_CORE_UI_REVISION`；
2. CI 的 `actions/checkout` → 精确 ref 检出 core-ui 源码；
3. CI 打包步骤 → 写 `core-ui.dll.revision` 戳记；
4. CI 门禁 S8 → 比对绑定符号与该 revision 的 `include/ui_core.h`。

---

## 6. 依赖与影响面

### 6.1 外部依赖（新增）

| 依赖 | 用途 | 理由 |
|---|---|---|
| `libloading` 0.8 | 动态加载 core-ui.dll | 唯一新增依赖；见 `spec/ffi-core-ui.md` §2 |
| `windows` 0.62 / `windows-sys` 0.59 | Win32 / COM | **沿用上游**，不换 |
| `serde` / `serde_json` | 配置 / 状态序列化 | 沿用上游 |
| `anyhow` / `tracing` / `tracing-subscriber` / `dirs` | 沿用上游 | |

**不新增**：不引入任何 GUI 框架、不引入 bindgen、不引入 webview。

### 6.2 影响面（改动会波及什么）

| 改动 | 影响 |
|---|---|
| 升级 core-ui revision | ① `zc-ffi-coreui` 绑定可能要加符号；② `.uix` 模板语法可能变；③ 门禁 S8 会先报出来 |
| 改 `zc-core` 的模型 | `zc-platform` / `zc-windows` / `zc-ui` / `zc-app` 全链受影响 → **模型改动必须先改 `zc-core`** |
| 改 `zc-platform` 的 trait | `zc-windows`（实现）+ `zc-ui`（消费）两处 |
| 改 `zc-ffi-coreui` 的绑定 | `zc-ui` + `zc-app` |
| 改 `.uix` | 只影响 `zc-ui` 的投影字段名（`ViewModel`） |

---

## 7. 命名与编码约定

沿用上游 zsdesk 的代码本体规范（实测：无 `.editorconfig` / `rustfmt.toml`，
规范体现在代码里）：

| 项 | 约定 |
|---|---|
| 模块文档 | 每个 crate 根有 `//!` 说明；每个重要模块有 `//!` 说明（上游风格） |
| 注释语言 | **中文**（上游风格） |
| 命名 | crate 用 kebab-case；类型用 PascalCase；函数/变量用 snake_case |
| 错误处理 | `anyhow::Result` 在应用层；库层用自有 `zc_core::error::Result`（上游已有） |
| 日志 | `tracing`（上游已有），**不打印路径里的用户名**（安全基线 §六） |
| unsafe | 只在 `zc-ffi-coreui` 与 `zc-windows`；其余 crate `#![forbid(unsafe_code)]` |

---

## 8. 机械门禁（与目录结构同步落地）

> 依据 `10-standards/mechanical-gates.md`：「没有执行器的规则，只是建议。」

| # | 门禁 | 检查什么 | 失败处置 | 命令 |
|---|---|---|---|---|
| G03 | 单文件行数 | 任一 `.rs` > 300 行 | 拆分 | `python3 scripts/gates/gates.py G03` |
| G04 | 分层方向 | 底层 crate 反向依赖上层 | 修正 | `python3 scripts/gates/gates.py G04` |
| **S9** | **crate 依赖无环** | `cargo metadata` 解析出的依赖图有环 | 修正 | `python3 scripts/gates/check_crate_dag.py` |
| **S10** | **FFI 层无业务依赖** | `zc-ffi-coreui/Cargo.toml` 含 `zc-*` 路径依赖 | 修正 | `python3 scripts/gates/check_crate_dag.py` |
| **S8** | **绑定符号 vs 上游头文件** | 绑定层声明的符号集合 ≠ pinned revision 头文件的 `ui_*` 集合 | 改绑定或改 revision | `python3 scripts/check-coreui-api.py` |
| G01 | emoji | 源码 / `.uix` / 文档出现 emoji | 换语义图标 | `python3 scripts/gates/gates.py G01` |
| G08 | 图标库唯一 | 同时出现多套图标库 | 统一到 `ui/assets/icons/*.svg` | 人工 + `gates.py G08` |
| E07 | 硬编码密钥 | 代码里出现字面量 token / 密钥 | 走环境变量 | `python3 scripts/gates/gates_engineering.py E07` |
| E11 | 外部调用无超时 | 调外部进程 / COM 无超时 | 加超时 | `python3 scripts/gates/gates_engineering.py E11` |

**首次基线**：Phase 3 开工前跑 `python3 scripts/gates/verify.py --phase 3`，
把结果写进 `spec/` 下的基线文件。**没有基线，后面「改坏了」无法判定。**

---

## 9. out-of-scope

1. **不引入** workspace 之外的第三个 crate 层级（不搞 DDD 目录）。
2. **不把 `.uix` 拆成多个文件互相 import** —— core-ui 1.5.0 已删 `<import>`，
   拆了也 import 不了。**一个 `.uix` 一个窗口**是硬约束。
3. **不做** workspace 多包并行发布（不搞 cargo workspace publish）。
4. **不把** `native/`（Listary 插件、路由 DLL）纳入 Rust workspace ——
   它们是独立的 .NET / C++ 工程，沿用上游的独立构建。

---

## 10. 关联文档

- `spec/ffi-core-ui.md` —— FFI 契约（状态归属、编码、所有权、线程、版本协商）
- `spec/migration-matrix.md` —— 文件级复用矩阵（对应本文件的目录树）
- `spec/ci-contract.md` —— 构建 / 打包 / 门禁
- `spec/milestones.md` —— 里程碑与验收
