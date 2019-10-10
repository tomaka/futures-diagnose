use crate::{LEVEL, TARGET, absolute_time, current_task};
use std::{borrow::Cow, sync::Arc, task::Waker};

/// Takes ownership of a `Waker`, and returns another `Waker` that wraps around it but with
/// logging.
pub(crate) fn waker_with_diag(waker: Waker, task_id: Cow<'static, str>) -> Waker {
    futures::task::waker(Arc::new(WakerWithDiag {
        inner: waker,
        task_id,
    }))
}

/// Wraps around a `Waker` and logs a message every time the task is waken up.
struct WakerWithDiag {
    /// The actual waker that does things.
    inner: Waker,
    /// Name of the task that `inner` wakes up.
    task_id: Cow<'static, str>,
}

impl futures::task::ArcWake for WakerWithDiag {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let point_in_time = absolute_time::now_since_abs_time();
        let current_thread_id = std::thread::current().id();
        let cur_thread = std::thread::current();
        let current_thread_name = cur_thread.name();
        log::log!(target: TARGET, LEVEL, "At {:?}, task {:?} got woken up by {:?}; name = {:?}; task = {:?}", point_in_time, arc_self.task_id, current_thread_id, current_thread_name, current_task::current_task());
        arc_self.inner.wake_by_ref()
    }
}
