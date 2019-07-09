mod date_time_utilities;

use super::{PruningStrategy, BackupFileMeta};
use time::Duration;
use chrono::{DateTime, Utc};

/// Keeps one backup for each month. It will be the one that's closest to the
/// 1st day of the respective month. Will only consider backups that are less
/// than `tolerance` away from the 1st of the month.
pub struct KeepOnePerMonth {
    tolerance: Duration,
}

impl KeepOnePerMonth {

    pub fn new(tolerance: Duration) -> KeepOnePerMonth {

        KeepOnePerMonth {
            tolerance,
        }
    }
}

impl PruningStrategy for KeepOnePerMonth {

    fn expendable_backups(&self, backups: &mut Vec<BackupFileMeta>) -> Vec<BackupFileMeta> {
        if backups.is_empty() {
            return Vec::with_capacity(0);
        }

        backups.sort_by(|a, b| a.date.cmp(&b.date));

        let oldest_date = backups.first().unwrap().date;
        let youngest_date = backups.last().unwrap().date;
        let backup_for_month = |date: DateTime<Utc>, skip_indices_before: usize| -> Option<usize> {
            let mut index_of_nearest_backup = None;

            for i in skip_indices_before..backups.len() {
                if backups[i].date < date - self.tolerance {
                    continue;
                }
                if backups[i].date > date + self.tolerance {
                    return None;
                }

                index_of_nearest_backup = Some(i);
                break;
            }

            if index_of_nearest_backup.is_none() {
                return None;
            }
            let mut index_of_nearest_backup = index_of_nearest_backup.unwrap();

            for i in (index_of_nearest_backup + 1)..backups.len() {
                if backups[i].date > date + self.tolerance {
                    return Some(index_of_nearest_backup);
                }

                if date_time_utilities::is_closer(date, backups[i].date, backups[index_of_nearest_backup].date) {
                    index_of_nearest_backup = i;
                }
                else {
                    return Some(i - 1);
                }
            }

            Some(index_of_nearest_backup)
        };
        let last_date = date_time_utilities::beginning_of_next_month(youngest_date);
        let mut date = date_time_utilities::beginning_of_month(oldest_date);
        let mut start_index = 0;
        let mut backups_to_keep_indices: Vec<usize> = vec![];

        while date <= last_date {
            let backup_index = backup_for_month(date, start_index);

            if backup_index.is_some() {
                backups_to_keep_indices.push(backup_index.unwrap());
                start_index = backup_index.unwrap() + 1;
            }

            date = date_time_utilities::beginning_of_next_month(date);
        }

        let mut expendable_backups = vec![];
        for i in (0..backups.len()).rev() {
            if !backups_to_keep_indices.contains(&i) {
                expendable_backups.insert(0, backups.remove(i));
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
        let strategy = KeepOnePerMonth::new(Duration::days(20));
        let mut backups = vec![
            build_meta("A", Utc.ymd(2014, 7, 19).and_hms(22, 0, 0)),
            build_meta("B", Utc.ymd(2014, 8, 1).and_hms(0, 0, 0)),
            build_meta("1", Utc.ymd(2014, 8, 2).and_hms(0, 0, 0)),
            build_meta("C", Utc.ymd(2014, 8, 30).and_hms(0, 0, 0)),
            build_meta("D", Utc.ymd(2014, 11, 1).and_hms(0, 0, 0)),
            build_meta("E", Utc.ymd(2014, 12, 1).and_hms(0, 0, 0)),
            build_meta("2", Utc.ymd(2014, 12, 2).and_hms(0, 0, 0)),
            build_meta("F", Utc.ymd(2014, 12, 30).and_hms(0, 0, 0)),
        ];

        let expendable_backups = strategy.expendable_backups(&mut backups);

        assert_eq!(collect_ids(expendable_backups), as_vector("12"));
        assert_eq!(collect_ids(backups), as_vector("ABCDEF"));
    }

    #[test]
    fn test_expendable_backups_when_no_backups_are_given() {
        let strategy = KeepOnePerMonth::new(Duration::days(20));
        let mut backups = vec![];

        let expendable_backups = strategy.expendable_backups(&mut backups);

        assert!(expendable_backups.is_empty());
        assert!(backups.is_empty());
    }

    #[test]
    fn test_expendable_backups_when_one_backup_is_given() {
        let strategy = KeepOnePerMonth::new(Duration::days(20));
        let mut backups = vec![
            build_meta("A", Utc.ymd(2014, 7, 19).and_hms(21, 46, 12)),
        ];

        let expendable_backups = strategy.expendable_backups(&mut backups);

        assert!(expendable_backups.is_empty());
        assert_eq!(collect_ids(backups), as_vector("A"));
    }

    #[test]
    fn test_expendable_backups_when_a_backup_is_given_that_does_not_belong_to_a_month() {
        let strategy = KeepOnePerMonth::new(Duration::days(1));
        let mut backups = vec![
            build_meta("A", Utc.ymd(2014, 7, 15).and_hms(0, 0, 0)),
        ];

        let expendable_backups = strategy.expendable_backups(&mut backups);

        assert_eq!(collect_ids(expendable_backups), as_vector("A"));
        assert!(backups.is_empty());
    }

    #[test]
    fn test_expendable_backups_when_the_closest_backup_was_already_used_for_another_month() {
        let strategy = KeepOnePerMonth::new(Duration::days(32));
        let mut backups = vec![
            build_meta("A", Utc.ymd(2014, 5, 31).and_hms(0, 0, 0)), // Would be closer to the first of June, but will already be used for May.
            build_meta("B", Utc.ymd(2014, 6, 3).and_hms(0, 0, 0)), // Should be used for June.
        ];

        let expendable_backups = strategy.expendable_backups(&mut backups);

        assert!(expendable_backups.is_empty());
        assert_eq!(collect_ids(backups), as_vector("AB"));
    }

    #[test]
    fn test_expendable_backups_when_the_closest_backup_is_before_the_beginning_of_the_month() {
        let strategy = KeepOnePerMonth::new(Duration::days(10));
        let mut backups = vec![
            build_meta("C", Utc.ymd(2014, 6, 4).and_hms(0, 0, 0)),
            build_meta("B", Utc.ymd(2014, 6, 2).and_hms(0, 0, 0)), // Is the first date in June.
            build_meta("A", Utc.ymd(2014, 5, 31).and_hms(0, 0, 0)), // Is the closest date to the first of June.
        ];

        let expendable_backups = strategy.expendable_backups(&mut backups);

        assert_eq!(collect_ids(expendable_backups), as_vector("BC"));
        assert_eq!(collect_ids(backups), as_vector("A"));
    }
}
