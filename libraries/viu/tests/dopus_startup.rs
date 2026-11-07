// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::json;
use std::{
    path::PathBuf,
    sync::{Arc, mpsc},
    time::Duration,
};
use viu::{
    bundle::Bundle,
    runtime::{Completion, Program, Runtime},
};

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
        let program = if embedded {
            Program {
                path: root.join("app.mix"),
                source: "$app_args=nil\n".to_owned()
                    + &std::fs::read_to_string(root.join("app.mix")).unwrap(),
                scene: "dopus".into(),
                citizen: "dopus".into(),
            }
        } else {
            Bundle::load(&root, &[profile.path().to_string_lossy().into_owned()])
                .unwrap()
                .program
        };
        let (tx, rx) = mpsc::channel();
        let mut runtime = Runtime::new(
            8,
            Arc::new(move || {
                let _ = tx.send(());
            }),
            None,
        );
        runtime.load(program).unwrap();
        let loaded = completion(&runtime, &rx);
        if embedded {
            assert!(
                loaded.model.is_none(),
                "explicit host profile must win over defaults"
            );
            // The production comparison host sends this event immediately
            // after loading, with these same two fields.
            runtime
                .event(
                    "dopus.initialise".into(),
                    json!({"app_dir":root,"profile":profile.path()}),
                )
                .unwrap();
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
