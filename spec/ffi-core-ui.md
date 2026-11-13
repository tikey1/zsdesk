# core-ui FFI 与呈现层契约

> 契约编号：`ZC-FFI-001`
> 状态：已定（随 D06–D11 生效）
> 上游依据：`ghboke/core-ui` @ `1840d689b51a32d392dbf06b8d594aef96696a80`（1.9.0 build 345，2026-11-01）
> 绑定层代码：`spec/rust-ffi-skeleton/`

---

## 1. 四句话结论

1. **状态留 Rust**（T1；已由用户 `assets/main.uix` 的 D-15 注释定调，Spec 落地）。`.uix` 的 QuickJS 只做**视图局部状态**；
   文件列表**根本不在 `.uix` 里** —— 它是 Rust 在运行时创建的 `CustomWidget`，由 Rust 在
   `ui_custom_on_draw` 回调里直接画（挂载方式见 §4.3.1）。
2. **运行时 `LoadLibrary` 动态加载**（D09），不链接 `core-ui.lib`。核心符号缺失 = 加载失败；
   可选符号缺失 = 降级。
3. **预览分三层**（D06）：进程内**不做**图片解码；预览窗格保留但内容降级为「元信息 + 纯文本 + 外部打开」；
   GuoheView 只做外部 exe 入口。
4. **深浅色由库接管**（D10）：Rust 侧调 `ui_theme_set_mode`；
   **`include/ui_core.h` 里不存在运行时替换 CSS 变量的 C API**（已查证，见 §5.6）→
   撞名 token 必须改用库 token，不能指望运行时改 `:root`。

> **本文件的签名来源声明**（ABI-NOTES 硬规则 1、3）：
> **绑定清单以 `docs/05-COREUI-EXPORTS.txt` 为准**（本机 DLL 的 2006 个导出 / **567** 个 `ui_*` 纯 C API）；
> 头文件只提供**签名**且**漏声明** 12 个本机实际导出的符号。
> 本文件里出现的所有签名**全部来自头文件**，属于 `docs/05-COREUI-ABI-NOTES.md` §2 的
> **U01~U12 未核实**状态 —— **不得凭导出名反推参数**；每接入一组 API 必须先过对应的 U 项最小验证。

---

## 2. 绑定策略：手写 `extern "C"` + 运行时符号表

### 2.1 三个候选

| 方案 | 做法 | 代价 |
|---|---|---|
| **A. 手写 extern + 运行时符号表**（选中） | 手工写 `extern "C" fn` 声明；`LoadLibrary` 后按名查符号；必需符号缺失即失败，可选符号缺失即降级 | 结构体布局会漂移 → 用 vendored 头文件 + CI 差异门禁补偿；手写成本（首版约 150 个声明） |
| B. bindgen | `build.rs` 跑 bindgen 生成绑定 | **编译期必须有 core-ui 头文件** → `cargo test --workspace` 依赖「先构建 core-ui」→ 破坏上游 CI 契约；还需要 libclang（CI 要装 LLVM，本机没有） |
| C. 纯链接期 `core-ui.lib` | `#![link(name = "core-ui")]` | 链接期就把两份 DLL 的 API 面差异变成「加载即失败」；且 CRT 不一致直接 `LNK2038`；`cargo test` 同样依赖 core-ui 已构建 |

### 2.2 选 A 的理由（按重要性）

1. **保住上游 CI 契约**。zsdesk 的 CI 第一步是 `cargo test --workspace --release --locked`。
   方案 B/C 会让「core-ui 的 C++ 构建」成为「Rust 单元测试」的前置条件 ——
   core-ui 构建一挂，Rust 侧连测试都跑不了，两套失败混在一个 job 里无法定位。
   动态加载让 Rust 侧**完全不需要 core-ui 存在**就能编译与测试。
2. **API 面差异只能运行时处理**。实测（`research/facts.md` §7.1）：仓库 main 声明 557 个 `ui_*`，
   本机 DLL 导出 567 个，**双方都有 555 个**。链接期会把这 12 个符号的差异变成硬失败；
   运行时可以按符号在/不在优雅降级。
3. **CRT 风险只在运行时的内存边界上**。链接期会直接撞 `LNK2038 RuntimeLibrary mismatch`；
   运行时加载没有这道坎，但代价是**内存所有权必须严格**（见 §3.2）。

### 2.3 选 A 的代价与补偿

| 代价 | 补偿措施（必须执行，不是「建议」） |
|---|---|
| 手写结构体布局可能与头文件漂移 | **首版完全不用 `UiWindowConfig`**：主窗口与设置窗口都走 `.uix` 的 `<window>` 标签 + `ui_page_open_window(page, NULL)`。要用时先核对 pinned revision 的 `include/ui_core.h`，核对结果写进本文件 §7 |
| 声明与上游 API 面脱节 | CI 门禁 S8：从 pinned revision 取出 `include/ui_core.h`，比对本绑定层声明的符号集合，**多出/缺少即失败** |
| 漏绑新 API | 可选符号表 + 能力表（§5.3）：缺失不崩，降级可见 |

---

## 3. 编码、所有权、线程 —— 三条硬契约

### 3.1 字符串编码：逐函数不同

实测：头文件里 `const char*` 出现 **72** 次、`const wchar_t*` 出现 **77** 处。
**同一个库两种编码混用，且逐函数不同。** 规则：

| 类别 | 编码 | 典型函数 |
|---|---|---|
| 资源 / 路径注册 / 页面 id / JSON / 菜单项 id | **UTF-8** | `ui_asset_register_dir`、`ui_page_load_file`（**例外：路径是 UTF-16**）、`ui_page_set_json`、`ui_widget_find_by_id`、`ui_menu_item_id` |
| 界面文本 / 窗口标题 / 拖放路径 / 绘制文本 | **UTF-16LE** | `ui_window_set_title`、`ui_label`、`ui_text_input_get_text`、`ui_window_on_drop`、`ui_draw_text` |

**上层 API 一律收 UTF-8 `&str`**，转换点唯一：`zsdesk-coreui::to_utf16` / `from_utf16`。
绑定层声明里用 `*_utf8` / `*_utf16` 后缀标注（见 `api.rs`）。

> ⚠️ 易错点：`ui_page_load_file` 的**路径是 `const wchar_t*`（UTF-16）**，
> 而 `ui_asset_register_dir` 的**目录是 `const char*`（UTF-8）**。
> 这两个函数挨在一起，很容易写反。

### 3.2 内存所有权：三分类

| 类 | 含义 | 处置 | 本绑定层的例子 |
|---|---|---|---|
| **【借用】** | 返回内部指针 | **不得 free**；对象被修改后失效 | `ui_core_version_string`、`ui_page_last_error`、`ui_menu_item_id`、`ui_menu_item_attr`、`ui_text_input_get_text` |
| **【转移】** | 返回调用方拥有的内存 | **必须**用对应的 core-ui 释放函数回收 | `ui_debug_dump_tree` → `ui_debug_free`；`ui_page_get_json` → `ui_page_free`；`ui_image_get_pixels` → `ui_image_free_pixels` |
| **【拷贝】** | 库内部复制了一份 | 调用方保留源缓冲区所有权，可立即释放 | `ui_bitmap_create`、`ui_draw_bitmap`、`ui_image_set_pixels`、`ui_asset_register_blob` |

**铁律：core-ui 的分配器与 Rust 的分配器绝不交叉。**
任何来自 core-ui 的指针，只能用 core-ui 自己的 free 函数回收。
用 `Vec::from_raw_parts` 或 `String::from_raw_parts` 去 free core-ui 的内存 = UB。

> 这条正是「三方 CRT 一致性」（D09）真正的风险面：即使两边都是 `/MD`，
> 跨模块的 `malloc`/`free` 配对在 MSVC 上恰好可行（同一个 CRT DLL），
> 但**不能依赖**。规则是「谁分配谁释放」，与 CRT 版本无关。

### 3.3 线程：只有 UI 线程能碰 core-ui

`docs/c-api.md` 明示：「工作线程里访问 widget 前先 marshal 到 UI 线程」。
core-ui 内部**没有为 widget 树加锁**，跨线程调用是数据竞争。

| 规则 | 实现 |
|---|---|
| 所有 `ui_*` 调用在 UI 线程 | `UiThread` 独占 `CoreUi`；`UiHandle` 跨线程只带裸指针 + 函数指针 |
| 工作线程 → UI 线程 | `UiHandle::invoke_sync(f)`（底层 `ui_window_invoke_sync`，同步阻塞） |
| 唯一例外 | `ui_core_version` / `ui_core_version_build` / `ui_core_version_string`：头文件明示「无需 ui_init，任意时刻可调」 |
| 死锁红线 | **UI 线程不得 join 一个正在 `invoke_sync` 的工作线程**；关闭顺序 = 停投递 → join → 退 `ui_run()` |

---

## 4. T1 状态归属：Rust 持有状态，`.uix` 只做视图

### 4.1 判据：42k 行 Rust 逻辑要不要重写

上游 zsdesk 的 42k 行 Rust 分层是（实测，`research/facts.md` §7.5）：

| crate | 行数 | 内容 | 与 UI 的耦合 |
|---|---|---|---|
| `zsdesk-core` | 1,999 | `ExplorerSession` / `ExplorerTab` / `ShellLocation` / `ExplorerUi` / `FileEntry` / `natural_cmp` | **零耦合**（crate 文档自述「领域层不依赖 HWND、COM 指针或 ZSUI 控件」） |
| `zsdesk-platform` | 107 | `DirectoryLister` / `IconProvider` / `FileOperator` / `CancellationToken` 等 trait | 零耦合 |
| `zsdesk-windows` | 9,205 | Shell / COM / 图标 / 菜单 / OLE | 无 UI 框架依赖 |
| `zsdesk-ui` | 1,725 | chrome 几何、Fluent 令牌、菜单定义 | **依赖 ZSUI**（feature `windows-win32`） |
| `zsdesk-app` | 29,114 | MVU 状态、命令、窗口宿主、预览进程、桌面格子 | **重度依赖 ZSUI**（`window/`、`desktop/`、`flyout_menu.rs`） |

**关键事实：状态的骨架本来就在 `zsdesk-core`，而且不依赖任何 UI 框架。**
所以「状态放 JS」这个选项的真实代价不是「多写一点胶水」，而是：

- `ExplorerSession`（标签栈、前进后退、收藏树、偏好）要重写成 JS 对象；
- `FileEntry` / `ListingBatch` / `RequestId`（防止旧请求覆盖新标签）要重写；
- `natural_cmp`（与资源管理器一致的本地化排序）要重写；
- 搜索（Everything IPC / MFT / USN）、文件操作（`IFileOperation`）、转发、迁移
  这些**已经是 Rust 且与 UI 无关**的部分，要么留在 Rust（于是变成「双份状态」，
  要做同步），要么一起重写成 JS（于是要引入 Node 侧的文件系统与 COM 绑定，
  远超「换个 UI 库」的范围）。

**结论：状态留 Rust，42k 行逻辑一行不重写。** 只重写「呈现层」
（`zsdesk-ui` 全部 + `zsdesk-app` 里依赖 ZSUI 的那部分），这与 D01 的判断一致。

### 4.2 但 `.uix` 的响应式是真实存在的 —— 那用它做什么

`.uix` 内嵌 QuickJS-NG，Vue 3 SFC 风格，`data()` / `computed` / `methods`，
状态变更触发响应式重绘。这是 core-ui 的**主推交互方式**，不用它等于放弃生态。

**分工**：Rust 是唯一真相源，`.uix` 是**被投影的视图**。

| 归属 | 内容 | 理由 |
|---|---|---|
| **Rust（权威）** | 会话、标签、当前位置、条目列表、排序、选择集、列宽、视图记忆、搜索结果、桌面格子状态 | 这些是领域状态，`zsdesk-core` 已经是这个形状 |
| **`.uix` JS（视图局部）** | 侧栏树的展开/折叠集合、地址栏未提交的编辑文本、过滤框文本、hover 行号、下拉菜单开合、对话框开合 | 这些是「纯 UI 瞬时态」，没有跨会话持久化需求，放 JS 最省事 |
| **`.uix` JS（禁止）** | 文件条目数组、选择集、排序规则、任何 O(n) 数据 | 见 §4.3 的性能论证 |

### 4.3 为什么文件列表**不能**放在 `.uix` 里

三条硬理由（都可实测）：

1. **`.uix` 没有 batching**。`docs/uix-guide.md` §3.5 明示「自动 batching / microtask flushing —— 同步触发」。
   一次 `this.items = [...]` 会**同步**触发所有依赖它的 computed 与绑定求值。
2. **`ui_page_set_json` / `ui_page_get_json` 是全量序列化**。一次 5,000 条目的目录
   （含名称 + 大小 + 时间 + 类型）序列化约 1–2 MB JSON（[估算]），每次目录切换都要
   Rust 序列化 → UTF-8 → QuickJS 解析 → 建对象 → 响应式 diff。
   这是每目录切换 O(n) 的固定成本，不可接受。
3. **`ui_page_watch` 的比较是「整个值的 JSON」**（`docs/uix-guide.md` §19 C）。
   拿它做文件列表的事件通道 = 每次变更 O(n) 序列化 + 比较。

**因此**：

- 文件列表 = **Rust 运行时创建的 `CustomWidget`**，挂进 `.uix` 的宿主容器
  （`<div id="list_host">`）里。挂载方式见 §4.3.1。
- Rust 在 draw 回调里直接画行（`ui_draw_text` + `ui_draw_bitmap_handle`），
  **数据不进 JS**。
- `.uix` 只承载**周边 chrome**：标题栏、地址栏、工具栏、标签条、侧栏树外壳、
  状态栏、预览窗格框架。

### 4.3.1 自绘控件怎么挂上去 —— **裁定：在 `.uix` 里声明 `<custom>`（静态声明）**

> 选项编号沿用前端 `docs/12-UIX-TAG-FIX.md` R5 的口径：
> **(a) = 运行时 `ui_custom()` + `ui_widget_add_child()`**；
> **(b) = `.uix` 里声明 `<custom id="file_list"/>`**。
> **裁定：选 (b)。**

**决定：`.uix` 里写 `<custom id="file_list" class="list-host"/>`，
Rust 用 `ui_page_on_widget_mount(page, "file_list", cb, ud)` 取句柄后挂绘制回调。**

**理由（决定性的一条是布局，不是 API 好不好用）：**

| 判据 | (a) 运行时挂载 | (b) 静态声明 |
|---|---|---|
| **尺寸从哪来** | 运行时 `ui_custom()` 造的控件**没有 CSS 级联结果** → 无 `flex-grow` / `width` / `height` → 在 flex 容器里**尺寸为 0** → **列表区空白** | core-ui 的工厂对 `<custom>` 明确走 CSS：`widget_factory.cpp:1114-1118` 注释原文「**CSS width/height 走 ApplyCommonStyle 自然生效**」→ `.list-host { flex: 1; }` 生效 |
| 回调是否可能 no-op | 否（自己造的必然是 CustomWidget） | **可能**：`ui_custom_on_draw` 的实现是 `As<ui::CustomWidget>(w)`，取不到就**静默 no-op**（`ui_api.cpp:2612-2616`）。但 `<custom>` 标签在工厂里**必然**造 CustomWidget → 不会 no-op |
| 布局重算 | `ui_widget_add_child` 会 `MarkLayoutAndRepaint()`，flex 会重跑（`flex.cpp` 遍历 live children）→ 但尺寸仍是 0 | 不需要 |
| Rust 要不要自己管 resize | 要（`ui_widget_set_size` + 每次 resize 跟随）→ 脆弱 | 不要 |
| 与「Rust 持有 EntryStore、`.uix` 只做视图」 | 相容（`.uix` 只有 div） | **相容**：`<custom>` 是**一个空画布**，**行**仍然全部由 Rust 画 —— 用户 `assets/main.uix` 注释写的是「**List rows** are NEVER created here」，不是「the list host is never created here」 |

**与 D-15 的一致性说明**：`assets/main.uix` 头注释的原文是
「List rows are NEVER created here; the file list is drawn by `ui_custom_on_draw` from the Rust EntryStore」。
→ 它约束的是**行**不由 `.uix` 创建；`<custom>` 只是一个**空的自绘画布**，
不违反 D-15。`.uix` 仍然**没有事件属性、没有业务脚本、不知道 Rust 的存在**。

**Rust 侧取句柄的两条路（都可用，建议后者）**：

| 路 | 做法 | 评价 |
|---|---|---|
| `ui_page_on_widget_mount(page, "file_list", cb, ud)` | 按 id **持久化**注册；页面重建后**自动回填**新实例 | ✅ **推荐**：主题切换会重建页面（§5.6.1），持久化回调能自动接上 |
| `ui_widget_find_by_id(page_root, "file_list")` | 直接按 id 取句柄 | 可用，但**页面重建后必须重取**（旧句柄失效） |

**页面重建时的自绘控件**：(b) 方案下 `<custom>` 是**静态声明**的 → 页面重建后
core-ui 会**重新造**一个 CustomWidget → **Rust 只需重挂回调**（`ui_page_on_widget_mount`
会自动回填），**不需要**重新 `ui_widget_add_child`。这比 (a) 少一层状态管理。

### 4.3.2 页面用 `ui_page_load_string` 加载，不是 `ui_page_load_file`（D10 的直接后果）

**实测依据**：库**没有**运行时替换 CSS 变量的 C API（前端 `docs/12-UIX-TAG-FIX.md` §4.2 独立复核，
与本文件 §5.6 的查证结论一致）。项目 token 只能靠 **Rust 在加载前把 `:root { … }` 拼进 `.uix` 源码** 生效。

| 项 | 决定 |
|---|---|
| 加载 API | **`ui_page_load_string(const char* html_source)`**（UTF-8 源码）—— 不是 `ui_page_load_file` |
| Rust 流程 | 读 `assets/main.uix`（UTF-8）→ 读 `assets/themes.json` → 拼 `:root { --app-*: … }` → 拼进源码 → `ui_page_load_string` |
| 编码 | `ui_page_load_string` 收 `const char*` = **UTF-8**（与 `ui_page_load_file` 的 `const wchar_t*` **不同**，别写混） |
| **打包后果** | ★ **`.uix` 与 `themes.json` 必须以文件形式进包** —— Rust 要在运行时读它们并拼源码。若只打进 exe 就无法注入 `:root` |
| 主题切换 | `ui_page_reload` 会**重放旧源码**（改不了 palette）→ 切换明暗必须**重新 `ui_page_load_string` 一个新页面**，并重走 §4.3.1 的装配流程 |

### 4.3.3 ★ `.uix` 解析的三条硬约束（2026-11-13 源码级核对）

> 需求来自前端在改 `.uix` 资产时发现的三处「资产写法与 core-ui 实际语义不符」。
> **架构师已独立核对上游源码**（下面每条都带行号），结论与前端一致，
> 但**路径与机制有两处需要修正**（见每条的「修正」行）。

#### 约束 1：`<window>` 必须自闭合，否则**整页 load 失败**

**核对结论：成立。**

```cpp
// src/ui/uix/sfc_parser.cpp:285-297
if (tagName == "window") {
    if (doc.hasWindow) { pushErr(doc, "duplicate <window> tag", tagLine, tagCol); }
    doc.hasWindow   = true;
    doc.windowAttrs = std::move(attrs);
    doc.windowLine  = tagLine;
    if (!selfClosed) {
        pushErr(doc, "<window> must be self-closing (use <window .../>)", tagLine, tagCol);
        skipUntilClose(c, "window", doc);
    }
}
```

| 事实 | 依据 |
|---|---|
| 非自闭合 → `pushErr` 记一条错误 | `sfc_parser.cpp:292-296` |
| `pushErr` **本身不**置 `ok=false`，但 `sfc_parser.cpp:373` 有 `if (!doc.errors.empty()) doc.ok = false;` | → **任何解析错误 = 整份文档失败** |
| 上游 demo 的写法确实是自闭合 | `demo/ui_demo.uix:1` `<window title="Core UI .uix Demo" width="1100" height="720" ... />` |
| **修正** | 前端引用的路径 `src/ui/page/sfc_parser.cpp` **不对** —— 实际在 **`src/ui/uix/sfc_parser.cpp`**（`src/ui/page/` 下没有这个文件）。**行号 294 是对的**（正是那句 `pushErr`） |

**契约**：

1. `.uix` 根标签必须是 `<window ... />`（自闭合）。
2. **Rust 侧必须能区分两类失败**：
   | 类别 | 判据 | 报错内容 |
   |---|---|---|
   | **整页解析失败** | `ui_page_load_string` 返回 **NULL / 0** | 取 `ui_page_last_error(page)` 的 UTF-8 文本（含行号列号），原样呈现 |
   | **局部控件缺失** | 加载**成功**，但 `ui_widget_find_by_id(root, id)` 返回 **0** | 「`id=<name>` 未在页面中找到」，并列出该页面声明的 id 清单供对照 |
   → **禁止**只判 `find_by_id` 的结果（那样整页解析失败会被误报成「某个控件缺失」）。
3. `ui_page_last_error` 是**【借用】**指针，只在回调/调用期间有效 → Rust 侧**当场拷成 `String`** 再用。

#### 约束 2：`text="..."` **不是**文本属性 —— 文字必须是**元素的第一个直接子文本节点**

**核对结论：成立，且前端描述需精确化。**

```cpp
// src/ui/page/widget_factory.cpp:54-61
std::string FirstPlainText(const ui::uix::Node& n) {
    for (const auto& c : n.children) {
        if (c->kind == ui::uix::NodeKind::Text)         return c->text;
        if (c->kind == ui::uix::NodeKind::Interpolation) return "";  // handled by binding
        if (c->kind == ui::uix::NodeKind::Element)      return "";  // non-text follows
    }
    return "";
}
// src/ui/page/widget_factory.cpp:1295-1301
std::string firstText = FirstPlainText(node);
w = ConstructByTag(tag, firstText, style, node);
```

| 事实 | 依据 |
|---|---|
| 控件文本 = 元素的**第一个直接子节点**若是 `Text` 则取其内容 | `widget_factory.cpp:54-61` |
| 若是 `Interpolation`（`{{ }}`）→ 返回 `""`，**由 binding 处理** | 同上，第 57 行 |
| 若是 `Element` → 返回 `""`（非文本优先） | 同上，第 58 行 |
| `text=` 属性**只在 `<menu>` 上被读**（作为 `title` 的别名） | `compiler.cpp:641/644`：`if (a.name == "title" || a.name == "text")` |
| `<label>` / `<span>` 等**完全不读** `text=` 属性 | `widget_factory.cpp` 全文无 `"text"` 属性读取（命中的三处分别是 `user-select` 的值、`cursor` 的值、`input` 的默认 type） |

**契约**：

1. `.uix` 里的文字一律写成**元素内部文本**：`<label>标签页由 Rust 侧 TabManager 提供</label>`；
   `<label text="…"/>` **无效**（文字为空）。
2. **Rust → `.uix` 推状态**一律走 `ui_page_set_text` / `ui_page_set_json` +
   模板里的 `{{ }}` 插值 —— **不靠 `text=` 属性**。
3. 插值节点（`{{ }}`）的初值由 binding 负责，`FirstPlainText` 对它返回 `""` → **不冲突**。

> **对「状态由 Rust 经 `ui_page_set_*` 推给 `.uix`」的影响**：
> 通道**不变**（`ui_page_set_text` / `set_json` → `{{ }}` 插值 → 重绘），
> 但**前提是模板里写的是 `{{ }}` 而不是 `text=`**。
> 这条要写进 `assets/*.uix` 的评审清单。

#### 约束 3：`window { … }` 选择器**匹配不到任何东西**；`:root { … }` **只定义 CSS 变量**

**核对结论：成立，但机制要说准（前端表述需修正）。**

```cpp
// src/ui/css/selector.cpp:57-70  —— 简单选择器的匹配规则
case SimpleKind::Tag:    return node.tag == ss.name;      // 按“标签名”匹配
case SimpleKind::Class:  return node.HasClass(ss.name);   // 按 class 匹配
case SimpleKind::Id:     return node.id == ss.name;
case SimpleKind::Pseudo: { uint32_t bit = PseudoToBit(ss.name); ... }
```

```cpp
// src/ui/css/cascade.cpp:9-16   —— IsRootSelector
// src/ui/css/cascade.cpp:124-126 —— ComputeStyle 里：
//   // Skip :root — those only define CSS vars, handled separately
//   if (IsRootSelector(sel)) continue;
```

| 事实 | 依据 |
|---|---|
| `window { … }` 是**标签选择器**，要求存在 `tag == "window"` 的节点 | `selector.cpp:59` |
| `<window>` 是 **SFC 层面的窗口配置标签**，**不产生任何 widget** | `sfc_parser.cpp:285-296`（只把 attrs 存进 `doc.windowAttrs`） |
| → **没有任何节点的 tag 是 `window`** → 该规则**匹配 0 个节点** → 整条无效 | 推论 |
| `:root` 是**伪类**（`state::Root`），但它在普通样式计算里被**显式跳过** | `selector.cpp:12` + `cascade.cpp:125` |
| `:root` 只用于**采集 CSS 自定义属性**（变量） | `cascade.cpp:18` `CollectRootVars` |
| class 选择器按 class 名匹配 | `selector.cpp:61` |

**契约**：

| 想做的事 | ❌ 错误写法 | ✅ 正确写法 |
|---|---|---|
| 设根窗口的背景/文字/字体 | `window { background: …; color: …; }` | **`<div class="root">` + `.root { background: …; color: …; }`** |
| 定义 CSS 变量（token） | — | `:root { --app-bg: #F3F3F3; … }`（**只放变量，不放普通样式**） |
| 根窗口尺寸 | `window { width: 100%; }` | `.root { flex: 1; }`（根 `<div>` 是 `<template>` 的唯一根元素，本就在 flex 容器里） |

> **修正**：前端 `docs/12` §2.3 写的是「`window { }` 选择器落到根节点 `.root`」——
> **结论对（用 `.root`），但机制不是「window 选择器自动落到 root」，而是
> 「`window` 标签选择器匹配不到任何东西；必须显式给根 `<div>` 加 `class="root"` 再用 `.root` 选择器」**。
> 写法上这两者结果一样，但**前者会让人以为 `window` 选择器有效**，将来 core-ui 若加了同名节点就会踩坑。


### 4.4 完整数据流

```
┌───────────────────────────────────────────────────────────────────────┐
│ 用户输入（鼠标 / 键盘 / 拖放）                                        │
└───────────────┬───────────────────────────────────────────────────────┘
                ↓
┌───────────────────────────────────────────────────────────────────────┐
│ core-ui 事件分发（UI 线程）                                          │
│  · 窗口级：ui_window_on_key / on_drop / on_right_click / on_menu     │
│  · 控件级：ui_widget_on_click / ui_custom_on_mouse_* / on_key_down   │
│  · 页面级：ui_page_watch（JS 改了响应式变量时回调）                   │
└───────────────┬───────────────────────────────────────────────────────┘
                ↓ 全部是 C 函数指针，直接进 Rust（不经过 JS）
┌───────────────────────────────────────────────────────────────────────┐
│ Rust 回调（UI 线程）                                                 │
│  · 转成 UiCommand（枚举）投递到 zsdesk-app 的状态机                        │
│  · 绝不在这层直接改 UI                                                │
└───────────────┬───────────────────────────────────────────────────────┘
                ↓
┌───────────────────────────────────────────────────────────────────────┐
│ Rust 状态机（zsdesk-app，唯一真相源）                                     │
│  · 改 ExplorerSession / 选择集 / 排序 / 视图记忆                       │
│  · diff 出「哪些视图状态变了」                                        │
└───────────────┬───────────────────────────────────────────────────────┘
                ↓
        ┌───────┴────────┬─────────────────────────┐
        ↓                ↓                         ↓
┌──────────────┐ ┌──────────────────┐  ┌──────────────────────────────┐
│ 小载荷投影    │ │ 文件列表模型      │  │ 预览模型                    │
│ ui_page_set_*│ │ zsdesk-ui 的 filelist │  │ ui_image_set_pixels 等      │
│ (tab 列表/   │ │ 状态（Rust 侧）   │  │                              │
│  树节点/状态)│ │                   │  │                              │
└──────┬───────┘ └────────┬─────────┘  └──────────────┬───────────────┘
       ↓                  ↓                            ↓
┌──────────────┐ ┌──────────────────┐  ┌──────────────────────────────┐
│ .uix 响应式   │ │ ui_custom_on_draw│  │ core-ui 图片控件重绘          │
│ → 重绘 chrome │ │ → Rust 直接画行   │  │                              │
└──────────────┘ └──────────────────┘  └──────────────────────────────┘
```

### 4.5 三个必须说清的问题

**Q1：谁发事件？**
core-ui 发。core-ui 把事件派发成**我们注册的 C 函数指针**。
`.uix` 里的 `@click="method"` 是 core-ui 内部消费 —— 我们**不注册**这类回调，
而是在 Rust 侧用 `ui_widget_on_click(widget, cb, ud)` 直接挂 C 回调。
好处：交互**不经过 QuickJS**，少一层序列化与延迟。

**Q2：谁持有状态？**
Rust。`.uix` 的 `data()` 里只有 §4.2 表中的「视图局部」字段。

**Q3：谁负责 diff？**
Rust。Rust 状态机改完后自己 diff（上游 zsdesk 已有这套 MVU 骨架，见
`zsdesk-app/src/app.rs` + `commands.rs`），diff 结果决定：
- 哪些 `.uix` 响应式变量要 `ui_page_set_*` 更新；
- 文件列表哪几行脏了（只需 `ui_window_invalidate` 或局部重绘）。

**Q4：回调在哪个线程？**
**UI 线程**。core-ui 的所有回调（窗口级、控件级、页面级、draw）都在 UI 线程触发。
工作线程的结果必须经 `invoke_sync` 回 UI 线程。

**Q5：回调里能调哪些 `ui_*`？**
- **可以**：所有 `ui_*`（因为已经在 UI 线程）。
- **但 draw 回调里禁止**：任何会**同步触发重绘**的调用（`ui_widget_set_visible`、
  `ui_window_invalidate`、`ui_page_set_json`）。draw 回调是「记录绘制指令」，
  在里面改状态会造成重入。正确做法：draw 回调只读快照 + 画；
  需要改状态时**记到 pending 队列**，在 draw 回调**之外**（例如
  `ui_custom_on_mouse_down` 回调里）改，然后调 `ui_window_invalidate`。

**Q6：重入是否安全？**
| 场景 | 安全性 |
|---|---|
| draw 回调里调 `ui_draw_*` | 安全（这就是它的用途） |
| draw 回调里调 `ui_widget_set_*` / `ui_page_set_json` | **不安全** —— 可能同步触发重绘 → 重入。禁止 |
| 事件回调里调 `ui_widget_set_*` / `ui_page_set_*` | 安全（core-ui 常规路径） |
| 事件回调里调 `ui_window_invalidate` | 安全 |
| 事件回调里调 `ui_menu_show` | 安全，但**会立即弹出菜单**；`.uix` 的 `<menu trigger>` 路径会自动处理锚点，优先用后者 |
| 事件回调里调 `DoDragDrop`（OLE 拖出） | core-ui **明确支持** —— `src/ui/ui_window.cpp` 注释：<br>「hook 可能跑模态循环（宿主在 @mousemove 里起 OLE DoDragDrop 拖出）：DoDragDrop 内部 SetCapture 夺走 capture → WM_CAPTURECHANGED 同步派发 → CancelMouseCapture() 清空 pressedWidget_。返回后必须重查」 |
| 事件回调里调 `ui_msgbox`（模态） | 安全（core-ui 支持模态），但模态期间 UI 线程被占，工作线程的 `invoke_sync` 会排队 |

---

## 5. 版本与能力协商

### 5.1 实测差异（`docs/05-COREUI-ABI-NOTES.md` V02~V05）

| 项 | 数量 |
|---|---|
| DLL 总导出 | **2006** |
| DLL 的 `ui_*` 纯 C API | **567** |
| 头文件声明的 `ui_*` | 557 |
| 双方都有（交集） | **555** |
| **DLL 比头文件多** | **12**：`ui_flush_gpu`、`ui_image_evict_tile`、`ui_image_view_gdi`、`ui_image_gdi_{set_file,set_pixels,set_parent,set_zoom,get_zoom,width,height,fit,clear}` |
| **头文件比 DLL 多** | **2**：`ui_core_embed_binary`、`ui_dialog` |

> **两处必须写清的修正**：
>
> 1. **`ui_core_embed_binary` 不是 C API** —— 它是 CMake 辅助函数
>    （`cmake/embed_binary.cmake` 的 `ui_core_embed_binary(TARGET FILE … OUT … VAR …)`），
>    用来生成内嵌字节的 .h 文件。**它从来不会出现在任何 DLL 的导出表里**，
>    因此**不在绑定清单里**，也不属于「缺失即降级」的能力。
>    发布态的资源注册走 `ui_asset_register_blob`。
> 2. **那 12 个「DLL 独有」符号在头文件里没有声明** → **签名未知** →
>    **不绑定、不调用**（ABI-NOTES 硬规则 3）。绑定层只对它们做**存在性探测**
>    （`GetProcAddress` 返回非空即存在），用于诊断报告。
>    它们的语义要等 **U08** 真机核对后才能用。

### 5.1.1 CI 构建产物 vs 本机 DLL 的导出面（关键推论）

CI 从**仓库 main** 编译 core-ui（D03），而那 12 个符号在仓库的 `include/ui_core.h` 里没有声明
（V04 原文：「上游头文件漏同步声明，不是本机版本领先」）。
→ **CI 产物上的 `ui_*` 导出面预计 = 555（交集）+ `ui_core_embed_binary`（若真被导出）+ `ui_dialog`（若真被导出）**，
**不含**那 12 个。

因此：
- 绑定层的**必需符号**必须落在 **555 的交集**里（两处都有）；
- 门禁 **S7** 比对 CI 产物的导出面时，**允许的差异集合 = {`ui_core_embed_binary`, `ui_dialog`}**，
  **出现第三类差异即失败**；
- `image_gdi` 能力族在 CI 产物上**必然不可用** → 任何功能**不得**依赖它。

### 5.2 协商方式

**不按版本号，按符号存在性。** 理由：本机那份 DLL 的 build 号未知，
版本号与能力不是单调关系（「本机更新」不等于「API 面更大」）。

```rust
// 必需符号：缺失 → 加载失败（LoadError::Missing），大声失败。
// 可选符号：缺失 → Option::None，调用方判 is_some() 后降级。
```

### 5.3 能力表（`Capabilities`）

| 能力 | 符号 | 缺失时的降级 |
|---|---|---|
| `dialog` | `ui_dialog`（头文件有声明，本机 DLL 无导出） | 用 `ui_msgbox` 模态对话框兜底 |
| 「本机 DLL 独有」族 | 12 个 `ui_image_gdi_*` / `ui_image_view_gdi` / `ui_image_evict_tile` / `ui_flush_gpu` | **只探测、不调用**（签名未知）；任何功能不得依赖它们 |

> 「本机 DLL 独有」族**不作为能力开关**存在 —— 它们在 CI 产物上根本不存在，
> 把它做成 `Option` 能力只会诱导后来者写出「在开发机上能跑、CI 产物上崩」的代码。

### 5.4 版本下限

绑定层要求 **build ≥ 345**（= pinned revision `1840d689…` 的版本）。
低于此值 → `LoadError::TooOld`，**不做降级**。

### 5.5 revision 戳记（证明「手边这份 DLL 就是 CI 构建的那份」）

CI 打包时在 `core-ui.dll` 旁写一个 `core-ui.dll.revision`（纯文本一行 SHA）。
运行时读取并与编译期固化的 `EXPECTED_REVISION` 比对：

| 结果 | 处置 |
|---|---|
| 一致 | 正常 |
| 不一致 | **打警告**（不阻断启动 —— 兜底加载本机 GuoheView 的 DLL 是允许的降级），写进诊断报告 |
| 缺失 | 同上，警告 |

CI 侧另有一道**硬门禁**：DLL 缺失 / 戳记不匹配 → 打包步骤失败（见 `spec/ci-contract.md` §5 S7）。

### 5.6 深浅色与 CSS token（D10）—— 两个查证结论

#### 结论一：**需要** Rust 侧调 `ui_theme_set_mode`

| 事实 | 来源 |
|---|---|
| `ui_theme_set_mode(UiThemeMode)` 存在（`UI_API void ui_theme_set_mode(UiThemeMode mode);`） | `include/ui_core.h` + `docs/05-COREUI-EXPORTS.txt` |
| `UiThemeMode`：`UI_THEME_DARK = 0` / `UI_THEME_LIGHT = 1` | `include/ui_core.h` |
| 切模式会触发 `RefreshAllPageThemes` → `RebuildThemeVars`，遍历所有 live page 重建变量 | `docs/11-COREUI-MARKUP-TAGS.md` §5.2.3（源码级） |

→ Rust 侧**需要**调 `ui_theme_set_mode` 来实现「跟随系统 / 用户手动切换」。
这是 core-ui **唯一**的明暗切换入口。

#### 结论二：**不存在**运行时替换 CSS 变量的 C API

已用两种方式查证，结论一致：

| 查证方式 | 结果 |
|---|---|
| `include/ui_core.h` 全文 grep `css_var` / `var_set` / `set_var` / `stylesheet` / `token` | **零命中** |
| `docs/05-COREUI-EXPORTS.txt` 的 **567** 个 `ui_*` 导出里 grep `var` / `css` / `style` | **只有** `ui_gh_img_view_{get,set}_shadow_style`（阴影样式，与 CSS 变量无关） |

**主题相关的 C API 全集只有这些**（无一能改 `:root` 变量）：

```
ui_theme_set_mode / ui_theme_get_mode
ui_theme_bg / content_bg / sidebar_bg / toolbar_bg / accent / text / divider   ← 只读
ui_theme_set_accent / ui_theme_set_accent_hex                                  ← 只能改品牌色
ui_theme_set_default_font / ui_theme_set_cjk_font
ui_theme_set_text_locale / ui_theme_set_text_render_mode
```

→ **「运行时替换 CSS 变量」= 无。**

唯一能在运行时改变颜色的 C API 是 **`ui_theme_set_accent_hex(const char*)`**，
且头文件明示它**跨 `ui_theme_set_mode` 切换深浅色仍保留**（品牌色）。

#### 由此得出的 token 路线（D10 的内容）

库的 34 个 token 与本 fork `assets/themes.json` 的撞名情况
（`docs/11-COREUI-MARKUP-TAGS.md` §5.3）：**只有 3 个撞名** ——
`--bg`、`--accent`、`--sidebar-bg`。

| 机制 | 行为 | 后果 |
|---|---|---|
| 首次加载 | 用户 `:root` 同名值**优先**（库用 `has("--accent")` 守卫跳过自己的默认） | 首屏颜色 = 用户的 |
| `ui_theme_set_mode` 之后 | 库**擦除**用户钉住的同名 key（`compiler.h` 原文：the library owns these keys） | **界面会突然变色** |

**三选一（架构侧初判）**：

| 路线 | 做法 | 代价 |
|---|---|---|
| (a) 改名避让 | `--bg`→`--app-bg`、`--accent`→`--app-accent`、`--sidebar-bg`→`--app-sidebar-bg` | 用户 token 不再被擦除 → 明暗切换时**不变** → 需自己提供两套值的切换机制，而运行时改 CSS 变量的 API 不存在 → 只能「重新加载页面」 |
| (b) 改用库 token | `--bg` → `--window-bg` / `--bg-2`；`--accent` 直接用库 accent | 值随库主题变；`themes.json` 的 light/dark 两套大部分作废；**可能与语义撞车** |
| (c) 混用 | 品牌色走库 accent，布局底色走避让名 | token 表最复杂 |

> ⚠️ **本节已被前端 `docs/12-UIX-TAG-FIX.md` §4.3 的逐 token 终表取代**
> （该文档做了更深一层的源码核对，发现了架构侧初判漏掉的一点：
> **库的 `--bg` = 内容区底色（light `#FFFFFF`），而本项目的 `--bg` = 窗口底色（`#F3F3F3`）——
> 两者不仅同名，语义还相反**）。架构侧采纳前端的终表。

### 5.6.1 最终 token 方案（采纳前端终表，架构侧只补运行时契约）

| 类别 | 数量 | 做法 |
|---|---:|---|
| **改名避让** | **16** | `--bg`→`--app-bg`、`--content-bg`→`--app-content-bg`、`--sidebar-bg`→`--app-sidebar-bg`、`--toolbar-bg`→`--app-toolbar-bg`、`--text`→`--app-text`、`--text-secondary`→`--app-text-secondary`、`--divider`→`--app-divider`、`--hover`→`--app-hover`、`--pressed`→`--app-pressed`、`--selection-bg`→`--app-selection-bg`、`--info-*`/`--warning-*`/`--danger-*` 共 6 个 → `--app-*` | 统一到 `--app-*` 命名空间，**一个都不保留原名**；这些键**不在**库的 `kThemeKeys` 里 → 不会被 `RebuildThemeVars` erase |
| **改用库 token** | **1** | **accent** —— CSS 侧读 `var(--accent)`，**不自定义**；由 Rust 侧 `ui_theme_set_accent_hex()` 写进库 palette 并同步到库 token，且**跨 `ui_theme_set_mode` 保留** | 若自定义 `--app-accent`，库自绘控件（primary 按钮 / 焦点环 / 滑块 / 进度条）仍是库默认蓝 `#3971EC`，与自绘区的 `#2563EB` **撞成两种蓝** |

**架构侧补的三条运行时契约**：

#### 契约 1：Rust 必须在 `ui_page_load_string` 之前注入 `:root` 块

读 `assets/themes.json` → 拼 `":root { --app-bg: #F3F3F3; … }"` → 拼进 `.uix` 源码 →
`ui_page_load_string`。
→ 页面用 **`ui_page_load_string`** 加载（§4.3.2）；`.uix` 与 `themes.json` **必须以文件形式进包**。

> **顺序是硬要求**：注入必须**早于** load。晚一步 → 自定义属性不存在 →
> `var(--app-bg)` 解析失败 → **静默回落到库默认色**，界面白得像「能跑」但配色全错。

#### 契约 2：Rust 初始化时的调用顺序与各自的影响范围

```
1. ui_theme_set_default_font(L"Segoe UI Variable")
   ui_theme_set_cjk_font(L"Segoe UI", L"Microsoft YaHei UI")
2. ui_theme_set_mode(UiThemeMode::Light /* 或 Dark */)
3. ui_theme_set_accent_hex("#2563EB")        // = 该模式 themes.json 的 accent
4. 拼 ":root { --app-bg: …; … }" 注入 .uix 源码 → ui_page_load_string(...)
```

| 步 | 影响范围 | 顺序为什么重要 |
|---|---|---|
| 1 | 控件字体（`theme::kFontFamily`）；**`.uix` 里的 `font-family` 对普通 LabelWidget 不生效**（`renderer.cpp` 读该属性的三处都在 SVG 文本路径） | 必须在 load 之前，否则首帧字体不对 |
| 2 `set_mode` | `RefreshAllPageThemes` → `RebuildThemeVars`：**重写库的 34 个 token**，并**擦除用户同名 `:root` 值**（`--app-*` 不在其中 → **不受影响**） | — |
| 3 `set_accent_hex` | `theme::SetAccent` → `RefreshAllPageThemes` → `RebuildThemeVars` → 同步到**库 token `--accent`**；**跨 `set_mode` 保留**（`theme.h:435-439`） | **必须在第 2 步之后或紧邻**：`set_mode` 会用**当前**的 accent override 重新派生 —— 顺序反了会拿到上一次的 accent |
| 4 | 加载页面 | 必须在 1–3 之后 |

> 第 3 步不能省：省了的话 `var(--accent)` 落到库默认 `#3971EC`，
> 与设计稿的 `#2563EB` 不符 → 表现为「自绘区蓝、按钮蓝」**两种蓝**。

#### 契约 3：深浅色切换 —— `--app-*` **不会自动变**，缺口用「Rust 侧重建页面」补

**先回答「会不会变」：不会。**

| 键 | 切 `ui_theme_set_mode` 时 | 依据 |
|---|---|---|
| 库的 34 个 token | **会被 erase 并用库 palette 重写** | `compiler.cpp:1672-1700`（`RebuildThemeVars`） |
| `--app-*`（16 个） | **不受影响** —— 它们**不在**库的 `kThemeKeys` 里，`RebuildThemeVars` 只处理那 34 个键 | `compiler.cpp:1647-1660` 的 `kThemeKeys[]` |

**但缺口是真的**：`:root` 只有**一套** → `themes.json` 的 light / dark 两套值**无法自动切换**。
只切 `set_mode` 的话 → **库控件变深、自绘的条/列表/面板/分隔线/文字还是浅色**（半深半浅）。

**候选与推荐**：

| 候选 | 做法 | 代价 | 判定 |
|---|---|---|---|
| **A. Rust 侧重建页面** | 用另一套 `--app-*` 重新拼 `:root` → 重新 `ui_page_load_string` 一个新 page → 重挂自绘控件回调 | 重建瞬间有一次闪烁；页面编译耗时（几十 ms 级 [估算]）；装配流程必须幂等 | ✅ **推荐** |
| B. 加载两份 `.uix`（light/dark 各一份） | 按主题加载对应那份 | **等价于 A**（仍是重建页面），但**两份文件要同步维护** → 改一处漏一处 | ❌ 无额外收益 |
| C. 只支持跟随系统 + 单套值 | 只做一套 token | **放弃明暗两套设计**（用户的 `themes.json` 有两套）→ 深色下不可用 | ❌ 不可接受 |

**推荐 A，并分两期落地**：

| 期 | 范围 | 说明 |
|---|---|---|
| **M1** | **启动时按系统主题选一套** → 拼 `:root` → load | 只有**启动时一次**重建/加载 → 零闪烁成本、零幂等压力。运行时手动切换**不做** |
| **M2+** | 手动切换：重建页面 | 重建有闪烁 → 需要评估并缓解（例如重建期间保留旧页的位图，直到新页就绪）。多标签时**只重建当前可见页**，其余标签在切到时按需重建（懒重建） |

> **为什么「重建页面」在咱们这儿不贵**：因为**状态在 Rust 的 EntryStore 里**，
> `.uix` 是**无状态视图** → 页面重建后 Rust 侧只需**重新投影**一次状态，
> 不需要从 `.uix` 里恢复任何东西。这正是 ADR-0001「状态留 Rust」的回报。
> （若状态在 JS 里，重建页面就会丢状态 —— 那才是这条路线真正的代价。）

#### 代价（必须承认）

1. 明暗切换时，**库自绘控件跟随库 palette 变，本项目自绘区域（`--app-*`）不会自动跟随** ——
   必须重建页面才变。这是改名避让路线的固有代价，已被接受；
2. **重建页面 = 重建控件树** → 自绘控件要重新取句柄并重挂回调（§4.3.1 的
   `ui_page_on_widget_mount` 按 id 持久化，**会自动回填** → 比运行时挂载省一层状态管理）；
   重建期间有一次闪烁（M1/M2 要量一下）；
3. 门禁 **S13**（最终表述见下）。

#### 门禁 S13 · 最终表述

> **S13 —— 禁止自定义任何「库 token 名」**
>
> **扫描 `assets/*.uix` 与 `assets/themes.json` 的自定义属性「定义」**
> （形如 `--name:` 的声明，**不包括** `var(--name)` 的引用）：
> 若 `name` ∈ 库的 **34 个 token 全集**
> （`--accent --accent-hover --accent-press --accent-text --accent-selected
> --window-bg --window-border --bg --bg-2 --bg-3 --bg-4 --bg-elevated
> --fg --fg-2 --fg-3 --fg-4 --fg-on-accent --border --border-subtle
> --sidebar-bg --sidebar-text --sidebar-hover
> --input-bg --input-border --input-border-hover --input-border-focus
> --card-bg --card-border --btn-bg --btn-hover --btn-press --btn-text
> --disabled-bg --disabled-text`）
> → **判失败**。
>
> **为什么从「3 个」扩到「34 个全集」**：
> ① 34 个里**任何一个**撞名，都会在 `set_mode` 时被 erase → 表现为「切主题后某处颜色突变」，
>    且**静默**（没有报错）；② 「挑几个改名」会让人误以为「其余的可以自定义」→
>    将来新增库 token 时会再踩一次；③ 统一到 `--app-*` 命名空间后，
>    **自定义属性与库 token 在名字空间上完全不相交** → 这条门禁可以用
>    「名字空间不相交」一句话概括，不依赖 34 个名字的清单是否最新。
>
> **附带两条**：
> - **禁止定义 `--app-accent`** —— accent 走**库 token**（`var(--accent)` 引用），
>   由 Rust 侧 `ui_theme_set_accent_hex()` 钉住；自定义它会导致「自绘区蓝 + 库控件蓝」两种蓝；
> - **允许** `var(--accent)` 这类**引用**（不是定义）→ 门禁只拦「定义」不拦「引用」。

**M1 验收项 V1.9**（深浅色切换不突变）就是这条的直接检验。

---

## 6. 首版绑定的子集清单

**原则**：首版只绑「M0 + M1 一定用得到」的符号；能通过 `.uix` 声明式做到的，不绑 imperative API。

> **清单的权威来源是 `docs/05-COREUI-EXPORTS.txt`（567 个 `ui_*`）**，
> 不是头文件（557 个）。头文件漏声明了 12 个本机实际导出的符号。
> **CI 门禁 S8** 会用导出表逐名校验本清单。

| 层 | 符号 | 首版 |
|---|---|---|
| 版本 | `ui_core_version` / `ui_core_version_build` / `ui_core_version_string` | ✅ 必需 |
| 生命周期 | `ui_init` / `ui_init_with_theme` / `ui_shutdown` / `ui_run` / `ui_quit` | ✅ 必需 |
| 窗口 | `ui_window_show` / `set_root` / `set_title` / `invalidate` / `relayout` / `hwnd` / `dpi` / `dpi_scale` / `set_min_size` / `set_rect` / `get_rect_screen` / `invoke_sync` | ✅ 必需 |
| 窗口回调 | `on_close` / `on_close_request` / `on_resize` / `on_drop` / `on_key` / `on_right_click` / `on_menu` / `on_mouse_button` | ✅ 必需 |
| 控件 | `ui_widget_find_by_id` / `set_id` / `add_child` / `remove_child` / `destroy` / `get_rect` / `set_visible` / `set_enabled` / `get_visible` / `get_enabled` / `set_size` / `set_tooltip` / `set_cursor` / `on_click` / `on_mouse_move` / `on_mouse_leave` / `on_mouse_wheel` / `on_focus` / `on_blur` | ✅ 必需 |
| 自绘 | `ui_custom` + `custom_on_{draw,mouse_down,mouse_move,mouse_up,mouse_wheel,key_down,char,layout}` + `custom_set_focusable` / `custom_set_focused` / `custom_get_focused` | ✅ 必需（**文件列表与导航树的落点**） |
| 绘制 | `ui_draw_{fill_rect,rect,fill_rounded_rect,rounded_rect,line,text,text_ex,measure_text,bitmap,push_clip,pop_clip}` + `ui_bitmap_create` / `destroy` / `ui_draw_bitmap_handle` | ✅ 必需 |
| 页面 | `ui_page_{load_file,destroy,reload,root,open_window,prepare_window,set_bool,set_int,set_float,set_text,set_json,get_json,free,last_error,watch,on_widget_mount,on_widget_unmount,menu}` | ✅ 必需 |
| 资源 | `ui_asset_{register_dir,register_blob,reset}` | ✅ 必需 |
| 主题 | `ui_theme_{set_mode,get_mode,bg,content_bg,sidebar_bg,toolbar_bg,accent,text,divider,set_accent_hex}` | ✅ 必需 |
| 菜单 | `ui_menu_{show,close}` / `ui_menu_item_id` / `ui_menu_item_attr` | ✅ 必需 |
| 反馈 | `ui_toast` / `ui_msgbox` | ✅ 必需 |
| 调试/离屏 | `ui_debug_{free,dump_tree,screenshot,screenshot_menu,server_start}` / `ui_offscreen_{begin,end}` | ✅ 必需（CI 验证用） |
| 图片 | `ui_image_{view,set_pixels,load_file,clear,fit,reset,set_zoom,get_zoom,width,height,get_pixels,free_pixels}` | ✅ 必需（M2 预览） |
| 窗口配置 | `ui_window_create` / `ui_page_open_window` 的 `UiWindowConfig*` | ⚠️ 声明但**不用**（见 §2.3） |
| 可选 | `ui_dialog`（头文件有声明、本机 DLL 无导出） | 🔶 可选（`Option`） |
| 仅探测不调用 | 12 个「本机 DLL 独有」符号（`ui_image_gdi_*` / `ui_image_view_gdi` / `ui_image_evict_tile` / `ui_flush_gpu`） | 🔶 只做存在性探测（签名未知） |
| 不在清单 | `ui_core_embed_binary` | ⛔ **不是 C API**（CMake 辅助函数） |
| 暂不绑 | `ui_gh_img_view_*`（12 个）、`ui_ocr_img_view_*`、`ui_menu_set_*`（视觉微调）、`ui_page_load_language_*`、`ui_widget_set_draggable`（HTML5 内部拖放，不是 OLE） | ⛔ 首版不绑 |

> **注意 `ui_widget_set_draggable` 不是 OLE 拖放。** 它是 core-ui 控件**之间**的
> HTML5 语义拖放（载荷是 UTF-8 字符串）。**拖出到资源管理器必须自己走 OLE**
> （`OleInitialize` + `SHCreateDataObject` + `DoDragDrop`），见 §7.3。

---

## 7. 三个呈现层的落点

### 7.1 T2 内嵌预览窗格

#### 候选对比

| 方案 | 能保住 | 丢掉 | 工作量 |
|---|---|---|---|
| **A. core-ui 内自绘预览** | 图片缩放/平移/瓦块、EXIF 面板框架、文本预览框架 | Office/WPS 渲染；HEIC/AVIF/PSD/RAW/JXL（core-ui 无这些解码器）；跨窗口联动 | 中（图片路径已有 `ui_image_view` + `ui_gh_img_view`） |
| **B. 独立顶层原生窗口** | 复用 zsdesk 现成的 `preview_process.rs`(351) / `wps_render_process.rs`(150) / `hover_preview.rs`(374) | 与主窗口的布局联动、焦点跟随；体验上是「另一个窗口」不是「窗格」 | 中高（要自己做停靠/跟随/置顶） |
| **C. 统一交给 GuoheView.exe** | 图片浏览的完整能力（D02 已定） | 一切进程内预览；窗格体验 | 极低（一条 `ShellExecute`） |

#### 决定（D06）：**预览分三层**

> **覆盖路线图 §4 P3**（原 P3 写的是「复用 `preview_process`/`quicklook`/`image_decode` 链路」
> 并把它叫作「GuoheView **强关联**图片预览」）。
> 对话 **D02** 已把「强关联」的形态定为**只外部调 exe** → 原 P3 的
> 「以 GuoheView 强关联为目标的进程内预览链路」作废。
>
> **D02 的收窄（本条为对话内的最新口径，优先于其字面表述）**：
> D02 原文写「不链接 `ghde.dll`、不做进程内解码」——
> 这里的「进程内解码」指的是**基于 GuoheView 的 `ghde.dll` 解码**（闭源、无公开 ABI）。
> **本项目自研的 WIC 解码不在此列**：WIC 是 Windows 公开 API，零闭源依赖，
> 且上游 `window/image_decode.rs` **实测零 UI 依赖**、可直接复用。
> → 结论：**不做 ghde 解码；做 WIC 解码。**

| 层 | 内容 | 落点 | 符号可用性（**已查 `docs/05-COREUI-EXPORTS.txt` 核实**） |
|---|---|---|---|
| **预览窗格本身** | 保留 —— `.uix` 的 `<div id="preview_host" class="preview-host"/>`（用户的 `main.uix` 已有，320px 宽） | `assets/main.uix` | — |
| **图片** | **留 core-ui 内**：Rust 用 WIC 解码 → BGRA **预乘** 像素 → `ui_image_set_pixels` → core-ui 的 ImageView 负责缩放/平移/适应窗口 | `crates/zsdesk-ui/src/preview/image.rs` | ✅ `ui_image_view`（导出表第 259 行）、`ui_image_set_pixels`（第 252 行）、`ui_image_view_plus`（第 261 行）、`ui_image_fit/set_zoom/get_zoom/width/height`、`ui_image_get_pixels/free_pixels`、`ui_bitmap_create/destroy`、`ui_draw_bitmap_handle` —— **全部在导出表内** |
| **文本 / 纯文本** | 留 core-ui 内：Rust 读前 N KB → `.uix` 的 `TextArea` | `crates/zsdesk-ui/src/preview/text.rs` | ✅ |
| **Office / WPS / PDF** | **降级为「用系统程序打开」** | `crates/zsdesk-ui/src/external.rs` | — |
| **GuoheView** | **只做外部 exe 入口**（D02）：优先系统文件关联 `ShellExecuteW(hwnd, L"open", <文件>, …)`，兜底显式指定 `C:\Program Files\GuoheView\GuoheView.exe` | `crates/zsdesk-ui/src/external.rs` | — |
| **HEIC / AVIF / PSD / RAW / JXL** | **不做**（WIC 解不了；GuoheView 有 `plugins/decoder/jxl.dll` 与 `resources/icons/*.ico`）→ 走外部 exe | 同上 | — |

#### ⚠️ 一次已排除的担忧（2026-11-13 核实）

团队提出过一个具体疑问：
> 「`ui_image_view` / `ui_image_set_pixels` 会不会其实只存在于 `ui_image_gdi_*` 一族里？
> 如果是，CI 构建的 DLL（仓库 main）就没有它们 → 方案不成立，必须改降级路径。」

**核实结论：不是。** 三条独立证据：

| 证据 | 结果 |
|---|---|
| `docs/05-COREUI-EXPORTS.txt` 逐名 grep | `ui_image_view`（行 259）、`ui_image_set_pixels`（行 252）、`ui_image_view_plus`（行 261）**都在导出表里** |
| 那 12 个「本机 DLL 独有」符号的真实名单 | `ui_flush_gpu`、`ui_image_evict_tile`、`ui_image_view_gdi`、`ui_image_gdi_{set_file,set_pixels,set_parent,set_zoom,get_zoom,width,height,fit,clear}` —— **不含** `ui_image_view` / `ui_image_set_pixels`（后者是标准 `ui_image_*` 族，共 90 个导出） |
| 头文件 ∩ DLL = 555 | `ui_image_view` / `ui_image_set_pixels` 在 `include/ui_core.h` 有声明**且**在导出表里 → 属交集 → **CI 从仓库 main 编出的 DLL 必然导出**（仓库 main 的 `ui_api.cpp` 有 `UI_API` 定义） |

→ **D06 的三层方案在 CI 产物上成立，无需降级。**
（另：`ui_gh_img_view_*` 54 个、`ui_ocr_img_view_*` 25 个也在导出表里 ——
但它们属于「瓦块金字塔」「OCR」的重型路径，**首版不绑**，见 §6。）

#### 像素格式：BGRA **预乘**（一个容易踩的坑）

`ui_image_set_pixels` / `ui_bitmap_create` 要求 **BGRA 预乘**（头文件原文：
「像素格式同 `ui_draw_bitmap`（BGRA 预乘）」）。
WIC 默认给的是 **32bppBGRA（非预乘）**。

→ **正确做法**：向 WIC 请求 **`GUID_WICPixelFormat32bppPBGRA`**（预乘 BGRA），
WIC 会在 `CopyPixels` 时直接做预乘 → **零额外转换**，像素可直接喂给 core-ui。

> 若用非预乘的 32bppBGRA，半透明边缘会出现**发暗/发黑**的描边（经典 premultiply 错误）。
> **U07**（`ui_bitmap_create` 的像素格式 / stride / 生命周期）仍需真机核对 ——
> 核对方法：造一张 4×4 的半透明测试图 → 灌进去 → 截图看边缘。

**代码处置（与 `spec/migration-matrix.md` 一致）**：

| 文件 | 处置 | 说明 |
|---|---|---|
| `window/image_decode.rs`（272 行，**纯 WIC，零 UI 依赖**） | **直接复用**，**接入首版预览路径** | 实测依赖只有 `super::{fit_dimensions, read_stream, wide, MAX_*}` —— 与 UI 无关。改造点：WIC 像素格式改成 `32bppPBGRA` |
| `window/preview_process.rs` / `quicklook.rs` / `quick_preview.rs` / `hover_preview.rs` | **改造**：宿主部分换成「core-ui 的 ImageView」 | 这四个文件的宿主是**原生窗口 / 子进程 / 自定义窗口消息**（实测：`preview_process` 用 `std::process::Command` + `HWND`；`hover_preview` 依赖 `super::WM_SHELL_NAVIGATED` 等自定义消息）→ core-ui 不可嵌 |
| `window/preview_handlers.rs` / `preview_selection.rs` / `thumbnail_regression.rs` | 改造 / 直接复用 | 见迁移矩阵 |

> **明确不做**（写进 out-of-scope）：
> **基于 `ghde.dll` 的任何解码**（闭源、无公开 ABI）；
> HEIC/AVIF/PSD/RAW/JXL 的进程内解码；Office/WPS/PDF 的进程内渲染；
> 进程内的 EXIF 面板、缩略图流式加载、多图胶片卷。
> 这些是 GuoheView.exe 的强项，**不要在自研侧复刻**。
>
> **降级通道与错误提示**的完整要求见 `spec/milestones.md` §3（M2 ↔ P3）。


### 7.2 T3 Shell 导航树

原生 `INameSpaceTreeControl` 是**子窗口类控件**（上游 `zsdesk-windows/src/shell/namespace_tree.rs`
1029 行就是这么用的：创建为 ZSUI 窗口的子 HWND）。core-ui 不能嵌子 HWND
（实测：`SetParent` 全仓只出现在 core-ui 自己的 GDI 子窗口；`include/ui_core.h:151`
明示 child window 语义不适用）。

| 候选 | 能保住 | 丢掉 | 代价 |
|---|---|---|---|
| **A. 自绘树**（`ui_custom`） | 虚拟文件夹（此电脑/回收站/网络/UNC）、展开收起、固定位置、拖放、Win11 密度感知 | 系统「快速访问」的自动排序与个性化；原生树的细微手感 | 中。`zsdesk-windows` 的 `IShellItem` / `location_from_shell_item` 可复用 |
| **B. 独立顶层窗口装原生树** | 原生树的全部行为 | 与主窗口的布局/焦点/置顶联动；两窗口的 DPI 同步 | 高，且是「假一体化」 |
| **C. 降级为「收藏 + 面包屑」** | 简单可靠 | 「快速访问」「此电脑」「网络」树 | 极低，但 D01 要求全量移植 → 只能作为 A 的兜底 |

#### 决定（D07）：**A，C 作为兜底**

- 树的**数据源**：`SHGetKnownFolderItem(FOLDERID_ComputerFolder / FOLDERID_RecycleBinFolder /
  FOLDERID_NetworkFolder / FOLDERID_Desktop / FOLDERID_Downloads …)` +
  `IShellItem` 枚举 + `shell:::{679F85CB-0220-4080-B29B-5540CC05AAB6}`（快速访问）。
  这些 `zsdesk-windows/src/shell/shell_item.rs`（385 行）**已有**，可复用。
- 树的**渲染**：`ui_custom` + `ui_draw_text` + `ui_draw_bitmap_handle`（展开箭头用 SVG 位图或自绘三角）。
- 树的**状态**（展开集合）：`.uix` JS 的视图局部状态（§4.2）。
- **兜底**：若自绘树在 M1 期限内做不完 → C（收藏 + 面包屑），并在 M2 补。

### 7.3 T4 自绘文件列表：逐项落点

> 落点 crate：`crates/zsdesk-ui/src/filelist/`；Shell 侧能力：`crates/zsdesk-windows/src/shell/`；
> 权威状态：`zsdesk-app`。

| 能力 | 用到的 Win32 / COM 接口 | 落点模块 | 备注 |
|---|---|---|---|
| **虚拟列表滚动 / 复用** | 无（纯自绘）：维护 `first_visible_index` + 固定 `row_height`；`ui_draw_push_clip` / `ui_draw_pop_clip` 裁剪；只画可见行 | `crates/zsdesk-ui/src/filelist/viewport.rs` + `draw.rs` | 行高按 DPI 缩放；超大目录只画 ±2 行余量 |
| **图标获取** | `IShellItemImageFactory::GetImage(SIZE{16,32}, SIIGF_ICON \| SIIGF_BIGGERSIZEOK)` → `HBITMAP` → `GetObjectW` + `GetDIBits` → BGRA | `crates/zsdesk-windows/src/shell/icons.rs`（**已有 467 行，可改造**） | 必须转成 BGRA 预乘（core-ui 的位图格式） |
| **缩略图获取** | `IShellItemImageFactory::GetImage(..., SIIGF_BIGGERSIZEOK)` 或 `ITinyThumbProvider`（`IThumbnailProvider`） | `crates/zsdesk-windows/src/shell/thumb.rs`（新增） | **异步**：工作线程取，回 UI 线程 `ui_bitmap_create` |
| **图标 / 缩略图缓存** | LRU：`HashMap<identity_key, Arc<IconBitmap>>` + 容量上限（按字节数，不是条目数） | `crates/zsdesk-ui/src/filelist/icon_cache.rs` | 上限走 §9 性能预算 |
| **详细信息列** | `IShellItem::GetDisplayInfo(SHGDUI_NAME / SHGDUI_SIZE / SHGDUI_DATETIME / SHGDUI_TYPE)`；或 `IShellPropertyStore`（`System.Size` / `System.DateModified` / `System.FileType`） | `crates/zsdesk-windows/src/shell/properties.rs`（新增） | 属性名从 `zsdesk-core` 的 `ViewPropertyKey` 取 |
| **排序** | **Rust 侧排**（不用 `ICompareListItems`）：`zsdesk-core/src/model/view.rs::natural_cmp` 已实现与资源管理器一致的本地化排序 | `crates/zsdesk-app/src/sort.rs` | 保持「文件夹在前 + 自然序」 |
| **多选 / 框选 / 反选** | 纯 Rust：`HashSet<identity_key>`；框选 = 行区间 ∩ 矩形；Ctrl 反选 | `crates/zsdesk-app/src/selection.rs` | `identity_key` 来自 `FileEntry`（卷 + 文件 ID，路径回退） |
| **拖放（进来）** | `ui_window_on_drop(win, cb, ud)` —— core-ui 内部已用 `DragQueryFileW(hDrop, 0xFFFFFFFF, …)` **遍历全部**并逐个回调（实测 `src/ui/ui_window.cpp:3815`） | `crates/zsdesk-app/src/window/drop.rs` | 回调给的是 `const wchar_t*`（UTF-16） |
| **拖放（出去）** | `OleInitialize` + `SHCreateDataObject`（`PIDL`/`IID_IShellItemArray`）+ 自写 `IDropSource` + `DoDragDrop` | `crates/zsdesk-windows/src/ole/drop.rs`（**已有 115 行，可改造**） | core-ui **明确支持**宿主在鼠标回调里起 `DoDragDrop`（见 §4.5 Q6） |
| **内联重命名** | `ui_custom_on_key_down`（F2）→ 自绘编辑框（`ui_draw_text` + 光标 + `ui_custom_on_char` 收字符）→ 提交走 `IFileOperation` 的 rename 或 `IShellItem::Rename` | `crates/zsdesk-ui/src/filelist/rename.rs` | 编辑框是**自绘**，不是 `.uix` 的 `TextInput`（`.uix` 控件无法嵌进自绘 widget 的坐标系） |
| **右键菜单** | **声明式**：`.uix` 里 `<menu id="ctx">` + `<menuitem v-for="cmd in ctxCommands">`，Rust 用 `ui_page_set_json(page, "ctxCommands", …)` 推菜单项；点击经 `ui_window_on_menu` + `ui_menu_item_id` 路由 | `crates/zsdesk-ui/src/filelist/context_menu.rs` + `ui/main.uix` | core-ui build 75+ **删除了 imperative 菜单构造**（`ui_menu_add_item` 已不存在），只能声明式 |
| **Shell 虚拟位置** | `SHGetKnownFolderItem(FOLDERID_RecycleBinFolder / ComputerFolder / NetworkFolder)`；UNC 走 `SHParseDisplayName` | `crates/zsdesk-windows/src/shell/shell_item.rs`（**已有 385 行**） | `ShellLocation`（`zsdesk-core`）已能表达 |
| **视图记忆** | 按文件夹路径 hash → 存储 | `crates/zsdesk-app/src/window/view_memory.rs`（**已有 250 行**） | 存储层 `crates/zsdesk-app/src/storage.rs` |
| **列宽记忆** | 同上 | `crates/zsdesk-app/src/storage.rs` | |
| **排序记忆** | 同上 | `crates/zsdesk-app/src/storage.rs` | |

#### 右键菜单的具体流程（因为 declarative 菜单有坑）

1. Rust 在 `ui_custom_on_mouse_down` 里检测 `btn == 2`（右键），
   由 widget-local 坐标算出**命中的行号**，存进 `pending_ctx_row`。
2. Rust 把该行的上下文（选中项、是否多选、是否可粘贴、是否在回收站…）
   组装成 JSON，`ui_page_set_json(page, "ctxCommands", json)`。
3. `.uix` 的 `<menu trigger="#file_list" event="rclick">` 弹出 ——
   core-ui 的 rclick 是 **deepest-match**（build 107-108），子 widget 的菜单
   不会被祖先抢走。
4. 用户点菜单项 → core-ui 调 `ui_window_on_menu(win, cb, ud)` →
   `ui_menu_item_id(item)` 拿到命令 id → Rust 结合 `pending_ctx_row` 执行。

> **风险点**：步骤 2 的 `ui_page_set_json` 与步骤 3 的菜单弹出之间的时序。
> `.uix` 的反应式菜单在弹出时读最新值（`docs/uix-guide.md` §17「右键弹出时永远读最新值」），
> 所以只要 `set_json` 在弹出**之前**完成即可。
> **M1 必须实测这条时序**（见 `spec/milestones.md` M1 验收项 V1.6）。

---

## 8. `.uix` ↔ Rust 的通道契约

| 方向 | 通道 | 载荷 | 频率 | 约束 |
|---|---|---|---|---|
| Rust → `.uix` | `ui_page_set_json(page, name, json)` | **小**：标签列表、树节点、状态栏文本、菜单项、工具栏可用性 | 状态变更时 | 载荷必须小；**文件列表禁止走这里** |
| Rust → `.uix`（单值） | `ui_page_set_text/int/bool/float` | 单值 | 状态变更时 | 标量优先 |
| `.uix` 内的显示位 | **模板里的 `{{ }}` 插值**（**不是** `text="…"` 属性 —— 见 §4.3.3 约束 2） | 文本 | 响应式 | 插值节点的初值由 binding 负责（`FirstPlainText` 对 `Interpolation` 返回 `""`） |
| Rust → `.uix` | `ui_page_set_text/int/bool/float` | 单值 | 状态变更时 | 标量优先 |
| `.uix` → Rust | `ui_page_watch(page, name, cb, ud)` | JSON（**仅回调期间有效**） | JS 改了响应式变量时 | 只 watch **小**变量；禁止 watch 文件列表 |
| `.uix` → Rust | `ui_widget_on_click(w, cb, ud)` 等 | 控件句柄 | 交互时 | **优先于 `ui_page_watch`**：直连 Rust，不过 JS |
| `.uix` → Rust | `ui_page_on_widget_mount(page, id, cb, ud)` | 控件句柄 | mount 时 | 用来给 `<custom id="file_list"/>` 挂 draw / 鼠标回调 |
| Rust → core-ui | `ui_window_invalidate(win)` | — | 需要重绘时 | draw 回调里**禁止**调用 |

**名称约定**：`.uix` 的响应式变量名用 `camelCase`，与 Rust 字段一一对应。
Rust 侧用一个 `ViewModel` 结构统一投影，**不散落各处的 `set_*` 调用**。

---

## 9. 性能预算（首版目标值）

> 数值目标均为 **[估算]**，M1 结束后用实测替换。

| 指标 | 目标 | 测量方式 |
|---|---|---|
| 目录切换 → 首屏可见行 | ≤ 120 ms（10k 条目录） | `ui_debug_screenshot` 时间戳 |
| 滚动帧耗时（P95） | ≤ 8 ms | draw 回调内计时 |
| 图标缓存命中率 | ≥ 95%（回到已访问目录） | 缓存统计 |
| 图标 / 缩略图缓存内存上限 | ≤ 64 MB | 缓存字节计数 |
| 单次 `ui_page_set_json` 载荷 | ≤ 64 KB | 断言 |
| core-ui.dll 体积 | 约 4.1 MB（上游实测值） | 打包产物 |

---

## 10. out-of-scope（本契约不覆盖）

1. **不实现** HTML5 语义拖放（`ui_widget_set_draggable`）—— 那不是 OLE，拖不出程序。
2. **不实现** core-ui 的 `<gh-img-view>` 瓦块金字塔喂数（`ui_gh_img_view_begin/set_tile`）——
   首版预览用 `ui_image_view` + 一次性像素。
3. **不实现** OCR（`ui_ocr_img_view_*`）。
4. **不实现** 自绘列表的**列头拖拽排序**以外的列宽拖拽（M1 只做列宽记忆，不做拖拽调宽）。
5. **不实现** 虚拟列表的**分组**（`ViewGrouping` 在 `zsdesk-core` 里有，但 UI 侧首版只按排序）。
6. **不承诺** 本机 GuoheView 目录里的 `core-ui.dll` 与 CI 构建的版本一致 ——
   那是**兜底**，不是分发渠道。

---

## 11. 端到端验证

```bash
# 1. CI 构建（唯一可编译验证的通道）
cargo test --workspace --release --locked
cargo build --release --p zsdesk-app

# 2. 启动自检：版本 + 能力 + revision 戳记
#    期望：日志出现 "core-ui 1.9.0.345 | <exe目录>\core-ui.dll | revision 一致"
#    能力行必须包含 image_gdi=false（CI 构建的是仓库 main，没有那 12 个符号）

# 3. 离屏渲染冒烟（不需要桌面会话）
#    ui_offscreen_begin/end 画一个矩形 → 断言像素非全零

# 4. 交互闭环（CI runner 无交互桌面 → 走 core-ui 的调试管道）
#    ui_debug_server_start(win, "zsdesk_coreui_ci")
#    PowerShell: 写入 \\.\pipe\zsdesk_coreui_ci → "click <id>" / "screenshot <path>"
#    断言：截图非空，且文件列表区域有内容
```

**关键错误流**：把 `core-ui.dll` 移走再启动 → 必须在启动时**大声失败**并给出
「找不到 core-ui.dll，已尝试：…」的完整候选列表，而不是静默降级或白屏。

---

## 12. 知识库落地记录

| 引用 | 改了 Spec 里的哪个值 |
|---|---|
| `10-standards/spec-as-contract.md` §2「点名文件/模块」 | §6 的绑定清单从「按需再定」改成**逐符号的三档清单**（必需/可选/暂不绑），并点名到 `spec/rust-ffi-skeleton/src/lib/api.rs` |
| `10-standards/spec-as-contract.md` §2「明确不做（out-of-scope）」 | 新增 §10「out-of-scope」节，明确列出 6 项不实现的内容 |
| `10-standards/spec-as-contract.md` §2「版本锚定」 | §5.4 把版本下限钉死为 **build ≥ 345**，并写明依据是 pinned revision `1840d689…` |
| `10-standards/runtime-defensive.md` 第二部分「校验放在边界」 | §3.2 把内存所有权写成三分类硬契约 —— **跨 FFI 边界就是跨信任域，必须校验/分类**；同进程内的 `ui_*` 调用不加防御性检查 |
| `10-standards/runtime-defensive.md` 第三部分「错误归一化」 | §5.3 能力表把「12 个符号缺失」归一化成 **6 个具名能力**，每项有确定的降级路径，而不是让调用方自己判断几种情况 |
| `10-standards/mechanical-gates.md` §3「失败即停」 | §2.3 把「结构体布局漂移」的补偿从「建议核对」改成 **CI 门禁 S8**；§5.5 的 revision 不一致在运行时只警告，但**在 CI 打包时判失败** |
| `20-quality/security-baseline.md` §二「路径穿越 / 命令注入」 | D02 的 `ShellExecute` 拉起 GuoheView 必须**参数数组化 + 路径规范化**，不接受拼接命令行（见 `DECISIONS.md` D06 与 `spec/milestones.md` §3） |

---

## 13. 关联文档

- `spec/crate-layout.md` —— crate 划分与依赖方向
- `spec/migration-matrix.md` —— 文件级复用矩阵
- `spec/ci-contract.md` —— 构建 / 打包 / 版本校验
- `spec/milestones.md` —— M0–M3 与验收标准
- `DECISIONS.md` —— D06–D11
