// SPDX-License-Identifier: MIT OR Apache-2.0
//! Bounded VT witnesses run against the same grid/parser used by real PTYs.
use super::*;

fn row(t: &Terminal, n: usize) -> String {
    let s = t.screen(false);
    s.cells[n * s.cols..(n + 1) * s.cols]
        .iter()
        // Rio stores untouched cells as NUL. Match gold's text snapshot and
        // raster convention while retaining exact raw-cell assertions below.
        .map(|c| if c.c == '\0' { ' ' } else { c.c })
        .collect()
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
    assert!(t.screen(false).cells.iter().all(|c| c.c == '\0'));
    assert!(!t.snapshot().contains('\0'));
    for n in 0..4 {
        assert_eq!(row(&t, n), " ".repeat(12));
    }
}

#[test]
fn scrolling_region_and_origin_mode_leave_outer_rows_untouched() {
    let t = Terminal::from_test_vt(
        8,
        5,
        b"1111\x1b[2;1H2222\x1b[3;1H3333\x1b[4;1H4444\x1b[5;1H5555\
          \x1b[2;4r\x1b[?6h\x1b[3;1HZ\n",
    );
    assert_eq!(row(&t, 0), "1111    ");
    assert_eq!(row(&t, 1), "3333    ");
    assert_eq!(row(&t, 2), "Z444    ");
    assert_eq!(row(&t, 3), "        ");
    assert_eq!(row(&t, 4), "5555    ");
    let mut parser = rio_vt::performer::handler::Processor::default();
    parser.advance(&mut *t.grid.lock(), b"\x1b[?6l\x1b[5;1HX");
    assert_eq!(row(&t, 4), "X555    ");
}

#[test]
fn character_delete_insert_and_erase_keep_columns() {
    let t = Terminal::from_test_vt(8, 2, b"abcdef\x1b[1;3H\x1b[2P");
    assert_eq!(row(&t, 0), "abef    ");
    let mut parser = rio_vt::performer::handler::Processor::default();
    parser.advance(&mut *t.grid.lock(), b"\x1b[2@XY");
    assert_eq!(row(&t, 0), "abXYef  ");
    parser.advance(&mut *t.grid.lock(), b"\x1b[2X");
    assert_eq!(row(&t, 0), "abXY    ");
}

#[test]
fn line_insert_delete_and_reverse_index_obey_region_edges() {
    let t = Terminal::from_test_vt(
        8,
        5,
        b"AAAA\x1b[2;1HBBBB\x1b[3;1HCCCC\x1b[4;1HDDDD\x1b[5;1HEEEE\
          \x1b[2;4r\x1b[2;1H\x1b[L",
    );
    assert_eq!(row(&t, 0), "AAAA    ");
    assert_eq!(row(&t, 1), "        ");
    assert_eq!(row(&t, 2), "BBBB    ");
    assert_eq!(row(&t, 3), "CCCC    ");
    assert_eq!(row(&t, 4), "EEEE    ");
    let mut parser = rio_vt::performer::handler::Processor::default();
    parser.advance(&mut *t.grid.lock(), b"\x1b[M");
    assert_eq!(row(&t, 1), "BBBB    ");
    assert_eq!(row(&t, 2), "CCCC    ");
    assert_eq!(row(&t, 3), "        ");
    parser.advance(&mut *t.grid.lock(), b"\x1bM");
    assert_eq!(row(&t, 1), "        ");
    assert_eq!(row(&t, 2), "BBBB    ");
    assert_eq!(row(&t, 3), "CCCC    ");
    assert_eq!(row(&t, 4), "EEEE    ");
}

#[test]
fn delayed_wrap_and_wrap_disabled_overwrite_the_last_column() {
    let t = Terminal::from_test_vt(8, 3, b"12345678");
    assert_eq!(row(&t, 0), "12345678");
    assert_eq!(row(&t, 1), "        ");
    let mut parser = rio_vt::performer::handler::Processor::default();
    parser.advance(&mut *t.grid.lock(), b"X");
    assert_eq!(row(&t, 1), "X       ");
    parser.advance(&mut *t.grid.lock(), b"\x1b[?7l\x1b[1;8Hyz");
    assert_eq!(row(&t, 0), "1234567z");
    assert_eq!(row(&t, 1), "X       ");
}

#[test]
fn dec_cursor_save_restore_and_explicit_tab_stops() {
    let t = Terminal::from_test_vt(24, 3, b"abc\x1b7\x1b[2;5HZZ\x1b8!");
    assert_eq!(&row(&t, 0)[..8], "abc!    ");
    assert_eq!(&row(&t, 1)[..8], "    ZZ  ");
    let mut parser = rio_vt::performer::handler::Processor::default();
    parser.advance(&mut *t.grid.lock(), b"\x1b[3;1HA\tB");
    // Rio retains HT in the first untouched skipped cell for text copying.
    // This is a raw grid witness: preserve that marker as well as the exact
    // destination column rather than treating it as a printed space.
    assert_eq!(&row(&t, 2)[..10], "A\t      B ");
    assert_eq!(t.screen(false).cells[2 * 24 + 8].c, 'B');
    parser.advance(&mut *t.grid.lock(), b"\x1b[3g\x1b[3;4H\x1bH\x1b[3;1H\tX");
    assert_eq!(&row(&t, 2)[..10], "A\t X    B ");
    assert_eq!(t.screen(false).cells[2 * 24 + 3].c, 'X');
}

#[test]
fn erase_line_modes_include_the_cursor_without_moving_it() {
    for (mode, expected) in [(0, "abc     "), (1, "    efgh"), (2, "        ")] {
        let t = Terminal::from_test_vt(8, 2, b"abcdefgh\x1b[1;4H");
        t.advance_test_vt(format!("\x1b[{mode}K").as_bytes());
        assert_eq!(row(&t, 0), expected, "EL {mode}");
        assert_eq!(t.screen(false).cursor, (3, 0), "EL {mode}");
        assert_eq!(row(&t, 1), "        ");
    }
}

#[test]
fn sgr_truecolour_inverse_bold_and_reset_reach_captured_cells() {
    let t = Terminal::from_test_vt(
        12,
        2,
        b"D\x1b[38;2;12;34;56;48;2;78;90;123mA\x1b[7mB\x1b[27;1mC\x1b[0mE",
    );
    let s = t.screen(false);
    assert_eq!(
        (s.cells[1].fg, s.cells[1].bg),
        ([12, 34, 56], [78, 90, 123])
    );
    assert_eq!(
        (s.cells[2].fg, s.cells[2].bg),
        ([78, 90, 123], [12, 34, 56])
    );
    // Gold brightens bold foregrounds by 40, even for truecolour, rather
    // than synthesising a heavier face. Keep that captured-cell contract.
    assert!(s.cells[3].bold);
    assert_eq!(
        (s.cells[3].fg, s.cells[3].bg),
        ([52, 74, 96], [78, 90, 123])
    );
    assert!(!s.cells[4].bold);
    assert_eq!(
        (s.cells[4].fg, s.cells[4].bg),
        (s.cells[0].fg, s.cells[0].bg)
    );
    assert_eq!(s.cursor, (5, 0));
}

#[test]
fn insert_mode_and_carriage_return_line_feed_remain_independent() {
    let t = Terminal::from_test_vt(8, 3, b"abcdef\x1b[1;3H\x1b[4hXY\x1b[4lZ");
    assert_eq!(row(&t, 0), "abXYZdef");
    t.advance_test_vt(b"\rQ\nR");
    assert_eq!(row(&t, 0), "QbXYZdef");
    assert_eq!(row(&t, 1), " R      ");
    assert_eq!(t.screen(false).cursor, (2, 1));
    t.advance_test_vt(b"\r\nS");
    assert_eq!(row(&t, 2), "S       ");
    assert_eq!(t.screen(false).cursor, (1, 2));
}

#[test]
fn erase_display_modes_preserve_cursor_and_include_its_cell() {
    for (mode, expected) in [
        (0, ["AAAAAAAA", "BBB     ", "        "]),
        (1, ["        ", "    BBBB", "CCCCCCCC"]),
        (2, ["        ", "        ", "        "]),
    ] {
        let t =
            Terminal::from_test_vt(8, 3, b"AAAAAAAA\x1b[2;1HBBBBBBBB\x1b[3;1HCCCCCCCC\x1b[2;4H");
        t.advance_test_vt(format!("\x1b[{mode}J").as_bytes());
        for (n, text) in expected.into_iter().enumerate() {
            assert_eq!(row(&t, n), text, "ED {mode}, row {n}");
        }
        assert_eq!(t.screen(false).cursor, (3, 1), "ED {mode}");
    }
}

#[test]
fn dec_special_graphics_returns_to_ascii_after_charset_reset() {
    let t = Terminal::from_test_vt(6, 3, b"\x1b(0lqk\r\nx x\r\nmqj\x1b(BA");
    assert_eq!(row(&t, 0), "┌─┐   ");
    assert_eq!(row(&t, 1), "│ │   ");
    assert_eq!(row(&t, 2), "└─┘A  ");
    assert_eq!(t.screen(false).cursor, (4, 2));
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
            assert_eq!(
                (a.c, a.width, a.fg, a.bg, a.bold),
                (b.c, b.width, b.fg, b.bg, b.bold),
                "split {split}"
            );
            assert_eq!(
                actual.clusters.get(a.extra),
                expected.clusters.get(b.extra),
                "split {split}"
            );
        }
    }
}

fn query_replies(receiver: &rio_vt::corcovado::channel::Receiver<rio_vt::event::Msg>) -> Vec<u8> {
    let mut bytes = Vec::new();
    while let Ok(message) = receiver.try_recv() {
        if let rio_vt::event::Msg::Input(reply) = message {
            bytes.extend_from_slice(&reply);
        }
    }
    bytes
}

#[test]
fn cursor_status_and_device_queries_survive_every_escape_chunk_boundary() {
    // Independent wire expectations for CPR, DSR and Gold's advertised DA1.
    // These replies go back to the program, never into the visible grid.
    let queries = b"\x1b[5;7H\x1b[6n\x1b[5n\x1b[c";
    let expected = b"\x1b[5;7R\x1b[0n\x1b[?62;4;6;22;52c";
    for split in 0..=queries.len() {
        let t = Terminal::from_test_vt(80, 24, b"untouched");
        let receiver = t.listener.test_input_receiver();
        let mut parser = rio_vt::performer::handler::Processor::default();
        parser.advance(&mut *t.grid.lock(), &queries[..split]);
        parser.advance(&mut *t.grid.lock(), &queries[split..]);
        assert_eq!(query_replies(&receiver), expected, "split {split}");
        assert_eq!(t.screen(false).cursor, (6, 4), "split {split}");
        assert_eq!(row(&t, 0), format!("untouched{}", " ".repeat(71)));
        for n in 1..24 {
            assert_eq!(row(&t, n), " ".repeat(80), "split {split}, row {n}");
        }
    }
}

#[test]
fn cursor_queries_follow_alternate_screen_entry_and_saved_main_cursor() {
    let t = Terminal::from_test_vt(80, 24, b"main\x1b[2;4H");
    let receiver = t.listener.test_input_receiver();
    t.advance_test_vt(b"\x1b[?1049h\x1b[12;30H\x1b[6n\x1b[?1049l\x1b[6n");
    assert_eq!(query_replies(&receiver), b"\x1b[12;30R\x1b[2;4R");
    assert!(!t.alternate_screen());
    assert_eq!(t.screen(false).cursor, (3, 1));
    assert_eq!(row(&t, 0), format!("main{}", " ".repeat(76)));
    assert_eq!(row(&t, 1), " ".repeat(80));
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
fn resize_reflows_a_logical_line_and_restores_its_cursor() {
    let t = Terminal::from_test_vt(10, 4, b"abcdefgh");
    t.resize(5, 4, 50, 40);
    // Gold's Rio reflow keeps the tail at the live viewport origin and
    // moves the leading wrapped segment into history. Inspect both parts,
    // rather than assuming a resize pins the logical line to screen row 0.
    assert_eq!(row(&t, 0), "fgh  ");
    assert_eq!(t.screen(false).cursor, (3, 0));
    t.scroll_view(ScrollRequest::Top);
    assert_eq!(t.display_offset(), 1);
    assert_eq!(row(&t, 0), "abcde");
    assert_eq!(row(&t, 1), "fgh  ");
    t.selection_start(0, 0, SelectionSide::Left, SelectionType::Simple);
    t.selection_update(2, 1, SelectionSide::Right);
    assert_eq!(t.selection_text().as_deref(), Some("abcdefgh"));
    t.selection_clear();
    t.scroll_view(ScrollRequest::Bottom);
    t.resize(10, 4, 100, 40);
    assert_eq!(row(&t, 0), "abcdefgh  ");
    assert_eq!(row(&t, 1), "          ");
    assert_eq!(t.screen(false).cursor, (8, 0));
}

#[test]
fn alternate_screen_resize_preserves_main_text_and_saved_cursor() {
    // Screen switching saves/restores the cursor but does not home it.
    let t = Terminal::from_test_vt(10, 4, b"main\x1b[?1049h\x1b[Halt");
    assert!(t.alternate_screen());
    t.resize(6, 4, 60, 40);
    assert_eq!(row(&t, 0), "alt   ");
    t.advance_test_vt(b"\x1b[?1049l!");
    assert!(!t.alternate_screen());
    assert_eq!(row(&t, 0), "main! ");
    assert_eq!(t.screen(false).cursor, (5, 0));
    assert_eq!(row(&t, 1), "      ");
}

#[test]
fn widening_restores_wrapped_text_at_each_cursor_column() {
    for width in 3..=10 {
        for count in 1..=20 {
            let text = "abcdefghijklmnopqrst"[..count].to_string();
            let t = Terminal::from_test_vt(24, 8, text.as_bytes());
            t.resize(width, 8, width * 10, 80);
            t.resize(24, 8, 240, 80);
            assert_eq!(
                row(&t, 0),
                format!("{text}{}", " ".repeat(24 - count)),
                "width {width}, count {count}"
            );
            assert_eq!(
                t.screen(false).cursor,
                (count, 0),
                "width {width}, count {count}"
            );
            assert_eq!(t.display_offset(), 0);
        }
    }
}

#[test]
fn real_mix_login_input_resize_exit_and_reap() {
    let Some(mix) = paths::env::os(paths::env::test::TEST_MIX_EXE) else {
        eprintln!(
            "skip real Mix PTY fixture: set {} to a staged interpreter",
            paths::env::test::TEST_MIX_EXE
        );
        return;
    };
    use std::os::unix::fs::PermissionsExt;
    let selected = std::path::Path::new(&mix);
    assert!(
        selected.is_file() && selected.metadata().unwrap().permissions().mode() & 0o111 != 0,
        "{} is not an executable file: {}",
        paths::env::test::TEST_MIX_EXE,
        selected.display()
    );
    struct Unbound;
    impl crate::terminal::native_session::SessionIssuer for Unbound {
        fn prepare(
            &self,
            _: u64,
        ) -> Option<(
            crate::terminal::native_session::PaneSession,
            crate::terminal::session_fd::LaunchFd,
        )> {
            None
        }
    }
    let home = tempfile::tempdir().unwrap();
    let settings = crate::terminal::config::Settings {
        config: crate::terminal::config::Config::default(),
        term: "xterm-256color".into(),
    };
    let mut t = Terminal::start_session_e2e(
        settings,
        &Unbound,
        mix.to_str()
            .expect("selected test interpreter path must be UTF-8"),
        home.path().to_str().unwrap().into(),
        vec![
            ("HOME".into(), home.path().to_str().unwrap().into()),
            (
                "MIX_BUS_SOCKET".into(),
                home.path().join("absent.sock").to_str().unwrap().into(),
            ),
        ],
    )
    .unwrap();
    let pid = t.pid;
    t.resize(120, 40, 1200, 800);
    t.listener
        .type_text("print(\"VT_\" .. \"WIDGET_PTY_OK\")\n")
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while !t.snapshot().contains("VT_WIDGET_PTY_OK") {
        assert!(
            Instant::now() < deadline,
            "Mix PTY did not produce output: {}",
            t.snapshot()
        );
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
    assert_eq!(
        unsafe { libc::waitpid(pid, std::ptr::null_mut(), libc::WNOHANG) },
        -1
    );
    assert_eq!(
        io::Error::last_os_error().raw_os_error(),
        Some(libc::ECHILD)
    );
}
