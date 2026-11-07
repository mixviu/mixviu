// SPDX-License-Identifier: MIT OR Apache-2.0
//! Bounded VT witnesses run against the same grid/parser used by real PTYs.
use super::*;

fn row(t: &Terminal, n: usize) -> String {
    let s = t.screen(false);
    s.cells[n * s.cols..(n + 1) * s.cols].iter().map(|c| c.c).collect()
}

#[test]
fn cursor_address_erase_insert_and_alternate_screen() {
    let t = Terminal::from_test_vt(12, 4, b"main\x1b[2;3Habc\x1b[2D\x1b[@X");
    assert_eq!(&row(&t, 0)[..4], "main");
    assert_eq!(&row(&t, 1)[..6], "  aXbc");
    let mut parser = rio_vt::performer::handler::Processor::default();
    parser.advance(&mut *t.grid.lock(), b"\x1b[?1049halt\x1b[?1049l");
    assert!(!t.alternate_screen());
    assert_eq!(&row(&t, 0)[..4], "main");
    parser.advance(&mut *t.grid.lock(), b"\x1b[2J\x1b[H");
    assert!(t.screen(false).cells.iter().all(|c| c.c == ' '));
}

#[test]
fn parser_retains_every_utf8_and_escape_chunk_boundary() {
    let bytes = "aé中e\u{301}\r\n\x1b[38;2;12;34;56mcolour\x1b[0m\x1b[2;8H!".as_bytes();
    let expected = Terminal::from_test_vt(20, 4, bytes).screen(false);
    for split in 0..=bytes.len() {
        let t = Terminal::from_test_vt(20, 4, b"");
        let mut parser = rio_vt::performer::handler::Processor::default();
        parser.advance(&mut *t.grid.lock(), &bytes[..split]);
        parser.advance(&mut *t.grid.lock(), &bytes[split..]);
        let actual = t.screen(false);
        assert_eq!(actual.cursor, expected.cursor, "split {split}");
        for (a, b) in actual.cells.iter().zip(&expected.cells) {
            assert_eq!((a.c, a.width, a.fg, a.bg, a.bold), (b.c, b.width, b.fg, b.bg, b.bold), "split {split}");
            assert_eq!(actual.clusters.get(a.extra), expected.clusters.get(b.extra), "split {split}");
        }
    }
}

#[test]
fn scrollback_pages_and_selection_use_the_native_grid() {
    let t = Terminal::from_test_vt(12, 3, b"one\r\ntwo\r\nthree\r\nfour\r\nfive");
    t.scroll_view(ScrollRequest::Top);
    assert_eq!(&row(&t, 0)[..3], "one");
    t.selection_start(0, 0, SelectionSide::Left, SelectionType::Simple);
    t.selection_update(2, 0, SelectionSide::Right);
    assert_eq!(t.selection_text().as_deref(), Some("one"));
    t.selection_clear();
    t.scroll_view(ScrollRequest::Bottom);
    assert_eq!(t.display_offset(), 0);
    assert_eq!(&row(&t, 2)[..4], "five");
}

#[test]
fn real_mix_login_input_resize_exit_and_reap() {
    struct Unbound;
    impl crate::terminal::native_session::SessionIssuer for Unbound {
        fn prepare(&self, _: u64) -> Option<(crate::terminal::native_session::PaneSession, crate::terminal::session_fd::LaunchFd)> { None }
    }
    let home = tempfile::tempdir().unwrap();
    let settings = crate::terminal::config::Settings {
        config: crate::terminal::config::Config::default(),
        term: "xterm-256color".into(),
    };
    let mut t = Terminal::start_session_e2e(
        settings, &Unbound, "/opt/mixos/bin/mix", home.path().to_str().unwrap().into(),
        vec![("HOME".into(), home.path().to_str().unwrap().into()),
             ("MIXOS_BUS_SOCKET".into(), home.path().join("absent.sock").to_str().unwrap().into())],
    ).unwrap();
    let pid = t.pid;
    t.resize(120, 40, 1200, 800);
    t.listener.type_text("print(\"VT_\" .. \"WIDGET_PTY_OK\")\n").unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while !t.snapshot().contains("VT_WIDGET_PTY_OK") {
        assert!(Instant::now() < deadline, "Mix PTY did not produce output: {}", t.snapshot());
        std::thread::sleep(Duration::from_millis(10));
    }
    let s = t.screen(false);
    assert_eq!((s.cols, s.rows), (120, 40));
    t.listener.type_text("exit(0)\n").unwrap();
    while !t.listener.quit.load(Ordering::Acquire) {
        assert!(Instant::now() < deadline, "Mix child did not exit");
        std::thread::sleep(Duration::from_millis(10));
    }
    t.shutdown();
    assert_eq!(unsafe { libc::waitpid(pid, std::ptr::null_mut(), libc::WNOHANG) }, -1);
    assert_eq!(io::Error::last_os_error().raw_os_error(), Some(libc::ECHILD));
}
