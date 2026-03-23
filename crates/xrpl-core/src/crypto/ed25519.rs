use crate::CoreError;
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use sha2::{Digest, Sha512};
use zeroize::Zeroize;

#[cfg(not(feature = "std"))]
use alloc::{string::ToString, vec::Vec};

/// Derive an Ed25519 keypair from a 16-byte seed.
/// XRPL Ed25519: SHA512(seed) → first 32 bytes → scalar → keypair.
/// Public key is prefixed with 0xED (33 bytes total on wire).
pub fn derive_keypair(seed: &[u8; 16]) -> Result<(Vec<u8>, Vec<u8>), CoreError> {
    let hash: [u8; 64] = Sha512::digest(seed).into();
    let mut secret_bytes = [0u8; 32];
    secret_bytes.copy_from_slice(&hash[..32]);

    let signing_key = SigningKey::from_bytes(&secret_bytes);
    let verifying_key = signing_key.verifying_key();

    // Private key: 32-byte scalar
    let private_key = secret_bytes.to_vec();

    // Public key: 0xED prefix + 32-byte compressed point = 33 bytes
    let mut public_key = Vec::with_capacity(33);
    public_key.push(0xED);
    public_key.extend_from_slice(verifying_key.as_bytes());

    // Zeroize intermediate key material
    secret_bytes.zeroize();

    Ok((private_key, public_key))
}

/// Sign a message with an Ed25519 private key.
/// Returns 64-byte signature.
pub fn sign(private_key: &[u8], message: &[u8]) -> Result<Vec<u8>, CoreError> {
    if private_key.len() != 32 {
        return Err(CoreError::InvalidKeyLength {
            expected: 32,
            got: private_key.len(),
        });
    }

    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(private_key);
    let signing_key = SigningKey::from_bytes(&key_bytes);
    let signature = signing_key.sign(message);

    Ok(signature.to_bytes().to_vec())
}

/// Verify an Ed25519 signature.
/// public_key should be 33 bytes (0xED + 32 bytes) or 32 bytes (raw).
pub fn verify(public_key: &[u8], message: &[u8], signature: &[u8]) -> Result<bool, CoreError> {
    let raw_key = if public_key.len() == 33 && public_key.first() == Some(&0xED) {
        public_key.get(1..).unwrap_or_default()
    } else if public_key.len() == 32 {
        public_key
    } else {
        return Err(CoreError::InvalidKeyLength {
            expected: 33,
            got: public_key.len(),
        });
    };

    if signature.len() != 64 {
        return Ok(false);
    }

    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(raw_key);
    let verifying_key = VerifyingKey::from_bytes(&key_bytes)
        .map_err(|e| CoreError::SigningFailed(e.to_string()))?;

    let mut sig_bytes = [0u8; 64];
    sig_bytes.copy_from_slice(signature);
    let sig = ed25519_dalek::Signature::from_bytes(&sig_bytes);

    Ok(verifying_key.verify_strict(message, &sig).is_ok())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn ed25519_derive_and_sign_verify() {
        let seed = [0u8; 16];
        let (private_key, public_key) = derive_keypair(&seed).unwrap();

        assert_eq!(public_key.len(), 33);
        assert_eq!(public_key[0], 0xED);
        assert_eq!(private_key.len(), 32);

        let message = b"test message";
        let sig = sign(&private_key, message).unwrap();
        assert_eq!(sig.len(), 64);

        let valid = verify(&public_key, message, &sig).unwrap();
        assert!(valid);

        // Tamper with message
        let invalid = verify(&public_key, b"wrong message", &sig).unwrap();
        assert!(!invalid);
    }

    #[test]
    fn ed25519_derive_sign_verify_roundtrip() {
        // Generate a random Ed25519 seed and verify full round-trip
        let seed = crate::crypto::signing::Seed::generate();
        let (privkey, pubkey) = derive_keypair(&seed.bytes).unwrap();

        assert_eq!(
            pubkey.len(),
            33,
            "Ed25519 public key must be 33 bytes (0xED prefix + 32 bytes)"
        );
        assert_eq!(pubkey[0], 0xED, "Ed25519 public key must start with 0xED");

        // Sign and verify
        let message = b"ed25519 roundtrip test";
        let sig = sign(&privkey, message).unwrap();
        assert!(verify(&pubkey, message, &sig).unwrap());

        // Derive address and verify format
        let account_id = crate::crypto::signing::public_key_to_account_id(&pubkey);
        let address = crate::address::encode_account_id(&account_id);
        assert!(address.starts_with('r'), "address must start with 'r'");
        assert!(
            address.len() >= 25 && address.len() <= 35,
            "address length must be valid"
        );

        // Deterministic: same seed bytes produce same keypair
        let (privkey2, pubkey2) = derive_keypair(&seed.bytes).unwrap();
        assert_eq!(privkey, privkey2);
        assert_eq!(pubkey, pubkey2);
    }
}
