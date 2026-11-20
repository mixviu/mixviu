//! The same connect_to_env entry point used by the OSD layer host.
use std::os::fd::AsRawFd;
use std::os::unix::{net::UnixStream, process::CommandExt};
use std::process::Command;

#[test]
fn osd_connection_does_not_escape_to_children() {
    if std::env::var_os("WAYLAND_SOCKET").is_some() {
        let connection = wayland_client::Connection::connect_to_env().unwrap();
        assert!(std::env::var_os("WAYLAND_SOCKET").is_none());
        // Check all descriptors in the descendant, including backend duplicates.
        let socket = std::fs::read_link("/proc/self/fd/200").unwrap();
        let status = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "descendant_socket_probe", "--nocapture"])
            .env("WAYLAND_DISPLAY", socket)
            .status().unwrap();
        assert!(status.success());
        drop(connection);
        return;
    }
    let (_server, client) = UnixStream::pair().unwrap();
    let fd = client.as_raw_fd();
    let mut command = Command::new(std::env::current_exe().unwrap());
    command.args(["--exact", "osd_connection_does_not_escape_to_children", "--nocapture"])
        .env("WAYLAND_SOCKET", "200").env_remove("WAYLAND_DISPLAY");
    // SAFETY: only async-signal-safe descriptor syscalls before exec.
    unsafe { command.pre_exec(move || {
        if libc::dup2(fd, 200) < 0 || libc::fcntl(200, libc::F_SETFD, 0) < 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(())
    }); }
    assert!(command.status().unwrap().success());
}

#[test]
fn descendant_socket_probe() {
    let Ok(socket) = std::env::var("WAYLAND_DISPLAY") else { return };
    if !socket.starts_with("socket:[") { return; }
    assert!(std::env::var_os("WAYLAND_SOCKET").is_none());
    for entry in std::fs::read_dir("/proc/self/fd").unwrap() {
        if let Ok(target) = std::fs::read_link(entry.unwrap().path()) {
            assert_ne!(target.to_string_lossy(), socket, "OSD socket leaked to exec child");
        }
    }
}
