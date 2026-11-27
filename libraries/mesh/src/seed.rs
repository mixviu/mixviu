// SPDX-License-Identifier: MIT OR Apache-2.0
//! Purpose-separated, erasing seeds and the shared descriptor-based file policy.
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use zeroize::Zeroizing;

/// D2 admission material. Deliberately not interchangeable with BusTlsSeed.
/// Raw bytes stay inside mesh; noded uses mesh::build_admit_response, which
/// signs only the validated D2 admission transcript.
/// ```compile_fail
/// let d2 = mesh::seed::D2Seed::new([7; 32]);
/// let raw = d2.bytes();
/// ```
/// ```compile_fail
/// let d2 = mesh::seed::D2Seed::new([7; 32]);
/// let raw: &[u8] = d2.as_ref();
/// ```
#[derive(Clone, PartialEq, Eq)]
pub struct D2Seed(Zeroizing<[u8; 32]>);
impl D2Seed {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(Zeroizing::new(bytes))
    }
    pub(crate) fn bytes(&self) -> &[u8; 32] {
        &self.0
    }
}
impl std::fmt::Debug for D2Seed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<redacted>")
    }
}

/// Independent TLS transport material; never construct this from a D2 seed.
///
/// Outside mesh, raw bytes cannot be passed to the private constructor:
/// ```compile_fail
/// let tls = mesh::seed::BusTlsSeed::new([7; 32]);
/// ```
/// Nor is a purpose-changing conversion provided:
/// ```compile_fail
/// use mesh::seed::{BusTlsSeed, D2Seed};
/// let tls: BusTlsSeed = D2Seed::new([7; 32]).into();
/// ```
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
    let secure = secure_mode(mode, owner, effective_uid);
    Ok(SeedFile {
        file,
        mode,
        owner,
        secure,
    })
}

fn secure_mode(mode: u32, owner: u32, effective_uid: u32) -> bool {
    if owner == effective_uid {
        mode & 0o077 == 0
    } else {
        owner == 0 && mode & 0o037 == 0
    }
}

/// Public result only: seed material never leaves this module.
pub struct GeneratedBusTlsSeed {
    pub path: PathBuf,
    pub public_key: [u8; 32],
}

/// Create a protected node-local seed, never replacing the active file.
/// Rotation stages alongside it; epoch selection and activation are separate.
pub fn generate_bus_tls_seed(
    path: &Path,
    rotate: bool,
) -> Result<GeneratedBusTlsSeed, SeedFileError> {
    if !path.is_absolute() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "seed path must be absolute",
        )
        .into());
    }
    // SAFETY: geteuid has no arguments or preconditions.
    let uid = unsafe { libc::geteuid() };
    let gid = if uid == 0 { Some(noded_gid()?) } else { None };
    let path = if rotate {
        let mut name = path.as_os_str().to_os_string();
        name.push(format!(".next-{}", uuid::Uuid::new_v4()));
        PathBuf::from(name)
    } else {
        path.to_owned()
    };
    write_generated_seed(&path, gid)
}

fn noded_gid() -> Result<u32, SeedFileError> {
    let mut buffer = vec![0u8; 16384];
    let mut entry = std::mem::MaybeUninit::<libc::group>::uninit();
    let mut found = std::ptr::null_mut();
    // SAFETY: the entry and scratch buffer are writable and live through lookup.
    let rc = unsafe {
        libc::getgrnam_r(
            c"noded".as_ptr(),
            entry.as_mut_ptr(),
            buffer.as_mut_ptr().cast(),
            buffer.len(),
            &mut found,
        )
    };
    if rc != 0 {
        return Err(std::io::Error::from_raw_os_error(rc).into());
    }
    if found.is_null() {
        return Err(
            std::io::Error::new(std::io::ErrorKind::NotFound, "noded group not found").into(),
        );
    }
    // SAFETY: successful lookup initialised entry; only the numeric gid escapes.
    Ok(unsafe { entry.assume_init() }.gr_gid)
}

fn write_generated_seed(
    path: &Path,
    root_gid: Option<u32>,
) -> Result<GeneratedBusTlsSeed, SeedFileError> {
    let mut bytes = Zeroizing::new([0u8; 32]);
    File::open("/dev/urandom")?.read_exact(bytes.as_mut())?;
    let seed = BusTlsSeed::new(*bytes);
    let public_key = ed25519_dalek::SigningKey::from_bytes(seed.bytes())
        .verifying_key()
        .to_bytes();
    // Start private even if later ownership setup fails. O_EXCL also refuses symlinks.
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .map_err(|error| {
            std::io::Error::new(error.kind(), format!("{}: {error}", path.display()))
        })?;
    // Arm cleanup only after this call has exclusively created the file.
    let result = (|| -> Result<GeneratedBusTlsSeed, SeedFileError> {
        if let Some(gid) = root_gid {
            // SAFETY: file owns the descriptor; uid/gid are numeric lookup results.
            if unsafe { libc::fchown(file.as_raw_fd(), 0, gid) } != 0 {
                return Err(std::io::Error::last_os_error().into());
            }
            file.set_permissions(std::fs::Permissions::from_mode(0o640))?;
        } else {
            file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
        }
        file.write_all(seed.bytes())?;
        file.sync_all()?;
        File::open(path.parent().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "seed has no parent")
        })?)?
        .sync_all()?;
        Ok(GeneratedBusTlsSeed {
            path: path.to_owned(),
            public_key,
        })
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(path);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn failed_ownership_setup_unlinks_created_seed() {
        if unsafe { libc::geteuid() } == 0 {
            return;
        }
        let path = std::env::temp_dir().join(format!("seed-chown-{}", uuid::Uuid::new_v4()));
        assert!(write_generated_seed(&path, Some(u32::MAX - 1)).is_err());
        assert!(!path.exists());
    }
    #[test]
    fn generated_seed_loads_and_is_create_only() {
        let dir = std::env::temp_dir().join(format!("bus-tls-gen-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("bus-tls.seed");
        // SAFETY: identity getters have no preconditions.
        let gid = unsafe { (libc::geteuid() == 0).then(|| libc::getegid()) };
        let generated = write_generated_seed(&path, gid).unwrap();
        let seed = crate::tls::load_seed(&path).unwrap();
        assert_eq!(
            generated.public_key,
            ed25519_dalek::SigningKey::from_bytes(seed.bytes())
                .verifying_key()
                .to_bytes()
        );
        let original = std::fs::read(&path).unwrap();
        assert!(write_generated_seed(&path, gid).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), original);
        assert_eq!(
            std::fs::metadata(&path).unwrap().mode() & 0o777,
            if gid.is_some() { 0o640 } else { 0o600 }
        );
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn group_read_is_reserved_for_root_owned_seeds() {
        assert!(secure_mode(0o600, 1000, 1000));
        assert!(!secure_mode(0o640, 1000, 1000));
        assert!(secure_mode(0o640, 0, 1000));
        assert!(!secure_mode(0o640, 0, 0));
        assert!(!secure_mode(0o600, 1001, 1000));
        for mode in [0o620, 0o604, 0o644, 0o666] {
            assert!(!secure_mode(mode, 0, 1000));
            assert!(!secure_mode(mode, 1000, 1000));
        }
    }
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
