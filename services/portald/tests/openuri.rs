// SPDX-License-Identifier: MIT OR Apache-2.0
//! Real session D-Bus and native broker, with a fake apps citizen.
use bus::native_client::SupervisedClient;
use futures_util::StreamExt;
use portald::{openuri::{OpenUri, INTERFACE}, portal::{BUS_NAME, OBJECT_PATH}};
use serde_json::{Value as Json, json};
use std::{collections::HashMap, io::{BufRead, BufReader}, process::{Child, Command, Stdio}, sync::Arc, time::Duration};
use zbus::{Connection, MatchRule, MessageStream, zvariant::{OwnedObjectPath, OwnedValue, Str, Fd}};

struct Daemon(Child);
impl Drop for Daemon {
    fn drop(&mut self) { let _ = self.0.kill(); let _ = self.0.wait(); }
}

async fn connect(address: &str) -> Connection {
    zbus::connection::Builder::address(address).unwrap().build().await.unwrap()
}

#[tokio::test]
async fn native_apps_and_dbus_contract() {
    let dir = tempfile::tempdir().unwrap();
    let mut daemon = Daemon(Command::new("dbus-daemon").args(["--session", "--nofork", "--nopidfile", "--print-address=1"]).arg(format!("--address=unix:path={}/bus", dir.path().display())).stdout(Stdio::piped()).spawn().unwrap());
    let mut address = String::new();
    BufReader::new(daemon.0.stdout.take().unwrap()).read_line(&mut address).unwrap();
    let broker = test_broker::Broker::start();
    let apps = Arc::new(SupervisedClient::connect_options("apps", &broker.url).with_commands(["apps.handler", "apps.open"].into_iter().map(|s| bus::CommandDescriptor::new(s, &[], "test", false)).collect()).connect().await.unwrap());
    let mut incoming = apps.incoming().unwrap();
    let (seen_tx, mut seen) = tokio::sync::mpsc::unbounded_channel();
    let provider = apps.clone();
    let task = tokio::spawn(async move {
        while let Some(command) = incoming.recv().await {
            let body: Json = serde_json::from_str(&command.body).unwrap();
            let rc = if command.command == "apps.handler" { if body["scheme"] == "https" { 0 } else { 14 } } else if body["uri"] == "refuse:" { 10 } else { 0 };
            seen_tx.send((command.command.clone(), body)).unwrap();
            provider.respond(&command, rc, &json!({"id":"fake"}).to_string()).await.unwrap();
        }
    });
    let native = Arc::new(SupervisedClient::connect_options("portald-test", &broker.url).connect().await.unwrap());
    let server = connect(address.trim()).await;
    server.object_server().at(OBJECT_PATH, OpenUri::new(native.clone())).await.unwrap();
    server.request_name(BUS_NAME).await.unwrap();
    let caller = connect(address.trim()).await;
    let proxy = zbus::Proxy::new(&caller, BUS_NAME, OBJECT_PATH, INTERFACE).await.unwrap();
    assert_eq!(proxy.get_property::<u32>("version").await.unwrap(), 5);
    for (scheme, expected) in [("https", true), ("file", false)] {
        assert_eq!(proxy.call::<_, _, bool>("SchemeSupported", &(scheme, HashMap::<String, OwnedValue>::new())).await.unwrap(), expected);
        assert_eq!(seen.recv().await.unwrap(), ("apps.handler".into(), json!({"scheme":scheme})));
    }
    for (i, method, uri, expected) in [(0, "OpenURI", "https://example.invalid/?a=%ff", 0), (1, "OpenURI", "refuse:", 2), (2, "OpenURI", &"x".repeat(8192), 0), (3, "OpenURI", &"é".repeat(4097), 2), (4, "OpenFile", "", 2), (5, "OpenDirectory", "", 2)] {
        let token = format!("test{i}");
        let path = portald::screencast::handle::request_path(caller.unique_name().unwrap().as_str(), &token).unwrap();
        let mut signals = MessageStream::for_match_rule(MatchRule::builder().msg_type(zbus::message::Type::Signal).interface("org.freedesktop.portal.Request").unwrap().member("Response").unwrap().path(path.clone()).unwrap().build(), &caller, Some(4)).await.unwrap();
        let options = HashMap::from([("handle_token", OwnedValue::from(Str::from(token.as_str()))), ("activation_token", OwnedValue::from(Str::from("as given\n"))), ("ask", OwnedValue::from(true)), ("writable", OwnedValue::from(true))]);
        let actual: OwnedObjectPath = if method == "OpenURI" {
            proxy.call(method, &("ignored", uri, options)).await.unwrap()
        } else {
            let file = std::fs::File::open("/dev/null").unwrap();
            proxy.call(method, &("ignored", Fd::from(&file), options)).await.unwrap()
        };
        assert_eq!(actual, path);
        let msg = tokio::time::timeout(Duration::from_secs(3), signals.next()).await.unwrap().unwrap().unwrap();
        let (code, results): (u32, HashMap<String, OwnedValue>) = msg.body().deserialize().unwrap();
        assert_eq!(code, expected);
        assert!(results.is_empty());
        if i < 3 { assert_eq!(seen.recv().await.unwrap(), ("apps.open".into(), json!({"uri":uri,"activation_token":"as given\n"}))); }
        else { assert!(seen.try_recv().is_err()); }
    }
    task.abort();
    apps.close().await;
    // Broker's unavailable status must be Failed, never false.
    let error = proxy.call::<_, _, bool>("SchemeSupported", &("https", HashMap::<String, OwnedValue>::new())).await.unwrap_err();
    assert!(error.to_string().contains("org.freedesktop.portal.Error.Failed"), "{error}");
    let options = HashMap::from([("handle_token", OwnedValue::from(Str::from("down")))]);
    let mut signals = MessageStream::for_match_rule(MatchRule::builder().msg_type(zbus::message::Type::Signal).interface("org.freedesktop.portal.Request").unwrap().member("Response").unwrap().build(), &caller, Some(4)).await.unwrap();
    let _: OwnedObjectPath = proxy.call("OpenURI", &("", "https://example.invalid/", options)).await.unwrap();
    let msg = tokio::time::timeout(Duration::from_secs(3), signals.next()).await.unwrap().unwrap().unwrap();
    assert_eq!(msg.body().deserialize::<(u32, HashMap<String, OwnedValue>)>().unwrap().0, 2);
    native.close().await;
    assert!(proxy.call::<_, _, bool>("SchemeSupported", &("https", HashMap::<String, OwnedValue>::new())).await.unwrap_err().to_string().contains("org.freedesktop.portal.Error.Failed"));
}
