// SPDX-License-Identifier: MIT OR Apache-2.0
//! Stable timestamp wire conventions over `time`.
//!
//! This is the bounded compatibility surface used by MixViu, not a calendar
//! implementation. `time` owns date validation, arithmetic, parsing and local
//! offsets. This module owns historical precision, suffixes, range limits and
//! strftime extensions, including representation of a parsed leap second.
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    fmt,
    marker::PhantomData,
    ops::{Add, Sub},
    str::FromStr,
};
use time::{OffsetDateTime, PrimitiveDateTime, UtcOffset};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Utc;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Local;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FixedOffset;

pub trait Zone: Copy {
    const UTC: bool = false;
    fn offset(at: OffsetDateTime) -> UtcOffset;
}
impl Zone for Utc {
    const UTC: bool = true;
    fn offset(_: OffsetDateTime) -> UtcOffset {
        UtcOffset::UTC
    }
}
impl Zone for Local {
    fn offset(at: OffsetDateTime) -> UtcOffset {
        UtcOffset::local_offset_at(at).unwrap_or(UtcOffset::UTC)
    }
}
impl Zone for FixedOffset {
    fn offset(at: OffsetDateTime) -> UtcOffset {
        at.offset()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DateTime<Z = Utc> {
    inner: OffsetDateTime,
    leap: bool,
    zone: PhantomData<Z>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecondsFormat {
    Secs,
    Millis,
    Micros,
    Nanos,
    AutoSi,
}

#[derive(Debug, Clone)]
pub struct ParseError(String);
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for ParseError {}
fn invalid(e: impl fmt::Display) -> ParseError {
    ParseError(e.to_string())
}

const MIN_YEAR: i32 = -262_143;
const MAX_YEAR: i32 = 262_142;
fn bounded(t: OffsetDateTime) -> bool {
    (MIN_YEAR..=MAX_YEAR).contains(&t.to_offset(UtcOffset::UTC).year())
}

impl<Z: Zone> DateTime<Z> {
    fn new(inner: OffsetDateTime, leap: bool) -> Self {
        Self {
            inner,
            leap,
            zone: PhantomData,
        }
    }
    pub fn timestamp(&self) -> i64 {
        self.inner.unix_timestamp()
    }
    pub fn timestamp_millis(&self) -> i64 {
        (self.inner.unix_timestamp_nanos().div_euclid(1_000_000)) as i64
            + if self.leap { 1000 } else { 0 }
    }
    pub fn timestamp_micros(&self) -> i64 {
        (self.inner.unix_timestamp_nanos().div_euclid(1000)) as i64
            + if self.leap { 1_000_000 } else { 0 }
    }
    pub fn with_timezone<T: Zone>(&self, _: &T) -> DateTime<T> {
        DateTime::new(self.inner.to_offset(T::offset(self.inner)), self.leap)
    }
    pub fn date_naive(&self) -> NaiveDate {
        NaiveDate(self.inner.date())
    }
    pub fn year(&self) -> i32 {
        self.inner.year()
    }
    pub fn checked_add_signed(&self, d: Duration) -> Option<Self> {
        let t = self.inner.checked_add(d.0)?;
        bounded(t).then(|| Self::new(t, false))
    }
    pub fn to_rfc3339(&self) -> String {
        self.to_rfc3339_opts(SecondsFormat::AutoSi, false)
    }
    pub fn to_rfc3339_opts(&self, precision: SecondsFormat, use_z: bool) -> String {
        let base = format_value(self.inner, self.leap, Z::UTC, "%Y-%m-%dT%H:%M:%S")
            .expect("static timestamp format");
        let frac = fraction(self.inner.nanosecond(), precision);
        let offset = if use_z && self.inner.offset().is_utc() {
            "Z".into()
        } else {
            offset_text(self.inner.offset(), 1)
        };
        format!("{base}{frac}{offset}")
    }
    pub fn to_rfc2822(&self) -> String {
        self.format("%a, %-d %b %Y %H:%M:%S %z").to_string()
    }
    pub fn format<'a>(&self, pattern: &'a str) -> Formatted<'a> {
        Formatted {
            inner: self.inner,
            leap: self.leap,
            utc: Z::UTC,
            pattern,
        }
    }
}
impl DateTime<Utc> {
    pub const MAX_UTC: Self = Self {
        inner: time::macros::datetime!(+262142-12-31 23:59:59.999999999 UTC),
        leap: false,
        zone: PhantomData,
    };
    pub fn from_timestamp(seconds: i64, nanos: u32) -> Option<Self> {
        let leap = nanos >= 1_000_000_000;
        if nanos >= 2_000_000_000 || (leap && seconds.rem_euclid(60) != 59) {
            return None;
        }
        let t = OffsetDateTime::from_unix_timestamp_nanos(
            i128::from(seconds) * 1_000_000_000 + i128::from(nanos % 1_000_000_000),
        )
        .ok()?;
        bounded(t).then(|| Self::new(t, leap))
    }
    pub fn from_timestamp_millis(ms: i64) -> Option<Self> {
        Self::from_timestamp(ms.div_euclid(1000), ms.rem_euclid(1000) as u32 * 1_000_000)
    }
}
impl Default for DateTime<Utc> {
    fn default() -> Self {
        Self::new(OffsetDateTime::UNIX_EPOCH, false)
    }
}
impl DateTime<FixedOffset> {
    pub fn parse_from_rfc3339(text: &str) -> Result<Self, ParseError> {
        // Preserve second 60 rather than time's RFC parser's leap-second clamp.
        let leap = text.as_bytes().get(17..19) == Some(b"60");
        let mut s = text.to_owned();
        if leap {
            s.replace_range(17..19, "59");
        }
        let t = OffsetDateTime::parse(&s, &time::format_description::well_known::Rfc3339)
            .map_err(invalid)?;
        if !bounded(t) {
            return Err(invalid("timestamp outside supported range"));
        }
        Ok(Self::new(t, leap))
    }
    pub fn parse_from_str(s: &str, pattern: &str) -> Result<Self, ParseError> {
        let desc = time::format_description::parse_strftime_borrowed(pattern).map_err(invalid)?;
        let t = OffsetDateTime::parse(&month_case(s), &desc).map_err(invalid)?;
        if !bounded(t) {
            return Err(invalid("timestamp outside supported range"));
        }
        Ok(Self::new(t, false))
    }
}
impl Utc {
    pub fn now() -> DateTime<Self> {
        DateTime::new(OffsetDateTime::now_utc(), false)
    }
    pub fn with_ymd_and_hms(
        &self,
        y: i32,
        m: u32,
        d: u32,
        h: u32,
        min: u32,
        s: u32,
    ) -> Option<DateTime<Self>> {
        NaiveDate::from_ymd_opt(y, m, d)?
            .and_hms_opt(h, min, s)
            .map(|v| v.and_utc())
    }
    pub fn timestamp_opt(&self, s: i64, ns: u32) -> LocalResult<DateTime<Self>> {
        LocalResult(DateTime::from_timestamp(s, ns))
    }
    pub fn timestamp_millis_opt(&self, ms: i64) -> LocalResult<DateTime<Self>> {
        LocalResult(DateTime::from_timestamp_millis(ms))
    }
}
impl Local {
    pub fn now() -> DateTime<Self> {
        Utc::now().with_timezone(&Self)
    }
}
pub struct LocalResult<T>(Option<T>);
impl<T> LocalResult<T> {
    pub fn single(self) -> Option<T> {
        self.0
    }
    pub fn unwrap(self) -> T {
        self.0.expect("valid datetime")
    }
}
impl From<std::time::SystemTime> for DateTime<Utc> {
    fn from(t: std::time::SystemTime) -> Self {
        Self::new(t.into(), false)
    }
}
impl<Z: Zone> FromStr for DateTime<Z> {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let t = DateTime::parse_from_rfc3339(s)?;
        Ok(Self::new(t.inner.to_offset(Z::offset(t.inner)), t.leap))
    }
}
impl<Z: Zone> fmt::Display for DateTime<Z> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_rfc3339())
    }
}
impl<Z: Zone> PartialEq for DateTime<Z> {
    fn eq(&self, b: &Self) -> bool {
        self.inner == b.inner && self.leap == b.leap
    }
}
impl<Z: Zone> Eq for DateTime<Z> {}
impl<Z: Zone> PartialOrd for DateTime<Z> {
    fn partial_cmp(&self, b: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(b))
    }
}
impl<Z: Zone> Ord for DateTime<Z> {
    fn cmp(&self, b: &Self) -> std::cmp::Ordering {
        (self.timestamp(), self.leap, self.inner.nanosecond()).cmp(&(
            b.timestamp(),
            b.leap,
            b.inner.nanosecond(),
        ))
    }
}
impl<Z: Zone> Serialize for DateTime<Z> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_rfc3339_opts(SecondsFormat::AutoSi, true))
    }
}
impl<'de, Z: Zone> Deserialize<'de> for DateTime<Z> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        String::deserialize(d)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}
impl<Z: Zone> Add<Duration> for DateTime<Z> {
    type Output = Self;
    fn add(self, d: Duration) -> Self {
        self.checked_add_signed(d)
            .expect("timestamp addition overflow")
    }
}
impl<Z: Zone> Sub<Duration> for DateTime<Z> {
    type Output = Self;
    fn sub(self, d: Duration) -> Self {
        self + Duration(-d.0)
    }
}
impl<Z: Zone> Sub for DateTime<Z> {
    type Output = Duration;
    fn sub(self, b: Self) -> Duration {
        Duration(self.inner - b.inner)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Duration(time::Duration);
impl Duration {
    pub const fn zero() -> Self {
        Self(time::Duration::ZERO)
    }
    pub const fn seconds(n: i64) -> Self {
        Self(time::Duration::seconds(n))
    }
    pub const fn milliseconds(n: i64) -> Self {
        Self(time::Duration::milliseconds(n))
    }
    pub const fn minutes(n: i64) -> Self {
        Self(time::Duration::minutes(n))
    }
    pub const fn hours(n: i64) -> Self {
        Self(time::Duration::hours(n))
    }
    pub const fn days(n: i64) -> Self {
        Self(time::Duration::days(n))
    }
    pub fn try_seconds(n: i64) -> Option<Self> {
        (n.unsigned_abs() <= (i64::MAX / 1000) as u64).then(|| Self::seconds(n))
    }
    pub fn num_seconds(self) -> i64 {
        self.0.whole_seconds()
    }
    pub fn num_days(self) -> i64 {
        self.0.whole_days()
    }
    pub fn abs(self) -> Self {
        Self(self.0.abs())
    }
}
impl Add for Duration {
    type Output = Self;
    fn add(self, b: Self) -> Self {
        Self(self.0 + b.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct NaiveDate(time::Date);
impl NaiveDate {
    pub fn pred_opt(self) -> Option<Self> {
        let d = self.0.previous_day()?;
        (d.year() >= MIN_YEAR).then_some(Self(d))
    }
    pub fn from_ymd_opt(y: i32, m: u32, d: u32) -> Option<Self> {
        if !(MIN_YEAR..=MAX_YEAR).contains(&y) {
            return None;
        }
        Some(Self(
            time::Date::from_calendar_date(
                y,
                u8::try_from(m).ok()?.try_into().ok()?,
                d.try_into().ok()?,
            )
            .ok()?,
        ))
    }
    pub fn parse_from_str(s: &str, p: &str) -> Result<Self, ParseError> {
        let desc = time::format_description::parse_strftime_borrowed(p).map_err(invalid)?;
        let d = time::Date::parse(&month_case(s), &desc).map_err(invalid)?;
        Self::from_ymd_opt(d.year(), d.month() as u32, d.day() as u32)
            .ok_or_else(|| invalid("date outside supported range"))
    }
    pub fn and_hms_opt(self, h: u32, m: u32, s: u32) -> Option<NaiveDateTime> {
        Some(NaiveDateTime(
            self.0
                .with_hms(h.try_into().ok()?, m.try_into().ok()?, s.try_into().ok()?)
                .ok()?,
            false,
        ))
    }
    pub fn num_days_from_ce(self) -> i32 {
        self.0.to_julian_day() - 1_721_425
    }
}
impl Sub for NaiveDate {
    type Output = Duration;
    fn sub(self, b: Self) -> Duration {
        Duration(self.0 - b.0)
    }
}
impl fmt::Display for NaiveDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(
            &format_value(self.0.midnight().assume_utc(), false, true, "%Y-%m-%d")
                .map_err(|_| fmt::Error)?,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct NaiveDateTime(PrimitiveDateTime, bool);
impl NaiveDateTime {
    pub fn parse_from_str(s: &str, p: &str) -> Result<Self, ParseError> {
        let desc = time::format_description::parse_strftime_borrowed(p).map_err(invalid)?;
        let mut input = month_case(s);
        let seconds = match p {
            "%Y-%m-%d %H:%M:%S" => Some(17..19),
            "%Y%m%dT%H%M%SZ" => Some(13..15),
            _ => None,
        };
        let leap = seconds.as_ref().is_some_and(|r| input.as_bytes().get(r.clone()) == Some(b"60"));
        if leap { input.replace_range(seconds.expect("seconds field"), "59"); }
        let t = PrimitiveDateTime::parse(&input, &desc).map_err(invalid)?;
        if !bounded(t.assume_utc()) {
            return Err(invalid("datetime outside supported range"));
        }
        Ok(Self(t, leap))
    }
    pub fn and_utc(self) -> DateTime<Utc> {
        DateTime::new(self.0.assume_utc(), self.1)
    }
}

fn month_case(s: &str) -> String {
    let mut s = s.to_owned();
    for month in [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ] {
        // IMAP dates have ASCII month names. Preserve all other input bytes.
        for i in (0..s.len().saturating_sub(2)).rev() {
            if s.is_char_boundary(i)
                && s.is_char_boundary(i + 3)
                && s[i..i + 3].eq_ignore_ascii_case(month)
            {
                s.replace_range(i..i + 3, month);
            }
        }
    }
    s
}
fn fraction(n: u32, p: SecondsFormat) -> String {
    match p {
        SecondsFormat::Secs => String::new(),
        SecondsFormat::Millis => format!(".{:03}", n / 1_000_000),
        SecondsFormat::Micros => format!(".{:06}", n / 1000),
        SecondsFormat::Nanos => format!(".{n:09}"),
        SecondsFormat::AutoSi => {
            if n == 0 {
                String::new()
            } else if n.is_multiple_of(1_000_000) {
                format!(".{:03}", n / 1_000_000)
            } else if n.is_multiple_of(1000) {
                format!(".{:06}", n / 1000)
            } else {
                format!(".{n:09}")
            }
        }
    }
}
fn offset_text(offset: UtcOffset, colons: usize) -> String {
    let n = offset.whole_seconds();
    let sign = if n < 0 { '-' } else { '+' };
    let n = n.unsigned_abs();
    let (h, m, s) = (n / 3600, n / 60 % 60, n % 60);
    match colons {
        0 => format!("{sign}{h:02}{m:02}"),
        1 => format!("{sign}{h:02}:{m:02}"),
        2 => format!("{sign}{h:02}:{m:02}:{s:02}"),
        _ => format!("{sign}{h:02}"),
    }
}
pub struct Formatted<'a> {
    inner: OffsetDateTime,
    leap: bool,
    utc: bool,
    pattern: &'a str,
}
impl fmt::Display for Formatted<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(
            &format_value(self.inner, self.leap, self.utc, self.pattern).map_err(|_| fmt::Error)?,
        )
    }
}

fn format_value(
    t: OffsetDateTime,
    leap: bool,
    utc: bool,
    pattern: &str,
) -> Result<String, ParseError> {
    let mut out = String::new();
    let mut iter = pattern.char_indices().peekable();
    while let Some((start, c)) = iter.next() {
        if c != '%' {
            out.push(c);
            continue;
        }
        let mut spec = String::new();
        let end = loop {
            let (i, c) = iter
                .next()
                .ok_or_else(|| invalid("incomplete strftime conversion"))?;
            spec.push(c);
            if c.is_ascii_alphabetic() || c == '%' || c == '+' {
                break i + c.len_utf8();
            }
        };
        let token = &pattern[start..end];
        match spec.as_str() {
            "%" => out.push('%'),
            "+" => out.push_str(&DateTime::<FixedOffset>::new(t, leap).to_rfc3339()),
            "Z" => out.push_str(&if utc {
                "UTC".into()
            } else {
                offset_text(t.offset(), 1)
            }),
            "z" => out.push_str(&offset_text(t.offset(), 0)),
            ":z" => out.push_str(&offset_text(t.offset(), 1)),
            "::z" => out.push_str(&offset_text(t.offset(), 2)),
            ":::z" => out.push_str(&offset_text(t.offset(), 3)),
            "f" => out.push_str(&format!("{:09}", t.nanosecond())),
            ".f" => out.push_str(&fraction(t.nanosecond(), SecondsFormat::AutoSi)),
            "S" if leap => out.push_str("60"),
            "w" => out.push_str(&t.weekday().number_days_from_sunday().to_string()),
            "Y" => {
                let y = t.year();
                if (0..=9999).contains(&y) {
                    out.push_str(&format!("{y:04}"));
                } else if y < 0 {
                    out.push_str(&format!("-{:04}", y.unsigned_abs()));
                } else {
                    out.push_str(&format!("+{y}"));
                }
            }
            "F" => out.push_str(&format_value(t, leap, utc, "%Y-%m-%d")?),
            "v" => out.push_str(&format_value(t, leap, utc, "%e-%b-%Y")?),
            "q" => out.push_str(&((t.month() as u8 - 1) / 3 + 1).to_string()),
            _ if spec.ends_with('f') => {
                let dot = spec.starts_with('.');
                let n = spec
                    .trim_start_matches('.')
                    .trim_end_matches('f')
                    .parse::<usize>()
                    .map_err(invalid)?;
                if ![3, 6, 9].contains(&n) {
                    return Err(invalid("invalid fractional precision"));
                }
                if dot {
                    out.push('.');
                }
                out.push_str(&format!("{:09}", t.nanosecond())[..n]);
            }
            _ => {
                let desc =
                    time::format_description::parse_strftime_borrowed(token).map_err(invalid)?;
                out.push_str(&t.format(&desc).map_err(invalid)?);
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn serde_utc_suffix_and_fraction_groups_are_stable() {
        for (nanos, fraction) in [
            (0, ""),
            (1_000_000, ".001"),
            (123_456_000, ".123456"),
            (123_456_789, ".123456789"),
        ] {
            let t = DateTime::from_timestamp(0, nanos).unwrap();
            let json = format!("\"1970-01-01T00:00:00{fraction}Z\"");
            assert_eq!(serde_json::to_string(&t).unwrap(), json);
            assert_eq!(serde_json::from_str::<DateTime<Utc>>(&json).unwrap(), t);
        }
    }
    #[test]
    fn checked_expiry_and_fractional_freshness() {
        let t = Utc::now();
        assert!(Duration::try_seconds(i64::MAX).is_none());
        assert!(
            DateTime::<Utc>::MAX_UTC
                .checked_add_signed(Duration::seconds(1))
                .is_none()
        );
        assert!(((t + Duration::milliseconds(300_001)) - t).abs() > Duration::seconds(300));
        assert_eq!(
            NaiveDate::from_ymd_opt(1, 1, 1).unwrap().num_days_from_ce(),
            1
        );
    }
    #[test]
    fn leap_second_spelling_and_epoch_values() {
        let t = DateTime::parse_from_rfc3339("2016-12-31T23:59:60.123Z").unwrap();
        assert_eq!(t.timestamp(), 1_483_228_799);
        assert_eq!(t.timestamp_millis(), 1_483_228_800_123);
        assert_eq!(t.to_rfc3339(), "2016-12-31T23:59:60.123+00:00");
        assert_eq!(t.format("%S %.3f").to_string(), "60 .123");
    }
}
