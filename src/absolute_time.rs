
//! Provides the [`absolute_instant`] function, returning the number of nanoseconds since some
//! arbitrary point in time.
//!
//! Be aware that, since the value is lazily-generated, the following code might panic:
//!
//! ```ignore
//! Instant::now().elapsed_since(absolute_instant())
//! ```
//!
//! This is a pretty big deal, but since this function is internal to this crate, and we can
//! easily find out where it's called, it should be ok.

use std::time::Instant;

pub(crate) fn absolute_instant() -> Instant {
    *REF_INSTANT
}

lazy_static::lazy_static! {
    static ref REF_INSTANT: Instant = Instant::now();
}
