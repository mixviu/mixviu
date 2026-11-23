//! Same-directory staging with atomic, no-clobber publication. Hard links
//! provide rename-like publication without replacing an existing destination.
use std::{
    fs::File,
    io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
pub(crate) struct Pending {
    pub file: File,
    temporary: PathBuf,
    destination: PathBuf,
    published: bool,
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
                        published: false,
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
        std::fs::hard_link(&self.temporary, &self.destination)?;
        self.published = true;
        self.sync_directory()?;
        Ok(())
    }
    fn sync_directory(&self) -> io::Result<()> {
        let parent = self.destination.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
        File::open(parent)?.sync_all()
    }
    pub fn commit(&mut self) {
        self.committed = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
impl Drop for Pending {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.temporary);
        if self.published && !self.committed {
            let _ = std::fs::remove_file(&self.destination);
        }
        let _ = self.sync_directory();
    }
}
