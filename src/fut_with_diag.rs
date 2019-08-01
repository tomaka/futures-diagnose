use crate::{LEVEL, ctxt_with_diag, current_task};
use std::sync::atomic::{AtomicU64, Ordering};
use std::{future::Future, pin::Pin, task::Context, task::Poll, thread::ThreadId};

/// Wraps around `T` and adds diagnostics to it.
pub(crate) struct WrappedFut<T> {
    inner: T,
    task_id: u64,
    /// Thread where we polled this future the latest.
    previous_thread: Option<ThreadId>,
}

impl<T> WrappedFut<T> {
    pub fn new(inner: T) -> Self {
        let new_task_id = {
            static NEXT_TASK_ID: AtomicU64 = AtomicU64::new(0);
            let id = NEXT_TASK_ID.fetch_add(1, Ordering::Relaxed);
            // If `id` is `u64::max_value` then we had an overflow.
            assert_ne!(id, u64::max_value());
            id
        };

        log::log!(LEVEL, "Task start: {:?}", new_task_id);

        WrappedFut {
            inner,
            task_id: new_task_id,
            previous_thread: None,
        }
    }
}

impl<T> Future for WrappedFut<T>
where
    T: Future<Output = ()> + Unpin,
{
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
        let my_task_id = self.task_id;

        let current_thread_id = std::thread::current().id();
        match &mut self.previous_thread {
            Some(id) if *id == current_thread_id => {},
            Some(id) => {
                log::log!(LEVEL, "Task {:?} changed thread poll from {:?} to {:?}",
                    my_task_id, *id, current_thread_id);
                *id = current_thread_id;
            },
            v @ None => *v = Some(current_thread_id),
        }

        let _guard = current_task::enter(current_task::CurrentTask::System);
        log::log!(LEVEL, "Entering poll for {:?}", my_task_id);
        let outcome = {
            let mut cx = ctxt_with_diag::WakerWithDiag::from(cx.waker());
            let mut cx = cx.context();

            let _guard2 = current_task::enter(current_task::CurrentTask::Task(my_task_id));
            Future::poll(Pin::new(&mut self.inner), &mut cx)
        };
        log::log!(LEVEL, "Leaving poll for {:?}", my_task_id);
        if let Poll::Ready(_) = outcome {
            log::log!(LEVEL, "Task end: {:?}", my_task_id);
        }
        outcome
    }
}

impl<T> Unpin for WrappedFut<T> {
}
