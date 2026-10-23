use crate::CoreError;
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use zeroize::{Zeroize, Zeroizing};

#[cfg(not(feature = "std"))]
use alloc::{string::ToString, vec::Vec};

/// Derive an Ed25519 key pair from a 16-byte seed, as `rippled` does: the
/// secret key is SHA-512Half(seed); the public key is 0xED + the 32-byte
/// point (33 bytes, as on the ledger). The secret key comes back in a buffer
/// wiped on drop.
///
/// ```
/// use xrpl_core::crypto::{ed25519, Seed};
///
/// let seed = Seed::from_base58("sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r")?;
/// let (secret, public_key) = ed25519::derive_keypair(seed.as_bytes())?;
/// assert_eq!(secret.len(), 32);
/// assert_eq!(
///     hex::encode_upper(&public_key),
///     "ED01FA53FA5A7E77798F882ECE20B1ABC00BB358A9E55A202D0D0676BD0CE37A63",
/// );
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
///
/// # Errors
///
/// None: every seed gives a key pair. The `Result` matches
/// [`secp256k1::derive_keypair`](super::secp256k1::derive_keypair), which
/// can fail.
pub fn derive_keypair(seed: &[u8; 16]) -> Result<(Zeroizing<Vec<u8>>, Vec<u8>), CoreError> {
    let secret = super::secret_sha512_half(&[seed]);
    // SigningKey wipes itself on drop.
    let signing_key = SigningKey::from_bytes(&secret);
    let mut public_key = Vec::with_capacity(33);
    public_key.push(0xED);
    public_key.extend_from_slice(signing_key.verifying_key().as_bytes());
    Ok((Zeroizing::new(secret.to_vec()), public_key))
}

/// The 33-byte public key (0xED + point) of a 32-byte Ed25519 secret key.
///
/// ```
/// use xrpl_core::crypto::{ed25519, Keypair, Seed};
///
/// let keys = Keypair::from_seed(&Seed::from_base58("sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r")?)?;
/// assert_eq!(ed25519::public_key(keys.private_key())?, keys.public_key());
/// assert!(ed25519::public_key(&[7; 31]).is_err());
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
///
/// # Errors
///
/// [`CoreError::InvalidKeyLength`] if `private_key` is not 32 bytes.
pub fn public_key(private_key: &[u8]) -> Result<Vec<u8>, CoreError> {
    let mut key_bytes = secret_key_bytes(private_key)?;
    let signing_key = SigningKey::from_bytes(&key_bytes);
    key_bytes.zeroize();
    let mut public_key = Vec::with_capacity(33);
    public_key.push(0xED);
    public_key.extend_from_slice(signing_key.verifying_key().as_bytes());
    Ok(public_key)
}

fn secret_key_bytes(private_key: &[u8]) -> Result<[u8; 32], CoreError> {
    private_key
        .try_into()
        .map_err(|_| CoreError::InvalidKeyLength {
            expected: 32,
            got: private_key.len(),
        })
}

/// Sign `message` (the message itself: Ed25519 hashes internally, and
/// `rippled` verifies Ed25519 over the unhashed signing data) with a 32-byte
/// secret key. Returns the 64-byte signature.
///
/// ```
/// use xrpl_core::crypto::{ed25519, Keypair, Seed};
///
/// let keys = Keypair::from_seed(&Seed::from_base58("sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r")?)?;
/// let signature = ed25519::sign(keys.private_key(), b"message")?;
/// assert_eq!(signature.len(), 64);
/// assert!(ed25519::verify(keys.public_key(), b"message", &signature)?);
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
///
/// # Errors
///
/// [`CoreError::InvalidKeyLength`] if `private_key` is not 32 bytes.
pub fn sign(private_key: &[u8], message: &[u8]) -> Result<Vec<u8>, CoreError> {
    let mut key_bytes = secret_key_bytes(private_key)?;
    let signing_key = SigningKey::from_bytes(&key_bytes);
    key_bytes.zeroize();
    Ok(signing_key.sign(message).to_bytes().to_vec())
}

/// Verify an Ed25519 signature over `message`. `public_key` is 33 bytes
/// (0xED + point, as on the ledger) or the raw 32-byte point. A malformed
/// key is an error; a signature that is not 64 bytes, or does not match,
/// is `Ok(false)`.
///
/// Verification is strict (`verify_strict`: no small-order keys or `R`),
/// which can refuse a signature `rippled` would take from such a key, never
/// the other way round.
///
/// ```
/// use xrpl_core::crypto::{ed25519, Keypair, Seed};
///
/// let keys = Keypair::from_seed(&Seed::from_base58("sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r")?)?;
/// let signature = keys.sign_message(b"message")?;
/// assert!(ed25519::verify(keys.public_key(), b"message", &signature)?);
/// // The raw 32-byte point works too.
/// assert!(ed25519::verify(&keys.public_key()[1..], b"message", &signature)?);
/// assert!(!ed25519::verify(keys.public_key(), b"another message", &signature)?);
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
///
/// # Errors
///
/// [`CoreError::InvalidKeyLength`] if `public_key` is neither 0xED + 32
/// bytes nor 32 bytes; [`CoreError::SigningFailed`] if its 32 bytes are not
/// a point on the curve.
pub fn verify(public_key: &[u8], message: &[u8], signature: &[u8]) -> Result<bool, CoreError> {
    let raw_key = match public_key {
        [0xED, rest @ ..] if rest.len() == 32 => rest,
        _ if public_key.len() == 32 => public_key,
        _ => {
            return Err(CoreError::InvalidKeyLength {
                expected: 33,
                got: public_key.len(),
            })
        }
    };
    let key_bytes: [u8; 32] = raw_key
        .try_into()
        .map_err(|_| CoreError::InvalidKeyLength {
            expected: 33,
            got: public_key.len(),
        })?;
    let verifying_key = VerifyingKey::from_bytes(&key_bytes)
        .map_err(|e| CoreError::SigningFailed(e.to_string()))?;
    let Ok(sig_bytes) = <[u8; 64]>::try_from(signature) else {
        return Ok(false);
    };
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
        let (privkey, pubkey) = derive_keypair(seed.as_bytes()).unwrap();

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
        let (privkey2, pubkey2) = derive_keypair(seed.as_bytes()).unwrap();
        assert_eq!(privkey, privkey2);
        assert_eq!(pubkey, pubkey2);
    }
}
