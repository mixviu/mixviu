// SPDX-License-Identifier: MIT OR Apache-2.0
//! Process-level oracles. Unsafe below is fixture setup in isolated processes,
//! never pre_exec and never a change to the test runner's signal state.
#![cfg(target_os = "linux")]
use std::io::{Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::process::{CommandExt, ExitStatusExt};
use std::os::unix::fs::PermissionsExt;
use std::process::{Command, Output, Stdio};

const MIX: &str = env!("CARGO_BIN_EXE_mix");

fn run(source: &str) -> Output {
    let dir = tempfile::tempdir().unwrap();
    let script = dir.path().join("probe.mix");
    std::fs::write(&script, source).unwrap();
    Command::new(MIX).arg("--no-prelude").arg(script)
        .env("MIX_STATS", "off").output().unwrap()
}

fn text(out: &Output) -> String {
    assert!(out.status.success(), "{out:?}");
    String::from_utf8(out.stdout.clone()).unwrap()
}

fn fixture_command(mode: &str) -> Command {
    let mut cmd = Command::new(std::env::current_exe().unwrap());
    cmd.args(["--exact", "fixture", "--nocapture"])
        .env("EXEC_FIXTURE", mode).env("MIX_STATS", "off");
    cmd
}

#[test]
fn fixture() {
    let Ok(mode) = std::env::var("EXEC_FIXTURE") else { return };
    if mode == "target" {
        let stat = std::fs::read_to_string("/proc/self/stat").unwrap();
        let fields: Vec<_> = stat.rsplit_once(") ").unwrap().1.split_whitespace().collect();
        println!("IDENTITY={} PGID={}", std::process::id(), fields[2]);
        println!("CWD={}", std::env::current_dir().unwrap().display());
        let entries: Vec<_> = std::fs::read_dir("/proc/self/fd").unwrap()
            .map(|e| e.unwrap().path()).collect();
        let mut fds: Vec<_> = entries.iter().filter(|p| std::fs::read_link(p).is_ok())
            .map(|p| p.file_name().unwrap().to_str().unwrap().parse::<i32>().unwrap()).collect();
        fds.sort();
        println!("FDS={fds:?}");
        eprintln!("TARGET_STDERR");
        let mut input = String::new();
        std::io::stdin().read_to_string(&mut input).unwrap();
        println!("INPUT={input}");
        std::process::exit(37);
    }
    // This helper is already a separate process. Establish a parent's ignored
    // signal and blocked mask before replacing it with the real mix binary.
    unsafe {
        assert_ne!(libc::signal(libc::SIGTTOU, libc::SIG_IGN), libc::SIG_ERR);
        let mut mask = std::mem::zeroed();
        assert_eq!(libc::sigemptyset(&mut mask), 0);
        assert_eq!(libc::sigaddset(&mut mask, libc::SIGUSR1), 0);
        assert_eq!(libc::sigaddset(&mut mask, libc::SIGTERM), 0);
        assert_eq!(libc::pthread_sigmask(libc::SIG_SETMASK, &mask, std::ptr::null_mut()), 0);
        if mode == "planted" {
            let file = std::fs::File::open("/dev/null").unwrap();
            assert_eq!(libc::dup2(file.as_raw_fd(), 200), 200);
        }
    }
    let source = std::env::var("EXEC_SOURCE").unwrap();
    let dir = tempfile::tempdir().unwrap();
    let script = dir.path().join("fixture.mix");
    std::fs::write(&script, source).unwrap();
    panic!("mix exec failed: {}", Command::new(MIX).arg("--no-prelude").arg(script).exec());
}

#[test]
fn replacement_preserves_identity_stdio_exit_and_cwd_without_owned_fds() {
    let dir = tempfile::tempdir().unwrap();
    let exe = std::env::current_exe().unwrap();
    for planted in [false, true] {
        let source = format!(r#"$lock = flock({lock:?})
$db = sqlopen(":memory:")
$socket = tcp_listen("127.0.0.1", 0)
print("BEFORE")
exec_argv([{exe:?}, "--exact", "fixture", "--nocapture"], {{cwd: {cwd:?}, env: {{EXEC_FIXTURE: "target"}}}})
die("RETURNED")"#, lock=dir.path().join("lock"), cwd=dir.path());
        let mut cmd = if planted { fixture_command("planted") } else {
            let mut c = Command::new(MIX);
            c.args(["--no-prelude", "-c", &source]); c
        };
        cmd.env("EXEC_SOURCE", &source).env("MIX_STATS", "off").process_group(0)
            .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut child = cmd.spawn().unwrap();
        let pid = child.id();
        child.stdin.take().unwrap().write_all(b"inherited input").unwrap();
        let out = child.wait_with_output().unwrap();
        assert_eq!(out.status.code(), Some(37), "{out:?}");
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(stdout.contains(&format!("IDENTITY={pid} PGID={pid}")), "{stdout}");
        assert!(stdout.contains("BEFORE\n"), "{stdout}");
        assert!(stdout.contains("INPUT=inherited input"), "{stdout}");
        assert!(stdout.contains(&format!("CWD={}", dir.path().display())), "{stdout}");
        let expected = if planted { "FDS=[0, 1, 2, 200]" } else { "FDS=[0, 1, 2]" };
        assert!(stdout.contains(expected), "{stdout}");
        assert!(String::from_utf8_lossy(&out.stderr).contains("TARGET_STDERR"));
    }
}

fn sigfield(status: &str, name: &str) -> u64 {
    u64::from_str_radix(status.lines().find_map(|l| l.strip_prefix(name)).unwrap().trim(), 16).unwrap()
}

#[test]
fn target_signal_state_comes_from_real_exec_not_rust_startup() {
    let out = fixture_command("signals").env("EXEC_SOURCE",
        "exec_argv([\"/usr/bin/cat\", \"/proc/self/status\"])").output().unwrap();
    let status = text(&out);
    let ignored = sigfield(&status, "SigIgn:");
    assert_ne!(ignored & (1 << (libc::SIGTTOU - 1)), 0);
    assert_eq!(ignored & (1 << (libc::SIGPIPE - 1)), 0);
    assert_eq!(sigfield(&status, "SigBlk:"), 0);
    assert_eq!(sigfield(&status, "SigCgt:"), 0);
}

#[test]
fn target_signal_and_path_env_options() {
    let out = run("exec_argv([\"/usr/bin/kill\", \"-TERM\", to_string(pid())])");
    assert_eq!(out.status.signal(), Some(libc::SIGTERM), "{out:?}");
    let out = run("exec_argv([\"env\"], {clear_env: true, env: {PATH: \"/usr/bin\", SENTINEL: \"kept\"}})");
    let stdout = text(&out);
    assert!(stdout.contains("SENTINEL=kept\n"), "{stdout}");
    assert!(!stdout.contains("MIX_STATS="));
}

#[test]
fn executable_text_without_shebang_never_runs_a_shell() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("plain-text");
    std::fs::write(&target, "/usr/bin/true\n").unwrap();
    std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o700)).unwrap();
    let out = run(&format!("try; exec_argv([{target:?}]); catch $e; print($e); end; print(\"CONTINUED\")"));
    let stdout = text(&out);
    assert!(stdout.contains("ENOEXEC") && stdout.contains("CONTINUED"), "{stdout}");
}

#[test]
fn failures_restore_state_env_and_cwd_and_continue() {
    let dir = tempfile::tempdir().unwrap();
    let denied = dir.path().join("denied");
    std::fs::write(&denied, "not executable").unwrap();
    for (target, errno) in [("/no/such/exec_argv", "ENOENT"), (denied.to_str().unwrap(), "EACCES")] {
        let source = format!(r#"$before = read_file("/proc/self/status")
$cwd = cwd()
try
  exec_argv([{target:?}], {{cwd: {dir:?}, clear_env: true, env: {{EXEC_SENTINEL: "changed"}}}})
catch $e
  print($e)
end
print("CWD_SAME=" .. (cwd() == $cwd))
print("ENV_SAME=" .. (env("EXEC_SENTINEL") == nil))
write_file({before:?}, $before)
write_file({after:?}, read_file("/proc/self/status"))
print("CONTINUED")"#, dir=dir.path(), before=dir.path().join("before"), after=dir.path().join("after"));
        let out = fixture_command("signals").env("EXEC_SOURCE", source).output().unwrap();
        let stdout = text(&out);
        assert!(stdout.contains(errno) && stdout.contains("CONTINUED") && stdout.contains("CWD_SAME=true") && stdout.contains("ENV_SAME=true"), "{stdout}");
        let before = std::fs::read_to_string(dir.path().join("before")).unwrap();
        let after = std::fs::read_to_string(dir.path().join("after")).unwrap();
        for field in ["SigIgn:", "SigBlk:", "SigCgt:"] {
            assert_eq!(sigfield(&before, field), sigfield(&after, field), "{field}");
        }
        assert_ne!(sigfield(&after, "SigIgn:") & (1 << (libc::SIGPIPE - 1)), 0);
    }
}

#[test]
fn failed_exec_restores_epipe_not_sigpipe_death() {
    let dir = tempfile::tempdir().unwrap();
    let ready = dir.path().join("ready");
    let go = dir.path().join("go");
    let result = dir.path().join("result");
    let source = format!(r#"try
  exec_argv(["/no/such/exec_argv"])
catch $e
  write_file({ready:?}, "ready")
end
while not exists({go:?})
  sleep(0.01)
end
try
  write_file("/proc/self/fd/1", "closed pipe")
catch $e
  write_file({result:?}, to_string($e))
end"#);
    let mut child = Command::new(MIX).args(["--no-prelude", "-c", &source])
        .env("MIX_STATS", "off").stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while !ready.exists() {
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    drop(child.stdout.take());
    std::fs::write(go, "go").unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "{out:?}");
    let error = std::fs::read_to_string(result).unwrap();
    assert!(error.contains("Broken pipe") || error.contains("os error 32"), "{error}");
}

#[test]
fn refuses_managed_children_handlers_and_unknown_options() {
    for source in [
        "$p = spawn([\"/usr/bin/sleep\", \"30\"], {exit_event: true}); exec_argv([\"/usr/bin/true\"])",
        "$p = spawn([\"/usr/bin/sleep\", \"30\"], {die_with_parent: true}); exec_argv([\"/usr/bin/true\"])",
        "on timer.tick; exec_argv([\"/usr/bin/true\"]); end; $t = timer_once(0.01, \"exec\"); sleep(0.2)",
    ] {
        let out = run(source);
        assert!(String::from_utf8_lossy(&out.stderr).contains("exec_argv refused"), "{out:?}");
    }
    for opts in ["{timeout: 1}", "{stdout: \"null\"}", "{env: {BAD: []}}", "{cwd: 2}", "{clear_env: 1}"] {
        let out = run(&format!("try; exec_argv([\"/usr/bin/true\"], {opts}); catch $e; print($e); end; print(\"continued\")"));
        let stdout = text(&out);
        assert!(stdout.contains("exec_argv") && stdout.contains("continued"), "{stdout}");
    }
}

#[test]
fn serve_init_refuses_replacement() {
    let mut broker = test_broker::Broker::start();
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("node.conf.mix");
    std::fs::write(&config, format!("noded: {{ unix_socket: {:?} }}\n", broker.endpoint)).unwrap();
    let script = dir.path().join("refuse.mix");
    std::fs::write(&script, "exec_argv([\"/usr/bin/true\"])\non probe.ping\n reply(0, \"pong\")\nend\n").unwrap();
    let out = Command::new(MIX).arg("--no-prelude").arg("--serve").arg(script)
        .args(["--name", "exec-refusal"])
        .env("HOME", dir.path()).env("MIX_NODE_CONFIG", config)
        .env("MIX_ETC", dir.path()).env("MIX_STATS", "off")
        .env_remove("MIX_SESSION_FD").env_remove("MIX_NODED_URL").env_remove("MIX_BUS_SOCKET")
        .output().unwrap();
    assert!(!out.status.success(), "{out:?}");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("exec_argv refused: --serve"), "{stderr}");
    broker.stop();
}
