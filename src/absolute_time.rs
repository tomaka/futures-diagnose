use std::time::{Duration, Instant};

/// Returns the number of nanoseconds that have elapsed between `time` and an arbitrary point in
/// time in the past.
///
/// Note that since the reference instant is lazily-generated, the following code will generate
/// the same value for `bef_ns` and `after_ns`:
///
/// ```ignore
/// let before = Instant::now();
/// thread::sleep(Duration::from_secs(5));
/// let after = Instant::now();
/// 
/// let bef_ns = elapsed_since_abs_time(before);
/// let after_ns = elapsed_since_abs_time(after);
/// ```
///
/// To remedy this, please call `elapsed_since_abs_time` as soon as possible.
pub(crate) fn elapsed_since_abs_time(time: Instant) -> u64 {
    let ref_instant = *REF_INSTANT;
    let dur = if time > ref_instant {
        time - ref_instant
    } else {
        Duration::new(0, 0)
    };
    dur.as_secs().saturating_mul(1_000_000_000).saturating_add(u64::from(dur.subsec_nanos()))
}

/// Shortcut for `elapsed_since_abs_time(Instant::now())`.
pub(crate) fn now_since_abs_time() -> u64 {
    elapsed_since_abs_time(Instant::now())
}

lazy_static::lazy_static! {
    static ref REF_INSTANT: Instant = Instant::now();
}
