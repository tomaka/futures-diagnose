//! Output a log entry.

use crate::absolute_time;
use serde::Serialize;
use std::{fs::File, io::Write, sync::Mutex, time::Instant};
use std::sync::atomic::{AtomicU32, Ordering};

pub fn log_poll(task_name: &str, task_id: u64, start: Instant, end: Instant, first_time: bool, last_time: bool) {
    let tid = current_thread_id();
    let start_ts = absolute_time::elapsed_since_abs_time(start) / 1_000;
    let end_ts = absolute_time::elapsed_since_abs_time(end) / 1_000;

    let cname = None;/*match end_ts - start_ts {
        0 ..= 999 => Some("good"),
        1_000 ..= 19_999 => Some("bad"),
        _ => Some("terrible"),
    };*/    // TODO: colors end up being unreadable

    write_record(&Record {
        cat: "polling",
        name: task_name,
        ph: "B",
        pid: 0,
        tid,
        ts: start_ts,
        dur: None,
        bp: None,
        id: None,
        arg: None,
        cname,
    });

    // TODO: I don't understand the documentation; no idea if that code is correct
    if !(first_time && last_time) {
        write_record(&Record {
            cat: "polling",
            name: task_name,
            ph: if first_time {
                "s"
            } else if last_time {
                "f"
            } else {
                "t"
            },
            pid: 0,
            tid,
            ts: if first_time {
                end_ts
            } else {
                start_ts
            },
            dur: None,
            bp: Some("e"),
            id: Some(task_id),
            arg: None,
            cname: None,
        });
    }

    write_record(&Record {
        cat: "polling",
        name: task_name,
        ph: "E",
        pid: 0,
        tid,
        ts: end_ts,
        dur: None,
        bp: None,
        id: None,
        arg: None,
        cname,
    });
}

pub fn log_wake_up(task_name: &str, task_id: u64) {
    write_record(&Record {
        cat: "wakeup",
        name: task_name,
        ph: "i",
        pid: 0,
        tid: current_thread_id(),
        ts: absolute_time::now_since_abs_time() / 1_000,
        dur: None,
        id: None,
        bp: None,
        arg: None,
        cname: None,
    });
}

fn write_record(record: &Record) {
    let mut serialized = serde_json::to_vec(&record).unwrap();
    serialized.extend_from_slice(b",\n");
    FILE_OUT.lock().unwrap().write_all(&serialized).unwrap();
}

fn current_thread_id() -> u32 {
    lazy_static::lazy_static! {
        static ref NEXT_THREAD_ID: AtomicU32 = AtomicU32::new(0);
    }
    thread_local! {
        static THREAD_ID: u32 = NEXT_THREAD_ID.fetch_add(1, Ordering::Relaxed);
    }
    THREAD_ID.with(|id| *id)
}

lazy_static::lazy_static! {
    static ref FILE_OUT: Mutex<File> = {
        let mut file = File::create("profile.json").unwrap();
        file.write_all(b"[\n").unwrap();
        Mutex::new(file)
    };
}

#[derive(Serialize)]
struct Record<'a> {
    cat: &'a str,
    name: &'a str,
    ph: &'static str,
    pid: u32,
    tid: u32,
    ts: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    dur: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bp: Option<&'a str>,
    arg: Option<serde_json::Value>,
    /// Name of the color.
    /// Possible values here: https://github.com/catapult-project/catapult/blob/11513e359cd60e369bbbd1f4f2ef648c1bccabd0/tracing/tracing/base/color_scheme.html#L29
    #[serde(skip_serializing_if = "Option::is_none")]
    cname: Option<&'a str>,
}
