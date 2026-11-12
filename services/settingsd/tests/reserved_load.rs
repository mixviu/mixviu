// SPDX-License-Identifier: MIT OR Apache-2.0
//! The defaults policy must not change the person's stored bindings.
use serde_json::json;
use settings::{Binding, registry};
use settingsd::{authority::Authority, store::Store};

#[test]
fn stored_arrow_and_fkey_overrides_load_unchanged_and_reach_the_app() {
    let binding = Binding {
        instance: "fixture".into(),
        profile: "default".into(),
    };
    for chords in [json!(["Alt+ArrowLeft"]), json!(["F12", "Ctrl+Shift+F1"])] {
        let dir = tempfile::tempdir().unwrap();
        let values = registry::Values::from([("keymaps.prefs.page.back".into(), chords.clone())]);
        let (store, accepted) = Store::create(dir.path(), binding.clone(), values.clone()).unwrap();
        drop(store);
        let before = std::fs::read(dir.path().join("desktop.conf.mix")).unwrap();
        let startup = std::process::Command::new(env!("CARGO_BIN_EXE_settingsd"))
            .args([
                "seed",
                "--root",
                dir.path().to_str().unwrap(),
                "--instance",
                "fixture",
            ])
            .output()
            .unwrap();
        assert!(
            startup.status.success(),
            "{}",
            String::from_utf8_lossy(&startup.stderr)
        );
        assert!(!String::from_utf8_lossy(&startup.stderr).contains("dropped"));
        let (store, loaded) = Store::open(dir.path(), &binding).unwrap();
        assert_eq!(loaded.values, values);
        assert_eq!(loaded.content_digest, accepted.content_digest);
        assert_eq!(
            std::fs::read(dir.path().join("desktop.conf.mix")).unwrap(),
            before
        );
        let mut authority = Authority::new(store, loaded).unwrap();
        assert_eq!(
            serde_json::to_value(&authority.snapshot.desktop.keymaps["prefs"]["page.back"])
                .unwrap(),
            chords
        );
        let described =
            settingsd::service::dispatch(&mut authority, "settings.describe", "{}").unwrap();
        assert!(described.is_object());
        let served = settingsd::service::dispatch(
            &mut authority,
            "settings.get",
            &json!({"binding":binding,"view":"values"}).to_string(),
        )
        .unwrap();
        assert_eq!(served["values"]["keymaps.prefs.page.back"], chords);
    }
}
