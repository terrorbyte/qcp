//! Configuration structure
// (c) 2024 Ross Younger

use std::time::Duration;

use clap::Parser;
use human_repr::{HumanCount as _, HumanDuration as _};
use serde::{Deserialize, Serialize};
use struct_field_names_as_array::FieldNamesAsSlice;

use crate::{
    transport::CongestionControllerType,
    util::{
        derive_deftly_template_Optionalify, humanu64::HumanU64, AddressFamily, PortRange,
        TimeFormat,
    },
};

use derive_deftly::Deftly;

/// The set of configurable options supported by qcp.
///
/// **Note:** The implementation of `default()` for this struct returns qcp's hard-wired configuration defaults.
///
/// This structure uses the [Optionalify](derive_deftly_template_Optionalify) deftly macro to automatically
/// define the `Configuration_Optional` struct, which is the same but has all members of type `Option<whatever>`.
/// This is the magic that lets us use the same underlying struct for CLI and saved configuration files:
/// the CLI uses the `_Optional` version , with everything defaulting to `None`.
/// The result is that wherever the user does not provide a value, values read from lower priority sources
/// (configuration files and system defaults) obtain.
///
// Maintainer note: None of the members of this struct should be Option<anything>. That leads to sunspots in the CLI and strange warts (Some(Some(foo))).
#[derive(Deftly)]
#[derive_deftly(Optionalify)]
#[deftly(visibility = "pub(crate)")]
#[derive(Debug, Clone, PartialEq, Eq, Parser, Deserialize, Serialize, FieldNamesAsSlice)]
pub struct Configuration {
    // TRANSPORT PARAMETERS ============================================================================
    // System bandwidth, UDP ports, timeout.
    /// The maximum network bandwidth we expect receiving data FROM the remote system.
    /// [default: 12500k]
    ///
    /// This may be specified directly as a number of bytes, or as an SI quantity
    /// like `10M` or `256k`. **Note that this is described in BYTES, not bits**;
    /// if (for example) you expect to fill a 1Gbit ethernet connection,
    /// 125M might be a suitable setting.
    #[arg(short('b'), long, alias("rx-bw"), help_heading("Network tuning"), display_order(1), value_name="bytes", value_parser=clap::value_parser!(HumanU64))]
    pub rx: HumanU64,
    /// The maximum network bandwidth we expect sending data TO the remote system,
    /// if it is different from the bandwidth FROM the system.
    ///
    /// (For example, when you are connected via an asymmetric last-mile DSL or fibre profile.)
    ///
    /// If not specified or 0, uses the value of `rx`.
    #[arg(short('B'), long, alias("tx-bw"), help_heading("Network tuning"), display_order(1), value_name="bytes", value_parser=clap::value_parser!(HumanU64))]
    pub tx: HumanU64,

    /// The expected network Round Trip time to the target system, in milliseconds.
    /// [default: 300]
    #[arg(
        short('r'),
        long,
        help_heading("Network tuning"),
        display_order(10),
        value_name("ms")
    )]
    pub rtt: u16,

    /// Specifies the congestion control algorithm to use.
    /// [default: cubic]
    #[arg(
        long,
        action,
        value_name = "alg",
        help_heading("Advanced network tuning"),
        display_order(0)
    )]
    #[clap(value_enum)]
    pub congestion: CongestionControllerType,

    /// _(Network wizards only!)_
    /// The initial value for the sending congestion control window.
    /// If unspecified, the active congestion control algorithm decides.
    ///
    /// _Setting this value too high reduces performance!_
    #[arg(
        long,
        help_heading("Advanced network tuning"),
        value_name = "bytes",
        display_order(0)
    )]
    pub initial_congestion_window: u64,

    /// Uses the given UDP port or range on the local endpoint.
    /// This can be useful when there is a firewall between the endpoints.
    ///
    /// For example: `12345`, `20000-20100`
    ///
    /// If unspecified, uses any available UDP port.
    #[arg(
        short = 'p',
        long,
        value_name("M-N"),
        help_heading("Connection"),
        display_order(0)
    )]
    pub port: PortRange,

    /// Connection timeout for the QUIC endpoints [seconds; default 5]
    ///
    /// This needs to be long enough for your network connection, but short enough to provide
    /// a timely indication that UDP may be blocked.
    #[arg(
        short,
        long,
        value_name("sec"),
        help_heading("Connection"),
        display_order(0)
    )]
    pub timeout: u16,

    // CLIENT OPTIONS ==================================================================================
    /// Forces use of a particular IP version when connecting to the remote. [default: any]
    ///
    // (see also [CliArgs::ipv4_alias__] and [CliArgs::ipv6_alias__])
    #[arg(
        long,
        help_heading("Connection"),
        group("ip address"),
        display_order(0)
    )]
    pub address_family: AddressFamily,

    /// Specifies the ssh client program to use [default: `ssh`]
    #[arg(long, help_heading("Connection"), display_order(0))]
    pub ssh: String,

    /// Provides an additional option or argument to pass to the ssh client. [default: none]
    ///
    /// **On the command line** you must repeat `-S` for each argument.
    /// For example, to pass `-i /dev/null` to ssh, specify: `-S -i -S /dev/null`
    #[arg(
        short = 'S',
        action,
        value_name("ssh-option"),
        allow_hyphen_values(true),
        help_heading("Connection"),
        display_order(0)
    )]
    pub ssh_options: Vec<String>,

    /// Uses the given UDP port or range on the remote endpoint.
    /// This can be useful when there is a firewall between the endpoints.
    ///
    /// For example: `12345`, `20000-20100`
    ///
    /// If unspecified, uses any available UDP port.
    #[arg(
        short = 'P',
        long,
        value_name("M-N"),
        help_heading("Connection"),
        display_order(0)
    )]
    pub remote_port: PortRange,

    /// Specifies the time format to use when printing messages to the console or to file
    /// [default: local]
    #[arg(
        short = 'T',
        long,
        value_name("FORMAT"),
        help_heading("Output"),
        next_line_help(true),
        display_order(0)
    )]
    pub time_format: TimeFormat,

    /// Alternative ssh config file(s)
    ///
    /// By default, qcp reads your user and system ssh config files to look for Hostname aliases.
    /// In some cases the logic in qcp may not read them successfully; this is an escape hatch,
    /// allowing you to specify one or more alternative files to read instead (which may be empty,
    /// nonexistent or /dev/null).
    ///
    /// This option is really intended to be used in a qcp configuration file.
    /// On the command line, you can repeat `--ssh-config file` as many times as needed.
    #[arg(long, value_name("FILE"), help_heading("Connection"), display_order(0))]
    pub ssh_config: Vec<String>,
}

impl Configuration {
    /// Computes the theoretical bandwidth-delay product for outbound data
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn bandwidth_delay_product_tx(&self) -> u64 {
        self.tx() * u64::from(self.rtt) / 1000
    }
    /// Computes the theoretical bandwidth-delay product for inbound data
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn bandwidth_delay_product_rx(&self) -> u64 {
        self.rx() * u64::from(self.rtt) / 1000
    }
    #[must_use]
    /// Receive bandwidth (accessor)
    pub fn rx(&self) -> u64 {
        *self.rx
    }
    #[must_use]
    /// Transmit bandwidth (accessor)
    pub fn tx(&self) -> u64 {
        match *self.tx {
            0 => self.rx(),
            tx => tx,
        }
    }
    /// RTT accessor as Duration
    #[must_use]
    pub fn rtt_duration(&self) -> Duration {
        Duration::from_millis(u64::from(self.rtt))
    }

    /// UDP kernel sending buffer size to use
    #[must_use]
    pub fn send_buffer() -> u64 {
        // UDP kernel buffers of 2MB have proven sufficient to get close to line speed on a 300Mbit downlink with 300ms RTT.
        2_097_152
    }
    /// UDP kernel receive buffer size to use
    #[must_use]
    pub fn recv_buffer() -> u64 {
        // UDP kernel buffers of 2MB have proven sufficient to get close to line speed on a 300Mbit downlink with 300ms RTT.
        2_097_152
    }

    /// QUIC receive window
    #[must_use]
    pub fn recv_window(&self) -> u64 {
        // The theoretical in-flight limit appears to be sufficient
        self.bandwidth_delay_product_rx()
    }

    /// QUIC send window
    #[must_use]
    pub fn send_window(&self) -> u64 {
        // There might be random added latency en route, so provide for a larger send window than theoretical.
        2 * self.bandwidth_delay_product_tx()
    }

    /// Accessor for `timeout`, as a Duration
    #[must_use]
    pub fn timeout_duration(&self) -> Duration {
        Duration::from_secs(self.timeout.into())
    }

    /// Formats the transport-related options for display
    #[must_use]
    pub fn format_transport_config(&self) -> String {
        let iwind = match self.initial_congestion_window {
            0 => "<default>".to_string(),
            s => s.human_count_bytes().to_string(),
        };
        let (tx, rx) = (self.tx(), self.rx());
        format!(
            "rx {rx} ({rxbits}), tx {tx} ({txbits}), rtt {rtt}, congestion algorithm {congestion:?} with initial window {iwind}",
            tx = tx.human_count_bytes(),
            txbits = (tx * 8).human_count("bit"),
            rx = rx.human_count_bytes(),
            rxbits = (rx * 8).human_count("bit"),
            rtt = self.rtt_duration().human_duration(),
            congestion = self.congestion,
        )
    }
}

impl Default for Configuration {
    /// **(Unusual!)**
    /// Returns qcp's hard-wired configuration defaults.
    fn default() -> Self {
        Self {
            // Transport
            rx: 12_500_000.into(),
            tx: 0.into(),
            rtt: 300,
            congestion: CongestionControllerType::Cubic,
            initial_congestion_window: 0,
            port: PortRange::default(),
            timeout: 5,

            // Client
            address_family: AddressFamily::Any,
            ssh: "ssh".into(),
            ssh_options: vec![],
            remote_port: PortRange::default(),
            time_format: TimeFormat::Local,
            ssh_config: Vec::new(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::Configuration;

    #[test]
    fn flattened() {
        let v = Configuration::default();
        let j = serde_json::to_string(&v).unwrap();
        let d = json::parse(&j).unwrap();
        assert!(!d.has_key("bw"));
        assert!(d.has_key("rtt"));
    }
}
