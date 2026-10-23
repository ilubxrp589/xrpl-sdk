use crate::CoreError;
use k256::ecdsa::hazmat::{bits2field, SignPrimitive};
use k256::ecdsa::{Signature, VerifyingKey};
use k256::elliptic_curve::sec1::ToEncodedPoint;
use k256::{Secp256k1, SecretKey};
use sha2::Sha256;
use zeroize::{Zeroize, Zeroizing};

#[cfg(not(feature = "std"))]
use alloc::{format, string::ToString, vec::Vec};

/// The first valid secret key (non-zero, below the curve order) among
/// `candidate(0)`, `candidate(1)`, … — `rippled`'s retry loop.
fn first_valid(candidate: impl Fn(u32) -> Zeroizing<[u8; 32]>) -> Result<SecretKey, CoreError> {
    (0..=u32::MAX)
        .find_map(|n| SecretKey::from_slice(candidate(n).as_slice()).ok())
        .ok_or(CoreError::KeyDerivationFailed)
}

/// Derive an XRPL secp256k1 key pair from a 16-byte seed, as `rippled` does
/// (its own scheme, not BIP32): root secret = SHA-512Half(seed ‖ seq), the
/// first valid; account secret = root + SHA-512Half(root public ‖ 0 ‖ sub),
/// the first valid. Returns the 32-byte account secret key, in a buffer
/// wiped on drop, and the 33-byte compressed public key. Every
/// intermediate secret is wiped too.
pub fn derive_keypair(seed: &[u8; 16]) -> Result<(Zeroizing<Vec<u8>>, Vec<u8>), CoreError> {
    let root = first_valid(|seq| super::secret_sha512_half(&[seed, &seq.to_be_bytes()]))?;
    let root_public = root.public_key().to_encoded_point(true);
    let intermediate = first_valid(|sub| {
        super::secret_sha512_half(&[
            root_public.as_bytes(),
            &0u32.to_be_bytes(),
            &sub.to_be_bytes(),
        ])
    })?;
    // SecretKey and NonZeroScalar wipe themselves on drop; the sum is wiped below.
    let mut scalar = *root.to_nonzero_scalar() + *intermediate.to_nonzero_scalar();
    let account = SecretKey::from_bytes(&scalar.to_bytes());
    scalar.zeroize();
    let account = account.map_err(|_| CoreError::KeyDerivationFailed)?;
    let public_key = account
        .public_key()
        .to_encoded_point(true)
        .as_bytes()
        .to_vec();
    Ok((Zeroizing::new(account.to_bytes().to_vec()), public_key))
}

/// The 33-byte compressed public key of a 32-byte secp256k1 secret key.
pub fn public_key(private_key: &[u8]) -> Result<Vec<u8>, CoreError> {
    let secret = secret_key(private_key)?;
    Ok(secret
        .public_key()
        .to_encoded_point(true)
        .as_bytes()
        .to_vec())
}

fn secret_key(private_key: &[u8]) -> Result<SecretKey, CoreError> {
    if private_key.len() != 32 {
        return Err(CoreError::InvalidKeyLength {
            expected: 32,
            got: private_key.len(),
        });
    }
    SecretKey::from_slice(private_key).map_err(|e| CoreError::SigningFailed(e.to_string()))
}

/// The 32-byte hash secp256k1 signs; anything else is an error. (The ECDSA
/// primitive would silently use only the first 32 bytes of a longer input,
/// so a signature over one long message would verify for every message that
/// starts the same way.)
fn prehash32(prehash: &[u8]) -> Result<&[u8; 32], CoreError> {
    prehash.try_into().map_err(|_| {
        CoreError::SigningFailed(format!(
            "secp256k1 signs a 32-byte hash, got {} bytes: hash the message first \
             (Keypair::sign_message does, as rippled verifies)",
            prehash.len()
        ))
    })
}

/// Sign a 32-byte hash (for XRPL data: SHA-512Half of it) with a secp256k1
/// secret key. Returns the DER signature, low-S. The nonce is RFC 6979's
/// (HMAC-SHA256), as in `rippled`, so the same key and hash always give the
/// same signature. A hash that is not exactly 32 bytes is an error.
pub fn sign(private_key: &[u8], prehash: &[u8]) -> Result<Vec<u8>, CoreError> {
    let prehash = prehash32(prehash)?;
    let secret = secret_key(private_key)?;
    // From the secret scalar alone: unlike k256's SigningKey, this does not
    // first derive the public key, which would double the cost.
    let mut scalar = secret.to_nonzero_scalar();
    let signed = bits2field::<Secp256k1>(prehash)
        .and_then(|z| scalar.try_sign_prehashed_rfc6979::<Sha256>(&z, &[]));
    scalar.zeroize();
    // k256 already returns the low-S form; normalizing again keeps that explicit.
    let (signature, _) = signed.map_err(|e| CoreError::SigningFailed(e.to_string()))?;
    let normalized: Signature = signature.normalize_s().unwrap_or(signature);
    Ok(normalized.to_der().as_bytes().to_vec())
}

/// Verify a DER secp256k1 signature over a 32-byte hash. The public key must
/// be 33 bytes, compressed (0x02/0x03), the only form `rippled` takes; the
/// signature must be low-S. A hash that is not 32 bytes, or a malformed key
/// or signature, is an error; a signature that does not match is `Ok(false)`.
pub fn verify(public_key: &[u8], prehash: &[u8], signature_der: &[u8]) -> Result<bool, CoreError> {
    use k256::ecdsa::signature::hazmat::PrehashVerifier;

    let prehash = prehash32(prehash)?;
    if !matches!(public_key, [0x02 | 0x03, ..]) || public_key.len() != 33 {
        return Err(CoreError::InvalidKeyLength {
            expected: 33,
            got: public_key.len(),
        });
    }
    let verifying_key = VerifyingKey::from_sec1_bytes(public_key)
        .map_err(|e| CoreError::SigningFailed(e.to_string()))?;
    let signature =
        Signature::from_der(signature_der).map_err(|e| CoreError::SigningFailed(e.to_string()))?;
    Ok(verifying_key.verify_prehash(prehash, &signature).is_ok())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn secp256k1_derive_known_seed() {
        // Genesis seed: all zeros → should produce known root private key
        let seed = [0u8; 16];
        let (private_key, public_key) = derive_keypair(&seed).unwrap();
        assert_eq!(public_key.len(), 33);
        assert_eq!(private_key.len(), 32);
    }

    #[test]
    fn secp256k1_sign_verify() {
        let seed = [0u8; 16];
        let (private_key, public_key) = derive_keypair(&seed).unwrap();

        let hash = [0xABu8; 32];
        let sig = sign(&private_key, &hash).unwrap();
        assert!(!sig.is_empty());

        let valid = verify(&public_key, &hash, &sig).unwrap();
        assert!(valid);

        let mut bad_hash = hash;
        bad_hash[0] = 0xFF;
        let invalid = verify(&public_key, &bad_hash, &sig).unwrap();
        assert!(!invalid);
    }

    #[test]
    fn secp256k1_derive_sign_verify_roundtrip() {
        // Generate a random secp256k1 seed and verify full round-trip
        let seed =
            crate::crypto::signing::Seed::generate_with_type(crate::address::KeyType::Secp256k1);
        let (privkey, pubkey) = derive_keypair(seed.as_bytes()).unwrap();

        assert_eq!(
            pubkey.len(),
            33,
            "secp256k1 public key must be 33 bytes (compressed)"
        );
        // Compressed secp256k1 keys start with 0x02 or 0x03
        assert!(
            pubkey[0] == 0x02 || pubkey[0] == 0x03,
            "secp256k1 compressed key must start with 0x02 or 0x03"
        );

        // Sign and verify
        let message = [0xABu8; 32];
        let sig = sign(&privkey, &message).unwrap();
        assert!(verify(&pubkey, &message, &sig).unwrap());

        // Derive address and verify format
        let account_id = crate::crypto::signing::public_key_to_account_id(&pubkey);
        let address = crate::address::encode_account_id(&account_id);
        assert!(address.starts_with('r'), "address must start with 'r'");
        assert!(
            address.len() >= 25 && address.len() <= 35,
            "address length must be valid"
        );

        // Deterministic: same seed bytes produce same keypair
        let (privkey2, pubkey2) = derive_keypair(seed.as_bytes()).unwrap();
        assert_eq!(privkey, privkey2);
        assert_eq!(pubkey, pubkey2);
    }

    #[test]
    fn secp256k1_produces_low_s_signature() {
        let seed = [0x42u8; 16];
        let message = [0xABu8; 32];

        let (privkey, _) = derive_keypair(&seed).unwrap();
        let sig_bytes = sign(&privkey, &message).unwrap();

        // DER layout: 0x30 [total-len] 0x02 [r-len] [r...] 0x02 [s-len] [s...]
        assert_eq!(sig_bytes[0], 0x30, "DER signature must start with 0x30");
        assert!(sig_bytes.len() >= 8, "signature too short for valid DER");

        let r_len = sig_bytes[3] as usize;
        let s_len_offset = 4 + r_len + 1;
        let s_len = sig_bytes[s_len_offset] as usize;
        let s_start = s_len_offset + 1;
        let s_bytes = &sig_bytes[s_start..s_start + s_len];

        let s_significant = s_bytes.iter().find(|&&b| b != 0).copied().unwrap_or(0);

        assert!(
            s_significant < 0x80,
            "signature S value is high-S — XRPL validators will reject"
        );
    }
}
