// SPDX-License-Identifier: MIT OR Apache-2.0
//! Captured against chrono 0.4.45 at pre-migration commit
//! 3806f83fde3492f6a1325ebafd94398639c2ec71, CBC job
//! 20261117-232708-2cac5d96-a1. Both accepted and rejected inputs are fixtures.
use datetime::{DateTime, NaiveDate, NaiveDateTime, SecondsFormat, Utc};
use serde_json::{Value, json};

#[test]
fn pre_migration_parse_corpus() {
    let rows: Vec<Value> = serde_json::from_str(include_str!("data/parse.json")).unwrap();
    for row in rows {
        let kind = row["kind"].as_str().unwrap();
        let text = row["text"].as_str().unwrap();
        let result = std::panic::catch_unwind(|| match kind {
            "date" => NaiveDate::parse_from_str(text, "%Y-%m-%d").ok().and_then(|d| d.and_hms_opt(0,0,0)).map(|d| d.and_utc()),
            "naive" => NaiveDateTime::parse_from_str(text, "%Y-%m-%d %H:%M:%S").ok().map(|d| d.and_utc()),
            "basic" => NaiveDateTime::parse_from_str(text, "%Y%m%dT%H%M%SZ").ok().map(|d| d.and_utc()),
            "imap" => DateTime::parse_from_str(text, "%d-%b-%Y %H:%M:%S %z").ok().map(|d| d.with_timezone(&Utc)),
            "relaxed" => text.parse::<DateTime<Utc>>().ok(),
            "rfc" => DateTime::parse_from_rfc3339(text).ok().map(|d| d.with_timezone(&Utc)),
            _ => panic!("unknown oracle group"),
        });
        assert!(result.is_ok(), "panic in {kind}: {text:?}");
        let actual = result.unwrap().map(|t| json!({"seconds":t.timestamp(),"millis":t.timestamp_millis(),"utc":t.to_rfc3339_opts(SecondsFormat::AutoSi,true)})).unwrap_or(Value::Null);
        assert_eq!(actual, row["expected"], "{kind}: {text:?}");
        if kind == "relaxed" {
            let parsed = serde_json::from_value::<DateTime<Utc>>(json!(text));
            assert_eq!(parsed.is_ok(), !row["expected"].is_null(), "serde: {text:?}");
            if let Ok(t) = parsed { assert_eq!(json!(t), row["expected"]["utc"]); }
        }
    }
}

#[test]
fn extended_year_offset_errors_never_panic() {
    for text in ["31-Dec-+999999 23:59:59 -0100", "01-Jan--999999 00:00:00 +0100"] {
        let result = std::panic::catch_unwind(|| DateTime::parse_from_str(text, "%d-%b-%Y %H:%M:%S %z"));
        assert!(result.is_ok(), "panic: {text}");
        assert!(result.unwrap().is_err(), "accepted: {text}");
    }
}

#[test]
fn weekday_padding_flags_keep_sunday_zero() {
    let t = DateTime::<Utc>::from_timestamp(1_700_000_000,0).unwrap();
    assert_eq!(t.format("%w %-w %_w %0w").to_string(), "2 2 2 2");
}
