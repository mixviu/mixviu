// SPDX-License-Identifier: MIT OR Apache-2.0
//! A prefix containing only bin/mix must remain a usable login shell.
#![cfg(target_os = "linux")]

use std::fs::{self, File};
use std::io::{Read, Write};
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

struct Bare {
    _scratch: tempfile::TempDir,
    prefix: PathBuf,
    home: PathBuf,
}

impl Bare {
    fn new() -> Self {
        let source = std::path::Path::new(env!("CARGO_BIN_EXE_mix"));
        let scratch = tempfile::tempdir_in(source.parent().unwrap()).unwrap();
        let prefix = scratch.path().join("prefix");
        let home = scratch.path().join("home");
        fs::create_dir_all(prefix.join("bin")).unwrap();
        fs::create_dir(&home).unwrap();
        fs::hard_link(source, prefix.join("bin/mix")).unwrap();
        assert!(!prefix.join("env.mx").exists());
        assert!(!prefix.join("layout.mx").exists());
        Self {
            _scratch: scratch,
            prefix,
            home,
        }
    }

    fn command(&self, explicit_prefix: bool) -> Command {
        let mut command = Command::new(self.prefix.join("bin/mix"));
        command
            .env_clear()
            .env("HOME", &self.home)
            .env("PATH", "/usr/bin:/bin")
            .env("TERM", "xterm")
            .current_dir(&self.home)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if explicit_prefix {
            command.env(paths::env::PREFIX, &self.prefix);
        }
        command
    }

    fn run(&self, explicit_prefix: bool, args: &[&str]) -> Output {
        let child = self.command(explicit_prefix).args(args).spawn().unwrap();
        bounded_output(child, Duration::from_secs(10))
    }
}

fn bounded_output(mut child: Child, limit: Duration) -> Output {
    let started = Instant::now();
    while child.try_wait().unwrap().is_none() {
        if started.elapsed() > limit {
            let _ = child.kill();
            let output = child.wait_with_output().unwrap();
            panic!(
                "bare shell timed out: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    child.wait_with_output().unwrap()
}

fn successful(output: Output) -> String {
    assert!(
        output.status.success(),
        "{:?}: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn binary_alone_supports_version_commands_scripts_cd_and_builtin_manual() {
    let bare = Bare::new();
    let script = bare.home.join("script.mix");
    fs::write(&script, "print(1)\n").unwrap();
    for explicit in [false, true] {
        assert!(successful(bare.run(explicit, &["--version"])).contains("mix"));
        assert_eq!(successful(bare.run(explicit, &["-c", "print(1)"])), "1\n");
        assert_eq!(
            successful(bare.run(explicit, &[script.to_str().unwrap()])),
            "1\n"
        );
        assert_eq!(
            successful(bare.run(explicit, &["-c", "cd; pwd"])).trim(),
            bare.home.to_str().unwrap()
        );
        let manual = successful(bare.run(explicit, &["man", "syntax"]));
        assert!(manual.contains("syntax"), "{manual}");
        assert!(!manual.contains("cannot read env.mx"), "{manual}");
    }
}

#[test]
fn unresolvable_prefix_does_not_prevent_script_or_builtin_prelude() {
    let bare = Bare::new();
    let script = bare.home.join("script.mix");
    fs::write(&script, "print(1)\n").unwrap();
    let child = bare
        .command(false)
        .env(paths::env::PREFIX, "relative-prefix")
        .arg(&script)
        .spawn()
        .unwrap();
    assert_eq!(
        successful(bounded_output(child, Duration::from_secs(10))),
        "1\n"
    );
}

#[test]
fn remote_default_absence_is_a_feature_configuration_error() {
    let bare = Bare::new();
    let source = "try ssh_mix(\"example.test\", \"print(1)\") catch $message, $error print($error.code); print($message) end";
    let output = successful(bare.run(false, &["-c", source]));
    assert!(
        output.lines().any(|line| line == "REMOTE_MIX_CONFIG"),
        "{output}"
    );
    assert!(output.contains(paths::env::REMOTE_EXE), "{output}");
    assert!(output.contains("mix_bin"), "{output}");
    assert!(
        output.contains(bare.prefix.join("env.mx").to_str().unwrap()),
        "{output}"
    );
}

#[test]
fn local_manual_and_apropos_keep_the_flat_cache_without_a_registry() {
    let bare = Bare::new();
    let cache = bare.home.join(".cache/mix/man");
    fs::create_dir_all(&cache).unwrap();
    fs::write(
        cache.join("bare-cache-only.md"),
        "# BareCachedHeading\n\nCached manual witness.\n",
    )
    .unwrap();
    let child = bare
        .command(false)
        .env(paths::env::MAN_SOURCE, "local")
        .args(["man", "bare-cache-only"])
        .spawn()
        .unwrap();
    let page = successful(bounded_output(child, Duration::from_secs(10)));
    assert!(page.contains("Cached manual witness."), "{page}");
    let search = successful(bare.run(false, &["apropos", "BareCachedHeading"]));
    assert!(search.contains("BareCachedHeading"), "{search}");
}

#[test]
fn binary_alone_starts_a_real_repl_evaluates_and_exits() {
    let bare = Bare::new();
    let mut master = -1;
    let mut slave = -1;
    assert_eq!(
        unsafe {
            libc::openpty(
                &mut master,
                &mut slave,
                std::ptr::null_mut(),
                std::ptr::null(),
                std::ptr::null(),
            )
        },
        0
    );
    let mut master = unsafe { File::from_raw_fd(master) };
    let slave = unsafe { File::from_raw_fd(slave) };
    let fd = slave.as_raw_fd();
    let mut command = bare.command(false);
    command
        .stdin(slave.try_clone().unwrap())
        .stdout(slave.try_clone().unwrap())
        .stderr(slave.try_clone().unwrap());
    unsafe {
        command.pre_exec(move || {
            if libc::setsid() < 0 || libc::ioctl(fd, libc::TIOCSCTTY, 0) < 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
        let flags = libc::fcntl(master.as_raw_fd(), libc::F_GETFL);
        assert_eq!(
            libc::fcntl(master.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK),
            0
        );
    }
    let mut child = command.spawn().unwrap();
    drop(command);
    drop(slave);
    let started = Instant::now();
    let mut transcript = Vec::new();
    let mut phase = 0;
    loop {
        let mut buffer = [0; 4096];
        match master.read(&mut buffer) {
            Ok(n) => transcript.extend_from_slice(&buffer[..n]),
            Err(e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.raw_os_error() == Some(libc::EIO) => {}
            Err(e) => panic!("PTY read: {e}"),
        }
        // The editor clears buffered input between reads. Act like a user:
        // submit one line only after each prompt has entered bracketed mode.
        let prompts = transcript.windows(8).filter(|bytes| *bytes == b"\x1b[?2004h").count();
        if phase == 0 && prompts >= 1 {
            master.write_all(b"print(\"BARE_\" .. \"REPL_OK\")\n").unwrap();
            phase = 1;
        } else if phase == 1 && prompts >= 2 {
            master.write_all(b"exit\n").unwrap();
            phase = 2;
        }
        if let Some(status) = child.try_wait().unwrap() {
            assert!(
                status.success(),
                "{status:?}: {}",
                String::from_utf8_lossy(&transcript)
            );
            break;
        }
        if started.elapsed() > Duration::from_secs(10) {
            let _ = child.kill();
            let _ = child.wait();
            panic!(
                "bare REPL timed out: {}",
                String::from_utf8_lossy(&transcript)
            );
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(
        String::from_utf8_lossy(&transcript).contains("BARE_REPL_OK"),
        "{}",
        String::from_utf8_lossy(&transcript)
    );
}

#[test]
fn binary_alone_keeps_the_fifteen_second_sigterm_backstop() {
    let bare = Bare::new();
    let fifo = bare.home.join("blocked.fifo");
    let path = std::ffi::CString::new(fifo.to_str().unwrap()).unwrap();
    assert_eq!(unsafe { libc::mkfifo(path.as_ptr(), 0o600) }, 0);
    let ready = bare.home.join("ready");
    let script = bare.home.join("blocked.mix");
    fs::write(
        &script,
        format!(
            "write_file({:?}, \"ready\")\nread_file_bytes({:?})\n",
            ready.to_str().unwrap(),
            fifo.to_str().unwrap()
        ),
    )
    .unwrap();
    let mut child = bare.command(false).arg(&script).spawn().unwrap();
    let startup = Instant::now();
    while !ready.exists() {
        assert!(
            child.try_wait().unwrap().is_none(),
            "bare script exited before blocking"
        );
        if startup.elapsed() > Duration::from_secs(10) {
            let _ = child.kill();
            let _ = child.wait();
            panic!("bare script did not reach FIFO");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    // The ready write precedes the blocking open; allow that final call to enter.
    std::thread::sleep(Duration::from_millis(100));
    let signalled = Instant::now();
    assert_eq!(unsafe { libc::kill(child.id() as i32, libc::SIGTERM) }, 0);
    let output = bounded_output(child, Duration::from_secs(18));
    assert_eq!(output.status.code(), Some(128 + libc::SIGTERM));
    assert!(
        signalled.elapsed() >= Duration::from_secs(14),
        "the 15s default was shortened"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("SIGTERM not honoured"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
