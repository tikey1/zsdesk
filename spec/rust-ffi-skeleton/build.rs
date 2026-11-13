//! 构建期：把「本二进制是针对哪个 core-ui revision 构建的」固化进产物。
//!
//! 存在的理由（D03 / T7）：
//! - CI 从源码构建 core-ui，产物随 exe 分发。运行时无法证明「手边的 core-ui.dll
//!   就是我们绑定的那份」，必须把 revision 写进 exe，再由 CI 校验 DLL 与之匹配。
//! - `cargo:rerun-if-changed` 保证改了 `packaging/core-ui-revision.txt` 就重编，
//!   不会出现「revision 文件改了但 exe 里还是旧值」的静默漂移。
//!
//! 查找顺序：
//! 1. 环境变量 `ZSCOREUI_REVISION`（CI 注入，最高优先级）；
//! 2. `<workspace>/packaging/core-ui-revision.txt`；
//! 3. `<crate>/packaging/core-ui-revision.txt`（骨架单独放置时）。
//!
//! 三者都没有时：发出 `cargo:warning` 并写入 `unpinned`。
//! 严格性不放在这里 —— 按 mechanical-gates「失败即停」的要求，缺 revision 应当由
//! CI 的可执行门禁判死（见 spec/ci-contract.md §5 步骤 S7），而不是让本机开发者
//! 无法编译。build.rs 只负责把事实如实写进产物。

use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=ZSCOREUI_REVISION");
    println!("cargo:rerun-if-env-changed=ZSCOREUI_REVISION_FILE");

    let revision = resolve_revision();
    println!("cargo:rustc-env=ZSCOREUI_CORE_UI_REVISION={revision}");
    println!("cargo:rustc-env=ZSCOREUI_CORE_UI_MIN_BUILD={}", MIN_BUILD);
}

/// 绑定层支持的最低 core-ui build 号。
///
/// 取自 `packaging/core-ui-revision.txt` 指向的那一版（1.9.0 build 345）。
/// 低于这个 build 的 core-ui.dll 缺少本绑定依赖的符号，加载时直接失败（大声失败，
/// 不静默降级）。改动这个常量 = 一次显式的兼容性决策，必须同步更新
/// spec/ffi-core-ui.md §6。
const MIN_BUILD: u32 = 345;

fn resolve_revision() -> String {
    if let Ok(v) = std::env::var("ZSCOREUI_REVISION") {
        let v = v.trim().to_string();
        if !v.is_empty() {
            return v;
        }
    }
    if let Ok(path) = std::env::var("ZSCOREUI_REVISION_FILE") {
        if let Some(v) = read_revision(Path::new(&path)) {
            return v;
        }
    }
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default());
    let candidates = [
        manifest.join("../../packaging/core-ui-revision.txt"),
        manifest.join("packaging/core-ui-revision.txt"),
    ];
    for c in candidates {
        if let Some(v) = read_revision(&c) {
            return v;
        }
    }
    println!(
        "cargo:warning=未找到 core-ui revision 文件；产物将标记为 unpinned。\
         CI 门禁会因此判失败（见 spec/ci-contract.md §5 S7）。"
    );
    "unpinned".to_string()
}

fn read_revision(path: &Path) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    let v = text.trim().to_string();
    if v.is_empty() {
        None
    } else {
        println!("cargo:rerun-if-changed={}", path.display());
        Some(v)
    }
}
