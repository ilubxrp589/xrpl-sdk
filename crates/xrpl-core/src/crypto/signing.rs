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
    /// The bytes.
    pub bytes: [u8; 16],
    /// The key type.
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
    /// The private key (wiped on drop).
    pub private_key: Vec<u8>,
    /// A public key, hex (`PublicKey`).
    pub public_key: Vec<u8>,
    /// The key type.
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

    /// Sign XRPL signing data (a hash prefix plus serialized fields) the way
    /// rippled verifies it (PublicKey.cpp `verify`): secp256k1 signs
    /// SHA512Half(data), Ed25519 signs the data itself.
    pub fn sign_signing_data(&self, data: &[u8]) -> Result<Vec<u8>, CoreError> {
        match self.key_type {
            KeyType::Ed25519 => self.sign(data),
            KeyType::Secp256k1 => self.sign(&sha512_half(data)),
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
#[allow(
    clippy::expect_used,
    reason = "no CSPRNG means no safe key; the native rand::rng() path panics the same way"
)]
fn generate_random_bytes_16() -> [u8; 16] {
    let mut bytes = [0u8; 16];
    getrandom::getrandom(&mut bytes)
        .expect("CSPRNG not available — cannot generate secure random bytes");
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
/// 4. Signs it (`Keypair::sign_signing_data`), sets TxnSignature
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

    let signature = keypair.sign_signing_data(&payload)?;

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
        let seed = Seed::generate_with_type(KeyType::Secp256k1);
        let encoded = seed.to_base58();
        let decoded = Seed::from_base58(&encoded).unwrap();
        assert_eq!(decoded.key_type, KeyType::Secp256k1);
        assert_eq!(decoded.bytes, seed.bytes);
        assert_eq!(decoded.to_base58(), encoded);
    }

    #[test]
    fn seed_roundtrip_ed25519() {
        let seed = Seed::generate();
        let encoded = seed.to_base58();
        let decoded = Seed::from_base58(&encoded).unwrap();
        assert_eq!(decoded.key_type, KeyType::Ed25519);
        assert_eq!(decoded.bytes, seed.bytes);
        assert_eq!(decoded.to_base58(), encoded);
    }

    #[test]
    fn address_derivation_secp256k1() {
        let seed = Seed::generate_with_type(KeyType::Secp256k1);
        let keypair = Keypair::from_seed(&seed).unwrap();
        let address = keypair.classic_address();
        // XRPL classic addresses always start with 'r' and are 25-35 chars
        assert!(address.starts_with('r'));
        assert!(address.len() >= 25 && address.len() <= 35);
        // Derivation is deterministic: same seed produces same address
        let keypair2 = Keypair::from_seed(&seed).unwrap();
        assert_eq!(keypair2.classic_address(), address);
    }

    #[test]
    fn address_derivation_ed25519() {
        let seed = Seed::generate();
        let keypair = Keypair::from_seed(&seed).unwrap();
        let address = keypair.classic_address();
        // XRPL classic addresses always start with 'r' and are 25-35 chars
        assert!(address.starts_with('r'));
        assert!(address.len() >= 25 && address.len() <= 35);
        // Derivation is deterministic: same seed produces same address
        let keypair2 = Keypair::from_seed(&seed).unwrap();
        assert_eq!(keypair2.classic_address(), address);
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

    // Known answers. Keys, addresses and the "test message" signatures are
    // xrpl.js's ripple-keypairs fixtures (test/fixtures/api.json); rippled's
    // wallet_propose derives the same keys and addresses. The fixture writes
    // private keys with a type byte (00 / ED) that the keypair does not hold.
    const SECP_SEED: &str = "sp5fghtJtpUorTwvof1NpDXAzNwf5";
    const SECP_ADDRESS: &str = "rU6K7V3Po4snVhBBaU29sesqs2qTQJWDw1";
    const ED_SEED: &str = "sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r";
    const ED_ADDRESS: &str = "rLUEXYuLiQptky37CqLcm9USQpPiz5rkpD";

    fn known_keypair(
        seed: &str,
        key_type: KeyType,
        private_key: &str,
        public_key: &str,
        address: &str,
    ) -> Keypair {
        let seed = Seed::from_base58(seed).unwrap();
        assert_eq!(seed.key_type, key_type);
        let keypair = Keypair::from_seed(&seed).unwrap();
        assert_eq!(hex::encode_upper(&keypair.private_key), private_key[2..]);
        assert_eq!(hex::encode_upper(&keypair.public_key), public_key);
        assert_eq!(keypair.classic_address(), address);
        keypair
    }

    #[test]
    fn known_answer_secp256k1_keys_address_and_signature() {
        let keypair = known_keypair(
            SECP_SEED,
            KeyType::Secp256k1,
            "00D78B9735C3F26501C7337B8A5727FD53A6EFDBC6AA55984F098488561F985E23",
            "030D58EB48B4420B1F7B9DF55087E0E29FEF0E8468F9A6825B01CA2C361042D435",
            SECP_ADDRESS,
        );
        // secp256k1 signs SHA512Half(message) (RFC 6979, low S).
        let hash = sha512_half(b"test message");
        let sig = "30440220583A91C95E54E6A651C47BEC22744E0B101E2C4060E7B08F6341657DAD9BC3EE02207D1489C7395DB0188D3A56A977ECBA54B36FA9371B40319655B1B4429E33EF2D";
        assert_eq!(hex::encode_upper(keypair.sign(&hash).unwrap()), sig);
        assert!(keypair.verify(&hash, &hex::decode(sig).unwrap()).unwrap());
    }

    #[test]
    fn known_answer_ed25519_keys_address_and_signature() {
        let keypair = known_keypair(
            ED_SEED,
            KeyType::Ed25519,
            "EDB4C4E046826BD26190D09715FC31F4E6A728204EADD112905B08B14B7F15C4F3",
            "ED01FA53FA5A7E77798F882ECE20B1ABC00BB358A9E55A202D0D0676BD0CE37A63",
            ED_ADDRESS,
        );
        // Ed25519 signs the message itself, with no hash first.
        let sig = "CB199E1BFD4E3DAA105E4832EEDFA36413E1F44205E4EFB9E27E826044C21E3E2E848BBC8195E8959BADF887599B7310AD1B7047EF11B682E0D068F73749750E";
        assert_eq!(
            hex::encode_upper(keypair.sign(b"test message").unwrap()),
            sig
        );
        assert!(keypair
            .verify(b"test message", &hex::decode(sig).unwrap())
            .unwrap());
    }

    // rippled 3.4.1 signed this Payment with the seeds above (`sign`, offline).
    // Both schemes are deterministic, so the blobs must match byte for byte.
    fn signed_blob(seed: &str, account: &str) -> String {
        let keypair = Keypair::from_seed(&Seed::from_base58(seed).unwrap()).unwrap();
        let tx = serde_json::json!({
            "TransactionType": "Payment",
            "Account": account,
            "Destination": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "Amount": "1000000",
            "Fee": "12",
            "Sequence": 1,
            "Flags": 0
        });
        let signed = sign_transaction(&tx, &keypair).unwrap();
        hex::encode_upper(crate::codec::encode_transaction_json(&signed, false).unwrap())
    }

    #[test]
    fn sign_transaction_matches_rippled_secp256k1() {
        assert_eq!(
            signed_blob(SECP_SEED, SECP_ADDRESS),
            "120000220000000024000000016140000000000F424068400000000000000C7321030D58EB48B4420B1F7B9DF55087E0E29FEF0E8468F9A6825B01CA2C361042D43574473045022100BC95504DC72DDBDAA0F5EAA973EC07E057FD9A3E7500AC3CF07454C68219985E02200B0992C47857A867F5664BB6F74E62F086A59A0C31848E0FB3819127E8EC0A1281148049717CC948789F32F267ADC2582484E3DFA6988314B5F762798A53D543A014CAF8B297CFF8F2F937E8"
        );
    }

    #[test]
    fn sign_transaction_matches_rippled_ed25519() {
        assert_eq!(
            signed_blob(ED_SEED, ED_ADDRESS),
            "120000220000000024000000016140000000000F424068400000000000000C7321ED01FA53FA5A7E77798F882ECE20B1ABC00BB358A9E55A202D0D0676BD0CE37A637440A7C427E6D84B1DB144466DF589D88C2F4CFC530743015EF5E5C31587FDA5334EB5A7E9C418DC17702A56860C28EEC2475098106330504543CEF0965D27EF570C8114D28B177E48D9A8D057E70F7E464B498367281B988314B5F762798A53D543A014CAF8B297CFF8F2F937E8"
        );
    }

    #[test]
    fn sign_transaction_produces_valid_blob() {
        use serde_json::json;

        let seed = Seed::generate_with_type(KeyType::Secp256k1);
        let keypair = Keypair::from_seed(&seed).unwrap();

        // Use a second generated seed for the destination address
        let dest_seed = Seed::generate();
        let dest_keypair = Keypair::from_seed(&dest_seed).unwrap();

        let tx = json!({
            "TransactionType": "Payment",
            "Account": keypair.classic_address(),
            "Destination": dest_keypair.classic_address(),
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
