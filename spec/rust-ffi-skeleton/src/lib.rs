//! `zc-ffi-coreui` —— core-ui C API 的运行时动态加载绑定层。
//!
//! # 这个 crate 的职责边界
//!
//! **只做三件事**：
//!
//! 1. 运行时加载 `core-ui.dll`，建立「必需符号 / 可选符号」两张表；
//! 2. 版本与能力协商（`ui_core_version*` + 按符号存在性判定）；
//! 3. UI 线程收敛（`invoke_sync` 封送）。
//!
//! **不做的事**（这些属于上层）：
//!
//! - 不画任何东西（文件列表 / 导航树的自绘在 `zc-ui`）；
//! - 不持有业务状态（状态在 `zc-app`，见 `spec/ffi-core-ui.md` §4 状态归属）；
//! - 不认识 `.uix` 的语义（页面装配在 `zc-ui`）；
//! - 不反向依赖任何内部 crate —— **FFI 层不得反向依赖 UI 业务层**。
//!
//! # unsafe 纪律
//!
//! 本 crate 是全仓库**唯一**允许 `unsafe` 的 crate（其余 crate 用
//! `#![forbid(unsafe_code)]`）。理由：core-ui 是 C ABI，绕不开。
//! 但 unsafe 被关在三个地方，且每处都有注释说明**为什么安全**：
//!
//! | 位置 | unsafe 的用途 |
//! |---|---|
//! | `api.rs` | 纯声明，无实现 |
//! | `loader.rs` | `Library::new` / `lib.get` |
//! | `ui_thread.rs` | 裸指针解引用 + trampoline |
//!
//! # 用法（骨架示意，不是业务逻辑）
//!
//! ```no_run
//! use zc_ffi_coreui::CoreUi;
//!
//! let core = CoreUi::load_default()?;              // 找不到就大声失败
//! println!("core-ui {} @ {}", core.version, core.dll_path.display());
//! println!("能力：{}", core.capabilities.summary());
//! # Ok::<(), zc_ffi_coreui::LoadError>(())
//! ```

#![deny(missing_debug_implementations)]

pub mod api;
pub mod loader;
pub mod ui_thread;
pub mod version;

pub use api::{
    UiBitmap, UiColor, UiDrawCtx, UiMenu, UiMenuItem, UiPage, UiRect, UiThemeMode, UiWidget, UiWindow,
};
pub use loader::{Api, CoreUi, LoadError, MissingSymbol, OptionalApi};
pub use ui_thread::{SharedUi, UiHandle, UiThread};
pub use version::{Capabilities, CoreUiVersion, RevisionStamp};

// ---------------------------------------------------------------------------
// 字符串编解码：UTF-8 ↔ UTF-16
//
// core-ui 的头文件里 `const char*` 与 `const wchar_t*` 混用（实测 72 : 77），
// 且**逐函数不同**。本 crate 的上层 API 一律收 UTF-8 `&str`，
// 转换只发生在这里，且每个转换点都有测试。
// ---------------------------------------------------------------------------

/// UTF-8 → 以 NUL 结尾的 UTF-16LE。
///
/// 传给 core-ui 的所有 `const wchar_t*` 参数都必须用它构造。
pub fn to_utf16(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// 以 NUL 结尾的 UTF-16LE → UTF-8 `String`。
///
/// 遇到未配对的代理项用 U+FFFD 替换，不 panic —— 界面文本不该因为一个坏码点
/// 让整个面板崩掉。
pub fn from_utf16(slice: &[u16]) -> String {
    String::from_utf16_lossy(slice)
}

/// 借用指针指向的 NUL 结尾 UTF-16LE → UTF-8 `String`。
///
/// # Safety
/// `ptr` 必须指向以 NUL 结尾的有效 UTF-16LE 缓冲区，且在调用期间有效。
pub unsafe fn from_utf16_ptr(ptr: *const u16) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let mut len = 0usize;
    while *ptr.add(len) != 0 {
        len += 1;
    }
    from_utf16(std::slice::from_raw_parts(ptr, len))
}

/// 借用指针指向的 NUL 结尾 UTF-8 → `&str`。
///
/// # Safety
/// `ptr` 必须指向以 NUL 结尾的有效 UTF-8 缓冲区，且在调用期间有效。
pub unsafe fn from_utf8_ptr<'a>(ptr: *const u8) -> &'a str {
    if ptr.is_null() {
        return "";
    }
    let mut len = 0usize;
    while *ptr.add(len) != 0 {
        len += 1;
    }
    std::str::from_utf8(std::slice::from_raw_parts(ptr, len)).unwrap_or("")
}

/// 启动自检报告。写进诊断日志，也写进「关于」对话框。
#[derive(Debug, Clone)]
pub struct StartupReport {
    /// 实测版本串（`ui_core_version_string`）。
    pub version_string: String,
    /// 语义化版本。
    pub version: CoreUiVersion,
    /// 实际加载的 DLL 路径。
    pub dll_path: String,
    /// 编译期固化的 revision。
    pub expected_revision: String,
    /// 旁置戳记的比对结果。
    pub revision_stamp: RevisionStamp,
    /// 能力清单。
    pub capabilities: Capabilities,
}

impl StartupReport {
    /// 生成一段人类可读的摘要。
    pub fn summary(&self) -> String {
        let stamp = match &self.revision_stamp {
            RevisionStamp::Match => "revision 一致".to_string(),
            RevisionStamp::Mismatch { expected, found } => {
                format!("revision 不一致：期望 {expected}，实际 {found}")
            }
            RevisionStamp::Missing => "无 revision 戳记（.revision 文件缺失）".to_string(),
        };
        format!(
            "core-ui {} | {} | {}\n能力：{}",
            self.version_string,
            self.dll_path,
            stamp,
            self.capabilities.summary()
        )
    }
}

impl CoreUi {
    /// 加载并做一次完整自检（版本 + 能力 + revision 戳记）。
    pub fn load_checked() -> Result<(Self, StartupReport), LoadError> {
        let core = Self::load_default()?;
        let report = StartupReport {
            version_string: core.version_string().to_string(),
            version: core.version,
            dll_path: core.dll_path.display().to_string(),
            expected_revision: version::EXPECTED_REVISION.to_string(),
            revision_stamp: RevisionStamp::check(&core.dll_path, version::EXPECTED_REVISION),
            capabilities: core.capabilities,
        };
        Ok((core, report))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf16_roundtrip() {
        let s = "文件列表 / File List";
        let w = to_utf16(s);
        assert_eq!(w.last().copied(), Some(0));
        assert_eq!(from_utf16(&w[..w.len() - 1]), s);
    }

    #[test]
    fn utf16_handles_broken_surrogate() {
        // 孤立的代理项：不得 panic。
        let broken = [0xD800u16, 0x0041u16];
        let out = from_utf16(&broken);
        assert!(out.contains('\u{FFFD}'));
    }
}
