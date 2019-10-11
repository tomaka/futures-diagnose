//! Contains definitions for the format of the messages.

use serde::{Deserialize, Serialize};
use std::time::Duration;

pub use serde_json::{from_slice, to_string, to_writer};

// TODO: add some custom attributes so that the generated JSON looks cleaner

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Message {
    #[serde(rename = "timestamp")]
    pub timestamp_ns: u64,
    #[serde(flatten)]
    pub data: MessageData,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum MessageData {
    #[serde(rename = "task_start")]
    TaskStart(Task),
    #[serde(rename = "task_end")]
    TaskEnd(Task),
    #[serde(rename = "task_wake_up")]
    TaskWakeUp {
        woken_up: Task,
        waker: Task,
        // TODO:
        /*waking_thread_id: String,
        waking_thread_name: String,*/
    },
    #[serde(rename = "polling_thread_change")]
    PollingThreadChange {
        task: Task,
        // TODO:
    },
    #[serde(rename = "poll_start")]
    PollStart(Task),
    #[serde(rename = "poll_end")]
    PollEnd {
        task: Task,
        #[serde(serialize_with = "ser_dur", deserialize_with = "deser_dur")]
        #[serde(rename = "poll_duration_ns")]
        poll_duration: Duration,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Task {
    pub name: String,
    pub id: u64,
}

fn ser_dur<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer
{
    let ns = duration.as_secs().saturating_mul(1_000_000_000)
        .saturating_add(u64::from(duration.subsec_nanos()));
    serializer.serialize_u64(ns)
}

fn deser_dur<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: serde::Deserializer<'de>
{
    let nanos = <u64 as Deserialize>::deserialize(deserializer)?;
    Ok(Duration::from_nanos(nanos))
}
