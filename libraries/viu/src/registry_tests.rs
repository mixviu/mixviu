// SPDX-License-Identifier: MIT OR Apache-2.0
use crate::runtime::{Program, Runtime, Services};
use std::{collections::BTreeSet, sync::{Arc, mpsc}, time::Duration};

#[test]
fn generated_lint_vocabulary_matches_the_live_complete_registry() {
    let temporary = tempfile::tempdir().unwrap();
    let wake: crate::runtime::Wake = Arc::new(|| {});
    let (clipboard, _requests) = crate::clipboard::Clipboard::channel(wake.clone());
    let (tx, rx) = mpsc::channel();
    let mut runtime = Runtime::with_services(4, wake.clone(), None, Services {
        registry_observer: Some(Arc::new(move |names| { tx.send(names).unwrap(); })),
        platform: Some(Default::default()),
        ui_state: Some(Default::default()),
        calls: Some(Default::default()),
        timers: Some(Default::default()),
        documents: Some(Default::default()),
        rasters: Some(Default::default()),
        macros: Some(macros::Session::with_roots("registry-test", Some(macros::Roots {
            builtin: temporary.path().join("builtin"), user: temporary.path().join("user"),
            home: temporary.path().into(), mix: temporary.path().join("unused-mix"),
        }))),
        chooser: Some(crate::chooser::Service::new(1, b"/tmp".to_vec(), wake).unwrap()),
        clipboard: Some(clipboard),
        ..Default::default()
    });
    runtime.load(Program { path: temporary.path().join("registry.mix"), source: "print(1)".into(), scene: "registry".into(), citizen: "registry".into() }).unwrap();
    let actual: BTreeSet<_> = rx.recv_timeout(Duration::from_secs(10)).unwrap().into_iter().collect();
    let generated: BTreeSet<_> = mix::analyzer::VIU_EXTENSIONS.iter().map(|name| (*name).to_owned()).collect();
    assert_eq!(actual, generated, "live host registrations must round-trip through lint generation");
}
