use crate::{LEVEL, ctxt_with_diag, current_task};
use pin_project::pin_project;
use std::sync::atomic::{AtomicU64, Ordering};
use std::{future::Future, pin::Pin, task::Context, task::Poll, thread::ThreadId};
use std::time::Instant;

/// Wraps around `T` and adds diagnostics to it.
#[pin_project]
pub struct DiagnoseFuture<T> {
    #[pin]
    inner: T,
    task_id: u64,
    /// Thread where we polled this future the latest.
    previous_thread: Option<ThreadId>,
}

impl<T> DiagnoseFuture<T> {
    pub fn new(inner: T) -> Self {
        let new_task_id = {
            static NEXT_TASK_ID: AtomicU64 = AtomicU64::new(0);
            let id = NEXT_TASK_ID.fetch_add(1, Ordering::Relaxed);
            // If `id` is `u64::max_value` then we had an overflow.
            assert_ne!(id, u64::max_value());
            id
        };

        log::log!(LEVEL, "Task start: {:?}", new_task_id);

        DiagnoseFuture {
            inner,
            task_id: new_task_id,
            previous_thread: None,
        }
    }
}

impl<T> Future for DiagnoseFuture<T>
where
    T: Future,
{
    type Output = T::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let this = self.project();
        let my_task_id = *this.task_id;

        let current_thread_id = std::thread::current().id();
        match this.previous_thread {
            Some(id) if *id == current_thread_id => {},
            Some(id) => {
                log::log!(LEVEL, "Task {:?} changed thread poll from {:?} to {:?}",
                    my_task_id, *id, current_thread_id);
                *id = current_thread_id;
            },
            v @ None => *v = Some(current_thread_id),
        }

        let _guard = current_task::enter(current_task::CurrentTask::System);
        let (outcome, before, after) = {
            let waker = ctxt_with_diag::WakerWithDiag::new(cx.waker(), my_task_id);
            let waker = waker.into_waker();
            let mut cx = Context::from_waker(&waker);

            let ref_instant = *REF_INSTANT;
            let before = Instant::now();
            log::log!(LEVEL, "At {:?}, entering poll for {:?}", before - ref_instant, my_task_id);
            let _guard2 = current_task::enter(current_task::CurrentTask::Task(my_task_id));
            let outcome = Future::poll(this.inner, &mut cx);
            let after = Instant::now();
            (outcome, before, after)
        };
        log::log!(LEVEL, "At {:?}, leaving poll for {:?}; took {:?}", after - *REF_INSTANT, my_task_id, after - before);
        if let Poll::Ready(_) = outcome {
            log::log!(LEVEL, "Task end: {:?}", my_task_id);
        }
        outcome
    }
}

lazy_static::lazy_static! {
    static ref REF_INSTANT: Instant = Instant::now();
}
