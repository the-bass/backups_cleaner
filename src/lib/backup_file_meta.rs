use chrono::{DateTime, Utc};

/// Internally used abstraction of a single backup file.
#[derive(Debug)]
pub struct BackupFileMeta {
    pub id: String,
    pub human_readable_id: String,
    pub date: DateTime<Utc>,
}
