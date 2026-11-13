//! core-ui C API 的手写 `extern "C"` 声明 —— **首版核心子集**。
//!
//! 本文件是「Rust 与 core-ui 之间唯一的 ABI 描述」。改这里等于改契约，
//! 必须同步更新 `spec/ffi-core-ui.md` §3 的函数表。
//!
//! ## 三条编码约定（每条声明都必须遵守）
//!
//! 1. **字符串编码逐函数不同**——core-ui 头文件里 `const char*` 出现 72 处、
//!    `const wchar_t*` 出现 77 处，**同一个库两种编码混用**。本文件里：
//!    - `*_utf8` 后缀 = `const char*`，内容是 **UTF-8**（资源路径、页面 id、
//!      JSON、菜单项 id）；
//!    - `*_utf16` 后缀 = `const wchar_t*`，内容是 **UTF-16LE**（界面文本、
//!      窗口标题、拖放路径、绘制文本）。
//!    上游 Rust API **一律收 UTF-8 `&str`**，转换只发生在本层的包装函数里。
//!
//! 2. **内存所有权三分类**——见每条声明上方的 `【借用】` / `【转移】` / `【拷贝】`：
//!    - 【借用】返回内部指针：调用方**不得** free，且在下一次修改该对象后失效；
//!    - 【转移】返回调用方拥有的内存：必须用**对应的** core-ui 释放函数回收
//!      （`ui_debug_free` / `ui_page_free` / `ui_image_free_pixels`），
//!      **绝不能**用 Rust 的 `Vec`/`String` 去 free；
//!    - 【拷贝】库内部复制了一份：调用方保留源缓冲区的所有权，可立即释放。
//!
//! 3. **线程约束**——除 `ui_core_version*` 外，**所有** `ui_*` 调用必须在
//!    core-ui 的 UI 线程上执行。工作线程的调用必须经 `ui_window_invoke_sync`
//!    封送（见 `ui_thread.rs`）。违反这一条是未定义行为，不是「慢一点」。
//!
//! ## 符号清单的权威来源
//!
//! **不是本文件，也不是头文件，而是 `docs/05-COREUI-EXPORTS.txt`**（本机 DLL 的
//! 2006 个导出，其中 **567** 个 `ui_*` 纯 C API）。
//! 头文件只提供**签名**，而且**漏声明**了 12 个本机 DLL 实际导出的符号
//! （见 `docs/05-COREUI-ABI-NOTES.md` V04）。
//!
//! ## 未核实项（`docs/05-COREUI-ABI-NOTES.md` §2 的 U01~U12）
//!
//! 本文件的签名**全部来自上游头文件**，属于「未核实」状态：
//! **U01** `ui_init_with_theme` 参数 · **U02** `ui_page_load_file` 路径编码与失败返回 ·
//! **U03** `ui_page_open_window` 的配置结构体布局 · **U04** `set_json`/`get_json` 的类型映射与释放方 ·
//! **U05** `ui_custom_on_draw` 回调签名/绘制子集/回调线程 · **U06** `ui_widget_on_mouse_*` 注册注销语义 ·
//! **U07** `ui_bitmap_create` 的像素格式与生命周期 · **U08** GDI 位图语义 ·
//! **U09** `UiColor` 取值域 · **U10** `ui_debug_free` 的适用范围 ·
//! **U11** `ui_scroll_set_content` 语义 · **U12** `ui_ocr_img_view_*` 可复用性。
//!
//! **不得凭导出名反推参数**（ABI-NOTES 硬规则 3）。
//! **每接入一组 API，必须先过对应的 U 项最小验证**（见 `spec/milestones.md` §1.5）。
//!
//! ## 为什么手写而不是 bindgen
//!
//! 见 `spec/ffi-core-ui.md` §2。一句话：bindgen 需要编译期存在 core-ui 头文件，
//! 会让 `cargo test --workspace` 依赖「先构建 core-ui」，直接破坏上游 CI 契约
//! （`cargo test --workspace --release --locked` 必须能独立跑通）。
//! 手写的代价是结构体布局会漂移 —— 用「vendored 头文件 + CI 差异门禁」补偿。

#![allow(non_camel_case_types)]

use core::ffi::c_void;

// ---------------------------------------------------------------------------
// 句柄与基础类型（core-ui 全部句柄都是 uint64_t，无 C++ 类型外泄）
// ---------------------------------------------------------------------------

/// 不透明控件句柄。
pub type UiWidget = u64;
/// 不透明窗口句柄。
pub type UiWindow = u64;
/// 不透明页面句柄（一个 `.uix` 文件）。
pub type UiPage = u64;
/// 不透明菜单句柄。
pub type UiMenu = u64;
/// 不透明位图句柄（`ui_bitmap_create` 产出）。
pub type UiBitmap = u64;
/// 绘制上下文，`void*`。
pub type UiDrawCtx = *mut c_void;
/// 被点击的菜单项，仅在回调期间有效。
pub type UiMenuItem = *const c_void;

/// 矩形。字段顺序与 `ui_core.h` 的 `UiRect` 一致。
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct UiRect {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl UiRect {
    pub fn width(&self) -> f32 {
        self.right - self.left
    }
    pub fn height(&self) -> f32 {
        self.bottom - self.top
    }
}

/// 颜色，RGBA 各通道 0.0–1.0。
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl UiColor {
    pub const TRANSPARENT: UiColor = UiColor { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        UiColor { r, g, b, a: 1.0 }
    }
}

/// 主题模式。**注意取值**：DARK=0、LIGHT=1。
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiThemeMode {
    Dark = 0,
    Light = 1,
}

/// 采样方式（`ui_draw_bitmap_ex` 用）。
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiImageSampling {
    Nearest = 0,
    Linear = 1,
    Cubic = 2,
}

// ---------------------------------------------------------------------------
// 回调签名
//
// 一律 `extern "C"`：x86-64 Windows 上 C 与 system ABI 一致（D00-7 只做 x64）。
// ---------------------------------------------------------------------------

pub type UiWindowCloseCallback = unsafe extern "C" fn(UiWindow, *mut c_void);
pub type UiWindowCloseRequestCallback = unsafe extern "C" fn(UiWindow, *mut c_void) -> i32;
pub type UiWindowResizeCallback = unsafe extern "C" fn(UiWindow, i32, i32, *mut c_void);
/// 路径是 UTF-16LE（`const wchar_t*`）。【借用】
pub type UiWindowDropCallback = unsafe extern "C" fn(UiWindow, *const u16, *mut c_void);
pub type UiWindowKeyCallback = unsafe extern "C" fn(UiWindow, i32, *mut c_void);
pub type UiRightClickCallback = unsafe extern "C" fn(UiWindow, f32, f32, *mut c_void);
pub type UiWindowMouseButtonCallback = unsafe extern "C" fn(UiWindow, i32, f32, f32, *mut c_void);
pub type UiMenuCallback = unsafe extern "C" fn(UiWindow, UiMenuItem, *mut c_void);
pub type UiClickCallback = unsafe extern "C" fn(UiWidget, *mut c_void);
pub type UiWidgetMouseMoveCallback = unsafe extern "C" fn(UiWidget, f32, f32, *mut c_void);
pub type UiWidgetMouseLeaveCallback = unsafe extern "C" fn(UiWidget, *mut c_void);
pub type UiWidgetWheelCallback = unsafe extern "C" fn(UiWidget, f32, f32, f32, *mut c_void);
pub type UiWidgetFocusCallback = unsafe extern "C" fn(UiWidget, *mut c_void);
pub type UiCustomDrawCallback = unsafe extern "C" fn(UiWidget, UiDrawCtx, UiRect, *mut c_void);
pub type UiCustomMouseCallback = unsafe extern "C" fn(UiWidget, f32, f32, i32, *mut c_void) -> i32;
pub type UiCustomWheelCallback = unsafe extern "C" fn(UiWidget, f32, f32, f32, *mut c_void) -> i32;
pub type UiCustomKeyCallback = unsafe extern "C" fn(UiWidget, i32, *mut c_void) -> i32;
pub type UiCustomCharCallback = unsafe extern "C" fn(UiWidget, i32, *mut c_void) -> i32;
pub type UiCustomLayoutCallback = unsafe extern "C" fn(UiWidget, UiRect, *mut c_void);
pub type UiWidgetLifecycleCallback = unsafe extern "C" fn(UiPage, UiWidget, *mut c_void);
/// `json` 是 UTF-8 JSON，**仅回调期间有效**。【借用】
pub type UiPageWatchCallback = unsafe extern "C" fn(UiPage, *const u8, *const u8, *mut c_void);
pub type UiInvokeCallback = unsafe extern "C" fn(*mut c_void);

/// `UiWindowConfig`。
///
/// **首版不使用**（Tier 2）。主窗口与设置窗口都走 `.uix` 的 `<window>` 标签 +
/// `ui_page_open_window(page, NULL)`，从而**完全不需要传结构体**，规避手写
/// `#[repr(C)]` 布局漂移的风险。若将来必须用（例如要设 `start_maximized`），
/// 必须先核对 pinned revision 的 `ui_core.h` 里本结构的实际布局，并把核对结果
/// 写进 `spec/ffi-core-ui.md` §7。
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UiWindowConfig {
    pub title: *const u16,
    pub width: i32,
    pub height: i32,
    pub system_frame: i32,
    pub resizable: i32,
    pub accept_files: i32,
    pub x: i32,
    pub y: i32,
    pub tool_window: i32,
    pub skip_animation: i32,
    pub icon_pixels: *const c_void,
    pub icon_width: i32,
    pub icon_height: i32,
    pub owner: UiWindow,
    pub start_maximized: i32,
    pub copy_swap_chain: i32,
    pub layered_canvas: i32,
}

// ---------------------------------------------------------------------------
// 版本与生命周期
// ---------------------------------------------------------------------------

extern "C" {
    /// 任意线程、任意时刻可调（头文件明示「无需 ui_init」）。
    pub fn ui_core_version(major: *mut i32, minor: *mut i32, patch: *mut i32);
    pub fn ui_core_version_build() -> i32;
    /// 【借用】返回静态字符串，进程生命周期内有效。
    pub fn ui_core_version_string() -> *const u8;

    pub fn ui_init() -> i32;
    pub fn ui_init_with_theme(mode: UiThemeMode) -> i32;
    pub fn ui_shutdown();
    pub fn ui_run() -> i32;
    pub fn ui_quit(exit_code: i32);
}

// ---------------------------------------------------------------------------
// 窗口
// ---------------------------------------------------------------------------

extern "C" {
    pub fn ui_window_create(config: *const UiWindowConfig) -> UiWindow;
    pub fn ui_window_destroy(win: UiWindow);
    pub fn ui_window_show(win: UiWindow);
    pub fn ui_window_show_immediate(win: UiWindow);
    pub fn ui_window_set_root(win: UiWindow, root: UiWidget);
    /// 标题是 UTF-16LE。
    pub fn ui_window_set_title(win: UiWindow, title: *const u16);
    pub fn ui_window_set_resizable(win: UiWindow, resizable: i32);
    pub fn ui_window_invalidate(win: UiWindow);
    pub fn ui_window_relayout(win: UiWindow);
    /// 【借用】返回 HWND（`void*`），进程内有效。
    pub fn ui_window_hwnd(win: UiWindow) -> *mut c_void;
    pub fn ui_window_dpi(win: UiWindow) -> i32;
    pub fn ui_window_dpi_scale(win: UiWindow) -> f32;
    pub fn ui_window_set_min_size(win: UiWindow, w_dip: i32, h_dip: i32);
    pub fn ui_window_set_rect(win: UiWindow, x: i32, y: i32, w: i32, h: i32);
    pub fn ui_window_get_rect_screen(win: UiWindow, x: *mut i32, y: *mut i32, w: *mut i32, h: *mut i32) -> i32;
    /// 把 `fn` 封送到 UI 线程同步执行；返回前 `fn` 一定已跑完。
    pub fn ui_window_invoke_sync(win: UiWindow, func: UiInvokeCallback, ud: *mut c_void);

    pub fn ui_window_on_close(win: UiWindow, cb: UiWindowCloseCallback, ud: *mut c_void);
    pub fn ui_window_on_close_request(win: UiWindow, cb: UiWindowCloseRequestCallback, ud: *mut c_void);
    pub fn ui_window_on_resize(win: UiWindow, cb: UiWindowResizeCallback, ud: *mut c_void);
    pub fn ui_window_on_drop(win: UiWindow, cb: UiWindowDropCallback, ud: *mut c_void);
    pub fn ui_window_on_key(win: UiWindow, cb: UiWindowKeyCallback, ud: *mut c_void);
    pub fn ui_window_on_right_click(win: UiWindow, cb: UiRightClickCallback, ud: *mut c_void);
    pub fn ui_window_on_menu(win: UiWindow, cb: UiMenuCallback, ud: *mut c_void);
    pub fn ui_window_on_mouse_button(win: UiWindow, cb: UiWindowMouseButtonCallback, ud: *mut c_void);
}

// ---------------------------------------------------------------------------
// 控件
// ---------------------------------------------------------------------------

extern "C" {
    pub fn ui_widget_find_by_id(root: UiWidget, id: *const u8) -> UiWidget;
    pub fn ui_widget_set_id(w: UiWidget, id: *const u8);
    pub fn ui_widget_add_child(parent: UiWidget, child: UiWidget);
    pub fn ui_widget_remove_child(parent: UiWidget, child: UiWidget);
    pub fn ui_widget_destroy(w: UiWidget);
    pub fn ui_widget_get_rect(w: UiWidget) -> UiRect;
    pub fn ui_widget_set_visible(w: UiWidget, visible: i32);
    pub fn ui_widget_set_enabled(w: UiWidget, enabled: i32);
    pub fn ui_widget_get_visible(w: UiWidget) -> i32;
    pub fn ui_widget_get_enabled(w: UiWidget) -> i32;
    pub fn ui_widget_set_size(w: UiWidget, width: f32, height: f32);
    pub fn ui_widget_set_tooltip(w: UiWidget, text: *const u16);
    pub fn ui_widget_set_cursor(w: UiWidget, cursor: i32);

    pub fn ui_widget_on_click(w: UiWidget, cb: UiClickCallback, ud: *mut c_void);
    pub fn ui_widget_on_mouse_move(w: UiWidget, cb: UiWidgetMouseMoveCallback, ud: *mut c_void);
    pub fn ui_widget_on_mouse_leave(w: UiWidget, cb: UiWidgetMouseLeaveCallback, ud: *mut c_void);
    pub fn ui_widget_on_mouse_wheel(w: UiWidget, cb: UiWidgetWheelCallback, ud: *mut c_void);
    pub fn ui_widget_on_focus(w: UiWidget, cb: UiWidgetFocusCallback, ud: *mut c_void);
    pub fn ui_widget_on_blur(w: UiWidget, cb: UiWidgetFocusCallback, ud: *mut c_void);
}

// ---------------------------------------------------------------------------
// 自绘控件（文件列表 / 导航树 的落点）
// ---------------------------------------------------------------------------

extern "C" {
    pub fn ui_custom() -> UiWidget;
    pub fn ui_custom_on_draw(w: UiWidget, cb: UiCustomDrawCallback, ud: *mut c_void);
    pub fn ui_custom_on_mouse_down(w: UiWidget, cb: UiCustomMouseCallback, ud: *mut c_void);
    pub fn ui_custom_on_mouse_move(w: UiWidget, cb: UiCustomMouseCallback, ud: *mut c_void);
    pub fn ui_custom_on_mouse_up(w: UiWidget, cb: UiCustomMouseCallback, ud: *mut c_void);
    pub fn ui_custom_on_mouse_wheel(w: UiWidget, cb: UiCustomWheelCallback, ud: *mut c_void);
    pub fn ui_custom_on_key_down(w: UiWidget, cb: UiCustomKeyCallback, ud: *mut c_void);
    pub fn ui_custom_on_char(w: UiWidget, cb: UiCustomCharCallback, ud: *mut c_void);
    pub fn ui_custom_on_layout(w: UiWidget, cb: UiCustomLayoutCallback, ud: *mut c_void);
    pub fn ui_custom_set_focusable(w: UiWidget, focusable: i32);
    pub fn ui_custom_set_focused(w: UiWidget, focused: i32);
    pub fn ui_custom_get_focused(w: UiWidget) -> i32;
}

// ---------------------------------------------------------------------------
// 绘制（仅在 `ui_custom_on_draw` 回调内有效）
// ---------------------------------------------------------------------------

extern "C" {
    pub fn ui_draw_fill_rect(ctx: UiDrawCtx, rect: UiRect, color: UiColor);
    pub fn ui_draw_rect(ctx: UiDrawCtx, rect: UiRect, color: UiColor, width: f32);
    pub fn ui_draw_fill_rounded_rect(ctx: UiDrawCtx, rect: UiRect, rx: f32, ry: f32, color: UiColor);
    pub fn ui_draw_rounded_rect(ctx: UiDrawCtx, rect: UiRect, rx: f32, ry: f32, color: UiColor, width: f32);
    pub fn ui_draw_line(ctx: UiDrawCtx, x1: f32, y1: f32, x2: f32, y2: f32, color: UiColor, width: f32);
    /// 文本是 UTF-16LE。
    pub fn ui_draw_text(ctx: UiDrawCtx, text: *const u16, rect: UiRect, color: UiColor, font_size: f32);
    pub fn ui_draw_text_ex(
        ctx: UiDrawCtx,
        text: *const u16,
        rect: UiRect,
        color: UiColor,
        font_size: f32,
        align: i32,
        bold: i32,
    );
    pub fn ui_draw_measure_text(ctx: UiDrawCtx, text: *const u16, font_size: f32) -> f32;
    /// 【拷贝】像素为 BGRA 预乘；库内部复制一份，调用返回后 `pixels` 可释放。
    pub fn ui_draw_bitmap(ctx: UiDrawCtx, pixels: *const u8, width: i32, height: i32, stride: i32, dest: UiRect);
    pub fn ui_draw_push_clip(ctx: UiDrawCtx, rect: UiRect);
    pub fn ui_draw_pop_clip(ctx: UiDrawCtx);

    /// 【拷贝】库保存自己的拷贝，调用返回后 `pixels` 可释放。
    pub fn ui_bitmap_create(pixels: *const u8, width: i32, height: i32, stride: i32) -> UiBitmap;
    pub fn ui_bitmap_destroy(bmp: UiBitmap);
    pub fn ui_draw_bitmap_handle(
        ctx: UiDrawCtx,
        bmp: UiBitmap,
        dest: UiRect,
        sampling: i32,
        opacity: f32,
    );
}

// ---------------------------------------------------------------------------
// 页面（`.uix`）
// ---------------------------------------------------------------------------

extern "C" {
    /// 路径是 UTF-16LE。
    pub fn ui_page_load_file(path: *const u16) -> UiPage;
    /// 源码是 **UTF-8**。★ 本项目实际用这个 —— Rust 要在加载前把 `:root { … }`
    /// 拼进 `.uix` 源码（库没有运行时替换 CSS 变量的 C API），
    /// 且深浅色切换必须用**新的**源码重新加载（`ui_page_reload` 会重放旧源码）。
    pub fn ui_page_load_string(html_source: *const u8) -> UiPage;
    pub fn ui_page_destroy(p: UiPage);
    pub fn ui_page_reload(p: UiPage) -> i32;
    pub fn ui_page_root(p: UiPage) -> UiWidget;
    pub fn ui_page_open_window(p: UiPage, override_defaults: *const UiWindowConfig) -> UiWindow;
    pub fn ui_page_prepare_window(p: UiPage, override_defaults: *const UiWindowConfig) -> UiWindow;

    pub fn ui_page_set_bool(p: UiPage, name: *const u8, value: i32);
    pub fn ui_page_set_int(p: UiPage, name: *const u8, value: i32);
    pub fn ui_page_set_float(p: UiPage, name: *const u8, value: f64);
    /// 文本是 UTF-16LE。
    pub fn ui_page_set_text(p: UiPage, name: *const u8, value: *const u16);
    /// JSON 是 UTF-8；解析失败返回 -1。
    pub fn ui_page_set_json(p: UiPage, name: *const u8, utf8_json: *const u8) -> i32;
    /// 【转移】返回 malloc 出来的 UTF-8 JSON，**必须**用 `ui_page_free` 释放。
    pub fn ui_page_get_json(p: UiPage, name: *const u8) -> *mut u8;
    pub fn ui_page_free(ptr: *mut c_void);

    /// 【借用】返回最后一次解析错误的 UTF-8 文本。
    pub fn ui_page_last_error(p: UiPage) -> *const u8;

    pub fn ui_page_watch(p: UiPage, name: *const u8, cb: UiPageWatchCallback, ud: *mut c_void);
    pub fn ui_page_on_widget_mount(p: UiPage, widget_id: *const u8, cb: UiWidgetLifecycleCallback, ud: *mut c_void);
    pub fn ui_page_on_widget_unmount(p: UiPage, widget_id: *const u8, cb: UiWidgetLifecycleCallback, ud: *mut c_void);

    pub fn ui_page_menu(p: UiPage, name: *const u8) -> UiMenu;
}

// ---------------------------------------------------------------------------
// 资源 / 主题 / 菜单 / 反馈
// ---------------------------------------------------------------------------

extern "C" {
    /// 目录是 UTF-8。
    pub fn ui_asset_register_dir(dir_utf8: *const u8);
    /// 【拷贝】库保存自己的拷贝。
    pub fn ui_asset_register_blob(name: *const u8, bytes: *const c_void, size: usize);
    pub fn ui_asset_reset();

    pub fn ui_theme_set_mode(mode: UiThemeMode);
    pub fn ui_theme_get_mode() -> UiThemeMode;
    pub fn ui_theme_bg() -> UiColor;
    pub fn ui_theme_content_bg() -> UiColor;
    pub fn ui_theme_sidebar_bg() -> UiColor;
    pub fn ui_theme_toolbar_bg() -> UiColor;
    pub fn ui_theme_accent() -> UiColor;
    pub fn ui_theme_text() -> UiColor;
    pub fn ui_theme_divider() -> UiColor;
    /// accent 覆盖串是 UTF-8；解析失败返回 -1。
    pub fn ui_theme_set_accent_hex(hex: *const u8) -> i32;

    pub fn ui_menu_show(win: UiWindow, menu: UiMenu, x: f32, y: f32);
    pub fn ui_menu_close(win: UiWindow);
    /// 【借用】菜单项 id，UTF-8，仅在回调期间有效。
    pub fn ui_menu_item_id(item: UiMenuItem) -> *const u8;
    /// 【借用】菜单项属性，UTF-8，仅在回调期间有效。
    pub fn ui_menu_item_attr(item: UiMenuItem, name: *const u8) -> *const u8;

    /// 文本是 UTF-16LE。
    pub fn ui_toast(win: UiWindow, text: *const u16, duration_ms: i32);
    /// 模态；按钮文案是 UTF-16LE 数组。
    pub fn ui_msgbox(
        win: UiWindow,
        title: *const u16,
        message: *const u16,
        buttons: *const *const u16,
        button_count: i32,
        default_idx: i32,
        cancel_idx: i32,
        icon: i32,
    );
}

// ---------------------------------------------------------------------------
// 调试 / 离屏（CI 验证用）
// ---------------------------------------------------------------------------
extern "C" {
    /// 【转移】必须用 `ui_debug_free` 释放。
    pub fn ui_debug_free(ptr: *mut c_void);
    /// 【转移】返回 malloc 出来的 UTF-8 JSON，必须用 `ui_debug_free` 释放。
    pub fn ui_debug_dump_tree(win: UiWindow) -> *mut u8;
    /// 路径是 UTF-16LE；成功返回 0。
    pub fn ui_debug_screenshot(win: UiWindow, out_path: *const u16) -> i32;
    pub fn ui_debug_screenshot_menu(win: UiWindow, out_path: *const u16) -> i32;
    /// 管道名是 UTF-8。
    pub fn ui_debug_server_start(win: UiWindow, pipe_name: *const u8) -> i32;

    /// 离屏渲染：返回一个可喂给任意 `ui_draw_*` 的上下文。
    pub fn ui_offscreen_begin(width: i32, height: i32) -> UiDrawCtx;
    /// 结束绘制并回读 BGRA 像素；成功返回 1。
    pub fn ui_offscreen_end(ctx: UiDrawCtx, out_bgra: *mut u8, stride: i32) -> i32;
}

// ---------------------------------------------------------------------------
// 图片控件（M2 预览用）
// ---------------------------------------------------------------------------

extern "C" {
    pub fn ui_image_view() -> UiWidget;
    /// 【拷贝】像素为 BGRA 预乘；库复制一份。
    pub fn ui_image_set_pixels(w: UiWidget, win: UiWindow, pixels: *const c_void, width: i32, height: i32, stride: i32);
    /// 路径是 UTF-16LE。
    pub fn ui_image_load_file(w: UiWidget, win: UiWindow, path: *const u16);
    pub fn ui_image_clear(w: UiWidget);
    pub fn ui_image_fit(w: UiWidget);
    pub fn ui_image_reset(w: UiWidget);
    pub fn ui_image_set_zoom(w: UiWidget, zoom: f32);
    pub fn ui_image_get_zoom(w: UiWidget) -> f32;
    pub fn ui_image_width(w: UiWidget) -> i32;
    pub fn ui_image_height(w: UiWidget) -> i32;
    /// 【转移】回读像素，必须用 `ui_image_free_pixels` 释放。
    pub fn ui_image_get_pixels(w: UiWidget, win: UiWindow, out_pixels: *mut *mut c_void, out_w: *mut i32, out_h: *mut i32) -> i32;
    pub fn ui_image_free_pixels(ptr: *mut c_void);
}

// ---------------------------------------------------------------------------
// 可选符号（`Option` 语义，缺失即降级）
//
// 声明放在 `loader.rs` 的 `define_optional_api!` 宏里（那里直接生成
// `Option<fn>` 字段）。此处只做**契约记录**：
//
// | 符号 | 头文件 | 本机 DLL | 处置 |
// |---|---|---|---|
// | `ui_dialog` | 有声明 `UI_API UiWidget ui_dialog(void);` | **无导出** | 可选；缺失时用 `ui_msgbox` 兜底 |
// | `ui_image_gdi_{set_file,set_pixels,set_parent,set_zoom,get_zoom,width,height,fit,clear}` | **无声明** | 有导出 | **不绑定、不调用**（签名未知，U08 待核对） |
// | `ui_image_view_gdi` / `ui_image_evict_tile` / `ui_flush_gpu` | **无声明** | 有导出 | **不绑定、不调用**（签名未知） |
//
// `ui_core_embed_binary` **不是 C API** —— 它是 CMake 辅助函数
// （`cmake/embed_binary.cmake`），用于生成内嵌字节的 .h 文件。
// 从来不会出现在任何 DLL 的导出表里，**不在绑定清单里**。
// 发布态的资源注册走 `ui_asset_register_blob`。
// ---------------------------------------------------------------------------
