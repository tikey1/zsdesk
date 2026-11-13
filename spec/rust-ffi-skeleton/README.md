# `zc-ffi-coreui` 骨架

core-ui C API 的**运行时动态加载**绑定层。这是全仓库唯一允许 `unsafe` 的 crate。

## 它做什么 / 不做什么

| 做 | 不做 |
|---|---|
| `LoadLibrary` 加载 `core-ui.dll`，建立必需/可选两张符号表 | 不画任何东西（自绘在 `zc-ui`） |
| 版本协商（`ui_core_version*`）+ 按符号存在性判能力 | 不持有业务状态（状态在 `zc-app`） |
| UI 线程收敛（`invoke_sync` 封送 + 线程断言） | 不认识 `.uix` 语义（页面装配在 `zc-ui`） |
| UTF-8 ↔ UTF-16 转换（唯一转换点） | **不反向依赖任何内部 crate** |

## 文件

| 文件 | 内容 |
|---|---|
| `Cargo.toml` | 只有 `libloading` 一个依赖。**刻意不引入 bindgen**（见 `spec/ffi-core-ui.md` §2） |
| `build.rs` | 把 `packaging/core-ui-revision.txt` 的 revision 固化进产物（`ZSCOREUI_CORE_UI_REVISION`） |
| `src/lib.rs` | crate 根；字符串编解码；启动自检报告 |
| `src/lib/api.rs` | 手写 `extern "C"` 声明（首版核心子集）+ 编码/所有权/线程三分类注释 |
| `src/lib/loader.rs` | `LoadLibrary` + 必需/可选符号表（宏生成） |
| `src/lib/version.rs` | 版本协商 + 能力表 + revision 戳记 |
| `src/lib/ui_thread.rs` | `UiThread`（独占）/ `UiHandle`（跨线程）/ `invoke_sync` |

## 放进真实仓库

```
zsdesk-coreui/
├── crates/
│   └── zc-ffi-coreui/     ← 本目录整体搬过来
└── packaging/
    └── core-ui-revision.txt   ← build.rs 会读它
```

`build.rs` 的查找顺序：
1. 环境变量 `ZSCOREUI_REVISION`；
2. `<workspace>/packaging/core-ui-revision.txt`；
3. `<crate>/packaging/core-ui-revision.txt`；
4. 都没有 → `cargo:warning` + 标记 `unpinned`。

> 严格性不在 build.rs，在 CI 门禁：缺 revision / DLL 与 revision 不一致 → CI 判失败。
> 见 `spec/ci-contract.md` §5 步骤 S7。

## 本地无法编译验证

本机（实测）无 cargo / rustc / cmake / MSVC —— **一切编译验证只能靠 CI**。
第一件事：把这个 crate 放进仓库，让 `cargo test --workspace --release --locked` 在 CI 上跑一遍。

预期会踩的编译坑（提前列好，省一轮 CI）：

1. `libloading` 的 `get` 需要**带结尾 NUL** 的符号名 —— 已在宏里用 `concat!($sym, "\0")` 处理。
2. `Api` 里存的是 `unsafe extern "C" fn(...)`，`Symbol<T>` 的 `Deref` 目标是 `T`，
   `*sym` 得到函数指针副本 —— 需要 `T: Copy`（函数指针是 Copy）。
3. `libloading::Symbol<T>` 的生命周期绑定在 `Library` 上 —— 我们立刻 `*sym` 拷出来，
   所以 `Api` 不带生命周期参数。
4. `UiTask` 里的 `Box<dyn FnOnce(&CoreUi) + Send>` 需要 `CoreUi` 能在闭包签名里出现
   （只读引用，没问题）。

## 已知的「首版不做」

- `UiWindowConfig` 只声明不使用（MVP 走 `.uix` 的 `<window>` 标签 + `ui_page_open_window(page, NULL)`，
  完全不需要传结构体）。要用之前必须核对 pinned revision 的 `ui_core.h` 实际布局。
- 没有任何业务封装（不画列表、不读页面状态）。那些在 `zc-ui`。
