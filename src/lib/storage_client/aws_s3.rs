use std::time::Duration;
use std::str::FromStr;
use rusoto_core::Region as AWSRegion;
use chrono::{DateTime, Utc};
use rusoto_s3::{S3, S3Client};
use super::{StorageClient, BackupFileMeta};

/// A client for AWS S3.
///
/// _NOTE_ Due to limitations of the AWS S3 API, cleaning with this client only works as expected
/// if your directory contains up to 1000 backups. Otherwise you might have to run the cleaner
/// several times in a row or clean manually so you're under 1000.
///
/// # Requirements
///
/// This implementation uses the access key stored in the environment variable `AWS_ACCESS_KEY_ID`
/// and the secret access key in `AWS_SECRET_ACCESS_KEY`. The respective AWS user needs to be
/// allowed to list all elements in the bucket, as well as delete objects from the backups
/// directory.
///
/// Assuming a bucket called `backups-cleaner-test-bucket` and backups in the directory `backups`,
/// the minimal policy would be
///
/// ```json
/// {
///     "Version": "2012-10-17",
///     "Statement": [
///         {
///             "Sid": "VisualEditor0",
///             "Effect": "Allow",
///             "Action": [
///                 "s3:ListBucket"
///             ],
///             "Resource": "arn:aws:s3:::backups-cleaner-test-bucket"
///         },
///         {
///             "Sid": "VisualEditor1",
///             "Effect": "Allow",
///             "Action": [
///                 "s3:DeleteObject"
///             ],
///             "Resource": "arn:aws:s3:::backups-cleaner-test-bucket/backups/*"
///         }
///     ]
/// }
/// ```
pub struct AwsS3 {
    s3_client: S3Client,
    bucket: String,
    prefix: String,
}

impl AwsS3 {

    pub fn new(region: String, bucket: String, prefix: String) -> AwsS3 {
        let region = AWSRegion::from_str(&region).unwrap();
        let s3_client = S3Client::new(region);

        AwsS3 {
            s3_client,
            bucket,
            prefix,
        }
    }

    fn object_to_backup_file_meta(&self, object: rusoto_s3::Object) -> BackupFileMeta {
        let last_modified_string = object.last_modified.unwrap();
        let last_modified = last_modified_string.parse::<DateTime<Utc>>().unwrap();
        let id = object.key.unwrap();

        BackupFileMeta {
            id: id.clone(),
            human_readable_id: id.clone(),
            date: last_modified,
        }
    }

    fn backup_file_meta_to_object_identifier(&self, backup_file_meta: BackupFileMeta) -> rusoto_s3::ObjectIdentifier {
        rusoto_s3::ObjectIdentifier {
            key: backup_file_meta.id, version_id: None
        }
    }
}

impl StorageClient for AwsS3 {

    fn stored_backups(&self) -> Vec<BackupFileMeta> {
        let list_request = rusoto_s3::ListObjectsV2Request {
            bucket: self.bucket.clone(),
            prefix: Some(self.prefix.clone()),
            delimiter: None,
            encoding_type: None,
            max_keys: None,
            request_payer: None,
            continuation_token: None,
            fetch_owner: None,
            start_after: None,
        };
        let objects = self.s3_client
            .list_objects_v2(list_request)
            .with_timeout(Duration::from_secs(3))
            .sync()
            .unwrap()
            .contents
            .unwrap();

        objects.into_iter().map(|object| self.object_to_backup_file_meta(object)).collect()
    }

    fn delete_backups(&self, backup_file_metas: Vec<BackupFileMeta>) -> usize {
        let objects_to_delete: Vec<rusoto_s3::ObjectIdentifier> = backup_file_metas
            .into_iter()
            .map(|backup_file_meta| self.backup_file_meta_to_object_identifier(backup_file_meta))
            .collect();

        let delete_request = rusoto_s3::DeleteObjectsRequest {
            bucket: self.bucket.clone(),
            bypass_governance_retention: None,
            mfa: None,
            request_payer: None,
            delete: rusoto_s3::Delete {
                objects: objects_to_delete,
                quiet: None,
            },
        };

        let delete_result = self.s3_client
            .delete_objects(delete_request)
            .with_timeout(Duration::from_secs(3))
            .sync()
            .unwrap();

        delete_result.deleted.unwrap().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let aws_s3_client = AwsS3::new(
            String::from("eu-west-2"),
            String::from("my-database-backups"),
            String::from("backups/")
        );

        assert_eq!(aws_s3_client.bucket, String::from("my-database-backups"));
        assert_eq!(aws_s3_client.prefix, String::from("backups/"));
    }

    #[test]
    #[should_panic]
    fn test_new_with_a_non_existing_region() {
        AwsS3::new(
            String::from("eu-west-9"),
            String::from("my-database-backups"),
            String::from("backups/")
        );
    }
}
