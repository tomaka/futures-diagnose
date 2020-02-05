// Copyright 2020 Pierre Krieger
//
// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
// OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use crate::{ctxt_with_diag, log_out};
use pin_project::pin_project;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use std::{borrow::Cow, fmt, future::Future, mem, pin::Pin, task::Context, task::Poll};

/// Wraps around a `Future` and adds diagnostics.
pub fn diagnose<T>(name: impl Into<Cow<'static, str>>, inner: T) -> DiagnoseFuture<T> {
    if log_out::is_enabled() {
        // TODO: hack, see doc of elapsed_since_abs_time
        crate::absolute_time::elapsed_since_abs_time(Instant::now());
    }

    DiagnoseFuture {
        inner,
        task_name: name.into(),
        task_id: {
            static NEXT_ID: AtomicU64 = AtomicU64::new(0);
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        },
        first_time_poll: true,
    }
}

/// Wraps around `Future` and adds diagnostics to it.
#[pin_project]
#[derive(Clone)]
pub struct DiagnoseFuture<T> {
    /// The inner future doing the actual work.
    #[pin]
    inner: T,
    task_name: Cow<'static, str>,
    task_id: u64,
    first_time_poll: bool,
}

impl<T> Future for DiagnoseFuture<T>
where
    T: Future,
{
    type Output = T::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let this = self.project();

        if !log_out::is_enabled() {
            return Future::poll(this.inner, cx);
        }

        let before = Instant::now();

        let waker = ctxt_with_diag::waker_with_diag(
            cx.waker().clone(),
            this.task_name.clone(),
            *this.task_id,
        );
        let mut cx = Context::from_waker(&waker);
        let outcome = Future::poll(this.inner, &mut cx);

        let after = Instant::now();

        log_out::log_poll(
            &this.task_name,
            *this.task_id,
            before,
            after,
            mem::replace(this.first_time_poll, false),
            outcome.is_ready(),
        );

        outcome
    }
}

impl<T> futures01::Future for DiagnoseFuture<T>
where
    T: futures01::Future,
{
    type Item = T::Item;
    type Error = T::Error;

    fn poll(&mut self) -> futures01::Poll<Self::Item, Self::Error> {
        if !log_out::is_enabled() {
            return self.inner.poll();
        }

        let before = Instant::now();
        let outcome = self.inner.poll();
        let after = Instant::now();
        let last_time = match outcome {
            Ok(futures01::Async::Ready(_)) => true,
            Ok(futures01::Async::NotReady) => false,
            Err(_) => true,
        };
        log_out::log_poll(
            &self.task_name,
            self.task_id,
            before,
            after,
            mem::replace(&mut self.first_time_poll, false),
            last_time,
        );
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
