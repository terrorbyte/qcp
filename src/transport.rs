// QUIC transport configuration
// (c) 2024 Ross Younger

use std::{sync::Arc, time::Duration};

use anyhow::Result;
use quinn::{congestion::CubicConfig, TransportConfig};

/// Network buffer size (hard-wired)
pub const SEND_BUFFER_SIZE: usize = 1_048_576;

/// Computes the theoretical receive window for a given bandwidth/RTT configuration
#[must_use]
pub fn receive_window_for(bandwidth_limit: u64, rtt_ms: u16) -> u64 {
    bandwidth_limit * u64::from(rtt_ms) / 1000
}

/// In some cases the theoretical receive window is less than the system default.
/// In such a case, don't suggest setting it smaller, that would be silly.
pub fn practical_receive_window_for(bandwidth_limit: u64, rtt_ms: u16) -> Result<u64> {
    use std::net::UdpSocket;
    let theoretical = receive_window_for(bandwidth_limit, rtt_ms);
    let sock = UdpSocket::bind("0.0.0.0:0")?;
    let current = crate::os::os::get_recvbuf(&sock)? as u64;
    Ok(std::cmp::max(theoretical, current))
}

/// Creates a config for `quinn::TransportConfig`
pub fn config_factory(
    bandwidth_limit: u64,
    rtt_ms: u16,
    initial_window: u64,
) -> Result<Arc<TransportConfig>> {
    let rtt = Duration::from_millis(u64::from(rtt_ms));
    #[allow(clippy::cast_possible_truncation)]
    let receive_window =
        practical_receive_window_for(bandwidth_limit, rtt_ms)?.clamp(0, u64::from(u32::MAX)) as u32;

    let mut config = TransportConfig::default();
    let _ = config
        .max_concurrent_bidi_streams(1u8.into())
        .max_concurrent_uni_streams(0u8.into())
        .initial_rtt(rtt)
        .stream_receive_window(receive_window.into())
        .send_window((receive_window * 8).into())
        .datagram_receive_buffer_size(Some(receive_window as usize))
        .datagram_send_buffer_size(SEND_BUFFER_SIZE);

    let mut cubic = CubicConfig::default();
    let _ = cubic.initial_window(initial_window);
    let _ = config.congestion_controller_factory(Arc::new(cubic));

    Ok(config.into())
}
