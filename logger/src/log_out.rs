//! Output a log entry.

use crate::absolute_time;

const LEVEL: log::Level = log::Level::Debug;
const TARGET: &str = "futures-profile";

pub fn log(data: futures_diagnose_exec_common::MessageData) {
    let now = absolute_time::now_since_abs_time();
    let message = futures_diagnose_exec_common::Message {
        timestamp_ns: now,
        data,
    };
    let serialized = futures_diagnose_exec_common::to_string(&message).unwrap();
    log::log!(target: TARGET, LEVEL, "{}", serialized);
}
