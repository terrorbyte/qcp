// QCP session wire protocol
// (c) 2024 Ross Younger

/*
 * The session protocol is used at the start of a QUIC (Quinn) bidirectional stream.
 * The protocol consists of Command and Response packets defined in schema/session.capnp.
 *
 * On the wire these look like:
 *   <length> <capnproto-encoded-struct>
 *
 * Integers are encoded in NETWORK BYTE ORDER.
 */

pub mod session_capnp {
    include!(concat!(env!("OUT_DIR"), "/session_capnp.rs"));
}

pub const COMMAND_RESPONSE_MAX_LENGTH: u16 = 1024;

pub fn decode_length(raw: &[u8]) -> u16 {
    let len_netorder: u16 = ((raw[0] as u16) << 8) | raw[1] as u16;
    u16::from_be(len_netorder)
}

pub fn encode_length(len: u16) -> Vec<u8> {
    let len_netorder = len.to_be();
    vec![(len_netorder >> 8) as u8, (len_netorder & 0xff) as u8]
}
