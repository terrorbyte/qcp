mod cert;
/// (c) 2024 Ross Younger
pub mod cli;
mod server;
mod styles;

/// Build-time info (from `built`)
pub mod build_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}
