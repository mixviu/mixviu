// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>

//! Typed compatibility surface over the canonical native ABP client.

use super::{ClientError, IncomingCommand};
use crate::{
    BusMessage,
    native_client::{NativeIncomingReceiver, NodedClient},
};
use std::collections::BTreeMap;
use std::sync::Mutex;
use tokio::sync::mpsc;

/// One registered broker connection. Explicit close provides deterministic
/// teardown; the native client owns framing, correlation and socket lifetime.
pub struct Connection {
    name: String,
    inner: NodedClient,
    incoming: Mutex<Option<ConnectionIncoming>>,
    protected_recipient: bool,
}

pub(crate) enum ConnectionIncoming {
    Legacy(NativeIncomingReceiver),
    #[cfg(unix)]
    Unix(tokio::sync::mpsc::UnboundedReceiver<crate::native_client::VerifiedCommand>),
}

impl ConnectionIncoming {
    pub(crate) async fn recv(&mut self) -> Option<(crate::native_client::BoundedIncomingEvent, Option<crate::native_session::BrokerPrincipal>)> {
        match self {
            Self::Legacy(receiver) => receiver.recv().await.map(|event| (event, None)),
            #[cfg(unix)]
            Self::Unix(receiver) => receiver.recv().await.map(|delivery| {
                let (command, principal) = delivery.into_parts();
                (crate::native_client::BoundedIncomingEvent::Command(command), principal)
            }),
        }
    }
}

#[derive(Clone, Default)]
pub(crate) struct ConnectionOptions {
    #[cfg(unix)]
    pub unix: Option<crate::native_client::UnixConnectOptions>,
    pub provenance: Option<crate::RegisterProvenance>,
    pub commands: Option<Vec<crate::CommandDescriptor>>,
    pub capacity: Option<usize>,
    /// Answer id-bearing requests the bounded lane drops with a correlated
    /// `BUSY` reply. Off by default: a dropped request is then silent.
    pub refuse_overflowed_requests: bool,
}

impl Connection {
    pub async fn connect(name: &str, url: &str) -> Result<Self, ClientError> {
        Self::connect_with_options(name, url, &ConnectionOptions::default()).await
    }

    pub(crate) async fn connect_with_options(
        name: &str,
        url: &str,
        options: &ConnectionOptions,
    ) -> Result<Self, ClientError> {
        #[cfg(unix)]
        if let Some(unix) = &options.unix {
            // Contextual supervision currently uses the same unbounded lane as
            // the default supervisor. Never silently change a bounded policy.
            if options.capacity.is_some() || unix.incoming_capacity.is_some() {
                return Err(ClientError::Native(anyhow::anyhow!("verified supervision requires an unbounded incoming lane")));
            }
            match NodedClient::connect_unix(name, url, unix, options.provenance.clone())
                .await.map_err(unix_client_error)? {
                crate::native_client::UnixConnectOutcome::VerifiedUnix(connection) => {
                    let (inner, incoming) = connection.into_supervised_parts();
                    let crate::native_client::unix::VerifiedIncoming::Unbounded(incoming) = incoming else { unreachable!("unbounded Unix options checked") };
                    if let Some(commands) = &options.commands {
                        inner.set_commands(commands.clone());
                    }
                    return Ok(Self { name: name.to_owned(), inner, incoming: Mutex::new(Some(ConnectionIncoming::Unix(incoming))), protected_recipient: true });
                }
                crate::native_client::UnixConnectOutcome::UnverifiedTcp { client, unix_error } => {
                    if let Some(commands) = &options.commands { client.set_commands(commands.clone()); }
                    tracing::warn!(service = name, error = %unix_error, "native endpoint missing; TCP recipient is unprotected");
                    let incoming = client.take_native_incoming().await.map(ConnectionIncoming::Legacy);
                    return Ok(Self { name: name.to_owned(), inner: client, incoming: Mutex::new(incoming), protected_recipient: false });
                }
            }
        }
        let inner = NodedClient::connect_with_provenance_and_capacity(
            name,
            url,
            options.provenance.clone(),
            options.capacity,
            options.commands.clone(),
            options.refuse_overflowed_requests,
        )
        .await
        .map_err(ClientError::from_native)?;
        let incoming = inner.take_native_incoming().await;
        Ok(Self {
            name: name.to_owned(),
            inner,
            incoming: Mutex::new(incoming.map(ConnectionIncoming::Legacy)),
            protected_recipient: false,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn protected_recipient(&self) -> bool {
        self.protected_recipient
    }
    pub fn is_connected(&self) -> bool {
        self.inner.is_connected()
    }
    pub fn take_incoming(&self) -> Option<mpsc::UnboundedReceiver<IncomingCommand>> {
        let mut incoming = self
            .incoming
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if matches!(&*incoming, Some(ConnectionIncoming::Legacy(NativeIncomingReceiver::Unbounded(_)))) {
            match incoming.take() {
                Some(ConnectionIncoming::Legacy(NativeIncomingReceiver::Unbounded(receiver))) => Some(receiver),
                _ => unreachable!("unbounded receiver checked under lock"),
            }
        } else {
            None
        }
    }

    pub(crate) fn take_native_incoming(&self) -> Option<ConnectionIncoming> {
        self.incoming
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .take()
    }

    pub async fn call_typed(
        &self,
        to: &str,
        command: &str,
        args: serde_json::Value,
    ) -> Result<crate::PortReply, ClientError> {
        self.inner
            .call_typed(to, command, args)
            .await
            .map_err(ClientError::from_native)
    }

    pub async fn send(
        &self,
        to: &str,
        command: &str,
        args: serde_json::Value,
    ) -> Result<(), ClientError> {
        self.inner
            .send(to, command, args)
            .await
            .map_err(ClientError::from_native)
    }

    pub async fn list_services(&self) -> Result<Vec<String>, ClientError> {
        self.inner
            .list_services()
            .await
            .map_err(ClientError::from_native)
    }

    pub async fn call(
        &self,
        to: &str,
        command: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, ClientError> {
        let body = if args.is_null() {
            String::new()
        } else {
            serde_json::to_string(&args)?
        };
        self.call_with_headers(to, command, &BTreeMap::new(), &body)
            .await
    }

    pub async fn call_with_headers(
        &self,
        to: &str,
        command: &str,
        headers: &BTreeMap<String, String>,
        body: &str,
    ) -> Result<serde_json::Value, ClientError> {
        let (rc, body, error) = self
            .call_with_headers_raw(to, command, headers, body)
            .await?;
        if rc >= crate::RC_ERROR {
            let mut response = BusMessage::new().with_body(&body);
            if let Some(error) = error {
                response.set("error", &error);
            }
            return Err(ClientError::Refused {
                rc,
                message: response.error_message(),
            });
        }
        Ok(if body.is_empty() {
            serde_json::Value::Null
        } else {
            serde_json::from_str(&body).unwrap_or(serde_json::Value::String(body))
        })
    }

    pub async fn call_with_headers_raw(
        &self,
        to: &str,
        command: &str,
        headers: &BTreeMap<String, String>,
        body: &str,
    ) -> Result<(u8, String, Option<String>), ClientError> {
        self.inner
            .call_with_headers_raw(to, command, headers, body)
            .await
            .map_err(ClientError::from_native)
    }

    /// [`Self::call_with_headers_raw`] waiting up to `limit` for the response.
    pub async fn call_with_headers_raw_within(
        &self,
        to: &str,
        command: &str,
        headers: &BTreeMap<String, String>,
        body: &str,
        limit: std::time::Duration,
    ) -> Result<(u8, String, Option<String>), ClientError> {
        self.inner
            .call_with_headers_raw_within(to, command, headers, body, limit)
            .await
            .map_err(ClientError::from_native)
    }

    pub async fn send_with_headers(
        &self,
        to: &str,
        command: &str,
        headers: &BTreeMap<String, String>,
        body: &str,
    ) -> Result<(), ClientError> {
        self.inner
            .send_with_headers(to, command, headers, body)
            .await
            .map_err(ClientError::from_native)
    }

    pub async fn respond_parts(
        &self,
        to: &str,
        command: &str,
        id: Option<&str>,
        rc: u8,
        body: &str,
    ) -> Result<(), ClientError> {
        self.inner
            .respond_parts(to, command, id, rc, body)
            .await
            .map_err(ClientError::from_native)
    }
    pub async fn deregister(&self) -> Result<(), ClientError> {
        self.inner
            .deregister()
            .await
            .map_err(ClientError::from_native)
    }
    pub async fn close(&self) {
        self.inner.close().await;
    }
    pub async fn send_raw(&self, message: &BusMessage) -> Result<(), ClientError> {
        self.inner
            .send_raw(message)
            .await
            .map_err(ClientError::from_native)
    }
}

#[cfg(unix)]
fn unix_client_error(error: crate::native_client::ConnectError) -> ClientError {
    // Endpoint diagnostics wrap the protocol error. Preserve registration
    // rejection through that chain so the supervisor's fatal policy is shared.
    let mut cause: &dyn std::error::Error = &error;
    loop {
        if let Some(rejection) = cause.downcast_ref::<crate::RegistrationRejected>() {
            return ClientError::Rejected(rejection.clone());
        }
        match cause.source() {
            Some(source) => cause = source,
            None => break,
        }
    }
    ClientError::Native(anyhow::Error::new(error))
}
