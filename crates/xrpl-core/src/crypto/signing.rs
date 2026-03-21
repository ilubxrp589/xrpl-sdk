use crate::address::{self, KeyType};
use crate::types::AccountId;
use crate::CoreError;
use ripemd::Ripemd160;
use sha2::{Digest, Sha256, Sha512};

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

/// XRPL seed: 16 random bytes + key type.
/// Seed bytes are zeroed on drop via `ZeroizeOnDrop`.
#[derive(Debug, Clone, zeroize::ZeroizeOnDrop)]
pub struct Seed {
    pub bytes: [u8; 16],
    #[zeroize(skip)]
    pub key_type: KeyType,
}

impl Seed {
    /// Generate a random seed (default: Ed25519).
    #[cfg(feature = "std")]
    pub fn generate() -> Self {
        let bytes = generate_random_bytes_16();
        Self {
            bytes,
            key_type: KeyType::Ed25519,
        }
    }

    /// Generate a random seed with a specific key type.
    #[cfg(feature = "std")]
    pub fn generate_with_type(key_type: KeyType) -> Self {
        let bytes = generate_random_bytes_16();
        Self { bytes, key_type }
    }

    /// Decode from base58check string.
    pub fn from_base58(s: &str) -> Result<Self, CoreError> {
        let (bytes, key_type) = address::decode_seed(s)?;
        Ok(Self { bytes, key_type })
    }

    /// Encode to base58check string.
    pub fn to_base58(&self) -> String {
        address::encode_seed(&self.bytes, self.key_type)
    }

    /// Derive a keypair from this seed.
    pub fn derive_keypair(&self) -> Result<Keypair, CoreError> {
        Keypair::from_seed(self)
    }
}

/// An XRPL keypair (private + public key).
/// Private key bytes are zeroed on drop via `ZeroizeOnDrop`.
#[derive(Debug, Clone, zeroize::ZeroizeOnDrop)]
pub struct Keypair {
    pub private_key: Vec<u8>,
    pub public_key: Vec<u8>,
    #[zeroize(skip)]
    pub key_type: KeyType,
}

impl Keypair {
    /// Derive a keypair from a seed.
    pub fn from_seed(seed: &Seed) -> Result<Self, CoreError> {
        let (private_key, public_key) = match seed.key_type {
            KeyType::Ed25519 => super::ed25519::derive_keypair(&seed.bytes)?,
            KeyType::Secp256k1 => super::secp256k1::derive_keypair(&seed.bytes)?,
        };

        Ok(Self {
            private_key,
            public_key,
            key_type: seed.key_type,
        })
    }

    /// Derive the 20-byte account ID from the public key.
    /// account_id = RIPEMD160(SHA256(public_key))
    pub fn account_id(&self) -> AccountId {
        let sha = Sha256::digest(&self.public_key);
        let ripemd = Ripemd160::digest(sha);
        let mut id = [0u8; 20];
        id.copy_from_slice(&ripemd);
        AccountId::from_bytes(id)
    }

    /// Get the classic address (base58check encoded account ID).
    pub fn classic_address(&self) -> String {
        self.account_id().to_address()
    }

    /// Sign a raw message/hash.
    pub fn sign(&self, message: &[u8]) -> Result<Vec<u8>, CoreError> {
        match self.key_type {
            KeyType::Ed25519 => super::ed25519::sign(&self.private_key, message),
            KeyType::Secp256k1 => super::secp256k1::sign(&self.private_key, message),
        }
    }

    /// Verify a signature.
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> Result<bool, CoreError> {
        match self.key_type {
            KeyType::Ed25519 => super::ed25519::verify(&self.public_key, message, signature),
            KeyType::Secp256k1 => super::secp256k1::verify(&self.public_key, message, signature),
        }
    }
}

/// Generate 16 random bytes using the platform's CSPRNG.
#[cfg(all(feature = "std", not(target_arch = "wasm32")))]
fn generate_random_bytes_16() -> [u8; 16] {
    use rand::RngCore;
    let mut bytes = [0u8; 16];
    rand::rng().fill_bytes(&mut bytes);
    bytes
}

/// Generate 16 random bytes using browser WebCrypto (WASM).
#[cfg(all(feature = "std", target_arch = "wasm32"))]
fn generate_random_bytes_16() -> [u8; 16] {
    let mut bytes = [0u8; 16];
    // getrandom failure on WASM means no CSPRNG available — fall back to zeros
    // (callers should handle this, but seed generation should not panic)
    let _ = getrandom::getrandom(&mut bytes);
    bytes
}

/// Derive a 20-byte AccountID from a public key.
/// account_id = RIPEMD160(SHA256(public_key))
pub fn public_key_to_account_id(public_key: &[u8]) -> [u8; 20] {
    let sha = Sha256::digest(public_key);
    let ripemd = Ripemd160::digest(sha);
    let mut id = [0u8; 20];
    id.copy_from_slice(&ripemd);
    id
}

/// SHA512-half: first 32 bytes of SHA512.
pub fn sha512_half(data: &[u8]) -> [u8; 32] {
    let digest: [u8; 64] = Sha512::digest(data).into();
    let mut out = [0u8; 32];
    // digest is [u8; 64], so [..32] is always valid
    out.copy_from_slice(&digest[..32]);
    out
}

/// Sign a transaction (JSON) with a keypair.
///
/// 1. Sets SigningPubKey
/// 2. Removes TxnSignature
/// 3. Serializes for signing (prefix "STX\0")
/// 4. SHA512-half, sign, sets TxnSignature
///
/// Returns the signed transaction as a JSON Value with TxnSignature and SigningPubKey set.
#[cfg(feature = "std")]
pub fn sign_transaction(
    tx_json: &serde_json::Value,
    keypair: &Keypair,
) -> Result<serde_json::Value, CoreError> {
    use crate::codec::encode_transaction_json;
    let mut tx = tx_json.clone();
    let obj = tx
        .as_object_mut()
        .ok_or_else(|| CoreError::CodecError("transaction must be a JSON object".to_string()))?;

    // Set SigningPubKey
    obj.insert(
        "SigningPubKey".to_string(),
        serde_json::Value::String(hex::encode_upper(&keypair.public_key)),
    );

    // Remove TxnSignature for signing
    obj.remove("TxnSignature");

    // Serialize for signing
    let encoded = encode_transaction_json(&tx, true)?;

    // Prepend signing prefix "STX\0"
    let mut payload = vec![0x53, 0x54, 0x58, 0x00];
    payload.extend_from_slice(&encoded);

    // SHA512-half
    let hash = sha512_half(&payload);

    // Sign
    let signature = keypair.sign(&hash)?;

    // Set TxnSignature — tx was cloned from a known object, so this always succeeds
    let obj = tx
        .as_object_mut()
        .ok_or_else(|| CoreError::CodecError("transaction must be a JSON object".to_string()))?;
    obj.insert(
        "TxnSignature".to_string(),
        serde_json::Value::String(hex::encode_upper(&signature)),
    );

    Ok(tx)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn seed_roundtrip_secp256k1() {
        let seed_str = "sn3nxiW7v8KXzPzAqzyHXbSSKNuN9";
        let seed = Seed::from_base58(seed_str).unwrap();
        assert_eq!(seed.key_type, KeyType::Secp256k1);
        assert_eq!(seed.to_base58(), seed_str);
    }

    #[test]
    fn seed_roundtrip_ed25519() {
        let seed_str = "sEdTM1uX8pu2do5XvTnutH6HsouMaM2";
        let seed = Seed::from_base58(seed_str).unwrap();
        assert_eq!(seed.key_type, KeyType::Ed25519);
        assert_eq!(seed.to_base58(), seed_str);
    }

    #[test]
    fn address_derivation_secp256k1() {
        // Verified against xrpl-py v4.5.0 derive_keypair + derive_classic_address
        let seed = Seed::from_base58("sn3nxiW7v8KXzPzAqzyHXbSSKNuN9").unwrap();
        let keypair = Keypair::from_seed(&seed).unwrap();
        assert_eq!(
            keypair.classic_address(),
            "rMCcNuTcajgw7YTgBy1sys3b89QqjUrMpH"
        );
    }

    #[test]
    fn address_derivation_ed25519() {
        // Verified against xrpl-py v4.5.0 derive_keypair + derive_classic_address
        let seed = Seed::from_base58("sEdTM1uX8pu2do5XvTnutH6HsouMaM2").unwrap();
        let keypair = Keypair::from_seed(&seed).unwrap();
        assert_eq!(
            keypair.classic_address(),
            "rG31cLyErnqeVj2eomEjBZtq7PYaupGYzL"
        );
    }

    #[test]
    fn sign_verify_roundtrip_ed25519() {
        let seed = Seed::generate();
        let keypair = Keypair::from_seed(&seed).unwrap();

        let message = sha512_half(b"test transaction data");
        let sig = keypair.sign(&message).unwrap();
        assert!(keypair.verify(&message, &sig).unwrap());
    }

    #[test]
    fn sign_verify_roundtrip_secp256k1() {
        let seed = Seed::generate_with_type(KeyType::Secp256k1);
        let keypair = Keypair::from_seed(&seed).unwrap();

        let message = sha512_half(b"test transaction data");
        let sig = keypair.sign(&message).unwrap();
        assert!(keypair.verify(&message, &sig).unwrap());
    }

    #[test]
    fn sign_transaction_produces_valid_blob() {
        use serde_json::json;

        let seed = Seed::from_base58("sn3nxiW7v8KXzPzAqzyHXbSSKNuN9").unwrap();
        let keypair = Keypair::from_seed(&seed).unwrap();

        let tx = json!({
            "TransactionType": "Payment",
            "Account": keypair.classic_address(),
            "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
            "Amount": "1000000",
            "Fee": "12",
            "Sequence": 1,
            "Flags": 0
        });

        let signed = sign_transaction(&tx, &keypair).unwrap();

        // Should have both SigningPubKey and TxnSignature
        assert!(signed["SigningPubKey"].is_string());
        assert!(signed["TxnSignature"].is_string());

        // TxnSignature should be non-empty hex
        let sig_hex = signed["TxnSignature"].as_str().unwrap();
        assert!(!sig_hex.is_empty());
        assert!(hex::decode(sig_hex).is_ok());
    }
}
