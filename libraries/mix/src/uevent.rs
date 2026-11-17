//! Linux uevent datagrams: inspect properties, retain the original wire bytes.
use std::collections::BTreeMap;

pub(crate) fn properties(
    bytes: &[u8],
    group: u32,
) -> Result<BTreeMap<String, String>, &'static str> {
    let payload = if group == 2 {
        if bytes.len() < 40
            || &bytes[..8] != b"libudev\0"
            || bytes[8..12] != 0xfeedcafeu32.to_be_bytes()
        {
            return Err("invalid libudev header");
        }
        let word =
            |offset| u32::from_ne_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
        let start = word(16);
        let end = start.checked_add(word(20)).ok_or("properties overflow")?;
        if word(12) < 40 || start < word(12) || end > bytes.len() {
            return Err("invalid properties range");
        }
        &bytes[start..end]
    } else {
        let start = bytes
            .iter()
            .position(|b| *b == 0)
            .ok_or("missing kernel prefix")?
            + 1;
        &bytes[start..]
    };
    if payload.last() != Some(&0) {
        return Err("unterminated properties");
    }
    let mut props = BTreeMap::new();
    for field in payload.split(|b| *b == 0).filter(|s| !s.is_empty()) {
        let field = std::str::from_utf8(field).map_err(|_| "non UTF-8 property")?;
        let (key, value) = field.split_once('=').ok_or("invalid property")?;
        if key.is_empty() || props.insert(key.to_owned(), value.to_owned()).is_some() {
            return Err("empty or duplicate property");
        }
    }
    if !["ACTION", "DEVPATH", "SUBSYSTEM", "SEQNUM"]
        .iter()
        .all(|k| props.contains_key(*k))
    {
        return Err("missing event property");
    }
    Ok(props)
}

pub(crate) fn matches(props: &BTreeMap<String, String>, subsystems: &[String]) -> bool {
    subsystems.is_empty()
        || props
            .get("SUBSYSTEM")
            .is_some_and(|s| subsystems.contains(s))
}

#[cfg(target_os = "linux")]
pub(crate) mod linux {
    use super::*;
    use crate::error::MixResult;
    use crate::native_events::{Queue, refusal};
    use std::{
        io,
        os::fd::{AsRawFd, FromRawFd, OwnedFd},
        sync::Arc,
        thread,
    };

    pub(crate) fn socket(group: u32) -> io::Result<OwnedFd> {
        // SAFETY: all syscall pointers refer to correctly sized live objects.
        let fd = unsafe {
            libc::socket(
                libc::AF_NETLINK,
                libc::SOCK_RAW | libc::SOCK_CLOEXEC | libc::SOCK_NONBLOCK,
                libc::NETLINK_KOBJECT_UEVENT,
            )
        };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        let fd = unsafe { OwnedFd::from_raw_fd(fd) };
        let one: libc::c_int = 1;
        if unsafe {
            libc::setsockopt(
                fd.as_raw_fd(),
                libc::SOL_SOCKET,
                libc::SO_PASSCRED,
                (&one as *const libc::c_int).cast(),
                size_of_val(&one) as _,
            )
        } < 0
        {
            return Err(io::Error::last_os_error());
        }
        let mut addr: libc::sockaddr_nl = unsafe { std::mem::zeroed() };
        addr.nl_family = libc::AF_NETLINK as _;
        addr.nl_groups = group;
        if unsafe {
            libc::bind(
                fd.as_raw_fd(),
                (&addr as *const libc::sockaddr_nl).cast(),
                size_of_val(&addr) as _,
            )
        } < 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(fd)
    }

    pub(crate) fn sender(path: String) -> crate::error::MixResult<OwnedFd> {
        if unsafe { libc::geteuid() } != 0 {
            return Err(refusal(
                "UEVENT_NETNS",
                "sender must run as uid 0 with CAP_SYS_ADMIN and CAP_NET_ADMIN",
            ));
        }
        // Never setns on an evaluator or runtime thread. The short-lived worker
        // exits in the target namespace, so no restore can fail or contaminate it.
        thread::Builder::new()
            .name("mix-uevent-netns".into())
            .spawn(move || {
                let ns = std::fs::File::open(path)?;
                if unsafe { libc::setns(ns.as_raw_fd(), libc::CLONE_NEWNET) } < 0 {
                    return Err(io::Error::last_os_error());
                }
                socket(0)
            })
            .map_err(|e| refusal("UEVENT_NETNS", e.to_string()))?
            .join()
            .map_err(|_| refusal("UEVENT_NETNS", "namespace thread panicked"))?
            .map_err(|e| {
                refusal(
                    "UEVENT_NETNS",
                    format!("setns/socket requires root with CAP_SYS_ADMIN and CAP_NET_ADMIN: {e}"),
                )
            })
    }

    pub(crate) fn send(fd: &OwnedFd, group: u32, bytes: &[u8]) -> MixResult<()> {
        if group != 1 && group != 2 {
            return Err(refusal("UEVENT_GROUP", "group must be 1 or 2"));
        }
        let mut addr: libc::sockaddr_nl = unsafe { std::mem::zeroed() };
        addr.nl_family = libc::AF_NETLINK as _;
        addr.nl_groups = group;
        let n = unsafe {
            libc::sendto(
                fd.as_raw_fd(),
                bytes.as_ptr().cast(),
                bytes.len(),
                0,
                (&addr as *const libc::sockaddr_nl).cast(),
                size_of_val(&addr) as _,
            )
        };
        if n < 0 {
            return Err(refusal(
                "UEVENT_SEND",
                io::Error::last_os_error().to_string(),
            ));
        }
        if n as usize != bytes.len() {
            return Err(refusal("UEVENT_SEND", "short datagram send"));
        }
        Ok(())
    }

    pub(crate) fn receive(fd: &OwnedFd, bytes: &mut [u8]) -> io::Result<(usize, u32)> {
        // usize storage provides cmsghdr alignment. Never trust data without
        // exactly one SCM_CREDENTIALS, and reject both data/control truncation.
        let mut control = [0usize; 16];
        let mut addr: libc::sockaddr_nl = unsafe { std::mem::zeroed() };
        let mut iov = libc::iovec {
            iov_base: bytes.as_mut_ptr().cast(),
            iov_len: bytes.len(),
        };
        let mut msg: libc::msghdr = unsafe { std::mem::zeroed() };
        msg.msg_name = (&mut addr as *mut libc::sockaddr_nl).cast();
        msg.msg_namelen = size_of_val(&addr) as _;
        msg.msg_iov = &mut iov;
        msg.msg_iovlen = 1;
        msg.msg_control = control.as_mut_ptr().cast();
        msg.msg_controllen = size_of_val(&control);
        let n = unsafe { libc::recvmsg(fd.as_raw_fd(), &mut msg, 0) };
        if n < 0 {
            return Err(io::Error::last_os_error());
        }
        if msg.msg_flags & (libc::MSG_TRUNC | libc::MSG_CTRUNC) != 0 {
            return Err(io::Error::other("truncated datagram/credentials"));
        }
        let mut uid = None;
        unsafe {
            let mut c = libc::CMSG_FIRSTHDR(&msg);
            while !c.is_null() {
                if (*c).cmsg_level == libc::SOL_SOCKET && (*c).cmsg_type == libc::SCM_CREDENTIALS {
                    if uid.is_some()
                        || (*c).cmsg_len != libc::CMSG_LEN(size_of::<libc::ucred>() as _) as usize
                    {
                        return Err(io::Error::other("invalid credentials"));
                    }
                    uid = Some(
                        std::ptr::read_unaligned(libc::CMSG_DATA(c).cast::<libc::ucred>()).uid,
                    );
                }
                c = libc::CMSG_NXTHDR(&msg, c);
            }
        }
        let uid = uid.ok_or_else(|| io::Error::other("missing SCM_CREDENTIALS"))?;
        if uid != 0 {
            return Err(io::Error::other("sender uid is not root"));
        }
        Ok((n as usize, uid))
    }

    pub(crate) struct Watch {
        cancel: std::os::unix::net::UnixStream,
        worker: Option<thread::JoinHandle<()>>,
    }
    impl Watch {
        pub(crate) fn new(
            queue: Arc<Queue>,
            handle: String,
            group: u32,
            subsystems: Vec<String>,
        ) -> MixResult<Self> {
            let fd = socket(group).map_err(|e| refusal("UEVENT_WATCH", e.to_string()))?;
            let (cancel, wake) = std::os::unix::net::UnixStream::pair()
                .map_err(|e| refusal("UEVENT_WATCH", e.to_string()))?;
            let worker = thread::Builder::new().name("mix-uevent".into()).spawn(move || {
                let mut bytes = vec![0u8; 256 * 1024];
                let mut serial = 0u64;
                loop {
                    let mut fds = [libc::pollfd {fd: fd.as_raw_fd(), events: libc::POLLIN, revents: 0}, libc::pollfd {fd: wake.as_raw_fd(), events: libc::POLLIN, revents: 0}];
                    let rc = unsafe { libc::poll(fds.as_mut_ptr(), 2, -1) };
                    if rc < 0 {
                        if io::Error::last_os_error().kind() == io::ErrorKind::Interrupted { continue; }
                        queue.source_closed(&handle, serde_json::json!({"error":"poll failed"})); break;
                    }
                    if fds[1].revents != 0 { break; }
                    match receive(&fd, &mut bytes) {
                        Ok((n, uid)) => {
                            let body = match properties(&bytes[..n], group) {
                                Ok(props) if matches(&props, &subsystems) => serde_json::json!({"properties":props,"uid":uid,"raw":bytes[..n]}),
                                Ok(_) => continue,
                                Err(reason) => serde_json::json!({"drop":reason}),
                            };
                            queue.source(&handle, vec![(format!("{serial:020}"), body)], false);
                            serial = serial.wrapping_add(1);
                        }
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock || e.kind() == io::ErrorKind::Interrupted => {},
                        Err(e) => {
                            queue.source(&handle, vec![(format!("{serial:020}"), serde_json::json!({"drop":e.to_string()}))], e.raw_os_error() == Some(libc::ENOBUFS));
                            serial = serial.wrapping_add(1);
                            if fds[0].revents & (libc::POLLERR | libc::POLLHUP | libc::POLLNVAL) != 0 && e.raw_os_error() != Some(libc::ENOBUFS) {
                                queue.source_closed(&handle, serde_json::json!({"error":e.to_string()})); break;
                            }
                        }
                    }
                }
            }).map_err(|e| refusal("UEVENT_WATCH", e.to_string()))?;
            Ok(Self {
                cancel,
                worker: Some(worker),
            })
        }
    }
    impl Drop for Watch {
        fn drop(&mut self) {
            let _ = self.cancel.shutdown(std::net::Shutdown::Write);
            if let Some(worker) = self.worker.take() {
                let _ = worker.join();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Vec<u8> {
        let props = b"ACTION=change\0DEVPATH=/devices/example/input/input0/event3\0SUBSYSTEM=input\0SEQNUM=42\0DEVNAME=/dev/input/event3\0";
        let mut wire = b"libudev\0".to_vec();
        wire.extend(0xfeedcafeu32.to_be_bytes());
        for n in [40u32, 40, props.len() as u32, 0, 0, 0, 0] {
            wire.extend(n.to_ne_bytes());
        }
        wire.extend(props);
        wire
    }
    #[test]
    fn fixture_properties_and_filter_do_not_modify_wire() {
        let wire = fixture();
        let before = wire.clone();
        let props = properties(&wire, 2).unwrap();
        assert_eq!(props["DEVNAME"], "/dev/input/event3");
        assert!(matches(&props, &["input".into()]));
        assert!(!matches(&props, &["sound".into()]));
        assert!(matches(&props, &[]));
        assert_eq!(wire, before);
    }
    #[test]
    fn rejects_bad_ranges_truncation_and_duplicate_properties() {
        let wire = fixture();
        for end in 0..40 {
            assert!(properties(&wire[..end], 2).is_err());
        }
        let mut bad = wire.clone();
        bad[16..20].copy_from_slice(&u32::MAX.to_ne_bytes());
        assert!(properties(&bad, 2).is_err());
        assert!(properties(&wire[..wire.len() - 1], 2).is_err());
        assert!(properties(b"change@/devices/example\0ACTION=change\0ACTION=add\0", 1).is_err());
    }
    #[test]
    fn kernel_fixture() {
        let props = properties(b"add@/devices/example\0ACTION=add\0DEVPATH=/devices/example\0SUBSYSTEM=hidraw\0SEQNUM=1\0",1).unwrap();
        assert_eq!(props["SUBSYSTEM"], "hidraw");
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[ignore = "root-only discovery; creates a temporary network namespace"]
    fn namespace_discovery() {
        use std::{
            os::fd::AsRawFd,
            process::{Command, Stdio},
            time::Duration,
        };
        assert_eq!(unsafe { libc::geteuid() }, 0, "root required");
        let listener = linux::socket(2).unwrap();
        let device = std::fs::read_dir("/sys/class/input")
            .unwrap()
            .filter_map(Result::ok)
            .find(|e| e.file_name().to_string_lossy().starts_with("event"))
            .map(|e| e.path())
            .expect("worker input device for real oracle");
        let trigger = Command::new("udevadm")
            .args(["trigger", "--action=change"])
            .arg(&device)
            .output()
            .unwrap();
        eprintln!(
            "oracle trigger: {:?}: {}",
            trigger.status,
            String::from_utf8_lossy(&trigger.stderr)
        );
        assert!(trigger.status.success(), "cannot capture real oracle");
        let mut poll = libc::pollfd {
            fd: listener.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        assert!(
            unsafe { libc::poll(&mut poll, 1, 3000) } > 0,
            "no real host udev event"
        );
        let mut wire = vec![0; 256 * 1024];
        let (n, uid) = linux::receive(&listener, &mut wire).unwrap();
        wire.truncate(n);
        let props = properties(&wire, 2).unwrap();
        eprintln!(
            "real oracle accepted: uid={uid} action={} subsystem={} bytes={n}",
            props["ACTION"], props["SUBSYSTEM"]
        );
        let mut ns = Command::new("unshare")
            .args(["-n", "sleep", "30"])
            .spawn()
            .unwrap();
        // This is a bounded discovery probe; the child owns its ephemeral ns.
        std::thread::sleep(Duration::from_millis(300));
        let path = format!("/proc/{}/ns/net", ns.id());
        let sender = linux::sender(path).unwrap();
        let monitor = Command::new("timeout")
            .env("SYSTEMD_LOG_LEVEL", "debug")
            .args([
                "3",
                "nsenter",
                "-t",
                &ns.id().to_string(),
                "-n",
                "udevadm",
                "monitor",
                "--udev",
                "--property",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        std::thread::sleep(Duration::from_millis(300));
        let sent = linux::send(&sender, 2, &wire);
        let output = monitor.wait_with_output().unwrap();
        let _ = ns.kill();
        let _ = ns.wait();
        eprintln!(
            "namespace send: {sent:?}; monitor stdout: {}; stderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        sent.unwrap();
        assert!(
            String::from_utf8_lossy(&output.stdout)
                .contains(&format!("SEQNUM={}", props["SEQNUM"])),
            "libudev must accept copied datagram"
        );
    }
}
