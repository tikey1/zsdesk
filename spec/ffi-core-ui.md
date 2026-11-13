# core-ui FFI 与呈现层契约

> 契约编号：`ZC-FFI-001`
> 状态：已定（随 D05–D11 生效）
> 上游依据：`ghboke/core-ui` @ `1840d689b51a32d392dbf06b8d594aef96696a80`（1.9.0 build 345，2026-11-01）
> 绑定层代码：`spec/rust-ffi-skeleton/`

---

## 1. 三句话结论

1. **状态留 Rust**（D05）。`.uix` 的 QuickJS 只做**视图局部状态**；文件列表**根本不在 `.uix` 里**，
   它是 `<custom id="file_list"/>`，由 Rust 在 `ui_custom_on_draw` 回调里直接画。
2. **运行时 `LoadLibrary` 动态加载**（D09），不链接 `core-ui.lib`。核心符号缺失 = 加载失败；
   可选符号缺失 = 降级。
3. **预览分三层**（D06）：图片预览留在 core-ui 内（`<gh-img-view>` / `ui_image_view`），
   Office/WPS 预览降级为「用系统程序打开」，GuoheView 只做外部 exe 入口。

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

**上层 API 一律收 UTF-8 `&str`**，转换点唯一：`zc-ffi-coreui::to_utf16` / `from_utf16`。
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

- 文件列表 = `.uix` 里的 `<custom id="file_list"/>`，
  Rust 用 `ui_page_on_widget_mount(page, "file_list", cb, ud)` 拿到句柄，
  再挂 `ui_custom_on_draw` / `ui_custom_on_mouse_*` / `ui_custom_on_key_down` /
  `ui_custom_on_char`。
- Rust 在 draw 回调里直接画行（`ui_draw_text` + `ui_draw_bitmap_handle`），
  **数据不进 JS**。
- `.uix` 只承载**周边 chrome**：标题栏、地址栏、工具栏、标签条、侧栏树外壳、
  状态栏、预览窗格框架。

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
│  · 转成 UiCommand（枚举）投递到 zc-app 的状态机                        │
│  · 绝不在这层直接改 UI                                                │
└───────────────┬───────────────────────────────────────────────────────┘
                ↓
┌───────────────────────────────────────────────────────────────────────┐
│ Rust 状态机（zc-app，唯一真相源）                                     │
│  · 改 ExplorerSession / 选择集 / 排序 / 视图记忆                       │
│  · diff 出「哪些视图状态变了」                                        │
└───────────────┬───────────────────────────────────────────────────────┘
                ↓
        ┌───────┴────────┬─────────────────────────┐
        ↓                ↓                         ↓
┌──────────────┐ ┌──────────────────┐  ┌──────────────────────────────┐
│ 小载荷投影    │ │ 文件列表模型      │  │ 预览模型                    │
│ ui_page_set_*│ │ zc-ui 的 filelist │  │ ui_image_set_pixels 等      │
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

### 5.1 实测差异（`research/facts.md` §7.1）

| 项 | 数量 |
|---|---|
| 仓库 `include/ui_core.h` 声明 `ui_*` | 557 |
| 本机 `core-ui.dll` 导出 `ui_*` | 567 |
| 双方都有 | 555 |
| 仓库有、本机无 | **2**：`ui_core_embed_binary`、`ui_dialog` |
| 本机有、仓库无 | **12**：`ui_image_gdi_*`（9）、`ui_image_view_gdi`、`ui_image_evict_tile`、`ui_flush_gpu` |

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
| `image_gdi` | `ui_image_gdi_*`（9 个） | 用 `ui_image_view` + `ui_image_set_pixels` 路径 |
| `image_view_gdi` | `ui_image_view_gdi` | 用 `ui_image_view` |
| `image_evict_tile` | `ui_image_evict_tile` | 大图瓦块不主动淘汰（内存上限靠 `image_clear`） |
| `flush_gpu` | `ui_flush_gpu` | 退出前不强制刷 GPU |
| `embed_binary` | `ui_core_embed_binary` | 改用 `ui_asset_register_dir`（开发态）或 `ui_asset_register_blob`（发布态读文件） |
| `dialog` | `ui_dialog` | 用 `ui_msgbox` 模态对话框 |

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

---

## 6. 首版绑定的子集清单

**原则**：首版只绑「M0 + M1 一定用得到」的符号；能通过 `.uix` 声明式做到的，不绑 imperative API。

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
| 可选 | `ui_image_gdi_*`（9）/ `ui_image_view_gdi` / `ui_image_evict_tile` / `ui_flush_gpu` / `ui_core_embed_binary` / `ui_dialog` | 🔶 可选（`Option`） |
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

#### 决定（D06）：**分层落地，不是一刀切**

| 内容 | 落点 | 理由 |
|---|---|---|
| **图片预览** | **A**：core-ui 内，`<gh-img-view>` 或 `ui_image_view` + Rust WIC 解码后 `ui_image_set_pixels` | zsdesk 的 `window/image_decode.rs` 已经用 **WIC** 解码（实测：zsdesk **完全不用** `ghde.dll`），代码可复用；core-ui 内嵌预览才有「悬停预览 / 空格预览」 |
| **文本 / 纯文本预览** | A：`.uix` 的 `<label>` / `TextArea` 承载，Rust 读文件喂文本 | 成本低 |
| **Office / WPS 文档预览** | **降级为「用系统程序打开」** | 进程内渲染 WPS/Office 需要宿主应用或打印转 PDF，core-ui 无对应能力；zsdesk 的 `wps_render_process.rs` 依赖外部 WPS 进程，收益不确定 |
| **HEIC/AVIF/PSD/RAW/JXL** | **C**：交给 GuoheView.exe | core-ui 与 WIC 都解不了；GuoheView 有 `plugins/decoder/jxl.dll` 与全套 `resources/icons/*.ico` |
| **GuoheView 作为「查看器」** | **C**（D02 已定） | `ShellExecute` 拉起 `C:\Program Files\GuoheView\GuoheView.exe <路径>` |

> **明确不做**（D02 代价，写进 out-of-scope）：
> 进程内的缩略图流式加载、EXIF 面板、缩放动画、翻页、多图胶片卷。
> 这些是 GuoheView.exe 的强项，不要在自研侧复刻。

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

> 落点 crate：`zc-ui/src/filelist/`；Shell 侧能力：`zc-windows/src/shell/`；
> 权威状态：`zc-app`。

| 能力 | 用到的 Win32 / COM 接口 | 落点模块 | 备注 |
|---|---|---|---|
| **虚拟列表滚动 / 复用** | 无（纯自绘）：维护 `first_visible_index` + 固定 `row_height`；`ui_draw_push_clip` / `ui_draw_pop_clip` 裁剪；只画可见行 | `zc-ui/src/filelist/viewport.rs` + `draw.rs` | 行高按 DPI 缩放；超大目录只画 ±2 行余量 |
| **图标获取** | `IShellItemImageFactory::GetImage(SIZE{16,32}, SIIGF_ICON \| SIIGF_BIGGERSIZEOK)` → `HBITMAP` → `GetObjectW` + `GetDIBits` → BGRA | `zc-windows/src/shell/icons.rs`（**已有 467 行，可改造**） | 必须转成 BGRA 预乘（core-ui 的位图格式） |
| **缩略图获取** | `IShellItemImageFactory::GetImage(..., SIIGF_BIGGERSIZEOK)` 或 `ITinyThumbProvider`（`IThumbnailProvider`） | `zc-windows/src/shell/thumb.rs`（新增） | **异步**：工作线程取，回 UI 线程 `ui_bitmap_create` |
| **图标 / 缩略图缓存** | LRU：`HashMap<identity_key, Arc<IconBitmap>>` + 容量上限（按字节数，不是条目数） | `zc-ui/src/filelist/icon_cache.rs` | 上限走 §9 性能预算 |
| **详细信息列** | `IShellItem::GetDisplayInfo(SHGDUI_NAME / SHGDUI_SIZE / SHGDUI_DATETIME / SHGDUI_TYPE)`；或 `IShellPropertyStore`（`System.Size` / `System.DateModified` / `System.FileType`） | `zc-windows/src/shell/properties.rs`（新增） | 属性名从 `zsdesk-core` 的 `ViewPropertyKey` 取 |
| **排序** | **Rust 侧排**（不用 `ICompareListItems`）：`zsdesk-core/src/model/view.rs::natural_cmp` 已实现与资源管理器一致的本地化排序 | `zc-app/src/sort.rs` | 保持「文件夹在前 + 自然序」 |
| **多选 / 框选 / 反选** | 纯 Rust：`HashSet<identity_key>`；框选 = 行区间 ∩ 矩形；Ctrl 反选 | `zc-app/src/selection.rs` | `identity_key` 来自 `FileEntry`（卷 + 文件 ID，路径回退） |
| **拖放（进来）** | `ui_window_on_drop(win, cb, ud)` —— core-ui 内部已用 `DragQueryFileW(hDrop, 0xFFFFFFFF, …)` **遍历全部**并逐个回调（实测 `src/ui/ui_window.cpp:3815`） | `zc-app/src/window/drop.rs` | 回调给的是 `const wchar_t*`（UTF-16） |
| **拖放（出去）** | `OleInitialize` + `SHCreateDataObject`（`PIDL`/`IID_IShellItemArray`）+ 自写 `IDropSource` + `DoDragDrop` | `zc-windows/src/ole/drop.rs`（**已有 115 行，可改造**） | core-ui **明确支持**宿主在鼠标回调里起 `DoDragDrop`（见 §4.5 Q6） |
| **内联重命名** | `ui_custom_on_key_down`（F2）→ 自绘编辑框（`ui_draw_text` + 光标 + `ui_custom_on_char` 收字符）→ 提交走 `IFileOperation` 的 rename 或 `IShellItem::Rename` | `zc-ui/src/filelist/rename.rs` | 编辑框是**自绘**，不是 `.uix` 的 `TextInput`（`.uix` 控件无法嵌进自绘 widget 的坐标系） |
| **右键菜单** | **声明式**：`.uix` 里 `<menu id="ctx">` + `<menuitem v-for="cmd in ctxCommands">`，Rust 用 `ui_page_set_json(page, "ctxCommands", …)` 推菜单项；点击经 `ui_window_on_menu` + `ui_menu_item_id` 路由 | `zc-ui/src/filelist/context_menu.rs` + `ui/main.uix` | core-ui build 75+ **删除了 imperative 菜单构造**（`ui_menu_add_item` 已不存在），只能声明式 |
| **Shell 虚拟位置** | `SHGetKnownFolderItem(FOLDERID_RecycleBinFolder / ComputerFolder / NetworkFolder)`；UNC 走 `SHParseDisplayName` | `zc-windows/src/shell/shell_item.rs`（**已有 385 行**） | `ShellLocation`（`zsdesk-core`）已能表达 |
| **视图记忆** | 按文件夹路径 hash → 存储 | `zc-app/src/window/view_memory.rs`（**已有 250 行**） | 存储层 `zc-app/src/storage.rs` |
| **列宽记忆** | 同上 | `zc-app/src/storage.rs` | |
| **排序记忆** | 同上 | `zc-app/src/storage.rs` | |

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
cargo build --release --p zc-app

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
| `20-quality/security-baseline.md` §二「路径穿越 / 命令注入」 | D02 的 `ShellExecute` 拉起 GuoheView 必须**参数数组化 + 路径规范化**，不接受拼接命令行（见 `DECISIONS.md` D11） |

---

## 13. 关联文档

- `spec/crate-layout.md` —— crate 划分与依赖方向
- `spec/migration-matrix.md` —— 文件级复用矩阵
- `spec/ci-contract.md` —— 构建 / 打包 / 版本校验
- `spec/milestones.md` —— M0–M3 与验收标准
- `DECISIONS.md` —— D05–D11
