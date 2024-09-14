// Server side event loop
// (c) 2024 Ross Younger

use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

use crate::cert::Credentials;
use anyhow::{anyhow, Result};
use quinn::crypto::rustls::QuicServerConfig;
use rustls::{server::WebPkiClientVerifier, RootCertStore};
use rustls_pki_types::CertificateDer;
use tracing::{error, info};

pub struct QcpServer<'a> {
    certificate: CertificateDer<'a>,
    endpoint: quinn::Endpoint,
}

impl QcpServer<'_> {
    pub fn new(port: Option<u16>, client_cert: Option<CertificateDer<'_>>) -> Result<Self> {
        let my_credentials = Credentials::generate()?;

        let tls_config = if let Some(cert) = client_cert {
            let mut root_store = RootCertStore::empty();
            root_store.add(cert)?;
            let root_store = Arc::new(root_store);
            let verifier = WebPkiClientVerifier::builder(root_store.clone()).build()?;

            rustls::ServerConfig::builder()
                .with_client_cert_verifier(verifier)
                .with_single_cert(
                    my_credentials.cert_chain(),
                    my_credentials.keypair.clone_key(),
                )?
        } else {
            rustls::ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(
                    my_credentials.cert_chain(),
                    my_credentials.keypair.clone_key(),
                )?
        };
        // N.B.: in ServerConfig docs, max_early_data_size should be set to u32::MAX

        let qsc = QuicServerConfig::try_from(tls_config)?;
        let config = quinn::ServerConfig::with_crypto(Arc::new(qsc));

        let port = port.unwrap_or(0);
        let addr = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port);
        let endpoint = quinn::Endpoint::server(config, addr)?;

        Ok(Self {
            certificate: my_credentials.certificate,
            endpoint,
        })
    }

    pub fn certificate(&self) -> &CertificateDer<'_> {
        &self.certificate
    }

    pub fn local_addr(&self) -> Result<SocketAddr> {
        Ok(self.endpoint.local_addr()?)
    }

    /// Runs the event loop until we receive a termination request.
    /// At present, termination is signalled by closing stdin.
    pub async fn run(&mut self) -> Result<()> {
        /* TODO IWBNI:
            // If stdin is closed, we're done
            let mut stdin = std::pin::pin!(tokio::io::stdin());
            let mut buffer = [0u8; 40];
            let stdin_fut = stdin.read(&mut buffer);
            tokio::pin!(stdin_fut);
        */
        /* TODO IWBNI:
         * Timeout if the client doesn't connect after a while
        let timeout = time::sleep(Duration::from_secs(10)); // XXX what timeout?
        tokio::pin!(timeout);
         */

        while let Some(conn) = self.endpoint.accept().await {
            let fut = QcpServer::handle_connection(conn);
            tokio::spawn(async move {
                if let Err(e) = fut.await {
                    error!("connection failed: {reason}", reason = e.to_string());
                }
            });
            // TBD: Only do this once ?
        }

        self.endpoint.close(0u32.into(), &[]);
        Ok(())
    }

    async fn handle_connection(conn: quinn::Incoming) -> Result<()> {
        let connection = conn.await?;
        info!("Accepting connection from {}", connection.remote_address()); // TEMP
        async {
            let stream = connection.accept_bi().await;
            let stream = match stream {
                Ok(s) => s,
                Err(quinn::ConnectionError::ApplicationClosed { .. }) => {
                    info!("Client connection closed");
                    return Ok(());
                }

                Err(e) => {
                    return Err(e);
                }
            };
            let fut = QcpServer::handle_request(stream);
            tokio::spawn(async move {
                if let Err(e) = fut.await {
                    error!("failed: {reason}", reason = e.to_string());
                }
            });
            // handle it...
            Ok(())
        }
        .await?;
        tokio::spawn(async move {
            // TEMP: Output any datagrams received
            while let Ok(bytes) = connection.read_datagram().await {
                println!("datagram: {:?}", bytes);
            }
        });
        Ok(())
    }

    async fn handle_request(
        (mut send, mut recv): (quinn::SendStream, quinn::RecvStream),
    ) -> Result<()> {
        let req = recv
            .read_to_end(64)
            .await
            .map_err(|e| anyhow!("failed reading client request: {}", e))?;

        // TODO: Handle request
        info!("TODO: Got data from client size {}", req.len());
        // It might be Get or Put; start processing it.
        // If it's Put, it will be followed by data !
        Ok(send.finish()?)
    }

    pub fn stats(&self) -> String {
        // ugh, quinn EndpointStats is not currently exported, I cannot return it as-is
        format!("{:?}", self.endpoint.stats())
    }
}
