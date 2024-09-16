// qcp client event loop
// (c) 2024 Ross Younger

use crate::protocol::control::{control_capnp, ServerMessage};
use crate::{cert::Credentials, protocol};

use anyhow::{Context, Result};
use capnp::message::ReaderOptions;
use quinn::crypto::rustls::QuicClientConfig;
use quinn::rustls;
use rustls::RootCertStore;
use rustls_pki_types::CertificateDer;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::process::{Child, Command};
use tokio::{self, io::AsyncReadExt, time::timeout, time::Duration};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
use tracing::{debug, error, info, span, trace, trace_span, Level};

use super::ClientArgs;

/// Main CLI entrypoint
#[tokio::main]
pub async fn client_main(args: &ClientArgs) -> anyhow::Result<()> {
    let server_hostname = "127.0.0.1"; // TEMP; this will come from parsed args

    let span = trace_span!("CLIENT");
    let _guard = span.enter();
    let credentials = crate::cert::Credentials::generate()?;

    info!("connecting to remote");
    let mut server = launch_server()?;

    wait_for_banner(&mut server, args.timeout).await?;

    let mut server_input = server.stdin.take().unwrap().compat_write();
    {
        let mut msg = ::capnp::message::Builder::new_default();
        let mut client_msg = msg.init_root::<control_capnp::client_message::Builder>();
        client_msg.set_cert(&credentials.certificate);
        trace!("sending client message");
        capnp_futures::serialize::write_message(&mut server_input, msg).await?;
    }

    let server_output = server.stdout.take().unwrap();
    let server_message = {
        trace!("waiting for server message");
        let reader =
            capnp_futures::serialize::read_message(server_output.compat(), ReaderOptions::new())
                .await?;
        let msg_reader: control_capnp::server_message::Reader = reader.get_root()?;
        let cert = Vec::<u8>::from(msg_reader.get_cert()?);
        let port = msg_reader.get_port();
        let name = msg_reader.get_name()?.to_string()?;
        ServerMessage { port, cert, name }
    };
    debug!(
        "Got server message; cert length {}, port {}, hostname {}",
        server_message.cert.len(),
        server_message.port,
        server_message.name
    );

    let server_host_port = format!("{server_hostname}:{}", server_message.port);
    let server_address = tokio::net::lookup_host(server_host_port)
        .await
        .map_err(|e| {
            error!("host name lookup failed1");
            e
        })?
        .next()
        .ok_or_else(|| {
            error!("host name lookup failed2");
            anyhow::anyhow!("host name lookup failed")
        })?;
    let endpoint = create_endpoint(&credentials, server_message.cert.into())?;
    trace!("Connecting to {server_address:?}");
    trace!("Local connection address is {:?}", endpoint.local_addr()?);
    // TODO timeout?
    let connection = endpoint
        .connect(server_address, &server_message.name)?
        .await?;

    connection.send_datagram("hello".as_bytes().into())?;

    info!("Work in progress...");
    // TODO: Send some commands
    let _temp_stream = connection.open_bi();

    tokio::time::sleep(Duration::from_secs(1)).await;

    info!("shutting down");
    server_input.into_inner().shutdown().await?;
    connection.close(0u8.into(), "".as_bytes());
    endpoint.wait_idle().await;
    server.wait().await?;
    Ok(())
}

fn launch_server() -> Result<Child> {
    let server = Command::new("qcpt")
        .args(["--debug"]) // TEMP; TODO make this configurable
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit()) // TODO: pipe this more nicely, output on error?
        .kill_on_drop(true)
        .spawn()
        .context("Could not launch remote server")?;
    Ok(server)
}

async fn wait_for_banner(server: &mut Child, timeout_s: u16) -> Result<()> {
    use protocol::control::BANNER;
    let channel = server.stdout.as_mut().expect("missing server stdout");
    let mut buf = [0u8; BANNER.len()];
    let mut reader = channel.take(buf.len() as u64);
    let n_fut = reader.read(&mut buf);

    let _n = timeout(Duration::from_secs(timeout_s.into()), n_fut)
        .await
        .with_context(|| "timed out reading server banner")??;

    let read_banner = std::str::from_utf8(&buf).with_context(|| "bad server banner")?;
    anyhow::ensure!(BANNER == read_banner, "banner not as expected");
    Ok(())
}

/// Creates the client endpoint:
/// `credentials` are generated locally.
/// `server_cert` comes from the control channel server message.
/// `destination` is the server's address (port from the control channel server message).
pub fn create_endpoint(
    credentials: &Credentials,
    server_cert: CertificateDer<'_>,
) -> Result<quinn::Endpoint> {
    let span = span!(Level::TRACE, "create_endpoint");
    let _guard = span.enter();
    trace!("Set up root store");
    let mut root_store = RootCertStore::empty();
    root_store.add(server_cert).map_err(|e| {
        error!("{e}");
        e
    })?;
    let root_store = Arc::new(root_store);

    trace!("create tls_config");
    let tls_config = Arc::new(
        rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_client_auth_cert(credentials.cert_chain(), credentials.keypair.clone_key())?,
    );

    trace!("create client config");
    let qcc = Arc::new(QuicClientConfig::try_from(tls_config)?);
    let config = quinn::ClientConfig::new(qcc);

    trace!("create endpoint");
    let addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0);
    let mut endpoint = quinn::Endpoint::client(addr.into())?;
    endpoint.set_default_client_config(config);

    Ok(endpoint)
}
