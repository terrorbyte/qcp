// qcp server event loop
// (c) 2024 Ross Younger

use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;

use crate::cert::Credentials;
use crate::protocol;
use crate::protocol::control::{control_capnp, ClientMessage};

use capnp::message::ReaderOptions;
use quinn::crypto::rustls::QuicServerConfig;
use quinn::rustls::server::WebPkiClientVerifier;
use quinn::rustls::{self, RootCertStore};
use rustls_pki_types::CertificateDer;
use tokio::io::Stdin;
use tokio_util::compat::{Compat, TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
use tracing::{debug, info, trace};

/// Main entrypoint
#[tokio::main]
pub async fn server_main() -> anyhow::Result<()> {
    let credentials = crate::cert::Credentials::generate()?;

    print!("{}", protocol::control::BANNER);
    let stdin = tokio::io::stdin().compat();
    let stdout = tokio::io::stdout().compat_write();
    let client_message = read_client_message(stdin).await.unwrap_or_else(|e| {
        // try to be helpful if there's a human reading
        eprintln!("ERROR: This program expects a binary data packet on stdin.\n{e}");
        std::process::exit(1);
    });
    trace!("got client message length {}", client_message.cert.len());

    // TODO: Allow port to be specified
    let endpoint = create_endpoint(&credentials, client_message.cert.into())?;
    info!("bound endpoint to port {}", endpoint.local_addr()?.port());
    {
        let mut msg = ::capnp::message::Builder::new_default();
        let mut server_msg = msg.init_root::<control_capnp::server_message::Builder>();
        server_msg.set_cert(&credentials.certificate);
        server_msg.set_port(endpoint.local_addr()?.port());
        trace!("sending server message");
        capnp_futures::serialize::write_message(stdout, msg).await?;
    }

    // NEXT: Control channel setup
    Ok(())
}

async fn read_client_message(stdin: Compat<Stdin>) -> anyhow::Result<ClientMessage> {
    debug!("waiting for client message");
    let reader = capnp_futures::serialize::read_message(stdin, ReaderOptions::new()).await?;
    let msg_reader: control_capnp::client_message::Reader = reader.get_root()?;
    let cert = Vec::<u8>::from(msg_reader.get_cert()?);
    Ok(ClientMessage { cert })
}

fn create_endpoint(
    credentials: &Credentials,
    client_cert: CertificateDer<'_>,
) -> anyhow::Result<quinn::Endpoint> {
    let mut root_store = RootCertStore::empty();
    root_store.add(client_cert)?;
    let root_store = Arc::new(root_store);
    let verifier = WebPkiClientVerifier::builder(root_store.clone()).build()?;
    let tls_config = rustls::ServerConfig::builder()
        .with_client_cert_verifier(verifier)
        .with_single_cert(credentials.cert_chain(), credentials.keypair.clone_key())?;

    // N.B.: in ServerConfig docs, max_early_data_size should be set to u32::MAX

    let qsc = QuicServerConfig::try_from(tls_config)?;
    let config = quinn::ServerConfig::with_crypto(Arc::new(qsc));

    // TODO let caller specify port
    let addr = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0);
    let endpoint = quinn::Endpoint::server(config, addr)?;

    Ok(endpoint)
}
