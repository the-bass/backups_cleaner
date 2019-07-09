//! Contains implementations of clients for various hosts such as AWS S3. Each
//! client implements the `StorageClient` trait, so they can all be used for
//! pruning in a consistent manner.
mod aws_s3;

use super::BackupFileMeta;
pub use aws_s3::AwsS3;

/// Methods required to use a client for pruning.
pub trait StorageClient {

    /// Returns a list of all stored backups.
    fn stored_backups(&self) -> Vec<BackupFileMeta>;

    /// Deletes all given `backups`. Returns the number of successfully deleted
    /// objects.
    fn delete_backups(&self, backups: Vec<BackupFileMeta>) -> usize;
}
