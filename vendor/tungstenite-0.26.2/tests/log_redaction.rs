// SPDX-License-Identifier: MIT OR Apache-2.0
// MixViu regression for the local payload-log redaction patch.
use std::io::{self, Cursor, Read, Write};
use std::sync::Mutex;
use tungstenite::{Message, WebSocket, protocol::{Role, CloseFrame, frame::{Frame, coding::{CloseCode, Data, OpCode}}}};

struct Capture(Mutex<Vec<String>>);
static LOGS: Capture = Capture(Mutex::new(Vec::new()));
impl log::Log for Capture {
    fn enabled(&self, _: &log::Metadata<'_>) -> bool { true }
    fn log(&self, record: &log::Record<'_>) {
        self.0.lock().unwrap().push(format!("{}", record.args()));
    }
    fn flush(&self) {}
}
#[derive(Default)]
struct Memory {
    input: Cursor<Vec<u8>>,
    output: Vec<u8>,
}
impl Read for Memory {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> { self.input.read(buf) }
}
impl Write for Memory {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> { self.output.write(buf) }
    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}

#[test]
fn transport_logs_keep_metadata_without_payloads_in_both_roles() {
    assert_eq!(log::STATIC_MAX_LEVEL, log::LevelFilter::Trace, "global log cap masks the regression");
    log::set_logger(&LOGS).unwrap();
    log::set_max_level(log::LevelFilter::Trace);
    const SECRET: &str = "FRAME-PRIVATE-KEY-AND-PROOF";
    for (sender, receiver) in [(Role::Client, Role::Server), (Role::Server, Role::Client)] {
        for message in [
            Message::text(SECRET),
            Message::binary(SECRET.as_bytes().to_vec()),
            Message::Ping(SECRET.as_bytes().to_vec().into()),
            Message::Pong(SECRET.as_bytes().to_vec().into()),
            Message::Frame(Frame::message(SECRET.as_bytes().to_vec(), OpCode::Data(Data::Binary), true)),
            Message::Close(Some(CloseFrame { code: CloseCode::Normal, reason: SECRET.into() })),
        ] {
            let mut tx = WebSocket::from_raw_socket(Memory::default(), sender, None);
            tx.send(message).unwrap();
            let memory = Memory { input: Cursor::new(tx.get_ref().output.clone()), output: Vec::new() };
            let mut rx = WebSocket::from_raw_socket(memory, receiver, None);
            let received = rx.read().unwrap();
            assert!(!received.is_empty());
        }
    }
    let output = LOGS.0.lock().unwrap().join("\n");
    let hex = SECRET.as_bytes().iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    assert!(!output.contains(SECRET), "{output}");
    assert!(!output.to_lowercase().contains(&hex), "{output}");
    for site in ["Sending frame:", "writing frame:", "received frame:", "Received message:", "Received close frame:", "Replying to close:"] {
        assert!(output.lines().any(|line| line.contains(site) && line.contains("opcode=") && line.contains("length=")), "missing metadata at {site}: {output}");
    }
}
