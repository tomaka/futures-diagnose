use crate::{ctxt_with_diag, current_task, log_out};
use pin_project::pin_project;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use std::{borrow::Cow, future::Future, pin::Pin, task::Context, task::Poll, thread::ThreadId};

/// Wraps around `Future` and adds diagnostics to it.
#[pin_project]
#[derive(Clone)]
pub struct DiagnoseFuture<T> {
    /// The inner future doing the actual work.
    #[pin]
    inner: T,
    /// Task we are manipulating.
    task: futures_diagnose_exec_common::Task,
    /// Thread where we polled this future the latest.
    previous_thread: Option<ThreadId>,
}

impl<T> DiagnoseFuture<T> {
    pub fn new(inner: T, name: impl Into<Cow<'static, str>>) -> Self {
        let name = name.into();
        let task = futures_diagnose_exec_common::Task {
            name: name.to_string(), // TODO: optimize
            id: {
                static NEXT_ID: AtomicU64 = AtomicU64::new(0);
                NEXT_ID.fetch_add(1, Ordering::Relaxed)
            },
        };
        log_out::log(futures_diagnose_exec_common::MessageData::TaskStart(
            task.clone(),
        ));

        DiagnoseFuture {
            inner,
            task,
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

        // TODO: restore
        /*let current_thread_id = std::thread::current().id();
        match this.previous_thread {
            Some(id) if *id == current_thread_id => {},
            Some(id) => {
                log::log!(target: TARGET, LEVEL, "Task {:?} changed thread poll from {:?} to {:?}",
                    this.task_id, *id, current_thread_id);
                *id = current_thread_id;
            },
            v @ None => *v = Some(current_thread_id),
        }*/

        let _guard = current_task::enter(current_task::CurrentTask::System);
        let before = Instant::now();
        let outcome = {
            let waker = ctxt_with_diag::waker_with_diag(cx.waker().clone(), this.task.clone());
            let mut cx = Context::from_waker(&waker);
            log_out::log(futures_diagnose_exec_common::MessageData::PollStart(
                this.task.clone(),
            ));
            //let _guard2 = current_task::enter(current_task::CurrentTask::Task(this.task_id.clone()));
            Future::poll(this.inner, &mut cx)
        };
        let after = Instant::now();
        log_out::log(futures_diagnose_exec_common::MessageData::PollEnd {
            task: this.task.clone(),
            poll_duration: after - before,
        });
        if let Poll::Ready(_) = outcome {
            log_out::log(futures_diagnose_exec_common::MessageData::TaskEnd(
                this.task.clone(),
            ));
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
