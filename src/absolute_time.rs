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

use std::time::{Duration, Instant};

/// Returns the number of nanoseconds that have elapsed between `time` and the first time this
/// function has been called in the program ever.
///
/// Note that, consequently, keep in mind that the value of `diff` in the following code will
/// be 0:
///
/// ```ignore
/// let before = Instant::now();
/// thread::sleep(Duration::from_secs(5));
/// let after = Instant::now();
///
/// let bef_ns = elapsed_since_abs_time(before);
/// let after_ns = elapsed_since_abs_time(after);
/// let diff = after_ns - bef_ns;
/// assert_eq!(diff, 0);
/// ```
///
/// To remedy this, please call `elapsed_since_abs_time` as soon as possible.
pub(crate) fn elapsed_since_abs_time(time: Instant) -> u64 {
    lazy_static::lazy_static! {
        static ref REF_INSTANT: Instant = Instant::now();
    }
    let ref_instant = *REF_INSTANT;
    let dur = if time > ref_instant {
        time - ref_instant
    } else {
        Duration::new(0, 0)
    };
    dur.as_secs()
        .saturating_mul(1_000_000_000)
        .saturating_add(u64::from(dur.subsec_nanos()))
}

/// Shortcut for `elapsed_since_abs_time(Instant::now())`.
pub(crate) fn now_since_abs_time() -> u64 {
    elapsed_since_abs_time(Instant::now())
}
