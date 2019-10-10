use crate::{LEVEL, absolute_time, ctxt_with_diag, current_task};
use pin_project::pin_project;
use std::sync::atomic::{AtomicU64, Ordering};
use std::{borrow::Cow, future::Future, pin::Pin, task::Context, task::Poll, thread::ThreadId};
use std::time::Instant;

/// Wraps around `T` and adds diagnostics to it.
#[pin_project]
pub struct DiagnoseFuture<T> {
    #[pin]
    inner: T,
    task_id: Cow<'static, str>,
    /// Thread where we polled this future the latest.
    previous_thread: Option<ThreadId>,
}

impl<T> DiagnoseFuture<T> {
    pub fn new(inner: T, name: impl Into<Cow<'static, str>>) -> Self {
        let name = name.into();
        log::log!(LEVEL, "Task start: {:?}", name);

        DiagnoseFuture {
            inner,
            task_id: name,
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
        let my_task_id = this.task_id;      // TODO: why this variable?

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
        let ref_instant = absolute_time::absolute_instant();
        let (outcome, before, after) = {
            let waker = ctxt_with_diag::WakerWithDiag::new(cx.waker(), my_task_id.clone());
            let waker = waker.into_waker();
            let mut cx = Context::from_waker(&waker);

            let before = Instant::now();
            log::log!(LEVEL, "At {:?}, entering poll for {:?}", before - ref_instant, my_task_id);
            let _guard2 = current_task::enter(current_task::CurrentTask::Task(my_task_id.clone()));
            let outcome = Future::poll(this.inner, &mut cx);
            let after = Instant::now();
            (outcome, before, after)
        };
        log::log!(LEVEL, "At {:?}, leaving poll for {:?}; took {:?}", after - ref_instant, my_task_id, after - before);
        if let Poll::Ready(_) = outcome {
            log::log!(LEVEL, "Task end: {:?}", my_task_id);
        }
        outcome
    }
}
