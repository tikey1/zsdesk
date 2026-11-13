# 仓库目录与 crate 分层

> 契约编号：`ZC-LAYOUT-001`（v2）
> 状态：已定（随 D11 生效）
> **v2 变更（2026-11-13）**：按 team-lead 已代写并准备推送的 CI 实现校准 ——
> ① **保留上游 crate 名**（`zsdesk-core` / `zsdesk-platform` / `zsdesk-ui` / `zsdesk-windows` / `zsdesk-app`），
> **不重命名**（理由见 §3）；② 工作区是 **`app/` + `zsui/` + `core-ui/` 三级同级**
> （对应 crate 里 `../../../xxx` 的 path 依赖）；③ `.uix` 资源在 **`assets/`**（仓库既有位置），
> 不是 `ui/`。**唯一新增的 crate 是 `zsdesk-coreui/`（core-ui 绑定层）。**

---

## 1. 目录树

### 1.1 CI 工作区（三级同级）

```
<workspace>/
├── app/            ← 本仓库（actions/checkout path: app）
├── zsui/           ← qiu7824/zsui @ packaging/zsui-revision.txt（C4 过渡期需要，M3 移除）
└── core-ui/        ← ghboke/core-ui @ packaging/core-ui-revision.txt（D03）
```

`crates/zsdesk-app/Cargo.toml` 里的 `zsui = { path = "../../../zsui" }` 正是靠这个布局成立。
**core-ui 不需要 path 依赖** —— Rust 侧走运行时动态加载（D09），**不链接 `core-ui.lib`**。

### 1.2 仓库内部（`app/`）

```
app/
├── Cargo.toml                       # workspace 根（沿用上游 edition 2021 / rust-version 1.89）
├── Cargo.lock                       # 提交
├── README.md / README-FORK.md
├── LICENSE                          # MIT
├── THIRD-PARTY-NOTICES.txt
│
├── crates/
│   ├── zsdesk-core/                 # L0 领域模型（上游同名，**不重命名**）
│   ├── zsdesk-platform/             # L1 平台抽象 trait（上游同名）
│   ├── zsdesk-windows/              # L2 Win32/COM/Shell 实现（上游同名）
│   ├── zsdesk-coreui/               # L2 ★新增★ core-ui 动态绑定层（**全仓唯一允许 unsafe**）
│   ├── zsdesk-ui/                   # L3 呈现层（上游同名，内容大幅扩写）
│   └── zsdesk-app/                  # L4 应用与状态机（上游同名）
│
├── assets/                          # ★ .uix 资源（仓库既有位置）
│   ├── main.uix                     # 主窗口页面
│   ├── preview.uix                  # 预览窗格页面
│   ├── about.uix                    # 关于对话框
│   ├── themes.json                  # 主题 token（light / dark 两套，键已改名为 --app-*）
│   └── icons/                       # ★ 锁定的 SVG 图标库（G08：全项目只用这一套）
│
├── packaging/
│   ├── core-ui-revision.txt         # ★ pinned revision（CI 按它 checkout）
│   ├── zsui-revision.txt            # C4 过渡期需要，M3 移除
│   ├── licenses/
│   ├── windows/                     # .ico / manifest
│   └── third-party-notices.txt
│
├── native/                          # 原生扩展（M3）：listary（.NET 插件）/ routing（C++ DLL）
│
├── scripts/
│   ├── package.ps1                  # 上游打包脚本（沿用）
│   ├── gates/                       # ★ 机械门禁脚本
│   └── check-coreui-api.py          # ★ 门禁 S8
│
├── spec/                            # ★ 本 Spec 目录
│   ├── crate-layout.md / ffi-core-ui.md / migration-matrix.md
│   ├── ci-contract.md / milestones.md
│   ├── adr/ADR-0001~0003
│   └── rust-ffi-skeleton/           # FFI 骨架（合并进 crates/zsdesk-coreui/）
│
├── docs/                            # 设计 / 调研 / CI 文档
├── research/facts.md                # 事实核对底稿
├── DECISIONS.md / SPEC.md / PROJECT_STATE.md
│
└── .github/workflows/build.yml      # ★ CI（team-lead 代写）
```

---

## 2. crate 职责与依赖方向

### 2.1 分层与依赖图

```
        L4            zsdesk-app ──────────────────────────────┐
                      │                                    │
        L3            zsdesk-ui ────────────┐                 │
                      │                   │                 │
        L2      zsdesk-windows        zsdesk-coreui ◄───────────┘
               │        │            （无内部依赖）
        L1   zsdesk-platform │
               │        │
        L0      zsdesk-core ◄┘
```

**依赖边（箭头 = 「依赖于」）：**

| crate | 依赖 | 理由 |
|---|---|---|
| `zsdesk-core` | （无内部依赖） | 领域模型，零 UI 框架依赖（沿用上游设计原则） |
| `zsdesk-platform` | `zsdesk-core` | trait 的返回类型来自领域模型 |
| `zsdesk-windows` | `zsdesk-platform`, `zsdesk-core` | 实现平台 trait |
| **`zsdesk-coreui`** | （**无内部依赖**） | **FFI 层不得反向依赖 UI 业务层** |
| `zsdesk-ui` | `zsdesk-core`, `zsdesk-platform`, `zsdesk-coreui` | 呈现层消费领域模型 + 平台 trait + core-ui 绑定 |
| `zsdesk-app` | `zsdesk-core`, `zsdesk-platform`, `zsdesk-windows`, `zsdesk-ui`, `zsdesk-coreui` | 应用层组装一切 |

### 2.2 三条不可违反的规则

1. **无环**。上表是一张 DAG，按 L0→L4 单向。CI 门禁 **S9** 用 `cargo metadata` 解析
   workspace 依赖图，**发现反向边即失败**。
2. **FFI 层不反向依赖 UI 业务层**。`zsdesk-coreui/Cargo.toml` 里**不允许出现**任何
   指向 `zsdesk-core` / `zsdesk-platform` / `zsdesk-ui` / `zsdesk-app` / `zsdesk-windows`
   的 path 依赖。门禁 **S10** 扫描该文件判失败。
3. **除 `zsdesk-coreui` 与 `zsdesk-windows` 外，其余 crate 一律 `#![forbid(unsafe_code)]`**。
   `zsdesk-windows` 需要 `unsafe`（COM / Win32）—— 它是第二个例外，
   但 unsafe 集中在 `src/shell/` 与 `src/ole/`，并在每个文件头注明理由。

### 2.3 每个 crate 的一句话职责

| crate | 一句话 |
|---|---|
| `zsdesk-core` | 「文件管理器有哪些状态」—— 领域模型，不认识 Windows |
| `zsdesk-platform` | 「平台要提供哪些能力」—— trait 定义 |
| `zsdesk-windows` | 「Windows 上这些能力怎么实现」—— COM / Win32 |
| **`zsdesk-coreui`** | 「怎么安全地跟 core-ui 说话」—— 动态加载 + 线程收敛 + 版本协商 |
| `zsdesk-ui` | 「界面长什么样、怎么画、怎么响应」—— 自绘控件 + 页面装配 |
| `zsdesk-app` | 「应用怎么运转」—— 状态机、命令、窗口宿主、生命周期 |

---

## 3. 与上游 zsdesk 的 crate 对应

| 上游 | 新仓 | 处置 |
|---|---|---|
| `zsdesk-core` | `zsdesk-core` | **不改名**，加 `model/preview.rs` |
| `zsdesk-platform` | `zsdesk-platform` | **不改名**，加 `ShellItemSource` trait |
| `zsdesk-windows` | `zsdesk-windows` | **不改名**，按迁移矩阵增删 |
| `zsdesk-ui` | `zsdesk-ui` | **不改名**，**大幅扩写**（自绘文件列表 / 导航树 / 页面装配） |
| `zsdesk-app` | `zsdesk-app` | **不改名**，按迁移矩阵拆分/删除呈现层代码 |
| （无） | **`zsdesk-coreui`** | **新增**，全仓唯一 unsafe crate |

### 3.1 为什么不重命名（v2 的决定）

初版方案曾提议统一改成 `zc-*`（zsdesk-coreui）前缀，理由是「一旦有第二个形态就会重名冲突」。
**采纳后推翻** —— 成本收益不划算：

| 项 | 评估 |
|---|---|
| 收益 | 命名一致性（次要美学问题） |
| 成本 | 138 个文件的 `use` 路径、`Cargo.toml` 的 path 依赖、`build.rs`、CI 命令、文档引用**全要改**；且 `zsui` / `core-ui` 是外部依赖（不能改名），出现「三个 `zsdesk-*` + 一个 `zsui` + 一个 `core-ui`」的混合命名 |
| 结论 | **保留上游名**。真正需要区分的是**新增的那一个** crate → `zsdesk-coreui`（名字已自解释） |

> 迁移矩阵（`spec/migration-matrix.md`）里的「新落点」列因此全部是
> **`crates/zsdesk-xxx/`**（保持同名），只有自绘相关的新模块是新增路径。

---

## 4. `.uix` 资源放在哪、怎么被找到

### 4.1 布局（仓库既有位置）

```
assets/
├── main.uix
├── preview.uix
├── about.uix
├── themes.json
└── icons/*.svg
```

### 4.2 运行时路径契约（★ 与打包强相关）

> **Rust 从 `<exe 同目录>\assets\` 读资源**（`<exe目录>` 由 `std::env::current_exe()` 推出）。
> 开发态同理（仓库根的 `assets/`）→ **同一套相对布局，打包时不用改代码**。

**打包必须把 `assets/` 整个目录拷进 ZIP** —— 🔴 **当前 CI 实现漏了这一步**，
详见 `spec/ci-contract.md` §2.4。

**运行时路径不变式**：`assets/` 在仓库根与在 ZIP 里是**同一个相对位置**（`<exe目录>\assets\`），
开发态与发布态**同一套代码路径** → 打包时不需要改任何代码。

### 4.3 页面加载方式（★ 不是 `ui_page_load_file`）

因为**库没有运行时替换 CSS 变量的 C API**（D10，`spec/ffi-core-ui.md` §4.3.2），
Rust 必须在加载前把 `:root { --app-*: … }` **拼进 `.uix` 源码**：

```
读 assets/main.uix（UTF-8）
  → 读 assets/themes.json（当前模式的 token）
  → 拼 ":root { --app-bg: …; … }"
  → 拼进 .uix 源码
  → ui_page_load_string(utf8_source)      ← 注意是 const char*（UTF-8），不是 const wchar_t*
```

| 约束 | 理由 |
|---|---|
| **一个 `.uix` = 一个窗口** | core-ui 1.5.0 起 `<import>` 子组件系统已删除；多页面要么在 C 端组合多个 page，要么复制模板 |
| **`<template>` 恰好一个根元素** | core-ui 强制 |
| **所有标签显式闭合**（`<br/>`） | core-ui L3 严格模式 |
| **模板里不写 `this.` 前缀** | 编译器自动改写 |
| **不用** `watch` / `ref` / `reactive` / 生命周期 hook | `.uix` 不支持（`docs/uix-guide.md` §3.5） |
| **列表数据整体替换，不原地 mutate** | 原地 mutate 不触发响应式 |
| **`.uix` 里只有布局/样式/token** | 用户 `assets/main.uix` 头注释的 **D-15「STATIC LAYOUT ONLY」** —— 无事件属性、无业务脚本；列表行由 Rust 侧 `ui_custom_on_draw` 从 Rust EntryStore 绘制；回调在 Rust 注册；首版只读 |
| **自绘控件用 `<custom>` 静态声明** | ★ **架构师裁定（2026-11-13）**：在 `.uix` 里写 `<custom id="file_list" class="list-host"/>`，Rust 用 `ui_page_on_widget_mount(page, "file_list", cb, ud)` 取句柄后挂绘制回调。**不用**运行时 `ui_custom()` + `ui_widget_add_child()`。理由：运行时造的控件**没有 CSS 级联结果** → 无 `flex-grow`/`width`/`height` → 在 flex 容器里**尺寸为 0** → 列表区空白；静态声明则由工厂明确「CSS width/height 走 ApplyCommonStyle 自然生效」（`widget_factory.cpp:1114-1118`）。详见 `spec/ffi-core-ui.md` §4.3.1 |
| **`<window>` 必须自闭合** | `sfc_parser.cpp:285-297`：非自闭合 → `pushErr` → `sfc_parser.cpp:373` `if (!doc.errors.empty()) doc.ok = false;` → **整页 load 失败**。Rust 侧必须能区分「整页解析失败」（`ui_page_load_string` 返回 NULL，取 `ui_page_last_error`）与「局部控件缺失」（加载成功但 `find_by_id` 返回 0） |
| **文字必须是元素内的文本节点** | `text="…"` **不是**文本属性（只有 `<menu>` 读它作 title 别名，`compiler.cpp:641/644`）。真实机制是 `FirstPlainText`（`widget_factory.cpp:54-61`）取**第一个直接子文本节点**。→ 写 `<label>文字</label>`，不是 `<label text="文字"/>` |
| **`:root` 只能注入到 `<style>` 块内** | ★ QA B1 复核 + 架构侧核对（`sfc_parser.cpp:260-267`）：`:root` 若**裸放在源码顶层** → `unexpected content at top level` → `doc.ok=false` → **整页 load 失败**（报错 "SFC parse failed"，极难定位）。顶层只允许 `<window>` `<template>` `<script>` `<style>` `<import>` `<link>`。→ Rust 拼 `:root` 时**追加一个完整的 `<style>` 块到源码末尾**（多个 `<style>` 块允许，`sfc_parser.cpp:321-330`）；且**加载前自检**「`:root` 是否落在某个 `<style>` 块内」，不满足即 fail-fast。完整规则见 `spec/ffi-core-ui.md` §4.3.2.1 |
| **`:root` 内只放 CSS 变量** | `cascade.cpp:124-126` 明确 `Skip :root — those only define CSS vars` → `:root { background/color/font-family }` 会被**静默丢弃**。→ `:root` 只写 `--app-*`；根节点的普通样式走 `.root` class |
| **根窗口样式用 `.root` class** | `window { … }` 是标签选择器（`selector.cpp:59` `node.tag == "window"`），而 `<window>` **不产生 widget**（`sfc_parser.cpp:285-296`）→ 匹配 0 个节点 → 整条无效；`:root { … }` 在普通样式计算里被**显式跳过**（`cascade.cpp:125`），**只定义 CSS 变量**。→ 根 `<div class="root">` + `.root { … }` |

---

## 5. `packaging/` 内容

| 文件 | 内容 | 谁维护 |
|---|---|---|
| `core-ui-revision.txt` | pinned revision（当前：`1840d689b51a32d392dbf06b8d594aef96696a80`） | 架构师，升级 core-ui 时改 |
| `zsui-revision.txt` | C4 过渡期需要；**M3 收口时删除**（连同 CI 的 checkout） | 沿用上游 |
| `licenses/` | 第三方许可原文（core-ui MIT / ZSUI MPL-2.0 / MinHook BSD-2 / Everything） | 合规 |
| `third-party-notices.txt` | 汇总声明 | 合规 |
| `windows/` | exe 图标（`.ico`）、manifest（UAC asInvoker、DPI awareness） | 运维 |

**`core-ui-revision.txt` 的消费点（唯一事实来源）：**

1. CI 的 `actions/checkout` → 精确 ref 检出 core-ui 源码；
2. CI 打包 → 写 `core-ui.dll.revision` 戳记（🔴 **当前实现未做**，见 `spec/ci-contract.md` §3.1）；
3. `zsdesk-coreui/build.rs` → 固化为 `ZSCOREUI_CORE_UI_REVISION`；
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
| 升级 core-ui revision | ① `zsdesk-coreui` 绑定可能要加符号；② `.uix` 模板语法可能变；③ 门禁 S7/S8 会先报出来 |
| 改 `zsdesk-core` 的模型 | `zsdesk-platform` / `zsdesk-windows` / `zsdesk-ui` / `zsdesk-app` 全链受影响 → **模型改动必须先改 `zsdesk-core`** |
| 改 `zsdesk-platform` 的 trait | `zsdesk-windows`（实现）+ `zsdesk-ui`（消费）两处 |
| 改 `zsdesk-coreui` 的绑定 | `zsdesk-ui` + `zsdesk-app` |
| 改 `assets/*.uix` | 只影响 `zsdesk-ui` 的投影字段名（`ViewModel`） |

---

## 7. 命名与编码约定

沿用上游 zsdesk 的代码本体规范（实测：无 `.editorconfig` / `rustfmt.toml`，
规范体现在代码里）：

| 项 | 约定 |
|---|---|
| 模块文档 | 每个 crate 根有 `//!` 说明；每个重要模块有 `//!` 说明（上游风格） |
| 注释语言 | **中文**（上游风格） |
| 命名 | crate 用 kebab-case；类型用 PascalCase；函数/变量用 snake_case |
| 错误处理 | `anyhow::Result` 在应用层；库层用自有 `zsdesk_core::error::Result`（上游已有） |
| 日志 | `tracing`（上游已有），**不打印路径里的用户名**（安全基线 §六） |
| unsafe | 只在 `zsdesk-coreui` 与 `zsdesk-windows`；其余 crate `#![forbid(unsafe_code)]` |

---

## 8. 机械门禁（与目录结构同步落地）

> 依据 `10-standards/mechanical-gates.md`：「没有执行器的规则，只是建议。」
> 每条门禁的**启用里程碑**见 `spec/milestones.md`。

| # | 门禁 | 检查什么 | 失败处置 | 命令 |
|---|---|---|---|---|
| G03 | 单文件行数 | 任一 `.rs` > 300 行 | 拆分 | `python3 scripts/gates/gates.py G03` |
| G04 | 分层方向 | 底层 crate 反向依赖上层 | 修正 | `python3 scripts/gates/gates.py G04` |
| G01 | emoji | 源码 / `.uix` / 文档出现 emoji | 换语义图标 | `python3 scripts/gates/gates.py G01` |
| G08 | 图标库唯一 | 同时出现多套图标库 | 统一到 `assets/icons/*.svg` | `python3 scripts/gates/gates.py G08` |
| E07 | 硬编码密钥 | 代码里出现字面量 token / 密钥 | 走环境变量 | `python3 scripts/gates/gates_engineering.py E07` |
| E11 | 外部调用无超时 | 调外部进程 / COM 无超时 | 加超时 | `python3 scripts/gates/gates_engineering.py E11` |
| **S7** | **core-ui 导出面** | CI 编出的 DLL 的 `ui_*` 集合 vs `docs/05-COREUI-EXPORTS.txt`；差异 ⊆ {`ui_core_embed_binary`,`ui_dialog`} | 修正构建 | `scripts/check-coreui-dll.ps1` |
| **S8** | **绑定面 vs 导出表** | 绑定层声明的符号 ⊆ S7 实测集合；必需∩可选=∅；必需不含 12 个「本机独有」符号 | 改绑定 | `python3 scripts/check-coreui-api.py` |
| **S9** | **crate 依赖无环** | `cargo metadata` 解析出的依赖图有环 | 修正 | `python3 scripts/gates/check_crate_dag.py` |
| **S10** | **FFI 层无业务依赖** | `zsdesk-coreui/Cargo.toml` 含指向业务 crate 的 path 依赖 | 修正 | 同上 |
| **S13** | **token 不撞名** | `assets/*.uix` / `themes.json` 不得用 17 个旧 token 名，不得自定义 `--app-accent` | 改资源 | 见 `docs/12-UIX-TAG-FIX.md` §4.3 |

**首次基线**：Phase 3 开工前跑 `python3 scripts/gates/verify.py --phase 3`，
把结果落盘。**没有基线，后面「改坏了」无法判定。**

---

## 9. out-of-scope

1. **不引入** workspace 之外的第三个 crate 层级（不搞 DDD 目录）。
2. **不把 `.uix` 拆成多个文件互相 import** —— core-ui 1.5.0 已删 `<import>`，
   拆了也 import 不了。**一个 `.uix` 一个窗口**是硬约束。
3. **不做** workspace 多包并行发布。
4. **不把** `native/`（Listary 插件、路由 DLL）纳入 Rust workspace ——
   它们是独立的 .NET / C++ 工程，沿用上游的独立构建。
5. **不重命名** 上游 crate（见 §3.1）。

---

## 10. 关联文档

- `spec/ffi-core-ui.md` —— FFI 契约（状态归属、编码、所有权、线程、版本协商、页面加载）
- `spec/migration-matrix.md` —— 文件级复用矩阵
- `spec/ci-contract.md` —— CI 评审与缺口
- `spec/milestones.md` —— M0~M3 与验收标准
