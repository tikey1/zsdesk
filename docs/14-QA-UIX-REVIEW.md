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
