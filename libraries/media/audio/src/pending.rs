//! Same-directory staging with atomic, no-clobber publication. Hard links
//! provide rename-like publication without replacing an existing destination.
use std::{
    fs::File,
    io,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
/// Staged output shared by offline media commands. Publish every member of a
/// transaction before committing any; Drop rolls back uncommitted publications.
pub struct Pending {
    pub file: File,
    temporary: PathBuf,
    destination: PathBuf,
    published: Option<(u64, u64)>,
    committed: bool,
}
impl Pending {
    pub fn new(destination: &Path) -> io::Result<Self> {
        match destination.symlink_metadata() {
            Ok(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "output already exists",
                ));
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
        let directory = destination.parent().unwrap_or(Path::new("."));
        for _ in 0..100 {
            let temporary = directory.join(format!(
                ".av-{}-{}.tmp",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match File::create_new(&temporary) {
                Ok(file) => {
                    return Ok(Self {
                        file,
                        temporary,
                        destination: destination.to_owned(),
                        published: None,
                        committed: false,
                    });
                }
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "cannot create temporary output",
        ))
    }
    pub fn publish(&mut self) -> io::Result<()> {
        self.file.sync_all()?;
        let expected = identity(&self.file.metadata()?);
        std::fs::hard_link(&self.temporary, &self.destination)?;
        self.published = Some(expected);
        let linked = identity(&self.destination.symlink_metadata()?);
        if linked != expected {
            remove_if_identity(&self.destination, linked);
            self.published = None;
            return Err(io::Error::other(
                "output inode differs from synced temporary descriptor",
            ));
        }
        self.sync_directory()?;
        Ok(())
    }
    fn sync_directory(&self) -> io::Result<()> {
        let parent = self
            .destination
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        File::open(parent)?.sync_all()
    }
    pub fn commit(&mut self) {
        self.committed = true;
    }
}
fn identity(metadata: &std::fs::Metadata) -> (u64, u64) {
    (metadata.dev(), metadata.ino())
}
fn remove_if_identity(path: &Path, expected: (u64, u64)) {
    if path
        .symlink_metadata()
        .is_ok_and(|m| identity(&m) == expected)
    {
        let _ = std::fs::remove_file(path);
    }
}

impl Drop for Pending {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.temporary);
        if let Some(expected) = self.published.filter(|_| !self.committed) {
            remove_if_identity(&self.destination, expected);
        }
        let _ = self.sync_directory();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn symlink_planted_after_staging_is_preserved() {
        let dir = std::env::temp_dir().join(format!("av-late-link-{}", std::process::id()));
        std::fs::create_dir(&dir).unwrap();
        let victim = dir.join("victim");
        let destination = dir.join("out");
        std::fs::write(&victim, b"keep").unwrap();
        {
            let mut pending = Pending::new(&destination).unwrap();
            std::os::unix::fs::symlink(&victim, &destination).unwrap();
            assert!(pending.publish().is_err());
        }
        assert!(destination.is_symlink());
        assert_eq!(std::fs::read(&victim).unwrap(), b"keep");
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn second_publication_failure_rolls_back_first() {
        let dir = std::env::temp_dir().join(format!("av-pair-{}", std::process::id()));
        std::fs::create_dir(&dir).unwrap();
        let first = dir.join("first");
        let second = dir.join("second");
        {
            let mut a = Pending::new(&first).unwrap();
            let mut b = Pending::new(&second).unwrap();
            a.publish().unwrap();
            // A destination planted after staging still cannot be replaced.
            std::fs::write(&second, b"planted").unwrap();
            assert!(b.publish().is_err());
        }
        assert!(!first.exists());
        assert_eq!(std::fs::read(&second).unwrap(), b"planted");
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
        std::fs::remove_file(second).unwrap();
        std::fs::remove_dir(dir).unwrap();
    }
    #[test]
    fn swapped_temporary_is_not_published() {
        use std::io::Write;
        let dir = std::env::temp_dir().join(format!("av-swap-{}", std::process::id()));
        std::fs::create_dir(&dir).unwrap();
        let destination = dir.join("out.webm");
        {
            let mut pending = Pending::new(&destination).unwrap();
            pending.file.write_all(b"master").unwrap();
            std::fs::remove_file(&pending.temporary).unwrap();
            std::fs::write(&pending.temporary, b"replacement").unwrap();
            assert!(
                pending
                    .publish()
                    .unwrap_err()
                    .to_string()
                    .contains("inode differs")
            );
            assert!(!destination.exists());
        }
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
        std::fs::remove_dir(dir).unwrap();
    }
    #[test]
    fn sync_failure_never_publishes() {
        let dir = std::env::temp_dir().join(format!("av-sync-{}", std::process::id()));
        std::fs::create_dir(&dir).unwrap();
        let destination = dir.join("out.webm");
        {
            let mut pending = Pending::new(&destination).unwrap();
            // A non-syncable descriptor models a late file durability error.
            pending.file = File::open("/dev/full").unwrap();
            assert!(pending.publish().is_err());
            assert!(!destination.exists());
        }
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
        std::fs::remove_dir(dir).unwrap();
    }
}
