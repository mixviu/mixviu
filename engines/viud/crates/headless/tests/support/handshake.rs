// SPDX-License-Identifier: MIT OR Apache-2.0
//! Deadline applies to partial lines as well as silence before the first byte.
use std::{
    io::{self, Read},
    os::fd::AsRawFd,
    time::Instant,
};

pub fn read_ready(
    input: &mut (impl Read + AsRawFd),
    prefix: &str,
    deadline: Instant,
) -> io::Result<String> {
    let mut pending = Vec::new();
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                format!("waiting for {prefix:?}"),
            ));
        }
        let mut fd = libc::pollfd {
            fd: input.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        // SAFETY: one valid, exclusively read descriptor, bounded timeout.
        let ready = unsafe {
            libc::poll(
                &mut fd,
                1,
                remaining.as_millis().clamp(1, i32::MAX as u128) as i32,
            )
        };
        if ready < 0 {
            let error = io::Error::last_os_error();
            if error.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            return Err(error);
        }
        if ready == 0 {
            continue;
        }
        let mut bytes = [0; 1024];
        // A pipe/socket read returns the available bytes; unlike read_line,
        // this never waits for a missing newline after poll reports readiness.
        let len = match input.read(&mut bytes) {
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            result => result?,
        };
        if len == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                format!("EOF before {prefix:?}"),
            ));
        }
        pending.extend_from_slice(&bytes[..len]);
        while let Some(end) = pending.iter().position(|byte| *byte == b'\n') {
            let line = String::from_utf8(pending.drain(..=end).collect())
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
            if let Some(record) = line.strip_prefix(prefix) {
                return Ok(record.trim_end().to_owned());
            }
        }
        if pending.len() > 64 * 1024 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "handshake line exceeds 64 KiB",
            ));
        }
    }
}

#[test]
fn silent_and_partial_handshakes_have_deadlines() {
    use std::{io::Write, os::unix::net::UnixStream, time::Duration};
    for partial in [b"".as_slice(), b"READY incomplete"] {
        let (mut input, mut writer) = UnixStream::pair().unwrap();
        writer.write_all(partial).unwrap();
        let error = read_ready(
            &mut input,
            "READY ",
            Instant::now() + Duration::from_millis(50),
        )
        .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    }
}
