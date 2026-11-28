// SPDX-License-Identifier: MIT OR Apache-2.0
#![cfg(feature = "native")]
use bus::native_client::NodedClient;
use bus::{BusMessage, CommandDescriptor};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio::time::{Duration, timeout};
use tokio_tungstenite::tungstenite::Message;

async fn old_broker() -> (
    String,
    mpsc::UnboundedReceiver<BusMessage>,
    mpsc::UnboundedSender<BusMessage>,
    tokio::task::JoinHandle<()>,
) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}", listener.local_addr().unwrap());
    let (seen_tx, seen) = mpsc::unbounded_channel();
    let (out, mut out_rx) = mpsc::unbounded_channel::<BusMessage>();
    let task = tokio::spawn(async move {
        let (socket, _) = listener.accept().await.unwrap();
        let mut socket = tokio_tungstenite::accept_async(socket).await.unwrap();
        loop {
            tokio::select! {
                Some(message) = out_rx.recv() => {
                    socket.send(Message::Text(message.to_wire().into())).await.unwrap();
                },
                next = socket.next() => {
                    let Some(Ok(Message::Text(text))) = next else { break; };
                    let request = bus::parse(&text).unwrap();
                    if request.message_type() == Some("request") {
                        let rc = if request.command_name() == Some("noded.commands.set") { "10" } else { "0" };
                        let mut reply = BusMessage::new().with_header("type", "response")
                            .with_header("from", "noded").with_header("rc", rc).with_body("{}");
                        if let Some(id) = request.get("id") { reply.set("id", id); }
                        socket.send(Message::Text(reply.to_wire().into())).await.unwrap();
                    }
                    seen_tx.send(request).unwrap();
                }
            }
        }
    });
    (url, seen, out, task)
}

async fn next(seen: &mut mpsc::UnboundedReceiver<BusMessage>) -> BusMessage {
    timeout(Duration::from_secs(3), seen.recv())
        .await
        .unwrap()
        .unwrap()
}

fn descriptors(name: &str) -> Vec<CommandDescriptor> {
    vec![CommandDescriptor::new(
        name,
        &[],
        "Example description",
        true,
    )]
}

#[tokio::test]
async fn anonymous_commands_never_publish_and_named_commands_publish_once_per_change() {
    let (url, mut seen, _, task) = old_broker().await;
    let client = NodedClient::connect_anonymous(&url)
        .await
        .unwrap()
        .with_commands(descriptors("example.read"));
    client
        .call("noded", "noded.ping", serde_json::Value::Null)
        .await
        .unwrap();
    assert_eq!(next(&mut seen).await.command_name(), Some("noded.ping"));
    client.close().await;
    task.abort();

    let (url, mut seen, _, task) = old_broker().await;
    let client = NodedClient::connect("example", &url).await.unwrap();
    assert_eq!(next(&mut seen).await.command_name(), Some("noded.register"));
    let initial = next(&mut seen).await;
    assert_eq!(initial.command_name(), Some("noded.commands.set"));
    assert_eq!(initial.body, "[]");
    client.set_commands(descriptors("example.read"));
    client.set_commands(descriptors("example.read"));
    client.set_commands(descriptors("example.write"));
    // A synchronous burst coalesces to its newest names, including deduped calls.
    for name in ["example.write"] {
        let update = next(&mut seen).await;
        assert_eq!(update.command_name(), Some("noded.commands.set"));
        assert_eq!(
            serde_json::from_str::<Vec<String>>(&update.body).unwrap(),
            [name]
        );
    }
    // A roundtrip fences all previous writes and proves no duplicate set.
    client
        .call("noded", "noded.ping", serde_json::Value::Null)
        .await
        .unwrap();
    assert_eq!(next(&mut seen).await.command_name(), Some("noded.ping"));
    client.close().await;
    task.abort();
}

#[tokio::test]
async fn unknown_manifest_rc_does_not_prevent_registration_or_serving() {
    let (url, mut seen, out, task) = old_broker().await;
    let client = NodedClient::connect("example", &url).await.unwrap();
    assert_eq!(next(&mut seen).await.command_name(), Some("noded.register"));
    assert_eq!(
        next(&mut seen).await.command_name(),
        Some("noded.commands.set")
    );
    let mut incoming = client.incoming_async().await.unwrap();
    out.send(
        BusMessage::new()
            .with_header("command", "example.read")
            .with_header("type", "request")
            .with_header("from", "caller")
            .with_header("id", "serve-example"),
    )
    .unwrap();
    let request = timeout(Duration::from_secs(3), incoming.recv())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(request.command, "example.read");
    client.respond(&request, 0, "{}").await.unwrap();
    let response = next(&mut seen).await;
    assert_eq!(response.message_type(), Some("response"));
    assert_eq!(response.get("id"), Some("serve-example"));
    client
        .call("noded", "noded.ping", serde_json::Value::Null)
        .await
        .unwrap();
    assert_eq!(next(&mut seen).await.command_name(), Some("noded.ping"));
    client.close().await;
    task.abort();
}
