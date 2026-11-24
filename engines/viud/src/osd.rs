// SPDX-License-Identifier: MIT OR Apache-2.0
//! Launch the reviewed scene host as a direct compositor child.
pub struct Host(std::process::Child, protocols::layershell::osd::Lease);

fn next_delay(delay: std::time::Duration) -> std::time::Duration {
    (delay * 2).min(std::time::Duration::from_secs(30))
}

/// Availability checks do not request compositor frames. A failed child loses
/// its pin before any replacement is spawned; repeated crashes back off.
pub fn supervise<S: 'static>(
    handle: smithay::reexports::calloop::LoopHandle<'static, S>,
    headless: bool,
    comp: String,
    display: smithay::reexports::wayland_server::DisplayHandle,
) {
    use smithay::reexports::calloop::timer::{TimeoutAction, Timer};
    use std::time::{Duration, Instant};
    let mut host = launch(headless, &comp, &display);
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
                    host = launch(headless, &comp, &display);
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
        self.1.revoke();
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

pub fn launch(
    headless: bool,
    comp: &str,
    display: &smithay::reexports::wayland_server::DisplayHandle,
) -> Option<Host> {
    use std::os::fd::AsRawFd;
    use std::os::unix::process::CommandExt;
    let executable = std::env::current_exe().ok()?;
    let bin = executable.parent()?;
    let prefix = bin.parent()?;
    let bundle = prefix.join("share/mixviu/apps/osd");
    let viu = bin.join("viu");
    #[cfg(feature = "backend-headless")]
    let (bundle, viu) = if headless {
        headless::test_osd_paths().unwrap_or((bundle, viu))
    } else {
        (bundle, viu)
    };
    let _ = headless;
    if !bundle.join("contract.json").is_file() {
        info!("OSD bundle is absent; scene host not started");
        return None;
    }
    let mut command = slots::library::process::child::hygiene::command(viu);
    let (server, client) = std::os::unix::net::UnixStream::pair().ok()?;
    let lease = protocols::layershell::osd::insert(display, server).ok()?;
    let fd = client.as_raw_fd();
    command
        .arg("run")
        .arg(bundle)
        .arg("--service")
        .arg(service(comp))
        .arg("--comp")
        .arg(comp)
        .stdin(std::process::Stdio::null())
        .env_remove("WAYLAND_DISPLAY")
        .env("WAYLAND_SOCKET", fd.to_string())
        .env("MIX_VIU_COMP_SERVICE", comp);
    let parent = std::process::id() as libc::pid_t;
    // SAFETY: the child hook uses only async-signal-safe syscalls, no allocation.
    // The parent check closes the race where viud dies before prctl is armed.
    unsafe {
        command.pre_exec(move || {
            if libc::fcntl(fd, libc::F_SETFD, 0) == -1 {
                return Err(std::io::Error::last_os_error());
            }
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
        Ok(child) => Some(Host(child, lease)),
        Err(error) => {
            error!("OSD host launch failed: {error}");
            None
        }
    }
}

// Keep the session's familiar name, while private compositors own private hosts.
fn service(comp: &str) -> String {
    if comp == "comp" {
        "osd".into()
    } else {
        format!("osd-{comp}")
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn private_compositors_do_not_claim_the_session_osd() {
        assert_eq!(super::service("comp"), "osd");
        assert_eq!(super::service("comp-headless"), "osd-comp-headless");
        assert_ne!(
            super::service("comp-headless"),
            super::service("comp-nested")
        );
    }
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
