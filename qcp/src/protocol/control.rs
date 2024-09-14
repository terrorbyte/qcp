// QCP control protocol
// (c) 2024 Ross Younger

/*
 * The control protocol is data passed between the local qcp client process and the remote qcp server process
 * before establishing the QUIC connection.
 * The two processes are usually connected by stdio, via ssh.
 *
 * The protocol looks like this:
 *   (Client creates remote server process)
 *   Server -> Client: Banner
 *   C -> S: `ClientMessage`
 *   S -> C: `ServerMessage`
 * The client then establishes a QUIC connection to the server, on the port given in the `ServerMessage`.
 * The client then opens one or more bidirectional QUIC streams ('sessions') on that connection.
 * See the session protocol for what happens there.
 *
 * On the wire the Client and Server messages are sent using capnproto with standard framing.
 */

pub mod control_capnp {
    include!(concat!(env!("OUT_DIR"), "/control_capnp.rs"));
}

pub const BANNER: &str = "qcpcs\n";

/// Rust type analogue to the capnproto struct
pub struct ClientMessage {
    pub cert: Vec<u8>,
}

/// Rust type analogue to the capnproto struct
pub struct ServerMessage {
    pub port: u16,
    pub cert: Vec<u8>,
}

#[cfg(test)]
mod tests {

    // These tests are really only exercising capnp, proving that we know how to drive it correctly.

    use super::{control_capnp, ClientMessage, ServerMessage};
    use anyhow::Result;
    use capnp::{message::ReaderOptions, serialize};

    pub fn encode_client(cert: &[u8]) -> Vec<u8> {
        let mut msg = ::capnp::message::Builder::new_default();
        let mut client_msg = msg.init_root::<control_capnp::client_message::Builder>();
        client_msg.set_cert(cert);
        serialize::write_message_to_words(&msg)
    }

    pub fn decode_client(wire: &[u8]) -> Result<ClientMessage> {
        use control_capnp::client_message;
        let reader = serialize::read_message(wire, ReaderOptions::new())?;
        let cert_reader: client_message::Reader = reader.get_root()?;
        let cert = Vec::<u8>::from(cert_reader.get_cert()?);
        Ok(ClientMessage { cert })
    }
    pub fn encode_server(port: u16, cert: &[u8]) -> Vec<u8> {
        let mut msg = ::capnp::message::Builder::new_default();
        let mut server_msg = msg.init_root::<control_capnp::server_message::Builder>();
        server_msg.set_port(port);
        server_msg.set_cert(cert);
        serialize::write_message_to_words(&msg)
    }
    pub fn decode_server(wire: &[u8]) -> Result<ServerMessage> {
        use control_capnp::server_message;
        let reader = serialize::read_message(wire, ReaderOptions::new())?;
        let msg_reader: server_message::Reader = reader.get_root()?;
        let cert = Vec::<u8>::from(msg_reader.get_cert()?);
        let port = msg_reader.get_port();
        Ok(ServerMessage { port, cert })
    }

    #[test]
    fn client_pairwise_alloc() -> Result<()> {
        // A single round trip test: encode, decode, check.

        // Random certificate data of a given length
        let cert = {
            let mut temp = Vec::<u8>::with_capacity(128);
            temp.fill_with(|| fastrand::u8(0..255));
            temp
        };

        let wire = encode_client(&cert);
        let decoded = decode_client(&wire)?;
        assert_eq!(cert, decoded.cert);
        Ok(())
    }

    #[test]
    fn server_pairwise_alloc() -> Result<()> {
        // A single round trip test: encode, decode, check.

        // Random certificate data of a given length
        let cert = {
            let mut temp = Vec::<u8>::with_capacity(128);
            temp.fill_with(|| fastrand::u8(0..255));
            temp
        };
        let port = fastrand::u16(1..65535);

        let wire = encode_server(port, &cert);
        let decoded = decode_server(&wire)?;
        assert_eq!(cert, decoded.cert);
        assert_eq!(port, decoded.port);
        Ok(())
    }
}
