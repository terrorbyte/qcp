/// Local X509 certificate management
/// (c) 2024 Ross Younger
use anyhow::Result;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};

/// In-memory represenatation of X509 credentials (for TLS)
pub struct Credentials {
    pub certificate: CertificateDer<'static>,
    pub keypair: PrivateKeyDer<'static>,
}

/*
fn dump(creds: &rcgen::CertifiedKey) {
    println!("{}{}\n", creds.cert.pem(), creds.key_pair.serialize_pem());
}
*/

impl Credentials {
    pub fn generate() -> Result<Self> {
        let hostname = gethostname::gethostname()
            .into_string()
            .unwrap_or("unknown.host.invalid".to_string());
        let raw = rcgen::generate_simple_self_signed([hostname])?;
        Ok(Credentials {
            certificate: raw.cert.der().clone(),
            keypair: rustls_pki_types::PrivateKeyDer::Pkcs8(raw.key_pair.serialize_der().into()),
        })
    }

    pub fn cert_chain(&self) -> Vec<CertificateDer<'static>> {
        vec![self.certificate.clone()]
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn generate_works() {
        super::Credentials::generate().unwrap();
    }
}
