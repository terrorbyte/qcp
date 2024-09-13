// Server side message output serialisation
// (c) 2024 Ross Younger

use serde::{Deserialize, Serialize};

use super::QcpServer;

/// Message sent by the client to the server, before the server starts up
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientMessage {
    pub cert: Vec<u8>,
}

/// Message emitted by the server on startup
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerMessage {
    port: u16,
    cert: Vec<u8>,
}

impl TryFrom<&QcpServer<'_>> for ServerMessage {
    type Error = anyhow::Error;

    fn try_from(srv: &QcpServer<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            port: srv.local_addr()?.port(),
            cert: srv.certificate().to_vec(),
        })
    }
}
