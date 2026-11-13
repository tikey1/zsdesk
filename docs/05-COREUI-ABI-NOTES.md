# Core UI ABI 核对笔记（本机实测）

> 对应 `docs/10-FORK-ROADMAP-COREUI.md` §3 **C2（Core UI ABI 闸门 K-01~K-18 尚未真机核对）**。
> 本文件把**可离线核实**的部分全部核实掉，把**必须真机调用**的部分明确留白，不编造签名。

- 探针对象：本机 `C:\Program Files\GuoheView\core-ui.dll`
- 文件大小：4,782,536 字节
- PE Machine：`0x8664`（**x86-64**）
- 上游版本声明：`ghboke/core-ui` @ main 的 `VERSION` / `version.json` = `1.9.0.345` → **与本机 DLL 一致**
  - ⚠️ 用户路线图 §3 C2 提到「具备 Core UI 1.9.0.346 的真机」；本机实测为 **1.9.0.345**，差一个 build 号，**真机核对前不要假定 346 有额外导出**。
- 方法：自写 PE 导出表解析器（仅用 Python 标准库，无外部依赖），逐符号名比对；与上游 `include/ui_core.h` 的声明做集合差集。
- 完整符号清单：**`docs/05-COREUI-EXPORTS.txt`**（2006 个导出，其中 567 个 `ui_*`）

---

## 1. VERIFIED —— 符号存在性与结构事实（已离线核实）

| # | 结论 | 证据 |
|---|---|---|
| V01 | DLL 为 x86-64 | PE Machine = `0x8664` |
| V02 | 导出总数 **2006**；其中 **`ui_*` 纯 C API 567 个** | 导出表逐名统计 |
| V03 | 公开头文件声明 **557** 个 `ui_*`；与 DLL 导出**交集 555** | 头文件正则 vs 导出表集合差 |
| V04 | **DLL 比头文件多 12 个导出**：`ui_flush_gpu`、`ui_image_evict_tile`、`ui_image_view_gdi`、`ui_image_gdi_{set_file,set_pixels,set_parent,set_zoom,get_zoom,width,height,fit,clear}` | 导出表；且 `src/ui/ui_api.cpp:117`、`:1707` 确有 `UI_API` 定义 → **上游头文件漏同步声明，不是本机版本领先** |
| V05 | 头文件比 DLL **多 2 个**：`ui_core_embed_binary`（编译期资源内嵌辅助，非运行期 API）、`ui_dialog` | 头文件有声明但 DLL 无此导出 → `ui_dialog` 属于**不可用**，需降级 |
| V06 | 生命周期与版本查询存在：`ui_init_with_theme`、`ui_shutdown`、`ui_quit`、`ui_run`、`ui_core_version`、`ui_core_version_build`、`ui_core_version_string` | 导出表 |
| V07 | 页面/窗口族 **79** 个：`ui_page_load_file`、`ui_page_load_string`、`ui_page_open_window`、`ui_page_destroy`、`ui_page_set_int`、`ui_page_set_json`、`ui_page_get_json`、`ui_page_set_locale`、`ui_page_bind_combobox_items`、`ui_page_set_text_list`、`ui_window_*` | 导出表 |
| V08 | 绘制族 **37** 个，含 `ui_draw_*` 与 `ui_custom_*`（`ui_custom_on_draw` 在内） | 导出表 |
| V09 | 图像族 **118** 个，含 `ui_image_*`、`ui_bitmap_create`、`ui_bitmap_destroy`、`ui_ocr_img_view_*` | 导出表 |
| V10 | 调试族 **71** 个（`ui_debug_*`），含 `ui_debug_free`、`ui_debug_tree`、`ui_debug_screenshot` | 导出表 |
| V11 | 主题/字体族 **33** 个，含 `ui_theme_*`（accent/bg/content_bg/divider、CJK 字体查询） | 导出表 |
| V12 | **core-ui 不提供原生 HWND 子控件嵌入**：全仓 `src/` 中 `SetParent` 仅出现在 `src/ui/image_view_gdi.cpp`（挂自己的 GDI 子窗口）；`include/ui_core.h:151` 明写 child window 语义不适用 | 源码级 grep |
| V13 | **core-ui 无虚拟列表/表格控件**：控件全集为 Label/Button/CheckBox/RadioButton/Toggle/Slider/ProgressBar/TextInput/TextArea/ComboBox/NavItem/Separator/TitleBar/IconButton/TabControl/SVG/Image/ImageView/Dialog/Toast/ContextMenu/CustomWidget；`ScrollView` 非虚拟化 | `docs/controls.md` + `docs/layout.md` |
| V14 | **`.uix` 的 `<script>` 不能直接调 C**：上游 demo 原文注释 `methods{} can't call C directly`，宿主只能轮询 `.uix` state | `demo/ui_demo.uix` 注释 |
| V15 | **字符串编码 UTF-8 / UTF-16 混用且逐函数不同**：头文件内 `const char*` 72 处、`const wchar_t*` 77 处 | 头文件统计 |
| V16 | **内存归属两类**：部分返回内部指针（不可 free，如 `ui_text_input_get_text`）、部分需显式释放（如 `ui_debug_toast_info` 需 `ui_debug_free`） | `docs/c-api.md` |
| V17 | **线程模型**：工作线程访问 widget 前必须 marshal 到 UI 线程 | `docs/c-api.md` |
| V18 | `.uix` 是 Vue 3 SFC（`<window>` + `<script>` + `<style>` + `<template>`），脚本由内嵌 QuickJS-NG 求值；i18n 用 `@key` 占位 + `.lang` 键值对（UTF-8 `key=value`） | `docs/uix-guide.md` + `demo/ui_demo.uix` |

---

## 2. UNVERIFIED —— 必须真机调用才能定的语义（**不得编造签名**）

> 以下每一项都**无法**从导出名反推。落地时按「先探测、再绑定、再最小验证」的顺序做。

| # | 待核对项 | 最小验证方法 |
|---|---|---|
| U01 | `ui_init_with_theme` 的确切参数个数与 `UiTheme` 取值 | 用 `ui_core_version()` 单独探针先验证可用性，再最小参数调用 |
| U02 | `ui_page_load_file` / `ui_page_load_string` 的路径编码（UTF-8 vs UTF-16）与失败返回值 | 传 UTF-8 路径（含中文名）加载 `.uix`，观察是否成功 |
| U03 | `ui_page_open_window` 的配置结构体布局与所有权 | 先用默认参数开窗，再逐字段试 |
| U04 | `ui_page_set_int` / `set_json` / `get_json` 的类型映射与 JSON 返回的释放方 | `set_json` → `get_json` 往返，比对返回指针是否需 `ui_debug_free` |
| U05 | `ui_custom_on_draw` 回调签名、绘制 API 的可用子集、回调线程 | 画一条线/一个矩形，验证调用后是否真的出现 |
| U06 | `ui_widget_on_mouse_*` / `on_key` 等回调的注册与注销语义、回调内可否再调 `ui_*` | 注册鼠标回调 → 触发 → 观察 |
| U07 | `ui_bitmap_create` 的像素格式（BGRA/RGBA）、stride 与生命周期 | 建一张 4×4 位图 → 绘制 → 校验颜色 |
| U08 | `ui_image_gdi_set_pixels` 的 GDI 语义（HBITMAP/内存 DC）与所有权 | 从 Shell 缩略图位图灌入 → 显示 |
| U09 | `ui_theme_*` 的颜色格式（`UiColor` 四个 float 的取值域与色彩空间） | 设置 accent → 截图比对 |
| U10 | `ui_debug_free` 是否是 `ui_debug_*` 返回值的唯一释放方 | 试 free 与不 free，观察是否崩溃（**只在隔离探针程序里做**） |
| U11 | `ui_scroll_set_content` 的语义（是否仅设内容区尺寸） | 只读探针 |
| U12 | `ui_ocr_img_view_*` 族是否可直接复用（OCR 能力在本机 GuoheView 内可用） | 只读探针 |

---

## 3. 对 C2 闸门的结论

| 子项 | 状态 |
|---|---|
| 符号存在性（2006 导出 / 567 纯 C API / 头文件缺口） | ✅ **已离线核实**，基线见 `docs/05-COREUI-EXPORTS.txt` |
| 结构事实（无原生控件嵌入 / 无虚拟列表 / `.uix` 不能调 C / 编码混用 / 内存两类 / 线程约束） | ✅ **已离线核实** |
| 参数语义与返回值（U01~U12） | ❌ **仍未核对**，需真机最小探针逐项验 |

→ **C2 从「完全阻塞」降级为「部分阻塞」**：可以开始写**绑定骨架与加载器**，但**不得**凭猜测写具体调用；每接入一组 API，必须先过对应的 U 项最小验证。
→ **落地顺序建议**：先做 `ui_core_version*` → `ui_init_with_theme` → `ui_page_load_string` → `ui_page_open_window` → `ui_run` 这条最小闭环（对应 U01/U02/U03），跑通再接 `ui_custom_on_draw`（U05）。
→ **签名声明的来源优先级**：① 上游 `include/ui_core.h`（**权威签名，但可能漏声明**） ② 本文件 §2 的最小验证结果（**语义权威**） ③ **任何情况下都不得凭导出名反推参数**。

---

## 4. 给实现侧的三条硬规则

1. **头文件不是 DLL 的完备契约** —— 绑定清单以 `docs/05-COREUI-EXPORTS.txt` 为准；头文件只提供签名。
2. **可选 API 必须先探测符号再调用** —— 运行时 `LoadLibrary` + `GetProcAddress` 探测；缺失就走降级（例：`ui_dialog` 在本机 DLL 缺失）。
3. **所有 core-ui 调用收敛到 UI 线程** —— 工作线程只做 IO 与解码，结果回传后由 UI 线程提交。

---

## 5. 与 GuoheView 的关系（备忘）

本机 `core-ui.dll` 与 `GuoheView.exe` 同源：`GuoheView.exe` 内嵌了 core-ui 的 `.uix` 模板源码（`<window title="GuoheView"`、`<TitleBar>`、`<custom id="sc_open">`、`v-model`、`@key` 占位），**是 core-ui 在真实产品中的现成参考样例**。
GuoheView 的 `ghde.dll` 是另一套 **323 个扁平 C ABI**（`ghde_*`）的图像解码引擎 —— 按用户已拍板的 D02，**本项目不链接它**，仅通过 `ShellExecute` 外部调用 `GuoheView.exe`。
