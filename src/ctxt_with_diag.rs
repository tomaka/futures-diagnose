use crate::log_out;
use std::{borrow::Cow, sync::Arc, task::Waker};

/// Takes ownership of a `Waker`, and returns another `Waker` that wraps around it but with
/// logging.
pub(crate) fn waker_with_diag(waker: Waker, task_name: Cow<'static, str>, task_id: u64) -> Waker {
    futures::task::waker(Arc::new(WakerWithDiag { inner: waker, task_name, task_id }))
}

/// Wraps around a `Waker` and logs a message every time the task is waken up.
struct WakerWithDiag {
    /// The actual waker that does things.
    inner: Waker,
    task_name: Cow<'static, str>,
    task_id: u64,
}

impl futures::task::ArcWake for WakerWithDiag {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        log_out::log_wake_up(&arc_self.task_name, arc_self.task_id);
        arc_self.inner.wake_by_ref();
    }
}
