//! Wraps around futures and profiles them.
//!
//! # Usage
//!
//! ```
//! futures::executor::block_on(futures_diagnose_exec::diagnose("task-name", async move {
//!     // ...
//! }))
//! ```
//!
//! Wrap all your futures into `futures_diagnose_exec::diagnose`. Then launch your program with
//! the `PROFILE_DIR` environment variable set to a path name. CPU profiling will automatically
//! be performed and JSON files written in the target directory.
//!
//! You can open the JSON files using the Chrome browser by opening the address
//! `chrome://tracing`.
//!

use futures::future::FutureObj;
use futures::task::{Spawn, SpawnError};
use std::{borrow::Cow, future::Future};

pub use fut_with_diag::{diagnose, DiagnoseFuture};

mod absolute_time;
mod ctxt_with_diag;
mod fut_with_diag;
mod log_out;

pub mod prelude {
    pub use crate::FutureExt as _;
    pub use crate::Future01Ext as _;
}

/// Extension trait on `Future`s.
pub trait FutureExt: Future {
    fn with_diagnostics(self, name: impl Into<Cow<'static, str>>) -> DiagnoseFuture<Self>
    where
        Self: Sized,
    {
        fut_with_diag::diagnose(name, self)
    }
}

impl<T> FutureExt for T where T: Future {}

/// Extension trait on `Future`s.
pub trait Future01Ext: futures01::Future {
    fn with_diagnostics(self, name: impl Into<Cow<'static, str>>) -> DiagnoseFuture<Self>
    where
        Self: Sized,
    {
        fut_with_diag::diagnose(name, self)
    }
}

impl<T> Future01Ext for T where T: futures01::Future {}

/// Wraps around a `T` and provides lots of diagnostics about tasks spawned through it.
pub struct DiagSpawn<T> {
    inner: T,
}

impl<T> DiagSpawn<T> {
    /// Wraps around `inner`.
    pub fn new(inner: T) -> Self {
        DiagSpawn { inner }
    }
}

impl<T> Spawn for DiagSpawn<T>
where
    T: Spawn,
{
    fn spawn_obj(&self, future: FutureObj<'static, ()>) -> Result<(), SpawnError> {
        /*let wrapped = Box::pin(fut_with_diag::DiagnoseFuture::new(future));
        self.inner.spawn_obj(FutureObj::from(wrapped))*/
        unimplemented!()
    }

    fn status(&self) -> Result<(), SpawnError> {
        self.inner.status()
    }
}
