# QCP session protocol wire format definitions
@0xb2af08ed873f6840;

enum Cmd {
    get @0;
    # Retrieves a file. This may fail if the file does not exist or the user doesn't have read permission.
    # If status is OK, server immediately follows by sending: FileHeader; file data; FileTrailer.
    # Then close the stream.
    # If the client needs to abort the transfer, it closes the stream.

    put @1;
    # Sends a file. This may fail for permissions or if the containing directory doesn't exist.
    # If status is OK, client then sends: FileHeader; file data; FileTrailer.
    # Then close the stream.
    # If the server needs to abort the transfer, it sends a TransferAbortInformation as a QUIC datagram,
    # then closes the stream.
}

# A command from client to server.
# Server must respond with a Response before anything else can happen on this connection.
struct Command {
    id @0 : Cmd;
    args : union {
        get@1: GetCmdArgs;
        put@2: PutCmdArgs;
    }

    struct GetCmdArgs {
        filename @0 : Text;
    }
    struct PutCmdArgs {
        filename @0 : Text;
    }
}

# Server's response to a Command
struct Response {
    status @0: Status; # Status code
    message @1: Text; # Human-readable message explaining the situation (may be empty)
}

enum Status {
    ok @0;
    fileNotFound @1;
    incorrectPermissions @2;
    directoryDoesNotExist @3;
    ioError @4;
    diskFull @5;
}

struct FileHeader {
    size @0 : UInt64;
    filename @1 : Text;
}

struct FileTrailer {
    # empty for now, this will probably have a checksum later
}

# Information about why a transfer was aborted. Intended to be displayed to the user.
struct TransferAbortInformation {
    filename @0 : Text; # The file we were transferring (our name for it)
    status @1 : Status;
    message @2 : Text; # Human-readable message explaining the situation
}
