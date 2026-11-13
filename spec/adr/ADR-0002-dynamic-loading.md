# ADR-0002：core-ui 运行时动态加载（不链接 `core-ui.lib`）

- 状态：已接受
- 日期：2026-11-13
- 决策者：架构师（高见远），依据 D03 / D09

## 背景

两个事实把「链接期还是运行期」变成了必须现在回答的问题：

1. **本机已经有一份同版本的 `core-ui.dll`**：
   `C:\Program Files\GuoheView\core-ui.dll`（4,782,536 字节，PE machine `0x8664`，
   版本 `1.9.0.345`，与仓库 `main` 的 `VERSION` 一致）。
   链接期会产生**静态 import 表**（`core-ui.dll`），Windows 加载器按
   「exe 目录 → 系统目录 → PATH」搜索。若 exe 目录**没有** DLL，就会命中 PATH 里
   **用户自己装的**那份 —— 而那份**不受我们控制**：用户升级 GuoheView 就可能换掉 API 面。
2. **两份 DLL 的 API 面不同**（实测，`docs/05-COREUI-EXPORTS.txt`）：
   DLL 总导出 2006，其中 `ui_*` **567**；头文件只声明 557，交集 555；
   DLL 独有 12 个（`ui_image_gdi_*` 等，头文件**漏声明**），头文件独有 2 个
   （`ui_dialog` 有声明但 DLL 无导出；`ui_core_embed_binary` **不是 C API**，是 CMake 辅助函数）。

另外两条约束：

- **D00-5 / D00-7**：CRT 统一 `/MD`，只做 x86-64；
- **上游 CI 契约**：`cargo test --workspace --release --locked` 必须是第一条命令。

## 决策

> **我们将采用运行时 `LoadLibrary` 动态加载：Rust 侧用 `libloading` 在运行时
> 解析符号；不链接 `core-ui.lib`；显式限定 DLL 搜索顺序；必需符号缺失即加载失败，
> 可选符号缺失即降级。**

具体：

| 项 | 决定 |
|---|---|
| 搜索顺序 | exe 同目录 → `exe/core-ui/` → `exe/../` → `C:\Program Files\GuoheView\core-ui.dll`（**兜底**，命中就打警告 + 比对 revision 戳记） |
| 打包 | `core-ui.dll` **必须与 exe 同目录**（门禁 S12 强制） |
| 符号表 | 必需 ≈ 150 个（缺失 → `LoadError::Missing`，**大声失败**）；可选 `ui_dialog`（缺失 → `ui_msgbox` 兜底）；12 个「DLL 独有」符号**只探测存在性，绝不绑定、绝不调用**（签名未知，U08 待核对） |
| 绑定方式 | **手写 `extern "C"`**（不用 bindgen，见下） |
| CRT | core-ui 用 `UI_CORE_MSVC_STATIC_CRT=OFF`（默认 `/MD`）；门禁 S7 用 `dumpbin /dependents` 判定必须出现 `VCRUNTIME140.dll` + `MSVCP140.dll` |
| 内存 | **谁分配谁释放** —— core-ui 的指针只用 `ui_debug_free` / `ui_page_free` / `ui_image_free_pixels` 回收 |

## 后果

- **变容易的**：
  - **保住上游 CI 契约**：`cargo test --workspace --release --locked` 在**没有 core-ui.dll、
    没有 core-ui 源码**的环境里跑通；core-ui 的 C++ 构建失败不会连累 Rust 侧单测；
  - **显式搜索路径**，不会被 PATH 里的别的副本劫持；
  - API 面差异从「加载即失败」变成「按符号在/不在优雅降级」；
  - 绕开链接期的 `LNK2038 RuntimeLibrary mismatch`。
- **变困难的**：
  - **失去编译期签名检查** → 用 CI 门禁 **S8**（逐符号比对 567 个导出）补偿；
  - 每次调用是间接调用（可忽略）；
  - 必须自己写约 150 条声明 + 维护两张符号表；
  - `UiWindowConfig` 结构体布局会漂移 → **首版完全不用它**（走 `.uix` 的 `<window>` 标签）。
- **引出的新决策**：
  - D08（绑定策略与线程收敛）；
  - D10（主题 token —— 因为没有运行时换 CSS 变量的 API）。

## 备选方案

| 方案 | 为什么没选 |
|---|---|
| **链接期链 `core-ui.lib`** | ① 静态 import 表会命中 PATH 里用户自己装的那份 DLL，那份不受我们控制；② API 面差异变成加载即失败；③ 需要 `core-ui.lib` 先存在 → `cargo test` 被 core-ui 的 C++ 构建绑架，破坏上游 CI 契约；④ CRT 不一致直接 `LNK2038` |
| **bindgen 生成绑定** | 需要编译期存在 core-ui 头文件（同样绑架 `cargo test`），还要 libclang（CI 要装 LLVM，本机没有）；且生成物随 revision 漂移，无法逐符号标注编码与所有权 |
| **C++ shim** | core-ui 官方明示「Rust / Go / Python / C# 可直接绑定，无需 C++ shim」；shim 会多一层 CRT 边界与一份要维护的 ABI |

> **不可变**：已接受后不得修改本文件；改变主意请新建 ADR 并标注「取代」。
