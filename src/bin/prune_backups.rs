use std::io;
use std::io::prelude::*;
use structopt::StructOpt;
use time::Duration;
use chrono::Utc;
use backups_cleaner::storage_client;
use backups_cleaner::storage_client::StorageClient;
use backups_cleaner::pruning_strategy;
use backups_cleaner::pruning_strategy::PruningStrategy;

#[derive(StructOpt, Debug)]
#[structopt(name = "Backups Cleaner")]
struct Opt {

    /// Asking for confirmation will be skipped, if this flag is provided.
    #[structopt(short = "y", long)]
    skip_confirmation: bool,

    /// Region the S3 bucket containing the backups is located in.
    #[structopt(short, long)]
    region: String,

    /// Name of the S3 bucket the backups are located in.
    #[structopt(short, long)]
    bucket: String,

    /// Prefix of the backups (directory).
    #[structopt(short, long, default_value = "")]
    prefix: String,

    /// Leave all backups within `keep_all_within` days unaltered.
    #[structopt(long)]
    keep_all_within: u16,

    /// Keep one backup per month within `one_per_month_within` days.
    #[structopt(long)]
    one_per_month_within: u16,

    #[structopt(long, default_value = "15")]
    one_per_month_tolerance: u16,
}

fn main() {
    let opt = Opt::from_args();

    let storage_client = storage_client::AwsS3::new(
        opt.region,
        opt.bucket,
        opt.prefix
    );
    let pruning_strategy = pruning_strategy::OlderThanButKeepOnePerMonth::new(
        Utc::now(),
        Duration::days(opt.keep_all_within as i64),
        Duration::days(opt.one_per_month_tolerance as i64),
        Duration::days(opt.one_per_month_within as i64),
    );

    let mut stored_backups = storage_client.stored_backups();
    println!("Found {} backups.", stored_backups.len());
    if stored_backups.len() == 1000 {
        println!(
            "Note, that the AWS S3 API only returns up to 1000 stored files, so you \
            might need to run this program several times to clean your bucket up completely."
        );
    }

    let expendable_backups = pruning_strategy.expendable_backups(&mut stored_backups);

    if expendable_backups.is_empty() {
        println!("No expendible backups found.");
        return;
    }

    println!(
        "This will delete {} of {} backups. Do you want to proceed? (y)",
        expendable_backups.len(),
        expendable_backups.len() + stored_backups.len()
    );

    let mut operation_confirmed = false;

    if opt.skip_confirmation {
        operation_confirmed = true;
    }
    else {
        let stdin = io::stdin();

        for line in stdin.lock().lines() {

            if line.unwrap() == "y" {
                operation_confirmed = true;
            }

            break;
        }
    }

    if !operation_confirmed { return; }

    println!("Removing expendible backups...");
    let number_of_deleted_objects = storage_client.delete_backups(expendable_backups);
    println!("Deleted {} backups.", number_of_deleted_objects);
}
