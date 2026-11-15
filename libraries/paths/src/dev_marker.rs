// SPDX-License-Identifier: MIT OR Apache-2.0
//! DEV.json v1: deliberately restricted JSON, with no escaped strings.
//! The complete public contract is beside env.mx in ../README.md.
use std::{collections::BTreeMap, io::Read, path::Path};

pub(crate) const CAPABILITY: &str = "mixviu-dev-snapshot:1";

fn open_marker(path: &Path) -> std::io::Result<std::fs::File> {
    #[cfg(not(target_os = "linux"))]
    return unsupported_open(path);
    #[cfg(target_os = "linux")]
    open_linux_marker(path)
}

#[cfg(any(test, not(target_os = "linux")))]
fn unsupported_open(path: &Path) -> std::io::Result<std::fs::File> {
    // Missing markers must remain NotFound before the unsupported reason.
    std::fs::symlink_metadata(path)?;
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "safe dev marker open is unsupported on this target",
    ))
}

#[cfg(any(test, target_os = "linux"))]
fn require_regular(file: std::fs::File) -> std::io::Result<std::fs::File> {
    if !file.metadata()?.file_type().is_file() {
        return Err(std::io::Error::other("opened marker is not a regular file"));
    }
    Ok(file)
}

#[cfg(target_os = "linux")]
fn open_linux_marker(path: &Path) -> std::io::Result<std::fs::File> {
    use std::os::unix::fs::OpenOptionsExt;
    // Reject special files before opening, then repeat the check on the handle.
    // The open flags below also close the replacement race between these checks.
    if !std::fs::symlink_metadata(path)?.file_type().is_file() {
        return Err(std::io::Error::other(
            "marker must be a regular file, not a symlink",
        ));
    }
    let file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)?;
    require_regular(file)
}

pub(super) fn accepts(
    prefix: &Path,
    member: &str,
    stamp: &str,
    base: &str,
) -> Result<bool, String> {
    let path = prefix.join("DEV.json");
    let file = match open_marker(&path) {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e.to_string()),
    };
    let mut bytes = Vec::new();
    file.take(65537)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    let source = std::str::from_utf8(&bytes).map_err(|e| e.to_string())?;
    let (gold, members) = parse(source)?;
    if gold != base || members.get(member).map(String::as_str) != Some(stamp) {
        return Err(format!(
            "does not authorise member {member} at {stamp} on base {base}"
        ));
    }
    Ok(true)
}

fn commit(s: &str) -> bool {
    s.len() == 40
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn member_name(name: &str) -> bool {
    name.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

fn parse(source: &str) -> Result<(String, BTreeMap<String, String>), String> {
    if source.len() > 65536 {
        return Err("exceeds 64 KiB".into());
    }
    let mut p = Json(source.as_bytes());
    p.token(b'{')?;
    let mut schema = false;
    let mut gold = None;
    let mut members = None;
    loop {
        let key = p.string()?;
        p.token(b':')?;
        match key.as_str() {
            "schema" if !schema => {
                p.token(b'1')?;
                schema = true;
            }
            "gold" if gold.is_none() => {
                let value = p.string()?;
                if !commit(&value) {
                    return Err("invalid gold commit".into());
                }
                gold = Some(value);
            }
            "members" if members.is_none() => {
                p.token(b'{')?;
                let mut map = BTreeMap::new();
                loop {
                    let name = p.string()?;
                    if !member_name(&name) {
                        return Err("invalid member name".into());
                    }
                    p.token(b':')?;
                    let value = p.string()?;
                    if !commit(&value) || map.insert(name, value).is_some() {
                        return Err("invalid or duplicate member commit".into());
                    }
                    if p.end_map()? {
                        break;
                    }
                }
                members = Some(map);
            }
            _ => return Err(format!("unknown or duplicate field {key}")),
        }
        if p.end_map()? {
            break;
        }
    }
    p.space();
    if !p.0.is_empty() || !schema {
        return Err("invalid schema or trailing data".into());
    }
    Ok((
        gold.ok_or("missing gold")?,
        members.ok_or("missing members")?,
    ))
}

struct Json<'a>(&'a [u8]);
impl Json<'_> {
    fn space(&mut self) {
        while self
            .0
            .first()
            .is_some_and(|b| matches!(b, b' ' | b'\n' | b'\r' | b'\t'))
        {
            self.0 = &self.0[1..];
        }
    }
    fn token(&mut self, b: u8) -> Result<(), String> {
        self.space();
        if self.0.first() != Some(&b) {
            return Err(format!("expected {}", char::from(b)));
        }
        self.0 = &self.0[1..];
        Ok(())
    }
    fn string(&mut self) -> Result<String, String> {
        self.token(b'"')?;
        let end = self
            .0
            .iter()
            .position(|b| *b == b'"')
            .ok_or("unclosed string")?;
        let value = &self.0[..end];
        if !value
            .iter()
            .all(|b| (0x20..=0x7e).contains(b) && *b != b'\\')
        {
            return Err("strings must be unescaped printable ASCII".into());
        }
        let value = String::from_utf8(value.to_vec()).map_err(|e| e.to_string())?;
        self.0 = &self.0[end + 1..];
        Ok(value)
    }
    fn end_map(&mut self) -> Result<bool, String> {
        self.space();
        if self.0.first() == Some(&b'}') {
            self.0 = &self.0[1..];
            Ok(true)
        } else {
            self.token(b',')?;
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    struct Scratch(std::path::PathBuf);
    impl Scratch {
        fn new(label: &str) -> Self {
            let path =
                std::env::temp_dir().join(format!("paths-marker-{label}-{}", std::process::id()));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn unsupported_open_preserves_not_found_before_unsupported() {
        let scratch = Scratch::new("unsupported");
        let marker = scratch.0.join("DEV.json");
        assert_eq!(
            super::unsupported_open(&marker).unwrap_err().kind(),
            std::io::ErrorKind::NotFound
        );
        std::fs::write(&marker, "{}").unwrap();
        let err = super::unsupported_open(&marker).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::Unsupported);
        assert_eq!(
            err.to_string(),
            "safe dev marker open is unsupported on this target"
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn handle_check_refuses_open_fifo_and_directory_before_reading() {
        use std::os::unix::fs::OpenOptionsExt;
        let scratch = Scratch::new("handles");
        let fifo = scratch.0.join("fifo");
        assert!(
            std::process::Command::new("mkfifo")
                .arg(&fifo)
                .status()
                .unwrap()
                .success()
        );
        let fifo = std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(fifo)
            .unwrap();
        let dir = std::fs::File::open(&scratch.0).unwrap();
        for file in [fifo, dir] {
            assert_eq!(
                super::require_regular(file).unwrap_err().to_string(),
                "opened marker is not a regular file"
            );
        }
        let regular = scratch.0.join("regular");
        std::fs::write(&regular, "{}").unwrap();
        assert!(super::require_regular(std::fs::File::open(regular).unwrap()).is_ok());
    }

    #[test]
    fn string_reader_refuses_escapes_before_member_validation() {
        let mut p = super::Json(br#""v\u0069u""#);
        assert_eq!(
            p.string().unwrap_err(),
            "strings must be unescaped printable ASCII"
        );
    }

    #[test]
    fn every_release_binary_and_package_has_an_authorisable_name() {
        let table = include_str!("../../../tools/release-set.conf.mix");
        let mut names = 0;
        for line in table
            .lines()
            .filter(|line| line.trim_start().starts_with("{package:"))
        {
            let fields: Vec<_> = line.split('"').collect();
            assert_eq!(fields.len(), 5, "release table row shape changed: {line}");
            for name in [fields[1], fields[3]] {
                assert!(
                    super::member_name(name),
                    "unsupported release member: {name}"
                );
                names += 1;
            }
        }
        assert!(names > 0, "release table rows must be checked");
    }
}
