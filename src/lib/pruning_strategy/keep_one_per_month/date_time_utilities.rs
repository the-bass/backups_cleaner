//! Helper functions for date time objects.
use chrono::{DateTime, Utc, Datelike};
use chrono::offset::TimeZone;

/// Return a new date that points to the beginning of the month
/// of `date`.
pub fn beginning_of_month(date: DateTime<Utc>) -> DateTime<Utc> {
    Utc.ymd(date.year(), date.month(), 1).and_hms(0, 0, 0)
}

/// Return a new date that points to the beginning of the following
/// month of `date`.
pub fn beginning_of_next_month(date: DateTime<Utc>) -> DateTime<Utc> {
    let year;
    let month;

    if date.month() == 12 {
        year = date.year() + 1;
        month = 1;
    }
    else {
        year = date.year();
        month = date.month() + 1;
    }

    Utc.ymd(year, month, 1).and_hms(0, 0, 0)
}

/// Returns `true`, if `date_a` is closer to `to_date` than `date_b`, `false`
/// otherwise.
pub fn is_closer(to_date: DateTime<Utc>, date_a: DateTime<Utc>, date_b: DateTime<Utc>) -> bool {
    (date_a - to_date).num_seconds().abs() < (date_b - to_date).num_seconds().abs()
}
