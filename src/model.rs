use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileState {
    pub status: String,
    pub fingerprint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub cwd: String,
    pub repo: Option<String>,
    pub branch: Option<String>,
    pub head: Option<String>,
    pub files: BTreeMap<String, FileState>,
    pub warning: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub session: String,
    pub command: String,
    pub started_ms: i64,
    pub finished_ms: Option<i64>,
    pub exit_code: Option<i32>,
    pub before: Snapshot,
    pub after: Option<Snapshot>,
}

pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}
