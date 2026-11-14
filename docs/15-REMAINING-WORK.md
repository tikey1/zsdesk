# 剩余工作流程（2026-11-13 收尾快照）

> 本文件记录**代码停止编写时**的未完成事项。
> 上一次已推送到 GitHub 的状态见 §1；本轮新增但**尚未验证**的改动见 §2；
> 后续里程碑见 §3；已知的判断与风险见 §4。

---

## 0. 一句话现状

Rust 侧的 core-ui 绑定层（`crates/zsdesk-coreui/`）与 CI 契约已经落地，
**六个机械门禁全部能在 CI 里真正执行**；
**core-ui 的 C++ 源码构建已经在 runner 上成功过一次** ——
run `43398079661`（第 9 次 CI）的 core-ui job 第 10 步
「Configure and build core-ui from source」= **success**，
随后的「DLL 是 Windows x86-64 映像」「Gate G2 版本戳记一致」「Gate S7 动态 VC++ 运行时」
**全部转绿**（详见 §2.1）。
仅剩两处待收口：**Rust job 的 `cargo test`**（doctest 里的 crate 名写错，已修未验）
与 **Gate S7' 导出面门禁**（原因未定位，已加 annotation 探针，下次 CI 自曝）。

---

## 0.1 第 9 次 CI（run `43398079661`）实测结果

| job | 步骤 | 结果 | 结论 |
|---|---|---|---|
| core-ui | 10. Configure and build core-ui from source | **success** | ★ **clang-cl 路径成立**，前 4 次失败确实是日志管道被中断，不是编译错误 |
| core-ui | 12. Verify core-ui.dll is a Windows x86-64 image | success | 产物架构正确 |
| core-ui | 13. Gate G2 — the DLL version must match the pinned revision | success | 戳记链路正确 |
| core-ui | 14. Gate S7 — core-ui.dll must link the dynamic VC++ runtime | success | `/MD` 动态 CRT 正确 |
| core-ui | 15. Gate S7' — the DLL export surface must match the source contract | **failure** | **原因未定位**（日志需 admin 才能下载）→ 本次已加 `::error::` 探针 |
| Rust | 6. Test | **failure** | 根因已定位：`lib.rs` 的 doctest 仍写骨架期的 crate 名 `zc_ffi_coreui` → 已改为 `zsdesk_coreui` |
| package | 全部 | skipped | 上游 job 失败 ⇒ 未执行 |

**关键限制**：GitHub 的 job 日志下载端点
`GET /repos/{owner}/{repo}/actions/jobs/{job_id}/logs` 要求 admin（实测 403
`Must have admin rights to Repository.`），
**外部唯一可读的通道是 check-run annotations**（公开可读，已实测可取）。
⇒ **凡是会失败的步骤，都必须把失败原因打成 `::error::` annotation**，
否则失败原因对外完全不可见（本轮已对 `cargo test` / S13 门禁 / Gate S7' 三处补上）。


---

## 1. 本轮已写入磁盘、**已通过本地验证**、随本次提交推送

| 项 | 文件 | 验证方式 |
|---|---|---|
| `:root` 注入模块（形态甲） | `crates/zsdesk-coreui/src/lib/theme.rs` | tree-sitter 语法零错误 + 9 条单元测试 + 5 条等价逻辑用例 |
| 注入路径进 CI 探针 | `crates/zsdesk-coreui/examples/probe.rs` | tree-sitter 语法零错误 |
| PE 导出表解析器 | `scripts/gates/pe_exports.py` | 用本机 `GuoheView\core-ui.dll` 实测，**逐字节复现** `docs/05` 的 567 个 `ui_*` 基线 |
| 导出面门禁 S7' | `scripts/gates/coreui_export_gate.py` | 正例通过；反例（截断 DLL / 无 `ui_*` 的 kernel32）均正确失败 |
| id 清单门禁 | `scripts/gates/ui_id_manifest_check.py` + `assets/ui-ids.json` | 本地通过；注入 typo 后正确失败 |
| CI 日志中断修复 | `.github/workflows/build.yml` | **第 9 次 CI 已证实有效**：core-ui 源码构建 success，日志完整落盘 3057 行，真实 clang warning 可见（不再是「恰好截断」） |
| CI 门禁接线（5 个 S13 门禁 + S7'） | `.github/workflows/build.yml` | 本地逐个跑通；**S7' 已在第 9 次 CI 里真正执行并失败** ⇒ 门禁确实有执行器，不是建议 |
| package job 补 zsui 检出 | `.github/workflows/build.yml` | 路径推导已核对；第 9 次 CI 因上游 job 失败被 skipped，**仍未实测** |
| 戳记来源收口 | `.github/workflows/build.yml` | core-ui job 不再上传 `.revision`，只由 Rust `build.rs` 产出 |
| doctest crate 名修正 | `crates/zsdesk-coreui/src/lib.rs` | `zc_ffi_coreui` → `zsdesk_coreui`（2 处），全仓无残留 |
| Python 版本钉死 + 3.9 兼容修正 | `.github/workflows/build.yml` + `scripts/gates/{pe_exports,coreui_export_gate,ui_id_manifest_check}.py` | 两个 job 各加 `setup-python@v5`（3.11）；ast 扫描确认全部门禁脚本 3.9 兼容 |
| 失败原因可视化（annotation 探针） | `.github/workflows/build.yml` | `cargo test` / S13 门禁 / Gate S7' 三处补 `::error::`，失败原因可经**公开** check-run annotations 读取 |
| 决策台账 D17–D20 | `DECISIONS.md` | — |

---

## 2. 待第 10 次 CI 收口的三件事

> 第 9 次 CI 已经跑过：core-ui 的 C++ 构建**成功**（§2.1 关闭），
> 但 rust / core-ui 两个 job 仍各有一处失败（§2.1b / §2.1c），package job 被连带跳过。
> 本节只列**仍未验证**的项；已由第 9 次 CI 证实有效的见 §1。

### 2.1 ✅ 已关闭：core-ui 的 C++ 构建

**run `43398079661` 的 core-ui job 第 10 步 = success**，随后的
「DLL 是 Windows x86-64 映像」/「Gate G2 戳记一致」/「Gate S7 动态 VC++ 运行时」全绿。
至此 **clang-cl + Ninja + llvm-rc 路径在我们 pin 的 revision 上成立**，M0 的最大未知已消除。

前 4 次失败的真实成因（留档，避免以后重踩）：

- 现象：日志都**恰好截断**在 `[72/97] ... binding.cpp.obj` 之后零输出；
  check-run 的 annotation 里全是 CMake 缓存变量，没有一条真正的 clang 报错。
- 根因：GitHub `shell: pwsh` 注入 `$ErrorActionPreference='stop'`，叠加 PowerShell 7.4+
  的 `$PSNativeCommandUseErrorActionPreference` 默认 `$true` ⇒ clang-cl 只要往 stderr
  写一行 warning 就抛 `NativeCommandExitException` ⇒ 管道中断 ⇒ `Tee-Object` 停止
  ⇒ **编译器诊断从未落盘**。
- 修法（已进 `build.yml`，已由第 9 次 CI 证实）：步骤开头设
  `$ErrorActionPreference='Continue'` + `$PSNativeCommandUseErrorActionPreference=$false`；
  annotation 提取正则 `-match` → **`-cmatch`**（前者不区分大小写，会把含 `ERROR` 的
  CMake 缓存变量全命中、占满 10 条配额）。

**保留的退路**：上游 core-ui 仓库**没有任何 Windows 构建 CI**（只有 docs 站点的
`deploy-pages.yml`）—— 「clang-cl 能编过」上游从未验证，现在只是**我们**验证过了。
若将来 pin 的 revision 变更导致该路径反复失败，可评估改用 VS 的 `cl.exe`，
或过渡期直接用本机 GuoheView 的定制 DLL。**当前不需要走退路。**

### 2.1b Rust job 的 `cargo test` —— 根因已定位、已修、待复验

第 9 次 CI 的 rust job 在第 6 步 `Test` 失败。
根因：`crates/zsdesk-coreui/src/lib.rs` 的模块文档 doctest 仍写骨架期的 crate 名
`zc_ffi_coreui`（2 处），而包名 `zsdesk-coreui` ⇒ Rust crate 名是 `zsdesk_coreui`
⇒ doctest 编译失败 ⇒ `cargo test --workspace --release --locked` 退出码 1。
已改为 `zsdesk_coreui`，全仓无残留。
⚠️ **本机无 Rust 工具链且缺 `zsui` path 依赖 ⇒ 无法本地复跑 `cargo test`**：
doctest 名字写错是**确定性失败点**，但「修掉它之后是否还有别的编译/测试失败」**只能由第 10 次 CI 回答**。
本次已给该步骤补 `::error::` 探针，把 `error[...]` / `FAILED` / `panicked` 等关键行打进 annotations。
**因该步骤失败，后面的 S13 门禁步骤与 `cargo build` 步骤均未执行** ⇒ §2.4 仍待验。

### 2.1c Gate S7' 导出面门禁 —— 原因未定位，已加三层探针

第 9 次 CI 里 S7' 失败但**外部看不到任何输出**：job 日志下载端点需 admin
（实测 403 `Must have admin rights to Repository.`），而脚本自身不产生 annotation。
未拿到真实报错前，不声称根因已确定。已做的三层处置：

1. **推定根因已修**：runner 自带 `python` 的版本会随镜像漂移，而当时的门禁脚本用了
   3.10+ 语法（模块级 `frozenset[str]` 注解、局部变量 `str | None`）——
   `from __future__ import annotations` **不作用于**模块级/局部变量注解，在 3.9 上会直接抛错。
   已全部改成 `# type:` 注释形式；ast 扫描确认 7 个门禁脚本均 3.9 兼容。
2. **消除漂移**：rust / core-ui 两个 job 各加 `setup-python@v5`（固定 **3.11**）
   ⇒ 门禁行为只取决于脚本，不取决于 runner 镜像。
3. **失败必须可见**：`cargo test` / S13 门禁 / Gate S7' 三处补 `::error::` annotation，
   把脚本末尾 20–25 行判定理由打进公开 check-run annotations。

**下一次 CI 的判读方式**：若 S7' 转绿 ⇒ 推定成立；若仍红 ⇒ annotations 里会直接给出
真实报错（缺符号 / 源码扫描为空 / PE 解析失败 / 路径不存在），按报错定位，不再盲猜。

### 2.2 package job 的 zsui 检出（仍未实测）

`crates/zsdesk-app` 与 `crates/zsdesk-ui` 的 path 依赖是 `../../../zsui`
⇒ 解析到 `GITHUB_WORKSPACE/zsui`（与 `app` 同级）。
Cargo 解析**整个 workspace** 的 manifest，缺一个 path 依赖就直接失败，
即使本步骤只想构建 `zsdesk-coreui` 一个 crate。
已补 `path: zsui` 的 checkout，ref 取自 `app/packaging/zsui-revision.txt`。
**第 9 次 CI 的 package job 因上游两 job 失败被 skipped** ⇒ 这条修复仍未经实测。
第 10 次 CI 若 rust / core-ui 都转绿，package job 将**首次真正执行**，
那时才能确认 zsui 检出、戳记比对、ZIP 组装三条链路。

### 2.3 S7' 的判定基准（口径留档，不是未验证项）

门禁基准 = **同一 checkout 里 `src/` 的 `UI_API` 定义集**，不是 `docs/05`
（那是本机 **346** 预编译 DLL 的实测面，CI 编的是 **345**，跨版本比对只得版本漂移），
也不是 `include/ui_core.h`（上游漏声明 12 个符号：`ui_image_gdi_*`×10 +
`ui_flush_gpu` + `ui_image_view_gdi` + `ui_image_evict_tile`）。
`ui_dialog` 头文件有、源码无 ⇒ **双向容忍并报告**。
本机用 346 DLL 对 345 源码验证通过；第 9 次 CI 里 S7' 已真正执行（失败原因见 §2.1c）。

### 2.4 五个 S13 门禁在 CI 里的首次执行（仍未发生）

`uix_tag_check` / `theme_contrast_check` / `qa_uix_verify` /
`s13_token_namespace_check` / `ui_id_manifest_check`
本地全绿，但**第 9 次 CI 的 rust job 在第 6 步 `Test` 就失败，后续步骤全部跳过**
⇒ 门禁步骤一次都没跑到。CI 的工作目录（`$GITHUB_WORKSPACE\app`）与本地不同，
路径解析是否一致，**仍未实测**。本次已给门禁步骤补 `::error::` 探针，
失败时会把判定理由打进 annotations。

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

### 5.1 读 CI 结果（不需要 admin 权限的通道）

job 日志下载端点要 admin（实测 403），但**对 public repo，check-run annotations 公开可读**。
本次已把 `cargo test` / S13 门禁 / Gate S7' 的失败原因打进 `::error::`，按下面读：

```bash
# 1) 最近几次 run
curl -s "https://api.github.com/repos/tikey1/zsdesk/actions/runs?per_page=5" \
  | python -c "import json,sys;[print(r['id'],r['conclusion'],r['created_at']) for r in json.load(sys.stdin)['workflow_runs']]"

# 2) 该 run 的三个 job（Actions 的 job id == check-run id）
curl -s "https://api.github.com/repos/tikey1/zsdesk/actions/runs/<RUN_ID>/jobs" \
  | python -c "import json,sys;[print(j['id'],j['name'],j['conclusion']) for j in json.load(sys.stdin)['jobs']]"

# 3) 失败原因
curl -s "https://api.github.com/repos/tikey1/zsdesk/check-runs/<JOB_ID>/annotations?per_page=100" \
  | python -c "import json,sys;[print('-',a['message']) for a in json.load(sys.stdin)]"
```

---

## 6. 协作纪律（沿用至今）

1. **git 操作统一由交付总监执行** —— 成员只写文件，不 commit / push。
2. **任何人不得对他人正在编辑的文件做批量改写**。
3. **作者不得自审** —— 产出需另一方独立复核（本轮 QA 与架构师的双向复核
   实际抓出了 6 个 P1 级问题，全部已修）。
4. **未验证的东西必须标注** —— 本文件即为该纪律的载体：
   §1 = 已验证，§2 = 未验证，§3 = 未开始。
