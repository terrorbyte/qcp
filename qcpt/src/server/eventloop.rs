// Server side event loop
// (c) 2024 Ross Younger

use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

use crate::cert::Credentials;
use anyhow::Result;
use quinn::crypto::rustls::QuicServerConfig;
use rustls::{server::WebPkiClientVerifier, RootCertStore};
use rustls_pki_types::CertificateDer;

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
}
