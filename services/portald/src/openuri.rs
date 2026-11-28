// SPDX-License-Identifier: MIT OR Apache-2.0
//! OpenURI v5: policy and launching belong to the apps Mix citizen.
use crate::filechooser::mapping::Options;
use crate::portal::PortalError;
use crate::screencast::{handle, request};
use bus::{PortReply, native_client::SupervisedClient};
use serde_json::{Value as Json, json};
use std::collections::HashMap;
use std::sync::{Arc, atomic::AtomicU64};
use std::time::Duration;
use zbus::{Connection, message::Header, object_server::ResponseDispatchNotifier};
use zbus::zvariant::{OwnedFd, OwnedObjectPath};

pub const INTERFACE: &str = "org.freedesktop.portal.OpenURI";
pub const VERSION: u32 = 5;
const MAX_URI: usize = 8192;
const IO_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Clone)]
pub struct Bridge {
    client: Arc<SupervisedClient>,
    counter: Arc<AtomicU64>,
}

impl Bridge {
    pub fn new(client: Arc<SupervisedClient>) -> Self {
        Self { client, counter: handle::counter() }
    }

    async fn call(&self, command: &str, body: Json) -> Result<bool, PortalError> {
        match tokio::time::timeout(IO_TIMEOUT, self.client.call_typed("apps", command, body)).await {
            Ok(Ok(PortReply::Ok { rc, .. } | PortReply::AppError { rc, .. })) if rc < 20 => Ok(rc == 0),
            // Do not log the reply/body: it could contain the URI or token.
            _ => Err(PortalError::Failed("apps unavailable".into())),
        }
    }
}

pub struct OpenUri {
    bridge: Bridge,
}

impl OpenUri {
    pub fn new(client: Arc<SupervisedClient>) -> Self {
        Self { bridge: Bridge::new(client) }
    }

    async fn begin(
        &self,
        uri: Option<&str>,
        options: Options,
        header: Header<'_>,
        connection: &Connection,
    ) -> Result<ResponseDispatchNotifier<OwnedObjectPath>, PortalError> {
        let owner = header.sender().ok_or_else(|| PortalError::InvalidArgument("no sender".into()))?.to_string();
        let path = handle::request_path(&owner, &handle::token(&options, "handle_token", &self.bridge.counter)?)?;
        let body = open_body(uri, &options)?;
        let ticket = request::export(connection, &path, &owner).await?;
        let bridge = self.bridge.clone();
        let connection = connection.clone();
        let (response, dispatched) = ResponseDispatchNotifier::new(path);
        tokio::spawn(async move {
            dispatched.await;
            if !request::live(&connection, &ticket).await { return; }
            let success = match body {
                Some(body) => bridge.call("apps.open", body).await.unwrap_or(false),
                None => false,
            };
            request::respond(&connection, &ticket, &owner, if success { request::SUCCESS } else { request::FAILED }, HashMap::new()).await;
        });
        Ok(response)
    }
}

fn open_body(uri: Option<&str>, options: &Options) -> Result<Option<Json>, PortalError> {
    let Some(uri) = uri.filter(|uri| uri.len() <= MAX_URI) else { return Ok(None); };
    let mut body = json!({"uri": uri});
    if let Some(token) = options.get("activation_token") {
        let token = token.downcast_ref::<&str>().map_err(|_| PortalError::InvalidArgument("activation_token must be a string".into()))?;
        body["activation_token"] = json!(token);
    }
    Ok(Some(body))
}

#[zbus::interface(name = "org.freedesktop.portal.OpenURI")]
impl OpenUri {
    #[zbus(property, name = "version")]
    fn version(&self) -> u32 { VERSION }

    #[zbus(name = "SchemeSupported")]
    async fn scheme_supported(&self, scheme: &str, options: Options) -> Result<bool, PortalError> {
        let _ = options;
        self.bridge.call("apps.handler", json!({"scheme": scheme})).await
    }

    #[zbus(name = "OpenURI")]
    async fn open_uri(&self, parent_window: &str, uri: &str, options: Options, #[zbus(header)] header: Header<'_>, #[zbus(connection)] connection: &Connection) -> Result<ResponseDispatchNotifier<OwnedObjectPath>, PortalError> {
        let _ = parent_window;
        self.begin(Some(uri), options, header, connection).await
    }

    #[zbus(name = "OpenFile")]
    async fn open_file(&self, parent_window: &str, fd: OwnedFd, options: Options, #[zbus(header)] header: Header<'_>, #[zbus(connection)] connection: &Connection) -> Result<ResponseDispatchNotifier<OwnedObjectPath>, PortalError> {
        let _ = (parent_window, fd);
        tracing::warn!("OpenURI.OpenFile is not implemented (stage 2)");
        self.begin(None, options, header, connection).await
    }

    #[zbus(name = "OpenDirectory")]
    async fn open_directory(&self, parent_window: &str, fd: OwnedFd, options: Options, #[zbus(header)] header: Header<'_>, #[zbus(connection)] connection: &Connection) -> Result<ResponseDispatchNotifier<OwnedObjectPath>, PortalError> {
        let _ = (parent_window, fd);
        tracing::warn!("OpenURI.OpenDirectory is not implemented (stage 2)");
        self.begin(None, options, header, connection).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zbus::zvariant::{OwnedValue, Str};

    #[test]
    fn uri_cap_is_bytes_and_content_and_token_pass_unparsed() {
        let options = Options::from([("activation_token".into(), OwnedValue::from(Str::from("bad\ntoken")))]);
        let uri = "é".repeat(4096);
        assert_eq!(open_body(Some(&uri), &options).unwrap().unwrap(), json!({"uri":uri,"activation_token":"bad\ntoken"}));
        assert!(open_body(Some(&(uri + "x")), &options).unwrap().is_none());
        let uri = "not even a URI\n";
        assert_eq!(open_body(Some(uri), &Options::new()).unwrap().unwrap(), json!({"uri":uri}));
        assert!(open_body(None, &options).unwrap().is_none());
    }
}
