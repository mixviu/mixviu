//! Obsolete app chords must never prevent a stored session starting.
use serde_json::json;
use settings::{Binding, Desktop, registry};
use settingsd::{
    authority::Authority,
    store::{Accepted, Store},
};

fn binding() -> Binding {
    Binding {
        instance: "fixture".into(),
        profile: "default".into(),
    }
}

#[test]
fn stored_reserved_profiles_start_and_keep_valid_settings_without_rewriting() {
    for (chords, remaining, dropped) in [
        (json!(["Alt+ArrowLeft"]), None, "Alt+ArrowLeft"),
        (json!(["F12"]), None, "F12"),
        (
            json!(["Ctrl+S", "Alt+ArrowLeft"]),
            Some(json!(["Ctrl+S"])),
            "Alt+ArrowLeft",
        ),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let (store, mut stored) =
            Store::create(dir.path(), binding(), registry::Values::new()).unwrap();
        drop(store);
        stored
            .values
            .insert("keymaps.prefs.page.back".into(), chords);
        stored
            .values
            .insert("keymaps.ced.file.save".into(), json!(["Ctrl+S"]));
        stored.seal().unwrap(); // reproduce a valid seal written by the previous binary
        let bytes = strict::to_string_pretty(&stored).unwrap();
        std::fs::write(dir.path().join("desktop.conf.mix"), &bytes).unwrap();
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
        let diagnostic = String::from_utf8_lossy(&startup.stderr);
        assert_eq!(
            diagnostic
                .lines()
                .filter(|l| l.contains("keymaps.prefs.page.back") && l.contains(dropped))
                .count(),
            1,
            "{diagnostic}"
        );
        assert_eq!(
            std::fs::read_to_string(dir.path().join("desktop.conf.mix")).unwrap(),
            bytes
        );
        let (store, loaded) = Store::open(dir.path(), &binding()).unwrap();
        assert_eq!(
            loaded.values.get("keymaps.prefs.page.back"),
            remaining.as_ref()
        );
        assert_eq!(loaded.values["keymaps.ced.file.save"], json!(["Ctrl+S"]));
        let mut authority = Authority::new(store, loaded).unwrap();
        let described =
            settingsd::service::dispatch(&mut authority, "settings.describe", "{}").unwrap();
        assert!(described.is_object());
        for chord in ["F12", "Alt+ArrowLeft", "command+Left"] {
            let refused = registry::apply(
                &authority.accepted.values,
                &registry::Values::from([("keymaps.prefs.page.back".into(), json!([chord]))]),
                &[],
            );
            assert!(refused.is_err(), "new write {chord} accepted");
            let request = settings::ApplyRequest {
                binding: binding(),
                expected_incarnation: authority.accepted.incarnation.clone(),
                expected_revision: authority.accepted.revision,
                operation_id: "reserved-write".into(),
                changes: registry::Values::from([(
                    "keymaps.prefs.page.back".into(),
                    json!([chord]),
                )]),
                reset: vec![],
                request_digest: None,
            };
            assert_eq!(
                authority.apply(request).unwrap_err()["status"],
                "validation_failed"
            );
        }
    }
}

#[test]
fn a_stored_mirror_with_reserved_keys_loads_offline_without_rewriting() {
    use settingsd::mirror::{self, Cache, Origin};
    let dir = tempfile::tempdir().unwrap();
    let mut old = Accepted::initial(binding(), registry::Values::new()).unwrap();
    old.values.insert(
        "keymaps.prefs.page.back".into(),
        json!(["Alt+ArrowLeft", "Ctrl+B"]),
    );
    old.seal().unwrap();
    let cache = Cache::of(&old, "authority").unwrap();
    let bytes = strict::to_string_pretty(&cache).unwrap();
    std::fs::write(dir.path().join(mirror::CACHE), &bytes).unwrap();
    let (loaded, origin) = mirror::open(dir.path(), &binding(), "authority", || {
        anyhow::bail!("offline")
    })
    .unwrap();
    assert_eq!(origin, Origin::Cached);
    assert_eq!(loaded.values["keymaps.prefs.page.back"], json!(["Ctrl+B"]));
    assert_eq!(
        std::fs::read_to_string(dir.path().join(mirror::CACHE)).unwrap(),
        bytes
    );
    Authority::mirror(&loaded).unwrap();
}

#[test]
fn format4_upgrade_removes_obsolete_keys_but_keeps_original_evidence() {
    let dir = tempfile::tempdir().unwrap();
    let mut old = settingsd::format4::Record::initial(binding(), Desktop::default()).unwrap();
    old.desktop.keymaps.insert(
        "ced".into(),
        std::collections::BTreeMap::from([(
            "file.save".into(),
            vec!["Ctrl+S".into(), "F12".into()],
        )]),
    );
    old.seal().unwrap();
    let bytes = strict::to_string_pretty(&old).unwrap();
    std::fs::write(dir.path().join("desktop.conf.mix"), &bytes).unwrap();
    let (store, loaded) = Store::open(dir.path(), &binding()).unwrap();
    assert!(store.migrated);
    assert_eq!(loaded.values["keymaps.ced.file.save"], json!(["Ctrl+S"]));
    let written: Accepted =
        strict::from_str(&std::fs::read_to_string(dir.path().join("desktop.conf.mix")).unwrap())
            .unwrap();
    assert_eq!(written.values, loaded.values);
    assert_eq!(
        std::fs::read_to_string(dir.path().join(settingsd::migration::KEPT)).unwrap(),
        bytes
    );
}
