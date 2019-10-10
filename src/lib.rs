use futures::future::FutureObj;
use futures::task::{Spawn, SpawnError};
use std::future::Future;

pub use fut_with_diag::DiagnoseFuture;

mod ctxt_with_diag;
mod current_task;
mod fut_with_diag;

const LEVEL: log::Level = log::Level::Debug;

pub trait FutureExt: Future {
    fn with_diagnostics(self) -> DiagnoseFuture<Self>
    where
        Self: Sized
    {
        DiagnoseFuture::new(self)
    }
}

impl<T> FutureExt for T where T: Future {}

/// Wraps around a `T` and provides lots of diagnostics about tasks spawned through it.
pub struct DiagSpawn<T> {
    inner: T,
}

impl<T> DiagSpawn<T> {
    /// Wraps around `inner`.
    pub fn new(inner: T) -> Self {
        DiagSpawn {
            inner
        }
    }
}

impl<T> Spawn for DiagSpawn<T>
where
    T: Spawn,
{
    fn spawn_obj(&mut self, future: FutureObj<'static, ()>) -> Result<(), SpawnError> {
        let wrapped = Box::pin(fut_with_diag::DiagnoseFuture::new(future));
        self.inner.spawn_obj(FutureObj::from(wrapped))
    }

    fn status(&self) -> Result<(), SpawnError> {
        self.inner.status()
    }
}

#[cfg(test)]
mod tests {
    use crate::DiagSpawn;
    use futures::executor::ThreadPool;

    #[test]
    fn basic() {
        let diag_spawn = DiagSpawn::new(ThreadPool::new().unwrap());
    }
}
