// SPDX-License-Identifier: MIT OR Apache-2.0
//! Registered environment names and lazy installed settings.
include!("env_names.rs");

use std::{
    ffi::OsString,
    sync::{
        OnceLock,
        atomic::{AtomicBool, Ordering},
    },
};

/// An empty override is unset, just as in Environment.
pub fn raw(name: &str) -> Option<OsString> {
    std::env::var_os(name).filter(|v| !v.is_empty())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingError {
    /// Not in the environment, not supplied by the caller's middle step,
    /// and not in the installed env.mx (or there is no installed env.mx).
    NotSet {
        name: String,
    },
    /// The environment value is set but is not UTF-8.
    NotUtf8 {
        name: String,
    },
    /// The prefix override is set but is not a usable prefix.
    NoPrefix {
        name: String,
        detail: String,
    },
    /// env.mx exists but could not be read.
    Unreadable {
        name: String,
        detail: String,
    },
    /// env.mx was read but is not schema-1 data.
    Malformed {
        name: String,
        detail: String,
    },
    UnknownSchema {
        name: String,
        schema: String,
    },
    /// env.mx belongs to another build than this binary.
    SnapshotMismatch {
        name: String,
        expected: String,
        actual: String,
    },
    /// A `mix:` root reference could not be expanded through the layout.
    Layout {
        name: String,
        detail: String,
    },
}
impl std::fmt::Display for SettingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotSet { name } => write!(f, "{name} is not set and has no installed default"),
            Self::NotUtf8 { name } => write!(f, "{name} is set but is not UTF-8"),
            Self::NoPrefix { name, detail } => {
                write!(f, "{name}: cannot find the install prefix: {detail}")
            }
            Self::Unreadable { name, detail } => write!(f, "{name}: cannot read env.mx: {detail}"),
            Self::Malformed { name, detail } => write!(f, "{name}: malformed env.mx: {detail}"),
            Self::UnknownSchema { name, schema } => {
                write!(f, "{name}: unknown env.mx schema {schema}")
            }
            Self::SnapshotMismatch {
                name,
                expected,
                actual,
            } => write!(
                f,
                "{name}: env.mx snapshot {actual} differs from binary {expected}"
            ),
            Self::Layout { name, detail } => {
                write!(f, "{name}: cannot expand root reference: {detail}")
            }
        }
    }
}
impl std::error::Error for SettingError {}

static DEFAULTS: OnceLock<
    std::result::Result<Option<super::env_document::Defaults>, super::env_document::Error>,
> = OnceLock::new();
pub(super) static SNAPSHOT: OnceLock<&'static str> = OnceLock::new();
/// Set when the first installed-default read starts.
static DEFAULTS_READ: AtomicBool = AtomicBool::new(false);
static CONFLICT_REPORTED: AtomicBool = AtomicBool::new(false);

/// Register the binary's full commit before its first installed setting
/// read. The first call wins and repeating it is harmless.
///
/// The env.mx read is cached once per process, so a registration that
/// arrives after it cannot check anything. In debug builds that is a panic;
/// release builds keep first-wins and the cached, unchecked result. A
/// different later value is ignored, and in debug builds reported once on
/// stderr. With no registration, installed defaults are read without a
/// commit check.
pub fn register_snapshot(commit: &'static str) {
    match SNAPSHOT.set(commit) {
        Ok(()) => {
            if cfg!(debug_assertions) && DEFAULTS_READ.load(Ordering::SeqCst) {
                panic!("paths: build snapshot registered after the first installed setting read");
            }
        }
        Err(_) => {
            let first = SNAPSHOT.get().copied().unwrap_or_default();
            if cfg!(debug_assertions)
                && first != commit
                && !CONFLICT_REPORTED.swap(true, Ordering::SeqCst)
            {
                eprintln!(
                    "paths: conflicting build snapshot {commit} ignored; {first} was registered first"
                );
            }
        }
    }
}

fn from_environment(name: &str) -> std::result::Result<Option<String>, SettingError> {
    raw(name)
        .map(|v| {
            v.into_string()
                .map_err(|_| SettingError::NotUtf8 { name: name.into() })
        })
        .transpose()
}

/// Environment wins, then the installed snapshot's default. Reading is lazy.
pub fn setting(name: &str) -> std::result::Result<String, SettingError> {
    setting_or(name, || None)
}

/// A three-step chain: the environment, else `middle()` (a config file,
/// manifest or probe the caller owns), else the installed default, else the
/// error. `middle` runs only when the environment does not answer.
pub fn setting_or(
    name: &str,
    middle: impl FnOnce() -> Option<String>,
) -> std::result::Result<String, SettingError> {
    if let Some(value) = from_environment(name)? {
        return Ok(value);
    }
    if let Some(value) = middle() {
        return Ok(value);
    }
    installed(name)
}

fn installed(name: &str) -> std::result::Result<String, SettingError> {
    let defaults = DEFAULTS.get_or_init(|| {
        DEFAULTS_READ.store(true, Ordering::SeqCst);
        // The captured process inputs, and the strict rule: a binary that is
        // not installed (a cargo tree) has no installed defaults.
        match super::settings_prefix(super::inputs())
            .map_err(super::env_document::Error::NoPrefix)?
        {
            Some(prefix) => super::env_document::read(&prefix),
            None => Ok(None),
        }
    });
    let defaults = defaults.as_ref().map_err(|e| e.for_name(name))?;
    let value = defaults
        .as_ref()
        .and_then(|d| d.values.get(name))
        .ok_or_else(|| SettingError::NotSet { name: name.into() })?;
    if value.starts_with("mix:") {
        super::resolve(value)
            .and_then(|p| {
                p.into_os_string()
                    .into_string()
                    .map_err(|_| "resolved setting path is not UTF-8".into())
            })
            .map_err(|detail| SettingError::Layout {
                name: name.into(),
                detail,
            })
    } else {
        Ok(value.clone())
    }
}
