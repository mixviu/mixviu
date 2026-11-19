// SPDX-License-Identifier: MIT OR Apache-2.0
//! Launch the reviewed scene host as a direct compositor child.
pub struct Host(std::process::Child);

fn next_delay(delay: std::time::Duration) -> std::time::Duration {
    (delay * 2).min(std::time::Duration::from_secs(30))
}

/// Availability checks do not request compositor frames. A failed child loses
/// its pin before any replacement is spawned; repeated crashes back off.
pub fn supervise<S: 'static>(
    handle: smithay::reexports::calloop::LoopHandle<'static, S>,
    headless: bool,
    comp: String,
) {
    use smithay::reexports::calloop::timer::{TimeoutAction, Timer};
    use std::time::{Duration, Instant};
    let mut host = launch(headless, &comp);
    let mut started = Instant::now();
    // Half-step seed: each failure doubles before scheduling its delay.
    let mut delay = Duration::from_millis(125);
    let mut retry = started + Duration::from_millis(250);
    if host.is_none() {
        delay = next_delay(delay);
    }
    handle
        .insert_source(
            Timer::from_duration(Duration::from_millis(250)),
            move |_, _, _| {
                let now = Instant::now();
                if let Some(child) = &mut host {
                    match child.0.try_wait() {
                        Ok(None) => {
                            if now.duration_since(started) >= Duration::from_secs(30) {
                                delay = Duration::from_millis(125);
                            }
                        }
                        _ => {
                            // Drop unbinds even if the child has already been reaped.
                            host = None;
                            delay = next_delay(delay);
                            retry = now + delay;
                        }
                    }
                } else if now >= retry {
                    host = launch(headless, &comp);
                    started = now;
                    if host.is_none() {
                        delay = next_delay(delay);
                    }
                    retry = now + delay;
                }
                TimeoutAction::ToDuration(if host.is_some() {
                    Duration::from_millis(250)
                } else {
                    retry.saturating_duration_since(now)
                })
            },
        )
        .expect("OSD supervisor timer");
}

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
    let bundle = prefix.join("share/mixviu/apps/osd");
    let viu = bin.join("viu");
    #[cfg(feature = "test-support")]
    let (bundle, viu) = if headless {
        (
            std::env::var_os("MIX_TEST_VIUD_OSD_BUNDLE")
                .map(std::path::PathBuf::from)
                .unwrap_or(bundle),
            std::env::var_os("MIX_TEST_VIU_EXE")
                .map(std::path::PathBuf::from)
                .unwrap_or(viu),
        )
    } else {
        (bundle, viu)
    };
    let _ = headless;
    if !bundle.join("contract.json").is_file() {
        info!("OSD bundle is absent; scene host not started");
        return None;
    }
    let mut command = slots::library::process::child::hygiene::command(viu);
    command
        .arg("run")
        .arg(bundle)
        .arg("--comp")
        .arg(comp)
        .stdin(std::process::Stdio::null())
        .env("MIX_VIU_COMP_SERVICE", comp);
    let parent = std::process::id() as libc::pid_t;
    // SAFETY: the child hook uses only async-signal-safe syscalls, no allocation.
    // The parent check closes the race where viud dies before prctl is armed.
    unsafe {
        command.pre_exec(move || {
            if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            if libc::getppid() != parent {
                libc::_exit(1);
            }
            Ok(())
        });
    }
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
        Err(error) => {
            error!("OSD host launch failed: {error}");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn restart_backoff_doubles_before_each_retry_and_caps() {
        use std::time::Duration;
        let mut delay = Duration::from_millis(125);
        let mut sequence = Vec::new();
        for _ in 0..10 {
            delay = super::next_delay(delay);
            sequence.push(delay.as_millis());
        }
        assert_eq!(
            sequence,
            [250, 500, 1000, 2000, 4000, 8000, 16000, 30000, 30000, 30000]
        );
    }
}
