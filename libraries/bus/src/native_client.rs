// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>

//! Native ABP client, including authenticated Unix broker ingress.
//! All requests pass through noded; no application socket transport is provided.

pub(crate) mod bounded;
mod native;
pub mod session;
mod types;
pub(crate) mod unix;

pub use crate::PortReply;
pub use crate::client::{
    ConnState, MAX_INITIAL_ATTEMPTS, RECONNECT_BACKOFF_MAX, SubscriptionRegistry, SupervisedClient,
    SupervisedError, initial_connect_backoff_budget,
};
pub use bounded::{
    BoundedIncomingEvent, BoundedIncomingInjector, BoundedIncomingReceiver,
    bounded_incoming_injector,
};
pub use native::{
    NameCollision, NativeIncomingReceiver, NodedClient, RegistrationRejected,
    RegistrationRejectionKind,
};
pub use types::IncomingCommand;
pub use unix::{
    BrokerAccount, ConnectError, Delivery, ServerPeerCredentials, UnixConnectOptions,
    UnixConnectOutcome, VerifiedCommand, VerifiedConnection,
};
