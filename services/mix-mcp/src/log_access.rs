//! Confined, bounded log reads. Limits refuse rather than silently truncate.
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::{Component, Path, PathBuf};

const MAX_BYTES: usize = 8 * 1024 * 1024;
const MAX_LINE: usize = 64 * 1024;
const MAX_RESULTS: usize = 10_000;

pub fn resolve(dir: &Path, name: &str) -> Result<PathBuf, String> {
    let input = Path::new(name);
    if input.is_absolute() || input.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err("log path refused: absolute paths and '..' components are forbidden".into());
    }
    let dir = dir.canonicalize().map_err(|e| format!("log directory: {e}"))?;
    let exact = dir.join(if name == "bus" { "bus.log" } else { name });
    let candidate = if exact.symlink_metadata().is_ok() {
        exact
    } else {
        let mut matches: Vec<_> = std::fs::read_dir(&dir)
            .map_err(|e| format!("log directory: {e}"))?
            .flatten()
            .filter(|e| e.file_name().to_str().is_some_and(|f| f.starts_with(name)))
            .map(|e| e.path())
            .collect();
        matches.sort();
        matches.pop().unwrap_or(exact)
    };
    let path = candidate.canonicalize().map_err(|e| format!("log path: {e}"))?;
    if !path.starts_with(&dir) {
        return Err("log path refused: symlink target escapes log directory".into());
    }
    if !path.is_file() {
        return Err("log path refused: expected a regular file".into());
    }
    Ok(path)
}

fn limit(n: usize) -> Result<(), String> {
    if n > MAX_RESULTS {
        return Err(format!("log read refused: result limit exceeds {MAX_RESULTS}"));
    }
    Ok(())
}

pub fn tail(path: &Path, n: usize) -> Result<String, String> {
    limit(n)?;
    if n == 0 { return Ok(String::new()); }
    let mut file = File::open(path).map_err(|e| format!("log read: {e}"))?;
    let len = file.metadata().map_err(|e| e.to_string())?.len();
    let start = len.saturating_sub(MAX_BYTES as u64);
    file.seek(SeekFrom::Start(start)).map_err(|e| e.to_string())?;
    let mut bytes = Vec::new();
    file.take(MAX_BYTES as u64).read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    // Discard the possibly partial first line before UTF-8 decoding.
    let bytes = if start > 0 {
        let first = bytes.iter().position(|b| *b == b'\n')
            .ok_or("log tail refused: line exceeds byte budget")?;
        &bytes[first + 1..]
    } else { &bytes[..] };
    let text = std::str::from_utf8(bytes).map_err(|e| format!("log UTF-8: {e}"))?;
    let lines: Vec<_> = text.lines().rev().take(n).collect();
    if start > 0 && lines.len() < n {
        return Err("log tail refused: requested lines exceed byte budget".into());
    }
    Ok(lines.into_iter().rev().collect::<Vec<_>>().join("\n"))
}

pub fn search(path: &Path, pattern: &str, n: usize) -> Result<String, String> {
    limit(n)?;
    if n == 0 { return Ok(String::new()); }
    let mut reader = BufReader::new(File::open(path).map_err(|e| format!("log read: {e}"))?);
    let mut matches = VecDeque::new();
    let mut retained = 0;
    let pattern = pattern.to_lowercase();
    let mut bytes = Vec::new();
    loop {
        bytes.clear();
        let read = Read::by_ref(&mut reader).take((MAX_LINE + 1) as u64).read_until(b'\n', &mut bytes)
            .map_err(|e| format!("log read: {e}"))?;
        if read == 0 { break; }
        if bytes.len() > MAX_LINE { return Err("log search refused: line exceeds byte budget".into()); }
        let line = std::str::from_utf8(&bytes).map_err(|e| format!("log UTF-8: {e}"))?
            .trim_end_matches('\n').trim_end_matches('\r');
        if line.to_lowercase().contains(&pattern) {
            if matches.len() == n { retained -= matches.pop_front().map_or(0, |s: String| s.len()); }
            retained += line.len();
            if retained > MAX_BYTES { return Err("log search refused: results exceed byte budget".into()); }
            matches.push_back(line.to_string());
        }
    }
    if matches.is_empty() { Ok("No matches found".into()) }
    else { Ok(matches.into_iter().collect::<Vec<_>>().join("\n")) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn paths_are_confined_including_prefix_and_symlinks() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(dir.path().join("bus.log"), "ok").unwrap();
        std::os::unix::fs::symlink(outside.path(), dir.path().join("escape.log")).unwrap();
        std::os::unix::fs::symlink(dir.path().join("bus.log"), dir.path().join("inside.log")).unwrap();
        for name in ["/etc/passwd", "../secret", "a/../bus.log", "escape.log", "escape"] {
            assert!(resolve(dir.path(), name).unwrap_err().contains("refused"), "{name}");
        }
        assert_eq!(resolve(dir.path(), "inside.log").unwrap(), resolve(dir.path(), "bus").unwrap());
    }

    #[test]
    fn large_logs_keep_last_matches_with_bounded_reads() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        for i in 0..500_000 { writeln!(file, "match {i:08} padding padding").unwrap(); }
        let expected = "match 00499998 padding padding\nmatch 00499999 padding padding";
        assert_eq!(tail(file.path(), 2).unwrap(), expected);
        assert_eq!(search(file.path(), "MATCH", 2).unwrap(), expected);
        assert!(tail(file.path(), usize::MAX).is_err());
        assert!(search(file.path(), "", usize::MAX).is_err());
        file.as_file().set_len(0).unwrap();
        file.as_file_mut().rewind().unwrap();
        file.write_all(&vec![b'x'; MAX_BYTES + 1]).unwrap();
        assert!(tail(file.path(), 1).unwrap_err().contains("budget"));
        assert!(search(file.path(), "x", 1).unwrap_err().contains("budget"));
    }
}
