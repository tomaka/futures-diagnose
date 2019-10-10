use crate::{LEVEL, TARGET, absolute_time, ctxt_with_diag, current_task};
use pin_project::pin_project;
use std::fmt;
use std::{borrow::Cow, future::Future, pin::Pin, task::Context, task::Poll, thread::ThreadId};
use std::time::Instant;

/// Wraps around `Future` and adds diagnostics to it.
#[pin_project]
#[derive(Clone)]
pub struct DiagnoseFuture<T> {
    /// The inner future doing the actual work.
    #[pin]
    inner: T,
    /// Name of the task.
    task_id: Cow<'static, str>,
    /// Thread where we polled this future the latest.
    previous_thread: Option<ThreadId>,
}

impl<T> DiagnoseFuture<T> {
    pub fn new(inner: T, name: impl Into<Cow<'static, str>>) -> Self {
        let name = name.into();
        let point_in_time = absolute_time::now_since_abs_time();
        log::log!(target: TARGET, LEVEL, "At {:?}, task start: {:?}", point_in_time, name);

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

        let current_thread_id = std::thread::current().id();
        match this.previous_thread {
            Some(id) if *id == current_thread_id => {},
            Some(id) => {
                log::log!(target: TARGET, LEVEL, "Task {:?} changed thread poll from {:?} to {:?}",
                    this.task_id, *id, current_thread_id);
                *id = current_thread_id;
            },
            v @ None => *v = Some(current_thread_id),
        }

        let _guard = current_task::enter(current_task::CurrentTask::System);
        let (outcome, before, after) = {
            let waker = ctxt_with_diag::waker_with_diag(cx.waker().clone(), this.task_id.clone());
            let mut cx = Context::from_waker(&waker);

            let before = Instant::now();
            log::log!(target: TARGET, LEVEL, "At {:?}, entering poll for {:?}", absolute_time::elapsed_since_abs_time(before), this.task_id);
            let _guard2 = current_task::enter(current_task::CurrentTask::Task(this.task_id.clone()));
            let outcome = Future::poll(this.inner, &mut cx);
            let after = Instant::now();
            (outcome, before, after)
        };
        let after_abs = absolute_time::elapsed_since_abs_time(after);
        log::log!(target: TARGET, LEVEL, "At {:?}, leaving poll for {:?}; took {:?}", after_abs, this.task_id, after - before);
        if let Poll::Ready(_) = outcome {
            log::log!(target: TARGET, LEVEL, "At {:?}, task end: {:?}", after_abs, this.task_id);
        }
        outcome
    }
}

impl<T> fmt::Debug for DiagnoseFuture<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
    }
}
