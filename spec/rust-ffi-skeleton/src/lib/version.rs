//! 版本与能力协商。
//!
//! ## 为什么必须协商（实测依据）
//!
//! `research/facts.md` §7.1 记录了 PE 导出表逐名比对的结果：
//!
//! | 项 | 数量 |
//! |---|---|
//! | 仓库 `include/ui_core.h` 声明 `ui_*` | 557 |
//! | 本机 `core-ui.dll` 导出 `ui_*` | 567 |
//! | 双方都有 | 555 |
//! | 仓库有、本机无 | 2（`ui_core_embed_binary`、`ui_dialog`）|
//! | 本机有、仓库无 | 12（`ui_image_gdi_*` 9 个、`ui_image_view_gdi`、`ui_image_evict_tile`、`ui_flush_gpu`）|
//!
//! **结论：两份 core-ui.dll 不是同一个 API 面，谁都不能假设对方存在。**
//! 本模块把这套差异固化成三件事：
//!
//! 1. `CoreUiVersion` —— 运行时实测版本（`ui_core_version*`），与编译期常量比对；
//! 2. `Capabilities` —— 按**符号是否存在**（而不是版本号）判定能力，
//!    因为本机那份 DLL 的 build 号未知，版本号不可靠，符号在/不在才是事实；
//! 3. `RevisionStamp` —— 旁置的 `<dll>.revision` 文本戳记，用来证明
//!    「手边这份 DLL 就是 CI 按 pinned revision 构建的那份」。
//!
//! ## 铁律
//!
//! **任何 `ui_*` 调用前不得假设该 API 必然存在。** 可选能力一律走
//! `optional.*` 的 `Option`，缺失即降级，不得崩溃、不得静默当成功。

use std::path::Path;

use crate::api::UiThemeMode;
use crate::loader::{Api, OptionalApi};

/// 编译期固化的 core-ui revision（来自 `packaging/core-ui-revision.txt`）。
pub const EXPECTED_REVISION: &str = env!("ZSCOREUI_CORE_UI_REVISION");

/// 编译期固化的最低 build 号。
pub const MIN_BUILD_ENV: &str = env!("ZSCOREUI_CORE_UI_MIN_BUILD");

/// 绑定层支持的最低 build 号。
pub fn min_build() -> u32 {
    MIN_BUILD_ENV.parse().unwrap_or(0)
}

/// core-ui 的运行时版本。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoreUiVersion {
    pub major: i32,
    pub minor: i32,
    pub patch: i32,
    pub build: i32,
}

impl CoreUiVersion {
    /// 实测当前加载的 DLL 版本。
    pub fn probe(api: &Api) -> Self {
        let (mut major, mut minor, mut patch) = (0i32, 0i32, 0i32);
        // SAFETY: 三个指针都是合法的可写 i32，且头文件允许传非 NULL。
        unsafe { (api.core_version)(&mut major, &mut minor, &mut patch) };
        // SAFETY: 无参数函数。
        let build = unsafe { (api.core_version_build)() };
        CoreUiVersion { major, minor, patch, build }
    }

    /// 是否满足最低 build 要求。
    pub fn meets_minimum(&self, min_build: u32) -> bool {
        (self.build as u32) >= min_build
    }
}

impl std::fmt::Display for CoreUiVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}.{}", self.major, self.minor, self.patch, self.build)
    }
}

/// 按**符号是否存在**判定的能力表。
///
/// 不用版本号判定，因为实测存在「本机 DLL 更新、仓库 main 更旧」的情况，
/// 版本号与能力不是单调关系。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Capabilities {
    /// GDI 位图路径（`ui_image_gdi_*`）—— 本机 DLL 有，仓库 main 无。
    pub image_gdi: bool,
    /// `ui_image_view_gdi` —— 本机 DLL 有，仓库 main 无。
    pub image_view_gdi: bool,
    /// `ui_image_evict_tile` —— 本机 DLL 有，仓库 main 无。
    pub image_evict_tile: bool,
    /// `ui_flush_gpu` —— 本机 DLL 有，仓库 main 无。
    pub flush_gpu: bool,
    /// `ui_core_embed_binary` —— 仓库 main 有，本机 DLL 无。
    pub embed_binary: bool,
    /// `ui_dialog` —— 仓库 main 有，本机 DLL 无。
    pub dialog: bool,
}

impl Capabilities {
    /// 依据可选符号表判定。
    pub fn detect(opt: &OptionalApi) -> Self {
        let image_gdi = opt.image_gdi_create.is_some()
            && opt.image_gdi_set_pixels.is_some()
            && opt.image_gdi_get_pixels.is_some()
            && opt.image_gdi_release.is_some()
            && opt.image_gdi_invalidate.is_some()
            && opt.image_gdi_set_zoom.is_some()
            && opt.image_gdi_get_zoom.is_some()
            && opt.image_gdi_width.is_some()
            && opt.image_gdi_height.is_some();
        Capabilities {
            image_gdi,
            image_view_gdi: opt.image_view_gdi.is_some(),
            image_evict_tile: opt.image_evict_tile.is_some(),
            flush_gpu: opt.flush_gpu.is_some(),
            embed_binary: opt.embed_binary.is_some(),
            dialog: opt.dialog.is_some(),
        }
    }

    /// 人类可读的能力清单（写进诊断报告与启动日志）。
    pub fn summary(&self) -> String {
        let mut v = Vec::new();
        v.push(format!("image_gdi={}", self.image_gdi));
        v.push(format!("image_view_gdi={}", self.image_view_gdi));
        v.push(format!("image_evict_tile={}", self.image_evict_tile));
        v.push(format!("flush_gpu={}", self.flush_gpu));
        v.push(format!("embed_binary={}", self.embed_binary));
        v.push(format!("dialog={}", self.dialog));
        v.join(" ")
    }
}

/// DLL 旁置的 revision 戳记。
///
/// CI 在打包时把 `packaging/core-ui-revision.txt` 的内容写成
/// `<dll 名>.revision`（纯文本，一行）。运行时读取并比对：
/// - 一致 → 一切正常；
/// - 不一致或缺失 → **打警告**（不阻断启动：兜底路径加载本机 GuoheView 的
///   DLL 是允许的降级行为），但写进诊断报告，让用户能一眼看出「你跑的不是
///   CI 构建的那份」。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RevisionStamp {
    Match,
    Mismatch { expected: String, found: String },
    Missing,
}

impl RevisionStamp {
    /// 读取 `<dll_path>.revision` 并比对。
    pub fn check(dll_path: &Path, expected: &str) -> Self {
        if expected == "unpinned" {
            return RevisionStamp::Missing;
        }
        let stamp_path = stamp_path_for(dll_path);
        match std::fs::read_to_string(&stamp_path) {
            Ok(text) => {
                let found = text.trim().to_string();
                if found == expected {
                    RevisionStamp::Match
                } else {
                    RevisionStamp::Mismatch { expected: expected.to_string(), found }
                }
            }
            Err(_) => RevisionStamp::Missing,
        }
    }
}

/// 戳记文件路径：`<dll 名>.revision`。
pub fn stamp_path_for(dll_path: &Path) -> std::path::PathBuf {
    let mut name = dll_path
        .file_name()
        .map(|s| s.to_os_string())
        .unwrap_or_default();
    name.push(".revision");
    dll_path.with_file_name(name)
}

/// 读取 C 字符串长度（不含结尾 NUL）。
///
/// # Safety
/// `ptr` 必须指向以 NUL 结尾的有效 C 字符串。
pub unsafe fn c_str_len(ptr: *const u8) -> usize {
    if ptr.is_null() {
        return 0;
    }
    let mut n = 0usize;
    while *ptr.add(n) != 0 {
        n += 1;
    }
    n
}

/// 主题模式的可读名（写日志用）。
pub fn theme_name(mode: UiThemeMode) -> &'static str {
    match mode {
        UiThemeMode::Dark => "dark",
        UiThemeMode::Light => "light",
    }
}
