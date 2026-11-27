// SPDX-License-Identifier: MIT OR Apache-2.0
//! Transactional process replacement. No fork, pre_exec or descriptor sweep.
use std::fs::File;
use std::io::{self, Write};
use std::os::fd::AsRawFd;
use std::os::unix::process::CommandExt;
use std::process::Command;

struct Signals {
    pipe: libc::sigaction,
    mask: libc::sigset_t,
    pipe_saved: bool,
    mask_saved: bool,
}

impl Signals {
    fn prepare() -> io::Result<Self> {
        // SAFETY: pointers to live initialised locals; pthread_sigmask changes
        // only the execing thread. sigaction(SIGPIPE) is process-wide, so the
        // A-to-B window is an accepted risk bounded by the refusals and flush.
        // Caught handlers are untouched: POSIX exec resets them, while inherited
        // ignores must survive. All-zero sigaction/sigset_t storage is valid;
        // sigemptyset initialises the mask before use.
        unsafe {
            let mut state = Self {
                pipe: std::mem::zeroed(),
                mask: std::mem::zeroed(),
                pipe_saved: false,
                mask_saved: false,
            };
            let mut dfl: libc::sigaction = std::mem::zeroed();
            dfl.sa_sigaction = libc::SIG_DFL;
            if libc::sigaction(libc::SIGPIPE, &dfl, &mut state.pipe) != 0 {
                return Err(io::Error::last_os_error());
            }
            state.pipe_saved = true;
            let mut empty = std::mem::zeroed();
            if libc::sigemptyset(&mut empty) != 0 {
                return Err(io::Error::last_os_error());
            }
            let rc = libc::pthread_sigmask(libc::SIG_SETMASK, &empty, &mut state.mask);
            if rc != 0 {
                return Err(io::Error::from_raw_os_error(rc));
            }
            state.mask_saved = true;
            Ok(state)
        }
    }

    fn restore(&mut self) -> io::Result<()> {
        let mut error = None;
        // SAFETY: B restores exactly what A saved, in reverse. Pointers refer
        // to live initialised saved values. The mask affects only this thread;
        // SIGPIPE restoration has the same bounded process-wide risk as A.
        unsafe {
            if self.mask_saved {
                let rc = libc::pthread_sigmask(libc::SIG_SETMASK, &self.mask, std::ptr::null_mut());
                if rc != 0 {
                    error = Some(io::Error::from_raw_os_error(rc));
                } else {
                    self.mask_saved = false;
                }
            }
            if self.pipe_saved {
                if libc::sigaction(libc::SIGPIPE, &self.pipe, std::ptr::null_mut()) != 0 {
                    error = Some(io::Error::last_os_error());
                } else {
                    self.pipe_saved = false;
                }
            }
        }
        error.map_or(Ok(()), Err)
    }
}

impl Drop for Signals {
    fn drop(&mut self) {
        if let Err(error) = self.restore() {
            eprintln!("exec_argv: signal rollback failed: {error}");
        }
    }
}

struct Cwd(Option<File>);

impl Cwd {
    fn restore(&mut self) -> io::Result<()> {
        if let Some(file) = &self.0 {
            std::env::set_current_dir(format!("/proc/self/fd/{}", file.as_raw_fd()))?;
            self.0 = None;
        }
        Ok(())
    }
}

impl Drop for Cwd {
    fn drop(&mut self) {
        if let Err(error) = self.restore() {
            eprintln!("exec_argv: cwd rollback failed: {error}");
        }
    }
}

pub(crate) fn replace(command: &mut Command, cwd: Option<&str>) -> io::Result<()> {
    io::stdout().flush()?;
    io::stderr().flush()?;
    let mut saved = Cwd(None);
    if let Some(cwd) = cwd {
        saved.0 = Some(File::open(".")?);
        std::env::set_current_dir(cwd)?;
    }
    let mut signals = Signals::prepare()?;
    let error = command.exec();
    let signal_restore = signals.restore();
    drop(signals);
    let cwd_restore = saved.restore();
    signal_restore?;
    cwd_restore?;
    Err(error)
}
