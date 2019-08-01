use std::{task::Context, task::Waker};

// TODO: not actually implemented; the code is a dummy
// TODO: this could be implemented in a more unsafe but performant way

/// Wraps around a `&Waker` but logs a message every time the task is waken up.
pub struct WakerWithDiag<'a> {
    inner: &'a Waker,
}

impl<'a> From<&'a Waker> for WakerWithDiag<'a> {
    fn from(inner: &'a Waker) -> WakerWithDiag<'a> {
        WakerWithDiag {
            inner
        }
    }
}

impl<'a> WakerWithDiag<'a> {
    pub fn context(&mut self) -> Context {
        Context::from_waker(self.inner)
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
