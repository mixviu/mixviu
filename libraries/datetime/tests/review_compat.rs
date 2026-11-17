// SPDX-License-Identifier: MIT OR Apache-2.0
// Values captured from chrono 0.4.45 on pre-migration commit 3806f83fde34,
// CBC job 20261117-235925-9f6c56d3-a1 (review_addendum_oracle).
use datetime::{DateTime, Duration, NaiveDate, NaiveDateTime, Utc};

#[test]
fn leap_subtraction_across_days_matches_chrono() {
    for (left, right, seconds) in [
        ("2017-01-01T00:00:00.123Z", "2016-12-31T23:59:60.123Z", 0),
        ("2017-01-01T01:00:00.123Z", "2016-12-31T23:59:60.123Z", 3_600),
        ("2017-01-01T12:00:00Z", "2016-12-31T00:29:60Z", 127_801),
        ("2016-12-31T12:00:00Z", "2016-12-31T00:29:60Z", 41_401),
    ] {
        let a = DateTime::parse_from_rfc3339(left).unwrap();
        let b = DateTime::parse_from_rfc3339(right).unwrap();
        assert_eq!(a - b, Duration::seconds(seconds), "{left} minus {right}");
        assert_eq!(b - a, Duration::seconds(-seconds), "{right} minus {left}");
    }
}

#[test]
fn leap_nanosecond_and_year_bounds_match_chrono() {
    for (seconds, valid) in [(0, false), (58, false), (59, true), (-1, true), (-2, false)] {
        assert_eq!(DateTime::<Utc>::from_timestamp(seconds, 1_123_000_000).is_some(), valid);
    }
    for (year, valid) in [(-262144, false), (-262143, true), (262142, true), (262143, false)] {
        assert_eq!(NaiveDate::from_ymd_opt(year, 1, 1).is_some(), valid);
    }
    assert_eq!(DateTime::<Utc>::MAX_UTC.to_string(), "+262142-12-31 23:59:59.999999999 UTC");
}

#[test]
fn strftime_leap_seconds_are_not_pattern_specific() {
    for (text, pattern) in [
        ("31-Dec-2016 23:59:60 +0000", "%d-%b-%Y %H:%M:%S %z"),
        ("2016-12-31T23:59:60+0000", "%Y-%m-%dT%H:%M:%S%z"),
    ] {
        let t = DateTime::parse_from_str(text, pattern).unwrap();
        assert_eq!(t.to_rfc3339(), "2016-12-31T23:59:60+00:00");
    }
    let t = NaiveDateTime::parse_from_str("2016/12/31 23:59:60", "%Y/%m/%d %H:%M:%S").unwrap();
    assert_eq!(t.and_utc().to_rfc3339(), "2016-12-31T23:59:60+00:00");
}
