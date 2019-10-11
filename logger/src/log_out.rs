//! Output a log entry.

use crate::absolute_time;
use serde::Serialize;
use std::{fs::File, io::Write, sync::Mutex, time::Instant};
use std::sync::atomic::{AtomicU32, Ordering};

const LEVEL: log::Level = log::Level::Debug;
const TARGET: &str = "futures-profile";

pub fn log_poll(task_name: &str, task_id: u64, start: Instant, end: Instant, first_time: bool, last_time: bool) {
    let tid = current_thread_id();
    let start_ts = absolute_time::elapsed_since_abs_time(start) / 1_000;
    let end_ts = absolute_time::elapsed_since_abs_time(end) / 1_000;

    if !first_time {
        write_record(&Record {
            cat: "polling",
            name: task_name,
            ph: "f",
            pid: 0,
            tid,
            ts: start_ts,
            dur: None,
            bp: None,
            id: Some(task_id),
            arg: None,
        });
    }

    write_record(&Record {
        cat: "polling",
        name: task_name,
        ph: "X",
        pid: 0,
        tid,
        ts: start_ts,
        dur: Some(end_ts - start_ts),
        bp: None,
        id: None,
        arg: None,
    });

    if !last_time {
        write_record(&Record {
            cat: "polling",
            name: task_name,
            ph: "s",
            pid: 0,
            tid,
            ts: end_ts,
            dur: None,
            bp: None,
            id: Some(task_id),
            arg: None,
        });
    }
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
    });
}

fn write_record(record: &Record) {
    let mut serialized = futures_diagnose_exec_common::to_string(&record).unwrap();
    serialized.push(',');
    FILE_OUT.lock().unwrap().write_all(&serialized.as_bytes()).unwrap();
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
        Mutex::new(File::create("profile.json").unwrap())
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
}
