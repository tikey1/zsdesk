//! UI 线程收敛。
//!
//! ## 规则（不可协商）
//!
//! > **跨出去的才校验，没跨出去的信类型。** —— 这里反过来用：
//! > **只有 UI 线程能碰 core-ui。工作线程一律通过 `invoke_sync` 封送。**
//!
//! `docs/c-api.md` 明示：「工作线程里访问 widget 前先 marshal 到 UI 线程」。
//! core-ui 内部没有为 widget 树做加锁，跨线程调用是数据竞争，不是「慢一点」。
//!
//! ## 为什么不用 `Arc<CoreUi>`
//!
//! `libloading::Library` 确实实现了 `Send + Sync`（libloading 0.8.9 的
//! `safe.rs` 末尾有 `unsafe impl Send for Library {}` / `unsafe impl Sync for Library {}`），
//! 所以 `Arc<CoreUi>` 在技术上可行。
//!
//! 但本模块**故意不用**：`Arc<CoreUi>` 允许任何持有者直接调用 core-ui，
//! 于是「调用必须发生在 UI 线程」这条规则就退化成一句约定。本模块的做法是：
//!
//! - **UI 线程独占 `CoreUi`**（不共享），跨线程只共享一个只含裸指针
//!   与函数指针的轻量句柄 `UiHandle`；
//! - `UiHandle` 里的 `*const CoreUi` **只会在 trampoline 里被解引用**，
//!   而 trampoline 由 core-ui 保证在 UI 线程上执行；
//! - `UiHandle` 里的 `window_invoke_sync` 函数指针在构造时（UI 线程）拷出来，
//!   工作线程调用时**根本不碰** `CoreUi`；
//! - `CoreUi` 在加载后其内容（`api` / `optional` / `version`）不再变化，
//!   所以「UI 线程读它」与「工作线程读函数指针副本」之间没有数据竞争。
//!
//! 换句话说：**类型系统替我们挡住了越界调用**，而不是靠 review。
//!
//! ## 死锁红线
//!
//! `invoke_sync` 是**阻塞**的：工作线程把任务塞进 UI 线程的队列，等它跑完。
//! 因此：
//!
//! - **UI 线程绝不能 join 一个正在 `invoke_sync` 的工作线程** —— 会死锁。
//! - 关闭顺序必须是：先让工作线程停止投递 → join 完 → 再退出 `ui_run()`。
//! - 工作线程的 `invoke_sync` 必须在 UI 线程还在跑消息循环时才有意义；
//!   `ui_run()` 返回后调用会挂住，因此 `UiHandle` 在 `ui_run()` 返回后即失效。

use std::marker::PhantomData;
use std::sync::Arc;
use std::thread::ThreadId;

use crate::api::{UiInvokeCallback, UiWindow};
use crate::loader::CoreUi;

/// 跨线程任务：一个「在 UI 线程上执行的闭包」。
struct UiTask {
    /// 只在 trampoline（UI 线程）里解引用。
    core: *const CoreUi,
    /// 一次性闭包。`None` 表示已被取走。
    action: Option<Box<dyn FnOnce(&CoreUi) + Send>>,
}

// SAFETY：`UiTask` 只在「创建它的线程」与「UI 线程」之间移动一次，
// 由 `ui_window_invoke_sync` 内部的 SendMessage 保证投递到 UI 线程。
// `core` 裸指针只在 UI 线程解引用；`action` 是 Send 的。
unsafe impl Send for UiTask {}

unsafe extern "C" fn task_trampoline(ud: *mut std::ffi::c_void) {
    // SAFETY：由 `invoke_sync` 保证 `ud` 指向一个存活的 `UiTask`，且本次调用
    // 是同步的（`ui_window_invoke_sync` 返回前回调一定已执行完）。
    let task = unsafe { &mut *(ud as *mut UiTask) };
    if let Some(action) = task.action.take() {
        // SAFETY：trampoline 由 core-ui 在 UI 线程上调用。
        let core = unsafe { &*task.core };
        action(core);
    }
}

/// 跨线程句柄。可 `Copy`，可交给工作线程。
#[derive(Clone, Copy)]
pub struct UiHandle {
    /// 从 UI 线程拷出来的封送函数指针，工作线程调用它时不需要解引用 `CoreUi`。
    invoke_sync: UiInvokeCallback,
    /// 只在 UI 线程解引用。
    core: *const CoreUi,
    window: UiWindow,
    /// 拥有该 UI 线程的线程 id。
    owner: ThreadId,
}

// SAFETY：`UiHandle` 不拥有 `CoreUi`，只是持有指针。
// - `core` 只在 UI 线程解引用（trampoline）。
// - `invoke_sync` 是不可变函数指针，`Copy` 语义。
// - `window` / `owner` 是不可变值。
unsafe impl Send for UiHandle {}
unsafe impl Sync for UiHandle {}

impl UiHandle {
    /// 当前线程是不是 UI 线程。
    pub fn is_ui_thread(&self) -> bool {
        std::thread::current().id() == self.owner
    }

    /// 不在 UI 线程就 panic，并说清是哪条规则被违反。
    ///
    /// 放在所有「直接调用 core-ui」的安全包装函数入口。
    pub fn assert_ui_thread(&self) {
        assert!(
            self.is_ui_thread(),
            "core-ui 调用必须发生在 UI 线程（违反 docs/c-api.md 的线程约束）"
        );
    }

    /// 把一段代码封送到 UI 线程同步执行；返回前一定已执行完。
    ///
    /// `action` 必须是 `Send`：它会被搬到 UI 线程上执行。
    pub fn invoke_sync<F>(&self, action: F)
    where
        F: FnOnce(&CoreUi) + Send + 'static,
    {
        if self.is_ui_thread() {
            // SAFETY：当前就是 UI 线程，直接调用。
            let core = unsafe { &*self.core };
            action(core);
            return;
        }

        let mut task = Box::new(UiTask {
            core: self.core,
            action: Some(Box::new(action)),
        });
        let ud = &mut *task as *mut UiTask as *mut std::ffi::c_void;
        // SAFETY：`task` 在栈上存活到本函数返回；`invoke_sync` 是同步的，
        // 返回时回调一定已经跑完，`task` 随后被 drop。
        unsafe { (self.invoke_sync)(self.window, task_trampoline, ud) };
    }
}

/// UI 线程宿主：独占 `CoreUi`，并提供跨线程句柄。
///
/// **生命周期顺序（不可颠倒）**：
/// 1. `UiThread::new` —— 在 UI 线程上创建；
/// 2. 页面 / 窗口创建与事件绑定；
/// 3. 工作线程启动（拿 `handle()` 副本）；
/// 4. `ui_run()` 阻塞；
/// 5. `ui_run()` 返回后：**先 join 所有工作线程，再销毁页面与窗口**；
/// 6. `UiThread` drop → 卸载 DLL。
///
/// 第 5 步的顺序不可换：工作线程还可能在 `invoke_sync` 里排队，
/// 而 `invoke_sync` 在 `ui_run()` 返回后无人处理。
pub struct UiThread {
    core: CoreUi,
    window: UiWindow,
    owner: ThreadId,
    /// `PhantomData` 提醒：非 Send/Sync，只能待在 UI 线程。
    _not_send: PhantomData<*const ()>,
}

impl UiThread {
    /// 在当前线程上创建 UI 线程宿主。
    pub fn new(core: CoreUi, window: UiWindow) -> Self {
        UiThread {
            core,
            window,
            owner: std::thread::current().id(),
            _not_send: PhantomData,
        }
    }

    /// 跨线程句柄（可复制、可发给工作线程）。
    pub fn handle(&self) -> UiHandle {
        UiHandle {
            // 从 UI 线程拷出函数指针：工作线程不再需要解引用 `CoreUi`。
            invoke_sync: self.core.api.window_invoke_sync,
            core: &self.core as *const CoreUi,
            window: self.window,
            owner: self.owner,
        }
    }

    /// 仅在 UI 线程：直接访问 core-ui。
    pub fn core(&self) -> &CoreUi {
        assert!(
            std::thread::current().id() == self.owner,
            "UiThread::core() 只能在 UI 线程调用"
        );
        &self.core
    }

    /// 仅在 UI 线程：访问 API 表。
    pub fn api(&self) -> &crate::loader::Api {
        &self.core().api
    }

    /// 仅在 UI 线程：访问可选 API 表。
    pub fn optional(&self) -> &crate::loader::OptionalApi {
        &self.core().optional
    }

    /// 窗口句柄（可在任意线程读）。
    pub fn window(&self) -> UiWindow {
        self.window
    }

    /// 当前线程是不是 UI 线程。
    pub fn is_ui_thread(&self) -> bool {
        std::thread::current().id() == self.owner
    }
}

/// 便捷别名：让上层写 `Arc<UiHandle>` 而不是 `Arc<UiThread>`。
pub type SharedUi = Arc<UiHandle>;
