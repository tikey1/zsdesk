//! core-ui 冒烟探针 —— 验证 U01~U03 最小闭环。
//!
//! ```text
//! cargo run --release -p zsdesk-coreui --example probe -- \
//!     [--dll <core-ui.dll>] [--uix <页面.uix>] [--id <控件 id>]
//! ```
//!
//! 退出码：`0` = 全部通过；`1` = 失败（原因打印到 stderr）。
//!
//! # 失败判据：**不能用「句柄 == 0」**
//!
//! `ui_page_load_string` 的头文件注释写着「Returns 0 on failure」，但实现里
//! `CompileAndAttachPage` 的返回值被**丢弃**（上游 `page_api.cpp:557`），
//! 因此 **SFC 解析或编译失败时仍然返回非 0 句柄**。
//!
//! 可靠的失败信号有两个，**任一成立即判失败**：
//!
//! 1. `ui_page_root(page) == 0` —— 页面没有根节点；
//! 2. `ui_page_last_error(page)` 非空 —— 有解析 / 编译错误文本。
//!
//! 详见 `docs/14-QA-UIX-REVIEW.md` §12 的 P0-1。
//!
//! # 这个探针验证什么
//!
//! | 步骤 | 对应未核实项 | 验证内容 |
//! |---|---|---|
//! | `ui_core_version*` | —— | DLL 能被加载、必需符号齐全 |
//! | `ui_init_with_theme` | U01 | 初始化能跑通（返回值语义未核实，只打印） |
//! | `ui_page_load_string` | U02 | `.uix` 能被解析；顺带验证根 `<window>` 自闭合规则 |
//! | `ui_page_open_window` | U03 | 窗口能创建（传 NULL，不碰结构体布局） |
//! | `ui_widget_find_by_id` | —— | 资产里的 id 能取到句柄（验证 `.uix` 与 Rust 侧 id 对得上） |
//! | `:root` 注入 | U04 | **注入路径真跑一遍**（形态甲 / 剥 BOM / 只认 `:root` 内的定义） |

use std::path::PathBuf;
use std::process::ExitCode;

use zsdesk_coreui::api::UiThemeMode;
use zsdesk_coreui::CoreUi;

/// 最小可用的 `.uix`。
///
/// 根 `<window>` 必须**自闭合**（上游 `src/ui/uix/sfc_parser.cpp:285-297`）：
/// 非自闭合会 `pushErr` → `doc.ok = false` → **整页加载失败**，
/// 不是「缺一个控件」，而是「一个窗口都出不来」。
const MINIMAL_UIX: &str = r#"<window title="probe" width="320" height="200" />
<template>
  <div class="root"><label>probe</label></div>
</template>
<style>
  .root { flex: 1; }
</style>"#;

/// C 字符串长度（不含结尾 NUL）。
fn c_str_len(p: *const u8) -> usize {
    if p.is_null() {
        return 0;
    }
    let mut n = 0usize;
    // SAFETY: 调用方保证 p 指向以 NUL 结尾的有效缓冲区。
    while unsafe { *p.add(n) } != 0 {
        n += 1;
    }
    n
}

/// 按生产路径生成「注入 `:root` 后」的页面源码。
///
/// ★ 关键：这里调用的是**生产代码** `RootVars::inject_into`，
/// 不是探针自己重写一遍注入逻辑 —— 否则探针验证的是一个「影子实现」，
/// 生产代码改了探针还在绿灯，验证就失去意义。
///
/// 注入用的变量集刻意**只放一个** `--app-*` 变量：足以验证
/// 「注入位置合法（能被 core-ui 解析）+ 机制生效」，又不引入
/// 「变量值对不对」的额外维度（那是 UI 层的验收项，不是 ABI 冒烟的职责）。
fn build_injected_source(page_source: &str) -> Result<String, String> {
    let mut vars = zsdesk_coreui::RootVars::new();
    vars.set("--app-accent", "#007acc").map_err(|e| e.to_string())?;
    vars.inject_into(page_source).map_err(|e| e.to_string())
}

fn main() -> ExitCode {
    let mut dll: Option<PathBuf> = None;
    let mut uix: Option<PathBuf> = None;
    let mut probe_id: Option<String> = None;

    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--dll" => dll = args.next().map(PathBuf::from),
            "--uix" => uix = args.next().map(PathBuf::from),
            "--id" => probe_id = args.next(),
            "--help" | "-h" => {
                println!("用法: probe [--dll <core-ui.dll>] [--uix <页面.uix>] [--id <控件 id>]");
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("[FAIL] 未知参数: {other}");
                return ExitCode::FAILURE;
            }
        }
    }

    // ---- 1. 加载 DLL ----
    let core = match dll {
        Some(ref p) => CoreUi::load_from(p),
        None => CoreUi::load_default(),
    };
    let core = match core {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[FAIL] 加载 core-ui 失败: {e}");
            return ExitCode::FAILURE;
        }
    };
    println!("[OK] 已加载: {}", core.dll_path.display());
    println!("[OK] 运行时版本: {}", core.version_string());
    println!(
        "[OK] 编译期固化的 revision: {}",
        zsdesk_coreui::version::EXPECTED_REVISION
    );
    println!("[OK] 能力: {}", core.capabilities.summary());

    // ---- 2. 初始化（U01）----
    // 返回值语义未核实（U01）→ 只打印，不作为判据。
    // SAFETY: core-ui 已被成功加载，必需符号已由 loader 校验存在。
    let init_rc = unsafe { (core.api.init_with_theme)(UiThemeMode::Light) };
    println!("[OK] ui_init_with_theme 返回 {init_rc}");

    // ---- 3. 载入页面（U02）----
    let source = match uix {
        Some(ref p) => match std::fs::read_to_string(p) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[FAIL] 读取 .uix 失败: {e}");
                unsafe { (core.api.shutdown)() };
                return ExitCode::FAILURE;
            }
        },
        None => MINIMAL_UIX.to_string(),
    };

    // ---- 3a. 走【真实的注入路径】（U04）----
    //
    // 这一步不是可选的装饰：theme.rs 的注入逻辑（形态甲、剥 BOM、
    // 只认 :root 规则内的定义）如果不在这里跑，CI 就永远验证不到它 ——
    // 而它的失败模式全是【静默】的（颜色回落库默认色，零报错）。
    //
    // 注入后的源码必须能被 core-ui 正常解析 ⇒ 这一步同时验证了
    // 「注入位置合法」（在 <style> 内、顶层标签顺序合法）。
    let injected = match build_injected_source(&source) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[FAIL] :root 注入失败: {e}");
            unsafe { (core.api.shutdown)() };
            return ExitCode::FAILURE;
        }
    };
    println!("[OK] :root 注入完成（{} → {} 字节）", source.len(), injected.len());

    // SAFETY: source 是活的 Rust 字符串，core-ui 在调用期间只读它。
    let page = unsafe { (core.api.page_load_string)(injected.as_ptr()) };
    println!("[INFO] ui_page_load_string 返回句柄 {page}");

    // **失败判据**：句柄非 0 不代表成功。
    // SAFETY: page 是刚返回的句柄；last_error 返回借用指针，进程内有效。
    let last_error = unsafe { (core.api.page_last_error)(page) };
    let err_text = if last_error.is_null() {
        String::new()
    } else {
        // SAFETY: last_error 指向以 NUL 结尾的 UTF-8 文本。
        unsafe {
            String::from_utf8_lossy(std::slice::from_raw_parts(last_error, c_str_len(last_error)))
        }
    };
    // SAFETY: 同上。
    let root = unsafe { (core.api.page_root)(page) };

    if root == 0 || !err_text.is_empty() {
        eprintln!("[FAIL] 页面加载失败：root={root} error={err_text}");
        // SAFETY: page 是有效句柄。
        unsafe { (core.api.page_destroy)(page) };
        unsafe { (core.api.shutdown)() };
        return ExitCode::FAILURE;
    }
    println!("[OK] 页面加载成功：root={root}");

    // ---- 4. 开窗（U03）----
    // 传 NULL：不使用 `UiWindowConfig`，规避手写 #[repr(C)] 的布局漂移风险。
    // SAFETY: page 是有效句柄。
    let window = unsafe { (core.api.page_open_window)(page, std::ptr::null()) };
    if window == 0 {
        eprintln!("[FAIL] 窗口创建失败（句柄为 0）");
        unsafe { (core.api.page_destroy)(page) };
        unsafe { (core.api.shutdown)() };
        return ExitCode::FAILURE;
    }
    println!("[OK] 窗口已创建：{window}");

    // ---- 5. 可选：验证资产里的 id 能取到句柄 ----
    if let Some(id) = probe_id {
        let mut buf = id.into_bytes();
        buf.push(0);
        // SAFETY: root 是有效句柄；buf 是以 NUL 结尾的 UTF-8。
        let w = unsafe { (core.api.widget_find_by_id)(root, buf.as_ptr()) };
        if w == 0 {
            eprintln!("[FAIL] 找不到 id 对应的控件");
            unsafe { (core.api.page_destroy)(page) };
            unsafe { (core.api.shutdown)() };
            return ExitCode::FAILURE;
        }
        println!("[OK] id 可取到句柄：{w}");
    }

    // ---- 6. 收尾 ----
    // SAFETY: page / window 均为本次持有的有效句柄。
    unsafe { (core.api.page_destroy)(page) };
    unsafe { (core.api.shutdown)() };
    println!("[OK] 探针通过");
    ExitCode::SUCCESS
}
