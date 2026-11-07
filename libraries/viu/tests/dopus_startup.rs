// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::json;
use std::{
    path::PathBuf,
    sync::{Arc, mpsc},
    time::Duration,
};
use viu::{
    bundle::Bundle,
    runtime::{Completion, Runtime},
};

#[path = "../../../apps/dopus/src/mix_start.rs"]
mod mix_start;

fn completion(runtime: &Runtime, rx: &mpsc::Receiver<()>) -> Completion {
    rx.recv_timeout(Duration::from_secs(10)).unwrap();
    let done = runtime.try_completion().unwrap();
    assert!(done.error.is_none(), "{:?}\n{}", done.error, done.stderr);
    done
}

#[test]
fn bundle_autostarts_and_embedded_host_initialises_before_first_command() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/dopus");
    for embedded in [false, true] {
        let profile = tempfile::tempdir().unwrap();
        std::fs::write(profile.path().join("config.conf.mix"), json!({"left":{"path":profile.path(),"previews":false},"right":{"path":profile.path(),"previews":false}}).to_string()).unwrap();
        let (tx, rx) = mpsc::channel();
        let mut runtime = Runtime::new(
            8,
            Arc::new(move || {
                let _ = tx.send(());
            }),
            None,
        );
        if embedded {
            mix_start::initialise(&mut runtime, &root, profile.path()).unwrap();
        } else {
            runtime.load(Bundle::load(&root, &[profile.path().to_string_lossy().into_owned()]).unwrap().program).unwrap();
        }
        let loaded = completion(&runtime, &rx);
        if embedded {
            assert!(
                loaded.model.is_none(),
                "explicit host profile must win over defaults"
            );
            // Consume the initialise event sent by the production helper,
            // rather than reproducing a sender that might be removed there.
            let initialised = completion(&runtime, &rx);
            assert_eq!(initialised.reply.unwrap().body["ready"], true);
            assert!(initialised.model.is_some());
        } else {
            // No initialise event: losing bundle auto-start must fail here.
            assert!(loaded.model.is_some(), "bundle must publish on load");
        }
        runtime
            .event("dopus.execute".into(), json!({"id":"view.toggle-hidden"}))
            .unwrap();
        let done = completion(&runtime, &rx);
        assert_eq!(
            done.reply.unwrap().rc,
            0,
            "first command must not return NOT_READY"
        );
        assert!(done.model.is_some());
    }
}
