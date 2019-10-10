use crate::{TARGET, LEVEL, absolute_time, current_task};
use std::{borrow::Cow, sync::Arc, task::Context, task::Waker, time::Instant};

/// Wraps around a `Waker` and logs a message every time the task is waken up.
pub struct WakerWithDiag {
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

impl WakerWithDiag {
    pub fn new(inner: &Waker, task_id: Cow<'static, str>) -> WakerWithDiag {
        WakerWithDiag {
            inner: inner.clone(),
            task_id,
        }
    }

    pub fn into_waker(self) -> Waker {
        futures::task::waker(Arc::new(self))
    }
}

#[cfg(test)]
mod tests {
    use futures::{prelude::*, channel::oneshot, ready};
    use std::{pin::Pin, task::Context, task::Poll};
    use super::*;

    #[test]
    fn api_works() {
        #![allow(unused)]
        struct MyFut(oneshot::Receiver<()>);
        impl Future for MyFut {
            type Output = ();
            fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
                let mut with_diag = WakerWithDiag::from(cx.waker());
                let _ = ready!(Future::poll(Pin::new(&mut self.0), &mut with_diag.context()));
                Poll::Ready(())
            }
        }
    }
}
