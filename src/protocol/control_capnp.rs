//! Low-level protocol structures and serialisation, autogenerated from `control.capnp`
// (c) 2024 Ross Younger
#![allow(
    missing_debug_implementations,
    single_use_lifetimes,
    unreachable_pub,
    missing_docs,
    clippy::expl_impl_clone_on_copy,
    clippy::match_same_arms,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::needless_lifetimes,
    clippy::semicolon_if_nothing_returned,
    clippy::uninlined_format_args,
    clippy::used_underscore_binding
)]

include!(concat!(env!("OUT_DIR"), "/control_capnp.rs"));

use client_message::ConnectionType;
use std::net::IpAddr;

impl From<IpAddr> for ConnectionType {
    fn from(value: IpAddr) -> Self {
        match value {
            IpAddr::V4(_) => ConnectionType::Ipv4,
            IpAddr::V6(_) => ConnectionType::Ipv6,
        }
    }
}
