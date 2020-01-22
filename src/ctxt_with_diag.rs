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

use crate::log_out;
use std::{borrow::Cow, sync::Arc, task::Waker};

/// Takes ownership of a `Waker`, and returns another `Waker` that wraps around it but with
/// logging.
pub(crate) fn waker_with_diag(waker: Waker, task_name: Cow<'static, str>, task_id: u64) -> Waker {
    futures::task::waker(Arc::new(WakerWithDiag {
        inner: waker,
        task_name,
        task_id,
    }))
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
