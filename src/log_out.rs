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

//! Output a log entry.

use crate::absolute_time;
use parking_lot::Mutex;
use serde::Serialize;
use std::sync::atomic::{AtomicU32, Ordering};
use std::{
    env,
    fs::{self, File},
    io::{self, Write as _},
    path::PathBuf,
    process,
    time::{Duration, Instant},
};

/// Interval at which logs rotate between files.
const LOGS_ROTATION: Duration = Duration::from_secs(30);

/// Returns true if logging is enabled.
pub fn is_enabled() -> bool {
    *LOGGING_ENABLED
}

/// Produce a single log entry about a call to `Future::poll`.
///
/// - `task_id` must be a unique identifier. Used to distinguish multiple tasks with the same
///   name.
/// - `start` and `end` are the `Instant`s when we respectively entered and left the `poll`
///   method.
/// - `first_time` must be true if this is the first time ever that `Future` is polled.
/// - `last_time` must be true if the `Future` has ended as a result of this `poll`.
///
pub fn log_poll(
    task_name: &str,
    task_id: u64,
    start: Instant,
    end: Instant,
    first_time: bool,
    last_time: bool,
) {
    if !is_enabled() {
        return;
    }

    let tid = current_thread_id();
    let start_ts = absolute_time::elapsed_since_abs_time(start) / 1_000;
    let end_ts = absolute_time::elapsed_since_abs_time(end) / 1_000;

    let cname = None; /*match end_ts - start_ts {
                          0 ..= 999 => Some("good"),
                          1_000 ..= 19_999 => Some("bad"),
                          _ => Some("terrible"),
                      };*/
    // TODO: colors end up being unreadable

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
            ts: if first_time { end_ts } else { start_ts },
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

/// Produce a single log entry about a call to `Waker::wake` or `Waker::wake_by_ref`.
///
/// `task_name` and `task_id` are the task that is being woken up.
pub fn log_wake_up(task_name: &str, _task_id: u64) {
    if !is_enabled() {
        return;
    }

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

/// Returns a unique number identifying the current thread. The number is used to differentiate
/// between threads and doesn't have any meaning per se.
fn current_thread_id() -> u32 {
    lazy_static::lazy_static! {
        static ref NEXT_THREAD_ID: AtomicU32 = AtomicU32::new(0);
    }
    thread_local! {
        static THREAD_ID: u32 = NEXT_THREAD_ID.fetch_add(1, Ordering::Relaxed);
    }
    THREAD_ID.with(|id| *id)
}

/// Write out a single record.
fn write_record(record: &Record) {
    let mut serialized = serde_json::to_vec(&record).unwrap();
    serialized.extend_from_slice(b",\n");

    let mut output = OUTPUT.lock();
    let output = match output.as_mut() {
        Some(o) => o,
        None => return,
    };

    if output.next_rotation <= Instant::now() {
        // Note: we don't write `]` at the end of the file because the latest entry contains
        // a trailing coma, which would lead to invalid JSON.
        output.file.sync_all().unwrap();

        let source_path = output.out_directory.join("profile.json");
        fs::rename(
            &source_path,
            output
                .out_directory
                .join(format!("profile.{}.{}.json", process::id(), output.next_filename_suffix)),
        )
        .unwrap();
        output.file = File::create(&source_path).unwrap();
        output.file.write_all(b"[\n").unwrap();

        output.next_filename_suffix += 1;
        output.next_rotation += LOGS_ROTATION;
    }

    output.file.write_all(&serialized).unwrap();
}

lazy_static::lazy_static! {
    static ref LOGGING_ENABLED: bool = env::var_os("PROFILE_DIR").is_some();

    static ref OUTPUT: Mutex<Option<OutputState>> = {
        let out_directory = if let Some(v) = env::var_os("PROFILE_DIR") {
            PathBuf::from(v)
        } else {
            return Mutex::new(None)
        };

        match fs::create_dir(&out_directory) {
            Ok(()) => {}
            Err(ref err) if err.kind() == io::ErrorKind::AlreadyExists => {},
            Err(err) => panic!("{:?}", err),
        };

        let mut file = File::create(out_directory.join("profile.json")).unwrap();
        file.write_all(b"[\n").unwrap();
        Mutex::new(Some(OutputState {
            file,
            out_directory,
            next_filename_suffix: 0,
            next_rotation: Instant::now() + LOGS_ROTATION,
        }))
    };
}

struct OutputState {
    file: File,
    out_directory: PathBuf,
    next_filename_suffix: u32,
    next_rotation: Instant,
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
