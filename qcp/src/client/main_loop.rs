// qcp client event loop
// (c) 2024 Ross Younger

use crate::protocol::control::{control_capnp, ServerMessage};
use crate::protocol::session::session_capnp::Status;
use crate::protocol::session::{session_capnp, Response};
use crate::protocol::StreamPair;
use crate::{cert::Credentials, protocol};

use anyhow::{Context, Result};
use capnp::message::ReaderOptions;
use quinn::crypto::rustls::QuicClientConfig;
use quinn::rustls;
use rustls::RootCertStore;
use rustls_pki_types::CertificateDer;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::process::{Child, Command};
use tokio::{self, io::AsyncReadExt, time::timeout, time::Duration};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
use tracing::{debug, error, info, span, trace, trace_span, warn, Level};

use super::ClientArgs;

const CONNECTION_TIMEOUT: Duration = Duration::from_secs(5);

/// Main CLI entrypoint
#[tokio::main]
pub async fn client_main(args: &ClientArgs) -> anyhow::Result<()> {
    let server_hostname = "localhost"; // TEMP; this will come from parsed args

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

    let server_address = async_dns::lookup(server_hostname)
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

    let server_address_port = match server_address.ip_address {
        std::net::IpAddr::V4(ip) => SocketAddrV4::new(ip, server_message.port).into(),
        std::net::IpAddr::V6(ip) => SocketAddrV6::new(ip, server_message.port, 0, 0).into(),
    };

    let endpoint = create_endpoint(
        &credentials,
        server_message.cert.into(),
        &server_address_port,
    )?;

    trace!("Connecting to {server_address_port:?}");
    trace!("Local connection address is {:?}", endpoint.local_addr()?);

    let connection_fut = endpoint.connect(server_address_port, &server_message.name)?;
    let timeout_fut = tokio::time::sleep(CONNECTION_TIMEOUT);
    tokio::pin!(connection_fut, timeout_fut);

    let connection = tokio::select! {
        _ = timeout_fut => {
            warn!("UDP connection to QUIC endpoint timed out");
            endpoint.close(0u32.into(), &[0u8;0]);
            None
        },
        c = &mut connection_fut => {
            match c {
                Ok(conn) => Some(conn),
                Err(e) => {
                    warn!("Error connecting: {e}");
                    None
                },
            }
        },
    };

    if let Some(ref c) = connection {
        // Hard wire a single GET for now.
        // TODO this will become spawned ?
        let mut stream: StreamPair = c.open_bi().await?.into();
        do_get(&mut stream, "testfile").await?;
    }

    info!("shutting down");
    server_input.into_inner().shutdown().await?;
    let _ = connection.map(|c| c.close(0u8.into(), "".as_bytes()));
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
    server_addr: &SocketAddr,
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
    let addr: SocketAddr = match server_addr {
        SocketAddr::V4(_) => SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0).into(),
        SocketAddr::V6(_) => SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, 0, 0, 0).into(),
    };
    let mut endpoint = quinn::Endpoint::client(addr)?;
    endpoint.set_default_client_config(config);

    Ok(endpoint)
}

async fn do_get(stream: &mut StreamPair, filename: &str) -> Result<()> {
    let span = span!(Level::TRACE, "do_get");
    let _guard = span.enter();

    let mut msg = ::capnp::message::Builder::new_default();
    {
        let command_msg = msg.init_root::<session_capnp::command::Builder>();
        let mut args = command_msg.init_args().init_get();
        args.set_filename(filename);
    }
    trace!("send GET");
    capnp_futures::serialize::write_message(&mut stream.send, msg).await?;

    // TODO protocol timeout?
    trace!("await response");
    let response = Response::read(&mut stream.recv).await?;
    trace!("GET {response:?}");

    if response.status != Status::Ok {
        let msg_msg = match response.message {
            Some(s) => format!("with message {s}"),
            None => "".to_string(),
        };
        anyhow::bail!(format!(
            "GET {filename} failed: server returned status {status:?} {msg_msg}",
            status = response.status
        ));
    }

    // Error handling: what happens on return?

    todo!();
}
