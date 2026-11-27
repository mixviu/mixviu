// SPDX-License-Identifier: MIT OR Apache-2.0
//! Purpose-separated, erasing seeds and the shared descriptor-based file policy.
use std::fs::{File, OpenOptions};
use std::io::Read;
use std::os::fd::AsRawFd;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::Path;
use zeroize::Zeroizing;

/// D2 admission material. Deliberately not interchangeable with BusTlsSeed.
#[derive(Clone, PartialEq, Eq)]
pub struct D2Seed(Zeroizing<[u8; 32]>);
impl D2Seed {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(Zeroizing::new(bytes))
    }
}
impl AsRef<[u8]> for D2Seed {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}
impl std::fmt::Debug for D2Seed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<redacted>")
    }
}

/// Independent TLS transport material; never construct this from a D2 seed.
///
/// Outside mesh, raw D2 bytes cannot be passed to the private constructor:
/// ```compile_fail
/// use mesh::seed::{BusTlsSeed, D2Seed};
/// let d2 = D2Seed::new([7; 32]);
/// let tls = BusTlsSeed::new(d2.as_ref().try_into().unwrap());
/// ```
/// Nor is a purpose-changing conversion provided:
/// ```compile_fail
/// use mesh::seed::{BusTlsSeed, D2Seed};
/// let tls: BusTlsSeed = D2Seed::new([7; 32]).into();
/// ```
#[derive(Clone, PartialEq, Eq)]
pub struct BusTlsSeed(Zeroizing<[u8; 32]>);
impl BusTlsSeed {
    pub(crate) fn new(bytes: [u8; 32]) -> Self {
        Self(Zeroizing::new(bytes))
    }
    pub(crate) fn bytes(&self) -> &[u8; 32] {
        &self.0
    }
}
impl std::fmt::Debug for BusTlsSeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<redacted>")
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SeedFileError {
    #[error("seed file IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("seed is not a regular file")]
    NotRegular,
    #[error("insecure seed file: mode={mode:o}, owner={owner}")]
    Insecure { mode: u32, owner: u32 },
    #[error("TLS seed must be exactly 32 bytes")]
    Length,
}

/// fstat results belong to this opened descriptor, never to a prior path lookup.
pub struct SeedFile {
    file: File,
    pub mode: u32,
    pub owner: u32,
    pub secure: bool,
}

impl SeedFile {
    /// Read from the checked regular-file descriptor with O_NONBLOCK cleared.
    pub fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
        self.file.read(bytes)
    }

    /// Read at most limit bytes into erasing storage. None means the opened
    /// file exceeds the limit; never accept a valid prefix of an oversized file.
    pub fn read_bounded(&mut self, limit: u64) -> std::io::Result<Option<Zeroizing<Vec<u8>>>> {
        let mut bytes = Zeroizing::new(Vec::new());
        (&mut self.file).take(limit).read_to_end(&mut bytes)?;
        if self.file.metadata()?.len() > limit {
            return Ok(None);
        }
        Ok(Some(bytes))
    }
}

/// Open without following symlinks or blocking on FIFOs, then require a regular
/// file. Report the common root/euid, 0640-compatible policy separately so D2
/// can warn during rollout while TLS refuses.
pub fn open_seed(path: &Path) -> Result<SeedFile, SeedFileError> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC)
        .open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        return Err(SeedFileError::NotRegular);
    }
    // SAFETY: file owns a live descriptor; these commands take/return flags.
    let flags = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_GETFL) };
    if flags == -1 {
        return Err(std::io::Error::last_os_error().into());
    }
    // SAFETY: the fd remains owned and flags came from F_GETFL above.
    if unsafe { libc::fcntl(file.as_raw_fd(), libc::F_SETFL, flags & !libc::O_NONBLOCK) } == -1 {
        return Err(std::io::Error::last_os_error().into());
    }
    let mode = metadata.mode() & 0o7777;
    let owner = metadata.uid();
    // SAFETY: geteuid has no pointer arguments or preconditions.
    let effective_uid = unsafe { libc::geteuid() };
    let secure = mode & 0o037 == 0 && (owner == 0 || owner == effective_uid);
    Ok(SeedFile {
        file,
        mode,
        owner,
        secure,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn regular_seed_descriptor_is_blocking() {
        let path = std::env::temp_dir().join(format!("seed-flags-{}", uuid::Uuid::new_v4()));
        std::fs::write(&path, [7; 32]).unwrap();
        let opened = open_seed(&path).unwrap();
        // SAFETY: opened owns the live descriptor.
        let flags = unsafe { libc::fcntl(opened.file.as_raw_fd(), libc::F_GETFL) };
        assert_ne!(flags, -1);
        assert_eq!(flags & libc::O_NONBLOCK, 0);
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn purpose_seed_debug_is_redacted() {
        for debug in [
            format!("{:?}", D2Seed::new([173; 32])),
            format!("{:?}", BusTlsSeed::new([173; 32])),
        ] {
            assert_eq!(debug, "<redacted>");
            assert!(!debug.contains("173"));
        }
    }
}
