// SPDX-License-Identifier: MIT OR Apache-2.0
//! Registered environment names and lazy installed settings.
include!("env_names.rs");

use std::{ffi::OsString, sync::OnceLock};

/// An empty override is unset, just as in Environment.
pub fn raw(name: &str) -> Option<OsString> {
    std::env::var_os(name).filter(|v| !v.is_empty())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingError {
    NotSet { name: String },
    Unreadable { name: String, detail: String },
    Malformed { name: String, detail: String },
    UnknownSchema { name: String, schema: String },
    SnapshotMismatch { name: String, expected: String, actual: String },
}
impl std::fmt::Display for SettingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotSet { name } => write!(f, "{name} is not set and has no installed default"),
            Self::Unreadable { name, detail } => write!(f, "{name}: cannot read env.mx: {detail}"),
            Self::Malformed { name, detail } => write!(f, "{name}: malformed env.mx: {detail}"),
            Self::UnknownSchema { name, schema } => write!(f, "{name}: unknown env.mx schema {schema}"),
            Self::SnapshotMismatch { name, expected, actual } => write!(f, "{name}: env.mx snapshot {actual} differs from binary {expected}"),
        }
    }
}
impl std::error::Error for SettingError {}

static DEFAULTS: OnceLock<std::result::Result<Option<super::env_document::Defaults>, super::env_document::Error>> = OnceLock::new();

/// Environment wins, then the installed snapshot's default. Reading is lazy.
pub fn setting(name: &str) -> std::result::Result<String, SettingError> {
    if let Some(value) = raw(name) {
        return value.into_string().map_err(|_| SettingError::Malformed {
            name: name.into(), detail: "environment value is not UTF-8".into(),
        });
    }
    let defaults = DEFAULTS.get_or_init(|| {
        let prefix = super::Layout::from_source(&super::Environment::current(), "")
            .and_then(|d| d.try_root(super::Root::Prefix))
            .map_err(super::env_document::Error::Unreadable)?;
        super::env_document::read(&prefix, stamp!(GIT_SHA_FULL))
    });
    let defaults = defaults.as_ref().map_err(|e| e.for_name(name))?;
    let value = defaults.as_ref().and_then(|d| d.values.get(name))
        .ok_or_else(|| SettingError::NotSet { name: name.into() })?;
    if value.starts_with("mix:") {
        super::resolve(value).map(|p| p.to_string_lossy().into_owned())
            .map_err(|detail| SettingError::Malformed { name: name.into(), detail })
    } else { Ok(value.clone()) }
}

