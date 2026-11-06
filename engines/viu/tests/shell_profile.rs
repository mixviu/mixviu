// SPDX-License-Identifier: MIT OR Apache-2.0
use std::{
    path::PathBuf,
    sync::{Arc, mpsc},
    time::Duration,
};

#[test]
fn engine_embeds_the_shell_mix_profile_without_app_feature_unification() {
    let calls = [
        "json_parse",
        "re_match",
        "toml_parse",
        "date_format",
        "url_parse",
        "uuid",
        "hash_sha256",
        "http_get",
        "sqlopen",
        "ws_connect",
        "tar_list",
    ];
    let mut source = String::new();
    for name in calls {
        // Zero arguments exercise dispatch but cannot open a file or network
        // connection. An arity refusal is expected; a missing feature is not.
        source.push_str(&format!("try {name}() catch $message if contains($message,\"feature\") or contains($message,\"undefined function\") then raise(\"PROFILE\",$message) end end\n"));
    }
    let (tx, rx) = mpsc::channel();
    let mut runtime = viu::runtime::Runtime::new(
        2,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        None,
    );
    runtime
        .load(viu::runtime::Program {
            path: PathBuf::from("shell-profile.mix"),
            source,
            scene: "profile".into(),
            citizen: "profile".into(),
        })
        .unwrap();
    rx.recv_timeout(Duration::from_secs(10)).unwrap();
    let completed = runtime.try_completion().unwrap();
    assert!(completed.error.is_none(), "{:?}", completed.error);
}
