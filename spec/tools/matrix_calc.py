import collections, os, sys

BASE = r"C:/Users/tx/AppData/Local/Temp/upstream/zsdesk-main"

CLS = {
"crates/zsdesk-core/src/error.rs":"直接复用",
"crates/zsdesk-core/src/lib.rs":"直接复用",
"crates/zsdesk-core/src/model.rs":"直接复用",
"crates/zsdesk-core/src/model/desktop.rs":"直接复用",
"crates/zsdesk-core/src/model/explorer.rs":"直接复用",
"crates/zsdesk-core/src/model/tab.rs":"直接复用",
"crates/zsdesk-core/src/model/view.rs":"直接复用",
"crates/zsdesk-core/src/path.rs":"直接复用",
"crates/zsdesk-platform/src/lib.rs":"直接复用",
"crates/zsdesk-platform/src/shell.rs":"改造",
"crates/zsdesk-ui/src/lib.rs":"重写",
"crates/zsdesk-ui/src/chrome.rs":"重写",
"crates/zsdesk-ui/src/menus.rs":"改造",
"crates/zsdesk-ui/src/theme.rs":"改造",
"crates/zsdesk-windows/build.rs":"直接复用",
"crates/zsdesk-windows/src/desktop.rs":"直接复用",
"crates/zsdesk-windows/src/enumerate.rs":"直接复用",
"crates/zsdesk-windows/src/flyout.rs":"改造",
"crates/zsdesk-windows/src/flyout_layout.rs":"改造",
"crates/zsdesk-windows/src/gfx.rs":"改造",
"crates/zsdesk-windows/src/integration.rs":"直接复用",
"crates/zsdesk-windows/src/lib.rs":"直接复用",
"crates/zsdesk-windows/src/ole/drop.rs":"改造",
"crates/zsdesk-windows/src/ole/mod.rs":"直接复用",
"crates/zsdesk-windows/src/shell/commands.rs":"直接复用",
"crates/zsdesk-windows/src/shell/events.rs":"改造",
"crates/zsdesk-windows/src/shell/explorer_browser.rs":"改造",
"crates/zsdesk-windows/src/shell/file_transfer.rs":"直接复用",
"crates/zsdesk-windows/src/shell/file_transfer_tests.rs":"直接复用",
"crates/zsdesk-windows/src/shell/icons.rs":"直接复用",
"crates/zsdesk-windows/src/shell/icons_alpha_tests.rs":"直接复用",
"crates/zsdesk-windows/src/shell/menu.rs":"改造",
"crates/zsdesk-windows/src/shell/menu_runtime_tests.rs":"改造",
"crates/zsdesk-windows/src/shell/mod.rs":"直接复用",
"crates/zsdesk-windows/src/shell/namespace_tree.rs":"重写",
"crates/zsdesk-windows/src/shell/new_menu.rs":"改造",
"crates/zsdesk-windows/src/shell/notify.rs":"直接复用",
"crates/zsdesk-windows/src/shell/pidl.rs":"直接复用",
"crates/zsdesk-windows/src/shell/quick_menu.rs":"改造",
"crates/zsdesk-windows/src/shell/search.rs":"直接复用",
"crates/zsdesk-windows/src/shell/shell_item.rs":"直接复用",
"crates/zsdesk-windows/src/shell/shell_item_contract_tests.rs":"直接复用",
"crates/zsdesk-windows/src/shell/view_background.rs":"删除",
"crates/zsdesk-windows/src/shell/view_state.rs":"改造",
"crates/zsdesk-windows/src/taskbar.rs":"直接复用",
"crates/zsdesk-windows/src/theme.rs":"改造",
"crates/zsdesk-app/build.rs":"直接复用",
"crates/zsdesk-app/examples/desktop_probe.rs":"直接复用",
"crates/zsdesk-app/examples/enum_probe.rs":"直接复用",
"crates/zsdesk-app/src/app.rs":"改造",
"crates/zsdesk-app/src/commands.rs":"改造",
"crates/zsdesk-app/src/context_menu_background.rs":"改造",
"crates/zsdesk-app/src/context_menu_process.rs":"改造",
"crates/zsdesk-app/src/context_menu_process_tests.rs":"改造",
"crates/zsdesk-app/src/context_preferences.rs":"直接复用",
"crates/zsdesk-app/src/desktop/context_menu_regression.rs":"改造",
"crates/zsdesk-app/src/desktop/fence.rs":"直接复用",
"crates/zsdesk-app/src/desktop/host.rs":"改造",
"crates/zsdesk-app/src/desktop/layer_regression.rs":"改造",
"crates/zsdesk-app/src/desktop/menu_worker.rs":"直接复用",
"crates/zsdesk-app/src/desktop/migration.rs":"直接复用",
"crates/zsdesk-app/src/desktop/mod.rs":"改造",
"crates/zsdesk-app/src/desktop/return_to_desktop.rs":"直接复用",
"crates/zsdesk-app/src/desktop/tencent_import.rs":"直接复用",
"crates/zsdesk-app/src/desktop/window_message_regression.rs":"改造",
"crates/zsdesk-app/src/edition.rs":"直接复用",
"crates/zsdesk-app/src/explorer_forwarder.rs":"直接复用",
"crates/zsdesk-app/src/explorer_forwarder/visibility_guard.rs":"直接复用",
"crates/zsdesk-app/src/fast_search.rs":"直接复用",
"crates/zsdesk-app/src/fast_search/everything.rs":"直接复用",
"crates/zsdesk-app/src/fast_search/everything_runtime.rs":"直接复用",
"crates/zsdesk-app/src/fast_search/metadata.rs":"直接复用",
"crates/zsdesk-app/src/fast_search/native.rs":"直接复用",
"crates/zsdesk-app/src/fast_search/native_scan.rs":"直接复用",
"crates/zsdesk-app/src/fast_search/ntfs.rs":"直接复用",
"crates/zsdesk-app/src/fast_search/query.rs":"直接复用",
"crates/zsdesk-app/src/fast_search/runtime_regression.rs":"直接复用",
"crates/zsdesk-app/src/fast_search/snapshot.rs":"直接复用",
"crates/zsdesk-app/src/flyout_menu.rs":"改造",
"crates/zsdesk-app/src/framework_windows.rs":"删除",
"crates/zsdesk-app/src/instance.rs":"改造",
"crates/zsdesk-app/src/main.rs":"重写",
"crates/zsdesk-app/src/quick_search/control.rs":"改造",
"crates/zsdesk-app/src/quick_search/mod.rs":"直接复用",
"crates/zsdesk-app/src/quick_search/resident.rs":"直接复用",
"crates/zsdesk-app/src/reveal.rs":"直接复用",
"crates/zsdesk-app/src/routing_control.rs":"直接复用",
"crates/zsdesk-app/src/runtime.rs":"改造",
"crates/zsdesk-app/src/shell_window_registry.rs":"改造",
"crates/zsdesk-app/src/storage.rs":"直接复用",
"crates/zsdesk-app/src/win_e_bridge.rs":"改造",
"crates/zsdesk-app/src/window/address_input.rs":"改造",
"crates/zsdesk-app/src/window/bookmarks.rs":"直接复用",
"crates/zsdesk-app/src/window/context_menu.rs":"改造",
"crates/zsdesk-app/src/window/context_menu_regression.rs":"改造",
"crates/zsdesk-app/src/window/deferred_selection.rs":"改造",
"crates/zsdesk-app/src/window/diagnostics.rs":"改造",
"crates/zsdesk-app/src/window/dual_pane.rs":"重写",
"crates/zsdesk-app/src/window/everything_regression.rs":"直接复用",
"crates/zsdesk-app/src/window/folder_sizes.rs":"直接复用",
"crates/zsdesk-app/src/window/geometry.rs":"改造",
"crates/zsdesk-app/src/window/geometry_regression.rs":"改造",
"crates/zsdesk-app/src/window/global_search.rs":"改造",
"crates/zsdesk-app/src/window/hover_preview.rs":"改造",
"crates/zsdesk-app/src/window/image_decode.rs":"直接复用",
"crates/zsdesk-app/src/window/input.rs":"重写",
"crates/zsdesk-app/src/window/integrated_search.rs":"直接复用",
"crates/zsdesk-app/src/window/integration_navigation_regression.rs":"直接复用",
"crates/zsdesk-app/src/window/interaction_regression.rs":"改造",
"crates/zsdesk-app/src/window/keyboard.rs":"改造",
"crates/zsdesk-app/src/window/listary.rs":"直接复用",
"crates/zsdesk-app/src/window/mod.rs":"重写",
"crates/zsdesk-app/src/window/mouse_gestures.rs":"重写",
"crates/zsdesk-app/src/window/mouse_navigation.rs":"直接复用",
"crates/zsdesk-app/src/window/navigation.rs":"改造",
"crates/zsdesk-app/src/window/paint.rs":"重写",
"crates/zsdesk-app/src/window/pane_input.rs":"改造",
"crates/zsdesk-app/src/window/preview_handlers.rs":"改造",
"crates/zsdesk-app/src/window/preview_lifecycle_regression.rs":"改造",
"crates/zsdesk-app/src/window/preview_process.rs":"改造",
"crates/zsdesk-app/src/window/preview_selection.rs":"直接复用",
"crates/zsdesk-app/src/window/quick_preview.rs":"改造",
"crates/zsdesk-app/src/window/quicklook.rs":"改造",
"crates/zsdesk-app/src/window/quicklook_process_tests.rs":"改造",
"crates/zsdesk-app/src/window/routing_settings.rs":"直接复用",
"crates/zsdesk-app/src/window/search_panel.rs":"改造",
"crates/zsdesk-app/src/window/search_regression.rs":"改造",
"crates/zsdesk-app/src/window/selection_regression.rs":"改造",
"crates/zsdesk-app/src/window/shell_registration.rs":"直接复用",
"crates/zsdesk-app/src/window/shell_reveal_regression.rs":"直接复用",
"crates/zsdesk-app/src/window/startup_settings.rs":"直接复用",
"crates/zsdesk-app/src/window/terminal.rs":"改造",
"crates/zsdesk-app/src/window/thumbnail_regression.rs":"改造",
"crates/zsdesk-app/src/window/view_memory.rs":"直接复用",
"crates/zsdesk-app/src/window/wheel_input.rs":"重写",
"crates/zsdesk-app/src/window/window_actions.rs":"改造",
"crates/zsdesk-app/src/window/wps_preview.rs":"删除",
"crates/zsdesk-app/src/window/wps_render_process.rs":"删除",
}

# sanity: every .rs file classified exactly once
actual = set()
for root, dirs, files in os.walk(os.path.join(BASE, "crates")):
    for f in files:
        if f.endswith(".rs"):
            actual.add(os.path.relpath(os.path.join(root, f), BASE).replace("\\", "/"))

cls = set(CLS)
print("未分类:", sorted(actual - cls))
print("多余:", sorted(cls - actual))
print("总数 actual=%d classified=%d" % (len(actual), len(cls)))

tot = collections.defaultdict(lambda: [0, 0])
pc = collections.defaultdict(lambda: collections.defaultdict(lambda: [0, 0]))
for p, c in CLS.items():
    n = sum(1 for _ in open(os.path.join(BASE, p), encoding="utf-8", errors="replace"))
    tot[c][0] += 1; tot[c][1] += n
    crate = p.split("/")[1]
    pc[crate][c][0] += 1; pc[crate][c][1] += n

print("\n=== 总计 ===")
tf = tl = 0
for c in ["直接复用", "改造", "重写", "删除"]:
    print("%s: %d files, %d lines" % (c, tot[c][0], tot[c][1]))
    tf += tot[c][0]; tl += tot[c][1]
print("TOTAL: %d files, %d lines" % (tf, tl))

print("\n=== 分 crate ===")
for crate in ["zsdesk-core", "zsdesk-platform", "zsdesk-ui", "zsdesk-windows", "zsdesk-app"]:
    d = pc[crate]
    s = sum(v[0] for v in d.values()); sl = sum(v[1] for v in d.values())
    parts = " | ".join("%s %d文件/%d行" % (c, d[c][0], d[c][1]) for c in ["直接复用", "改造", "重写", "删除"] if c in d)
    print("%-16s %2d文件 %5d行  %s" % (crate, s, sl, parts))
