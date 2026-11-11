// SPDX-License-Identifier: MIT OR Apache-2.0
//! Flat env.mx schema reader. Shares lexical string rules, not layout parsing.
use super::document::Reader;
use std::{collections::BTreeMap, io::Read, path::Path};

#[derive(Debug, Clone)]
pub(super) enum Error {
    NoPrefix(String),
    Unreadable(String),
    Malformed(String),
    UnknownSchema(String),
    SnapshotMismatch { expected: String, actual: String },
}
impl Error {
    pub fn for_name(&self, name: &str) -> super::env::SettingError {
        use super::env::SettingError as E;
        let name = name.into();
        match self {
            Self::NoPrefix(detail) => E::NoPrefix {
                name,
                detail: detail.clone(),
            },
            Self::Unreadable(detail) => E::Unreadable {
                name,
                detail: detail.clone(),
            },
            Self::Malformed(detail) => E::Malformed {
                name,
                detail: detail.clone(),
            },
            Self::UnknownSchema(schema) => E::UnknownSchema {
                name,
                schema: schema.clone(),
            },
            Self::SnapshotMismatch { expected, actual } => E::SnapshotMismatch {
                name,
                expected: expected.clone(),
                actual: actual.clone(),
            },
        }
    }
}
#[derive(Debug)]
pub(super) struct Defaults {
    pub values: BTreeMap<String, String>,
}

pub(super) fn read(prefix: &Path) -> Result<Option<Defaults>, Error> {
    let file = match std::fs::File::open(prefix.join("env.mx")) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(Error::Unreadable(e.to_string())),
    };
    let mut bytes = Vec::new();
    file.take(65537)
        .read_to_end(&mut bytes)
        .map_err(|e| Error::Unreadable(e.to_string()))?;
    let source = std::str::from_utf8(&bytes).map_err(|e| Error::Malformed(e.to_string()))?;
    parse(source, super::env::SNAPSHOT.get().copied()).map(Some)
}
pub(super) fn parse(source: &str, stamp: Option<&str>) -> Result<Defaults, Error> {
    if source.len() > 65536 {
        return Err(Error::Malformed("exceeds 64 KiB".into()));
    }
    if source.starts_with('\u{feff}') {
        return Err(Error::Malformed("starts with a byte order mark".into()));
    }
    let mut p = Reader {
        chars: source.chars().peekable(),
    };
    p.space();
    let braced = p.chars.peek() == Some(&'{');
    if braced {
        p.chars.next();
    }
    let mut values = BTreeMap::new();
    let mut schema = None;
    loop {
        p.space();
        if braced && p.chars.peek() == Some(&'}') {
            p.chars.next();
            p.space();
            if p.chars.next().is_some() {
                return Err(Error::Malformed("data after map".into()));
            }
            break;
        }
        if p.chars.peek().is_none() {
            if braced {
                return Err(Error::Malformed("unclosed map".into()));
            }
            break;
        }
        let key = if matches!(p.chars.peek(), Some('\'' | '"')) {
            p.string().map_err(Error::Malformed)?
        } else {
            let mut key = String::new();
            while p
                .chars
                .peek()
                .is_some_and(|c| c.is_ascii_alphanumeric() || *c == '_')
            {
                key.push(p.chars.next().unwrap());
            }
            key
        };
        p.space();
        if key.is_empty() || p.chars.next() != Some(':') {
            return Err(Error::Malformed("expected key: value".into()));
        }
        p.space();
        let value = if key == "schema" {
            let mut n = String::new();
            while p.chars.peek().is_some_and(char::is_ascii_digit) {
                n.push(p.chars.next().unwrap());
            }
            if n.is_empty() || schema.replace(n.clone()).is_some() {
                return Err(Error::Malformed("invalid or duplicate schema".into()));
            }
            n
        } else {
            let value = p.string().map_err(Error::Malformed)?;
            if value.contains(['\n', '\r']) {
                return Err(Error::Malformed(format!("{key} contains a newline")));
            }
            value
        };
        if key != "schema"
            && key != "commit"
            && (!key.starts_with("MIX_")
                || !key
                    .chars()
                    .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_'))
        {
            return Err(Error::Malformed(format!("invalid setting name {key}")));
        }
        if values.insert(key, value).is_some() {
            return Err(Error::Malformed("duplicate key".into()));
        }
        let newline = p.space();
        if p.chars.peek() == Some(&',') {
            p.chars.next();
        } else if p.chars.peek().is_some() && p.chars.peek() != Some(&'}') && (braced || !newline) {
            return Err(Error::Malformed("expected entry separator".into()));
        }
    }
    let schema = schema.ok_or_else(|| Error::Malformed("missing schema".into()))?;
    if schema != "1" {
        return Err(Error::UnknownSchema(schema));
    }
    let commit = values
        .remove("commit")
        .ok_or_else(|| Error::Malformed("missing commit".into()))?;
    if commit.len() != 40 || !commit.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')) {
        return Err(Error::Malformed(
            "commit must be 40 lower-case hex digits".into(),
        ));
    }
    if let Some(expected) = stamp
        && expected != commit
    {
        return Err(Error::SnapshotMismatch {
            expected: expected.into(),
            actual: commit,
        });
    }
    values.remove("schema");
    Ok(Defaults { values })
}
