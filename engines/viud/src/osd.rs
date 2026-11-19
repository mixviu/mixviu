// SPDX-License-Identifier: MIT OR Apache-2.0
//! Launch the reviewed scene host as a direct compositor child.
pub struct Host(std::process::Child);

impl Drop for Host {
    fn drop(&mut self) {
        protocols::layershell::osd::unbind();
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

pub fn launch(headless: bool, comp: &str) -> Option<Host> {
    use std::os::unix::process::CommandExt;
    let executable = std::env::current_exe().ok()?;
    let bin = executable.parent()?;
    let prefix = bin.parent()?;
    let mut bundle = prefix.join("share/mixviu/apps/osd");
    let mut viu = bin.join("viu");
    #[cfg(feature = "test-support")]
    if headless {
        if let Some(path) = std::env::var_os("MIX_TEST_VIUD_OSD_BUNDLE") { bundle = path.into(); }
        if let Some(path) = std::env::var_os("MIX_TEST_VIU_EXE") { viu = path.into(); }
    }
    let _ = headless;
    if !bundle.join("contract.json").is_file() { return None; }
    let mut command = slots::library::process::child::hygiene::command(viu);
    command.arg("run").arg(bundle).arg("--comp").arg(comp)
        .env("MIX_VIU_COMP_SERVICE", comp);
    let parent = std::process::id() as libc::pid_t;
    // SAFETY: the child hook uses only async-signal-safe syscalls, no allocation.
    // The parent check closes the race where viud dies before prctl is armed.
    unsafe { command.pre_exec(move || {
        if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL) != 0 {
            return Err(std::io::Error::last_os_error());
        }
        if libc::getppid() != parent { libc::_exit(1); }
        Ok(())
    }); }
    match slots::library::process::child::spawn::spawn_awaited(&mut command) {
        Ok(mut child) => {
            if !protocols::layershell::osd::bind(child.id() as i32) {
                let _ = child.kill();
                let _ = child.wait();
                error!("OSD host identity could not be pinned");
                return None;
            }
            Some(Host(child))
        }
        Err(error) => { error!("OSD host launch failed: {error}"); None },
    }
}
