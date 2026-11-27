// SPDX-License-Identifier: MIT OR Apache-2.0
//! Purpose-separated, erasing seeds and the shared descriptor-based file policy.
use std::fs::{File, OpenOptions};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::Path;
use zeroize::Zeroizing;

/// D2 admission material. Deliberately not interchangeable with BusTlsSeed.
#[derive(Clone, PartialEq, Eq)]
pub struct D2Seed(Zeroizing<[u8; 32]>);
impl D2Seed {
    pub fn new(bytes: [u8; 32]) -> Self { Self(Zeroizing::new(bytes)) }
}
impl AsRef<[u8]> for D2Seed {
    fn as_ref(&self) -> &[u8] { self.0.as_ref() }
}
impl std::fmt::Debug for D2Seed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str("<redacted>") }
}

/// Independent TLS transport material; never construct this from a D2 seed.
#[derive(Clone, PartialEq, Eq)]
pub struct BusTlsSeed(Zeroizing<[u8; 32]>);
impl BusTlsSeed {
    pub fn new(bytes: [u8; 32]) -> Self { Self(Zeroizing::new(bytes)) }
    pub(crate) fn bytes(&self) -> &[u8; 32] { &self.0 }
}
impl std::fmt::Debug for BusTlsSeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str("<redacted>") }
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
    pub file: File,
    pub mode: u32,
    pub owner: u32,
    pub secure: bool,
}

/// Open without following symlinks or blocking on FIFOs, then require a regular
/// file. Report the common root/euid, 0640-compatible policy separately so D2
/// can warn during rollout while TLS refuses.
pub fn open_seed(path: &Path) -> Result<SeedFile, SeedFileError> {
    let file = OpenOptions::new().read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC)
        .open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() { return Err(SeedFileError::NotRegular); }
    let mode = metadata.mode() & 0o7777;
    let owner = metadata.uid();
    // SAFETY: geteuid has no pointer arguments or preconditions.
    let effective_uid = unsafe { libc::geteuid() };
    let secure = mode & 0o037 == 0 && (owner == 0 || owner == effective_uid);
    Ok(SeedFile { file, mode, owner, secure })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn purpose_seed_debug_is_redacted() {
        for debug in [format!("{:?}", D2Seed::new([173; 32])), format!("{:?}", BusTlsSeed::new([173; 32]))] {
            assert_eq!(debug, "<redacted>");
            assert!(!debug.contains("173"));
        }
    }
}
