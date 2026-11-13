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
