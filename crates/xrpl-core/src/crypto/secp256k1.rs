use crate::CoreError;
use ecdsa::hazmat::sign_prehashed_rfc6979;
use k256::ecdsa::{Signature, VerifyingKey};
use k256::elliptic_curve::sec1::ToSec1Point;
use k256::sha2::Sha256;
use k256::{Secp256k1, SecretKey};
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
///
/// ```
/// use xrpl_core::crypto::{secp256k1, Seed};
///
/// let seed = Seed::from_base58("sp5fghtJtpUorTwvof1NpDXAzNwf5")?;
/// let (secret, public_key) = secp256k1::derive_keypair(seed.as_bytes())?;
/// assert_eq!(secret.len(), 32);
/// assert_eq!(
///     hex::encode_upper(&public_key),
///     "030D58EB48B4420B1F7B9DF55087E0E29FEF0E8468F9A6825B01CA2C361042D435",
/// );
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
///
/// # Errors
///
/// [`CoreError::KeyDerivationFailed`] if no valid secret key turns up in
/// 2^32 tries, or the account secret comes to zero: `rippled` fails there
/// too, and neither has any real chance of happening.
pub fn derive_keypair(seed: &[u8; 16]) -> Result<(Zeroizing<Vec<u8>>, Vec<u8>), CoreError> {
    let root = first_valid(|seq| super::secret_sha512_half(&[seed, &seq.to_be_bytes()]))?;
    let root_public = root.public_key().to_sec1_point(true);
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
    let public_key = account.public_key().to_sec1_point(true).as_bytes().to_vec();
    Ok((Zeroizing::new(account.to_bytes().to_vec()), public_key))
}

/// The 33-byte compressed public key of a 32-byte secp256k1 secret key.
///
/// ```
/// use xrpl_core::crypto::{secp256k1, Keypair, Seed};
///
/// let keys = Keypair::from_seed(&Seed::from_base58("sp5fghtJtpUorTwvof1NpDXAzNwf5")?)?;
/// assert_eq!(secp256k1::public_key(keys.private_key())?, keys.public_key());
/// // Zero is not a secret key.
/// assert!(secp256k1::public_key(&[0; 32]).is_err());
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
///
/// # Errors
///
/// [`CoreError::InvalidKeyLength`] if `private_key` is not 32 bytes;
/// [`CoreError::SigningFailed`] if it is zero or not below the curve order.
pub fn public_key(private_key: &[u8]) -> Result<Vec<u8>, CoreError> {
    let secret = secret_key(private_key)?;
    Ok(secret.public_key().to_sec1_point(true).as_bytes().to_vec())
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
///
/// ```
/// use xrpl_core::crypto::{secp256k1, sha512_half, Keypair, Seed};
///
/// let keys = Keypair::from_seed(&Seed::from_base58("sp5fghtJtpUorTwvof1NpDXAzNwf5")?)?;
/// let signature = secp256k1::sign(keys.private_key(), &sha512_half(b"message"))?;
/// assert!(secp256k1::verify(keys.public_key(), &sha512_half(b"message"), &signature)?);
/// // `Keypair::sign_message` hashes first, then signs the same way.
/// assert_eq!(signature, keys.sign_message(b"message")?);
/// assert!(secp256k1::sign(keys.private_key(), b"message").is_err());
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
///
/// # Errors
///
/// [`CoreError::SigningFailed`] if `prehash` is not 32 bytes or the key is
/// zero or not below the curve order; [`CoreError::InvalidKeyLength`] if
/// `private_key` is not 32 bytes.
pub fn sign(private_key: &[u8], prehash: &[u8]) -> Result<Vec<u8>, CoreError> {
    let prehash = prehash32(prehash)?;
    let secret = secret_key(private_key)?;
    // From the secret scalar alone: unlike k256's SigningKey, this does not
    // first derive the public key, which would double the cost.
    let mut scalar = secret.to_nonzero_scalar();
    let (signature, _) = sign_prehashed_rfc6979::<Secp256k1, Sha256>(&scalar, prehash, &[]);
    scalar.zeroize();
    // k256 already returns the low-S form; normalizing again keeps that explicit.
    Ok(signature.normalize_s().to_der().as_bytes().to_vec())
}

/// Verify a DER secp256k1 signature over a 32-byte hash. The public key must
/// be 33 bytes, compressed (0x02/0x03), the only form `rippled` takes; the
/// signature must be low-S. A hash that is not 32 bytes, or a malformed key
/// or signature, is an error; a signature that does not match is `Ok(false)`.
///
/// ```
/// use xrpl_core::crypto::{secp256k1, sha512_half, Keypair, Seed};
///
/// let keys = Keypair::from_seed(&Seed::from_base58("sp5fghtJtpUorTwvof1NpDXAzNwf5")?)?;
/// let signature = keys.sign_message(b"message")?;
/// assert!(secp256k1::verify(keys.public_key(), &sha512_half(b"message"), &signature)?);
/// assert!(!secp256k1::verify(keys.public_key(), &sha512_half(b"other"), &signature)?);
/// assert!(secp256k1::verify(keys.public_key(), &sha512_half(b"message"), b"not DER").is_err());
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
///
/// # Errors
///
/// [`CoreError::InvalidKeyLength`] if `public_key` is not 33 bytes starting
/// 0x02 or 0x03; [`CoreError::SigningFailed`] if `prehash` is not 32 bytes,
/// the key is not a point on the curve, or the signature is not DER.
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

/// The low-S form of a DER signature, the only form `rippled` takes (it
/// refuses a signature that is not fully canonical): a signature with a high
/// S, as some KMS return, becomes the equivalent one with `n - S`; a low-S
/// signature comes back as it is.
///
/// # Errors
///
/// [`CoreError::SigningFailed`] if `signature_der` is not a DER signature.
#[cfg(feature = "std")]
pub(crate) fn low_s(signature_der: &[u8]) -> Result<Vec<u8>, CoreError> {
    let signature =
        Signature::from_der(signature_der).map_err(|e| CoreError::SigningFailed(e.to_string()))?;
    let low = signature.normalize_s();
    Ok(if low == signature {
        signature_der.to_vec()
    } else {
        low.to_der().as_bytes().to_vec()
    })
}

/// A public key in the compressed form the XRP Ledger writes (33 bytes,
/// `0x02` or `0x03` then X), from the uncompressed form (65 bytes, `0x04`
/// then X and Y) that a KMS or HSM may give (the last 65 bytes of an
/// AWS KMS `GetPublicKey` answer, for instance). A compressed key comes
/// back as it is.
///
/// ```
/// use xrpl_core::crypto::{secp256k1, Keypair, Seed};
///
/// let keys = Keypair::from_seed(&Seed::from_base58("sp5fghtJtpUorTwvof1NpDXAzNwf5")?)?;
/// let compressed = secp256k1::compressed_public_key(keys.public_key())?;
/// assert_eq!(compressed, keys.public_key());
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
///
/// # Errors
///
/// [`CoreError::SigningFailed`] if `public_key` is not a SEC1 point on the
/// curve (65 bytes starting `0x04`, or 33 starting `0x02` / `0x03`).
pub fn compressed_public_key(public_key: &[u8]) -> Result<Vec<u8>, CoreError> {
    let key = VerifyingKey::from_sec1_bytes(public_key)
        .map_err(|e| CoreError::SigningFailed(format!("not a secp256k1 public key: {e}")))?;
    Ok(key.to_sec1_point(true).as_bytes().to_vec())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;
    #[test]
    fn a_high_s_signature_becomes_the_low_s_one() {
        use k256::elliptic_curve::ops::Neg;
        let (private_key, public_key) = derive_keypair(&[0u8; 16]).unwrap();
        let hash = [0x5Au8; 32];
        let low = sign(&private_key, &hash).unwrap();
        let parsed = Signature::from_der(&low).unwrap();
        let (r, s) = parsed.split_scalars();
        let high = Signature::from_scalars(r, s.neg()).unwrap();
        let high_der = high.to_der().as_bytes().to_vec();
        assert_ne!(high_der, low);
        // rippled refuses the high-S form; low_s gives back the one it takes.
        assert!(!verify(&public_key, &hash, &high_der).unwrap());
        assert_eq!(low_s(&high_der).unwrap(), low);
        assert_eq!(low_s(&low).unwrap(), low);
        assert!(low_s(b"not DER").is_err());
    }

    #[test]
    fn uncompressed_public_keys_compress() {
        let (private_key, public_key) = derive_keypair(&[7u8; 16]).unwrap();
        let secret = secret_key(&private_key).unwrap();
        let uncompressed = secret.public_key().to_sec1_point(false);
        assert_eq!(uncompressed.as_bytes().len(), 65);
        assert_eq!(
            compressed_public_key(uncompressed.as_bytes()).unwrap(),
            public_key
        );
        assert_eq!(compressed_public_key(&public_key).unwrap(), public_key);
        assert!(compressed_public_key(&[0x04; 65]).is_err());
    }

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
