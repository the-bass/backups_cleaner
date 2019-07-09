use super::{PruningStrategy, BackupFileMeta, KeepOnePerMonth, OlderThan};
use time::Duration;
use chrono::{DateTime, Utc};

/// Considers backups expendable, that are older than `keep_all_within` from `reference_time`,
/// but still keeps one per month from those within `one_per_month_within` from `reference_time`.
pub struct OlderThanButKeepOnePerMonth {

    /// The current time.
    reference_time: DateTime<Utc>,

    /// Don't touch any backups within `keep_all_within` from `reference_time`.
    keep_all_within: Duration,

    /// Accept backups belonging to a month, that are within `one_per_month_tolerance`
    /// from the 1st of the respective month.
    one_per_month_tolerance: Duration,

    /// Consider all backups older than `one_per_month_within` from `reference_time` expendable.
    one_per_month_within: Duration,
}

impl OlderThanButKeepOnePerMonth {

    pub fn new(
        reference_time: DateTime<Utc>,
        keep_all_within: Duration,
        one_per_month_tolerance: Duration,
        one_per_month_within: Duration,
    ) -> OlderThanButKeepOnePerMonth {

        // Panic, if the options are contradictory.
        assert!(keep_all_within <= one_per_month_within);

        OlderThanButKeepOnePerMonth {
            reference_time,
            keep_all_within,
            one_per_month_tolerance,
            one_per_month_within,
        }
    }
}

impl PruningStrategy for OlderThanButKeepOnePerMonth {

    fn expendable_backups(&self, backups: &mut Vec<BackupFileMeta>) -> Vec<BackupFileMeta> {
        let mut expendable_backups = vec![];

        let mut very_old_backups = OlderThan::new(self.one_per_month_within, self.reference_time).expendable_backups(backups);
        expendable_backups.append(&mut very_old_backups);

        let mut older_backups = OlderThan::new(self.keep_all_within, self.reference_time).expendable_backups(backups);

        let mut expendable_older_backups = KeepOnePerMonth::new(self.one_per_month_tolerance).expendable_backups(&mut older_backups);

        expendable_backups.append(&mut expendable_older_backups);
        backups.append(&mut older_backups);

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
        let strategy = OlderThanButKeepOnePerMonth::new(
            Utc.ymd(2014, 6, 15).and_hms(0, 0, 0),
            Duration::days(1),
            Duration::days(15),
            Duration::days(90),
        );

        let mut backups = vec![
            // The below should not be considered expendable, as it's _after_
            // `reference_time`.
            build_meta("A", Utc.ymd(2015, 6, 15).and_hms(0, 0, 0)),

            // All of the below should not be considered expendable, as they are within
            // `keep_all_within`.
            build_meta("B", Utc.ymd(2014, 6, 15).and_hms(0, 0, 0)),
            build_meta("C", Utc.ymd(2014, 6, 14).and_hms(16, 0, 0)),
            build_meta("D", Utc.ymd(2014, 6, 14).and_hms(8, 0, 0)),
            build_meta("E", Utc.ymd(2014, 6, 14).and_hms(0, 0, 0)),

            // Only one per month of the below should be kept.
            build_meta("F", Utc.ymd(2014, 6, 13).and_hms(23, 59, 59)),
            build_meta("G", Utc.ymd(2014, 6, 3).and_hms(0, 0, 0)),
            build_meta("H", Utc.ymd(2014, 6, 1).and_hms(0, 0, 0)), // Should be kept for June.
            build_meta("I", Utc.ymd(2014, 5, 31).and_hms(0, 0, 0)),

            build_meta("J", Utc.ymd(2014, 5, 17).and_hms(0, 0, 0)),

            build_meta("K", Utc.ymd(2014, 4, 3).and_hms(0, 0, 0)),
            build_meta("L", Utc.ymd(2014, 4, 2).and_hms(0, 0, 0)),
            build_meta("M", Utc.ymd(2014, 3, 31).and_hms(0, 0, 0)), // Should be kept for April.

            // The below is expendable, as it's older than `one_per_month_within` from
            // `reference_time`.
            build_meta("N", Utc.ymd(2014, 3, 1).and_hms(0, 0, 0)),
        ];

        let expendable_backups = strategy.expendable_backups(&mut backups);

        assert_eq!(collect_ids(expendable_backups), as_vector("NLKJIGF"));
        assert_eq!(collect_ids(backups), as_vector("ABCDEMH"));
    }

    #[test]
    fn test_expendable_backups_with_no_backups_given() {
        let strategy = OlderThanButKeepOnePerMonth::new(
            Utc.ymd(2014, 6, 15).and_hms(0, 0, 0),
            Duration::days(1),
            Duration::days(15),
            Duration::days(120),
        );
        let mut backups = vec![];

        let expendable_backups = strategy.expendable_backups(&mut backups);

        assert!(expendable_backups.is_empty());
        assert!(backups.is_empty());
    }

    #[test]
    #[should_panic]
    fn test_new_when_one_per_month_within_is_less_than_keep_all_within() {
        OlderThanButKeepOnePerMonth::new(
            Utc.ymd(2014, 6, 15).and_hms(0, 0, 0),
            Duration::days(2),
            Duration::days(15),
            Duration::days(1),
        );
    }

    #[test]
    fn test_new_when_one_per_month_within_equal_to_keep_all_within() {
        // Should not panic.
        OlderThanButKeepOnePerMonth::new(
            Utc.ymd(2014, 6, 15).and_hms(0, 0, 0),
            Duration::days(1),
            Duration::days(15),
            Duration::days(1),
        );
    }
}
