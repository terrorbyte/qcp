pub mod protocol;

/// Build-time info (from `built`)
pub mod build_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}
