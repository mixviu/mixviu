// SPDX-License-Identifier: MIT OR Apache-2.0
//! Native OSC 52 requests and replies, with one immutable owner per read.
//! Mix supplies the live permission flag. Formatters never enter JSON.
use super::{Sessions, State};
use crate::terminal::model::ClipboardRequest;
use std::sync::{Arc, Mutex, Weak, atomic::Ordering};

pub(crate) enum WindowClipboardRequest {
    Write {
        primary: bool,
        text: String,
    },
    Read {
        primary: bool,
        reply: PaneClipboardReply,
    },
}

pub(crate) struct PaneClipboardReply {
    state: Weak<Mutex<State>>,
    handle: String,
    owner: Weak<Mutex<crate::terminal::Terminal>>,
    formatter: Arc<dyn Fn(&str) -> String + Send + Sync>,
}
impl PaneClipboardReply {
    /// A close or tab focus change cannot redirect this answer to another pane.
    /// Hold ownership and the terminal lock through the bounded queue write.
    pub(crate) fn answer(self, text: &str) -> Result<bool, String> {
        let Some(state) = self.state.upgrade() else {
            return Ok(false);
        };
        let Some(terminal) = self.owner.upgrade() else {
            return Ok(false);
        };
        let locked = terminal.lock().map_err(|_| "terminal poisoned")?;
        let state = state.lock().map_err(|_| "terminal owner poisoned")?;
        if state
            .sessions
            .get(&self.handle)
            .is_none_or(|current| !Arc::ptr_eq(current, &terminal))
        {
            return Ok(false);
        }
        locked.clipboard_reply(&*self.formatter, text)?;
        Ok(true)
    }
}

impl Sessions {
    pub(super) fn configure_clipboard(&self, allowed: bool) {
        self.osc52_read.store(allowed, Ordering::Release);
        let panes = self
            .state
            .lock()
            .unwrap()
            .sessions
            .values()
            .cloned()
            .collect::<Vec<_>>();
        for pane in panes {
            pane.lock().unwrap().set_osc52_read(allowed);
        }
    }
    pub(crate) fn take_clipboard_requests(&self) -> Vec<WindowClipboardRequest> {
        let mut panes = {
            let mut state = self.state.lock().unwrap();
            let mut panes = state
                .sessions
                .iter()
                .map(|(handle, pane)| (handle.clone(), pane.clone()))
                .collect::<Vec<_>>();
            if !panes.is_empty() {
                let start = state.clipboard_cursor % panes.len();
                panes.rotate_left(start);
                state.clipboard_cursor = (start + 1) % panes.len();
            }
            panes
        };
        let mut out = Vec::new();
        for (handle, owner) in panes.drain(..) {
            let pane = owner.lock().unwrap();
            while let Some(request) = pane.take_clipboard_request() {
                out.push(match request {
                    ClipboardRequest::Store { primary, text } => {
                        WindowClipboardRequest::Write { primary, text }
                    }
                    ClipboardRequest::Load { primary, reply } => WindowClipboardRequest::Read {
                        primary,
                        reply: PaneClipboardReply {
                            state: Arc::downgrade(&self.state),
                            handle: handle.clone(),
                            owner: Arc::downgrade(&owner),
                            formatter: reply,
                        },
                    },
                });
                if out.len() == 16 {
                    // More panes/requests may remain. Wake the existing host;
                    // never drain and discard requests to fit a frame budget.
                    (self.wake)();
                    return out;
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal::Terminal;
    #[test]
    fn hidden_panes_keep_request_order_and_the_next_frame_keeps_the_remainder() {
        let sessions = Sessions::fixture();
        let bytes = b"\x1b]52;p;d29ybGQ=\x07".repeat(16);
        sessions.state.lock().unwrap().sessions.insert(
            "vt-2".into(),
            Arc::new(Mutex::new(Terminal::from_test_vt(80, 24, &bytes))),
        );
        sessions.state.lock().unwrap().sessions.insert(
            "vt-3".into(),
            Arc::new(Mutex::new(Terminal::from_test_vt(
                80,
                24,
                b"\x1b]52;c;aGVsbG8=\x07",
            ))),
        );
        let first = sessions.take_clipboard_requests();
        assert_eq!(first.len(), 16);
        for request in first {
            assert!(
                matches!(request,WindowClipboardRequest::Write {primary:true,text} if text=="world")
            );
        }
        assert!(
            matches!(&sessions.take_clipboard_requests()[..],[WindowClipboardRequest::Write {primary:false,text}] if text=="hello")
        );
        assert!(sessions.take_clipboard_requests().is_empty());
    }
    #[test]
    fn a_busy_low_handle_cannot_starve_another_panes_clipboard_request() {
        use rio_vt::clipboard::ClipboardType;
        use rio_vt::event::{EventListener, RioEvent, WindowId};
        let sessions = Sessions::fixture();
        let first = sessions.get("vt-1").unwrap();
        let fill = || {
            for _ in 0..16 {
                first.lock().unwrap().listener.send_event(
                    RioEvent::ClipboardStore(ClipboardType::Clipboard, "busy".into()),
                    WindowId::from(0),
                );
            }
        };
        sessions.state.lock().unwrap().sessions.insert(
            "vt-2".into(),
            Arc::new(Mutex::new(Terminal::from_test_vt(
                80,
                24,
                b"\x1b]52;p;d29ybGQ=\x07",
            ))),
        );
        fill();
        assert_eq!(sessions.take_clipboard_requests().len(), 16);
        fill();
        assert!(
            matches!(&sessions.take_clipboard_requests()[0],WindowClipboardRequest::Write {primary:true,text} if text=="world")
        );
    }
    #[test]
    fn reply_owner_survives_other_activity_but_never_a_close_or_handle_replacement() {
        let sessions = Sessions::fixture();
        let original = sessions.get("vt-1").unwrap();
        let receiver = original.lock().unwrap().listener.test_input_receiver();
        let read = || {
            while let Ok(message) = receiver.try_recv() {
                if let rio_vt::event::Msg::Input(bytes) = message {
                    return Some(bytes.into_owned());
                }
            }
            None
        };
        let make = || PaneClipboardReply {
            state: Arc::downgrade(&sessions.state),
            handle: "vt-1".into(),
            owner: Arc::downgrade(&original),
            formatter: Arc::new(|text| format!("reply:{text}")),
        };
        sessions.state.lock().unwrap().sessions.insert(
            "vt-2".into(),
            Arc::new(Mutex::new(Terminal::from_test_vt(80, 24, b"another pane"))),
        );
        assert!(make().answer("one").unwrap());
        assert_eq!(read(), Some(b"reply:one".to_vec()));
        let late = make();
        sessions
            .call("close", "vt-1", serde_json::json!({}))
            .unwrap();
        assert!(!late.answer("must not write").unwrap());
        let late = make();
        sessions.state.lock().unwrap().sessions.insert(
            "vt-1".into(),
            Arc::new(Mutex::new(Terminal::from_test_vt(80, 24, b"replacement"))),
        );
        assert!(!late.answer("must not retarget").unwrap());
        assert!(read().is_none());
    }
    #[test]
    fn live_read_permission_applies_to_existing_and_new_panes_without_disabling_writes() {
        use rio_vt::clipboard::ClipboardType;
        use rio_vt::event::{EventListener, RioEvent, WindowId};
        let sessions = Sessions::fixture();
        let query = |handle: &str| {
            sessions
                .get(handle)
                .unwrap()
                .lock()
                .unwrap()
                .listener
                .send_event(
                    RioEvent::ClipboardLoad(
                        0,
                        ClipboardType::Selection,
                        Arc::new(|text| format!("primary:{text}")),
                    ),
                    WindowId::from(0),
                )
        };
        query("vt-1");
        assert!(sessions.take_clipboard_requests().is_empty());
        assert!(
            sessions
                .call(
                    "clipboard.configure",
                    "",
                    serde_json::json!({"read_allowed":"true"})
                )
                .is_err()
        );
        sessions
            .call(
                "clipboard.configure",
                "",
                serde_json::json!({"read_allowed":true}),
            )
            .unwrap();
        query("vt-1");
        assert!(matches!(
            &sessions.take_clipboard_requests()[..],
            [WindowClipboardRequest::Read { primary: true, .. }]
        ));
        let home = tempfile::tempdir().unwrap();
        let created = sessions
            .call(
                "create",
                "",
                serde_json::json!({"pane_id":42,"cwd":home.path()}),
            )
            .unwrap();
        let handle = created["handle"].as_str().unwrap();
        query(handle);
        assert!(matches!(
            &sessions.take_clipboard_requests()[..],
            [WindowClipboardRequest::Read { primary: true, .. }]
        ));
        sessions
            .call(
                "clipboard.configure",
                "",
                serde_json::json!({"read_allowed":false}),
            )
            .unwrap();
        query(handle);
        query("vt-1");
        assert!(sessions.take_clipboard_requests().is_empty());
        sessions
            .get(handle)
            .unwrap()
            .lock()
            .unwrap()
            .listener
            .send_event(
                RioEvent::ClipboardStore(ClipboardType::Clipboard, "writes remain allowed".into()),
                WindowId::from(0),
            );
        assert!(
            matches!(&sessions.take_clipboard_requests()[..],[WindowClipboardRequest::Write {primary:false,text}] if text=="writes remain allowed")
        );
        sessions
            .call("close", handle, serde_json::json!({}))
            .unwrap();
    }
}
