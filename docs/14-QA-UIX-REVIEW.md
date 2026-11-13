# QA 复核报告：`.uix` 资产与 core-ui 标签修正（第一轮）

> 复核对象：`assets/main.uix`、`assets/preview.uix`、`assets/about.uix`、`assets/themes.json`、`docs/12-UIX-TAG-FIX.md`、`docs/13-UIX-FIRST-RENDER-CHECKLIST.md`、`docs/11-COREUI-MARKUP-TAGS.md`
> 复核依据：上游 core-ui 源码（只读副本 `/tmp/upstream/core-ui-main`，实路径 `C:/Users/tx/AppData/Local/Temp/upstream/core-ui-main`）
> 复核人：QA（严过关）　　日期：2026-11-13
> **本轮只审不改。** 下列"建议改法"由前端在第二轮执行；本报告未修改任何 `assets/**`、`docs/12`、`docs/13`。

---

## 0. 裁决总表

| # | 裁定项 | 裁决 | 一句话 |
|---|---|---|---|
| 1 | `<window>` 自闭合 | **通过** | 三个资产均已自闭合；上游 `sfc_parser.cpp:292-297` 确认非自闭合 → `doc.ok=false` → 整页不加载 |
| 2 | 标签白名单 | **通过** | 三个资产共 6 种控件标签全部命中 `ConstructByTag`；**零遗留未识别标签** |
| 3 | `text=` 属性 | **通过** | 25 处已全部改为元素内部文本；`text` 属性仅 `<menuitem>` 消费 |
| 4 | 选择器 | **通过** | `window {}` 确实不匹配任何控件；class / id / `:root` 均受支持 |
| 5 | 主题 token | **有条件通过** | `--app-` 方案正确且必要；但**不会自动跟随深浅色**，且缺 3 项落地契约 |
| 6 | ID / CLASS 完整性 | **通过** | main 21 / preview 6 / about 0 个 id **一个没少、一个没改**；零 class 丢失 |
| 7 | D-15 静态布局 only | **通过** | 零事件属性、零绑定、零插值；`<script>` 仅含常量 |
| 8 | `docs/11` 与源码一致性 | **有条件通过** | 标签表本身正确；**2 处计数错误** + 3 处可补强 + 2 处小数点漂移 |

**另发现 5 项前端未记录的问题**：P1 ×3、P2 ×3、P3 ×1（见 §3）。

---

## 1. 逐项裁定与证据

### 1.1 `<window>` 自闭合 —— 通过

**上游规则**（`src/ui/uix/sfc_parser.cpp`）：

```
292:            if (!selfClosed) {
293:                pushErr(doc,
294:                        "<window> must be self-closing (use <window .../>)",
295:                        tagLine, tagCol);
296:                skipUntilClose(c, "window", doc);
297:            }
...
373:    if (!doc.errors.empty()) doc.ok = false;
```

**整页失败的传导链**（`src/ui/page/page_api.cpp:249-256`）：

```
249:    auto sfc = ui::uix::ParseSfc(rawText);
250:    if (!sfc.ok) {
251:        std::ostringstream os;
252:        os << "SFC parse failed:\n";
...
255:        return false;
256:    }
```

**资产现状**：三个文件的 `<window>` 均已自闭合 —— `assets/main.uix:60`、`assets/preview.uix:36`、`assets/about.uix:30`。

**交叉验证**：上游 demo `demo/ui_demo.uix:1-4` 的写法确为自闭合 ——

```
<window title="Core UI .uix Demo" width="1100" height="720"
        min-width="700" min-height="500"
        resizable="true" centered="true" frameless="true" theme="light"/>
```

> **路径更正**：本轮任务书写的 `src/ui/page/sfc_parser.cpp` **不存在**；真实路径是 `src/ui/uix/sfc_parser.cpp`。
> 前端 `docs/12` §1 引用的路径**正确**，以它为准。

**结论**：作者的 P0 发现 #1 **成立**，修正到位。

---

### 1.2 标签白名单 —— 通过

**唯一权威**：`src/ui/page/widget_factory.cpp` → `ConstructByTag`（`:913-1194`）+ `ConstructInput`（`:1197-1284`）。

**完整支持表（本次从源码重新提取）**：

| 类别 | 标签 | 落地 |
|---|---|---|
| 容器 | `div` `section` `article` `main` `header` `footer` `aside` `nav` `ul` `ol` | `VBoxWidget`/`HBoxWidget`（按 CSS `flex-direction`，`:918-927`） |
| 容器 | `li` | `HBoxWidget` |
| 文本 | `span` `a` `label` `small` `strong` `em` | `LabelWidget`，wrap=true，**display=Inline**（`:942-953`） |
| 文本 | `p` | `LabelWidget`，wrap=true，**不设 display**（`:955-958`） |
| 文本 | `h1`~`h6` | `LabelWidget` + 固定字号 + 粗体（`:960-971`） |
| 控件 | `button` `hr` `Separator`/`separator` `input` `textarea` `select` `chips-input`/`ChipsInput` `Toggle`/`toggle` `ProgressBar`/`progress-bar`/`progressbar` `Expander`/`expander` `tabs`/`TabControl` `TitleBar`/`title-bar` `ScrollView`/`scroll-view` `custom`/`Custom` `img` `svg` | 见 `:975-1190` |
| 控件 | `gh_img_view`/`gh-img-view`/`gh_img`/`gh-img` | `GhImgViewWidget`（`:1001-1003`） |
| 控件 | `ocr_img_view`/`ocr-img-view`/`OcrImgView`/`ocr-img` | `OcrImgViewWidget`（`:1009-1011`） |
| 特殊 | `link` `meta` `head` `title` | 返回 nullptr → **产生一条非致命编译告警**，不产生控件（`:1104-1109`） |

**资产实测（剥注释 + 剥 style/script 块后统计）**：

| 文件 | 模板内标签 | 未识别标签 |
|---|---|---|
| `main.uix` | `div`×14 `span`×7 `input`×3 `label`×2 `separator`×4 | **无** |
| `preview.uix` | `div`×3 `span`×1 `label`×4 `separator`×1 | **无** |
| `about.uix` | `div`×1 `label`×13 `separator`×2 | **无** |

SFC 外层（`window`/`script`/`style`/`template`）由 `sfc_parser.cpp` 消费，不是控件 ✓。

**重要事实（同时解决了 `docs/11` §4 的 M01）**：未知标签**不会报错、不会崩溃**，而是落到兜底分支 ——

```
1192:    // Default: container
1193:    return std::make_shared<VBoxWidget>();
```

⇒ **标签写错 = 静默变成一个空 VBox 盒子**，页面上表现为"该有文字的地方是空白"。这正是 `docs/13` M01 的答案。

**结论**：作者的 P0 发现 #2（标签不识别）**成立且已修完**；作者在 `docs/12` §1 引用的行号区间 `:913-1194` 与 `:1192-1193` **逐条核对无误**。

---

### 1.3 `text=` 属性 —— 通过

**文本来源**（`src/ui/page/widget_factory.cpp:54-61`）：

```
54: std::string FirstPlainText(const ui::uix::Node& n) {
55:     for (const auto& c : n.children) {
56:         if (c->kind == ui::uix::NodeKind::Text) return c->text;
57:         if (c->kind == ui::uix::NodeKind::Interpolation) return "";
58:         if (c->kind == ui::uix::NodeKind::Element) return "";
```

⇒ 只读**子文本节点**，不读属性。

**`text` 属性的唯一消费点**：全仓 `name == "text"` 只出现在 menuitem 分支 —— `src/ui/page/compiler.cpp:641` / `:644`。
`BuildWidget` 的通用属性循环（`widget_factory.cpp:1306-1334`）只处理 `id` / `hitTransparent` / `hitClient` / `draggable` / `drag-data` / `drop-target` / `visible`，**没有 `text`**。

**资产实测**：三个文件中 `text="..."` **零残留**（脚本 + grep 双查）✓。所有文字已写在元素内部。

> 附带修好了一个隐性 bug：原 main.uix 的 `<script>` 是 `const themeToken = "fluent2";`，
> 而 `src/ui/uix/script_runtime.cpp:189-193` 明确要求 `export default`，否则报
> "missing `export default`" → `AttachQuickJS` 提前 return → **JS 运行时根本不挂载**。
> 新写法 `export default { data() { return { themeToken: "fluent2" }; } }` 才是正确形态。

**结论**：作者的 P0 发现 #3 **成立**，修正到位，且顺带修好了 `<script>` 契约。

---

### 1.4 选择器 —— 通过

**`window { }` 是否匹配根节点：不匹配。** 证据链：

1. `src/ui/css/selector.cpp:53-55` —— 标签选择器只做字符串相等：
   ```
   case SimpleKind::Tag:
       return node.tag == ss.name;
   ```
2. `src/ui/page/compiler.cpp:132-152` —— `MatchNode.tag` 取自**模板节点**的标签名：
   ```
   134:    m.tag = n.tag;
   ```
3. `<window>` 是 SFC 外壳（`sfc_parser.cpp:285`），**永远不会成为模板节点**。

⇒ 模板里不存在 tag 为 `window` 的元素 ⇒ **`window { ... }` 零命中**。作者的 P0 发现 #4 **成立**。

**其余选择器支持度**：

| 选择器 | 支持 | 证据 |
|---|---|---|
| `.class` | ✅ | `selector.cpp:56-57` + `compiler.cpp:142-146`（按空白切分） |
| `#id` | ✅ | `selector.cpp:58-59` + `compiler.cpp:140-141` |
| `:root` | ✅ 但只采集自定义属性 | `cascade.cpp:9-14` / `:18-33`；`:125` 在常规级联里跳过 `:root` |
| 非 `:root` 级的自定义属性 | ❌ 被忽略 | `cascade.cpp:134-135` |
| 标签选择器（`div` 等） | ✅ | `selector.cpp:53-55` |

**资产核对**：`<style>` 里现在全部是 class 选择器，**零 `#id` 选择器、零 `window {}` 选择器** ✓。

> **建议（P2-D，见 §3）**：`assets/main.uix:82` 的 `.tab-active` 用的是**库 token** `var(--accent)`，
> 与其余全部 `--app-*` 混用两个命名空间。该规则当前三重失效（见 §3 P2-D），复活时需一并决策。

---

### 1.5 主题 token —— 有条件通过

#### (a) 改名后，light/dark 两套值到底怎么生效

**机制一（首载）**：用户 `:root` 同名值优先 —— `compiler.cpp:1579-1600` 的 `if (!has("--x"))` 守卫。

**机制二（切模式）**：`RebuildThemeVars`（`compiler.cpp:1647-1704`）**只**擦写 34 个 `kThemeKeys`：

```
1648:    static const char* kThemeKeys[] = {
1649:        "--accent","--accent-hover","--accent-press","--accent-text","--accent-selected",
...      (共 34 项)
1658:    };
1663:    vars.erase(std::remove_if(vars.begin(), vars.end(),
1664:                              [&](const auto& kv) { return isThemeKey(kv.first); }),
1665:               vars.end());
```

**机制三（重算）**：`PageState::RefreshThemeStyles`（`page_state.cpp:688-702`）调用 `RebuildThemeVars` 后遍历控件树重算级联 —— **它只重写这 34 个键，不会重新从样式表采集 `:root`**。

**⇒ 明确结论**：

1. `--app-*` 前缀**正确且必要**。它们不在 `kThemeKeys` 里 ⇒ 不会被库擦除 ⇒ 首次注入的值**永久保留**，不会被库主题接管。前端的判断准确。
2. **但 `--app-*` 不会自动跟随深浅色**。调用 `ui_theme_set_mode(Dark)` 后：
   - 库自绘控件（按钮 / 输入框 / 焦点环 / 勾选框）→ **变深**
   - 本项目自绘区域（条 / 列表 / 面板 / 分隔线 / 文字）→ **保持浅色**
   ⇒ 出现**半深半浅的混搭**。这**不是** bug，是当前方案的必然结果，前端已在 `docs/12` §4.4 如实登记 ✓。
3. 切换的唯一可行路径是**重建页面**（重新拼 `:root` + `ui_page_load_string`），因为 `ui_page_reload` 重放的是**旧源码**，改不了 palette。

#### (b) `ui_theme_set_accent_hex` 与 wipe 行为是否冲突

**不冲突**，且有源码保证：

```
// src/ui/theme.h
408: inline bool& AccentOverrideActive() { static bool b = false; return b; }
409: inline D2D1_COLOR_F& AccentOverrideValue() { ... }
435: inline void SetMode(Mode m) {
436:     CurrentMode() = m;
437:     Current() = (m == Mode::Light) ? MakeLight() : MakeDark();
438:     /* 用户的 accent override 跨 mode 保留, 重新派生即可 */
439:     if (AccentOverrideActive()) ApplyAccent(AccentOverrideValue());
440: }
446: inline void SetAccent(const D2D1_COLOR_F& base) { ... ApplyAccent(base); }
```

C 头文件的对外承诺同样明确 —— `include/ui_core.h:1386`：
> `跨 ui_theme_set_mode 切换深/浅色仍保留 (再调一次深浅切换后还是品牌色)`

⇒ **陷阱在于"保留的是同一个值"**：若 Rust 先 `ui_theme_set_accent_hex("#2563EB")` 再 `ui_theme_set_mode(Dark)`，
accent **仍是浅色的 #2563EB**，不会自动变成深色的 #60A5FA。
⇒ **每次切模式后必须重新调 `ui_theme_set_accent_hex()` 传入目标模式的色值**。
前端在 `docs/13` C3 已写明这一点 ✓，判读正确。

#### (c) 这套方案**还缺什么**（前端未写全的 3 项）

| # | 缺口 | 依据 | 后果 |
|---|---|---|---|
| **缺 1** | **`:root` 块的注入位置没规定** | `sfc_parser.cpp:260-266`：顶层出现非 `<` 开头的内容 → 报 "unexpected content at top level" → `doc.ok=false` → `page_api.cpp:250-256` 整页 load 失败 | **若 Rust 把 `:root{}` 拼在 `.uix` 源码最前面 → 三个窗口一个都出不来**。必须拼进 `<style>…</style>` **块内部** |
| **缺 2** | **重建页面的副作用没登记** | `ui_page_load_string` 返回**新** page；旧 page 的 `ui_widget_find_by_id` 句柄全部失效 | 切主题后必须**重新取 27 个句柄** + 销毁旧 page，否则 Rust 侧拿着悬空句柄写文字 |
| **缺 3** | **注入失败是静默的** | `cascade.cpp:79-81`：`var()` 查不到且无 fallback → 解析为**空串**；`value.cpp:331-338` `ParseColor("")` 返回 false → 该属性不生效 | 注入没做 → 界面**能出来但全是库默认色**（纯白底 + #424242 文字 + #D1D1D1 分隔线），容易被误判成"CSS 没生效"，排查成本高 |

> **无运行时替换 CSS 变量的 C API** —— 已核实 `include/ui_core.h` 全文无 `ui_*css_var*` / `*_set_var*` / `*_set_style*` 一类符号。前端 `docs/12` §4.2 的结论**正确**。

---

### 1.6 ID / CLASS 完整性 —— 通过

**方法**：用 `git show HEAD:assets/*.uix` 取改前版本，与当前版本逐 id / 逐 class 机械比对。

| 文件 | 改前 id | 改后 id | 丢失 | 改名 | 新增 |
|---|---|---|---|---|---|
| `main.uix` | 21 | 21 | **0** | **0** | 0 |
| `preview.uix` | 6 | 6 | **0** | **0** | 0 |
| `about.uix` | 0 | 0 | 0 | 0 | 0 |

逐个 id 拼写完全一致（含大小写）：`titlebar_host` `global_search` `tab_host` `tab_placeholder` `command_host` `nav_hint` `address_host` `breadcrumb` `address` `search` `infobar_host` `nav_host` `list_host` `preview_host` `status_host` `status_item_count` `status_selected` `status_text` `status_progress` `status_thumb_cache` `status_image_channel` / `preview_title` `preview_body` `preview_hint` `preview_channel` `preview_size` `preview_state`。

**class**：**零丢失**。新增 8 个，全部有对应 CSS 规则（无一孤儿）：
`main` +`root` +`tab-placeholder` +`nav-hint` +`vsep`；`preview` +`root` +`divider`；`about` +`root` +`divider`。
反向核对：**元素上带的 class 全部有 CSS 规则**（无孤儿 class）✓。

**元素总数**（模板体，不含 SFC 外壳）：

| 文件 | 改前 | 改后 | 结论 |
|---|---|---|---|
| `main.uix` | 30 | 30 | 不变 ✓ |
| `preview.uix` | 9 | 9 | 不变 ✓ |
| `about.uix` | 16 | 16 | 不变 ✓ |

> 门禁脚本报的 33 / 12 / 19 = 模板元素 + 3 个 SFC 外壳标签（`window`/`style`/`template`）。定义不同、数字都对，但见 §3 P2-C 的措辞问题。

---

### 1.7 D-15 静态布局 only —— 通过

三个文件机械扫描结果：

| 检查项 | 结果 |
|---|---|
| 事件属性（`onClick` / `onChange` / `onInput` / `onKeyDown` / `onMouseDown`） | **0** |
| 指令（`v-model` / `v-if` / `v-for` / `v-show`） | **0** |
| 绑定 / 事件（`:xxx` / `@xxx`） | **0** |
| `{{ }}` 插值 | **0** |
| 业务脚本 | `<script>` 内仅 `export default { data() { return { themeToken: "fluent2" }; } }` —— 一个常量，无业务逻辑 ✓ |

**结论**：D-15 合规。

---

### 1.8 `docs/11-COREUI-MARKUP-TAGS.md` 与源码一致性 —— 有条件通过

#### ✅ 正确的部分（逐条核对无误）

- §1.1 容器表（含"没有 `<hbox>`/`<vbox>`，横排靠 `flex-direction`"）—— 与 `:918-939` 一致
- §1.2 文本表（`span`/`label`/… 同分支 + wrap=true + display=inline；`p` 不设 display；`h1~h6` 字号 28/22/18/16/14/12 + 粗体）—— 与 `:942-971` 一致
- §1.3 控件表 —— 逐个与 `:975-1190` 一致
- §1.4 `link`/`meta`/`head`/`title` —— 与 `:1104-1109` 一致
- §5.1 库 34 个 token 全集 —— 与 `compiler.cpp:1648-1658` **逐字一致**（我数过：5+7+5+2+3+4+2+4+2 = 34 ✓）
- §5.2 三个机制 —— 与 `compiler.cpp:1579-1600` / `:1647-1704` / `page_state.cpp:688-702` 一致
- §5.3 撞名表 —— 与 34 个库 token 比对正确
- §6 "只有 6 个标签名需要机械替换" —— 正确（`text` `hbox` `vbox` `vseparator` `text-input` `panel` 均不在 `ConstructByTag` 中）

#### ❌ 错误 2 处（`docs/11` §2 计数表）

| 表中写的 | 实测（main.uix 改前） | 判定 |
|---|---|---|
| `<text>` × 10 | **7** | ❌ **错** |
| `<text-input>` × 2 | **3** | ❌ **错** |
| `<hbox>` × 7 | 7 | ✓ |
| `<vbox>` × 1 | 1 | ✓ |
| `<vseparator>` × 2 | 2 | ✓ |
| `<panel>` × 4 | 4 | ✓ |
| `<separator>` × 4 | 4 | ✓ |
| `<label>` × 2 | 2 | ✓ |

> **前端已在 `docs/12` §0 指出并更正了这一点**（真实三文件合计为 `text` 25 / `hbox` 8 / `vbox` 3 / `text-input` 3 / `panel` 5 / `vseparator` 2 = **46**，我独立实测全部吻合 ✓）。
> ⇒ **`docs/11` §2 仍需回填这 2 个数字**，否则两份文档互相矛盾。

#### ➕ 可补强 3 处

| # | 项 | 补什么 |
|---|---|---|
| 1 | §1.3 `input` 行 | 补 `type="radio"` / `"range"` / `"number"` / `"password"`（`widget_factory.cpp:1207-1283`） |
| 2 | §4 M01 | 已可从源码结案：**未知标签 → `widget_factory.cpp:1192-1193` → 空 VBox，不报错不崩溃**（`docs/13` M01 已答，需回填 `docs/11`） |
| 3 | §4 M06 | 已可从源码结案：`SeparatorWidget(bool vertical=false)`（`controls.h:200`）存在，但 `ConstructByTag`（`widget_factory.cpp:1059-1061`）**从不传 true**，且全仓无 `vertical`/`orientation` 属性 ⇒ **标记里 `<separator>` 永远水平**（`docs/13` M06 已答，需回填 `docs/11`） |

#### 🔸 小数点漂移 2 处（建议级）

| 位置 | 写的 | 实测 |
|---|---|---|
| `docs/11` §3「`ui_debug_*` 族 71 个」 | 71 | `include/ui_core.h` 中唯一符号 **72** 个 |
| `docs/11` §6「`value.h:91` 提供 Resolve a var(--name) reference」 | value.h:91 | `LookupVar` 声明在 **value.h:93**（`cascade.cpp:79` 调用它） |

---

## 2. 门禁实测（本轮 QA 亲自跑）

```
$ python3 scripts/gates/uix_tag_check.py
  → exit 0，三个资产全部检查通过

$ python3 scripts/gates/theme_contrast_check.py
  → exit 0
```

> **注**：`scripts/gates/` 目前是 **untracked**（`git status` 显示 `?? scripts/gates/`），CI 无法执行这两个脚本 —— 见 §3 P2-B。

**QA 独立复算（G05 真实前景/背景配对）**：沿真实 DOM 树取「元素自身 color × 最近祖先-or-self 的 background」，
逐对算 WCAG 对比度：

| 文件 | 配对 | light | dark |
|---|---|---|---|
| main | `--app-text-secondary` on `--app-toolbar-bg` | 5.78:1 | 6.75:1 |
| main | `--app-text` on `--app-toolbar-bg` | 16.08:1 | 12.99:1 |
| main | `--app-text` on `--app-info-bg` | 15.83:1 | 13.39:1 |
| preview | `--app-text` on `--app-toolbar-bg` | 16.08:1 | 12.99:1 |
| preview | `--app-text-secondary` on `--app-sidebar-bg` | 5.98:1 | 7.31:1 |
| preview | `--app-info-fg` on `--app-sidebar-bg` | 6.30:1 | 7.28:1 |
| about | `--app-text` on `--app-content-bg` | 17.22:1 | 12.99:1 |
| about | `--app-text-secondary` on `--app-content-bg` | 6.19:1 | 6.75:1 |

⇒ **真实配对全部 ≥ 4.5:1（最低 5.78:1）→ G05 通过 ✓**。
⇒ 另：**无任何文字元素缺 `color` 声明**，**无任何文字元素缺有效背景** ✓。

---

## 3. 前端未记录的问题（我独立发现）

### P1-A · `<separator>` 的 CSS `background` 完全无效 —— `docs/12` §2.6 的结论是错的

**证据**（`src/ui/controls.cpp:635-643`）：

```
635: void SeparatorWidget::OnDraw(Renderer& r) {
636:     if (vertical_) {
637:         float cx = (rect.left + rect.right) / 2;
638:         r.DrawLine(cx, rect.top, cx, rect.bottom, theme::kDivider());
639:     } else {
640:         float cy = (rect.top + rect.bottom) / 2;
641:         r.DrawLine(rect.left, cy, rect.right, cy, theme::kDivider());
642:     }
643: }
```

`SeparatorWidget::OnDraw` **没有调用 `Widget::OnDraw(r)`**（对照 `SliderWidget::OnDraw`，`controls.cpp:666` 有调用），
而 `Widget::OnDraw`（`widget.cpp:328-344`）才是绘制 `bgColor` 的地方
⇒ **CSS `background` 设置进了 `bgColor`，但永远不会被画出来**。
线条颜色写死为 `theme::kDivider()`：light = `grey::g82` = **#D1D1D1**，dark = `grey::g26` = **#424242**。

**受影响的资产行**：`assets/main.uix:124`、`:128`、`:134`、`:143`；`assets/preview.uix:62`；`assets/about.uix:46`、`:57`
—— 共 7 个 `<separator class="divider" />`，其 `.divider { background: var(--app-divider); }` 规则**零效果**。

**连带失真的文档结论**：
- `docs/12` §2.6「现：`<separator class="divider" />` + `.divider { background: var(--app-divider); }`（`#E0E0E0`）」→ **不成立**
- `docs/13` L9 排障列「线不见了 → `.divider` 的 `background` 未生效」→ **查错了方向**
- `docs/13` C5 期望「分隔线 `#3D3D3D`」→ **实际是 `#424242`**（库色，不是项目 token）
- `docs/13` §8 把「分隔线 #D1D1D1 vs #E0E0E0」列为 **P2 可记账** → 实际这是**永久状态**，不是偶发

**建议改法（三选一，倾向 a）**：
- **(a) 保留 `<separator>`，接受库色，并改文档说明** —— 库色**随主题自动切换**，比一个静态 token 更合理；只需把 `docs/12` §2.6 与 `docs/13` L9/C5/§8 的表述改成"分隔线用库色 `theme::kDivider()`（light #D1D1D1 / dark #424242），不参与项目 token"。
- (b) 改成 `<div class="divider">` 并补 `height: 1px; flex-shrink: 0;` —— div **会**画背景（`.vsep` 就是这么做的，已验证有效）
- (c) Rust 侧自绘

> `.vsep`（`assets/main.uix:138`、`:140`，CSS 在 `:102`）是 `<div>`，**背景生效** ✓ —— 这条路已经被验证可行。

### P1-B · `font-weight: 600` 是空声明 —— 两个标题不会加粗

**证据**（`src/ui/page/widget_factory.cpp:1392-1394`）：

```
1392:        if (style.Has("font-weight")) {
1393:            const std::string& fw = style.Get("font-weight");
1394:            if (fw == "bold" || fw == "700") lbl->Bold();
```

`600` 落到 else，**什么都不做**。

**受影响**：`assets/preview.uix:43`（`.preview-title`）、`assets/about.uix:35`（`.heading`）。

> **前端已在 `docs/12` §5 R8 与 `docs/13` F6 记录了这一点并留待裁决** ✓ —— 本条不是新发现，是**唯一的待裁决项**，我不替设计师拍板。
> **QA 建议**：采纳 `docs/12` R8 的选项 (c) —— 标题字号提到 14px，靠字号 + 颜色做层级（与 `docs/07` §4「字重仅 400/500」和知识库「字重不许超 500，靠字号和颜色深浅做层级」一致）。
> **同时需修正 `docs/13` F3**：现写「16px、**粗体**」，与 F6「当前预期是不加粗」自相矛盾。

### P1-C · `docs/12` §6 的「执行结果（实际输出，退出码 0）」与脚本真实输出不符

`docs/12` §6 粘贴的输出块与今天跑脚本的真实输出**不一致**：

| 位置 | `docs/12` §6 粘贴 | **实际运行结果** |
|---|---|---|
| main `<div>` | ×21 | **×14** |
| main `<label>` | ×4 | **×2** |
| main `<span>` | ×8 | **×7** |
| preview `<div>` | ×5 | **×3** |
| preview `<label>` | ×5 | **×4** |
| about `<div>` | ×2 | **×1** |

粘贴块的开标签+闭标签混算规则不自洽（main/preview 像 open+close，about 像 open-only），**没有任何单一规则能复现它**。

违反 `references/10-standards/mechanical-gates.md §四.3 只上报真正跑过的命令` —— 复核者无法复现这份"证据"。

**建议改法**：重新粘贴当前真实输出，或**更好**：§6 只写命令 + 判定标准 + 退出码，把输出交给 CI 存档（`contract-prose.md §二.4 有界输出，给指针不给倾倒`）。
**脚本本身没问题** —— 它的断言（旧标签零残留 / D-15 / id 集合 / 元素总数 / 硬编码色）实测全部通过。

### P2-A · G05 门禁验的是 UI 里根本不存在的组合

`theme_contrast_check.py` 报的最低对比度是 `--app-accent` on `--app-selection-bg`（4.51:1），
但 **`--app-selection-bg` 从未被任何 `.uix` 用作背景**。

实测 token 使用率：`themes.json` 定义 17 个，`.uix` 实际引用 10 个。
**定义但未引用**：`--app-hover` `--app-pressed` `--app-selection-bg` `--app-warning-bg` `--app-danger-bg` `--app-warning-fg` `--app-danger-fg`（`--app-accent` 由 Rust 消费喂 `ui_theme_set_accent_hex`，属预期）。

⇒ 门禁在验一对永远不会同框的颜色；真实配对我已复算（§2）**全部 ≥ 4.5:1，结论正确**，
但**这个检查器在未来可能掩盖真实回归**（真实配对变差时，未引用 token 的更差值可能把它顶下来）。

**建议改法**：检查器改为沿 DOM 树取「有效前景 × 最近祖先背景」，只统计 `.uix` 真正引用到的 token。

### P2-B · `scripts/gates/` 未纳入版本控制

`git status` → `?? scripts/gates/`。这两个脚本正是前端自己产出 §6 证据的来源；不提交则 CI 跑不了、证据无法从仓库复现。
**建议**：`git add scripts/gates/`（或按 `spec/crate-layout.md` 归到既有工具目录）。

### P2-C · "元素总数 33/12/19" 的口径会误导

`docs/12` §0「元素总数 **不变**（main 33→33、preview 12→12、about 19→19）——只换标签，不增删元素」
中的 33/12/19 **含 3 个 SFC 外壳标签**（`window`/`style`/`template`）。**模板体真实元素数是 30 / 9 / 16**。
`docs/12` §3 自己的逐标签表列的是 30 行（main）/ 9 行（preview）/ 7 组（about）⇒ **§0 与 §3 自相矛盾**。

**建议改法**：口径写明，或统一改成 30 / 9 / 16。

### P2-D · `.tab-active` 同时踩三个坑

`assets/main.uix:82`：`.tab-active { color: var(--app-text); border-bottom: 2px solid var(--accent); }`

1. `border-bottom` **不受支持** —— 只认 `border` / `border-color` / `border-width` / `border-radius`（`widget_factory.cpp:631-653`）⇒ 整条声明作废
2. `var(--accent)` 是**库 token**，与其余 `--app-*` 混用两个命名空间（库 token 随主题自动切，`--app-*` 不切）
3. **`.tab-active` 在三个资产里没有任何元素挂这个 class** ⇒ 整条规则是死代码

前端已记录第 1 点（`docs/12` §5 R6），**第 2、3 点未记录**。
**建议**：要么删掉这条死规则，要么复活时同时定「用什么 token」+「Rust 侧怎么加 class」。

### P3 · `font-family` 在 `.root` 上是纯设计记录

`font-family` **不在** core-ui 支持的 CSS 属性全集内（实测 **66** 个受支持属性，见 §7），对普通 `LabelWidget` 不生效 —— 前端 `docs/12` §1 与 `docs/13` F1 已如实标注 ✓。
建议在 `.root` 规则旁加一行注释「设计记录，字体由 `ui_theme_set_default_font` 决定」，避免后来者以为它生效。

---

## 4. 必须修的项清单（按三档）

### 🔴 阻塞（不修则第一版界面出不来 / 关键结论失真）

| # | 位置 | 问题 | 建议改法 | 责任方 |
|---|---|---|---|---|
| **B1** | `assets/themes.json` `revision.inject`（`:8`） | **`:root` 块的注入位置未规定**。若 Rust 把它拼在 `.uix` 源码顶层 → `sfc_parser.cpp:260-266` 报 "unexpected content at top level" → `doc.ok=false` → **整页 load 失败，三个窗口一个都出不来** | 注入说明改为：「把 `:root { … }` **拼进 `<style>…</style>` 块内部**（紧跟块首），**不可**放在源码顶层」 | 前端（改文档）+ 后端（按此实现） |
| **B2** | `docs/12` §2.6 / `docs/13` L9 / C5 / §8 | **`<separator>` 的 CSS `background` 无效**，7 处分隔线实际用库色（light #D1D1D1 / dark #424242），文档结论相反 | 按 P1-A 选项 (a) 改文档表述（建议保留 `<separator>`，库色随主题自动切换）；如选 (b) 则 7 处改 `<div>` + `height:1px; flex-shrink:0` | 前端 |
| **B3** | `docs/12` §6 | 粘贴的「实际输出」**无法复现**，违反 `mechanical-gates.md §四.3` | 重贴真实输出，或改为「命令 + 判定 + 退出码」，输出交 CI 存档 | 前端 |

### 🟠 重要（不阻塞渲染，但结论/契约会失真）

| # | 位置 | 问题 | 建议改法 | 责任方 |
|---|---|---|---|---|
| **I1** | `docs/11` §2 计数表 | `<text>` 写 10（实 **7**）、`<text-input>` 写 2（实 **3**） | 改为 7 / 3；或直接改成三文件合计（25 / 3）并标注口径 | 前端（`docs/11` 作者） |
| **I2** | `docs/11` §4 M01 / M06 | 已被源码结案（未知标签→空 VBox；`<separator>` 永远水平），仍标 UNVERIFIED | 回填结论，注明依据行号 | 前端（`docs/11` 作者） |
| **I3** | `docs/12` §4.4 + `docs/13` §6 | **重建页面的副作用未登记**：切主题后 27 个 `ui_widget_find_by_id` 句柄全部失效，旧 page 需销毁 | 在 `docs/12` §4.4 / §5 R4 增一行：「切主题后必须重取全部句柄 + 销毁旧 page」 | 前端（文档）+ 后端（实现） |
| **I4** | `docs/12` §5 R8 + `docs/13` F3/F6 | `font-weight: 600` 待裁决；且 F3「16px、粗体」与 F6「不加粗」自相矛盾 | **需设计师裁决**（QA 倾向 R8 的选项 (c)：提字号到 14px 靠字号+颜色做层级）；裁决后同时改 F3 | 设计师（裁决）+ 前端（改文档） |
| **I5** | `scripts/gates/theme_contrast_check.py` | G05 验的是 UI 里不存在的 token 组合，可能掩盖真实回归 | 改为沿 DOM 树算真实配对（QA 已在 §2 给出算法与结果） | 前端 |
| **I6** | `scripts/gates/` | 未纳入版本控制，CI 跑不了、证据无法复现 | `git add scripts/gates/` | 前端 |
| **I7** | `assets/main.uix:82` | `.tab-active` 用库 token `var(--accent)` 与 `--app-*` 混用；且该 class 无任何元素使用 | 复活时一并定 token 方案；否则删除该死规则 | 前端 |

### 🟡 建议

| # | 位置 | 问题 | 建议改法 |
|---|---|---|---|
| **S1** | `docs/12` §0 | 元素总数口径（33/12/19 含 SFC 外壳）与 §3 自相矛盾 | 写明口径，或统一为 30/9/16 |
| **S2** | `docs/11` §3 / §6 | `ui_debug_*` 写 71（实 72）；`value.h:91`（实 `value.h:93`） | 改数 |
| **S3** | `docs/11` §1.3 | `input` 行漏了 `radio`/`range`/`number`/`password` | 补齐 |
| **S4** | `assets/*.uix` `.root` 规则 | `font-family` 为设计记录，易被误读为生效 | 加注释说明 |
| **S5** | `docs/13` §9 | 未收「根 `<div class="root">` 是否铺满窗口」这条 | 已核：`ui_window.cpp:1533` `root_->rect = {0,0,size.width,size.height}` → 根控件铺满客户端区 ✓，可从待验清单划掉 |

---

## 5. 真机验证时该重点看什么

> 依据 `references/50-design-systems/ui-acceptance.md`（多断点 / 状态矩阵 / 暗色专项）与 `docs/13` 已有清单，本节只列**本轮新增或修正**的重点。

### 5.1 最高优先（能直接证伪 P1-A / P1-B / 缺 1）

| # | 看什么 | 期望 | 说明 |
|---|---|---|---|
| **V1** | **分隔线颜色** | 浅色下**#D1D1D1**（不是设计稿的 #E0E0E0）；深色下 **#424242**（不是 #3D3D3D） | 验证 P1-A。若看到 #E0E0E0，说明我对 `SeparatorWidget::OnDraw` 的判读错了 |
| **V2** | **「预览」标题 / 关于面板标题「tabbed_files 1.2.3」是否加粗** | **不加粗**（当前 `font-weight: 600` 是空声明） | 验证 P1-B。裁决后此项会变 |
| **V3** | **窗口是否出现** | 出现且非白屏 | 若一个窗口都出不来 → 查 B1：`:root` 是否被拼到了源码顶层 |
| **V4** | **窗口底色** | 浅色 **#F3F3F3** | 若是纯白 + 文字 #424242 + 分隔线 #D1D1D1 → `:root` **没注入成功**（静默失败，见缺 3） |
| **V5** | **`ui_page_last_error(page)`** | **空字符串** | 非空时按 `docs/13` E1 的分类定位 |

### 5.2 深浅色切换（本次裁定新增）

| # | 看什么 | 期望 |
|---|---|---|
| **V6** | 切深色**前**先调 `ui_theme_set_mode(Dark)`，观察自绘区 | 仍浅色（`--app-*` 不跟随）—— 这是**预期**，不是 bug |
| **V7** | 切深色**后**的 accent | **仍是 #2563EB**（accent override 跨模式保留的是"同一个值"）→ 必须重调 `ui_theme_set_accent_hex("#60A5FA")` 才是深色蓝 |
| **V8** | 切深色后的分隔线 | **#424242**（库色随主题切），不是 #3D3D3D |

### 5.3 验收纪律（不可省略）

- **每个页面 6 张截图**：`screens/375-dark.png` / `screens/834-light.png` / `screens/1440-light.png` 起步
  （`ui-acceptance.md §七 截图存档 —— 没有截图的 UI 交付，视为未验收`）
- **6 种状态**：加载 / 空 / 错误 / 权限不足 / 超长内容 / 极值
- **暗色五项**：表面分层、正文与次要文字 ≥4.5:1、边框无阴影仍可见、图片不变白块、去掉阴影层级仍可辨
- **浮层三验**：可关闭（点外部 + Esc）/ 在视口内 / 不被裁
- **DPI**：100 / 125 / 150 / 200% 四档下 1px 分隔线可见性（`docs/13` D1）
- **Rust 侧**：切主题重建页面后，**27 个句柄全部重新取到**（对应 I3）

---

## 6. 本轮审查的边界（必读）

> **本机无 Rust / C++ 工具链。** 本轮**没有编译、没有渲染、没有真机加载**任何东西。
> 全部结论为**源码级静态审查**（上游 core-ui 源码只读副本 + 本仓库资产文本 + 机械脚本复算）。
>
> - 本文**不含任何"已通过"的渲染结论**。`docs/13` 的真机清单**尚未执行**，它仍是待办。
> - 本轮实际执行过的命令只有三条：`python3 scripts/gates/uix_tag_check.py`（exit 0）、
>   `python3 scripts/gates/theme_contrast_check.py`（exit 0）、以及 QA 自写的机械比对与对比度复算脚本（§1.6、§2）。
> - 「未识别标签」「选择器不匹配」「CSS 属性不生效」「token 不被擦除」等结论**基于源码阅读**，
>   其**渲染期表现仍需真机确认**（尤其 V1、V2 两条直接证伪/证实 P1-A、P1-B）。
> - `ui-acceptance.md §十 自查`要求的「没有『应该没问题』这种结论」——本文对每条结论都标注了
>   **文件:行号 + 源码行号**，凡无依据处一律写「未核实」，没有以形容词代替数字。

---

## 7. 引用与依据

| 用途 | 依据 |
|---|---|
| SFC 解析 / 自闭合 / 顶层内容限制 | `src/ui/uix/sfc_parser.cpp:260-266, 285-297, 373`；`src/ui/page/page_api.cpp:249-256` |
| 标签支持表 / 未知标签兜底 / `text` 属性 | `src/ui/page/widget_factory.cpp:54-61, 913-1194, 1192-1193, 1197-1284, 1306-1334`；`src/ui/page/compiler.cpp:641, 644` |
| 选择器匹配 | `src/ui/css/selector.cpp:50-79`；`src/ui/page/compiler.cpp:132-152` |
| CSS 级联 / var() / :root / 继承 | `src/ui/css/cascade.cpp:9-33, 35-90, 92-110, 112-172`；`src/ui/css/css_parser.cpp:283-322`；`src/ui/css/value.cpp:331-338` |
| 主题 token 擦除 / 重建 | `src/ui/page/compiler.cpp:1577-1641, 1645-1704`；`src/ui/page/page_state.cpp:688-702` |
| accent 覆盖跨模式 | `src/ui/theme.h:407-456`；`include/ui_core.h:1382-1394` |
| 脚本 `export default` 契约 | `src/ui/uix/script_runtime.cpp:151-199` |
| 分隔线绘制 | `src/ui/controls.h:200-207`；`src/ui/controls.cpp:635-643`；`src/ui/widget.cpp:328-344` |
| 字重语义 | `src/ui/page/widget_factory.cpp:1392-1394` |
| 受支持 CSS 属性全集（66 项） | `src/ui/page/widget_factory.cpp` 中全部 `Has("…")` 取值（`font-family` 与 `border-bottom` 均**不在**其中） |
| 窗口根控件铺满 | `src/ui/ui_window.cpp:1533` |
| 门禁纪律 | `references/10-standards/mechanical-gates.md §二 G02/G05/G13、§四.2、§四.3` |
| 文档写法纪律 | `references/10-standards/contract-prose.md §二.2、§二.4` |
| 验收纪律 | `references/50-design-systems/ui-acceptance.md §三、§五、§七、§十` |
| 知识库路由 | `references/ROUTER.md §一（阶段 4 验收）、§四（读完必须落地一条）` |

**本轮落地的一条**：按 `references/10-standards/mechanical-gates.md §四.3 只上报真正跑过的命令`，
本报告 §2 只贴了三条**实际执行**的命令及其真实输出；`docs/12` §6 无法复现的输出块被判为 **P1-C**；
按 `references/50-design-systems/ui-acceptance.md §七 截图存档`，§5 把三张基准截图列为不可省略的交付项，
并按 `references/10-standards/contract-prose.md §二.4 有界输出，给指针不给倾倒`，超长明细落在本文件，回传给主理人的摘要只给结论 + 位置。

---

## 8. 第二轮复核结论（B2 已确认 · B1/B3/I1–I5 逐条裁定）

> 本节是 QA 在第一轮意见落地后的复跑记录。§1–§7 是第一轮裁定，**未改动**。
> 裁决来源：交付总监（team-lead）第二轮决策 + QA 独立复核。

### 8.1 B2（`<separator>` 的 CSS `background` 无效）—— ✅ 已确认修复

| 项 | 结果 |
|---|---|
| 三个文件的**真实 `<separator>` 元素**（剥 HTML 注释 + CSS 注释后统计） | main **0** / preview **0** / about **0** ✅ |
| 替代写法 | `<div class="divider" />` + `.divider { height: 1px; background: var(--app-divider); flex-shrink: 0; }` ✅ |
| QA 自写门禁 `qa_uix_verify.py` 的对应 WARN | **已消失** ✅ |
| 其余结构性判定（标签 / 旧标签 / 元素数 / id / D-15 / 自闭合 / 硬编码色 / var() / G05） | **全部仍 PASS** ✅ |

**连带修对了 `docs/13` 两处期望值**（原先是错的，改成 `<div>` 后反而成立）：
- M04 ④「分隔线是不是 `#E0E0E0`」→ 现在**正确**（div 会画背景）✅
- C5「深色下分隔线 `#3D3D3D`」→ 现在**正确** ✅

### 8.2 B1（`:root` 注入位置）—— ✅ 点 1 已落地；点 2/3 由构造保证；点 4 尚无落点

定稿四点逐条核对 `assets/themes.json` 的 `revision` 段：

| B1 点 | 核对结果 |
|---|---|
| 1. 拼进 `<style>…</style>` 块**内部**，禁止源码顶层 | ✅ **已落地**。`revision.inject` 已改写为「必须注入到 `<style>` 块【内部】（紧跟 `<style>` 开头），**绝不能**放在源码顶层」，并引用 `sfc_parser.cpp:260-266` 的判据 |
| 2. `:root` 内只允许 CSS 变量声明 | ✅ **由构造保证**。`themes.json` 的 17 个键**全部**是 `--app-*` 自定义属性，无 `background`/`color`/`font` ✅ |
| 3. 根节点的 `background`/`color`/`font-family` 走根 `<div class="root">` 的 class 规则 | ✅ **已满足**。三个文件的 `.root` 规则都带 `background` + `color`（`font-family` 是设计记录，不生效但无害） |
| 4. Rust 侧加载前 **fail-fast 自检**（校验 `:root` 位置在 `<style>` 之内） | ⛔ **尚无落点**。`crates/` 下现为 `zsdesk-app` / `zsdesk-core` / `zsdesk-platform` / `zsdesk-ui` / `zsdesk-windows` —— **没有 `zsdesk-coreui`**（绑定层未落地）→ 建议与 `docs/12` §5 的 R1–R4 同批登记为 Rust 侧待办 |

**另确认**：`revision.switch` 已补上 I3 的句柄失效说明（「旧的 27 个 `ui_widget_find_by_id` 句柄全部失效 → 必须重取全部句柄并销毁旧 page；`ui_page_reload` 会重放旧源码，改不了 palette」）✅

### 8.3 B3（`docs/12` §6 的输出块不可复现）—— ⛔ 仍开着

需前端二选一：重贴当前真实输出，或按 `contract-prose.md §二.4` 改为「命令 + 判定 + 退出码」、输出交 CI 存档。

### 8.4 I1 / I2（`docs/11`）—— ✅ 已回填，QA 复核通过

| 项 | 复核结果 |
|---|---|
| §2 计数表 | ✅ 全部正确（`<text>` 7 / `<text-input>` 3 / `<hbox>` 7 / `<vbox>` 1 / `<vseparator>` 2 / `<panel>` 4 / `<separator>` 4 / `<label>` 2），并已加「只统计开标签」说明 ✅ |
| M01 | ✅ QA 回填：未知标签 → `widget_factory.cpp:1192-1193` → **空 VBox，静默不报错不崩溃** |
| M06 | ✅ 前端已回填，结论与 QA 独立判定**逐条一致**（含 `<separator>` 背景永不生效、线条色写死 `theme::kDivider()`、light `#D1D1D1` / dark `#424242`） |

**🆕 新发现的一处口径冲突**（建议一并回填）：

> 同一个「元素总数」现在有**三个值**：
> - `docs/11` §2 下方：**34 / 12 / 19**（= 全部开标签，含 `window`/`script`/`style`/`template` 四个 SFC 外壳标签）
> - `scripts/gates/uix_tag_check.py`：**33 / 12 / 19**（= 开标签，但剥掉了 `<script>` 块）
> - QA `docs/14` §1.6：**30 / 9 / 16**（= **模板体**元素数，不含 SFC 外壳标签）
>
> 三个值各自自洽，但**没有一处写明口径** → 建议统一采用 **「模板体元素数」= 30 / 9 / 16**（这才是「不增删元素」讨论的对象），并在三处都写明「不含 4 个 SFC 外壳标签」。

### 8.5 I4（字重 600 空声明）—— ✅ 已裁决采纳 (c)，⏳ 待前端落地

**裁决**：采纳选项 (c)（标题删掉 `font-weight`、字号提到 14px、靠字号 + 颜色做层级）。
**依据**：`docs/07-UI-DESIGN-WINUI3.md` §4「字重仅 400/500」，而 core-ui 只认 `bold`/`700`（`widget_factory.cpp:1392-1394`）→ **500 同样是空声明** → 两条路都违反文档 → 靠字号 + 颜色是唯一同时满足「不改用户设计约束」与「core-ui 能力边界」的做法。

**待前端落地的三处**（QA 不自行改）：
1. `assets/preview.uix` `.preview-title`、`assets/about.uix` `.heading`：删 `font-weight: 600`，字号按 `docs/07` 提到 **14px**
2. `docs/13` **F3**（现写「16px、**粗体**」）→ 改成「14px、**不加粗**、靠颜色做层级」
3. `docs/13` **F6**（现写「当前预期是不加粗 …… 是**待裁决项**」）与 **§9 第 8 条**（现写「需要设计师裁决的是要不要改」）→ 改成「**已裁决采纳 (c)**」

**QA 复核状态**：⏳ **未落地**，前端改完后复跑 `qa_uix_verify.py`（`font-weight: 600` 的两条 WARN 应消失）。

### 8.6 I5（G05 验错对象）—— ✅ 已由前端落地，QA 复核通过，数字完全吻合

`scripts/gates/theme_contrast_check.py` 已重写为 **DOM 树遍历口径**（class 规则解析 + 最近祖先-or-self 的 `background` + 库默认兜底），不再是「只对比 themes.json 里的 token 组合」。

**实跑真实输出**：

```
=== G05 双主题对比度（按 .uix 真实渲染关系计算）===
  [light] 真实文字元素 55 处，最低 5.78:1（span.app-title #616161 on #F7F7F7）
        assets/main.uix span.app-title: #616161 on #F7F7F7 = 5.78:1 「tabbed_files」
        assets/main.uix label.nav-hint: #616161 on #F7F7F7 = 5.78:1 「后退 前进 上级 刷新 · 查看 排序 预览」
        assets/preview.uix label.preview-hint: #616161 on #FBFBFB = 5.98:1 「选中单个文件以预览」
  [dark] 真实文字元素 55 处，最低 6.75:1（span.app-title #B3B3B3 on #2B2B2B）
        ...
PASS —— 真实文字元素的对比度全部 ≥ 4.5:1，间距取值全部在刻度上。
```

- ✅ 输出已按要求列出**真实元素 × 背景 × 比值 + 文本片段**
- ✅ **light 5.78:1 / dark 6.75:1 与 QA 独立复算的数字完全一致**（`qa_uix_verify.py` 同口径得出同样的 5.78 / 6.75）
- ✅ 库默认兜底值经核对**正确**：`LabelWidget` 无 `color` 规则时用 `theme::kBtnText()`（`controls.cpp:61-68`）= light **`#242424`**（grey::g14）/ dark `#FFFFFF`；窗口底兜底 light `#FFFFFF` / dark `#292929`
- ✅ G13 间距刻度仍 0 处非刻度

### 8.7 I3（切主题后 27 个句柄失效）—— ⛔ `docs/13` 缺一条验证步骤

**现状**：`docs/13` 的 C1 只写了「必须按 `docs/12` §4.4 **重建页面**（重注 `:root` + `ui_page_load_string`）」，
但**没有任何一条检查重建后句柄是否重取**；H1 只覆盖**首次**加载的 27 个 id。

**建议在 `docs/13` §6（深浅色切换）补一条**（QA 不自行改 docs/13 —— 前端所有）：

| # | 看什么 | 期望 | 异常怎么定位 |
|---|---|---|---|
| **C6** | 切深色**并重建页面之后**，27 个 id（main 21 + preview 6）是否仍全部能 `ui_widget_find_by_id` 取到 | 全部命中 | 取不到 → Rust 侧没在重建后重新绑定句柄，仍拿着旧 page 的悬空句柄 |

### 8.8 撤回我自己的两条 S2（数字类断言必须自己实测）

| 我在第一轮提的 | 复核结果 | 处置 |
|---|---|---|
| 「`docs/11` §3 的 `ui_debug_*` 71 个 → 应为 72」 | **我错了**。实测 `include/ui_core.h` 中 `UI_API` 声明的唯一 `ui_debug_*` 符号恰为 **71** 个（第 72 个只出现在注释里）→ docs/11 写的 71 **正确** | **撤回** |
| 「`docs/11` §6 的 `value.h:91` → 应为 93」 | **我错了**。`value.h:91` 正是 `// Resolve a var(--name) reference…` 注释行，`:93` 是声明行 → 引用 91 **合法**（team-lead 已自行改为 91-93） | **撤回** |

**教训**：数字类断言必须**自己实测**，不能凭印象 —— 这两条都是我凭印象写的，核一下就发现是错的。

### 8.9 一处 `docs/12` 的数字需要更正（P3）

`docs/12` §2.4 写「库默认文字色是 `contentText = #424242`（light，`theme.h` grey::g26）」→ **数字错**。
`LabelWidget` 无 `color` 规则时用 `theme::kBtnText()`（`controls.cpp:61-68`）= **`#242424`**（grey::g14）。
结论方向不变（都偏灰、都比 `--app-text` 浅一档），但数字要改。

---

## 9. CI 契约一致性核对（`spec/ci-contract.md` v3 vs `.github/workflows/build.yml`）

> **范围**：只核对「结果是否符合契约」，**不审实现写法**，**不改 `build.yml`**（交付总监所有）。
> **方法**：逐行读 `build.yml`（452 行）与契约 v3 的「当前态」栏 + §3 判定表 + §5 诊断段，逐条比对。
> **边界**：本次只做**静态核对**；CI 是否已实跑通过**未能核实**（需 CI run 自证）。

### 9.1 结论

> **契约的「目标态」全部成立**（构建结果约束：x64 / `/MD` / 版本匹配 / 导出面 / 同目录 / assets 进包）——
> §5.1 的绕法（直接调 cmake + `-DFETCHCONTENT_SOURCE_DIR_QUICKJS` + `ilammy/msvc-dev-cmd@v1`）**已按 §5.4 的参数清单落地**。
> **但契约的「当前态」栏落后于 `build.yml` 共 8 处**，其中 **1 处是实质冲突（revision 戳记：三方口径不一致）**、
> **1 处是规则与现行资产直接对撞（S13 禁 `--app-accent`）**。这两条必须裁决，其余 6 条是文档同步。

### 9.2 「当前态」栏与 `build.yml` 的 8 处出入

| # | 契约位置 | 契约「当前态」写的 | `build.yml` 实际 | 判定 | 建议 |
|---|---|---|---|---|---|
| **C-1** | 头部第 5 行 | 「**当前 402 行** / 3 个 job」 | **452 行** / 3 个 job | 🟡 过期 | 行数改 452（job 数 3 ✓） |
| **C-2** | §2 快照第 52 行 | 「Pre-fetch QuickJS-NG into the FetchContent slot ← 预置 **`_deps/quickjs-src`**」 | 预置到 **`$RUNNER_TEMP\quickjs`**（`build.yml:184-185`），**在 `clang-cl-build/` 之外**（符合 §5.4 要点 3），经 **`-DFETCHCONTENT_SOURCE_DIR_QUICKJS`** 消费（`build.yml:215`） | 🟡 过期 | 当前态改成新路径 + 官方变量 |
| **C-3** | §3.2 第 12 行 | 「`git clone --depth 1 --branch v0.14.0 **--recurse-submodules**` → `_deps/quickjs-src`」，判定「✅ 这是第 12 步失败的加固」 | **`--recurse-submodules` 已去掉**（`build.yml:185`）→ H2 已落地 | 🟡 过期 | 删掉 `--recurse-submodules`；判定改为「🟡 机制已换、**尚未实跑自证**」 |
| **C-4** | §3.2 第 9 行 | 「已实现（用上游脚本）」，并注「若 H1 成立则改为 CI 直接调 cmake」 | **已直接调 cmake**（`build.yml:210-218`），步骤名 `Configure and build core-ui from source` → **H1 绕法已落地** | 🟡 过期 | 当前态改为「直接调 cmake」；§5.1「障碍与绕法」应标为**已实施** |
| **C-5** | §2 快照 + §3.2 | 未记录 | 三个新事实：**`ilammy/msvc-dev-cmd@v1`** 导入 MSVC x64 环境（`build.yml:99-101`）；**`::error::` workflow command** 把异常做成 check-run annotation（`build.yml:225,233`）；**日志尾部进 step summary**（`build.yml:239-241`） | 🟡 漏记 | 当前态补三项 |
| **C-6** | §3.3 第 17 行 + §4.2 | 当前态「打包拷 `app/target/release/core-ui.dll.revision`（**若存在**）→ **戳记由 Rust 侧 build.rs/构建产出**」；判定「🟡 **戳记的产出方要定**」 | **实际由 CI 的 Gate G2 步骤写出**（`build.yml:286-289`：`Set-Content "$dll.revision"`）→ **永远是 CI 产出、永远存在** | ⛔ **实质冲突** | 见 §9.3 |
| **C-7** | §4.2 契约要求 | 「CI 侧加一条断言『**打包进去的戳记 == `packaging/core-ui-revision.txt`**』」 | **未实现**。包内戳记内容 = `VERSION` + 换行 + `revision`（两行），而 `packaging/core-ui-revision.txt` = 单行 SHA `1840d689…` | ⛔ **断言缺失** | 见 §9.3 |
| **C-8** | §4.4 S13 行 | 「扫描 `assets/*.uix` + `assets/themes.json` 的自定义属性定义 → 命中库 34 个 token 全集即失败；**另禁自定义 `--app-accent`**」 | `assets/themes.json` **定义**了 `--app-accent`（light `#2563EB` / dark `#60A5FA`），且 `revision.decision` 与 `docs/12` §4.3 #8 都明确「accent 保留 `--app-accent` 作为唯一源值，由 Rust 调 `ui_theme_set_accent_hex()`」 | ⛔ **规则与现行资产直接对撞** | 见 §9.4 |

### 9.3 C-6 / C-7 —— revision 戳记的**三方口径不一致**（最需要裁决的一条）

| 来源 | 口径 | 原文要点 |
|---|---|---|
| `spec/ffi-core-ui.md` §5.5 | **甲**（已定） | 「**谁写**：**Rust 侧** `crates/zsdesk-coreui/build.rs` 在**编译期**写出。**CI 不自己写戳记**」／「**写什么**：`packaging/core-ui-revision.txt` 的内容去首尾空白后，**原样写入** + 一个换行；**无 BOM**」／「**CI 做什么**：**只校验**……不等即**打包失败**」 |
| `spec/ci-contract.md` §3.3 行 17 + §4.2 | **未定**（行 17 判定「🟡 产出方要定」；§4.2 说「架构侧推荐：甲」） | 与 §5.5 的「已定」**自相矛盾** |
| `.github/workflows/build.yml` | **乙**（实际实现） | `build.yml:286-289` 在 Gate G2 步骤里 `Set-Content "$dll.revision"`，内容 = `VERSION` + 换行 + `revision`（**两行**） |

**后果（可判定，不是推测）**：

1. **戳记内容与运行时比对口径不一致** —— `ffi-core-ui.md` §5.5 规定运行时的 `EXPECTED_REVISION` 就是 `packaging/core-ui-revision.txt` 的内容（SHA），而包内戳记首行是 `VERSION`（如 `1.9.0.346`）→ `trim()` 后**必然不等** → 每次启动都会报「戳记与编译期常量不一致」的警告（按 §5.5 的运行时语义是**警告、不阻断启动**，所以不会挂，但**自证机制等于失效**）。
2. **契约要求的打包断言缺失** → 没有一道机器门禁能挡住「戳记内容错了」。
3. **编码方面没有 BOM 问题** ✅ —— `shell: pwsh` 用的是 PowerShell 7，`Set-Content -Encoding utf8` **不带 BOM**，符合 §5.5 的「无 BOM」要求。（若 runner 回落成 Windows PowerShell 5.1 才会有 BOM，但 `shell: pwsh` 已显式指定。）
4. **位置方面满足要求** ✅ —— `build.yml` 写到 `core-ui/clang-cl-build/core-ui.dll.revision`，经 artifact 搬运后落到 `app/target/release/`（即 exe 同目录）。

**建议（QA 只给建议，不改实现）**：

- 短期（M1 前，绑定层未落地）：`build.yml` 可以继续写戳记，但**内容应改成 `packaging/core-ui-revision.txt` 的原样内容**（单行 SHA），并**补上契约 §4.2 要求的断言**「包内戳记 trim == `packaging/core-ui-revision.txt` trim，不等即打包失败」；同时在契约 §3.3 行 17 与 §4.2 里把「产出方要定」改成**「当前态：CI 暂代写入（绑定层落地后移交 Rust build.rs）」**，避免两处 spec 各说一套。
- 长期（M1 后）：按 `ffi-core-ui.md` §5.5 口径甲，戳记改由 `crates/zsdesk-coreui/build.rs` 写，CI 只校验。
- **无论哪条口径，都必须有一条机器断言**把「包内戳记 == `packaging/core-ui-revision.txt`」钉死 —— 这是 `mechanical-gates.md §二`「没有执行器的规则只是建议」的直接应用。

### 9.4 C-8 —— S13 的「禁 `--app-accent`」与现行 token 方案对撞（需要裁决）

**事实**：

- 契约 §4.4 的 S13 规则原文：「扫描 `assets/*.uix` + `assets/themes.json` 的自定义属性**定义**（`--name:`，不含 `var(--name)` 引用）→ 命中库 34 个 token 全集即失败；**另禁自定义 `--app-accent`**」。
- `assets/themes.json` 现定义 `--app-accent`（`revision` 段的 `decision` 明写「accent 保留 `--app-accent` 作为唯一源值，由 Rust 调 `ui_theme_set_accent_hex()` 写进库 token `--accent`」）。
- 实测：`--app-accent` **不在**库 34 个 token 里（库的叫 `--accent`）→ S13 的**前半句不会触发**；冲突的是**单独那一句**「另禁自定义 `--app-accent`」。
- 实测 `.uix` 引用了 10 个 token，**`--app-accent` 不在其中** ✅（`assets/main.uix:82` 的 `.tab-active` 用的是**库 token** `var(--accent)`）。

**建议**：把 S13 改成两条可机械判死的规则：

1. **禁「与库 34 个 token 同名」的自定义属性**（含 `--accent` / `--bg` / `--sidebar-bg` / `--fg` …）—— 这才是 S13 真正要防的（切主题时被库接管）。
2. **`--app-accent` 允许存在**，但追加一条断言：**`--app-accent` 不得出现在任何 `.uix` 的 `var()` 引用里**（它只是 Rust → `ui_theme_set_accent_hex()` 的搬运值，不是 CSS 变量）。

### 9.5 `build.yml` 里的两处注释残留（不影响功能，建议清理）

| 位置 | 问题 |
|---|---|
| `build.yml:162-167` | **遗留注释块**仍写「预置该源码到 FetchContent 的默认落点 **`_deps/quickjs-src`**」「**预置失败不阻断：让上游脚本自己走 FetchContent 下载路径**」—— 与 168-179 的新注释、与实际代码（**直接调 cmake**；预置失败 `exit $LASTEXITCODE` 阻断）**三处矛盾**。建议整块删除 |
| `build.yml:394-396` | 注释说「Rust 必须在加载前把 `:root` 拼进 `.uix` 源码」→ **没写「拼进 `<style>` 块内部」**，与 B1 定稿（§8.2）不一致。建议补一句，避免实现方拼到源码顶层 |

### 9.6 契约 §4.4「4 个脚本都不存在」→ 现状需记一笔

契约 §4.4 的 4 个脚本**仍不存在**（`check_crate_dag.py` / `check-coreui-api.py` / `verify.py` / `gates_engineering.py`）→ §3.5 第 28–30 行的 ⛔ 判定**不变** ✅。
但 `scripts/gates/` 现在**已存在**且有 3 个 `.uix` 专项门禁（`uix_tag_check.py` / `theme_contrast_check.py` / `qa_uix_verify.py`），**尚未接进 CI** → 建议在契约当前态记一笔。

### 9.7 未能核实的（诚实边界）

- **本版「直接调 cmake」路径是否已实跑通过** —— 本次只做静态核对，**没有 CI run 结果可查**，无法确认。契约 §5 记录的两次失败都发生在**旧路径**上。⇒ **这一条必须由 CI run 自证**（run 号 + 全绿日志），不能靠文档措辞。
- `build.yml` 的 `::error::` annotation 是否真能在**外部公开 API** 读到（契约 §3.2 行 13 只要求「日志可取」）—— 未实跑，未核实。

---

## 10. 协作纪律（本轮新增，team-lead 与 QA 共同遵守）

> **任何人不得对「他人正在编辑的文件」做批量改写**（`sed -i` / 全文替换 / 格式化工具 / 批量重写）。
> 确需改动，**先 SendMessage 协调**，或**只改自己产出的文件**。

**缘起**：本轮 QA 在 `assets/about.uix` 上做负控测试时误用了 `sed -i`，而前端同时在改该文件。虽经逐行核对**无内容损失**（其 B2 修改完整保留、测试残留零命中），但流程不可接受 —— 竞态下**任何一次还原都可能覆盖对方的改动**。

**落地细则**：

| # | 规则 | 落地方式 |
|---|---|---|
| 1 | QA 的**负控测试一律在临时副本**上做 | 复制资产到临时目录 → `python3 scripts/gates/qa_uix_verify.py --root <副本目录>` → 副本用完即弃。**绝不碰工作区文件** |
| 2 | 改他人文件前**先 SendMessage 协调** | 协调结论写进消息，双方确认后再动 |
| 3 | 被授权后**只做最小、可定位的定点编辑** | 用 `Edit` 精确替换单行/单段，不整文件重写；改完立刻回读校验（本次 M01 回填即按此做：改完 grep + 逐行核对，确认无乱码） |
| 4 | **QA 的门禁脚本全程只读** | `qa_uix_verify.py` 与 `theme_contrast_check.py` **零写操作**（已实测：无 `open(...,'w')` / `write_text` / `shutil` / `os.remove`）→ 可放心对工作区跑 |
| 5 | 数字类断言**必须自己实测** | 不得凭印象写数字（见 §8.8 的两条撤回） |

---

## 11. 第三轮复核（前端回应后的验收）

> 依据：前端在 `docs/12` §8「对 QA 第一轮复核的逐条回应」中声明 3 阻塞 + 7 重要 + 5 建议全部处理。
> QA 逐条**复跑门禁 + 机械比对 + 上游源码核对**后的验收结果。§1–§10 未改动。

### 11.1 已验收（8 项，机械复核通过）

| 项 | 验收证据（本次实跑） |
|---|---|
| **B1** 注入位置 | ✅ `assets/themes.json` 新增 `revision.inject_failure_modes`（**3 条失败模式**：顶层注入 → SFC parse failed / 完全没注入 → 静默回落库色 / 注入晚于 load → 同上）；`inject` 已写明「必须注入到 `<style>` 块【内部】…绝不能放在源码顶层」；`docs/13` §0 新增 **4 条【P0】自查**（位置 / 确实注入 / 时序 / 句柄重取）✅ |
| **B2** `<separator>` | ✅ **三个文件真实 `<separator>` 元素归零**（剥 HTML + CSS 注释后 main **0** / preview **0** / about **0**；文件里剩下的 1 处/文件全在**变更记录注释**与 CSS 说明注释里）。`<div class="divider">` = 4 / 1 / 2 = **7 处** ✅ |
| **B3** 输出复现 | ✅ `docs/12` §6 已重写为仓库脚本的真实输出 + 断言清单 + 判定标准；实跑 `uix_tag_check.py` → EXIT=0，输出与文档一致 ✅ |
| **I1** 计数 | ✅ `docs/11` §2 已是 main.uix 单文件实测口径；`docs/12` §0 补了「三文件合计 46」与「模板体 30/9/16」 ✅ |
| **I2** M01/M06 | ✅ M01 已结案（QA 回填）；M06 前端已结案，结论与 QA 独立判定逐条一致 ✅ |
| **I3** 句柄失效 | ✅ `docs/12` §4.4 新增「重建页面的副作用」三步（destroy 旧 page → 重取 27 个句柄 → 重挂回调）；`themes.json` 的 `revision.switch` 同步；`docs/13` §0 加了【P0】自查第 4 条 ✅ |
| **I5** 对比度口径 | ✅ **55 个真实文字元素 × 2 主题 = 110 对**，最低 **5.78:1 (light) / 6.75:1 (dark)** —— 与 QA 独立复算**完全一致**；G13 间距 0 处越界 ✅ |
| **I6** 入库 | ✅ `git add scripts/gates/ docs/12 docs/13` 已执行（前端自述 status 为已暂存）—— 未经 QA 独立核验（**本轮不执行 git 命令**） |
| **S1** 口径统一 | ✅ `uix_tag_check.py` 已改成「**模板体元素 30 个**」（main 30 / preview 9 / about 16），与 `docs/11` / `docs/12` / `docs/14` 一致 ✅ |

**额外机械核验（本次 QA 独立跑）**：
- **ID 保全**：main **21/21**、preview **6/6**、about **0/0** —— 丢失 0、改名 0、新增 0 ✅
- **模板体元素数**：30 / 9 / 16 —— 与改前一致 ✅
- **无新增缺陷**：`qa_uix_verify.py` 的 WARN 集合与上一轮**完全相同**（8 条：border-bottom / font-family ×3 / 死 class ×5 / font-weight ×2），**没有因本轮改动引入任何新问题** ✅
- **前端引用的上游佐证 —— 部分属实**：✅ `demo/ui_demo.uix` 确有 `<div class="sep"/>`（实测 **19 处**）+ `:203` 的 `.sep { height: 1; background: var(--border-subtle); }` → **这是支持 B2 改法的强证据**，上游作者自己也用 `div + background` 做分隔线 ✅
- ✅ **前端对 `ui_page_reload` 的判断正确** —— QA 独立核对 `page_api.cpp:565-577`：reload 走的是 `CompileAndAttachPage(*e, e->sourceText)`（或重读 `e->sourcePath`）→ **改不了 palette**，且重新 attach → **句柄同样失效** ✅

### 11.2 ⛔ X-1 · I4 裁决**未落地**（执行与裁决不一致 —— 需 team-lead 拍板）

**裁决内容**（team-lead 第二轮决策）：采纳选项 (c) —— `.preview-title` / `.heading` 的 `font-weight: 600` **删掉**，字号按 `docs/07` 提到 **14px**，层级靠颜色区分；`docs/13` 的 F3 与 F6 一并改成「**14px、不加粗、靠颜色做层级**」。

**前端实做**（QA 本次实跑）：

| 位置 | 现状 | 与裁决的差距 |
|---|---|---|
| `assets/preview.uix:52` | `.preview-title { padding: 8px; **font-weight: 600**; color: var(--app-text); white-space: nowrap; }` | `font-weight: 600` **未删**；该规则本就无 `font-size` → **未提 14px** |
| `assets/about.uix:44` | `.heading { display: block; padding: 12px; **font-size: 16px**; **font-weight: 600**; color: var(--app-text); }` | `font-weight: 600` **未删**；字号仍是 **16px**（未提 14px） |
| `docs/13:134` F3 | 「16px、**不加粗**（`font-weight: 600` 在 core-ui 是空声明，见 F6）」 | 矛盾**已消除** ✅，但不是裁决要的「**14px**、不加粗、靠颜色做层级」 |
| `docs/13:137` F6 | 仍写「……**是待裁决项**」 | **未按裁决更新** |
| `docs/13:197` §9 第 8 条 | 仍写「**需要设计师裁决的是「要不要改」**」 | **未按裁决更新** |

前端自述：「**R8 三选一保持未裁决，等设计师**」。

**机器信号**：`qa_uix_verify.py` 的两条 `font-weight: 600` WARN **仍在**（`assets/preview.uix` / `assets/about.uix`）—— 这正好是「裁决未落地」的自动检测。

> **需要 team-lead 拍板**（QA 不越权裁决、也不替前端改）：
> **(甲) 补落地 (c)** —— 删两处 `font-weight: 600`、`.heading` 字号 16px→14px、`.preview-title` 补 `font-size: 14px`、`docs/13` F3/F6/§9 第 8 条改成「已裁决采纳 (c) / 14px / 不加粗 / 靠颜色做层级」；
> **(乙) 改裁决** —— 若 `docs/07` 的 16px 标题字号是硬约束，则 (c) 的「提到 14px」反而**降级**了标题（16→14 是变小），此时应改裁为「保持 16px、去掉 `font-weight: 600`、靠颜色做层级」。

**✅ QA 已复核 `docs/07` 原文，(乙) 不成立 —— (甲) 是唯一正解**：

`docs/07-UI-DESIGN-WINUI3.md:95` 逐字：
> **字体**：Segoe UI Variable（Win11）/ Segoe UI（Win10），正文 13 px、次要 12 px、**标题 14 px**；**字重仅 400 / 500**。中文回退 Microsoft YaHei UI。

⇒ **16px → 14px 不是「降级」，是「修正到用户自己的设计文档」** —— 现在 `.heading` 的 **16px 本身就违反 `docs/07` 的「标题 14 px」**。

**两个标题的当前实际渲染字号**（QA 核对 `controls.h`：`LabelWidget::fontSize_` 默认 = `theme::kFontSizeNormal` = **14.0f**，`theme.h:584`）：

| 元素 | 当前声明 | 实际渲染字号 | 与 `docs/07`「标题 14 px」 |
|---|---|---|---|
| `.preview-title`（`preview.uix:52`） | **无** `font-size` | 库默认 **14px** | ✅ **恰好符合**（但依赖库默认值，库一改就漂移 → 建议显式写 `font-size: 14px`） |
| `.heading`（`about.uix:44`） | `font-size: 16px` | **16px** | ❌ **违反** |

⇒ **(甲) 的具体动作**（可直接照做）：
1. `assets/about.uix:44` `.heading`：`font-size: 16px` → **`font-size: 14px`**，并**删掉** `font-weight: 600`
2. `assets/preview.uix:52` `.preview-title`：**删掉** `font-weight: 600`，并**显式补** `font-size: 14px`（别依赖库默认值）
3. `docs/13:134` F3 → 「**14px、不加粗**，靠颜色做层级」；`docs/13:137` F6 + `:197` §9 第 8 条 → 「**已裁决采纳 (c)**」

⇒ 落地后 `qa_uix_verify.py` 的两条 `font-weight: 600` WARN 会**同时消失**（机器信号闭环）。

### 11.3 ⛔ X-2 · 前端引用的上游 demo 证据有一处事实错误

| 前端说法 | QA 实测 | 判定 |
|---|---|---|
| 「`demo/ui_demo.uix:426` 等 **10 处**全是 `<div class="sep"/>`」 | `grep -c 'class="sep"'` = **19** 处 | 🟡 数字偏小（不影响结论） |
| 「`<separator>` 在 demo 里**一次都没出现**」 | **3 处真实 `<separator/>`**：`demo/ui_demo.uix:663, 672, 687` | ❌ **不实** |
| 「demo 用 `.sep { height: 1; background: var(--border-subtle); }`」 | ✅ `demo/ui_demo.uix:203` 逐字属实 | ✅ 成立 |

**影响**：**不推翻 B2**（demo 那 3 处 `<separator/>` 仍然只能渲染成库色，与 §1 的结论一致），
但会误导读者以为上游从不用 `<separator>`。建议改成：「demo 里 **19 处用 `<div class="sep"/>`** 做分隔线、另有 **3 处 `<separator/>`**（那 3 处只能是库色）」。

### 11.4 ⛔ X-3 · `docs/11` 新写的 `ui_debug` 口径第二个数字不对

| 口径 | 前端写在 `docs/11` 的 | QA 实测 | 判定 |
|---|---|---|---|
| 导出声明数 | 「71 个」 | `grep -c "UI_API.*ui_debug_"` = **71** | ✅ **正确** |
| 全文出现次数 | 「全文出现该前缀 **73** 处含 2 处注释」 | 全文出现 **83 行**；唯一符号名 **72** 个 | ❌ **73 对不上任何一种口径** |

建议改成：「**71 个导出声明**（`grep -c "UI_API.*ui_debug_"` = 71）；全文出现该前缀 **83 行**，其中唯一符号名 **72** 个（1 个仅见于注释）」。

### 11.5 ⛔ X-4 · `themes.json` 的 `inject_failure_modes` 复制了 QA 指出的错数字

`revision.inject_failure_modes` 第 2 条写：「……界面能出来但全是库默认色（纯白底 / **#424242** 文字 / #D1D1D1 分隔线）」

→ **`#424242` 应为 `#242424`** —— `LabelWidget` 无 `color` 规则时用 `theme::kBtnText()`（`controls.cpp:61-68`）= **`#242424`**（grey::g14）。
这正是 §8.9 已指出的那处错数字，**被原样复制进了新字段**。（`#D1D1D1` 分隔线那半句现在也要改：本轮分隔线已全改 `<div>`，深浅色下分别是 `--app-divider` 的 **#E0E0E0 / #3D3D3D**，不再是库色。）

### 11.6 本轮未能核实的（诚实边界）

- **`git add` 是否真的执行了** —— 按 team-lead 要求本轮**未执行任何 git 命令**，I6 的 status 属前端自述，**未经 QA 独立核验**。
- **`docs/12` §8 的逐条回应内容是否与我的裁定一致** —— QA 只核了其中与机械可判项相关的部分（§8.1–§8.5 已列出）；散文部分未逐字校对。
- **真机渲染** —— 仍未渲染验证；`docs/13` 真机清单**尚未执行**。

---

## 12. 第四轮：FFI 契约 v3（D15 · F1–F5）交叉验证 + 三个 Rust 侧 P0 静态定性

> 依据：`DECISIONS.md` **D15**（架构师 FFI v3 §9.4 的五条口径）+ **D14**（品牌色来源）+ **D16**（戳记/S13 裁决）。
> 方法：逐条回上游 core-ui 源码（只读副本）取证，**不用对方的结论做前提**。
> 范围：本轮**未改任何他人文件**（`assets/**` 前端 / `spec/**` 架构师 / `build.yml` 交付总监）、**未执行 git 命令**。

### 12.1 六点逐条结论

#### 12.1.1 F1 vs 我的 B1 —— ✅ **完全一致**，并补一条**「静默丢弃」的精确机制**

| 断言 | 取证 | 结论 |
|---|---|---|
| `:root` 必须拼进 `<style>` 块内部 | `sfc_parser.cpp:260-266`：顶层非 `<` 内容 → `pushErr` + `doc.ok=false` → `page_api.cpp:250-256` 返回 false | ✅ 与 B1 逐字一致 |
| `:root` 内只允许 `--app-*` 变量声明 | `cascade.cpp:120-131` 的 `ComputeStyle`：`if (IsRootSelector(sel)) continue;` → **整条 `:root` 规则被跳过**，普通属性**永远不会被应用**；`cascade.cpp:26-30` 的 `CollectRootVars` 只收 `property[0]=='-' && property[1]=='-'` 的属性 | ✅ 一致 |

**回答 team-lead 的关键问题**：`:root` 里写 `background` / `color` / `font` ⇒ **静默丢弃**（不是报错、不是警告）。
`ComputeStyle` 压根不看 `:root` 规则的普通声明，`CollectRootVars` 也只挑 `--` 开头的 → **无任何诊断输出**。

⇒ **Rust 侧自检应检查什么**（可机械判死，两条）：
1. **位置**：生成的 `:root` 块必须落在 `<style>…</style>` **内部**（正则确认 `<style` 与 `:root` 的下标关系）
2. **内容**：`:root` 块内**每一条声明**都必须匹配 `--app-[a-z0-9-]+\s*:` —— 出现任何非 `--app-*` 声明即 fail-fast（否则它会被静默吞掉，表现为「颜色没生效」这种极难定位的现象）

#### 12.1.2 F2 vs 我的 I3 —— ✅ **一致**，且「两套值由 Rust 侧持有」的理解**正确**

`page_state.cpp:688-702` 的 `RefreshThemeStyles`：
```
690:    if (!page_.cssVars) return;
691:    RebuildThemeVars(*page_.cssVars);      ← 只 erase + 重写库那 34 个键
...     遍历控件树重算级联（recomputeStyle 读同一份 cssVars）
```
⇒ **它不重新采集 `:root`** ⇒ `--app-*` 的值在切模式后**原封不动** ⇒ CSS 自己**永远不会**响应深浅色。

**确认 team-lead 的理解**：
> 「深浅两套值必须由 Rust 侧持有、切换时选一份注入，而不是靠 CSS 自己响应。」

✅ **完全正确**。具体形态：`themes.json` 的 `light` / `dark` 两段由 Rust 侧内嵌持有 → 切主题时 Rust **选一段** → 拼进 `:root` → 注入 `<style>` → `ui_page_load_string` 重建。
⇒ 补一条实现要点：**CSS 侧没有「深浅色选择器」这回事** —— `.uix` 里**不允许**出现 `.dark` / `[data-theme=dark]` 之类的分支（core-ui 的选择器匹配里没有主题态位，`selector.cpp:60-64` 的伪类只认 hover/pressed/focus/disabled/checked/root/drag-over）。

#### 12.1.3 F3 —— ✅ **口径一致**；销毁 API 名 = **`ui_page_destroy`**；但「是否必须先解绑回调」的答案比预想的更精细

**销毁 API（team-lead 记不住的那个符号）**：

```
include/ui_core.h:1911   UI_API void ui_page_destroy(UiPage p);
src/ui/page/page_api.cpp:561-563
    UI_API void ui_page_destroy(UiPage p) { ui::page::Registry().Erase(p); }
src/ui/page/page_api.cpp:59-62   void Erase(uint64_t id) { …; entries_.erase(id); }
```
⇒ `ui_page_destroy(p)` → `PageRegistry::Erase(p)` → **`entries_.erase(id)` 销毁 `PageEntry`** → `PageState` 析构 → **该页的控件树（`CompiledPage::root`）随之释放**。
⇒ **不销毁会泄漏**：`page_api.cpp:414-422` 的 `DestroyAllPages` 明确打印「leaked page(s) not destroyed by host」。

**「销毁前是否必须先解绑回调」—— 精确答案（三层）**：

| 层 | 是否需要手工处理 | 取证 |
|---|---|---|
| **库内回调**（widget 的 `onClick` / `onDraw` / `ud` 等 `std::function` 成员） | **不需要** —— 它们是 widget 的成员，随 widget 一起析构 | `controls.h` 的 `std::function` 成员 |
| **句柄表** | ⚠️ **需要** —— `ui_page_destroy` **不碰句柄表** | `HandleTable::Insert` 存的是 `shared_ptr<Widget>`（`handle_table.cpp:5-10`）；`ui_page_destroy` 只 erase `PageEntry` |
| **宿主的 `ud` 指针** | ⚠️ **需要** —— 见下 | `ui_widget_destroy(w)` → `Ctx().handles.Remove(widget)`（`ui_api.cpp:1886-1888`） |

**⇒ 真实失败模式不是 use-after-free，而是「静默 no-op」**：
`ui_widget_find_by_id` 会把找到的 widget 的 `shared_ptr` **插入句柄表**（`ui_api.cpp:1897-1898`）⇒ 销毁页面后，那 27 个 widget **对象仍然活着**（被句柄表强引用），只是**脱离了控件树**。
⇒ 于是 Rust 若继续用旧句柄写文字：**不崩溃、不报错、不打日志** —— `ui_label_set_text(old_handle, "x")` 成功，但**屏幕上什么都不变**（widget 不在任何窗口的内容树里）。
⇒ **这正是 F3「必须重新绑定」的硬理由**：不重绑的代价是**静默失效**，比崩溃更难发现。

**建议的销毁顺序**（比「先解绑回调」更准确）：
1. 停止使用旧句柄
2. 对 27 个旧句柄调 **`ui_widget_destroy(old_handle)`** → 从句柄表移除 → widget 与其回调、宿主的 `ud` 引用一起释放（**顺带解决句柄表泄漏：每次切主题泄漏 27 个句柄**）
3. `ui_page_destroy(old_page)` → 释放控件树与该页的 `PageState`（含 QuickJS 运行时，`page_state.h:23` 有 `~PageState()`）
4. `ui_page_load_string(new_source)` → 新页
5. 重新 `ui_widget_find_by_id` 全部 27 个 id 并**重新注册回调**

#### 12.1.4 F4 —— ✅ 与 F2/F3 **不矛盾**

- 字体是**进程级全局**，不在 `Colors` 里：`theme.h:498+` 的 `DefaultFontFamily()` / `TextSettings`，由 `ui_theme_set_default_font` / `ui_theme_set_cjk_font` 写
- `theme.h:435-440` 的 `SetMode` **只**替换 `Current()`（Colors）+ 重派生 accent 变体 ⇒ **不动字体**
⇒ **切主题 = 重建页面 + 重新注入 `:root` + 重新绑定句柄；字体一步都不用动** ✅ 与 F2/F3 完全自洽。

#### 12.1.5 F5 —— ⚠️ **结论正确，但因果表述有误**（建议改措辞）

**F5 原文**：「`ui_theme_set_accent_hex` **不随** `ui_theme_set_mode` 保留生效」⇒ **按字面是错的**。

**取证**（`src/ui/theme.h`）：
```
408: inline bool& AccentOverrideActive() { static bool b = false; return b; }
409: inline D2D1_COLOR_F& AccentOverrideValue() { … }
435: inline void SetMode(Mode m) {
436:     CurrentMode() = m;
437:     Current() = (m == Mode::Light) ? MakeLight() : MakeDark();   ← accent 回到库默认
438:     /* 用户的 accent override 跨 mode 保留, 重新派生即可 */
439:     if (AccentOverrideActive()) ApplyAccent(AccentOverrideValue());  ← ★ override 被重新应用
440: }
446: inline void SetAccent(const ui::D2D1_COLOR_F& base) { … ApplyAccent(base); }
```
⇒ **override 跨模式「保留生效」是真的**，且 C 头文件也这么承诺（`ui_core.h:1386`：「跨 `ui_theme_set_mode` 切换深/浅色仍保留（再调一次深浅切换后还是品牌色）」）。
⇒ **真正不成立的是「不会按新模式换一个基色」**：`ApplyAccent(AccentOverrideValue())` 是**用同一个基色**重新派生 hover/press/text/selected 变体（`theme.h:417-433`），**基色仍是上一次的**。

**⇒ 准确表述（建议替换 F5 的因果句）**：
> 「accent override **跨模式保留**：切到深色后**基色仍是上一次那个值**（`SetMode` 只用它重新派生 hover/press/text 变体），**不会按新模式换一个基色** ⇒ 要拿到新模式的强调色，**必须重读系统 AccentColor 并重新调用 `ui_theme_set_accent_hex()`**。」

**⇒ 实操结论与 D14 第 4 步完全一致**（「切换深浅色时重读并重调，并在重建页面前完成」）✅ —— 只是**理由不同**：
不是「被清空了要补回来」，而是「**留的是旧值，要换成新值**」。这个差别会影响实现者的排查方向，值得写准。

**前端 `themes.json` 的措辞复核**（team-lead 要求）：

| 位置 | 现状 | 复核 |
|---|---|---|
| `revision.decision` | 「accent 是唯一的例外：它**不自定义**，由 Rust 侧读系统强调色后调 `ui_theme_set_accent_hex()` 写入【库 token】`--accent`；`.uix` 里一律读 `var(--accent)`（不要读 `var(--app-accent)`）… `--app-accent` 仅作【读不到系统色时的回退源值】」 | ✅ **与 D14 逐字兼容**，措辞准确 |
| `revision.switch` | 「…库自绘控件另走 `ui_theme_set_mode()`。」 | 🟡 **缺一句**：切主题时必须**重读 AccentColor 并重调 `ui_theme_set_accent_hex()`**，否则 accent 停留在旧模式的基色。建议补 |
| `docs/12:304` | 「accent override 跨模式保留（`theme.h:435-439`）」 | ✅ **属实**（与 `ui_core.h:1386` 一致），但**不完整** —— 建议补「保留的是同一个基色；要新模式的强调色必须重调」 |
| `docs/12:336` | 「切 mode 会用当前 accent override **重新派生**…」 | 🟡 「重新派生」只对**变体**成立（hover/press/text/selected），**基色不变**。建议改成「用同一基色重新派生变体」 |

#### 12.1.6 三个 Rust 侧 P0 —— **静态可定 2 个 / 必须探针 1 个**

| # | P0 | 静态可定性 | 结论 |
|---|---|---|---|
| **P0-1** | `ui_page_load_string` 的所有权约定 | ✅ **可静态确定** | 见下 |
| **P0-2** | `ui_page_set_json` 的所有权与线程约束 | 🟡 **所有权可定、线程约束不可定** | 见下 |
| **P0-3** | `ui_custom()` 的精确签名 | ✅ **可静态确定** | 见下 |

**P0-1 · `ui_page_load_string` —— 静态可定（含一条头文件与实现不符的坑）**

```c
/* include/ui_core.h:1907-1911 */
/* Load a .uix page (SFC: <window> + <template> + <script> + <style>) from
 * file or string. Returns 0 on failure. */
UI_API UiPage   ui_page_load_file(const wchar_t* path);
UI_API UiPage   ui_page_load_string(const char* html_source);
UI_API void     ui_page_destroy(UiPage p);
```
```cpp
/* src/ui/page/page_api.cpp:553-559 */
UI_API UiPage ui_page_load_string(const char* html_source) {
    if (!html_source) return 0;
    auto entry = std::make_unique<ui::page::PageEntry>();
    entry->sourceText = html_source;                 // ← std::string = const char* ⇒ 【拷贝】
    ui::page::CompileAndAttachPage(*entry, entry->sourceText);   // ← ★ 返回值被丢弃
    return ui::page::Registry().Insert(std::move(entry));
}
```
⇒ **可静态确定的三条**：
1. **入参不转移所有权**：`entry->sourceText = html_source` 是 `std::string` 赋值 ⇒ **拷贝** ⇒ 调用方**保留**缓冲区所有权，调用返回后**可立即释放**（栈上临时串安全）
2. **返回值是 `UiPage` 句柄**，必须用 **`ui_page_destroy(p)`** 释放（否则 `DestroyAllPages` 会报「leaked page(s) not destroyed by host」）
3. **只有 `html_source == nullptr` 才返回 0**

⚠️ **★ 头文件与实现不符（必须写进 FFI 契约）**：头文件说「**Returns 0 on failure**」，但实现里 **`CompileAndAttachPage` 的返回值被丢弃** ⇒
**SFC 解析/编译失败时，仍然会返回一个非 0 的 page 句柄！**
⇒ **Rust 侧不能用「句柄 == 0」判断失败**。可靠的失败信号是：
- **`ui_page_root(p) == 0`**（失败时 `e.state` 为空 → `page_api.cpp:644-650` 返回 0）✅ **推荐**
- 或 **`ui_page_last_error(p)` 非空**（`ui_core.h:2032`；`page_api.cpp:385` 的注释逐字写着「load **返回值仍是成功**，但开发期/CI 能立刻发现」）

**P0-2 · `ui_page_set_json` —— 所有权可定；线程约束必须探针**

```c
/* include/ui_core.h:1940-1952 */
/* ui_page_get_json 返回 malloc 出的 UTF-8 JSON, 调用方用 ui_page_free 释放. */
UI_API int      ui_page_set_json(UiPage p, const char* name, const char* utf8_json);
UI_API char*    ui_page_get_json(UiPage p, const char* name);
UI_API void     ui_page_free    (void* ptr);
```
```cpp
/* src/ui/page/page_api.cpp:683-694 */
UI_API int ui_page_set_json(UiPage p, const char* name, const char* utf8_json) {
    auto* e = ui::page::Registry().Get(p);
    if (!e || !e->state || !name || !utf8_json) return -1;
    ui::expr::Value v;  std::string err;
    if (!ui::expr::ParseJson(std::string(utf8_json), v, err)) { …; return -1; }  // ← std::string(...) 拷贝
    e->state->SetValue(name, std::move(v));
    return 0;
}
```
⇒ **所有权可静态确定**：
- **入参**：`utf8_json` 被 `std::string(utf8_json)` **拷贝** ⇒ 调用方保留所有权、**无需释放**；`name` 同理（`const char*` → `std::string` 形参拷贝）
- **返回值**：`0` = ok，`-1` = 解析错误**或** 参数/page 无效（实现里两种情况都返回 -1，**头文件只说了「-1=parse error」** ⇒ 建议契约补一句「-1 也包含 page/参数无效」）
- **出参**：`ui_page_get_json` 返回 **malloc 的 UTF-8 JSON** ⇒ **必须用 `ui_page_free(ptr)` 释放**（头文件逐字说明）✅

⇒ **线程约束：头文件未声明 ⇒ 必须探针**。取证：`ui_core.h` 里唯一的「所有 API 仅 UI 线程」在 **OCR 段落内**（`:904`，作用域是 OCR 那一节），**不是全局声明**；Page API 段落**没有任何线程说明**。
但从实现可推断**强烈建议只在 UI 线程调**：`ui_page_set_json` → `PageState::SetValue` → 会**改写控件树与 QuickJS 响应式状态**，而 `PageRegistry` 虽有 `std::mutex`（`page_api.cpp:49/55/60`），**锁的只是句柄表，不是控件树** ⇒ **非 UI 线程并发调用 = 数据竞争**。
⇒ **裁决建议**：契约里写「**必须在 UI 线程调用**」，并由真机探针**实测确认**（从非 UI 线程调一次，看是否崩/是否有断言）。

**P0-3 · `ui_custom()` —— 静态可定（签名逐字）**

```c
/* include/ui_core.h:1415-1423 */
UI_API UiWidget ui_custom(void);
UI_API void ui_custom_on_draw(UiWidget w, UiCustomDrawCallback cb, void* ud);
UI_API void ui_custom_on_mouse_down(UiWidget w, UiCustomMouseCallback cb, void* ud);
UI_API void ui_custom_on_mouse_move(UiWidget w, UiCustomMouseCallback cb, void* ud);
UI_API void ui_custom_on_mouse_up(UiWidget w, UiCustomMouseCallback cb, void* ud);
UI_API void ui_custom_on_mouse_wheel(UiWidget w, UiCustomWheelCallback cb, void* ud);
UI_API void ui_custom_on_key_down(UiWidget w, UiCustomKeyCallback cb, void* ud);
UI_API void ui_custom_on_char(UiWidget w, UiCustomCharCallback cb, void* ud);
UI_API void ui_custom_on_layout(UiWidget w, UiCustomLayoutCallback cb, void* ud);
```
```cpp
/* src/ui/ui_api.cpp:2605-2610 */
UI_API UiWidget ui_custom(void) {
    auto w = std::make_shared<ui::CustomWidget>();
    UiWidget h = Reg(w);  w->apiHandle = h;  return h;
}
```
⇒ **可静态确定**：
- **签名逐字**：`UiWidget ui_custom(void)` —— **无参数**、返回一个 widget 句柄
- **它创建的是一个「孤儿」widget**（`make_shared<CustomWidget>()` + 注册进句柄表），**不挂进任何树** ⇒ 必须自己调 **`ui_widget_add_child(parent, child)`**（`ui_core.h:1083`；实现 `ui_api.cpp:1868-1875` → `p->AddChild(c)`）才能上屏
- **`ui_custom_on_draw` 只对 `CustomWidget` 生效**：`ui_api.cpp:2612-2616` 的 `CUSTOM_SET_CB` 宏先 `As<ui::CustomWidget>(w)`，**取不到就静默 return** ⇒ 给别的 widget 挂这些回调**静默无效**
⇒ **契约可写死**：「`ui_custom()` 无参 → 返回孤儿 widget；必须 `ui_widget_add_child` 挂到宿主；8 个 `ui_custom_on_*` 只对 `CustomWidget` 生效，挂错类型静默 no-op。」
⇒ **仍需探针**：挂载后**重绘触发时机**（`ui_custom_on_draw` 在什么条件下被调用、窗口不可见时是否跳过、多次挂载是否重复绘制）—— 头文件未说明 ⇒ **维持 UNVERIFIED**。

### 12.2 分歧清单（4 条 —— 全部是**表述/精度**问题，无一条推翻结论）

| # | 位置 | 现在的说法 | 问题 | 建议 |
|---|---|---|---|---|
| **Δ1** | `DECISIONS.md` D15 的 F5 | 「`ui_theme_set_accent_hex` **不随** `ui_theme_set_mode` 保留生效」 | **按字面与 `theme.h:435-440` + `ui_core.h:1386` 相反** —— override 确实保留并被重新应用 | 改为「**跨模式保留同一个基色**；`SetMode` 只用它重新派生变体，**不换基色** ⇒ 切主题必须重读 AccentColor 并重调」 |
| **Δ2** | `DECISIONS.md` D15 F5 的括注 | 「（保留的是同一个值，但不会自动重新应用）」 | 「不会自动重新应用」也不准确 —— `SetMode` 里**确实**又 `ApplyAccent` 了一次 | 同 Δ1：「保留的是同一个**基色**；不会按新模式**换基色**」 |
| **Δ3** | `docs/12:336` | 「切 mode 会用当前 accent override **重新派生**」 | 「重新派生」只对**变体**成立，基色不变 | 改成「用同一基色重新派生 hover/press/text 变体」 |
| **Δ4** | `assets/themes.json` `revision.switch` | 「…库自绘控件另走 `ui_theme_set_mode()`。」 | **缺 D14 第 4 步**：切主题时必须重读 AccentColor 并重调 `ui_theme_set_accent_hex()`，否则 accent 停在旧模式基色 | 补一句 |

**⇒ 无实质分歧**：D15 的 F1 / F2 / F3 / F4 与我的 B1 / I3 / I3 **逐条同向且互证**；F5 的**结论**同向、**理由表述**需修正。

### 12.3 仍需真机探针的清单（不能静态定性的部分）

| # | 待探针项 | 为什么静态定不了 | 最小探针 |
|---|---|---|---|
| **R1** | `ui_page_set_json` 的**线程约束** | Page API 段落**无线程声明**；`ui_core.h:904` 的「仅 UI 线程」是 **OCR 段落内**的局部声明 | 从工作线程调一次 `ui_page_set_json`，看是否崩 / 有无断言 |
| **R2** | `ui_custom()` 的**重绘触发时机** | 头文件未说明 `on_draw` 的调用条件 | 挂一个 `on_draw` 计数，打印到日志，确认什么触发 |
| **R3** | `ui_custom()` 挂载后的**是否需 `ui_widget_destroy`** | 句柄表语义已知，但「孤儿 widget 的正确释放序列」无文档 | 反复创建/销毁 1000 次，观测句柄表增长 |
| **R4** | `ui_page_load_string` **失败时**的完整行为 | 已静态确定「返回非 0 句柄」，但失败态的 `ui_page_root` / `last_error` 具体取值要实测 | 喂一个坏 `.uix`，打印三者 |
| **R5** | 切主题重建后**旧句柄的行为** | 已静态推出「静默 no-op」，但要实测确认**确实不崩**且**确实无变化** | 切一次主题，对旧句柄 `ui_label_set_text`，看日志与画面 |

### 12.4 CI 契约一致性复查（D16 裁决后的现状）

- ✅ **`build.yml` 的旧方案遗留注释块已清理**（我上轮 §9.5 指的 `162-167` 那块「预置到 `_deps/quickjs-src` / 预置失败不阻断」已不存在）
- ⚠️ **`build.yml` 已涨到 479 行**（我上轮核对时 452 行 → 第 5 次 CI 增加了逐行 annotation）
- ⛔ **`spec/ci-contract.md` 的「当前态」栏尚未同步** —— D16 说「由架构师同步」，但实测仍是我上轮指出的 5 处过期：
  - 头部第 5 行仍写「当前 **402** 行」→ 实际 **479** 行
  - §2 快照第 52 行仍写「预置 **`_deps/quickjs-src`**」→ 实际 `$RUNNER_TEMP\quickjs` + 官方变量
  - §3.2 第 12 行仍写「**`--recurse-submodules`** → `_deps/quickjs-src`」+ 判定「✅ 这是第 12 步失败的加固」→ 实际已去掉 `--recurse-submodules`
  - §3.2 第 9 行仍写「已实现（用上游脚本）」→ 实际已直接调 cmake
  - §2 / §3 仍未记录 `ilammy/msvc-dev-cmd@v1`、`::error::` annotation、step summary 三项
  - （§5 的 H1/H2 **诊断段**里出现 `_deps/quickjs-src` / `--recurse-submodules` 是**正确的** —— 那是历史诊断记录，不是当前态。**只有 §2 / §3 两栏需要改**。）
- ⇒ **本轮我只报告，不改**（`spec/**` 是架构师的）。

---

## 13. 第五轮：架构师 D15 四口径 + D13 交叉验证

> 依据：架构师来函（`:root` 注入的**四口径** + 规则 5 + **D13** 已知限制口径 + S13 按文件类型区分）。
> 方法：**逐条回上游 core-ui 源码取证**，不用来函结论做前提；**未改 `spec/**`（架构师所有）**。

### 13.1 四口径逐条核对

| 口径 | 架构师所述 | QA 独立取证 | 判定 |
|---|---|---|---|
| **1 · 位置** | 形态甲＝源码末尾追加完整 `<style>` 块；形态乙＝插进已有 `<style>` 块内部（要求该块是**最后一个**） | ✅ **形态甲合法**：`sfc_parser.cpp:321-330` 的 `<style>` 分支是 `doc.styles.push_back(std::move(blk))` —— **无 duplicate 检查**（对比 `<window>`/`<template>`/`<script>` 都有）⇒ **多个 `<style>` 块允许** ✓<br>✅ **形态乙需已有 `<style>`**：三个资产都有 ✓ | ✅ **位置部分成立**；但「**最后一个**」这半句 **⛔ 方向错**（见 §13.2 Δ-A） |
| **2 · 只允许 CSS 变量** | `background`/`color`/`font-family` **静默丢弃** | `cascade.cpp:125` 的 `if (IsRootSelector(sel)) continue;` → **整条 `:root` 规则被跳过**；`cascade.cpp:26-30` 的 `CollectRootVars` 只收 `--` 开头 ⇒ **零诊断输出** | ✅ **完全成立**（引用的 `:124-126` 区间准确） |
| **3 · 根样式走 `.root`** | 根 `<div class="root">` + `.root { … }` | 与我 §1.4 一致；三个资产的 `.root` 规则都带 `background` + `color` ✓ | ✅ **成立** |
| **4 · fail-fast 自检** | Rust 侧 load 前断言每个 `:root` 落在某个 `<style>` 区间内，不满足在 Rust 侧直接报错 | ✅ 与我 §12.1.1 一致；**V1.12 已落** `spec/milestones.md:129`（「构造坏源码 → Rust 侧报错」）✓ | ✅ **成立**；🟡 但自检**只查位置不查内容**（见 §13.3 Δ-B） |
| **规则 5 · 键集** | `themes.json` 当前模式的**全部 `--app-*` 键，减去 `--app-accent`** | ✅ `themes.json` 17 个 `--app-*` → **注入 16**；当前三个 `.uix` 实际引用的 **9** 个 token（`.tab-active` 删除后）**全部**在这 16 里、**没有一个**是 `--app-accent` ✓；16 个与库 34 个 token **零同名** ✓ | ✅ **完全成立** |

### 13.2 ⛔ Δ-A（最要紧）—— 口径 4 与形态乙的「最后一个 `<style>`」**方向与机制相反**

**机制真相**（这是 CSS 级联常识在这里**不适用**的地方）：

| 环节 | 源码 | 语义 |
|---|---|---|
| 收集 | `cascade.cpp:18-33` `CollectRootVars` | 按**文档顺序**把 `:root` 的自定义属性推进 `vars` 向量 |
| 查值 | `value.cpp:572-581` `LookupVar` | `for (auto& kv : vars) { if (kv.first == name) { out = kv.second; return true; } }` ⇒ **返回第一个命中** |
| 佐证 | `cascade.h:41` 注释逐字 | 「Collect all :root variables from stylesheet. **(Returns first set encountered.)**」 |
| 级联 | `cascade.cpp:125` | `:root` 规则**被整条跳过** ⇒ **根本不进 CSS 级联** |

⇒ **自定义属性之间是「先定义者赢」，不是「后者覆盖前者」。**
（`cascade.h` 的注释与 `compiler.cpp:1587-1600` 的 `if (!has("--accent"))` 一起解释了为什么「用户 `:root` 值赢库默认」：用户项在向量里排在库默认**之前** ⇒ first-wins ⇒ 用户赢。**同一个机制，同一个方向。**）

**⇒ 口径 4 的推理「注入位置要靠后 ⇒ Rust 的值生效」不成立**：

| 场景 | 若 `.uix` 里将来出现 `:root { --app-bg: … }` | 结果 |
|---|---|---|
| **形态甲**（Rust 的 `:root` 追加在**末尾**） | `.uix` 的 `:root` 在**前** | **`.uix` 的值赢** ⇒ Rust 的注入被**静默忽略** ⇒ 切主题后颜色不变 ⇒ **静默失效** |
| **形态乙**（插进**最后一个** `<style>`） | 同上，`.uix` 的 `:root` 更早 | 同上，**仍然输** |
| 真正要的 | 注入必须**早于**任何已有 `:root` | Rust 的值赢 ✓ |

**⇒ 今天没有暴露**：实测三个 `.uix` 的**生效** `:root` 规则数 = **0**
（`main.uix` 那 3 处 `:root` 命中**全在 HTML 注释里**：`assets/main.uix:45,46,75`；剥注释后 = 0）⇒ 形态甲/乙 现在都能工作。
**但这正是我一直在抓的那类「静默失效」陷阱** —— 一旦有人往 `.uix` 里加一条 `:root`，注入会被无声吃掉，表现为「切主题颜色不变」。

**⇒ 建议改法（三处）**：
1. **口径 4 改为**：「注入的 `:root` 必须是源码中**第一个** `:root`**（先于任何其他 `:root`）」，并把理由改成「`LookupVar` 取**第一个**命中；`ComputeStyle` 跳过 `:root` ⇒ 自定义属性**不进 CSS 级联**，不存在『后者覆盖前者』」
2. **形态乙改为**：插进**第一个** `<style>` 块**的开头**
3. **形态甲改为**：追加 `<style>` 块，**前提是源码中此前无 `:root`** —— 由 Rust 自检断言
4. **Rust 自检加第 3 条断言**：「源码中**所有** `:root` 的出现位置**都在**注入块之后」⇒ 配合一个验收项（V1.12 目前只测了「裸放顶层」这一种坏情形，**测不到**「`.uix` 自带 `:root` 抢先」这一种）

### 13.3 🟡 Δ-B —— 两条补强（不推翻结论）

**Δ-B1 · D13 的关键词清单里 `backdrop` **不是**零命中**

实测 `backdrop` 有 **11 处**命中，分两类：
- ① CSS 属性 **`backdrop-filter` / `backdrop-blur` 在任意 widget 上受支持** —— `widget_factory.cpp:606-610` → `w.backdropBlur = parseBlur(...)`；`widget.h:245` `float backdropBlur = 0.0f; // CSS backdrop-filter: blur(...)`
- ② 上下文菜单的背景模糊捕获 —— `context_menu.cpp` 的 `backdropBlurRadius_` / `backdropResourceKey_`

**窗口级** Mica/Acrylic 确实零命中 —— `DWMWA_SYSTEMBACKDROP` / `DWMWA_MICA` / `SetWindowCompositionAttribute` / `ACCENT_POLICY` **全仓零命中** ✓

⇒ **结论成立**（窗口材质不可实现），但**关键词证据要改口径**：把 `backdrop` 从「零命中」清单移出，改列为「**widget 级模糊可用（CSS `backdrop-filter`）；窗口级 Mica/Acrylic 零命中**」。
（否则读者会以为「连 widget 级模糊都没有」，而实际上 `.uix` 里写 `backdrop-filter: blur(8px)` 是**受支持的** —— 这对将来做浮层很有用。）

**其余零命中项我逐个复核过，全部属实** ✓：`uia` / `UIAccessibility` / `accessibility` / `narrator` / `HighContrast` / `AppsUseLightTheme` / `mica` / `Mica` / `acrylic` / `Acrylic` 各 **0 个文件命中**；`DwmSetWindowAttribute` 仅用于 `DWMWA_TRANSITIONS_FORCEDISABLED`（`ui_window.cpp:304/464/523`）与 `kDwmWindowCornerPreference`（`:1200` 圆角偏好）—— **与架构师所述逐字一致** ✓

**Δ-B2 · Rust 自检只查「位置」、不查「内容」**

口径 4 的自检三步（解析 `<style>` 区间 → 断言 `:root` 落在区间内 → fail-fast）**没有**覆盖「`:root` 内的声明是否只有 `--app-*`」。
⇒ 建议**加第 4 条断言**：`:root` 块内**每条声明**都必须匹配 `--app-[a-z0-9-]+\s*:`，出现任何非 `--app-*` 即 fail-fast。
**为什么值得加**：非变量声明是**静默丢弃**（口径 2 已证），Rust 侧不拦 → 症状是「某些颜色没生效」，现场极难定位；拦在拼接阶段只需一条断言。
⇒ 配套验收项可并入 V1.12（或新增 V1.13）。

### 13.4 D13 对我第一~四轮结论**无影响**

- 我提过的全部 P0/重要项都是 **`.uix` 语法 / 契约类**：`<window>` 自闭合、标签白名单、`text=` 属性、`window{}` 选择器、主题 token 擦除、id 保全、D-15 静态布局、CI 契约 —— **没有一条是材质 / 高对比度 / 读屏** ✓
- 特别澄清一条可能被误并的：**我的 G05 对比度检查是「文本 vs 背景的 WCAG 对比度（light/dark 两套主题）」**，与 **Windows 高对比度模式（High Contrast）是两回事** ⇒ 不会被误归入「已知限制」，也不受 D13 影响 ✓
- 唯一沾边的一条是 `docs/13` 的 F1（字体）与 L1（分隔线可见性），均属**普通主题下的可读性**，不是高对比度模式 ⇒ 仍在验收范围内 ✓

### 13.5 ✅ S13 按文件类型区分 —— 与我的 §9.4 建议逐条一致

`spec/ffi-core-ui.md:932`「禁止定义 `--app-accent`」+ 规则 5「注入键集排除它」+ `themes.json` 作**回退源值**（非 CSS 变量）三者自洽 ✓；与 D16 裁决「themes.json 允许 / `.uix` 禁止 / 任何文件禁止库 token 全集」一致 ✓；
且**实测支持**：`themes.json` 的 17 个键与库 34 个 token **零同名** ✓；`.uix` 引用集里**无 `--app-accent`** ✓。

### 13.6 本轮 QA 侧状态

- 四个门禁 **EXIT=0**；QA WARN **5 条**（3 × `font-family` 设计记录 + 4 条死规则/Rust 自绘参照，**无一条是缺陷**）
- **I4 (c) 已落地**（`font-weight: 600` 删除、标题 14px）、**`.tab-active` 已删除**、**`<custom id="list_host">` 已落**、元素数仍 30/9/16、**id 21/6/0 全保全**
- 边界不变：仍为**静态审查** —— 未编译、未渲染、未真机加载；`docs/13` 真机清单尚未执行。

---

## 14. 第六轮：前端 X-2 / X-3 / X-4 验收 + 口径归一

> 依据：前端第三轮回执（X-2/X-3/X-4 已改完，请求复跑）。方法：复跑门禁 + 机械比对 + 回上游源码取证。

### 14.1 ✅ X-2 —— 验收通过

`docs/12:126` 已改为「demo 的水平分隔线**主力是** `<div class="sep"/>` + `.sep { height: 1; background: var(--border-subtle); }`（`demo/ui_demo.uix:203` 定义规则，**19 处**）—— ⚠️ **更正**：我第一轮写「demo 里 `<separator>` 一次都没出现」**不实** —— demo 另有 **3 处真实 `<separator/>`**（`:663, 672, 687`），**那 3 处只能是库色**（印证而非反例）」。

⇒ **措辞准确、且比我的原话更有说服力**（把「反例」主动改成「印证」）✓。`docs/12:499` 的更正记录也在 ✓。

### 14.2 ✅ X-3 —— 验收通过，**且口径归一**（我与前端的差异 = 正则量词，两边都到 71）

前端给的三条命令我**逐条复现**：

```
grep -c "UI_API.*ui_debug_" include/ui_core.h          → 71   （导出声明数）✓
grep -c "ui_debug_" include/ui_core.h                    → 83   （提到该前缀的行数）✓
grep -o "ui_debug_[a-zA-Z0-9_]*" | sort -u | wc -l      → 73   （唯一符号名，* 量词）
```

**我上轮报的是 72**，原因是**我用的正则量词是 `+` 不是 `*`**：

| 量词 | 唯一符号名 | 差异来源 |
|---|---|---|
| `ui_debug_[a-zA-Z0-9_]+`（我上轮） | **72** | `ui_debug_*` 这个占位：`*` 不在字符类里、`+` 又要求至少 1 个字符 ⇒ **完全不匹配** |
| `ui_debug_[a-zA-Z0-9_]*`（前端） | **73** | `ui_debug_*` 被截成 `ui_debug_` ⇒ 多出 1 个 |

`diff` 实测：两个集合**只差一个** `ui_debug_` ✓。
⇒ **两边减掉各自的注释占位后都等于 71**，**真符号数 = 71**（`UI_API` 声明数也是 71）✓ —— **两个口径都对，不是矛盾。**

⇒ **采纳前端的措辞**（`docs/11:116` 现写：「71 个导出声明；全文 83 行提到该前缀；若按唯一符号名统计会得到 73 个 —— 其中 2 个是注释里的前缀占位（`ui_debug_*` 与 `ui_debug_post_*`）… **真符号仍是 71 个**」）：
**它把三条命令都写了出来，任何人可复现**，比我只给结论更耐用 ✓。**我上轮 §8.8 里「唯一符号名 72 个」这个数字作废**，以本节为准（结论「71」始终未变）。

### 14.3 ✅ X-4 —— 验收通过，**且前端的推理比我原来的说法更准**（已回源码核实）

前端把「没注入」的现场改成「**白底 + #242424 文字 + 分隔线直接消失**」，并解释「分隔线已是 `<div>`，`--app-divider` 未定义 → `background: var(--app-divider)` 解析失败 → **画不出来**（不是回落成某个颜色）」。

**QA 回源码逐条核实**：

| 现象 | 取证 | 判定 |
|---|---|---|
| **分隔线直接消失**（不是回落成某色） | `background: var(--app-divider)` → `ResolveVars`（`cascade.cpp:79-81`）查不到且无 fallback → 解析为**空串** → `ParseColor("")` 返回 false（`value.cpp:331-338`）⇒ **`bgColor` 从未被设置** ⇒ `<div>` 透明 ⇒ 1px 线在白底上看不见 | ✅ **完全正确** |
| **文字回落 #242424** | `widget_factory.cpp:625`：`if (parseColor(s.Get("color"), c)) { w.css.hasFg = true; … }` ⇒ **解析失败时 `hasFg` 保持 false** → `controls.cpp:66-68`：`if (css.hasFg) … else if (customColor_) … else c = theme::kBtnText();` ⇒ **回落到库默认文字色** = `theme::kBtnText()` = **#242424**（grey::g14，light） | ✅ **完全正确** |
| **窗口底纯白 #FFFFFF** | `.root` 的 `background: var(--app-bg)` 同样解析失败 ⇒ 根控件透明 ⇒ 露出窗口默认底（light = white） | ✅ **正确** |

⇒ **前端的版本比我的原版精确**：我把「分隔线回落成库色 #D1D1D1」写进了 `themes.json` 的失败模式，那是**分隔线还是 `<separator>` 时**的现象；**改成 `<div>` 之后它确实会「消失」而不是回落** —— 前端按新实现重算，**是对的** ✓。
`docs/13:28`（§0 自查）与 `docs/13:76`（M04 异常定位）已同步 ✓；`docs/13:199` 也已把「不再是库色 #D1D1D1 / #424242」写明 ✓。

### 14.4 ⛔ 形态甲的一个**补充前提**（不是要改回形态乙，而是要补一条断言 + 改一句措辞）

**背景**：前端把 `docs/12` §4.2 与 `themes.json` 的 `revision.inject` 统一成**形态甲**（末尾追加完整 `<style>` 块），并注明「我第一轮写的『紧跟既有 `<style>` 开头』（形态乙）也合法，但要求该 `<style>` 是**最后一个** `<style>` 块…**Rust 不依赖这个巧合**，两边口径已统一为甲」。

**QA 立场（与 §13.2 Δ-A 同源，但收敛为最小改动）**：

- **形态甲作为「实现策略」是好的** —— Rust 不需要解析已有 `<style>` 块，**更简单、更不易错** ⇒ **不建议改回形态乙** ✓
- 但**「必须是最后一个 `<style>`」这半句的依据是反的**：`LookupVar` 取**第一个**命中（`value.cpp:572-581`；`cascade.h:41`「Returns first set encountered」），且 `:root` **不进 CSS 级联**（`cascade.cpp:125`）⇒ 自定义属性是**先定义者赢**。
  ⇒ 形态甲真正需要的前提**不是**「注入块在最后」，而是「**源码中此前没有任何 `:root` 规则**」。
  ⇒ **今天这个前提成立**：实测三个 `.uix` 的**生效 `:root` 规则数 = 0**（`main.uix` 那 3 处命中全在 HTML 注释里）✓
- ⇒ **建议改两处**（措辞，不改策略）：
  1. `docs/12` §4.2 与 `themes.json` `revision.inject` 里「形态乙…要求该 `<style>` 是**最后一个** `<style>` 块」→ 改成「形态乙需插在**任何已有 `:root` 规则之前**（`LookupVar` 取第一个命中，`:root` 不进 CSS 级联 ⇒ **先定义者赢**，不是『后者覆盖前者』）」
  2. **Rust 自检加第 3 条断言**：「注入的 `:root` 必须是源码中**第一个** `:root`」—— 这条同时覆盖形态甲（前提：此前无 `:root`）与形态乙（插在第一个 `<style>` 开头）
  ⇒ 配套验收项建议并入 **V1.12**（现只测「裸放顶层」一种坏情形，**测不到**「`.uix` 自带 `:root` 抢先」这一种）

> **本条已在 §13.2 Δ-A 同步发给架构师**（那封函给的是完整论证）；此处是给前端的**最小改动版**——你们不需要改策略，只需要把「最后一个」的依据换成正确的、并在自检里加一条断言。

### 14.5 本轮 QA 侧状态（复跑结果）

| 项 | 结果 |
|---|---|
| 三个门禁 | **全部 EXIT=0** ✓ |
| QA WARN | **5 条**（3 × `font-family` 设计记录 + 4 条死规则/Rust 自绘参照，**无一条是缺陷**）✓ |
| ID 保全 | main **21/21**、preview **6/6**、about **0/0** ✓ |
| 模板体元素数 | **30 / 9 / 16**（与改前一致）✓ |
| 生效 `:root` 规则数 | **0 / 0 / 0** ✓（形态甲的前提今天成立） |
| 文件完整性 | `docs/14` **1240+ 行、零乱码** ✓ |

**边界不变**：仍为**静态审查** —— 未编译、未渲染、未真机加载；`docs/13` 真机清单**尚未执行**；CI **未实跑自证**。

---

## 15. 第七轮：口径 ⑤ 落地验收 + 主题分支约束门禁落地 + 资产并发变更

### 15.0 本轮结论一览

| # | 事项 | 裁定 | 关键证据 |
|---|---|---|---|
| **Q7-1** | 「自定义属性先定义者赢」这条机制是否真的成立 | ✅ **成立** —— 源码级四证合一 | `cascade.cpp:18-33` / `value.cpp:572-579` / `cascade.h:41` / `cascade.cpp:124-125` |
| **Q7-2** | 前端是否已把口径 ⑤ 落地 | ✅ **已落地**（themes.json ⑤ + docs/12 §4.2 五条硬口径 + §5 R11 + 并入 V1.12） | `assets/themes.json:14`、`docs/12:276-282`、`docs/12:391` |
| **Q7-3** | themes.json / docs/12 内部是否自洽 | ⛔ **不自洽** —— 同一处「末尾追加」与口径 ⑤「必须是源码中第一个 `:root`」互相打架 | `themes.json:14` 句首 vs 同字段口径 ⑤；`docs/12:271-273` |
| **Q7-4** | docs/12 §5 R5「旧句柄 = 写野内存 = 静默崩溃/花屏」 | ⛔ **机制写错**（结论对、机制错、且漏了泄漏） | `handle_table.h:31`、`page_api.cpp:561-563`、`ui_context.cpp:125`、`ui_api.cpp:1886-1888`；与本报告 §12.1.3 自相矛盾 |
| **Q7-5** | 「`.uix` 禁主题分支选择器」要不要进断言 | ✅ **要** —— 已落进 QA 自己的门禁，**未动**前端的 `uix_tag_check.py` | `scripts/gates/qa_uix_verify.py` 断言 14/15/16 + 负控 EXIT=1 |
| **Q7-6** | 资产在本轮复核窗口内被并发修改 | ⚠️ main **21→26 id**、**30→35 元素**；前端随后已全量同步文档，**仅 §5 R5 两处漏改**（见 §15.7） | mtime `16:42:50` vs QA 读取 `16:43:04`；两套独立门禁同数字 |

### 15.1 Q7-1 —— 口径 ⑤ 的机制，源码级独立复核（四证合一）

前端已把「自定义属性先定义者赢」写进 `assets/themes.json` `revision.inject` 的**口径 ⑤**、`docs/12` §4.2 的**五条硬口径**表、§5 的 **R11**，并注明「并入 V1.12」。
**QA 不采信来函结论，逐条回上游源码取证，四条独立证据全部对上**：

| # | 证据 | 源码 | 逐字 |
|---|---|---|---|
| 1 | 变量按**文档顺序**从**每一个** `:root` 规则里收集（不是只收第一个 `:root`） | `src/ui/css/cascade.cpp:18-33` `CollectRootVars` | `for (const auto& rule : sheet.rules) { … if (d.property[0]=='-' && d.property[1]=='-') out.emplace_back(...) }` |
| 2 | 查变量**返回第一个命中** | `src/ui/css/value.cpp:572-579` `LookupVar` | `for (auto& kv : vars) { if (kv.first == name) { out = kv.second; return true; } }` |
| 3 | 头文件注释**逐字**确认 | `src/ui/css/cascade.h:41` | `// Collect all :root variables from stylesheet. (Returns first set encountered.)` |
| 4 | `:root` 规则**不参与 CSS 层叠**（所以「后者覆盖前者」这条常规直觉在这里**不成立**） | `src/ui/css/cascade.cpp:124-125` | `// Skip :root — those only define CSS vars, handled separately` / `if (IsRootSelector(sel)) continue;` |

**⇒ 裁定：口径 ⑤ 成立，前端这轮改对了。** 机制表述与 QA 第六轮的原始论证**逐字一致**。

**补充一条本轮新查的、能把口径 ⑤ 从「结论」升级为「可判定条件」的证据**（前端材料里还没有）：

| 证据 | 源码 | 作用 |
|---|---|---|
| `<style>` 块的规则按**块在源码里的先后**依次 push 进同一张 `sheet.rules` | `src/ui/page/page_api.cpp:333-350` | ⇒ 「源码里第一个 `:root`」= 「`sheet.rules` 里第一个 `:root`」，口径 ⑤ **可判定** |
| `<link>` 引的 CSS **排在所有 inline `<style>` 之前** | `src/ui/page/page_api.cpp:271`（`ResolveLinkBlocks` 先）与 `:276`（inline styles 后） | ⇒ 若将来用了 `<link>`，它里的 `:root` 会**赢过** inline 注入的 `:root` ⇒ 口径 ⑤ 的完整表述必须包含「`<link>` CSS 里也没有 `:root`」 |
| SFC 顶层标签**不限顺序** | `src/ui/uix/sfc_parser.cpp:252-297`（顶层循环只判 tag 名，不判位置） | ⇒ 把注入块放在**源码最前**在语法上**合法**（见 §15.2 的修法甲A） |

**行号漂移（P3，不阻塞）**：前端在 **7 处**写的是 `value.cpp:573-581`：
`assets/themes.json:14`（`revision.inject`）、`docs/12:277`（形态甲/乙说明）、`:289`（口径 ⑤ 行）、`:396`（R11）、`:532`、`:541`（§9 历史行）、`docs/13:33`（【P0】自查条）。
**实测 `LookupVar` 的真实范围是 `value.cpp:572-579`**（`grep -n` 输出：`572:bool LookupVar(const std::string& name,` … `579:}`）。
⇒ 建议统一改成 **`value.cpp:572-579`**。依据 `mechanical-gates.md`「行号必须可复核」—— 一个差 1 行的引用会让复核者以为漏看了函数体。

### 15.2 ⛔ Q7-3 —— 剩余不一致：「末尾追加」与口径 ⑤ 在同一句里打架

**事实**（逐字）：

| 位置 | 原文 |
|---|---|
| `assets/themes.json:14` `revision.inject` 句首 | 「Rust 在 `ui_page_load_string` 之前，在 .uix 源码【**末尾追加**一个完整的 `<style>` 块】」 |
| `assets/themes.json:14` `revision.inject` 口径 ⑤ | 「【**注入的 `:root` 必须是源码中第一个 `:root`**】」 |
| `docs/12:271-273` | 「Rust 实现走**形态甲** —— 在 `.uix` 源码**末尾追加**一个完整的 `<style>` 块」+「已统一为甲（追加在末尾），并由 Rust 侧加载前自检兜底（见下面口径 ⑤）」 |

**⇒ 这两句话不能同时成立**：「末尾追加」把注入块放到**最后**，「必须是第一个 `:root`」要求它**最前**。
两者只在**一个前提下**才兼容：**`.uix` 里完全没有 `:root`**（此时「最后一个」也就是「唯一一个」）。
`docs/12:283-284` 与 `themes.json` 口径 ⑤ 末尾都写了这个前提（「当前前提成立：生效 `:root` 规则数 = 0」）⇒ **前提是真的**，但**它是隐式的**，且它正是「将来会静默咬人」的那一类缺口。

**⇒ 两条修法（各有代价，QA 只给依据与代价，不替架构师/前端拍板）**：

| 方案 | 改法 | 代价 | 收益 |
|---|---|---|---|
| **甲A（QA 推荐）** | 把注入位置从「源码**末尾**追加」改成「源码**最前**追加」 | Rust 侧同一个字符串 concat，`prepend` 与 `append` **代码量相同**；SFC 顶层不限顺序（`sfc_parser.cpp:252-297`）⇒ 语法合法 | 口径 ⑤ **无条件成立**，不再依赖「`.uix` 没有 `:root`」这个巧合；将来有人在 `.uix` 里写 `:root` 也**不会**静默压掉注入值 |
| **甲B（最小改动）** | 保留「末尾追加」，把口径 ⑤ 的措辞改成「**前提：`.uix` 内不得出现任何 `:root`**」，并把这个前提做成**机器断言** | 依赖一个前提；前提一旦被破坏，**必须有人跑门禁**才会发现 | 文档改动最小；且本轮 QA 已把该断言落地（§15.4 断言 15） |

**⇒ QA 建议：甲A + 断言 15 兜底（两者不冲突，断言 15 同时守住「`.uix` 里没有第二个 `:root` 真相源」）。**
若采甲A，还需在 Rust 加载前自检里**顺带查一条**：「`<link>` 引的 CSS 里没有 `:root`」—— 因为 `page_api.cpp:271` 把 `<link>` 块排在 inline `<style>` 之前（§15.1 表末）。

### 15.3 ⛔ Q7-4 —— docs/12 §5 R5 的「旧句柄失效机制」写错了（结论对、机制错、且漏了泄漏）

**现状原文**（`docs/12:392` R5）：

> 「必须**主动重取全部 27 个句柄**（旧句柄 = **写野内存 = 静默崩溃/花屏**，**不会报「句柄无效」**）」

**QA 源码级复核 —— 机制不成立**（本报告 §12.1.3 已就同一问题给过结论，此处补齐引用）：

| 事实 | 源码 |
|---|---|
| 句柄表存的是 **`shared_ptr<Widget>`**（强引用） | `src/ui/handle_table.h:31`、`src/ui/handle_table.cpp:5-10` |
| `ui_page_destroy` **只** erase `PageEntry`，**不碰句柄表** | `src/ui/page/page_api.cpp:561-563` |
| 句柄表**只在 `Context::Shutdown` 时 `Clear()`** | `src/ui/ui_context.cpp:125` |
| `ui_widget_find_by_id` 会把 widget 的 `shared_ptr` **插入**句柄表 | `src/ui/ui_api.cpp:1897-1898` |
| `ui_widget_destroy(h)` → `Ctx().handles.Remove(widget)` | `src/ui/ui_api.cpp:1886-1888` |

**⇒ 旧句柄指向的是一个「还活着、但已脱离控件树的孤儿控件」**：

- ❌ **不是**「写野内存」—— 对象没被释放，句柄仍然有效；
- ❌ **不是**「崩溃 / 花屏」—— 写它不触发任何非法访问；
- ✅ **是**「**静默无效**」：API 返回成功，但 widget 不在任何窗口的内容树里 ⇒ **屏幕上什么都不变**（本报告 §12.1.3 原文：「不崩溃、不报错、不打日志」）；
- ✅ **另外还有一条 R5 没写的后果：每次切主题泄漏旧页面的整棵控件子树** —— 句柄表强引用着它们，直到 `Context::Shutdown` 才释放。

**⇒ 建议 R5 改写为（措辞，结论不动）**：

> 「旧句柄 = **指向已脱树的孤儿控件**（句柄表强引用，对象还活着）= **静默无效**：API 返回成功但界面毫无变化，**不会崩、也不会有任何提示** ⇒ 必须主动重取全部句柄。
> ⚠️ **并且要防泄漏**：切主题前对每个旧句柄调 `ui_widget_destroy(old_handle)`（`ui_api.cpp:1886-1888` → 从句柄表移除），否则每次切主题都泄漏旧页面的整棵控件子树，直到进程结束。」

**⇒ 顺带更正一个跨文档数字**：R5 / §4.4 / `docs/13` C6 里写的「**27** 个句柄（main 21 + preview 6）」**在本轮已过期**（见 §15.5）。

### 15.4 ✅ Q7-5 —— 前端的提问：主题分支约束要不要进断言？**要，但加在 QA 自己的门禁里**

**前端原话**：「（这条目前只是文档约束 —— 要不要我把它加进 `uix_tag_check.py` 的断言？那个脚本是我写的，但按纪律 `scripts/gates/` 是你的文件域，我先问。）」

**⇒ QA 裁定：加，但由 QA 加在 `scripts/gates/qa_uix_verify.py`，**不去改** `uix_tag_check.py`。**

**理由（三条，都不是客套）**：

1. **文件域**：`scripts/gates/` 是 QA 的文件域，但 `uix_tag_check.py` 是**前端写的**。QA 改别人正在维护的脚本，会让对方失去「自己脚本自己掌控」的确定性 —— 这与第 10 节的协作纪律同源。QA 在**自己**的门禁里加断言 = **零侵入**，且由 QA 自己承担维护责任。
2. **两边门禁职责不同，不重复**：`uix_tag_check.py` 查的是「**标签**在不在 core-ui 支持表内 / 旧标签有没有残留 / D-15 合规 / id 与元素数保全」；本轮这三条断言查的是「**CSS 选择器与变量**会不会静默失效」—— 属于不同维度，**互补而非重复**。
3. **负控归属**：QA 的门禁由 QA 做负控（注入缺陷 → 必须 EXIT=1）⇒ 断言的**可信度**由 QA 背书，放在 QA 的脚本里最合适。

**⇒ 本轮已落地三条断言（`scripts/gates/qa_uix_verify.py`）**：

| # | 断言 | 依据（上游源码） | 违反后果 |
|---|---|---|---|
| **14** | **`.uix` 里不得出现主题分支选择器 / 条件规则**：`.dark` / `.light` / `.theme-dark` / `[data-theme=…]` / `[data-mode=…]` / `@media` / `@supports` | `selector.cpp:6-15` `PseudoToBit` 只认 hover/pressed/focus/disabled/checked/root/drag-over；未知伪类 bit=0 → `selector.cpp:60-64` 直接 false；`css_parser.cpp:319-333` `SkipAtRule` 把 `@` 规则**整块丢弃**且「won't error out」 | 静默失效（不报错、不生效），与 `text=` / `border-bottom` / `<separator>` 的 `background` 同一类坑 |
| **15** | **`:root` 块里不得定义 `--app-*`**（口径 ⑤ 的**静态侧兑现**） | `cascade.cpp:18-33` + `value.cpp:572-579` + `cascade.cpp:124-125` | `.uix` 自带的 `:root` 与 Rust 注入的 `:root` **同名撞车**：两者只有一个生效、另一个被静默丢弃（不报错） |
| **16** | **`.uix` 里不得出现 `var(--app-accent)`**（口径 ③ 的静态侧兑现） | `themes.json` `revision.decision` / 架构 D14 | `--app-accent` 是**数据字段**不是 CSS 变量，不进 `:root`；`.uix` 一律读库 token `var(--accent)` |

**负控（QA 亲自跑，临时副本，不碰真资产）**：在 `main.uix` 的**临时副本**里注入 5 类缺陷
（`.dark` / `[data-theme="dark"]` / `@media (prefers-color-scheme: dark)` / `:root{--app-bg…}` / `var(--app-accent)`），
跑 `python3 scripts/gates/qa_uix_verify.py --root <临时根>` ⇒ **EXIT=1**，10 处 FAIL，逐条带 `文件:行`：

```
FAIL  assets/main.uix: 模板体元素数 35 ≠ 期望 30            ← 这条是「钉死值已过期」（§15.5），非注入缺陷
FAIL  assets/main.uix:52: 硬编码色值 → .dark { background: #FFFFFF; }
FAIL  assets/main.uix:53: 硬编码色值 → [data-theme="dark"] .x { color: #FFFFFF; }
FAIL  assets/main.uix:54: 硬编码色值 → @media (prefers-color-scheme: dark) { .y { color: #FFFFFF; } }
FAIL  assets/main.uix:55: 硬编码色值 → :root { --app-bg: #FFFFFF; --app-text: #000000; }
FAIL  assets/main.uix: CSS 里有 @media 规则 → **整块被解析器丢弃、静默失效**（css_parser.cpp:319-333 SkipAtRule）
FAIL  assets/main.uix: 主题分支选择器 `dark`（`.dark`）→ **永远不命中**（selector.cpp:6-15 / 60-64）
FAIL  assets/main.uix: 主题分支选择器 `[data-theme`（`[data-theme="dark"] .x`）→ **永远不命中**
FAIL  assets/main.uix: :root 块里定义了 ['--app-bg', '--app-text'] —— 与 Rust 注入的 token 同名，**两者只有一个生效、另一个被静默丢弃**
FAIL  assets/main.uix: 出现 var(--app-accent) —— 它是 Rust 的【回退源值（数据字段）】，不是 CSS 变量
```

⇒ **断言 14/15/16 全部能抓、能定位到行**；临时副本已删除，真资产**零写入**。

### 15.5 ⚠️ Q7-6 —— 资产在复核窗口内被并发修改，钉死值与文档数字同时过期

**时间线（客观事实）**：

| 时刻 | 事件 |
|---|---|
| 16:42:50 | `assets/main.uix` / `assets/preview.uix` 的 mtime（本轮 QA 读取时的**最后一次**写入） |
| 16:43:04 | QA 复跑 `qa_uix_verify.py` ⇒ 报「模板体元素数 **35** ≠ 期望 30」——**此时距最后一次写入仅 14 秒** |
| 16:43:31 | 前端把 `scripts/gates/uix_tag_check.py` 的钉死值更新为 **35 / 26** |
| 16:43:40 | QA 复测（只读）确认：main 35 元素 / 26 id；preview 9 / 6；about 16 / 0 |

⇒ **QA 未对 `assets/**` 做任何写操作**（第 10 节纪律：不动别人正在编辑的文件）。QA 在发现移动靶后的处置是：**先只读记录 → 等前端钉死自己的基线 → 再钉自己的基线**。

**资产实际变化（前后两组数字，均由 QA 独立实测）**：

| 文件 | 改前 | 改后 | 变化 |
|---|---|---|---|
| `assets/main.uix` | 30 元素 / **21 id** | **35** 元素 / **26 id** | ＋5 元素、＋5 id |
| `assets/preview.uix` | 9 元素 / 6 id | 9 元素 / 6 id | 标签变（`preview_body` `<div>` → `<custom>`） |
| `assets/about.uix` | 16 元素 / 0 id | 16 元素 / 0 id | 未变 |

**新增/变更的 id**：`preview_image`、`preview_host_name`、`preview_host_size`、`preview_host_channel`、`preview_host_state`（main，新增）；`preview_body`（preview，**标签从 `<div>` 改成 `<custom>`**）。

**⇒ 三条后果**（**状态已于 16:45:46 复核更新** —— 前端在 QA 写本节期间已把其中两条同步掉，QA 不把已修项当开放项报）：

| # | 后果 | **当前状态（16:45:46 实测）** |
|---|---|---|
| **1** | **句柄总数 27 → 32**（main 26 + preview 6） | ✅ **已同步**：`docs/12` §0（`:17`）、§4.4（`:347`/`:350`/`:356`）、§5 **R4**（`:394`）、§6（`:441`）、§9（`:545`）；`docs/13` §0（`:31`）、**H1**（`:123`）、**C6**（`:162`）；`assets/themes.json` `revision.switch`；`scripts/gates/uix_tag_check.py` 期望值 ⇒ **全部已是 32** ✓<br>⛔ **但 `docs/12:397`（§5 **R5**）漏改** —— 同一节里还写着「重取全部 **27** 个句柄」 |
| **2** | **模板体元素数 30 / 9 / 16 → 35 / 9 / 16** | ✅ **已同步**：`docs/12` §0（`:15`，含「含 SFC 外壳则是 39/12/19」）、§3.1 表、§9（`:545`）；`uix_tag_check.py` 期望值 ⇒ **已是 35 / 9 / 16** ✓<br>⚠️ 仅 `docs/12` §9 的**历史行**（`:505` S1 行、`:527` QA 第二轮行）仍写当轮的旧值 —— 历史记录本就该保留旧值，但建议各加一句「（当轮值；现为 35 / 9 / 16）」以免被读成现值 |
| **3** | **宿主归属变了** | ✅ **架构第七轮已裁定**（`docs/12:545`「架构第七轮 · 元素数与句柄数变了」：把 `preview_host` 拆成「容器 + `<custom id="preview_image">` + 4 个元信息 `<label>`」）⇒ **QA 不代裁、此项关闭**<br>⛔ **但 `docs/12` §5 **R5** 的「逐个判定」表没跟上**：该表只列了 `list_host` / `nav_host` / `breadcrumb` / `preview_host` **四个**，**新增的 `preview_body`（已改 `<custom>`）与 `preview_image`（新增 `<custom>`）两行没有** ⇒ R5 表与资产不一致 |

**关于后果 3 的 QA 观察（供架构参考，不下裁定）**：

- **布局核对已过**：`<div>` 默认是 **VBox（列）** —— `widget_factory.cpp:919-927` `makeContainer()`：`flex-direction` 缺省 → `VBoxWidget`；`.preview-host` 未设 `flex-direction` ⇒ 列排 ✓（`<custom id="preview_image">` 在上、4 个 `<label>` 在下）。**不会**出现「图片和文字并排」的破版。
- **尺寸核对已过**：`<custom>` → `CustomWidget`，CSS `width/height` 走 `ApplyCommonStyle` **自然生效**（`widget_factory.cpp:1111-1118` 注释逐字）⇒ `.preview-image { height: 128px }`、`.preview-body { flex: 1 }` 都会生效。
- **文字标签核对已过**：`.preview-meta { display: block }` 覆盖了 `<label>` 默认的 `display: inline`（`widget_factory.cpp:942-953`）⇒ 4 个 meta 行会拉伸占满宽度 ✓。
- **⇒ R5 建议补一句措辞**：「**混合容器**：外层保留 `<div>`（它装的是 core-ui 控件），其中 Rust 自绘的那一块用**嵌套 `<custom>`**」—— 免得下一轮又有人把 `preview_host` 也改成 `<custom>`（那会**丢掉 4 个 `<label>`**，因为 `<custom>` 是空的自绘画布）。

**⇒ QA 已同步钉自己的基线**：`scripts/gates/qa_uix_verify.py` 的 `EXPECTED_IDS` / `EXPECTED_TEMPLATE_ELEMENTS` 已更新为 **main 35/26、preview 9/6、about 16/0**，并在源码注释里写明「基线 = 2026-11-13 16:43 读数」。
**交叉验证**：前端自己的 `uix_tag_check.py`（16:43:31 更新）报出**同样的 35 / 26** ⇒ **两套独立实现同数字** ✓（`mechanical-gates.md`「没有执行器的规则只是建议」的反面：两个执行器给出同一个数）。

### 15.6 本轮复跑的门禁输出（QA 亲自跑，非引用）

| 门禁 | 退出码 | 结果 |
|---|---|---|
| `python3 scripts/gates/qa_uix_verify.py` | **0** | PASS —— 无阻塞项；WARN 5 条（3 × `font-family` 设计记录 + 2 条死规则，**无一条是缺陷**） |
| `python3 scripts/gates/theme_contrast_check.py` | **0** | PASS —— 真实文字元素对比度全部 ≥ 4.5:1；非刻度间距 0 处 |
| `python3 scripts/gates/uix_tag_check.py`（前端脚本，QA 只跑不改） | **0** | PASS —— 模板体元素 **35 / 9 / 16**、id **26 / 6 / 0**，全部保全 |

**新断言的逐资产输出（三个 `.uix` 各 4 条，全 PASS）**：

```
PASS  无主题条件规则（无 @media/@supports）（at-rule 名单：无）
PASS  无主题分支选择器（无 .dark/.light/[data-theme=…]）
PASS  `:root` 块数 0 → Rust 注入的 :root 必然是源码中第一个 :root（口径 ⑤ 成立）
PASS  无 var(--app-accent)（accent 走库 token var(--accent)）
```

**边界不变**：仍为**静态审查** —— 未编译、未渲染、未真机加载；`docs/13` 真机清单**尚未执行**；CI **未实跑自证**；本轮所有结论只覆盖**源码级**语义。

### 15.7 本轮结清项与仍开放项（16:46 实测状态）

**✅ 本轮结清（前端已落地，QA 已复核）**：

| 项 | 证据 |
|---|---|
| 口径 ⑤ 写进 `themes.json` `revision.inject` | `assets/themes.json:14`（五条硬口径 ①~⑤） |
| 口径 ⑤ 写进 `docs/12` §4.2 | `docs/12:276-282`（**五条**硬口径表）+ `:273` |
| 口径 ⑤ 登记为 Rust 侧待办 | `docs/12:391`（**R11**）+「并入 V1.12」 |
| 「形态乙要求最后一个 `<style>` 块」这句**依据反了**已更正 | `docs/12:527`、`:536`（「第五轮更正」段） |
| 句柄数 / 元素数全量同步 27→32、30→35 | `docs/12:15`/`:17`/`:347`/`:394`/`:441`/`:545`；`docs/13:31`/`:123`/`:162`；`themes.json` `revision.switch`；`uix_tag_check.py` |
| 「`.uix` 里不允许深浅色分支」升格为文档专节 | `docs/12:149`（§2.7） |
| `nav_host` → `<custom>` | `assets/main.uix:176`（实测 `<custom id="nav_host" class="nav-host" />`） |
| R8 / §8 I4 两处旧表述统一为「已裁决(c) 且已落地」 | `docs/12:394`（R8）、`:495`（§8 I4 行） |

**⛔ 仍开放（按三档）**：

| 档 | # | 事项 | 位置 | 建议 |
|---|---|---|---|---|
| **阻塞** | — | **无** | — | 前端本轮无阻塞项 |
| **重要** | **O-1** | 「末尾追加」与口径 ⑤「必须是源码中第一个 `:root`」在同一字段里**自相矛盾** | `assets/themes.json:14` 句首 vs 同字段口径 ⑤；`docs/12:271-273` | 采 **甲A**（改成「源码最前追加」，`sfc_parser.cpp:252-297` 顶层不限顺序 ⇒ 合法）或 **甲B**（保留末尾 + 写明前提 + 断言 15 兜底）。见 §15.2 |
| **重要** | **O-2** | 「旧句柄 = 写野内存 = 静默崩溃/花屏」**机制写错**，且同一句里**句柄数还写着 27** | `docs/12:397`（§5 R5） | 改成「旧句柄 = **指向已脱树的孤儿控件**（句柄表强引用，对象还活着）= **静默无效**，不会崩也不会有任何提示」+ 补「切主题前先 `ui_widget_destroy(old_handle)` 防泄漏」+ 数字改 32。见 §15.3 |
| **重要** | **O-3** | §5 R5 的「逐个判定」表**漏了 `preview_body` / `preview_image`** 两个新 `<custom>` 宿主 | `docs/12:397` | 补 2 行 + 补一句「**混合容器**：外层保留 `<div>`，其中 Rust 自绘的那一块用嵌套 `<custom>`」（否则下一轮有人会把 `preview_host` 也改成 `<custom>`，**丢掉 4 个 `<label>`**） |
| **建议** | **O-4** | `LookupVar` 行号引用漂移：写 `value.cpp:573-581`，实际 **`572-579`** | `assets/themes.json:14`；`docs/12:277`/`:289`/`:396`/`:532`/`:541`；`docs/13:33`（**共 7 处**） | 统一改成 `value.cpp:572-579`（行号必须可复核） |
| **建议** | **O-5** | §9 历史行仍写当轮旧值（`27 个句柄` / `30 / 9 / 16`） | `docs/12:500`/`:505`/`:527`/`:539` | 各加一句「（当轮值；现为 32 个句柄 / 35 / 9 / 16）」，避免被读成现值 |

### 15.8 本轮新增引用与依据（上游 core-ui 只读副本 `C:/Users/tx/AppData/Local/Temp/upstream/core-ui-main`）

| 用途 | 路径:行 |
|---|---|
| `:root` 变量按文档顺序**全部**收集 | `src/ui/css/cascade.cpp:18-33` |
| 查变量**取第一个命中** | `src/ui/css/value.cpp:572-579` |
| 「Returns first set encountered.」逐字注释 | `src/ui/css/cascade.h:41` |
| `:root` 规则**不进 CSS 级叠** | `src/ui/css/cascade.cpp:124-125` |
| `<style>` 块规则按块序 push 进同一张 sheet | `src/ui/page/page_api.cpp:333-350` |
| `<link>` 块排在 inline `<style>` 之前 | `src/ui/page/page_api.cpp:271` / `:276` |
| SFC 顶层标签**不限顺序** | `src/ui/uix/sfc_parser.cpp:252-297` |
| 库 token 追加在用户 `:root` 之后（用户先定义者赢） | `src/ui/page/compiler.cpp:1574-1579`、`:1587+` |
| `RebuildThemeVars` 擦除并重写 34 个库 token | `src/ui/page/compiler.cpp:1647-1703` |
| `RefreshThemeStyles` **不重新采集** `:root` | `src/ui/page/page_state.cpp:688-702` |
| 伪类白名单（无主题态位） | `src/ui/css/selector.cpp:6-15`、`:60-64` |
| `@` 规则整块丢弃 | `src/ui/css/css_parser.cpp:319-333` |
| `<div>` 默认 VBox（列） | `src/ui/page/widget_factory.cpp:919-927` |
| `<custom>` → `CustomWidget`，CSS 宽高自然生效 | `src/ui/page/widget_factory.cpp:1111-1118` |
| 句柄表存 `shared_ptr` | `src/ui/handle_table.h:31`、`src/ui/handle_table.cpp:5-10` |
| `ui_page_destroy` 不碰句柄表 | `src/ui/page/page_api.cpp:561-563` |
| 句柄表只在 Shutdown 时 Clear | `src/ui/ui_context.cpp:125` |
| `ui_widget_destroy` → `handles.Remove` | `src/ui/ui_api.cpp:1886-1888` |
| `ui_widget_find_by_id` 插入句柄 | `src/ui/ui_api.cpp:1897-1898` |

---

## 16. 第八轮：架构 Δ-A 契约反转落地 + Δ-B1/Δ-B2 复核 + **门禁 S13 落地**

### 16.0 本轮结论一览

| # | 事项 | 裁定 | 证据 |
|---|---|---|---|
| **Q8-1** | 架构 Δ-A（注入位置）是否落地 | ✅ **已落地，且是甲A**（源码最前插入）⇒ **§15.7 的 O-1 结清** | `spec/ffi-core-ui.md:269-279`、`:318`；`assets/themes.json` `revision.inject`；`docs/12:276` |
| **Q8-2** | 甲A 是否**无条件**成立 | ⚠️ **还差一个前提**：`<link>` CSS 排在 inline `<style>` 之前 | `page_api.cpp:271` / `:276`；`spec/ffi-core-ui.md` 全文**未提** `<link>` 的这条影响 |
| **Q8-3** | 架构 Δ-B1（`backdrop` 不是零命中） | ✅ **成立**，QA 独立复核通过 | `widget_factory.cpp:606-611`、`widget.h:245` |
| **Q8-4** | 架构 Δ-B2（`:root` 内只允许变量） | ✅ **成立**；静态侧对应已落进 S13 | `cascade.cpp:125` + `cascade.cpp:18-33`（只收 `--`） |
| **Q8-5** | **门禁 S13**（正式脚本，QA 文件域） | ✅ **已落地**：`scripts/gates/s13_token_namespace_check.py`，EXIT=0，负控 EXIT=1 | 见 §16.5 |
| **Q8-6** | 运维 P0（`package` job 缺 `zsui` checkout） | ✅ **已修**，QA 复核通过 | `.github/workflows/build.yml:363-380` |
| **Q8-7** | 前端本轮的 X-3 / 形态乙依据更正 | ✅ 复核通过（数字口径 71 / 2 处注释占位一致） | `docs/12:538`、`docs/11:116` |

### 16.1 ✅ Q8-1 —— Δ-A 落地，**O-1 结清**

架构师已把契约**规则 (d) 从「必须最后」反转为「必须是源码里第一个 `:root`」**，并把注入形态从「源码末尾追加」改为「**源码最前面插入一个完整的 `<style>` 块**」。
**前端同一轮也独立跟进了**：`assets/themes.json` `revision.inject` 已写「在 .uix 源码【**最前面插入**一个完整的 `<style>` 块】…（顶层块顺序无关 —— 解析器按标签名分派到各自容器…插最前合法，且天然保证「注入的是第一个 `:root`」）」，`docs/12:276` 已写「**第八轮更正注入形态**…（**不是**「末尾追加」）」。
⇒ **架构契约与前端实现口径现已一致**，§15.2 的 **O-1 结清**。

**QA 复核结论：架构师这一版的推导链与 QA 第六轮/第七轮的独立取证完全一致，无新增分歧。** 三处「加厚」值得记：

| 架构侧新增 | QA 复核 | 评价 |
|---|---|---|
| 规则由 4 条扩为 **6 条**（新增「只允许 CSS 变量」的**断言版** + 键集排除） | ✓ 与 QA 第七轮 §15.4 断言 15/16 的方向一致 | ✅ 与 QA 的门禁侧实现互补 |
| Rust 自检由 1 条扩为 **4 条**（位置 / 顺序 / 内容 / 键集） | ✓ 位置/顺序直接对应口径 ①⑤；内容对应 Δ-B2；键集对应 S13 | ✅ **四条的失败症状各不相同**（"SFC parse failed" / 无报错 / 无报错 / 无报错），QA 认为这个切分是对的 |
| **V1.12 扩为 4 个坏源码构造** | ✓ `spec/milestones.md:129` 已写：① 裸放顶层 ② `.uix` 自带 `:root` 在前 ③ 混入普通声明 ④ 键集含 `--app-accent` | ✅ QA 第七轮 §15.1 指出的「现有 V1.12 只测一种坏情形」已补上 |

### 16.2 ⚠️ Q8-2 —— 甲A 还差一个前提：`<link>` CSS 排在 inline `<style>` **之前**

**事实**（QA 复核）：`src/ui/page/page_api.cpp`

```
:271   std::vector<StyleBlock> blocks = ResolveLinkBlocks(sfc.links);   // ← link 块先
:276   blocks.push_back(std::move(blk));                                 // ← inline <style> 后
:333   for (auto& blk : blocks) { … sheet.rules.push_back(r); }            // ← 按 blocks 顺序进同一张 sheet
```

⇒ **`<link>` 引的 CSS 永远排在所有 inline `<style>` 之前**。
⇒ 形态甲把注入块放在**源码最前**，只保证它是**第一个 inline `<style>` 块**；若 `<link>` CSS 里定义了 `:root`，**注入值仍然会输**（先定义者赢）。

**⇒ 三个层面的影响**：

| 层面 | 现状 | QA 建议 |
|---|---|---|
| **当前资产** | **零 `<link>`**（`grep '<link' assets/*.uix` → 无命中）⇒ 现在不会出问题 | 无需改 |
| **Rust 自检断言 2** | 契约写的是「断言**注入块是源码中第一个 `:root`**（除注入块外，源码中**没有**任何 `:root` 出现在注入块**之前**）」—— 若实现只扫**inline 源码**，就**扫不到 `<link>` CSS 里的 `:root`** | 断言 2 的**扫描范围**要写清：要么扩到 `<link>` 引的 CSS，要么**另加一条断言**「本项目 `.uix` 不得使用 `<link>`」（更简单，且现状已满足） |
| **契约文本** | `spec/ffi-core-ui.md` 全文**只在两处提到 `<link>`**，都是「顶层允许的六种标签」语境，**没有一处**提到「`<link>` CSS 排在 inline `<style>` 之前」这个影响 | 建议在规则 4（或形态甲那行）补一句：「⚠️ 本条以『不使用 `<link>`』为前提 —— `<link>` 块恒排在 inline `<style>` 之前（`page_api.cpp:271`）」 |

**⇒ 这是甲A 落地后**唯一**残留的、有源码依据的缺口。QA 判 **P2（建议）**，不阻塞：三个资产零 `<link>`，且 `spec/ffi-core-ui.md` 的规则 1 已要求「注入块在 `<style>` 内」、规则 4 已要求「注入块是第一个 `:root`」，只要 Rust 自检**按契约实现到位**，漏 `<link>` 的概率很低。
**但契约文本应当把前提写出来** —— 否则将来有人加 `<link>` 时，这条前提就消失在一句「顶层块顺序无关」里（该句对 `<link>` 而言**不成立**：`<link>` 块的位置由 `ResolveLinkBlocks` 决定，与它在源码里的位置无关）。

### 16.3 ✅ Q8-3 —— Δ-B1（`backdrop` 不是零命中）成立

**QA 独立复核**（不采信转述）：

```
src/ui/page/widget_factory.cpp:606-611
    if (s.Has("backdrop-filter")) { w.backdropBlur = parseBlur(s.Get("backdrop-filter")); }
    if (s.Has("backdrop-blur"))   { w.backdropBlur = parseBlur(s.Get("backdrop-blur")); }

src/ui/widget.h:245
    float backdropBlur = 0.0f;                 // CSS backdrop-filter: blur(...)
```

⇒ **widget 级 CSS 模糊确实受支持**（`backdrop-filter` 与 `backdrop-blur` 两个属性名都认）⇒ 架构师把 `backdrop` 从 `spec/milestones.md` §8.1 的「零命中」移出、改为「不是零命中」是**对的**。
⇒ **窗口级**（`DWMWA_SYSTEMBACKDROP` / `DWMWA_MICA` / `SetWindowCompositionAttribute` / `ACCENT_POLICY`）仍为**零命中** ⇒ **D13 的三项限制结论不变** ✓（QA 第七轮 §13 已核过那批零命中，两轮口径一致）。
⇒ 关于命中数口径差异（架构侧 38 处 / QA 侧 11 处）：架构师已选择在文档里写「多处命中」并标注口径，避免以后被当成矛盾 —— **QA 认可这个处理**，不再追数字。

### 16.4 ✅ Q8-4 —— Δ-B2（`:root` 内只允许变量）成立，且静态侧已落进 S13

**QA 独立复核**：非 `--` 开头的声明在 `:root` 规则里**被静默丢弃** —— `cascade.cpp:125` 把整条 `:root` 规则从级联里跳过，`CollectRootVars`（`cascade.cpp:18-33`）只收 `--` 开头的属性 ⇒ `background`/`color`/`font-family` 既不进级联、也不进变量表，**双路丢弃、零报错**。
⇒ 架构把它写成**规则 6 + 自检断言 3**（每条声明匹配 `--app-[a-z0-9-]+\s*:`）**是对的**。

**⇒ QA 的静态侧对应已落地**：S13 附加检查「全部 token 名符合 `--app-[a-z0-9-]+`」（§16.5）。这是 Rust 自检断言 3 在**静态资产层**的对应物 —— Rust 侧管的是「**注入的那段** `:root` 里的声明形状」，S13 管的是「**资产与 themes.json 里的 token 名形状**」。

### 16.5 ✅ Q8-5 —— **门禁 S13 已落地**（架构明确把正式脚本放在 QA 文件域）

**架构师授权**：「门禁 S13 的正式脚本在 `scripts/gates/`（你的文件域）—— 我已授权**前端**把等价断言加进他们自己写的 `uix_tag_check.py`（作为预检），但**正式门禁脚本仍归你**。」并给出**逐字口径**（① 34 个全集 → 失败；② `.uix` 内定义 `--app-accent` → 失败；③ `themes.json` 内 `--app-accent` **允许**；④ `var(--accent)` 等**引用不拦**；另：`.uix` 内不得出现 `var(--app-accent)`）。

**⇒ QA 产出：`scripts/gates/s13_token_namespace_check.py`（新建，独立门禁脚本）**

**QA 的两条实现决定**（架构明说「放哪个脚本」「名单 vs 命名空间」由 QA 定）：

| 决定 | QA 的选择 | 理由（逐字写进脚本头部） |
|---|---|---|
| **放哪个脚本** | **独立成脚本**，不塞进 `qa_uix_verify.py` | S13 是**契约门禁**（有独立编号，会被 `spec/milestones.md` / `spec/ci-contract.md` 引用），独立脚本可被 CI **单点调用**、退出码自洽；`qa_uix_verify.py` 是「资产质量门」，两者职责不同、互不依赖。<br>注：S13 的「另：`.uix` 内不得出现 `var(--app-accent)`」与 `qa_uix_verify.py` 的**断言 16 有意重叠** —— 两个独立执行器，一个门禁失败不应掩盖另一个 |
| **① 做成名单还是命名空间不相交** | **名单**（显式 34 个名字，逐字抄自 `compiler.cpp:1648-1658`） | 项目 token 集是 `themes.json` 里**显式枚举**的一组名字，**不是**靠前缀推导的命名空间 ⇒「`--app-` 前缀 vs 库 token 名」这个判据是**空的**（`--app-bg` 天然 ≠ `--bg`）。<br>S13 要挡的真实失败模式是「**把库 token 名误抄进项目 token 集**」（在 `themes.json` 里写 `"--bg": "#FFF"`、或在 `.uix` 的 `:root` 里写 `--bg:`）。**前缀规则挡不住这种错，只有 34 个名字的成员判定挡得住。** |

**名单漂移自检**：脚本每次运行都把 34 个名字**排序打印**（便于与上游一行 `sed` 输出逐字 diff），且**条数 ≠ 34 时直接 FAIL**（不静默放过）。复核命令写进脚本头部：
`sed -n '1648,1658p' <上游>/src/ui/page/compiler.cpp`

**「定义」与「引用」的区分口径**（对应 ④）：

| 文件 | 「定义」怎么认 | 「引用」怎么认 |
|---|---|---|
| `.uix` | CSS 里的 `--name:`（**排除** `var(` 前缀形态）；注释已剥离 | `var(--name)` |
| `themes.json` | **所有以 `--` 开头的 JSON 键**（递归）；`revision.*` 里的**说明文字是 prose，不是定义** | `revision.*` 文字里的 `var(--x)` 属**文档提及**，脚本会**单独标注**，不与真引用混计 |

**当前实测（EXIT=0）**：

```
PASS  库 token 名单 34 个（compiler.cpp:1648-1658）
=== assets/main.uix ===     ① 定义侧：0 个 token 定义，零个撞库 token
                          ② 未定义 `--app-accent`
                          另：未出现 `var(--app-accent)`
                          ④ 引用侧：25 处 var() 引用**不拦**
=== assets/preview.uix ===  同上，7 处引用
=== assets/about.uix ===    同上，5 处引用
=== assets/themes.json ===
      ① 定义侧：34 个 token 定义（17 light + 17 dark），零个撞库 token
      ③ `--app-accent` **允许**（/light/--app-accent, /dark/--app-accent）—— D14 的回退源值
      ④ 引用侧：0 处真引用；4 处出现在 revision.* 说明文字里 —— 文档提及，不是引用
      附加：全部 token 名符合 `--app-[a-z0-9-]+`
      附加：light / dark 键集一致（各 17 个）
PASS —— S13 通过（0 处提示）
```

**负控（QA 亲自跑，临时副本，不碰真资产）**：注入 6 类缺陷 ⇒ **EXIT=1**，8 处 FAIL，逐条带 `文件:行`：

```
FAIL  assets/main.uix:58: 定义了库的 token `--bg` → 与库 token 撞名…会被 RebuildThemeVars 擦除重写
FAIL  assets/main.uix:58: 定义了 `--app-accent` —— 它是 Rust 的【回退源值（数据字段，不是 CSS 变量）】
FAIL  assets/main.uix:59: 出现 `var(--app-accent)` —— accent 在 .uix 里一律读库 token var(--accent)
FAIL  assets/main.uix:58: token 名 `--bg` 不符合 `--app-[a-z0-9-]+`
FAIL  assets/themes.json: 定义了库的 token `--bg`（/light/--bg）
FAIL  assets/themes.json: token 名 `--bg`（/light/--bg）不符合 `--app-[a-z0-9-]+`
FAIL  assets/themes.json: token 名 `--app_Bad`（/dark/--app_Bad）不符合 `--app-[a-z0-9-]+`
FAIL  assets/themes.json: light/dark 键集不一致 —— 只在 light: ['--app-hover', '--bg'] / 只在 dark: ['--app_Bad']
```

⇒ **四条逐字口径全部被覆盖且能定位到行**；临时副本已删除，真资产**零写入**。

**⇒ 另加三条「附加」检查（QA 增补，**不改变**架构师的四条逐字口径**，各自独立标注）**：

| # | 附加检查 | 对应架构师的哪一条 | 为什么值得加 |
|---|---|---|---|
| A | token 名形状 `--app-[a-z0-9-]+` | 自检断言 3（内容）的**静态对应** | Rust 侧管注入段，S13 管资产侧；两侧合起来才闭环 |
| B | `themes.json` 的 **light / dark 键集必须一致** | 规则 4「切主题要重注入」的前提 | 键集不等 ⇒ 切到缺键的那一侧 ⇒ 该 token 静默回落到**库默认色**（不报错） |
| C | 库 token **名单条数 ≠ 34 时直接 FAIL** | ① 的自保 | 上游升级后库新增 token 而名单没同步 ⇒ S13 会**漏判**，必须让门禁自己先炸 |

### 16.6 ✅ Q8-6 —— 运维 P0（`package` job 缺 `zsui` checkout）已修，QA 复核通过

**QA 独立复核**（未转述）：

| 事实 | 证据 |
|---|---|
| `zsui` 是**仓外同级目录**的 path 依赖 | `crates/zsdesk-app/Cargo.toml:21`、`crates/zsdesk-ui/Cargo.toml:10/11/14` → `path = "../../../zsui"` |
| Cargo **整体解析 workspace** ⇒ 缺一个 path 依赖就连「只构建一个 crate」也炸 | `Cargo.toml` `[workspace] members = [...6 crates]`；`crates/zsdesk-coreui/Cargo.toml` 自己只依赖 `libloading`，但仍被牵连 |
| `rust` job 已 checkout `zsui` | `build.yml:30-44`（`app` + `zsui` 两个 checkout，zsui 用 `app/packaging/zsui-revision.txt` 的钉版） |
| `package` job **也已** checkout `zsui` | `build.yml:359-380`：`app` → 读钉版 → `repository: qiu7824/zsui` / `path: zsui`；**且注释块（`:363-368`）逐字写明了这条推理** |
| 无 `.gitmodules` | 根目录无该文件 ⇒ 「递归子模块」这条路确实无关（与架构师对 H2 的订正一致） |

⇒ **P0 已闭环** ✓。

### 16.7 ✅ Q8-7 —— 前端本轮确认项复核

| 前端声明 | QA 复核 |
|---|---|
| 形态乙依据已从「必须是最后一个 `<style>` 块」改为「此前不能有 `:root`」 | ✅ `docs/12:277`、`:536` 已改 |
| Rust 自检第 3 条断言已登记为 `docs/12` §5 **R11**，并注明并入 V1.12 | ✅ `docs/12:396`；`spec/milestones.md:129` 的 V1.12 已扩为 4 个坏构造 |
| 三个 `.uix` 剥注释后生效 `:root` 规则数 = **0 / 0 / 0** | ✅ QA 独立复算一致（`main.uix` 的 3 处命中全在 HTML 注释里） |
| X-3 量词：「真符号 71，另有 2 个注释里的前缀占位」 | ✅ 与 QA 第七轮结论一致；`docs/11:116` 的三口径写法可复现 |
| 本轮只动了 `docs/12` / `docs/13` / `assets/themes.json` | ✅ 与 mtime 一致（`assets/*.uix` 未动） |

### 16.8 本轮复跑的门禁输出（QA 亲自跑）

| 门禁 | 退出码 | 结果 |
|---|---|---|
| `scripts/gates/qa_uix_verify.py` | **0** | PASS（WARN 5 条，无一是缺陷） |
| `scripts/gates/theme_contrast_check.py` | **0** | PASS |
| `scripts/gates/uix_tag_check.py`（前端脚本，QA 只跑不改） | **0** | PASS（35 / 9 / 16，id 26 / 6 / 0） |
| **`scripts/gates/s13_token_namespace_check.py`（本轮新建）** | **0** | **PASS —— S13 通过（0 处提示）** |

**边界不变**：仍为**静态审查** —— 未编译、未渲染、未真机加载；`docs/13` 真机清单**尚未执行**；CI **未实跑自证**。

**⚠️ 一条必须记下的事实**：**当前 `build.yml` 里没有任何一步调用 `scripts/gates/**`**（`grep -n 'scripts/gates\|qa_uix_verify\|uix_tag_check\|theme_contrast' .github/workflows/build.yml` → **零命中**）。
⇒ 按 `mechanical-gates.md`「没有执行器的规则只是建议」：**包括 S13 在内，这些门禁目前都只在本地手工跑，CI 不执行**。
⇒ 门禁要变成「门」而不是「建议」，需要有人在 CI 里加一步调用它们。**`build.yml` 是 team-lead 的文件域，QA 不擅自改** ⇒ 已列入本轮给 team-lead 的待办。

### 16.9 本轮新增引用与依据

| 用途 | 路径:行 |
|---|---|
| `backdrop-filter` / `backdrop-blur` 受支持 | `src/ui/page/widget_factory.cpp:606-611` |
| `backdropBlur` 字段 | `src/ui/widget.h:245` |
| 库 token 34 个名单（kThemeKeys） | `src/ui/page/compiler.cpp:1648-1658` |
| `<link>` 块排在 inline `<style>` 之前 | `src/ui/page/page_api.cpp:271` / `:276` |
| 块规则按序进同一张 sheet | `src/ui/page/page_api.cpp:333-350` |
| `zsui` 是仓外同级 path 依赖 | `crates/zsdesk-app/Cargo.toml:21`、`crates/zsdesk-ui/Cargo.toml:10/11/14` |
| `package` job 已 checkout `zsui` | `.github/workflows/build.yml:359-380` |
| 契约新口径（六条规则 / 四条自检 / V1.12 四坏构造） | `spec/ffi-core-ui.md:248`、`:269-279`、`:299-316`、`:318`；`spec/milestones.md:129` |

---
