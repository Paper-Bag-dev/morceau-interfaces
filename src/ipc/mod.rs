//! The protocol between the CLI and the host: request and reply types (protocol), the server (server) over the transport (transport: a local
//! socket by default, localhost TCP with MORC_TRANSPORT=tcp), the events the host loop handles (events).

pub mod events;
pub mod protocol;
pub mod server;
pub mod transport;

