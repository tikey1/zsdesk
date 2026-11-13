# SPEC —— ZSDesk × core-ui 技术规格

> 项目：`tikey1/zsdesk` → core-ui 化改造（工作区代号 `zsdesk-coreui`）
> 契约编号：`ZC-SPEC-001`
> 版本：v1（2026-11-13）
> 决策优先级（用户明确）：**本次对话中的选择 > 用户 fork 路线图（`docs/10-FORK-ROADMAP-COREUI.md`）> 本 Spec 的建议**。冲突处以对话为准并标注「覆盖路线图 §x」。

---

## 0. 这份文档是什么

**是**：一页能读完的技术总览 + 索引。细节在 `spec/` 下的五份契约里。

**不是**：PRD、视觉规范、UX 流程、逐行实现说明。那些分别是 PM / 设计师 / 实现侧的事。

**读者路径**：

| 你要做的事 | 读哪份 |
|---|---|
| 知道整体怎么搭的 | **本文** |
| 改 crate 划分 / 加模块 | `spec/crate-layout.md` |
| 接 core-ui 的 C API / 改 `.uix` 与 Rust 的通道 | `spec/ffi-core-ui.md` |
| 知道上游某个文件是复用/改造/重写/删 | `spec/migration-matrix.md` |
| 写 CI / 打包脚本 | `spec/ci-contract.md` |
| 排期 / 验收 | `spec/milestones.md` |
| 查 ADR | `spec/adr/` |

---

## 1. 一句话架构

> **Rust 持有全部状态与业务逻辑；core-ui 只负责画 chrome 与「Rust 自绘的列表行」；
> `.uix` 是被投影的静态布局，不是状态容器；core-ui 以 `LoadLibrary` 动态加载，
> 缺失即失败或降级，绝不静默。**

### 1.1 分层图

```
┌──────────────────────────────────────────────────────────────────────┐
│  zsdesk-app (L4)   状态机 · 命令 · 窗口宿主 · 搜索 · 桌面格子 · 转发        │
│                ExplorerSession / 选择集 / 排序 / 视图记忆  ← 唯一真相源   │
└───────┬──────────────────────────────────────────────────────────────┘
        │ 状态投影（小载荷）
        ▼
┌──────────────────────────────────────────────────────────────────────┐
│  zsdesk-ui (L3)    .uix 页面装配 · 自绘文件列表 · 自绘导航树 · 预览窗格      │
│                ▲ draw 回调读 Rust 快照    │ 事件回调 → Rust 命令       │
└───────┬──────────────────────────────────────────────────────────────┘
        │ core-ui C API（unsafe 集中于此）
        ▼
┌──────────────────────────────────────────────────────────────────────┐
│  zsdesk-coreui (L2)   LoadLibrary · 必需/可选符号表 · 版本协商       │
│                       UI 线程收敛（invoke_sync）                      │
└───────┬──────────────────────────────────────────────────────────────┘
        │ LoadLibrary("core-ui.dll")
        ▼
┌──────────────────────────────────────────────────────────────────────┐
│  core-ui.dll (CI 从 pinned revision 构建)                            │
│  ① .uix 静态布局（标签/样式/token）                                  │
│  ② <custom id="list_host"> → ui_custom_on_draw → Rust 画行         │
└──────────────────────────────────────────────────────────────────────┘

旁路：
  zsdesk-core (L0)  ←  zsdesk-platform (L1)  ←  zsdesk-windows (L2)   （Shell / COM / 搜索）
```

### 1.2 依赖方向（单向，无环）

```
zsdesk-app ──► zsdesk-ui ──► zsdesk-coreui
   │         │
   │         └──► zsdesk-platform ──► zsdesk-core
   └────────────► zsdesk-windows ────┘
```

- `zsdesk-coreui` **零内部依赖** → FFI 层不可能反向依赖 UI 业务层。
- 除 `zsdesk-coreui` 与 `zsdesk-windows` 外，其余 crate 一律 `#![forbid(unsafe_code)]`。
- 门禁 **S9/S10** 用 `cargo metadata` 机器判定这两条。

---

## 2. 决策索引

### 2.1 已定（D00–D20）

| # | 决策 | 状态 | 一句话 | 代价 |
|---|---|---|---|---|
| D00-1~7 | 用户前提：基准 zsdesk / UI 换 core-ui / GuoheView 强关联 / Win10-Fluent 观感 / Rust 为主 + v14 运行库 / 沿用上游规范 / 只做 x64 | 已定 | — | — |
| **D01** | 功能范围 = **全量移植 zsdesk** | 已定 | 不裁功能，只收窄交付形态 | 42,150 行 Rust 要按四档逐文件处置 |
| **D02** | GuoheView = **只外部调 exe** | 已定 | `ShellExecute` 拉起 `GuoheView.exe` | 进程内预览归零；**覆盖路线图 §4 P3** |
| **D03** | core-ui = **CI 从源码构建** | 已定 | `packaging/core-ui-revision.txt` 钉 revision | CI 需 VS2022 + LLVM + CMake + Ninja；`rc.exe` 挂起须换 `llvm-rc` |
| **D04** | 文件列表 = **core-ui 全自绘** | 已定 | `<custom>` + Rust `ui_custom_on_draw` | 自研虚拟列表全套交互；导航树连带不能嵌 |
| **D05** | 交付形态 = **唯一 `full` edition + `windows-x64`** | 已定 | 删掉多 edition 矩阵 | 上游 CI 两步「多 edition」步骤要删 |
| **D06** | 预览 = **分三层** | 已定 | 图片**留程序内**（WIC 解码 → BGRA 预乘 → **Rust 在 `<custom id="preview_image">` 里 blit**，D06-1 机制细化）/ 文本留 core-ui 内 / Office·WPS 降级为「用系统程序打开」/ GuoheView **只做外部 exe** | 不做 ghde 解码（闭源无 ABI）；HEIC/AVIF/PSD/RAW/JXL 进程内不做；**缩放/平移要自己实现**（首版只 fit） |
| **D07** | 导航树 = **`<custom>` 自绘 + Rust `IShellItem` 枚举** | 已定 | 兜底 = 收藏 + 面包屑 | 丢原生树的「快速访问」自动排序 |
| **D08** | FFI = **手写 extern + 运行时符号表** | 已定 | 必需缺失即失败；可选缺失即降级；**12 个「头文件漏声明」的符号已可正常绑定**（签名取自 `src/ui/ui_api.cpp`） | 无编译期签名检查 → 用门禁 S8 补偿；签名有两个来源 |
| **D09** | 链接 = **运行时 `LoadLibrary`**（不链 `core-ui.lib`） | 已定 | 显式限定搜索路径 | 失去编译期类型检查 |
| **D10** | 主题 = **库接管**，Rust 调 `ui_theme_set_mode` | 已定 | 16 个 token 改名 `--app-*`；accent 走库 token + `ui_theme_set_accent_hex` | 无运行时 CSS 变量 API → 页面用 `ui_page_load_string` 注入 `:root`（**必须在 `<style>` 块内**，否则整页失败；且必须是源码里**第一个** `:root` —— core-ui 自定义属性**先定义者赢**）；切明暗须**重建页面** |
| **D11** | 目录结构 + 迁移矩阵 + 里程碑落地 | 已定 | **保留上游 5 个 crate 名 + 新增 `zsdesk-coreui`**；138 文件四档；M0~M3 | 约 5,900–10,600 行新增 [估算] |
| **D12** | `.uix` 资产契约 + 自绘挂载选型 | 已定 | 选 **(b) `.uix` 里声明 `<custom>`**（静态声明，id = `list_host`）；`<window>` 必须自闭合；文字用元素内文本节点；根样式用 `.root` class；**布局分隔线用 1px `<div class="sep"/>`**；**宿主标签按「是否 Rust 自绘」判定**（`nav_host`→`<custom>`、`breadcrumb`→`<label>`、`preview_host`→`<div>`+`<custom id="preview_image">` 子宿主 + 4 个 `<label class="preview-meta">`）；**页面重建后 32 个句柄 + 挂载钩子必须全部重取/重注册** | `.uix` 资产需多一次机械评审；根 `<div>` 要显式加 `class="root"`；切主题必须重建页面 + 重取句柄（漏一个 = 写野内存） |
| **D13** | 无障碍与窗口材质 → **接受降级并记录** | 已定 | core-ui 无 UIA / 无系统色感知 / 无 Mica 入口 → **高对比度、读屏、Mica/Acrylic 全部不可实现** | 三项能力缺失，**必须**写进 `DELIVERY.md` 与 `README-FORK.md`；上游补 UIA 时复评 |
| **D14** | 品牌色 → **Rust 侧读系统 `AccentColor`** | 已定 | 读注册表 `HKCU\...\Explorer\AccentColor`（**BGR 序**）→ `#RRGGBB` → `ui_theme_set_accent_hex()` → 库 token `--accent`；读不到回退 `themes.json` 的 `--app-accent` | Rust 侧多一处注册表读取；BGR 写反会变橙色系；切模式后必须重读重调 |
| **D15** | `.uix` 通道与深浅色的**五条契约口径**（F1~F5） | 已定 | F1 `:root` 注入 / F2 切主题必须重建页面 / F3 重建后 32 个句柄全失效 / F4 字体不触发重建 / F5 accent 跨模式保留 → 切主题要**重读重调** | ⚠️ **F1 的「注入位置」已被 D20 取代**（first-wins 下放块尾仍会被页面自己的 `:root` 抢先）；F1 其余部分有效 |
| **D16** | 戳记口径 + S13 门禁口径 | 已定 | 戳记 = **单行 SHA**（M0 由 CI 代写但必须机器断言，M1 改 Rust `build.rs` 产出）；S13 = 两条可机械判死的规则 | M0→M1 要换产出方 |
| **D17** | CI 日志截断的根因与修法 | 已定 | `shell: pwsh` 注入 `$ErrorActionPreference='stop'` × PS7.4+ 的 `$PSNativeCommandUseErrorActionPreference=$true` → clang-cl 写一行 warning 就被判失败 → **真实诊断从未落盘**；修法：core-ui job 第一行 `Continue` + `$false`；annotation 正则 `-match` → `-cmatch` | 不修则 CI 报「编译错误」而实际**看不到任何错误** |
| **D18** | **门禁必须在 CI 里真正执行** | 已定 | 资产类五道门进 `rust` job（`::error::` 暴露是哪一条）+ 第六道 `ui_id_manifest_check.py`；**解析 id 必须先剥 HTML 注释** | 门禁不在 CI 里跑 = 只是建议 |
| **D19** | **S7' 的判定基准** | 已定 | 基准 = **同一 checkout 里 `src/` 的 `UI_API` 定义集**（不用 `docs/05` 的本机 346 基线、不用漏声明 12 个符号的头文件；`ui_dialog` 容忍并报告） | 跨版本比对只会得到「版本漂移」 |
| **D20** | **`:root` 注入形态 = 甲** | 已定 | 在**源码最前面**插入一个完整的 `<style>` 块（因为自定义属性**先定义者赢**） | 形态乙合法但脆弱；裸放顶层 = 整页失败 |

> 每条的完整记录（背景 / 备选 / 后果）见 `DECISIONS.md` 与 `spec/adr/`。

### 2.2 冲突判定（C1~C5）

| 冲突项 | 判定 | 依据 |
|---|---|---|
| C1 .NET 10 | **不引入** | 对话原话「Rust 为主」+ 路线图一致 |
| C2 Core UI ABI 闸门 | **部分推进**：符号存在性已离线核实（2006 导出 / 567 `ui_*` / 18 VERIFIED）；**U01~U12 语义仍需真机** | `docs/05-COREUI-ABI-NOTES.md` |
| C3 功能保留范围 | **全量保留** | = D01 |
| C4 ZSUI 过渡 | **过渡期保留双通道**（CI 继续 checkout `qiu7824/zsui`），M3 收口移除 | D02 + D04 的直接后果 |
| C5 fork 可见性 / CI | fork **public**；**Actions 已启用并跑绿**（commit `4a48349`，`Windows build #1`，9m58s） | 实测 |

---

## 3. 契约索引

| 契约 | 文件 | 编号 | 覆盖的决策 |
|---|---|---|---|
| 目录与分层 | `spec/crate-layout.md` | ZC-LAYOUT-001 | D11 / **D14**（§4.4 启动初始化落点） |
| FFI 与呈现层 | `spec/ffi-core-ui.md` | ZC-FFI-001 | D05 / D06 / D07 / D08 / D09 / D10 / **D13** / **D14** |
| 迁移矩阵 | `spec/migration-matrix.md` | ZC-MIG-001 | D01 / D11 |
| CI 与打包（**目标态 vs 当前态 两栏**） | `spec/ci-contract.md` | ZC-CI-001 v3 | D03 / D05 / D09 / D11 / **D13**（§4.6c 交付条款） |
| 里程碑 | `spec/milestones.md` | ZC-MILESTONE-001 | D11 / **D13**（§8 已知限制清单） / **D14**（V1.11） |
| 绑定骨架（代码） | `spec/rust-ffi-skeleton/` | — | D08 / D09 |

### 3.1 十个技术问题的结论（速查）

| # | 问题 | 结论 |
|---|---|---|
| T1 | 状态归属 | **留 Rust**。`.uix` 只做静态布局 + 视图局部态；文件列表**不在 `.uix` 里**，是 `<custom>` 由 Rust 画。判据：42,150 行 Rust 里 `zsdesk-core`(1,999 行) 本就不依赖 UI 框架，搬到 JS = 重写整个领域层 |
| T2 | 预览 | **分三层**：图片留 core-ui 内（`ui_image_view` + WIC → `ui_image_set_pixels`，**符号已核实存在**）/ 文本留 core-ui 内 / Office·WPS 外部打开 / GuoheView 只做外部 exe |
| T3 | 导航树 | **`<custom>` 自绘** + Rust `IShellItem` 枚举；兜底 = 收藏 + 面包屑 |
| T4 | 自绘文件列表 | **`<custom id="list_host" class="list-host"/>`（静态声明，D12-1；id 以资产为准）** + Rust 虚拟化；图标走 `IShellItemImageFactory` → BGRA；拖出走 OLE `DoDragDrop`；右键菜单走 `.uix` 声明式 `<menu>` |
| T5 | FFI 绑定 | **手写 extern + 运行时符号表**；UTF-8/UTF-16 **逐函数不同**；内存三分类；调用收敛 UI 线程；版本按**符号存在性**协商 |
| T6 | CRT 与链接 | **运行时 `LoadLibrary`**；三方统一 `/MD`；**谁分配谁释放**；S7 用 `dumpbin /dependents` 机器校验 |
| T7 | CI 契约 | 3 个 job（`rust` / `core-ui` / `package`）**互不依赖**；门禁 S1~S13 |
| T8 | 迁移矩阵 | 138 文件 → 直接复用 68 / 改造 56 / 重写 10 / 删除 4 |
| T9 | 目录结构 | 保留上游 5 crate 名 + 新增 `zsdesk-coreui`；CI 工作区 `app/`+`zsui/`+`core-ui/` 三级同级；`.uix` 在 `assets/`；单向依赖 |
| T-新增 | `.uix` 资产契约 | `<window>` 自闭合（否则**整页失败**）；文字写元素内文本节点（`text=` 无效）；根样式用 `.root` class（`window{}` 匹配 0 个节点，`:root{}` 只定义变量）；**`:root` 注入必须包在 `<style>` 块内**（裸放顶层 → `unexpected content at top level` → 整页失败），且只放 CSS 变量 |
| T10 | 里程碑 | M0↔P1 / M1↔P2 / M2↔P3 / M3↔P4 |

---

## 4. 里程碑索引

| 里程碑 | ↔ 路线图 | 主题 | 关键验收 |
|---|---|---|---|
| **M0** | P1 | 仓库骨架 + CI 绿灯 + 门禁 + 绑定骨架 | **Actions 跑出第一次 full x64 绿灯**；`cargo test --workspace --release --locked` 绿且**不碰 core-ui.dll**；S7~S10 全绿；**打包含 `assets/`**（🔴 当前缺口） |
| **M1** | P2 | 窗口与文件浏览 | 10,000 文件目录首屏 ≤ 120 ms；滚动 P95 ≤ 8 ms；右键菜单时序实测；OLE 拖出探针；深浅色切换不突变 |
| **M2** | P3 | **GuoheView 外部调用 + 降级通道 + 错误提示**（**覆盖路线图 §4 P3**） | GuoheView 能打开文件；未安装/未注册关联各有可区分提示与替代动作；调用参数数组化；不阻塞 UI 线程 |
| **M3** | P4 | 其余功能迁移 + 完整回归 + 发布 | 搜索/桌面格子/右键菜单宿主/网络版/Listary/腾讯迁移全通；`zsui` 依赖移除；便携包 + SHA-256 manifest |

详见 `spec/milestones.md`。

---

## 5. 术语表（人话 + 决定 + 代价）

> 术语不是标签，是要落地的决策。写不出「人话」= 还没理解它对用户意味着什么；
> 写不出「代价」= 这个决策没被评估过。

```
[术语] core-ui
[人话] 一个 Windows 原生 UI 框架（DLL），自带 Fluent 2 观感，能自绘控件，也能加载 .uix 页面
[决定] 运行时 LoadLibrary 加载 CI 自建的 core-ui.dll；只绑必需的 API 子集
[代价] 中：不能改它的源码，只能用它给的 API；升级要跟着它走

[术语] .uix
[人话] core-ui 的页面文件，像 Vue 单文件组件，但里面没有 DOM、没有网络、没有 npm
[决定] 只声明布局/样式/token（用户的 D-15「STATIC LAYOUT ONLY」）；不写事件、不写业务脚本
[代价] 低：约束换来「界面骨架与业务逻辑彻底分离」，Rust 侧完全掌控行为

[术语] 自绘（custom）
[人话] 给一块空白画布，程序自己往里画什么、画多少，全自己说了算
[决定] 文件列表与导航树都用 <custom>，在 Rust 的绘制回调里画
[代价] 高：虚拟滚动、命中测试、多选、框选、重命名全要自己做；但换来性能与外观的自由

[术语] EntryStore
[人话] Rust 侧那份「当前目录有哪些文件」的清单，是列表的唯一数据来源
[决定] 列表数据不进 .uix、不进 QuickJS；绘制回调直接读 Rust 快照
[代价] 低：避免了每次更新都序列化几 MB JSON 的开销

[术语] 动态加载（LoadLibrary）
[人话] 程序启动时再去找那个 DLL，找不到就明确报错，而不是编译时就绑死
[决定] 显式限定搜索路径：exe 同目录优先，GuoheView 目录只做兜底且要打警告
[代价] 中：失去编译期类型检查，改用 CI 门禁 S8 逐个符号核对

[术语] 必需符号 / 可选符号
[人话] 必需 = 没有它程序没法跑；可选 = 没有它某个功能降级但程序还活着
[决定] 必需缺失 → 启动即失败并列出候选路径；可选缺失 → 走降级分支并记进诊断报告
[代价] 低：让「静默白屏」变成「明确的错误」

[术语] 动态 CRT（/MD）
[人话] 程序运行时去系统里找 C++ 运行库，而不是把运行库塞进 exe
[决定] Rust MSVC、clang-cl core-ui、用户已装的 v14 运行库三方统一 /MD
[代价] 低：目标机必须装 VC++ x64 运行库（打包脚本强制校验）

[术语] 谁分配谁释放
[人话] 从别人那里拿到的内存，要用人家给的那个释放函数还回去
[决定] core-ui 返回的指针一律用 ui_debug_free / ui_page_free / ui_image_free_pixels 释放
[代价] 低：一条铁律，换来不踩跨模块堆的坑

[术语] 覆盖路线图 §x
[人话] 用户 fork 路线图里那条被本次对话的决定顶掉了
[决定] 冲突时以对话为准，并在下游文档标注被覆盖的位置
[代价] 低：避免两份文档各说一套

[术语] 品牌色（accent）
[人话] 程序里那个「主色」—— 按钮、选中项、焦点环都是它
[决定] 跟随 Windows 的系统强调色：Rust 侧读注册表 AccentColor → 调 ui_theme_set_accent_hex 写进库 token --accent；读不到就退回 themes.json 里的 --app-accent（D14）
[代价] 低：用户换系统主题色，程序跟着变；代价是 Rust 多一处注册表读取，且通道顺序写反会变橙色

[术语] 已知限制
[人话] 换框架后确实做不到、而且用户能感知到的那几件事
[决定] 高对比度、读屏、Mica 材质三项列为已知限制（core-ui 无入口），逐条写进 DELIVERY.md 与 README-FORK.md（D13）
[代价] 中：少三样能力，但写清楚就不会变成「交付后才发现」
```

---

## 6. 关键数字

| 指标 | 值 | 来源 |
|---|---|---|
| 上游 zsdesk Rust 代码量 | **138 文件 / 42,150 行** | 实测 `find crates -name '*.rs' \| xargs cat \| wc -l` |
| 直接复用 | 68 文件 / 15,671 行（37.2%） | `spec/tools/matrix_calc.py` |
| 改造 | 56 文件 / 20,193 行（47.9%） | 同上 |
| 重写 | 10 文件 / 5,600 行（13.3%） | 同上 |
| 删除 | 4 文件 / 686 行（1.6%） | 同上 |
| 本仓新增代码 | **5,900–10,600 行** [估算] | `spec/migration-matrix.md` §6 |
| core-ui.dll 导出总数 | **2006**（其中 `ui_*` **567**） | `docs/05-COREUI-EXPORTS.txt` |
| 头文件声明 `ui_*` | 557（与 DLL 交集 555） | 同上 |
| DLL 独有 / 头文件独有 | 12 / 2 | `docs/05-COREUI-ABI-NOTES.md` V04/V05 |
| core-ui 版本 | `1.9.0.345`（pinned revision `1840d689…`） | `VERSION` / `version.json` |
| core-ui.dll 体积 | 4,782,536 字节（本机） | 实测 |
| CI 单次运行时长 | 9m58s（`Windows build #1`） | 实测 |
| 本机工具链 | **无** cargo / rustc / cmake / MSVC | 实测 |

---

## 7. 风险与早期探测

> 「最可能翻车的 3 个点」—— 每条都配**具体的早期探测手段**，
> 不等它真的翻车才发现。

| # | 风险 | 为什么会翻车 | 早期探测手段 | 探测时机 |
|---|---|---|---|---|
| **1** | **自绘文件列表的交互细节** | 虚拟滚动 + 命中测试 + 框选 + 反向选择 + 就地重命名，全是新代码；上游 `IExplorerBrowser` 给的白拿 | **M1 第一周只做「滚动 + 画行 + 单击选中」**，量出每项成本再排后续；draw 回调里断言「画出的行数 ≤ 可见行数 + 4」 | M1 第 1 周 |
| **2** | **声明式右键菜单的时序** | core-ui build 75+ 删了 imperative 菜单构造；菜单只能 `.uix` 声明 + `ui_page_set_json` 推数据，而「推数据」与「弹出」之间的先后没有文档承诺 | 做一个最小探针：`set_json` → 立即 `ui_menu_show` → 用 `ui_debug_screenshot_menu` 截图，看菜单项是否已是最新值 | M1 第 2 周 |
| **3** | **OLE 拖出** | core-ui 在 `ui_window.cpp` 注释里提到宿主可起 `DoDragDrop`、`WM_CAPTURECHANGED` 会复位 pressed 状态 —— 但这是**注释，不是承诺**，没有文档、没有测试 | 最小探针：在一个 `<custom>` 里按住鼠标拖到窗口外，看 `DoDragDrop` 是否返回 `DRAGDROP_S_DROP`；同时观察 core-ui 是否崩溃或卡住 | M1 第 2 周 |

**次级风险**：

| # | 风险 | 早期探测 |
|---|---|---|
| 4 | 12 个 U 项语义未核实（U01~U12） | M0 按 ABI-NOTES 建议顺序做最小闭环：`ui_core_version*` → `ui_init_with_theme` → `ui_page_load_string` → `ui_page_open_window` → `ui_run` |
| 5 | 深浅色切换突变（token 撞名） | M1 的 V1.9：切一次主题，截图对比前后两帧 |
| 6 | `GuoheView.exe` 的命令行参数形式未核实 | M2 第一步：真机 `ShellExecuteW` 传路径，观察是否正确打开；不认则走「exe + 参数」兜底并记录实际形式 |
| 7 | `zsdesk-app/src/desktop/host.rs`（2,681 行）是最大单文件 | M3 才碰；M0~M2 不动它 |
| 9 | **`:root` 注入被 `.uix` 自带的 `:root` 抢先 → 静默失效** | core-ui 的自定义属性是**先定义者赢**（`LookupVar` 返回**第一个**命中；`:root` 规则被整条跳过 → 根本不参与 CSS 级联）→ 注入若排在 `.uix` 的 `:root` 之后 → **`.uix` 的值赢** → Rust 的注入被**静默忽略** → 症状「切主题颜色不变」，**零报错** | 注入块插在**源码最前面**；Rust 自检加**顺序断言**（+ 位置/内容/键集共四条）→ M1 的 V1.12 覆盖「`.uix` 自带 `:root` 抢先」这个坏情形 |
| 8 | **切主题后 32 个句柄失效 → 写野内存** | `ui_page_load_string` 返回新 page，旧控件被销毁；旧句柄继续用 = **静默崩溃 / 花屏，不会报「句柄无效」**。`ui_page_reload` 也重编译（重放旧源码），**同样失效** | M1 的 V1.13：切一次主题 → 状态栏 / 列表 / 预览 / 面包屑**四处都动**；代码里把句柄表收敛到**一个**结构（`page_ctx.rs`），重建走**唯一一条路径**（`spec/ffi-core-ui.md` §4.6） |

---

## 8. out-of-scope（本次不做）

1. **不做** 32 位 x86（D00-7 / D05）。
2. **不做** `standard` / `network` / `desktop` / `themed` / `desktop-box` 单版（D05）。
3. **不引入** .NET 10（C1）。
4. **不链接** `ghde.dll`、**不随包分发** GuoheView 的任何二进制（D02 / C4）。
5. **不做** 进程内图片解码、Office/WPS/PDF 渲染（D06）。
6. **不实现** core-ui 的 HTML5 语义拖放（`ui_widget_set_draggable`）—— 那不是 OLE，拖不出程序。
7. **不实现** core-ui 的 `<gh-img-view>` 瓦块金字塔喂数、OCR（`ui_ocr_img_view_*`）。
8. **不做** 单文件 exe 分发（`ui_core_embed_binary` 不是 C API）→ 走便携 ZIP + 资源目录。
9. **不承诺** 本机 GuoheView 目录里的 `core-ui.dll` 与 CI 构建的版本一致 —— 那是**兜底**，不是分发渠道。
10. **不做** Mica / Acrylic 窗口材质（D13）—— core-ui 无此入口；用户 `docs/07` §4 原文把 Mica 写成「**可选**」且「**不依赖** Acrylic / 模糊」→ 丢掉它**不违背设计意图**。
11. **不支持** 系统高对比度模式与读屏 / Narrator（D13）—— core-ui 无 UIA 桥接 → 属**已知限制**，必须写进 `DELIVERY.md` 与 `README-FORK.md`（清单见 `spec/milestones.md` §8）。
12. **不在 core-ui 侧读系统强调色**（D14）—— core-ui 全仓零命中，**「跟随系统 `AccentColor`」由 Rust 侧读注册表实现**。

---

## 9. 端到端验证（怎么证明这套东西真的能用）

```powershell
# ① Rust 侧（不需要 core-ui）
cargo test --workspace --release --locked
python3 scripts/gates/verify.py --phase 3
python3 scripts/gates/gates_engineering.py --min-sev P0
python3 scripts/gates/check_crate_dag.py
python3 scripts/check-coreui-api.py --exports docs/05-COREUI-EXPORTS.txt

# ② core-ui 侧
pwsh scripts/check-coreui-dll.ps1 -DllPath core-ui/clang-cl-build/core-ui.dll `
    -ExpectedRevision (Get-Content packaging/core-ui-revision.txt -Raw).Trim() `
    -ExpectedExports docs/05-COREUI-EXPORTS.txt
#   → x64 / 动态 CRT / 导出面差异只落在已知清单 / 版本 1.9.0.345

# ③ 产物侧
pwsh scripts/check-coreui-dll.ps1 -DllPath stage/core-ui.dll -RequireBesideExe stage/ZSDesk.exe

# ④ 关键错误流（必须判失败，不是静默降级）
#   移走 core-ui.dll → 启动 → 非零码退出 + 打印完整候选路径列表
#   未装 GuoheView   → 明确的降级提示 + 两个替代动作
```

---

## 10. 知识库落地记录

> 读了 5 篇（按 `references/ROUTER.md` 的「1 定方案」组 +「4 验收」组）：
> `10-standards/spec-as-contract.md`、`10-standards/runtime-defensive.md`、
> `10-standards/mechanical-gates.md`、`40-architecture/mvp-stack.md`、`20-quality/security-baseline.md`。

| 引用 | 改了 Spec 里的哪个值 |
|---|---|
| `spec-as-contract` §2「点名文件/模块」 | `spec/ffi-core-ui.md` §6 的绑定清单从「按需再定」改成**逐符号三档清单**（必需 / 可选 / 暂不绑）；`spec/migration-matrix.md` 改成**逐文件四档 + 依赖核对证据** |
| `spec-as-contract` §2「明确不做」 | `spec/ffi-core-ui.md` §10 新增 6 条 out-of-scope；`SPEC.md` §8 新增 **12** 条（含 D13 的无障碍/材质降级、D14 的系统色由 Rust 读） |
| `spec-as-contract` §2「版本锚定」 | `spec/ffi-core-ui.md` §5.4 把版本下限钉死为 **build ≥ 345**（= pinned revision `1840d689…`） |
| `spec-as-contract` §4「端到端验证步骤」 | `spec/ci-contract.md` §7 与 `spec/milestones.md` §7 各留一条可执行验证（含**关键错误流**） |
| `runtime-defensive` 第二部分「校验放在边界」 | `spec/ffi-core-ui.md` §3.2 把内存所有权写成**三分类**硬契约 —— 跨 FFI 就是跨信任域；同进程内的 `ui_*` 调用不加防御性噪音 |
| `runtime-defensive` 第三部分「错误归一化」 | `spec/ffi-core-ui.md` §5.3 把「12 个符号缺失」归一化成**具名能力**，每项有确定降级路径 |
| `runtime-defensive` 第三部分「重试边界要显式」 | `spec/milestones.md` §3 的 M2 要求外部调用**不阻塞 UI 线程**、失败**归一化**、不静默降级 |
| `mechanical-gates` §3「失败即停」 | 把「结构体布局漂移」的补偿从建议改成 **CI 门禁 S8**；把「导出面差异」变成 **门禁 S7** |
| `mechanical-gates` §4「只报真跑过的」 | `spec/ci-contract.md` 全部门禁都绑了**可执行命令**；本机无工具链的事实写进 §6，不得把「未验证」写成「已通过」 |
| `security-baseline` §二「路径穿越 / 命令注入」 | `spec/milestones.md` M2 要求 GuoheView 调用**参数数组化**、路径规范化、日志脱敏（E02/E16） |
| `mvp-stack` | **不适用**：该表是 Web/SaaS 技术栈（Next.js / FastAPI / PostgreSQL 等），与本项目（Rust + Win32 + core-ui DLL）无交集 → **不套用**，只借用其「选型要给代价」的方法 |

---

## 11. 关联文档

| 文件 | 内容 |
|---|---|
| `DECISIONS.md` | 决策台账（D00–D11 + C1~C5 判定） |
| `PROJECT_STATE.md` | 项目状态与下一步 |
| `research/facts.md` | 事实核对底稿（10 组实测） |
| `docs/05-COREUI-EXPORTS.txt` | 本机 core-ui.dll 的 2006 个导出（**绑定清单权威来源**） |
| `docs/05-COREUI-ABI-NOTES.md` | 18 VERIFIED / 12 UNVERIFIED（U01~U12） |
| `docs/11-COREUI-MARKUP-TAGS.md` | `.uix` 标签权威表 + 主题 token 冲突 |
| `docs/10-FORK-ROADMAP-COREUI.md` | 用户 fork 路线图（被覆盖处已标注） |
| `assets/main.uix` | 既定的静态布局（D-15） |
| `spec/adr/` | 关键决策记录 |
