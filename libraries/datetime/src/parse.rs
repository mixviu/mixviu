// SPDX-License-Identifier: MIT OR Apache-2.0
//! Lexical compatibility for the strftime subset used by workspace parsers.
//! Numeric padding and whitespace are accepted here; time validates the calendar.
use super::{MAX_YEAR, MIN_YEAR, ParseError, bounded, invalid};
use time::{OffsetDateTime, PrimitiveDateTime, UtcOffset};

pub(super) struct Parsed {
    pub naive: PrimitiveDateTime,
    pub leap: bool,
    pub offset: Option<UtcOffset>,
}
impl Parsed {
    pub fn zoned(self) -> Result<(OffsetDateTime, bool), ParseError> {
        let offset = self.offset.ok_or_else(|| invalid("missing UTC offset"))?;
        // build() checks our year range before constructing this value. It is
        // strictly inside time's range, with room for every legal UTC offset.
        let t = self.naive.assume_offset(offset);
        if !bounded(t) {
            return Err(invalid("timestamp outside supported range"));
        }
        Ok((t, self.leap))
    }
}

#[derive(Default)]
struct Fields {
    year: Option<i32>,
    month: Option<u32>,
    day: Option<u32>,
    hour: Option<u32>,
    minute: Option<u32>,
    second: Option<u32>,
    nanos: u32,
    offset: Option<UtcOffset>,
}
impl Fields {
    fn build(self, clock: bool) -> Result<Parsed, ParseError> {
        let year = self.year.ok_or_else(|| invalid("missing year"))?;
        if !(MIN_YEAR..=MAX_YEAR).contains(&year) {
            return Err(invalid("date outside supported range"));
        }
        let month = self.month.ok_or_else(|| invalid("missing month"))?;
        let day = self.day.ok_or_else(|| invalid("missing day"))?;
        let date = time::Date::from_calendar_date(
            year,
            u8::try_from(month)
                .map_err(invalid)?
                .try_into()
                .map_err(invalid)?,
            day.try_into().map_err(invalid)?,
        )
        .map_err(invalid)?;
        let get = |field: Option<u32>| {
            if clock {
                field.ok_or_else(|| invalid("missing clock field"))
            } else {
                Ok(field.unwrap_or(0))
            }
        };
        let hour = get(self.hour)?;
        let minute = get(self.minute)?;
        let second = get(self.second)?;
        if second > 60 {
            return Err(invalid("second outside supported range"));
        }
        let leap = second == 60;
        let naive = date
            .with_hms_nano(
                hour.try_into().map_err(invalid)?,
                minute.try_into().map_err(invalid)?,
                second.min(59) as u8,
                self.nanos,
            )
            .map_err(invalid)?;
        Ok(Parsed {
            naive,
            leap,
            offset: self.offset,
        })
    }
}

struct Input<'a>(&'a str);
impl Input<'_> {
    fn space(&mut self) {
        self.0 = self.0.trim_start();
    }
    fn literal(&mut self, c: char) -> Result<(), ParseError> {
        self.0 = self
            .0
            .strip_prefix(c)
            .ok_or_else(|| invalid("date literal does not match"))?;
        Ok(())
    }
    fn digits(&mut self, max: usize, trim: bool) -> Result<u32, ParseError> {
        if trim {
            self.space();
        }
        let n = self
            .0
            .bytes()
            .take(max)
            .take_while(u8::is_ascii_digit)
            .count();
        if n == 0 {
            return Err(invalid("expected numeric date field"));
        }
        let value = self.0[..n].parse().map_err(invalid)?;
        self.0 = &self.0[n..];
        Ok(value)
    }
    fn year(&mut self) -> Result<i32, ParseError> {
        self.space();
        let signed = self.0.starts_with(['+', '-']);
        let negative = self.0.starts_with('-');
        if signed {
            self.0 = &self.0[1..];
        }
        let n = self.digits(if signed { usize::MAX } else { 4 }, false)?;
        let year = i32::try_from(n).map_err(invalid)?;
        Ok(if negative { -year } else { year })
    }
    fn month(&mut self) -> Result<u32, ParseError> {
        let word = self
            .0
            .get(..3)
            .ok_or_else(|| invalid("expected month name"))?;
        let month = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ]
        .iter()
        .position(|m| word.eq_ignore_ascii_case(m))
        .ok_or_else(|| invalid("invalid month name"))?;
        self.0 = &self.0[3..];
        Ok(month as u32 + 1)
    }
    fn fraction(&mut self) -> Result<u32, ParseError> {
        if !self.0.starts_with('.') {
            return Ok(0);
        }
        self.0 = &self.0[1..];
        let n = self.0.bytes().take_while(u8::is_ascii_digit).count();
        if n == 0 {
            return Err(invalid("missing fractional digits"));
        }
        let significant = n.min(9);
        let value = self.0[..significant].parse::<u32>().map_err(invalid)?
            * 10u32.pow(9 - significant as u32);
        self.0 = &self.0[n..];
        Ok(value)
    }
    fn offset(&mut self, zulu: bool) -> Result<UtcOffset, ParseError> {
        if zulu && self.0.starts_with(['Z', 'z']) {
            self.0 = &self.0[1..];
            return Ok(UtcOffset::UTC);
        }
        let sign = match self.0.chars().next() {
            Some('+') => 1,
            Some('-' | '−') => -1,
            _ => return Err(invalid("expected UTC offset")),
        };
        let n = self
            .0
            .chars()
            .next()
            .ok_or_else(|| invalid("missing offset"))?
            .len_utf8();
        self.0 = &self.0[n..];
        // Unlike calendar numeric fields, an offset has exactly two digits
        // per component, and cannot have whitespace immediately after its sign.
        let component = |input: &mut Self| -> Result<u32, ParseError> {
            if input
                .0
                .as_bytes()
                .get(..2)
                .is_none_or(|v| !v.iter().all(u8::is_ascii_digit))
            {
                return Err(invalid("expected two offset digits"));
            }
            input.digits(2, false)
        };
        let hours = component(self)?;
        self.0 = self
            .0
            .trim_start_matches(|c: char| c == ':' || c.is_whitespace());
        let minutes = component(self)?;
        if hours > 23 || minutes > 59 {
            return Err(invalid("UTC offset outside supported range"));
        }
        UtcOffset::from_whole_seconds(sign * (hours as i32 * 3600 + minutes as i32 * 60))
            .map_err(invalid)
    }
    fn finish(&mut self, trailing_space: bool) -> Result<(), ParseError> {
        if trailing_space {
            self.space();
        }
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(invalid("trailing date input"))
        }
    }
}

pub(super) fn strftime(text: &str, pattern: &str, clock: bool) -> Result<Parsed, ParseError> {
    let mut input = Input(text);
    let mut fields = Fields::default();
    let mut pattern = pattern.chars();
    while let Some(c) = pattern.next() {
        if c.is_whitespace() {
            input.space();
            continue;
        }
        if c != '%' {
            input.literal(c)?;
            continue;
        }
        match pattern.next() {
            Some('Y') => fields.year = Some(input.year()?),
            Some('m') => fields.month = Some(input.digits(2, true)?),
            Some('b' | 'h') => fields.month = Some(input.month()?),
            Some('d' | 'e') => fields.day = Some(input.digits(2, true)?),
            Some('H') => fields.hour = Some(input.digits(2, true)?),
            Some('M') => fields.minute = Some(input.digits(2, true)?),
            Some('S') => fields.second = Some(input.digits(2, true)?),
            Some('z') => fields.offset = Some(input.offset(false)?),
            Some('%') => input.literal('%')?,
            _ => return Err(invalid("unsupported date parse pattern")),
        }
    }
    input.finish(false)?;
    fields.build(clock)
}

pub(super) fn relaxed(text: &str) -> Result<Parsed, ParseError> {
    let mut input = Input(text);
    let mut fields = Fields {
        year: Some(input.year()?),
        ..Fields::default()
    };
    input.space();
    input.literal('-')?;
    fields.month = Some(input.digits(2, true)?);
    input.space();
    input.literal('-')?;
    fields.day = Some(input.digits(2, true)?);
    match input.0.chars().next() {
        Some(c @ ('T' | 't' | ' ')) => input.literal(c)?,
        _ => return Err(invalid("invalid date-time separator")),
    }
    fields.hour = Some(input.digits(2, true)?);
    input.space();
    input.literal(':')?;
    fields.minute = Some(input.digits(2, true)?);
    input.space();
    input.literal(':')?;
    fields.second = Some(input.digits(2, true)?);
    fields.nanos = input.fraction()?;
    input.space();
    fields.offset = Some(
        if input
            .0
            .get(..3)
            .is_some_and(|s| s.eq_ignore_ascii_case("UTC"))
        {
            input.0 = &input.0[3..];
            UtcOffset::UTC
        } else {
            input.offset(true)?
        },
    );
    input.finish(true)?;
    fields.build(true)
}
