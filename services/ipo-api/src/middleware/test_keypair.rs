//! Process-wide RSA keypair generated on demand for middleware tests.
//!
//! Generating a 2048-bit RSA key is slow (hundreds of milliseconds) so the
//! keypair is produced once per test process via [`OnceLock`] and reused
//! across modules. This keeps no private key material in version control
//! while staying fast enough for the cargo-test loop.

use std::sync::OnceLock;

use rsa::pkcs1::EncodeRsaPublicKey;
use rsa::pkcs8::{EncodePrivateKey, LineEnding};
use rsa::{RsaPrivateKey, RsaPublicKey};

/// PEM-encoded RSA keypair materialized once per test process.
pub struct TestKeypair {
    pub public_pem: Vec<u8>,
    pub private_pem: Vec<u8>,
}

/// Returns the shared test keypair, generating it on first access.
pub fn test_keypair() -> &'static TestKeypair {
    static KEYPAIR: OnceLock<TestKeypair> = OnceLock::new();

    KEYPAIR.get_or_init(|| {
        let mut rng = rand::thread_rng();
        let private_key = RsaPrivateKey::new(&mut rng, 2048).expect("generate RSA key");
        let public_key = RsaPublicKey::from(&private_key);

        let public_pem = public_key
            .to_pkcs1_pem(LineEnding::LF)
            .expect("encode public key as PKCS#1 PEM")
            .into_bytes();
        let private_pem = private_key
            .to_pkcs8_pem(LineEnding::LF)
            .expect("encode private key as PKCS#8 PEM")
            .as_bytes()
            .to_vec();

        TestKeypair {
            public_pem,
            private_pem,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::test_keypair;

    #[test]
    fn generates_pem_material_and_caches_across_calls() {
        let first = test_keypair();
        let second = test_keypair();

        assert!(first
            .public_pem
            .starts_with(b"-----BEGIN RSA PUBLIC KEY-----"));
        assert!(first
            .private_pem
            .starts_with(b"-----BEGIN PRIVATE KEY-----"));
        assert!(std::ptr::eq(first, second), "must cache across calls");
    }
}
