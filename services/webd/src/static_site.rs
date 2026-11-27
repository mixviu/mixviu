// SPDX-License-Identifier: MIT OR Apache-2.0
//! Static-site policy shared by startup and the runtime vhost namespace.
use axum::http::{HeaderMap, HeaderName, HeaderValue, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use std::collections::BTreeMap;

pub(crate) fn headers(values: &BTreeMap<String, String>) -> anyhow::Result<HeaderMap> {
    let mut result = HeaderMap::new();
    for (name, value) in values {
        let name = HeaderName::from_bytes(name.as_bytes())?;
        anyhow::ensure!(
            !matches!(
                name.as_str(),
                "content-length"
                    | "transfer-encoding"
                    | "connection"
                    | "keep-alive"
                    | "proxy-connection"
                    | "proxy-authenticate"
                    | "proxy-authorization"
                    | "te"
                    | "trailer"
                    | "upgrade"
                    | "content-encoding"
                    | "content-range"
            ),
            "forbidden vhost header: {name}"
        );
        result.insert(name, HeaderValue::from_str(value)?);
    }
    Ok(result)
}

pub(crate) fn validate(
    host: &str,
    aliases: &[String],
    target: Option<&str>,
    values: &BTreeMap<String, String>,
) -> anyhow::Result<()> {
    headers(values)?;
    if let Some(target) = target {
        let uri: Uri = target.parse()?;
        let authority = uri
            .authority()
            .ok_or_else(|| anyhow::anyhow!("redirect requires an HTTPS origin"))?;
        anyhow::ensure!(
            uri.scheme_str() == Some("https")
                && !authority.as_str().contains('@')
                && !target.contains(['?', '#'])
                && !target["https://".len()..].contains('/'),
            "redirect requires an absolute HTTPS origin without path, query or fragment"
        );
        let dest = daemon::http_host::parse_request_host(authority.as_str())
            .ok_or_else(|| anyhow::anyhow!("invalid redirect hostname"))?;
        anyhow::ensure!(
            !std::iter::once(host)
                .chain(aliases.iter().map(String::as_str))
                .filter_map(daemon::http_host::parse_request_host)
                .any(|h| h == dest),
            "redirect points to its own vhost"
        );
    }
    Ok(())
}

pub(crate) fn apply(response: &mut HeaderMap, defaults: &HeaderMap) {
    for (name, value) in defaults {
        if name == axum::http::header::VARY {
            for token in value
                .to_str()
                .unwrap_or("")
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                ensure_vary(response, token);
            }
        } else if !response.contains_key(name) {
            response.insert(name, value.clone());
        }
    }
}

pub(crate) fn ensure_vary(headers: &mut HeaderMap, token: &str) {
    let present = headers
        .get_all(axum::http::header::VARY)
        .iter()
        .any(|value| {
            value.to_str().is_ok_and(|value| {
                value
                    .split(',')
                    .any(|item| item.trim().eq_ignore_ascii_case(token))
            })
        });
    if !present && let Ok(value) = HeaderValue::from_str(token) {
        headers.append(axum::http::header::VARY, value);
    }
}

pub(crate) fn redirect(target: &str, uri: &Uri, defaults: &HeaderMap) -> Response {
    let location = format!(
        "{target}{}",
        uri.path_and_query()
            .map(|p| p.as_str())
            .filter(|p| p.starts_with('/'))
            .unwrap_or("/")
    );
    let mut response = (
        StatusCode::PERMANENT_REDIRECT,
        [(axum::http::header::LOCATION, location)],
    )
        .into_response();
    apply(response.headers_mut(), defaults);
    response
}
