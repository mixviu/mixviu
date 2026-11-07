// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::{Value, json};
use std::{
    io::{Read, Write},
    path::PathBuf,
    sync::{Arc, mpsc},
    time::Duration,
};
use viu::runtime::{Program, Runtime};
fn probe(reply: Value, call: &str) -> (u8, Value) {
    probe_sequence(vec![reply], call)
}
fn probe_sequence(replies: Vec<Value>, call: &str) -> (u8, Value) {
    probe_module(replies, call, "mail.mix", Vec::new())
}
fn probe_module(
    replies: Vec<Value>,
    call: &str,
    module: &str,
    auth: Vec<&'static str>,
) -> (u8, Value) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let bodies: Vec<_> = replies
        .iter()
        .map(|r| r.to_string().replace("ORIGIN", &url))
        .collect();
    let server = std::thread::spawn(move || {
        for (index, body) in bodies.into_iter().enumerate() {
            let until = std::time::Instant::now() + Duration::from_secs(10);
            let mut socket = loop {
                match listener.accept() {
                    Ok((socket, _)) => break socket,
                    Err(e)
                        if e.kind() == std::io::ErrorKind::WouldBlock
                            && std::time::Instant::now() < until =>
                    {
                        std::thread::sleep(Duration::from_millis(5))
                    }
                    Err(e) => panic!("client did not deliver expected request {index}: {e}"),
                }
            };
            socket
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut bytes = Vec::new();
            let mut buf = [0; 4096];
            loop {
                let n = socket.read(&mut buf).unwrap();
                assert!(n > 0, "request ended before its headers/body");
                bytes.extend_from_slice(&buf[..n]);
                if let Some(end) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                    let header = String::from_utf8_lossy(&bytes[..end]);
                    let length = header
                        .lines()
                        .find_map(|line| {
                            line.to_ascii_lowercase()
                                .strip_prefix("content-length:")
                                .map(|s| s.trim().parse::<usize>().unwrap())
                        })
                        .unwrap_or(0);
                    if bytes.len() >= end + 4 + length {
                        break;
                    }
                }
            }
            assert!(
                String::from_utf8_lossy(&bytes)
                    .contains(auth.get(index).copied().unwrap_or("Basic dXNlcjpzZWNyZXQ="))
            );
            write!(
                socket,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            )
            .unwrap();
        }
    });
    let scratch = tempfile::tempdir().unwrap();
    let password = scratch.path().join("password");
    std::fs::write(&password, "secret\n").unwrap();
    let (tx, rx) = mpsc::channel();
    let mut runtime = Runtime::new(
        8,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        None,
    );
    runtime
        .load(Program {
            path: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/email/probe.mix"),
            source: format!(
                r#"
$m=require("{module}")
on probe
  try
    $e=$event.args.entry
    $r={call}
    reply(0,json_encode($r))
  catch $message, $err
    reply(10,json_encode({{code:$err.code,message:$message}}))
  end
end
"#
            ),
            scene: "email".into(),
            citizen: "email".into(),
        })
        .unwrap();
    rx.recv_timeout(Duration::from_secs(10)).unwrap();
    assert!(runtime.try_completion().unwrap().error.is_none());
    runtime
        .event(
            "probe".into(),
            json!({"entry":{"url":url,"user":"user","password_file":password}}),
        )
        .unwrap();
    rx.recv_timeout(Duration::from_secs(15)).unwrap();
    let done = runtime.try_completion().unwrap();
    assert!(done.error.is_none(), "{:?}", done.error);
    let reply = done.reply.unwrap();
    server.join().unwrap();
    (reply.rc, reply.body)
}

#[test]
fn gold_writes_accept_missing_error_maps_and_preserve_partial_failures() {
    let session = r#"{account:"1",api:$e.url,token:nil,using:[]}"#;
    let (rc, body) = probe_module(
        vec![json!({"methodResponses":[["Email/set",{"updated":{"a":null}},"c0"]]})],
        &format!(r#"$m.batch($e,{session},"email.mark",{{ids:["a"],read:true}})"#),
        "protocol.mix",
        vec![],
    );
    assert_eq!(rc, 0, "{body}");
    assert_eq!(body["done"], json!(["a"]));
    assert_eq!(body["partial"], false);
    let (rc, body) = probe_module(
        vec![
            json!({"methodResponses":[["Email/set",{"updated":{"a":null},"notUpdated":{"missing":{"type":"notFound"}}},"c0"]]}),
        ],
        &format!(r#"$m.batch($e,{session},"email.mark",{{ids:["a","missing"],read:false}})"#),
        "protocol.mix",
        vec![],
    );
    assert_eq!(rc, 0, "{body}");
    assert_eq!(body["done"], json!(["a"]));
    assert_eq!(
        body["failed"],
        json!([{"id":"missing","reason":"notFound"}])
    );
    assert_eq!(body["partial"], true);
}
#[test]
fn gold_token_session_and_query_preserve_bearer_auth_and_query_order() {
    let session = json!({"apiUrl":"ORIGIN/api/mail","accounts":{"1":{"name":"One"}},"primaryAccounts":{"urn:ietf:params:jmap:mail":"1"},"capabilities":{"urn:ietf:params:jmap:core":{},"urn:ietf:params:jmap:mail":{}}});
    let (rc, body) = probe_module(
        vec![
            json!({"token":"test-token","accountId":"1"}),
            session,
            json!({"methodResponses":[["Email/query",{"ids":["a","b"],"total":2},"c0"]]}),
            json!({"methodResponses":[["Email/get",{"list":[{"id":"b","subject":"Second"},{"id":"a","subject":"First","hasAttachment":false}]},"c0"]]}),
        ],
        r#"if true then $s=$m.connect($e);$m.list($e,$s,{mailbox:"inbox",limit:100}) end"#,
        "protocol.mix",
        vec![
            "Basic dXNlcjpzZWNyZXQ=",
            "Bearer test-token",
            "Bearer test-token",
            "Bearer test-token",
        ],
    );
    assert_eq!(rc, 0, "{body}");
    assert_eq!(body["emails"][0]["id"], "a");
    assert_eq!(body["emails"][1]["id"], "b");
    assert!(body["emails"][0].get("bodyValues").is_some());
    assert_eq!(body["emails"][0]["bodyValues"], Value::Null);
    assert_eq!(body["emails"][0]["hasAttachment"], false);
    let (rc, body) = probe_module(
        vec![
            json!({"token":"test-token","accountId":"1"}),
            json!({"apiUrl":"https://example.test/leak","accounts":{"1":{}},"capabilities":{}}),
        ],
        "$m.connect($e)",
        "protocol.mix",
        vec!["Basic dXNlcjpzZWNyZXQ=", "Bearer test-token"],
    );
    assert_eq!(rc, 10);
    assert_eq!(body["code"], "SERVER");
}
#[test]
fn query_order_is_preserved_and_different_get_identities_are_refused() {
    let query = json!({"methodResponses":[["Email/query",{"ids":["a","b"],"total":2},"c0"]]});
    let get = json!({"methodResponses":[["Email/get",{"list":[{"id":"b","subject":"Second"},{"id":"a","subject":"First"}]},"c0"]]});
    let (rc, body) = probe_sequence(vec![query.clone(), get], "$m.page($e,\"1\",\"inbox\",0)");
    assert_eq!(rc, 0, "{body}");
    assert_eq!(body["messages"][0]["id"], "a");
    assert_eq!(body["messages"][1]["id"], "b");
    let (rc, body) = probe_sequence(
        vec![
            query,
            json!({"methodResponses":[["Email/get",{"list":[{"id":"a"},{"id":"other"}]},"c0"]]}),
        ],
        "$m.page($e,\"1\",\"inbox\",0)",
    );
    assert_eq!(rc, 10);
    assert_eq!(body["code"], "SNAPSHOT");
}
#[test]
fn embedded_jmap_session_uses_io_and_refuses_off_origin_endpoints() {
    let session = json!({"apiUrl":"ORIGIN/jmap","accounts":{"1":{"name":"One"}},"primaryAccounts":{"urn:ietf:params:jmap:mail":"1"}});
    let (rc, reply) = probe(session, "$m.session($e)");
    assert_eq!(rc, 0);
    assert_eq!(reply["account"], "1");
    let (rc, reply) = probe(
        json!({"apiUrl":"https://example.test/jmap"}),
        "$m.session($e)",
    );
    assert_eq!(rc, 10);
    assert_eq!(reply["code"], "ORIGIN");
}
#[test]
fn embedded_jmap_refuses_unmatched_methods_and_partial_message_bodies() {
    let (rc, _) = probe(
        json!({"methodResponses":[["Wrong/get",{},"c0"]]}),
        "$m.call($e,\"Mailbox/get\",{accountId:\"1\"})",
    );
    assert_eq!(rc, 10);
    let (rc, reply) = probe(
        json!({"methodResponses":[["Email/get",{"list":[{"id":"x","textBody":[{"partId":"p"}],"bodyValues":{"p":{"value":"partial","isTruncated":true}}}]},"c0"]]}),
        "$m.message($e,\"1\",\"x\")",
    );
    assert_eq!(rc, 10);
    assert_eq!(reply["code"], "BODY");
}
