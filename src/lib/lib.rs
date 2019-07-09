//! Provides methods to free your backups directories from expendable backups.
//!
//! # Example
//!
//! ```rust,no_run
//! use time::Duration;
//! use chrono::Utc;
//! use backups_cleaner::storage_client::StorageClient;
//! use backups_cleaner::pruning_strategy::PruningStrategy;
//!
//! // Pick your storage client.
//! let storage_client = backups_cleaner::storage_client::AwsS3::new(
//!     String::from("eu-1"),
//!     String::from("hp-database-backups"),
//!     String::from("psql_backups"),
//! );
//!
//! // Pick a pruning strategy.
//! let pruning_strategy = backups_cleaner::pruning_strategy::OlderThan::new(
//!     Duration::days(2),
//!     Utc::now(),
//! );
//!
//! // Perform pruning. The following code works for whatever client and strategy you
//! // have chosen.
//! let mut stored_backups = storage_client.stored_backups();
//! let expendable_backups = pruning_strategy.expendable_backups(&mut stored_backups);
//! storage_client.delete_backups(expendable_backups);
//! ```
mod backup_file_meta;
pub mod storage_client;
pub mod pruning_strategy;

pub use backup_file_meta::BackupFileMeta;
