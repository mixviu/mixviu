// SPDX-License-Identifier: MIT OR Apache-2.0
//! Bounded HTTP text transport. Protocol, authentication and retry policy belong
//! to the Mix caller. Redirects/proxies/retries are deliberately not implicit.
use serde_json::{Value, json};
use std::{
    io::{Read, Write},
    net::IpAddr,
    os::unix::fs::OpenOptionsExt,
    rc::Rc,
    time::Duration,
};
fn error(message: &str) -> mix::MixError {
    mix::MixError::RuntimeError {
        msg: message.into(),
        span: None,
    }
}
pub fn binding() -> mix::evaluator::ExtFn {
    Rc::new(|args| {
        Box::pin(async move {
            if args.len() != 1 {
                return Err(error("viu_http expects one request map"));
            }
            let request = crate::runtime::to_json(&args[0])?;
            crate::runtime::from_json(text_request(&request).await?)
        })
    })
}
pub async fn text_request(request: &Value) -> mix::MixResult<Value> {
    if !request.is_object()
        || (!request["body"].is_null() && !request["body"].is_string())
        || (!request["headers"].is_null() && !request["headers"].is_object())
        || (!request["timeout"].is_null() && request["timeout"].as_u64().is_none())
        || (!request["method"].is_null() && !request["method"].is_string())
        || (!request["ca_pem"].is_null() && !request["ca_pem"].is_string())
    {
        return Err(error("Invalid HTTP request field types"));
    }
    let url = reqwest::Url::parse(
        request["url"]
            .as_str()
            .ok_or_else(|| error("HTTP URL required"))?,
    )
    .map_err(|_| error("Invalid HTTP URL"))?;
    let loopback = url.host_str().is_some_and(|h| {
        h == "localhost"
            || h.trim_matches(['[', ']'])
                .parse::<IpAddr>()
                .is_ok_and(|ip| ip.is_loopback())
    });
    if !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || !(url.scheme() == "https" || (url.scheme() == "http" && loopback))
    {
        return Err(error(
            "HTTP requires HTTPS or loopback HTTP, without URL credentials/fragment",
        ));
    }
    let timeout = request["timeout"].as_u64().unwrap_or(10);
    if !(1..=300).contains(&timeout) {
        return Err(error("HTTP deadline must be 1..300 seconds"));
    }
    let method = match request["method"].as_str().unwrap_or("GET") {
        "GET" => reqwest::Method::GET,
        "POST" => reqwest::Method::POST,
        _ => return Err(error("HTTP method must be GET or POST")),
    };
    let limit = |key: &str, default: u64| -> mix::MixResult<usize> {
        match request.get(key) {
            None => Ok(default as usize),
            Some(value) => value
                .as_u64()
                .filter(|n| *n > 0 && *n <= 64 * 1024 * 1024)
                .map(|n| n as usize)
                .ok_or_else(|| error("HTTP byte limit must be 1..64 MiB")),
        }
    };
    let max_request = limit("max_request", 1024 * 1024)?;
    let max_response = limit("max_response", 4 * 1024 * 1024)?;
    let mut body = request["body"].as_str().unwrap_or("").as_bytes().to_vec();
    if let Some(path) = request.get("request_file") {
        let path = path
            .as_str()
            .filter(|p| std::path::Path::new(p).is_absolute())
            .ok_or_else(|| error("HTTP request_file must be absolute"))?;
        if !body.is_empty() {
            return Err(error("HTTP body and request_file exclude one another"));
        }
        let file = std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NONBLOCK | libc::O_CLOEXEC)
            .open(path)
            .map_err(|_| error("Cannot open HTTP request file"))?;
        let metadata = file
            .metadata()
            .map_err(|_| error("Cannot inspect HTTP request file"))?;
        if !metadata.is_file() || metadata.len() > max_request as u64 {
            return Err(error("HTTP request file exceeds regular-file/byte bounds"));
        }
        file.take(max_request as u64 + 1)
            .read_to_end(&mut body)
            .map_err(|_| error("Cannot read HTTP request file"))?;
    }
    if body.len() > max_request {
        return Err(error("HTTP request exceeds its byte limit"));
    }
    let destination = request
        .get("response_file")
        .map(|v| {
            v.as_str()
                .filter(|p| std::path::Path::new(p).is_absolute())
                .ok_or_else(|| error("HTTP response_file must be absolute"))
        })
        .transpose()?;
    let mut builder = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .no_proxy()
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(timeout));
    if let Some(pem) = request["ca_pem"].as_str() {
        if pem.len() > 1024 * 1024 {
            return Err(error("HTTP CA exceeds 1 MiB"));
        }
        builder = builder.add_root_certificate(
            reqwest::Certificate::from_pem(pem.as_bytes()).map_err(|_| error("Invalid HTTP CA"))?,
        );
    }
    let client = builder
        .build()
        .map_err(|_| error("HTTP client setup failed"))?;
    let mut call = client.request(method, url).body(body);
    if !request["basic"].is_null() {
        let username = request["basic"]["username"]
            .as_str()
            .ok_or_else(|| error("HTTP Basic username required"))?;
        let password = request["basic"]["password"]
            .as_str()
            .ok_or_else(|| error("HTTP Basic password required"))?;
        if username.len() + password.len() > 8192 || username.contains(':') {
            return Err(error("Invalid HTTP Basic credentials"));
        }
        call = call.basic_auth(username, Some(password));
    }
    if let Some(headers) = request["headers"].as_object() {
        if headers.len() > 32 || serde_json::to_string(headers).unwrap_or_default().len() > 16384 {
            return Err(error("HTTP header budget exceeded"));
        }
        for (name, value) in headers {
            let name = reqwest::header::HeaderName::from_bytes(name.as_bytes())
                .map_err(|_| error("Invalid HTTP header name"))?;
            let value = reqwest::header::HeaderValue::from_str(
                value
                    .as_str()
                    .ok_or_else(|| error("HTTP header must be text"))?,
            )
            .map_err(|_| error("Invalid HTTP header value"))?;
            call = call.header(name, value);
        }
    }
    tokio::time::timeout(Duration::from_secs(timeout), async {
        let mut response = call
            .send()
            .await
            .map_err(|_| error("HTTP transport failed; delivery outcome may be unknown"))?;
        let status = response.status().as_u16();
        let mut bytes = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| error("HTTP response interrupted"))?
        {
            if bytes.len() + chunk.len() > max_response {
                return Err(error("HTTP response exceeds its byte limit"));
            }
            bytes.extend_from_slice(&chunk);
        }
        if let Some(path) = destination.filter(|_| (200..300).contains(&status)) {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(path)
                .map_err(|_| {
                    error("Cannot create HTTP response file (an existing file is never replaced)")
                })?;
            if file
                .write_all(&bytes)
                .and_then(|_| file.sync_all())
                .is_err()
            {
                let _ = std::fs::remove_file(path);
                return Err(error("Cannot save HTTP response file"));
            }
            return Ok(json!({"status":status,"path":path,"bytes":bytes.len()}));
        }
        let body =
            String::from_utf8(bytes).map_err(|_| error("HTTP text response is not UTF-8"))?;
        Ok(json!({"status":status,"body":body}))
    })
    .await
    .map_err(|_| error("HTTP deadline; delivery outcome may be unknown"))?
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    #[test]
    fn binary_files_keep_bytes_enforce_bounds_and_never_replace_a_download() {
        let root = tempfile::tempdir().unwrap();
        let upload = root.path().join("upload");
        let saved = root.path().join("saved");
        let limited = root.path().join("limited");
        std::fs::write(&upload, [0, 255, 42]).unwrap();
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            for _ in 0..3 {
                let (mut socket, _) = listener.accept().unwrap();
                socket
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                let mut bytes = Vec::new();
                let mut buf = [0; 4096];
                loop {
                    let n = socket.read(&mut buf).unwrap();
                    assert!(n > 0);
                    bytes.extend_from_slice(&buf[..n]);
                    if let Some(end) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                        if bytes.len() >= end + 4 + 3 {
                            assert_eq!(&bytes[end + 4..], [0, 255, 42]);
                            break;
                        }
                    }
                }
                socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\nConnection: close\r\n\r\n\0\xff\x2a").unwrap();
            }
        });
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let request = json!({"url":url,"method":"POST","request_file":upload,"response_file":saved,"max_response":3});
        let reply = rt.block_on(text_request(&request)).unwrap();
        assert_eq!(reply["bytes"], 3);
        assert_eq!(std::fs::read(&saved).unwrap(), [0, 255, 42]);
        assert!(rt.block_on(text_request(&request)).is_err());
        assert_eq!(std::fs::read(&saved).unwrap(), [0, 255, 42]);
        assert!(rt.block_on(text_request(&json!({"url":url,"method":"POST","request_file":upload,"response_file":limited,"max_response":2}))).is_err());
        assert!(!limited.exists());
        server.join().unwrap();
    }
    #[test]
    fn bounded_http_text_and_redirects_use_the_real_socket() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            for (status, body) in [("200 OK", "hello π"), ("302 Found", "")] {
                let (mut socket, _) = listener.accept().unwrap();
                let mut buf = [0; 4096];
                let _ = socket.read(&mut buf).unwrap();
                write!(socket,"HTTP/1.1 {status}\r\nContent-Length: {}\r\nLocation: http://127.0.0.1:1/leak\r\nConnection: close\r\n\r\n{body}",body.len()).unwrap();
            }
        });
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        assert_eq!(
            rt.block_on(text_request(&json!({"url":url}))).unwrap()["body"],
            "hello π"
        );
        assert_eq!(
            rt.block_on(text_request(&json!({"url":url}))).unwrap()["status"],
            302
        );
        server.join().unwrap();
        for invalid in [
            json!({"url":"http://example.test/"}),
            json!({"url":"https://user:secret@example.test/"}),
            json!({"url":url,"timeout":301}),
            json!({"url":url,"body":"x".repeat(1024*1024+1)}),
            json!({"url":url,"method":42}),
            json!({"url":url,"ca_pem":false}),
        ] {
            assert!(rt.block_on(text_request(&invalid)).is_err());
        }
    }
}
