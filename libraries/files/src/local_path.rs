// SPDX-License-Identifier: MIT OR Apache-2.0
//! Lossless local path tokens. UTF-8 paths retain their existing spelling;
//! other absolute Unix paths use the existing local file-URI codec.
use std::{
    os::unix::ffi::{OsStrExt, OsStringExt},
    path::{Path, PathBuf},
};
pub fn encode(path: &Path) -> Result<String, String> {
    match path.to_str() {
        Some(path) => Ok(path.into()),
        None => crate::chooser::file_uri(path.as_os_str().as_bytes()).map_err(|e| e.message),
    }
}
pub fn decode(text: &str) -> Result<PathBuf, String> {
    if text.starts_with("file://") {
        Ok(std::ffi::OsString::from_vec(
            crate::chooser::location(&serde_json::json!(text)).map_err(|e| e.message)?,
        )
        .into())
    } else {
        Ok(text.into())
    }
}
pub mod serde_path {
    use super::*;
    use serde::{Deserialize, Deserializer, Serializer};
    pub fn serialize<S: Serializer>(path: &Path, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&encode(path).map_err(serde::ser::Error::custom)?)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<PathBuf, D::Error> {
        decode(&String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn utf8_spelling_stays_compatible_and_raw_paths_roundtrip() {
        for text in ["/tmp/a b", "relative", "~/Downloads"] {
            assert_eq!(encode(&decode(text).unwrap()).unwrap(), text);
        }
        let raw = Path::new(std::ffi::OsStr::from_bytes(b"/tmp/raw\xff"));
        let token = encode(raw).unwrap();
        assert!(token.contains("%FF"));
        assert_eq!(decode(&token).unwrap(), raw);
        assert!(decode("file://remote.example/path").is_err());
    }
}
