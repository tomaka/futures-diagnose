use crate::{current_task, log_out};
use std::{borrow::Cow, sync::Arc, task::Waker};

/// Takes ownership of a `Waker`, and returns another `Waker` that wraps around it but with
/// logging.
pub(crate) fn waker_with_diag(waker: Waker, task: futures_diagnose_exec_common::Task) -> Waker {
    futures::task::waker(Arc::new(WakerWithDiag {
        inner: waker,
        task,
    }))
}

/// Wraps around a `Waker` and logs a message every time the task is waken up.
struct WakerWithDiag {
    /// The actual waker that does things.
    inner: Waker,
    /// The task that `inner` wakes up.
    task: futures_diagnose_exec_common::Task,
}

impl futures::task::ArcWake for WakerWithDiag {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        log_out::log(futures_diagnose_exec_common::MessageData::TaskWakeUp {
            woken_up: arc_self.task.clone(),
            waker: arc_self.task.clone(),       // TODO: wrong
        });
        arc_self.inner.wake_by_ref();
    }
}
