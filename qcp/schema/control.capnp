# QCP control protocol wire format definitions
@0xeb4bca415866a714;

struct ClientMessage {
    cert @0: Data; # Client's self-signed certificate (DER)
}

struct ServerMessage {
    port @0: UInt16; # UDP port the server has bound to
    cert @1: Data; # Server's self-signed certificate (DER)
}
