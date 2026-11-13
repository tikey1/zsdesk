//! 运行时动态加载 core-ui.dll，并建立「必需符号 / 可选符号」两张表。
//!
//! ## 为什么是运行时加载而不是链接期链接 `core-ui.lib`
//!
//! 见 `spec/ffi-core-ui.md` §3。三条硬理由：
//!
//! 1. **CI 契约**：上游 zsdesk 的 CI 第一步就是 `cargo test --workspace --release
//!    --locked`。如果 Rust 侧链接 `core-ui.lib`，那么「先构建 core-ui」就成了
//!    `cargo test` 的前置条件 —— CI 的步骤顺序被 core-ui 的 C++ 构建绑架，
//!    一旦 core-ui 构建失败，连 Rust 侧的单元测试都跑不了。动态加载让
//!    Rust 侧完全不需要 core-ui 存在就能编译与测试。
//! 2. **API 面不一致**（实测）：本机 `C:\Program Files\GuoheView\core-ui.dll`
//!    导出 567 个 `ui_*`，仓库 main 只有 557 个；本机多 12 个
//!    （`ui_image_gdi_*` 9 个 + `ui_image_view_gdi` + `ui_image_evict_tile` +
//!    `ui_flush_gpu`），仓库多 2 个（`ui_core_embed_binary` + `ui_dialog`）。
//!    链接期会把这套差异变成「加载即失败」，运行时可以按符号缺失优雅降级。
//! 3. **CRT 冲突**：链接期会直接撞 `LNK2038 RuntimeLibrary mismatch`；
//!    运行时加载不存在这道坎，但代价是**内存所有权必须严格**（见 `api.rs`）。

use std::ffi::CString;
use std::path::{Path, PathBuf};

use crate::api::*;
use crate::version::{Capabilities, CoreUiVersion};

/// 符号缺失。加载期遇到「必需符号缺失」直接返回它 —— 大声失败，不静默降级。
#[derive(Debug)]
pub struct MissingSymbol {
    pub name: &'static str,
}

impl std::fmt::Display for MissingSymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "core-ui.dll 缺少必需导出符号：{}", self.name)
    }
}

impl std::error::Error for MissingSymbol {}

/// 加载失败的总错误类型。
#[derive(Debug)]
pub enum LoadError {
    /// DLL 文件不存在。
    NotFound(Vec<PathBuf>),
    /// DLL 存在但不是合法的 PE / 依赖缺失 / 位数不符。
    Load(std::io::Error),
    /// 缺少必需符号。
    Missing(MissingSymbol),
    /// 版本低于绑定层支持的最低版本。
    TooOld {
        found: CoreUiVersion,
        min_build: u32,
    },
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::NotFound(paths) => {
                write!(f, "找不到 core-ui.dll，已尝试：")?;
                for p in paths {
                    write!(f, "\n  - {}", p.display())?;
                }
                write!(f, "\n提示：CI 产物应把 core-ui.dll 放在 exe 同目录。")
            }
            LoadError::Load(e) => write!(f, "加载 core-ui.dll 失败：{e}"),
            LoadError::Missing(m) => write!(f, "{m}"),
            LoadError::TooOld { found, min_build } => write!(
                f,
                "core-ui.dll 版本过旧：实测 {}.{}.{}.{}（build {}），绑定层要求 build >= {}",
                found.major, found.minor, found.patch, found.build, found.build, min_build
            ),
        }
    }
}

impl std::error::Error for LoadError {}

/// 必需符号表：缺一个就加载失败。
macro_rules! define_required_api {
    ($( $field:ident : $ret:ty = $sym:literal ),* $(,)?) => {
        /// 必需符号表。语义：**缺失即加载失败**。
        #[derive(Debug, Clone, Copy)]
        pub struct Api {
            $( pub $field: $ret, )*
        }

        impl Api {
            unsafe fn bind(lib: &libloading::Library) -> Result<Self, MissingSymbol> {
                Ok(Api {
                    $( $field: {
                        // 符号名带结尾 NUL：libloading 的 `cstr_cow_from_bytes`
                        // 会直接借用这块内存传给 `GetProcAddress`，零分配。
                        let sym: libloading::Symbol<$ret> = lib
                            .get(concat!($sym, "\0").as_bytes())
                            .map_err(|_| MissingSymbol { name: $sym })?;
                        *sym
                    }, )*
                })
            }
        }
    };
}

/// 可选符号表：缺失即降级（`None`）。
macro_rules! define_optional_api {
    ($( $field:ident : $ret:ty = $sym:literal ),* $(,)?) => {
        /// 可选符号表。语义：**缺失即降级**，调用方必须先判 `is_some()`。
        ///
        /// 这里的每一项都对应「本机 DLL 与仓库 main 的 API 面差异」（实测），
        /// 或者「本版本才有」的增强能力。
        #[derive(Debug, Clone, Copy, Default)]
        pub struct OptionalApi {
            $( pub $field: Option<$ret>, )*
        }

        impl OptionalApi {
            unsafe fn bind(lib: &libloading::Library) -> Self {
                OptionalApi {
                    $( $field: lib
                        .get::<$ret>(concat!($sym, "\0").as_bytes())
                        .ok()
                        .map(|s| *s), )*
                }
            }
        }
    };
}

define_required_api! {
    // ---- 版本 ----
    core_version: unsafe extern "C" fn(*mut i32, *mut i32, *mut i32) = "ui_core_version",
    core_version_build: unsafe extern "C" fn() -> i32 = "ui_core_version_build",
    core_version_string: unsafe extern "C" fn() -> *const u8 = "ui_core_version_string",

    // ---- 生命周期 ----
    init: unsafe extern "C" fn() -> i32 = "ui_init",
    init_with_theme: unsafe extern "C" fn(UiThemeMode) -> i32 = "ui_init_with_theme",
    shutdown: unsafe extern "C" fn() = "ui_shutdown",
    run: unsafe extern "C" fn() -> i32 = "ui_run",
    quit: unsafe extern "C" fn(i32) = "ui_quit",

    // ---- 窗口 ----
    window_create: unsafe extern "C" fn(*const UiWindowConfig) -> UiWindow = "ui_window_create",
    window_destroy: unsafe extern "C" fn(UiWindow) = "ui_window_destroy",
    window_show: unsafe extern "C" fn(UiWindow) = "ui_window_show",
    window_show_immediate: unsafe extern "C" fn(UiWindow) = "ui_window_show_immediate",
    window_set_root: unsafe extern "C" fn(UiWindow, UiWidget) = "ui_window_set_root",
    window_set_title: unsafe extern "C" fn(UiWindow, *const u16) = "ui_window_set_title",
    window_set_resizable: unsafe extern "C" fn(UiWindow, i32) = "ui_window_set_resizable",
    window_invalidate: unsafe extern "C" fn(UiWindow) = "ui_window_invalidate",
    window_relayout: unsafe extern "C" fn(UiWindow) = "ui_window_relayout",
    window_hwnd: unsafe extern "C" fn(UiWindow) -> *mut core::ffi::c_void = "ui_window_hwnd",
    window_dpi: unsafe extern "C" fn(UiWindow) -> i32 = "ui_window_dpi",
    window_dpi_scale: unsafe extern "C" fn(UiWindow) -> f32 = "ui_window_dpi_scale",
    window_set_min_size: unsafe extern "C" fn(UiWindow, i32, i32) = "ui_window_set_min_size",
    window_set_rect: unsafe extern "C" fn(UiWindow, i32, i32, i32, i32) = "ui_window_set_rect",
    window_get_rect_screen: unsafe extern "C" fn(UiWindow, *mut i32, *mut i32, *mut i32, *mut i32) -> i32 = "ui_window_get_rect_screen",
    window_invoke_sync: unsafe extern "C" fn(UiWindow, UiInvokeCallback, *mut core::ffi::c_void) = "ui_window_invoke_sync",

    window_on_close: unsafe extern "C" fn(UiWindow, UiWindowCloseCallback, *mut core::ffi::c_void) = "ui_window_on_close",
    window_on_close_request: unsafe extern "C" fn(UiWindow, UiWindowCloseRequestCallback, *mut core::ffi::c_void) = "ui_window_on_close_request",
    window_on_resize: unsafe extern "C" fn(UiWindow, UiWindowResizeCallback, *mut core::ffi::c_void) = "ui_window_on_resize",
    window_on_drop: unsafe extern "C" fn(UiWindow, UiWindowDropCallback, *mut core::ffi::c_void) = "ui_window_on_drop",
    window_on_key: unsafe extern "C" fn(UiWindow, UiWindowKeyCallback, *mut core::ffi::c_void) = "ui_window_on_key",
    window_on_right_click: unsafe extern "C" fn(UiWindow, UiRightClickCallback, *mut core::ffi::c_void) = "ui_window_on_right_click",
    window_on_menu: unsafe extern "C" fn(UiWindow, UiMenuCallback, *mut core::ffi::c_void) = "ui_window_on_menu",
    window_on_mouse_button: unsafe extern "C" fn(UiWindow, UiWindowMouseButtonCallback, *mut core::ffi::c_void) = "ui_window_on_mouse_button",

    // ---- 控件 ----
    widget_find_by_id: unsafe extern "C" fn(UiWidget, *const u8) -> UiWidget = "ui_widget_find_by_id",
    widget_set_id: unsafe extern "C" fn(UiWidget, *const u8) = "ui_widget_set_id",
    widget_add_child: unsafe extern "C" fn(UiWidget, UiWidget) = "ui_widget_add_child",
    widget_remove_child: unsafe extern "C" fn(UiWidget, UiWidget) = "ui_widget_remove_child",
    widget_destroy: unsafe extern "C" fn(UiWidget) = "ui_widget_destroy",
    widget_get_rect: unsafe extern "C" fn(UiWidget) -> UiRect = "ui_widget_get_rect",
    widget_set_visible: unsafe extern "C" fn(UiWidget, i32) = "ui_widget_set_visible",
    widget_set_enabled: unsafe extern "C" fn(UiWidget, i32) = "ui_widget_set_enabled",
    widget_get_visible: unsafe extern "C" fn(UiWidget) -> i32 = "ui_widget_get_visible",
    widget_get_enabled: unsafe extern "C" fn(UiWidget) -> i32 = "ui_widget_get_enabled",
    widget_set_size: unsafe extern "C" fn(UiWidget, f32, f32) = "ui_widget_set_size",
    widget_set_tooltip: unsafe extern "C" fn(UiWidget, *const u16) = "ui_widget_set_tooltip",
    widget_set_cursor: unsafe extern "C" fn(UiWidget, i32) = "ui_widget_set_cursor",
    widget_on_click: unsafe extern "C" fn(UiWidget, UiClickCallback, *mut core::ffi::c_void) = "ui_widget_on_click",
    widget_on_mouse_move: unsafe extern "C" fn(UiWidget, UiWidgetMouseMoveCallback, *mut core::ffi::c_void) = "ui_widget_on_mouse_move",
    widget_on_mouse_leave: unsafe extern "C" fn(UiWidget, UiWidgetMouseLeaveCallback, *mut core::ffi::c_void) = "ui_widget_on_mouse_leave",
    widget_on_mouse_wheel: unsafe extern "C" fn(UiWidget, UiWidgetWheelCallback, *mut core::ffi::c_void) = "ui_widget_on_mouse_wheel",
    widget_on_focus: unsafe extern "C" fn(UiWidget, UiWidgetFocusCallback, *mut core::ffi::c_void) = "ui_widget_on_focus",
    widget_on_blur: unsafe extern "C" fn(UiWidget, UiWidgetFocusCallback, *mut core::ffi::c_void) = "ui_widget_on_blur",

    // ---- 自绘控件 ----
    custom: unsafe extern "C" fn() -> UiWidget = "ui_custom",
    custom_on_draw: unsafe extern "C" fn(UiWidget, UiCustomDrawCallback, *mut core::ffi::c_void) = "ui_custom_on_draw",
    custom_on_mouse_down: unsafe extern "C" fn(UiWidget, UiCustomMouseCallback, *mut core::ffi::c_void) = "ui_custom_on_mouse_down",
    custom_on_mouse_move: unsafe extern "C" fn(UiWidget, UiCustomMouseCallback, *mut core::ffi::c_void) = "ui_custom_on_mouse_move",
    custom_on_mouse_up: unsafe extern "C" fn(UiWidget, UiCustomMouseCallback, *mut core::ffi::c_void) = "ui_custom_on_mouse_up",
    custom_on_mouse_wheel: unsafe extern "C" fn(UiWidget, UiCustomWheelCallback, *mut core::ffi::c_void) = "ui_custom_on_mouse_wheel",
    custom_on_key_down: unsafe extern "C" fn(UiWidget, UiCustomKeyCallback, *mut core::ffi::c_void) = "ui_custom_on_key_down",
    custom_on_char: unsafe extern "C" fn(UiWidget, UiCustomCharCallback, *mut core::ffi::c_void) = "ui_custom_on_char",
    custom_on_layout: unsafe extern "C" fn(UiWidget, UiCustomLayoutCallback, *mut core::ffi::c_void) = "ui_custom_on_layout",
    custom_set_focusable: unsafe extern "C" fn(UiWidget, i32) = "ui_custom_set_focusable",
    custom_set_focused: unsafe extern "C" fn(UiWidget, i32) = "ui_custom_set_focused",
    custom_get_focused: unsafe extern "C" fn(UiWidget) -> i32 = "ui_custom_get_focused",

    // ---- 绘制 ----
    draw_fill_rect: unsafe extern "C" fn(UiDrawCtx, UiRect, UiColor) = "ui_draw_fill_rect",
    draw_rect: unsafe extern "C" fn(UiDrawCtx, UiRect, UiColor, f32) = "ui_draw_rect",
    draw_fill_rounded_rect: unsafe extern "C" fn(UiDrawCtx, UiRect, f32, f32, UiColor) = "ui_draw_fill_rounded_rect",
    draw_rounded_rect: unsafe extern "C" fn(UiDrawCtx, UiRect, f32, f32, UiColor, f32) = "ui_draw_rounded_rect",
    draw_line: unsafe extern "C" fn(UiDrawCtx, f32, f32, f32, f32, UiColor, f32) = "ui_draw_line",
    draw_text: unsafe extern "C" fn(UiDrawCtx, *const u16, UiRect, UiColor, f32) = "ui_draw_text",
    draw_text_ex: unsafe extern "C" fn(UiDrawCtx, *const u16, UiRect, UiColor, f32, i32, i32) = "ui_draw_text_ex",
    draw_measure_text: unsafe extern "C" fn(UiDrawCtx, *const u16, f32) -> f32 = "ui_draw_measure_text",
    draw_bitmap: unsafe extern "C" fn(UiDrawCtx, *const u8, i32, i32, i32, UiRect) = "ui_draw_bitmap",
    draw_push_clip: unsafe extern "C" fn(UiDrawCtx, UiRect) = "ui_draw_push_clip",
    draw_pop_clip: unsafe extern "C" fn(UiDrawCtx) = "ui_draw_pop_clip",
    bitmap_create: unsafe extern "C" fn(*const u8, i32, i32, i32) -> UiBitmap = "ui_bitmap_create",
    bitmap_destroy: unsafe extern "C" fn(UiBitmap) = "ui_bitmap_destroy",
    draw_bitmap_handle: unsafe extern "C" fn(UiDrawCtx, UiBitmap, UiRect, i32, f32) = "ui_draw_bitmap_handle",

    // ---- 页面 ----
    page_load_file: unsafe extern "C" fn(*const u16) -> UiPage = "ui_page_load_file",
    page_destroy: unsafe extern "C" fn(UiPage) = "ui_page_destroy",
    page_reload: unsafe extern "C" fn(UiPage) -> i32 = "ui_page_reload",
    page_root: unsafe extern "C" fn(UiPage) -> UiWidget = "ui_page_root",
    page_open_window: unsafe extern "C" fn(UiPage, *const UiWindowConfig) -> UiWindow = "ui_page_open_window",
    page_prepare_window: unsafe extern "C" fn(UiPage, *const UiWindowConfig) -> UiWindow = "ui_page_prepare_window",
    page_set_bool: unsafe extern "C" fn(UiPage, *const u8, i32) = "ui_page_set_bool",
    page_set_int: unsafe extern "C" fn(UiPage, *const u8, i32) = "ui_page_set_int",
    page_set_float: unsafe extern "C" fn(UiPage, *const u8, f64) = "ui_page_set_float",
    page_set_text: unsafe extern "C" fn(UiPage, *const u8, *const u16) = "ui_page_set_text",
    page_set_json: unsafe extern "C" fn(UiPage, *const u8, *const u8) -> i32 = "ui_page_set_json",
    page_get_json: unsafe extern "C" fn(UiPage, *const u8) -> *mut u8 = "ui_page_get_json",
    page_free: unsafe extern "C" fn(*mut core::ffi::c_void) = "ui_page_free",
    page_last_error: unsafe extern "C" fn(UiPage) -> *const u8 = "ui_page_last_error",
    page_watch: unsafe extern "C" fn(UiPage, *const u8, UiPageWatchCallback, *mut core::ffi::c_void) = "ui_page_watch",
    page_on_widget_mount: unsafe extern "C" fn(UiPage, *const u8, UiWidgetLifecycleCallback, *mut core::ffi::c_void) = "ui_page_on_widget_mount",
    page_on_widget_unmount: unsafe extern "C" fn(UiPage, *const u8, UiWidgetLifecycleCallback, *mut core::ffi::c_void) = "ui_page_on_widget_unmount",
    page_menu: unsafe extern "C" fn(UiPage, *const u8) -> UiMenu = "ui_page_menu",

    // ---- 资源 / 主题 / 菜单 / 反馈 ----
    asset_register_dir: unsafe extern "C" fn(*const u8) = "ui_asset_register_dir",
    asset_register_blob: unsafe extern "C" fn(*const u8, *const core::ffi::c_void, usize) = "ui_asset_register_blob",
    asset_reset: unsafe extern "C" fn() = "ui_asset_reset",
    theme_set_mode: unsafe extern "C" fn(UiThemeMode) = "ui_theme_set_mode",
    theme_get_mode: unsafe extern "C" fn() -> UiThemeMode = "ui_theme_get_mode",
    theme_bg: unsafe extern "C" fn() -> UiColor = "ui_theme_bg",
    theme_content_bg: unsafe extern "C" fn() -> UiColor = "ui_theme_content_bg",
    theme_sidebar_bg: unsafe extern "C" fn() -> UiColor = "ui_theme_sidebar_bg",
    theme_toolbar_bg: unsafe extern "C" fn() -> UiColor = "ui_theme_toolbar_bg",
    theme_accent: unsafe extern "C" fn() -> UiColor = "ui_theme_accent",
    theme_text: unsafe extern "C" fn() -> UiColor = "ui_theme_text",
    theme_divider: unsafe extern "C" fn() -> UiColor = "ui_theme_divider",
    theme_set_accent_hex: unsafe extern "C" fn(*const u8) -> i32 = "ui_theme_set_accent_hex",
    menu_show: unsafe extern "C" fn(UiWindow, UiMenu, f32, f32) = "ui_menu_show",
    menu_close: unsafe extern "C" fn(UiWindow) = "ui_menu_close",
    menu_item_id: unsafe extern "C" fn(UiMenuItem) -> *const u8 = "ui_menu_item_id",
    menu_item_attr: unsafe extern "C" fn(UiMenuItem, *const u8) -> *const u8 = "ui_menu_item_attr",
    toast: unsafe extern "C" fn(UiWindow, *const u16, i32) = "ui_toast",
    msgbox: unsafe extern "C" fn(UiWindow, *const u16, *const u16, *const *const u16, i32, i32, i32, i32) -> i32 = "ui_msgbox",

    // ---- 调试 / 离屏 ----
    debug_free: unsafe extern "C" fn(*mut core::ffi::c_void) = "ui_debug_free",
    debug_dump_tree: unsafe extern "C" fn(UiWindow) -> *mut u8 = "ui_debug_dump_tree",
    debug_screenshot: unsafe extern "C" fn(UiWindow, *const u16) -> i32 = "ui_debug_screenshot",
    debug_screenshot_menu: unsafe extern "C" fn(UiWindow, *const u16) -> i32 = "ui_debug_screenshot_menu",
    debug_server_start: unsafe extern "C" fn(UiWindow, *const u8) -> i32 = "ui_debug_server_start",
    offscreen_begin: unsafe extern "C" fn(i32, i32) -> UiDrawCtx = "ui_offscreen_begin",
    offscreen_end: unsafe extern "C" fn(UiDrawCtx, *mut u8, i32) -> i32 = "ui_offscreen_end",

    // ---- 图片 ----
    image_view: unsafe extern "C" fn() -> UiWidget = "ui_image_view",
    image_set_pixels: unsafe extern "C" fn(UiWidget, UiWindow, *const core::ffi::c_void, i32, i32, i32) = "ui_image_set_pixels",
    image_load_file: unsafe extern "C" fn(UiWidget, UiWindow, *const u16) = "ui_image_load_file",
    image_clear: unsafe extern "C" fn(UiWidget) = "ui_image_clear",
    image_fit: unsafe extern "C" fn(UiWidget) = "ui_image_fit",
    image_reset: unsafe extern "C" fn(UiWidget) = "ui_image_reset",
    image_set_zoom: unsafe extern "C" fn(UiWidget, f32) = "ui_image_set_zoom",
    image_get_zoom: unsafe extern "C" fn(UiWidget) -> f32 = "ui_image_get_zoom",
    image_width: unsafe extern "C" fn(UiWidget) -> i32 = "ui_image_width",
    image_height: unsafe extern "C" fn(UiWidget) -> i32 = "ui_image_height",
    image_get_pixels: unsafe extern "C" fn(UiWidget, UiWindow, *mut *mut core::ffi::c_void, *mut i32, *mut i32) -> i32 = "ui_image_get_pixels",
    image_free_pixels: unsafe extern "C" fn(*mut core::ffi::c_void) = "ui_image_free_pixels",
}

define_optional_api! {
    // 本机 GuoheView 目录的 core-ui.dll **有**、仓库 main **没有**（实测 12 个中的 4 类）。
    // 缺失时对应能力降级，不得崩溃。
    image_gdi_create: unsafe extern "C" fn(UiWidget, UiWindow, *const core::ffi::c_void, i32, i32, i32) -> i32 = "ui_image_gdi_create",
    image_gdi_set_pixels: unsafe extern "C" fn(UiWidget, UiWindow, *const core::ffi::c_void, i32, i32, i32) -> i32 = "ui_image_gdi_set_pixels",
    image_gdi_get_pixels: unsafe extern "C" fn(UiWidget, UiWindow, *mut *mut core::ffi::c_void, *mut i32, *mut i32) -> i32 = "ui_image_gdi_get_pixels",
    image_gdi_release: unsafe extern "C" fn(UiWidget) = "ui_image_gdi_release",
    image_gdi_invalidate: unsafe extern "C" fn(UiWidget) = "ui_image_gdi_invalidate",
    image_gdi_set_zoom: unsafe extern "C" fn(UiWidget, f32) = "ui_image_gdi_set_zoom",
    image_gdi_get_zoom: unsafe extern "C" fn(UiWidget) -> f32 = "ui_image_gdi_get_zoom",
    image_gdi_width: unsafe extern "C" fn(UiWidget) -> i32 = "ui_image_gdi_width",
    image_gdi_height: unsafe extern "C" fn(UiWidget) -> i32 = "ui_image_gdi_height",
    image_view_gdi: unsafe extern "C" fn() -> UiWidget = "ui_image_view_gdi",
    image_evict_tile: unsafe extern "C" fn(UiWidget, i32, i32) = "ui_image_evict_tile",
    flush_gpu: unsafe extern "C" fn() = "ui_flush_gpu",

    // 仓库 main **有**、本机 DLL **没有**（实测 2 个）。
    // 缺失时用「资源目录注册 + 运行时读文件」的降级路径。
    embed_binary: unsafe extern "C" fn(*const core::ffi::c_void, usize, *const u8) -> i32 = "ui_core_embed_binary",
    dialog: unsafe extern "C" fn() -> UiWidget = "ui_dialog",
}

/// 已加载的 core-ui 运行时。
///
/// `Drop` 时卸载 DLL。因为 `Api` 里存的是**裸函数指针**，`Drop` 必须在
/// 没有任何回调在飞、没有任何 widget 存活时执行 —— 由上层 `UiThread`
/// 保证「先销毁页面与窗口，再释放 CoreUi」。
pub struct CoreUi {
    /// 必须持有：句柄释放后所有函数指针失效。
    lib: libloading::Library,
    pub api: Api,
    pub optional: OptionalApi,
    pub version: CoreUiVersion,
    pub capabilities: Capabilities,
    /// 实际加载的 DLL 路径。
    pub dll_path: PathBuf,
}

impl CoreUi {
    /// 从给定路径加载。
    pub fn load_from(path: &Path) -> Result<Self, LoadError> {
        // libloading 需要以 NUL 结尾的 UTF-8 路径。
        let c_path = CString::new(path.as_os_str().to_string_lossy().as_bytes())
            .map_err(|_| LoadError::NotFound(vec![path.to_path_buf()]))?;
        // SAFETY: 路径由调用方给出；加载的是用户本机已安装 / CI 自建的 DLL。
        // 位数必须与本进程一致（x86-64，D00-7），libloading 在不匹配时会报错。
        let lib = unsafe { libloading::Library::new(c_path) }.map_err(LoadError::Load)?;

        // SAFETY: 符号名与 `api.rs` 的声明一致；缺失由 bind 返回 Err 而非崩溃。
        let api = unsafe { Api::bind(&lib) }.map_err(LoadError::Missing)?;
        // SAFETY: 同上；缺失降级为 None。
        let optional = unsafe { OptionalApi::bind(&lib) };

        let version = CoreUiVersion::probe(&api);
        let min_build = crate::version::min_build();
        if !version.meets_minimum(min_build) {
            return Err(LoadError::TooOld { found: version, min_build });
        }
        let capabilities = Capabilities::detect(&optional);

        Ok(CoreUi {
            lib,
            api,
            optional,
            version,
            capabilities,
            dll_path: path.to_path_buf(),
        })
    }

    /// 按搜索顺序自动找到并加载 core-ui.dll。
    ///
    /// 搜索顺序（**先到先得**）：
    /// 1. `<exe 同目录>\core-ui.dll` —— CI 产物的权威副本；
    /// 2. `<exe 同目录>\core-ui\core-ui.dll`；
    /// 3. `<exe 同目录>\..\core-ui.dll`；
    /// 4. `C:\Program Files\GuoheView\core-ui.dll` —— **兜底**。
    ///
    /// 第 4 项是兜底而非首选：本机那份 DLL 的 API 面与仓库 main 不同
    /// （实测多 12 个 / 少 2 个），且版本未必与 `packaging/core-ui-revision.txt`
    /// 对得上。加载到兜底路径时，`revision.rs` 会比对旁置的 `.revision` 戳记，
    /// 不一致就打警告。
    pub fn load_default() -> Result<Self, LoadError> {
        let candidates = default_candidates();
        for path in &candidates {
            if path.is_file() {
                return Self::load_from(path);
            }
        }
        Err(LoadError::NotFound(candidates))
    }

    /// 版本字符串（`ui_core_version_string`，借用）。
    pub fn version_string(&self) -> &str {
        // SAFETY: 头文件明示该指针指向静态字符串，进程生命周期内有效。
        unsafe {
            let p = (self.api.core_version_string)();
            if p.is_null() {
                return "<null>";
            }
            std::str::from_utf8(std::slice::from_raw_parts(p, crate::version::c_str_len(p)))
                .unwrap_or("<not-utf8>")
        }
    }
}

/// 候选 DLL 路径。
pub fn default_candidates() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            out.push(dir.join("core-ui.dll"));
            out.push(dir.join("core-ui").join("core-ui.dll"));
            if let Some(parent) = dir.parent() {
                out.push(parent.join("core-ui.dll"));
            }
        }
    }
    out.push(PathBuf::from(r"C:\Program Files\GuoheView\core-ui.dll"));
    out
}
