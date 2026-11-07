// SPDX-License-Identifier: MIT OR Apache-2.0
use maild::config::Config;

#[test]
fn all_config_paths_resolve_through_both_loaders() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.conf.mix");
    let text = r#"{
        database: "mix:var/maild/mail.db",
        blob_dir: "mix:var/maild/blobs", mds_dir: "mix:var/maild/mds",
        dkim_private_key: "mix:etc/maild/dkim.pem",
        tls_cert: "mix:etc/maild/cert.pem", tls_key: "mix:etc/maild/key.pem",
        tls_key_root: "mix:var/maild/tls",
        inbound_filter: "mix:etc/maild/filter.mix",
        spam_db_dir: "mix:var/maild/spam", spam_baseline_db: "mix:share/maild/spam.db",
        rules_pack_path: "mix:etc/maild/rules.conf.mix", rule_stats_dir: "mix:var/maild/rules",
        dkim: {key_root: "mix:var/maild/dkim", domain: [{domain: "example.com", selector: "default",
            algorithm: "rsa-sha256", key_path: "mix:etc/maild/domain.pem"}]},
        tls: {identity: [{server_name: "example.com", cert: "mix:etc/maild/sni.pem", key: "mix:etc/maild/sni.key"}]}
    }"#;
    std::fs::write(&path, text).unwrap();
    for cfg in [
        Config::load(path.to_str().unwrap()).unwrap(),
        Config::try_load_optional(&path).unwrap().unwrap(),
    ] {
        let pairs = [
            (cfg.database_path, "mix:var/maild/mail.db"),
            (cfg.blob_dir, "mix:var/maild/blobs"),
            (cfg.mds_dir, "mix:var/maild/mds"),
            (cfg.dkim_private_key.unwrap(), "mix:etc/maild/dkim.pem"),
            (cfg.tls_cert.unwrap(), "mix:etc/maild/cert.pem"),
            (cfg.tls_key.unwrap(), "mix:etc/maild/key.pem"),
            (
                cfg.tls_key_root.unwrap().to_str().unwrap().into(),
                "mix:var/maild/tls",
            ),
            (cfg.inbound_filter.unwrap(), "mix:etc/maild/filter.mix"),
            (cfg.spam_db_dir.unwrap(), "mix:var/maild/spam"),
            (cfg.spam_baseline_db.unwrap(), "mix:share/maild/spam.db"),
            (
                cfg.rules_pack_path.unwrap(),
                "mix:etc/maild/rules.conf.mix",
            ),
            (cfg.rule_stats_dir.unwrap(), "mix:var/maild/rules"),
            (
                cfg.dkim.key_root.unwrap().to_str().unwrap().into(),
                "mix:var/maild/dkim",
            ),
            (
                cfg.dkim.domains[0].key_path.to_str().unwrap().into(),
                "mix:etc/maild/domain.pem",
            ),
            (cfg.tls.identity[0].cert.clone(), "mix:etc/maild/sni.pem"),
            (cfg.tls.identity[0].key.clone(), "mix:etc/maild/sni.key"),
        ];
        for (actual, logical) in pairs {
            assert_eq!(
                std::path::Path::new(&actual),
                paths::resolve(logical).unwrap()
            );
        }
    }
}

#[test]
fn invalid_logical_paths_fail_and_user_paths_keep_their_meaning() {
    for invalid in ["mix:var/../escape", "mixos:unknown/file"] {
        let cfg = Config {
            blob_dir: invalid.into(),
            ..Default::default()
        };
        assert!(cfg.resolve_paths().is_err());
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.conf.mix");
        std::fs::write(&path, format!("{{blob_dir: \"{invalid}\"}}")).unwrap();
        assert!(Config::load(path.to_str().unwrap()).is_err());
        assert!(Config::try_load_optional(&path).is_err());
    }
    let cfg = Config {
        database_path: "relative/mail.db".into(),
        blob_dir: "/home/user/blobs".into(),
        ..Default::default()
    }
    .resolve_paths()
    .unwrap();
    assert_eq!(cfg.database_path, "relative/mail.db");
    assert_eq!(cfg.blob_dir, "/home/user/blobs");
}
