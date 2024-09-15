// QCP session wire protocol
// (c) 2024 Ross Younger

/*
 * The session protocol frames a QUIC (Quinn) bidirectional stream.
 * The protocol consists of Command and Response packets defined in schema/session.capnp.
 * Packets are sent using the standard capnp framing.
 *
 * Client -> Server: <initiates stream>
 * C -> S : Command packet
 * S -> C : Response packet
 * (Then they do whatever is appropriate for the command. See the notes in session.capnp.)
 */

pub mod session_capnp {
    include!(concat!(env!("OUT_DIR"), "/session_capnp.rs"));
}

pub const COMMAND_RESPONSE_MAX_LENGTH: u16 = 1024;
