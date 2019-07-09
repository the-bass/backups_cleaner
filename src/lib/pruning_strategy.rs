//! Contains strategies for deciding which of the stored backups are expendable.
mod older_than;
mod keep_one_per_month;
mod older_than_but_keep_history;

use super::BackupFileMeta;
pub use older_than::OlderThan;
pub use keep_one_per_month::KeepOnePerMonth;
pub use older_than_but_keep_history::OlderThanButKeepOnePerMonth;

/// Each pruning strategy should implement this trait, so it can be used to perform
/// the pruning.
pub trait PruningStrategy {

    /// Removes all expendable backups from the given `backups`
    fn expendable_backups(&self, backups: &mut Vec<BackupFileMeta>) -> Vec<BackupFileMeta>;
}

/// A collection of helper methods that come in handy when writing tests
/// for pruning strategies.
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};

    pub fn collect_ids(backup_file_metas: Vec<BackupFileMeta>) -> Vec<String> {
        backup_file_metas
            .iter()
            .map(|backup_file_meta| backup_file_meta.id.clone())
            .collect()
    }

    pub fn build_meta(id: &str, date: DateTime<Utc>) -> BackupFileMeta {
        BackupFileMeta {
            id: String::from(id),
            human_readable_id: String::from(id),
            date,
        }
    }

    pub fn as_vector(ids: &str) -> Vec<String> {
        ids.chars().map(|character| String::from(character.to_string())).collect()
    }
}
