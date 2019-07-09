use super::{PruningStrategy, BackupFileMeta};
use time::Duration;
use chrono::{DateTime, Utc};

/// Considers all backups expendable, that are older than `duration` from `reference_time`.
pub struct OlderThan {
    duration: Duration,
    reference_time: DateTime<Utc>,
}

impl OlderThan {

    pub fn new(duration: Duration, reference_time: DateTime<Utc>) -> OlderThan {
        OlderThan {
            duration,
            reference_time,
        }
    }

    fn too_old(&self, backup: &BackupFileMeta) -> bool {
        self.reference_time.signed_duration_since(backup.date) > self.duration
    }
}

impl PruningStrategy for OlderThan {

    fn expendable_backups(&self, backups: &mut Vec<BackupFileMeta>) -> Vec<BackupFileMeta> {
        let mut expendable_backups: Vec<BackupFileMeta> = vec![];
        let mut index = backups.len();

        for _ in 0..backups.len() {
            index -= 1;

            if self.too_old(&backups[index]) {
                let backup = backups.remove(index);
                expendable_backups.insert(0, backup);
            }
        }

        expendable_backups
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::tests::{build_meta, collect_ids, as_vector};
    use chrono::Utc;
    use chrono::offset::TimeZone;

    #[test]
    fn test_expendable_backups() {
        let strategy = OlderThan {
            reference_time: Utc.ymd(2014, 11, 14).and_hms(8, 9, 10),
            duration: Duration::minutes(1),
        };

        let mut backups = vec![
            // A day _after_ `reference_time` shouldn't be considered expendable.
            build_meta("0", Utc.ymd(2014, 11, 15).and_hms(8, 9, 10)),

            // `duration` + 1 second old backups should be considered expendable.
            build_meta("C", Utc.ymd(2014, 11, 14).and_hms(8, 8, 9)),

            // Same date as `reference_time` shouldn't be considered expendable.
            build_meta("A", Utc.ymd(2014, 11, 14).and_hms(8, 9, 10)),

            // Very old backups should be considered expendable.
            build_meta("D", Utc.ymd(2013, 11, 14).and_hms(8, 9, 10)),

            // Exactly `duration` before `reference_time` shouldn't be considered expendable.
            build_meta("B", Utc.ymd(2014, 11, 14).and_hms(8, 8, 10)),
        ];

        let expendable_backups = strategy.expendable_backups(&mut backups);

        assert_eq!(collect_ids(expendable_backups), as_vector("CD"));
        assert_eq!(collect_ids(backups), as_vector("0AB"));
    }

    #[test]
    fn test_expendable_backups_with_no_backups_given() {
        let strategy = OlderThan {
            reference_time: Utc.ymd(2014, 11, 14).and_hms(8, 9, 10),
            duration: Duration::minutes(1),
        };
        let mut backups = vec![];

        let expendable_backups = strategy.expendable_backups(&mut backups);

        assert!(expendable_backups.is_empty());
        assert!(backups.is_empty());
    }
}
