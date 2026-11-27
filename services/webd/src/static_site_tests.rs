use std::collections::BTreeMap;

#[tokio::test]
async fn static_site_config_bootstrap_directory_and_routes_roundtrip() {
    let td = TempDir::new().unwrap();
    let config: config::node::WebdConfig = config::from_conf_mix_str(r#"
http_listen: "127.0.0.1:80"
listener: [{ id: "public", bind: "192.0.2.1:443", vhosts: ["www.example.com"], http2: true }]
vhost: [{ host: "www.example.com", redirect: "https://example.com",
  headers: { "content-security-policy": "default-src 'self'" },
  future_field: "ignored",
  acme: { provider: "letsencrypt_staging", challenge: "http01", contact_email: "ops@example.com" } }]
"#).unwrap();
    let config: config::node::WebdConfig = serde_json::from_value(serde_json::to_value(config).unwrap()).unwrap();
    assert!(config.listener[0].http2);
    let resolved = resolve_node_state(base_inputs(&config, &td)).unwrap();
    let state = &resolved.vhosts["www.example.com"];
    assert_eq!(state.redirect.as_deref(), Some("https://example.com"));
    assert_eq!(state.headers["content-security-policy"], "default-src 'self'");
    let store = Arc::new(props::sqlite::SqliteStore::new("webd", Connection::open_in_memory().unwrap()).unwrap());
    let mut router = props::PropsRouter::new("webd");
    let (runtime, _events) = vhosts_namespace::register_vhosts_namespace(&mut router, &store).unwrap();
    vhosts_bootstrap::bootstrap_upsert_from_config(&runtime, &config, 1000).await.unwrap();
    let rows = vhosts_namespace::snapshot_rows(&runtime).await.unwrap();
    assert_eq!(rows[0].redirect, state.redirect);
    assert_eq!(rows[0].headers["content-security-policy"], "default-src 'self'");
    let directory = vhost_directory::from_namespace_rows(&rows, &resolved.vhosts, &HashSet::new()).unwrap();
    assert_eq!(directory.primaries[0].state.redirect, state.redirect);
    let mut runtime_rows = rows;
    runtime_rows[0].source = Some("bus_runtime".into());
    let runtime_directory = vhost_directory::from_namespace_rows(&runtime_rows, &HashMap::new(), &HashSet::new()).unwrap();
    assert_eq!(runtime_directory.primaries[0].state.redirect, state.redirect);
    assert_eq!(runtime_directory.primaries[0].state.headers, state.headers);
    let node = synth_node(&td);
    node.vhosts.store(Arc::new(directory));
    let routes: serde_json::Value = serde_json::from_str(&bus::routes::list_body(&node)).unwrap();
    assert_eq!(routes["vhosts"][0]["redirect"], "https://example.com");
}

#[tokio::test]
async fn static_site_plain_listener_remains_http1() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let td = TempDir::new().unwrap();
    let app = build_router(static_site_node(&td, None));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let (stream, peer_addr) = listener.accept().await.unwrap();
        WebdConnHandler { app }.handle(AcceptedStream::Tcp(stream), ConnCtx {
            listener_id: "plain".into(), local_addr: addr, peer_addr, external: false, sni: None, tls: None,
        }).await;
    });
    let mut client = tokio::net::TcpStream::connect(addr).await.unwrap();
    client.write_all(b"GET /index.html HTTP/1.1\r\nHost: site-a.example\r\nConnection: close\r\n\r\n").await.unwrap();
    let mut response = String::new();
    client.read_to_string(&mut response).await.unwrap();
    assert!(response.starts_with("HTTP/1.1 200"));
    assert!(response.ends_with("SITE-A"));
    server.await.unwrap();
}

#[tokio::test]
async fn static_site_handler_owns_headers() {
    let td = TempDir::new().unwrap();
    let node = static_site_node(&td, None);
    let root = node.vhosts.load().primaries[0].state.www_dir.clone();
    std::fs::write(root.join("owned.mix"),
            "return { status: 200, headers: { \"content-security-policy\": \"default-src 'none'\" }, body: \"handler\" }\n").unwrap();
    node.handlers
        .store(Arc::new(mix_handler::HandlerTable::from_rows(vec![
            handlers_namespace::HandlerRow {
                route_id: "owned".into(),
                vhost_fqdn: "site-a.example".into(),
                method: "GET".into(),
                path_pattern: "/owned".into(),
                handler_kind: "mix".into(),
                handler_ref: "owned.mix".into(),
                enabled: true,
                capabilities: vec![],
            },
        ])));
    let response = build_router(node)
        .oneshot(req("site-a.example", "/owned"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()["content-security-policy"],
        "default-src 'none'"
    );
    assert_eq!(body_bytes(response).await, b"handler");
}

fn static_site_node(td: &TempDir, redirect: Option<&str>) -> Arc<NodeState> {
    static_site_node_with_ws(td, redirect, None)
}
fn static_site_node_with_ws(
    td: &TempDir,
    redirect: Option<&str>,
    ws: Option<String>,
) -> Arc<NodeState> {
    static_site_node_with_upstreams(td, redirect, ws, None)
}
fn static_site_node_with_upstreams(td: &TempDir, redirect: Option<&str>, ws: Option<String>, jmap: Option<String>) -> Arc<NodeState> {
    let node = synth_node(td);
    let root = node.vhosts.load().primaries[0].state.www_dir.clone();
    let vhost = Arc::new(VhostState {
        fqdn: "site-a.example".into(),
        www_dir: root,
        redirect: redirect.map(str::to_owned),
        headers: static_site::headers(&BTreeMap::from([(
            "content-security-policy".into(),
            "default-src 'self'".into(),
        )]))
        .unwrap(),
        db: None,
        jmap_upstream: jmap,
        noded_ws: ws,
        docs_dir: None,
        dev_session_email: None,
        dev_session_password: None,
        public_read_email: None,
        public_read_password: None,
        system_sender_email: None,
        system_sender_password: None,
        mfa_break_glass: false,
        stats: Arc::new(stats::WebdStats::new()),
        session_epoch_cache: SessionEpochCache::default(),
        public_response_cache: public_response_cache::Cache::default(),
    });
    node.vhosts.store(Arc::new(
        vhost_directory::VhostDirectory::build(vec![vhost_directory::VhostDirectoryEntry {
            state: vhost,
            aliases: vec![],
        }])
        .unwrap(),
    ));
    node
}

#[tokio::test]
async fn static_site_redirect_preserves_uri_and_head() {
    let td = TempDir::new().unwrap();
    let node = static_site_node(&td, Some("https://example.com"));
    for app in [build_router(node.clone()), build_http_redirect_router(node)] {
        for method in ["GET", "HEAD", "POST"] {
            let mut request = req("site-a.example", "/any/path?q=%2F");
            *request.method_mut() = method.parse().unwrap();
            let response = app.clone().oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::PERMANENT_REDIRECT);
            assert_eq!(
                response.headers()["location"],
                "https://example.com/any/path?q=%2F"
            );
            assert_eq!(
                response.headers()["content-security-policy"],
                "default-src 'self'"
            );
            assert!(body_bytes(response).await.is_empty());
        }
    }
}

#[tokio::test]
async fn static_site_redirect_preserves_acme_lane() {
    let td = TempDir::new().unwrap();
    let node = static_site_node(&td, Some("https://example.com"));
    seed_challenge(&node, SAMPLE_TOKEN, "proof").await;
    let response = build_http_redirect_router(node)
        .oneshot(req(
            "site-a.example",
            &format!("/.well-known/acme-challenge/{SAMPLE_TOKEN}"),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body_bytes(response).await, b"proof");
}

#[tokio::test]
async fn static_site_encoding_headers_ranges_and_revalidation() {
    let td = TempDir::new().unwrap();
    let node = static_site_node(&td, None);
    let root = node.vhosts.load().primaries[0].state.www_dir.clone();
    // ServeDir transports prebuilt bytes without decoding them.
    std::fs::write(root.join("probe.css"), b"plain-css").unwrap();
    std::fs::write(root.join("probe.css.br"), b"brotli-css").unwrap();
    std::fs::write(root.join("probe.css.gz"), b"gzip-css").unwrap();
    let app = build_router(node);
    for (accept, encoding, expected) in [
        ("br", Some("br"), &b"brotli-css"[..]),
        ("gzip", Some("gzip"), &b"gzip-css"[..]),
        ("identity", None, &b"plain-css"[..]),
        ("br;q=0, gzip;q=0", None, &b"plain-css"[..]),
    ] {
        let mut request = req("site-a.example", "/probe.css");
        request
            .headers_mut()
            .insert("accept-encoding", accept.parse().unwrap());
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get("content-encoding")
                .map(|v| v.to_str().unwrap()),
            encoding
        );
        assert!(
            response
                .headers()
                .get_all("vary")
                .iter()
                .any(|v| v.to_str().unwrap().contains("accept-encoding"))
        );
        assert_eq!(
            response.headers()["content-security-policy"],
            "default-src 'self'"
        );
        assert_eq!(response.headers()["x-content-type-options"], "nosniff");
        let modified = response.headers()["last-modified"].clone();
        assert_eq!(body_bytes(response).await, expected);
        let mut conditional = req("site-a.example", "/probe.css");
        conditional
            .headers_mut()
            .insert("accept-encoding", accept.parse().unwrap());
        conditional
            .headers_mut()
            .insert("if-modified-since", modified);
        let response = app.clone().oneshot(conditional).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
        assert!(response.headers().contains_key("vary"));
    }
    let mut request = req("site-a.example", "/probe.css");
    request
        .headers_mut()
        .insert("accept-encoding", "br".parse().unwrap());
    request
        .headers_mut()
        .insert("range", "bytes=0-2".parse().unwrap());
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
    assert_eq!(response.headers()["content-encoding"], "br");
    assert_eq!(body_bytes(response).await, b"bro");
}

#[test]
fn static_site_invalid_rows_drop_at_resolve() {
    use config::node::{WebdAcmeChallenge, WebdAcmeProvider, WebdVhostAcmeConfig, WebdVhostConfig};
    let td = TempDir::new().unwrap();
    let healthy = WebdVhostConfig {
        host: "good.example".into(),
        redirect: Some("https://example.com".into()),
        acme: Some(WebdVhostAcmeConfig {
            provider: WebdAcmeProvider::LetsEncryptStaging,
            challenge: WebdAcmeChallenge::Http01,
            contact_email: "ops@example.com".into(),
        }),
        ..Default::default()
    };
    let mut bad_rows = Vec::new();
    for target in [
        "http://example.com",
        "/relative",
        "https://example.com/path",
        "https://example.com/",
        "https://example.com?q",
        "https://example.com#fragment",
        "https://bad.example",
        "https://u@example.com",
    ] {
        let mut row = healthy.clone();
        row.host = "bad.example".into();
        row.redirect = Some(target.into());
        bad_rows.push(row);
    }
    for field in ["cms", "jmap", "ws"] {
        let mut row = healthy.clone();
        row.host = "bad.example".into();
        match field {
            "cms" => row.cms_db_path = Some("/missing".into()),
            "jmap" => row.jmap_upstream = Some("https://example.com".into()),
            _ => row.noded_ws = Some("wss://example.com".into()),
        }
        bad_rows.push(row);
    }
    for (name, value) in [
        ("connection", "close"),
        ("content-length", "1"),
        ("transfer-encoding", "chunked"),
        ("upgrade", "websocket"),
        ("te", "trailers"),
        ("bad name", "ok"),
        ("x-test", "bad\r\nvalue"),
    ] {
        let mut row = healthy.clone();
        row.host = "bad.example".into();
        row.headers.insert(name.into(), value.into());
        bad_rows.push(row);
    }
    for row in bad_rows {
        let webd = config::node::WebdConfig {
            http_listen: Some("127.0.0.1:80".into()),
            vhost: vec![healthy.clone(), row],
            ..Default::default()
        };
        let result = resolve_node_state(base_inputs(&webd, &td)).unwrap();
        assert!(result.vhosts.contains_key("good.example"));
        assert!(!result.vhosts.contains_key("bad.example"));
        assert_eq!(result.disabled_vhosts.len(), 1);
    }
}

#[test]
fn static_site_header_defaults_preserve_response_owner() {
    let defaults =
        static_site::headers(&BTreeMap::from([("x-owner".into(), "config".into())])).unwrap();
    let mut response = HeaderMap::new();
    response.insert("x-owner", "handler".parse().unwrap());
    static_site::apply(&mut response, &defaults);
    assert_eq!(response["x-owner"], "handler");
}

#[tokio::test]
async fn static_site_tls_alpn_serves_h2_and_http1() {
    use http_body_util::BodyExt;
    let td = TempDir::new().unwrap();
    let cert = rcgen::generate_simple_self_signed(vec!["site-a.example".into()]).unwrap();
    let cert_path = td.path().join("cert.pem");
    let key_path = td.path().join("key.pem");
    std::fs::write(&cert_path, cert.cert.pem()).unwrap();
    std::fs::write(&key_path, cert.key_pair.serialize_pem()).unwrap();
    // SNI's existing test path deliberately bypasses the production LE validator.
    let resolver = Arc::new(
        SniCertResolver::from_config(
            &[TlsIdentityConfig {
                server_name: "site-a.example".into(),
                cert: cert_path.to_string_lossy().into_owned(),
                key: key_path.to_string_lossy().into_owned(),
                default: true,
                no_sni_fallback: true,
            }],
            false,
        )
        .unwrap(),
    );
    let tls = ListenerTls::with_alpn(Some(resolver), vec![b"h2".to_vec(), b"http/1.1".to_vec()]);
    let (_, server_config) = tls.current().unwrap();
    assert_eq!(
        server_config.alpn_protocols,
        [b"h2".to_vec(), b"http/1.1".to_vec()]
    );
    let mut roots = rustls::RootCertStore::empty();
    roots.add(cert.cert.der().clone()).unwrap();
    let mut client_config = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    client_config.alpn_protocols = vec![b"h2".to_vec()];
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    use futures_util::{SinkExt, StreamExt};
    let upstream = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let upstream_addr = upstream.local_addr().unwrap();
    let echo = tokio::spawn(async move {
        let (stream, _) = upstream.accept().await.unwrap();
        let mut socket = tokio_tungstenite::accept_async(stream).await.unwrap();
        let message = socket.next().await.unwrap().unwrap();
        socket.send(message).await.unwrap();
    });
    let mail = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let mail_addr = mail.local_addr().unwrap();
    let upstream_app = Router::new()
        .route("/jmap", axum::routing::post(|headers: HeaderMap| async move {
            format!("{}|{}", headers["host"].to_str().unwrap(), headers["x-forwarded-host"].to_str().unwrap())
        }))
        .route("/jmap/blob/test", axum::routing::get(|headers: HeaderMap| async move {
            assert_eq!(headers["x-forwarded-host"], "site-a.example");
            "blob-bytes"
        }))
        .route("/jmap/eventsource", axum::routing::get(|headers: HeaderMap| async move {
            assert_eq!(headers["x-forwarded-host"], "site-a.example");
            let stream = futures_util::stream::unfold(0, |n| async move {
                if n == 2 { return None; }
                if n == 1 { tokio::time::sleep(std::time::Duration::from_millis(200)).await; }
                Some((Ok::<_, std::io::Error>(axum::body::Bytes::from_static(b"data: ping\\n\\n")), n+1))
            });
            ([(axum::http::header::CONTENT_TYPE, "text/event-stream")], Body::from_stream(stream))
        }));
    let mail_task = tokio::spawn(async move { axum::serve(mail, upstream_app).await.unwrap(); });
    let node = static_site_node_with_upstreams(&td, None, Some(format!("ws://{upstream_addr}/ws")), Some(format!("http://{mail_addr}")));
    let root = node.vhosts.load().primaries[0].state.www_dir.clone();
    std::fs::write(root.join("host.mix"), "return $HOST .. \"|\" .. $HEADERS.host\\n").unwrap();
    node.handlers.store(Arc::new(mix_handler::HandlerTable::from_rows(vec![handlers_namespace::HandlerRow {
        route_id: "host".into(), vhost_fqdn: "site-a.example".into(), method: "GET".into(),
        path_pattern: "/host".into(), handler_kind: "mix".into(), handler_ref: "host.mix".into(),
        enabled: true, capabilities: vec![],
    }])));
    let app = build_router(node);
    let task = tokio::spawn(async move {
        for protocol in [&b"h2"[..], &b"http/1.1"[..]] {
            let (stream, peer_addr) = listener.accept().await.unwrap();
            let stream = tokio_rustls::TlsAcceptor::from(server_config.clone())
                .accept(stream)
                .await
                .unwrap();
            assert_eq!(stream.get_ref().1.alpn_protocol(), Some(protocol));
            let app = app.clone();
            let tls = tls.clone();
            tokio::spawn(async move {
                WebdConnHandler { app }
                    .handle(
                        AcceptedStream::Tls(Box::new(stream)),
                        ConnCtx {
                            listener_id: "test".into(),
                            local_addr: addr,
                            peer_addr,
                            external: false,
                            sni: Some("site-a.example".into()),
                            tls: Some(tls),
                        },
                    )
                    .await;
            });
        }
    });
    let stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    let stream = tokio_rustls::TlsConnector::from(Arc::new(client_config.clone()))
        .connect("site-a.example".try_into().unwrap(), stream)
        .await
        .unwrap();
    let (mut client, connection) = hyper::client::conn::http2::handshake(
        hyper_util::rt::TokioExecutor::new(),
        hyper_util::rt::TokioIo::new(stream),
    )
    .await
    .unwrap();
    let driver = tokio::spawn(connection);
    // HTTP/2 sends :authority, without an HTTP/1 Host field.
    let request = axum::http::Request::builder()
        .uri("https://site-a.example/index.html")
        .body(http_body_util::Empty::<axum::body::Bytes>::new())
        .unwrap();
    let response = client.send_request(request).await.unwrap();
    assert_eq!(response.version(), axum::http::Version::HTTP_2);
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.into_body().collect().await.unwrap().to_bytes(),
        "SITE-A"
    );
    for (method, path, expected) in [
        ("POST", "/jmap", format!("{mail_addr}|site-a.example")),
        ("GET", "/jmap/blob/test", "blob-bytes".into()),
        ("GET", "/host", "site-a.example|site-a.example".into()),
    ] {
        let request = axum::http::Request::builder().method(method)
            .uri(format!("https://site-a.example{path}"))
            .body(http_body_util::Empty::<axum::body::Bytes>::new()).unwrap();
        let response = client.send_request(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.into_body().collect().await.unwrap().to_bytes(), expected);
    }
    let request = axum::http::Request::builder().uri("https://site-a.example/jmap/eventsource")
        .body(http_body_util::Empty::<axum::body::Bytes>::new()).unwrap();
    let response = client.send_request(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["content-type"], "text/event-stream");
    let mut body = response.into_body();
    assert!(body.frame().await.unwrap().unwrap().is_data());
    assert!(tokio::time::timeout(std::time::Duration::from_millis(50), body.frame()).await.is_err());
    assert!(body.frame().await.unwrap().unwrap().is_data());
    let request = axum::http::Request::builder().uri("https://site-a.example/index.html")
        .header("host", "other.example")
        .body(http_body_util::Empty::<axum::body::Bytes>::new()).unwrap();
    let response = client.send_request(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    mail_task.abort();
    // No RFC 8441: a WebSocket uses its own HTTP/1.1 TLS connection,
    // while the same listener also serves h2.
    client_config.alpn_protocols = vec![b"http/1.1".to_vec()];
    let stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    let stream = tokio_rustls::TlsConnector::from(Arc::new(client_config))
        .connect("site-a.example".try_into().unwrap(), stream)
        .await
        .unwrap();
    assert_eq!(stream.get_ref().1.alpn_protocol(), Some(&b"http/1.1"[..]));
    let (mut websocket, response) =
        tokio_tungstenite::client_async("wss://site-a.example/ws", stream)
            .await
            .unwrap();
    assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
    websocket
        .send(tokio_tungstenite::tungstenite::Message::Text(
            "roundtrip".into(),
        ))
        .await
        .unwrap();
    let message = tokio::time::timeout(std::time::Duration::from_secs(5), websocket.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(message.into_text().unwrap(), "roundtrip");
    echo.await.unwrap();
    task.await.unwrap();
    drop(client);
    driver.abort();
}
