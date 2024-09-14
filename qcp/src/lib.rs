// QCP library routines
// (c) 2024 Ross Younger

pub mod cert;
pub mod cli;
pub mod protocol;
pub mod styles;

/// Build-time info (from `built`)
pub mod build_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}
