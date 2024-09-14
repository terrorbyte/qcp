# QCP session protocol wire format definitions
@0xb2af08ed873f6840;

struct Command {
    args : union {
        get @0 : GetXArgs;
        put @1 : PutXArgs;
    }
    struct GetXArgs {
        filename @0 : Text;
    }
    struct PutXArgs {
        filename @1 : Text;
        size @0 : UInt64;
    }
}

struct Response {
    status @0: UInt32;
    message @1: Text;
}
