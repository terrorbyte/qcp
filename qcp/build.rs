#![allow(missing_docs)]

fn main() {
    capnpc::CompilerCommand::new()
        .src_prefix("schema")
        .file("schema/session.capnp")
        .default_parent_module(vec!["protocol::session".into()])
        .run()
        .expect("session protocol compiler command");

    capnpc::CompilerCommand::new()
        .src_prefix("schema")
        .file("schema/control.capnp")
        .default_parent_module(vec!["protocol::control".into()])
        .run()
        .expect("control protocol compiler command");
}
