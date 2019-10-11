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
