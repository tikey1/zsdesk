# 剩余工作流程（2026-11-13 收尾快照）

> 本文件记录**代码停止编写时**的未完成事项。
> 上一次已推送到 GitHub 的状态见 §1；本轮新增但**尚未验证**的改动见 §2；
> 后续里程碑见 §3；已知的判断与风险见 §4。

---

## 0. 一句话现状

Rust 侧的 core-ui 绑定层（`crates/zsdesk-coreui/`）与 CI 契约已经落地，
**六个机械门禁全部能在 CI 里真正执行**；
**core-ui 的 C++ 源码构建能否在 runner 上编过，仍然未知** ——
前两次 CI 失败已被定位为「日志管道被 PowerShell 中断」而非编译错误（见 §2.1），
修法已进 `build.yml`，**但还没有一次 CI 跑过修复后的版本**。

---

## 1. 本轮已写入磁盘、**已通过本地验证**、随本次提交推送

| 项 | 文件 | 验证方式 |
|---|---|---|
| `:root` 注入模块（形态甲） | `crates/zsdesk-coreui/src/lib/theme.rs` | tree-sitter 语法零错误 + 9 条单元测试 + 5 条等价逻辑用例 |
| 注入路径进 CI 探针 | `crates/zsdesk-coreui/examples/probe.rs` | tree-sitter 语法零错误 |
| PE 导出表解析器 | `scripts/gates/pe_exports.py` | 用本机 `GuoheView\core-ui.dll` 实测，**逐字节复现** `docs/05` 的 567 个 `ui_*` 基线 |
| 导出面门禁 S7' | `scripts/gates/coreui_export_gate.py` | 正例通过；反例（截断 DLL / 无 `ui_*` 的 kernel32）均正确失败 |
| id 清单门禁 | `scripts/gates/ui_id_manifest_check.py` + `assets/ui-ids.json` | 本地通过；注入 typo 后正确失败 |
| CI 日志中断修复 | `.github/workflows/build.yml` | YAML 解析通过；**尚未经真实 CI 验证** |
| CI 门禁接线（5 个 S13 门禁 + S7'） | `.github/workflows/build.yml` | 本地逐个跑通 |
| package job 补 zsui 检出 | `.github/workflows/build.yml` | 路径推导已核对，**尚未经真实 CI 验证** |
| 戳记来源收口 | `.github/workflows/build.yml` | core-ui job 不再上传 `.revision`，只由 Rust `build.rs` 产出 |
| 决策台账 D17–D20 | `DECISIONS.md` | — |

---

## 2. 本轮已写入、**尚未经真实 CI 验证**的项（下次 CI 是第一次真实验证）

### 2.1 ★ 最高优先级：core-ui 的 C++ 构建（未知能否编过）

**前两次 CI 的失败已被定性为「不是编译错误」**：

- 现象：连续两次（run `43391037201` / `43392589050`）日志都**恰好截断**在
  `[72/97] Building CXX object ... binding.cpp.obj` 这一行，之后**零输出**；
  check-run 的 8 条 annotation **没有一条真正的 clang 报错**，
  全是 CMake 缓存变量（`QJS_BUILD_WERROR: OFF` 等）。
- 根因：GitHub `shell: pwsh` 注入 `$ErrorActionPreference='stop'`，
  叠加 PowerShell 7.4+ 的 `$PSNativeCommandUseErrorActionPreference` 默认 `$true`
  ⇒ clang-cl **只要往 stderr 写一行 warning** 就抛 `NativeCommandExitException`
  ⇒ 管道中断 ⇒ `Tee-Object` 停止 ⇒ **编译器诊断从未落盘**。
- 修法（已在 `build.yml`）：步骤开头设
  ```powershell
  $ErrorActionPreference = 'Continue'
  $PSNativeCommandUseErrorActionPreference = $false
  ```
  并把 annotation 提取正则从 `-match`（不区分大小写，会误命中缓存变量）
  改为 `-cmatch`（区分大小写，只认 clang 的 `file.cpp(12,5): error:` /
  `error C2220:` / `fatal error:` 与 ninja 的 `FAILED:`）。

**下一步**：等这次 CI 跑完 → 读 core-ui job 的逐行 annotation →
- 若**编过了** → 进 §3 的 M1；
- 若**仍失败** → 此时日志里会有**真正的 clang 报错**，按报错定位修。
  （另有一条退路可考虑：上游 core-ui 仓库**没有任何 Windows 构建 CI**
  —— 只有 docs 站点的 `deploy-pages.yml` —— 意味着「clang-cl 能在 runner 上编过」
  这件事上游**从未验证过**。若 clang-cl 路径反复失败，可评估改用
  Visual Studio 自带的 `cl.exe`，或改用本机已有的 GuoheView 定制 DLL 作为过渡。）

### 2.2 package job 的 zsui 检出（路径推导已核对，未实测）

`crates/zsdesk-app` 与 `crates/zsdesk-ui` 的 path 依赖是 `../../../zsui`
⇒ 解析到 `GITHUB_WORKSPACE/zsui`（与 `app` 同级）。
Cargo 解析**整个 workspace** 的 manifest，缺一个 path 依赖就直接失败，
即使本步骤只想构建 `zsdesk-coreui` 一个 crate。
已补 `path: zsui` 的 checkout，ref 取自 `app/packaging/zsui-revision.txt`。

### 2.3 S7' 导出面门禁在真实构建产物上的表现

门禁的基准是「同一 checkout 里 `src/` 的 `UI_API` 定义集」。
**本机用 346 DLL 对 345 源码验证通过**，但 CI 编出的 345 DLL
是否与 346 完全同面，**未知**。
若 CI 编出的 DLL 缺某个符号（例如上游 345 与 346 之间有增删），
门禁会**如实报出** —— 那是需要判断的真实差异，不是误报。

### 2.4 五个 S13 门禁在 CI 里的首次执行

`uix_tag_check` / `theme_contrast_check` / `qa_uix_verify` /
`s13_token_namespace_check` / `ui_id_manifest_check`
本地全绿，但 CI 的**工作目录**（`$GITHUB_WORKSPACE\app`）与本地不同，
路径解析是否一致，**未实测**。

---

## 3. 后续里程碑（未开始）

| 里程碑 | 内容 | 依赖 |
|---|---|---|
| **M1** | Rust 侧真正用 core-ui 装上面板：读 `assets/*.uix` → 注入 `:root` → `ui_page_load_string` → 取 32 个句柄 → 自绘列表/导航树 → GuoheView 预览 | 依赖 §2.1 的 core-ui 构建成功 |
| **M2** | 文本预览上限与编码策略；GuoheView 深度集成 | M1 |
| **M3** | 去掉 zsui path 依赖（workspace 收口）；包体积红线实测 | M1 |
| 主题切换 | `ui_theme_set_mode` → **重建页面** → 重取全部 32 个句柄 + 在新 page 上重新注册 `ui_page_on_widget_mount` 钩子 + 重读系统 AccentColor 并重调 `ui_theme_set_accent_hex` | M1 |

---

## 4. 已知的判断与风险（留给下一轮）

### 4.1 已在源码级核实、但**只存在于文档/注释**、尚无机器门禁保护

- **`:root` 的「先定义者赢」** —— 靠 `value.cpp:572-579` 的 `LookupVar`
  首个命中语义；`:root` 跳过 CSS 级联（`cascade.cpp:125`）是「它只是变量容器」
  的原因，**不是**先定义者赢的原因（因果不要弄反）。
- **注入位置合法性** —— 顶层只允许 `<window>/<template>/<script>/<style>/<import>/<link>`
  （`sfc_parser.cpp:260-266`）；任何非 `<` 开头的顶层内容 ⇒ 整页不加载。
- **`<link>` 的优先级** —— `page_api.cpp:271` 把 link 排在所有 inline `<style>` 之前
  ⇒ 页面若有 `<link>` 且其 CSS 定义了 `:root`，注入值仍会被吃掉。
  **当前三个资产零 `<link>`**，但没有门禁守着这条。
- **BOM** —— 注入是前置的，BOM 会被挤到文件中部 ⇒ 整页不加载；
  `std::fs::read_to_string` 不剥 BOM。

### 4.2 已知的「静默失败」清单（无报错、界面不对）

| 症状 | 根因 |
|---|---|
| 界面全是库默认色，零报错 | 注入缺失 / 注入排在页面 `:root` 之后 / BOM / 变量名不在 `:root` 规则内 |
| 某块区域空白，零报错 | 该宿主是 `<div>` 而非 `<custom>` ⇒ `ui_custom_on_draw` 静默 no-op |
| 切主题颜色不变，零报错 | 同「注入被吃掉」 |
| 改文字没反应 | 句柄已失效（页面重建后）⇒ 往脱离控件树的 widget 写属性是**静默无效** |

### 4.3 已知缺口（有意留白，非本轮范围）

- **UIA / 无障碍**：core-ui **没有** UIA 支持 → 接受降级（用户 2026-11-13 拍板，D13）。
- **窗口材质**：core-ui **没有** Mica/Acrylic → 接受降级（D13）。
- **Rust 侧「注入自检」**：架构要求「位置/顺序/内容/键集」四条断言，
  `theme.rs` 目前在**构造上**保证了这些（只允许 `--app-*`、形态甲保证第一个 `:root`），
  但**没有**再写一个独立的「读回校验」——core-ui 无读回 CSS 变量的 C API，
  只能靠离屏渲染取像素（架构建议一次性冒烟，不进启动路径）。**未做。**
- **`about.uix` 的版本号**仍是写死的文本（已从 346 改成 345），
  理想做法是运行时从 DLL 读；about 页**尚未被 Rust 代码装配**，故不急。

---

## 5. 复现本次收尾的自检命令

```bash
# 五个 S13 门禁
for f in uix_tag_check theme_contrast_check qa_uix_verify \
         s13_token_namespace_check ui_id_manifest_check; do
  python scripts/gates/$f.py --root .
done

# S7' 导出面门禁（用本机 DLL + 上游源码作参照）
python scripts/gates/coreui_export_gate.py \
  --dll "C:/Program Files/GuoheView/core-ui.dll" \
  --source-dir <core-ui 源码>/src --include-dir <core-ui 源码>/include

# PE 导出表解析器的自校验（应输出与 docs/05 完全一致的 567 行）
python scripts/gates/pe_exports.py "C:/Program Files/GuoheView/core-ui.dll" \
  --prefix ui_ | diff - <(grep -oE "^ui_[a-z0-9_]+" docs/05-COREUI-EXPORTS.txt | sort -u)
```

---

## 6. 协作纪律（沿用至今）

1. **git 操作统一由交付总监执行** —— 成员只写文件，不 commit / push。
2. **任何人不得对他人正在编辑的文件做批量改写**。
3. **作者不得自审** —— 产出需另一方独立复核（本轮 QA 与架构师的双向复核
   实际抓出了 6 个 P1 级问题，全部已修）。
4. **未验证的东西必须标注** —— 本文件即为该纪律的载体：
   §1 = 已验证，§2 = 未验证，§3 = 未开始。
