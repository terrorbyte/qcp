// QCP transport - server side CLI
// (c) 2024 Ross Younger

use anyhow::Context;
use rustls_pki_types::CertificateDer;

use crate::server::message::ClientMessage;

use super::{QcpServer, ServerMessage};

/// Runs the server side transport
#[derive(Debug, clap::Args)]
pub struct ServerArgs {
    /// The UDP port to bind to (e.g. 51000)
    /// If not specified, a random port will be chosen.
    // TODO: support a range of ports
    #[arg(short = 'p', long)]
    port: Option<u16>,
    /// Disables checking of client TLS certificate. Use with caution!
    /// If this option is not given, the server will expect to be sent a json-encoded DER-encoded self-signed client certificate on stdin.
    #[arg(long, action)]
    insecure: bool,
}

/// Implementation of 'server' mode
pub fn server(args: &ServerArgs) -> anyhow::Result<()> {
    let client_cert: Option<CertificateDer<'static>> = if args.insecure {
        // Insecure mode: no certificate required
        None
    } else {
        let mut buffer = String::new();
        std::io::stdin()
            .read_line(&mut buffer)
            .with_context(|| "failed to read client message")?;
        let msg = serde_json::from_str::<ClientMessage>(&buffer)
            .with_context(|| "failed to read client message")?;
        let cert = CertificateDer::from_slice(&msg.cert).into_owned();
        Some(cert)
    };

    let event_loop = QcpServer::new(args.port, client_cert)?;
    let message: ServerMessage = (&event_loop).try_into()?;
    // TODO: There's probably a tidier way to format the messages, but serde will do for now.
    println!("{}", serde_json::to_string(&message)?);

    // TODO: run the event loop...
    Ok(())
}
