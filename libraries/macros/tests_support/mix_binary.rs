// SPDX-License-Identifier: MIT OR Apache-2.0
//! Select a test interpreter independently of the libtest executable.
use std::path::{Path, PathBuf};

pub fn executable(path: &Path) -> bool {
    use std::os::unix::ffi::OsStrExt;
    let Ok(path_c) = std::ffi::CString::new(path.as_os_str().as_bytes()) else {
        return false;
    };
    // SAFETY: path_c is NUL-terminated and alive for this effective-ID check.
    path.is_file()
        && unsafe {
            libc::faccessat(
                libc::AT_FDCWD,
                path_c.as_ptr(),
                libc::X_OK,
                libc::AT_EACCESS,
            ) == 0
        }
}

/// Tests require the exact interpreter selected by the build job.
pub fn select() -> Result<Option<PathBuf>, String> {
    if let Some(path) = paths::env::os(paths::env::test::TEST_E2E_EXE) {
        let path = PathBuf::from(path);
        if !executable(&path) {
            return Err(format!(
                "{} is not an executable file: {}",
                paths::env::test::TEST_E2E_EXE,
                path.display()
            ));
        }
        return path.canonicalize().map(Some).map_err(|e| e.to_string());
    }
    Err(format!("Set {} to the staged newly built mix interpreter (tools/ci.mix stage-install)", paths::env::test::TEST_E2E_EXE))
}
