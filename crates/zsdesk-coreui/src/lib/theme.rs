//! `.uix` 的 `:root` 变量注入 —— 把 Rust 侧的调色板写进页面源码。
//!
//! # 为什么需要这个模块
//!
//! core-ui **没有**读写 CSS 变量的 C API（导出面只有 `ui_widget_set_bg_color`
//! 这类写接口，没有 `get_style` / `get_var` / `computed`）。所以「把品牌色
//! 注入页面」只能走**改源码**这条路：`ui_page_load_string` 接受字符串，
//! 于是我们在加载前把一段 `:root { … }` 拼进 `.uix` 源码。
//!
//! # 四条硬约束（全部来自 core-ui v1.9.0.345 源码，行号为实测）
//!
//! ## 约束一：`:root` 块必须【在 `<style>` 内】
//!
//! `src/ui/uix/sfc_parser.cpp:260-266` —— 顶层只允许
//! `<window> / <template> / <script> / <style> / <import> / <link>`，
//! 任何不以 `<` 开头的顶层内容都会得到
//! `"unexpected content at top level"` → **整页解析失败**。
//!
//! ⇒ **不能**把 `:root { … }` 作为独立块放在源码顶层。
//!
//! ## 约束二：`<style>` 元素**不得**自闭合
//!
//! `src/ui/uix/sfc_parser.cpp:321-330` —— `<style>` 走 `collectBody(c, "style", ...)`
//! 收集**元素体**；若写成 `<style />` 自闭合形式，块体为空 → 注入的 `:root` 丢失。
//! （CSS 里的 `:root { … }` 本身是 `{ … }` 配对，与 SFC 的元素闭合是两回事。）
//!
//! ## 约束三：`<style>` 内可以有多条 rule
//!
//! `src/ui/css/css_parser.cpp:372-387` —— `ParseStylesheet` 循环解析，
//! 逐条 rule 压入 stylesheet。⇒ 往已有 `<style>` 里**前面**插一条
//! `:root` rule 是合法的，不会破坏原有的 rule。
//!
//! ## 约束四（★ 决定注入位置）：`:root` 是【先定义者赢】
//!
//! - `src/ui/css/value.cpp:572-579` —— `LookupVar` 遍历变量表，
//!   **第一个命中就 `return true`**；
//! - `src/ui/css/cascade.h:41` —— 注释逐字「Returns first set encountered.」
//! - `src/ui/css/cascade.cpp:18-33` —— `CollectRootVars` 按 rule 在
//!   stylesheet 中**出现的先后顺序**收集 `:root` 变量。
//!
//! ⇒ **同一个变量名被 `:root` 定义两次时，靠前的那次生效，靠后的被静默忽略**。
//!
//! ⚠️ **因果不要弄反**：`cascade.cpp:125` 让 `:root` 整条跳过 CSS 级联，
//! 那是**「`:root` 只是变量容器、不参与普通级联」**的原因，
//! **不是**「先定义者赢」的原因。先定义者赢来自 `LookupVar` 的首个命中语义。
//!
//! ## 约束五：注入前必须剥掉 UTF-8 BOM
//!
//! 注入是**前置**的 ⇒ 若原源码以 BOM 开头，BOM 会被挤到文件中部
//! → SFC 顶层出现非 `<` 字符 → `sfc_parser.cpp:260-266` 判
//! "unexpected content at top level" → **整页不加载**。
//! 而 `std::fs::read_to_string` **不剥 BOM**（BOM 会原样进字符串），
//! 所以必须在注入前显式剥。
//!
//! ## 约束六：页面**不能有 `<link>`**
//!
//! `src/ui/page/page_api.cpp:271` —— link 块排在所有 inline `<style>` **之前**
//! ⇒ 若 link 的 CSS 里定义了 `:root`，它仍然先定义者赢 → 注入值被吃掉。
//! 当前三个资产零 `<link>`。
//!
//! # 注入形态：甲（在源码最前面插入一个完整的 `<style>` 块）
//!
//! 本实现走**形态甲**（见 `spec/ffi-core-ui.md` §4.3.2.1），理由：
//!
//! 1. **不依赖页面是否已有 `<style>` 块** —— 形态乙（插进页面已有的
//!    `<style>` 块首）在页面没有 `<style>` 时直接无从下手，形态甲照样能注入。
//! 2. **天然保证「注入的是第一个 `:root`」** —— `doc.styles` 按源码中
//!    `<style>` 出现的顺序收集（`sfc_parser.cpp:330`），
//!    `page_api.cpp:333` 再按该顺序逐块 parse 压入同一 sheet
//!    ⇒ 插在最前面的块是 `blocks[0]` ⇒ 它的 `:root` 排在页面自己的 `:root` 之前
//!    ⇒ **页面自己的定义胜出**（符合「页面可覆盖品牌默认值」的直觉）。
//!
//! ⚠️ **不要用「在源码末尾追加」**：那样一旦 `.uix` 里出现自己的 `:root`，
//! **`.uix` 的值会赢**，Rust 的注入被**静默吃掉**
//! （症状：切主题颜色不变，**零报错**）。
//!
//! # 前缀约定
//!
//! core-ui 自己拥有 34 个 CSS 变量 token，其中 `--bg` / `--accent` /
//! `--sidebar-bg` 与本项目想要的语义**撞名**。因此本项目注入的变量
//! **一律带 `--app-` 前缀**，绝不与库 token 混用。
//!
//! ⚠️ `--app-accent` 是**例外**：它是 D14 的回退源值（**数据字段，不是
//! CSS 变量**）—— Rust 读出后直接喂 `ui_theme_set_accent_hex()`，
//! **不进 `:root`**。`.uix` 里一律读 `var(--accent)`（库 token），
//! 不得出现 `var(--app-accent)`。
//!
//! # F1–F5 契约（见 `spec/ffi-core-ui.md` §4）
//!
//! | 编号 | 契约 |
//! |---|---|
//! | F1 | 注入块只允许 `--app-*` 变量名 |
//! | F2 | `--app-*` **不随** `ui_theme_set_mode` 变化 → 切主题必须重建页面 |
//! | F3 | 重建后**全部句柄失效**，必须重取 |
//! | F4 | 字体是库 token，不需重建 |
//! | F5 | accent 基色跨模式保留，只重新派生变体 → 切主题必须重读重调 |

use std::collections::BTreeMap;

/// 注入变量的统一前缀。core-ui 自带的 token 一律不带此前缀。
pub const APP_VAR_PREFIX: &str = "--app-";

/// 一组要注入的 CSS 变量。
///
/// 用 [`BTreeMap`] 而非 `HashMap`：注入顺序必须**确定**，
/// 否则同一份配置两次注入会得到不同的源码，破坏可复现性。
#[derive(Debug, Clone, Default)]
pub struct RootVars {
    vars: BTreeMap<String, String>,
}

/// 注入失败的原因。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InjectError {
    /// 变量名不合法（空、含非法字符、或不以 `--app-` 开头）。
    BadVarName(String),
    /// 变量值为空 —— 写进 CSS 会让整条声明失效。
    EmptyValue(String),
}

impl std::fmt::Display for InjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InjectError::BadVarName(n) => {
                write!(f, "非法的变量名「{n}」：必须以 {APP_VAR_PREFIX} 开头")
            }
            InjectError::EmptyValue(n) => write!(f, "变量「{n}」的值为空"),
        }
    }
}

impl std::error::Error for InjectError {}

impl RootVars {
    /// 新建一组变量。
    pub fn new() -> Self {
        Self::default()
    }

    /// 加入一个变量。变量名必须以 `--app-` 开头（F1）。
    pub fn set(&mut self, name: &str, value: &str) -> Result<(), InjectError> {
        if !name.starts_with(APP_VAR_PREFIX) || name.len() == APP_VAR_PREFIX.len() {
            return Err(InjectError::BadVarName(name.to_string()));
        }
        if value.trim().is_empty() {
            return Err(InjectError::EmptyValue(name.to_string()));
        }
        self.vars.insert(name.to_string(), value.to_string());
        Ok(())
    }

    /// 当前变量数。
    pub fn len(&self) -> usize {
        self.vars.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.vars.is_empty()
    }

    /// 迭代变量（按名字排序，保证确定性）。
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.vars.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    /// 生成 `:root{…}` 这条 rule（不含首尾空白）。
    ///
    /// 变量名已按字典序排列 —— 注入顺序确定 ⇒ 同样输入得到同样源码。
    fn render_rule(&self, skip_defined_in: Option<&str>) -> String {
        let mut out = String::from(":root{");
        for (name, value) in self.iter() {
            // 页面已经定义过的同名变量不重复注入：
            // 先定义者赢意味着重复注入的那份本来就会被忽略，
            // 但留着会让源码出现「同一个变量定义两次」的歧义，徒增排查成本。
            if let Some(page) = skip_defined_in {
                if page_defines_var(page, name) {
                    continue;
                }
            }
            out.push_str(name);
            out.push(':');
            out.push_str(value);
            out.push(';');
        }
        out.push('}');
        out
    }

    /// 把变量注入 `.uix` 源码。
    ///
    /// 注入点：**源码最前面插入一个完整的 `<style>` 块**
    /// （形态甲，见 `spec/ffi-core-ui.md` §4.3.2.1）。
    ///
    /// 为什么不插进页面已有的 `<style>` 块（形态乙）：
    /// 形态甲不依赖页面是否已有 `<style>` 块 —— 页面没有时形态乙直接无从下手，
    /// 而形态甲照样能注入。且形态甲天然保证「注入的是第一个 `:root>`」。
    ///
    /// # Errors
    /// 变量名或值不合法时返回 [`InjectError`]。
    pub fn inject_into(&self, page_source: &str) -> Result<String, InjectError> {
        if self.is_empty() {
            return Ok(page_source.to_string());
        }

        // 剥掉 UTF-8 BOM。★ 必须剥：注入是【前置】的，若原源码以 BOM 开头，
        // BOM 会被挤到文件中部 → SFC 顶层出现非 '<' 字符 →
        // sfc_parser.cpp:260-266 判 "unexpected content at top level" → 整页不加载。
        // 而 Rust 的 std::fs::read_to_string **不剥 BOM**（BOM 会原样进字符串）。
        let page_source = page_source.strip_prefix('\u{feff}').unwrap_or(page_source);

        // 页面自己已经定义过的同名变量不重复注入。
        // 注意：core-ui **只采集 `:root` 规则里的自定义属性**
        // （cascade.cpp:134 跳过非-:root 级的自定义属性），
        // 所以判据必须是「某个 :root 块里定义了这个变量」，
        // 而不是「源码里出现过这个变量名」。
        let rule = self.render_rule(Some(page_source));

        // 整页都已定义 ⇒ 无需注入，原样返回。
        if rule == ":root{}" {
            return Ok(page_source.to_string());
        }

        // 形态甲：在源码最前面插入一个完整的 <style> 块。
        // 顶层标签顺序无关（sfc_parser 按标签名分派到各自容器），
        // 而 doc.styles 按源码出现顺序收集（sfc_parser.cpp:330）、
        // page_api.cpp:333 按该顺序逐块 parse 压入同一 sheet
        // → 插在最前面的块就是 blocks[0] → 它的 :root 是第一个 :root。
        //
        // ⚠️ 前提：页面**不能有 <link>**。page_api.cpp:271 把 link 块排在
        // 所有 inline <style> 之前 ⇒ 若 link 的 CSS 里定义了 :root，
        // 它仍然先定义者赢，注入值会被吃掉。当前三个资产零 <link>。
        let block = format!("<style>\n{rule}\n</style>\n");
        let mut out = String::with_capacity(page_source.len() + block.len());
        out.push_str(&block);
        out.push_str(page_source);
        Ok(out)
    }
}

/// 页面源码里是否已经定义了该 CSS 变量。
///
/// 判据必须精确到 **`:root` 规则内部**：
/// core-ui 只采集 `:root` 规则里的自定义属性
/// （`src/ui/css/cascade.cpp:134` 跳过非 `:root` 级的自定义属性），
/// 所以「别的选择器里写了 `--app-accent: …`」**不算**定义 ——
/// 那条声明永远不会被采集，若因此跳过注入，变量就静默解析为空串
/// （症状：界面全是库默认色，零报错）。
///
/// 同时必须**剥掉注释**：`.uix` 的注释里常写
/// `:root { --app-accent: #007acc }` 这类示例，照字面匹配会误判为「已定义」。
fn page_defines_var(page_source: &str, name: &str) -> bool {
    let text = strip_comments(page_source);
    for block in root_rule_blocks(&text) {
        if block_defines_var(block, name) {
            return true;
        }
    }
    false
}

/// 剥掉 HTML 注释（`<!-- … -->`）。
///
/// `.uix` 的注释是 SFC 的一部分（sfc_parser 有专门的 skipComment 分支），
/// 注释里的内容**不会**被 CSS 解析器看到。
fn strip_comments(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let bytes = src.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'<' && src[i..].starts_with("<!--") {
            match src[i + 4..].find("-->") {
                Some(rel) => {
                    i = i + 4 + rel + 3;
                    continue;
                }
                None => break,
            }
        }
        // 逐字符推进（按字节，但只关心 ASCII 的 '<'）。
        let ch_len = utf8_len(bytes[i]);
        out.push_str(&src[i..i + ch_len]);
        i += ch_len;
    }
    out
}

/// 取首字节对应的 UTF-8 字符长度。
fn utf8_len(first: u8) -> usize {
    match first {
        0x00..=0x7F => 1,
        0xC0..=0xDF => 2,
        0xE0..=0xEF => 3,
        0xF0..=0xF7 => 4,
        _ => 1,
    }
}

/// 找出所有 `:root { … }` 规则块的内容（不含选择器与花括号）。
fn root_rule_blocks(src: &str) -> Vec<&str> {
    let mut blocks = Vec::new();
    let bytes = src.as_bytes();
    let mut i = 0usize;

    while i < src.len() {
        let rest = &src[i..];
        let Some(rel) = rest.find(":root") else { break };
        let at = i + rel;
        let after = at + ":root".len();

        // 选择器后面必须是空白或 '{'。
        let next = bytes.get(after).copied();
        match next {
            Some(b'{') | Some(b' ') | Some(b'\t') | Some(b'\r') | Some(b'\n') => {}
            _ => {
                i = after;
                continue;
            }
        }

        // 找到 '{'。
        let Some(open_rel) = src[after..].find('{') else { break };
        let open = after + open_rel;

        // 配对 '}'（CSS 块内不嵌套花括号，但保守计数）。
        let mut depth = 0usize;
        let mut j = open;
        let mut close = None;
        while j < src.len() {
            match bytes[j] {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        close = Some(j);
                        break;
                    }
                }
                _ => {}
            }
            j += 1;
        }

        let Some(close) = close else { break };
        blocks.push(&src[open + 1..close]);
        i = close + 1;
    }

    blocks
}

/// 规则块内是否定义了某个自定义属性。
fn block_defines_var(block: &str, name: &str) -> bool {
    block
        .match_indices(name)
        .any(|(idx, _)| {
            let rest = &block[idx + name.len()..];
            // 变量名后面必须跟冒号（允许空白）——否则那是别的标识符的一部分。
            rest.trim_start().starts_with(':')
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> RootVars {
        let mut vars = RootVars::new();
        vars.set("--app-accent", "#007acc").unwrap();
        vars
    }

    #[test]
    fn rejects_non_app_prefixed_names() {
        let mut vars = RootVars::new();
        assert!(vars.set("--accent", "#007acc").is_err());
        assert!(vars.set("--app-", "#007acc").is_err());
        assert!(vars.set("--app-accent", "#007acc").is_ok());
    }

    #[test]
    fn rejects_empty_values() {
        let mut vars = RootVars::new();
        assert!(vars.set("--app-accent", "  ").is_err());
    }

    #[test]
    fn renders_self_closing_rule() {
        let vars = sample();
        let rule = vars.render_rule(None);
        assert_eq!(rule, ":root{--app-accent:#007acc;}");
        // 绝不能出现结束标签 —— 那是约束二。
        assert!(!rule.contains("</style>"));
        assert!(!rule.contains("</root>"));
    }

    #[test]
    fn injects_a_complete_style_block_at_the_front() {
        // 形态甲：在源码最前面插入一个完整的 <style> 块。
        // 约束一：:root 必须包在 <style> 里，裸放顶层会让整页解析失败。
        let vars = sample();
        let page = "<template><style>.a{color:red;}</style><div>x</div></template>";
        let out = vars.inject_into(page).unwrap();

        // 注入的 <style> 必须是源码的第一个块。
        assert!(out.starts_with("<style>"));
        let injected_style = out.find("<style>").unwrap();
        let page_style = out[out.find("<template>").unwrap()..].find("<style>").unwrap()
            + out.find("<template>").unwrap();
        assert!(injected_style < page_style, "injected block must come first");

        // :root 必须在 <style> 之内。
        let rule_at = out.find(":root{").unwrap();
        assert!(rule_at > injected_style);
        let style_close = out.find("</style>").unwrap();
        assert!(rule_at < style_close, "rule must be inside the style block");

        // 页面原有的 rule 一个没丢。
        assert!(out.contains(".a{color:red;}"));
    }

    #[test]
    fn page_own_definition_wins() {
        // 先定义者赢：注入的 rule 排在页面自己的 :root 之前 ⇒ 页面自己的值胜出。
        // 因此同名变量不重复注入（重复注入的那份会被静默忽略）。
        let vars = sample();
        let page = "<template><style>:root{--app-accent:#ff0000;}</style></template>";
        let out = vars.inject_into(page).unwrap();
        assert!(!out.contains("#007acc"), "must not inject a shadowed variable");
        assert!(out.contains("#ff0000"), "the page's own value must survive");
    }

    #[test]
    fn injects_even_without_any_existing_style_block() {
        // 形态甲不依赖页面已有 <style> 块 —— 页面没有时照样能注入。
        // （形态乙在这里直接无从下手，这是选形态甲的理由。）
        let vars = sample();
        let page = "<template><div>x</div></template>";
        let out = vars.inject_into(page).unwrap();
        assert!(out.starts_with("<style>"));
        assert!(out.contains(":root{--app-accent:#007acc;}"));
        assert!(out.ends_with(page));
    }

    #[test]
    fn injected_block_is_well_formed() {
        // 注入的块必须是完整闭合的 <style>…</style>。
        let vars = sample();
        let page = "<template></template>";
        let out = vars.inject_into(page).unwrap();
        assert_eq!(out.matches("<style>").count(), 1);
        assert_eq!(out.matches("</style>").count(), 1);
        assert!(out.find("<style>") < out.find("</style>"));
    }

    #[test]
    fn empty_set_is_a_noop() {
        let vars = RootVars::new();
        let page = "<template></template>";
        assert_eq!(vars.inject_into(page).unwrap(), page);
    }

    #[test]
    fn strips_utf8_bom_before_injecting() {
        // ★ BOM 必须在注入前剥掉：注入是前置的，不剥的话 BOM 会被挤到文件中部
        // → SFC 顶层出现非 '<' 字符 → "unexpected content at top level" → 整页不加载。
        let vars = sample();
        let page = "\u{feff}<template></template>";
        let out = vars.inject_into(page).unwrap();
        assert!(out.starts_with("<style>"), "BOM must not survive at the front");
        assert!(!out.contains('\u{feff}'), "BOM must be gone entirely");
        assert!(out.contains("<template></template>"));
    }

    #[test]
    fn definition_outside_root_rule_does_not_count() {
        // core-ui 只采集 :root 规则里的自定义属性（cascade.cpp:134）。
        // 所以别的选择器里定义了同名变量 **不算** 已定义 —— 那种声明永远不会被采集，
        // 若因此跳过注入，变量就静默解析为空串（界面全是库默认色，零报错）。
        let vars = sample();
        let page = "<template><style>.foo{--app-accent:#ff0000;}</style></template>";
        let out = vars.inject_into(page).unwrap();
        assert!(
            out.contains("#007acc"),
            "must still inject: a non-:root declaration is never collected"
        );
    }

    #[test]
    fn definition_inside_comment_does_not_count() {
        // .uix 的注释里常写示例，照字面匹配会误判为「已定义」→ 漏注入 → 静默回落库色。
        let vars = sample();
        let page = "<template><!-- :root{--app-accent:#ff0000;} --><style>.a{}</style></template>";
        let out = vars.inject_into(page).unwrap();
        assert!(out.contains("#007acc"), "must still inject: the comment is not parsed");
    }

    #[test]
    fn similarly_named_variable_does_not_shadow() {
        // --app-accent-color 不是 --app-accent。
        let mut vars = RootVars::new();
        vars.set("--app-accent", "#007acc").unwrap();
        let page = "<template><style>:root{--app-accent-color:#ff0000;}</style></template>";
        let out = vars.inject_into(page).unwrap();
        assert!(out.contains("#007acc"), "a longer name must not shadow the shorter one");
    }

    #[test]
    fn output_is_deterministic() {
        let mut vars = RootVars::new();
        vars.set("--app-z", "1").unwrap();
        vars.set("--app-a", "2").unwrap();
        let first = vars.render_rule(None);
        let second = vars.render_rule(None);
        assert_eq!(first, second);
        assert!(first.find("--app-a") < first.find("--app-z"));
    }
}
