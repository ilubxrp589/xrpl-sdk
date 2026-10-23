use crate::address::{self, KeyType};
use crate::types::AccountId;
use crate::CoreError;
use core::fmt;
use ripemd::Ripemd160;
use sha2::{Digest, Sha256, Sha512};
use zeroize::Zeroizing;

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

/// An XRPL seed: 16 secret bytes and the key type they derive (`s…` in
/// base58, `sEd…` for Ed25519). The bytes are wiped on drop, and `Debug`
/// shows only the key type, so a seed cannot reach a log by `{:?}`.
#[derive(Clone, zeroize::ZeroizeOnDrop)]
pub struct Seed {
    bytes: [u8; 16],
    #[zeroize(skip)]
    key_type: KeyType,
}

impl Seed {
    /// Generate a random seed (default: Ed25519) from the operating system's
    /// random source.
    #[cfg(feature = "std")]
    pub fn generate() -> Self {
        Self::generate_with_type(KeyType::Ed25519)
    }

    /// Generate a random seed with a specific key type.
    #[cfg(feature = "std")]
    pub fn generate_with_type(key_type: KeyType) -> Self {
        Self {
            bytes: generate_random_bytes_16(),
            key_type,
        }
    }

    /// A seed from its 16 bytes and key type.
    pub fn from_bytes(bytes: [u8; 16], key_type: KeyType) -> Self {
        Self { bytes, key_type }
    }

    /// Decode from base58check string.
    pub fn from_base58(s: &str) -> Result<Self, CoreError> {
        let (bytes, key_type) = address::decode_seed(s)?;
        Ok(Self { bytes, key_type })
    }

    /// Encode to base58check (`s…`), in a string wiped on drop.
    pub fn to_base58(&self) -> Zeroizing<String> {
        Zeroizing::new(address::encode_seed(&self.bytes, self.key_type))
    }

    /// The 16 secret bytes.
    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.bytes
    }

    /// The key type the seed derives.
    pub fn key_type(&self) -> KeyType {
        self.key_type
    }

    /// Derive a keypair from this seed.
    pub fn derive_keypair(&self) -> Result<Keypair, CoreError> {
        Keypair::from_seed(self)
    }
}

impl fmt::Debug for Seed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Seed")
            .field("key_type", &self.key_type)
            .field("bytes", &"[REDACTED]")
            .finish()
    }
}

/// An XRPL key pair. The secret key is wiped on drop and never shown by
/// `Debug`; the public key always belongs to the secret key (every
/// constructor derives it).
#[derive(Clone, zeroize::ZeroizeOnDrop)]
pub struct Keypair {
    private_key: Vec<u8>,
    #[zeroize(skip)]
    public_key: Vec<u8>,
    #[zeroize(skip)]
    key_type: KeyType,
}

impl Keypair {
    /// Derive a keypair from a seed.
    pub fn from_seed(seed: &Seed) -> Result<Self, CoreError> {
        let (mut private_key, public_key) = match seed.key_type {
            KeyType::Ed25519 => super::ed25519::derive_keypair(&seed.bytes)?,
            KeyType::Secp256k1 => super::secp256k1::derive_keypair(&seed.bytes)?,
        };
        Ok(Self {
            // Moved out of the wiped buffer, not copied.
            private_key: core::mem::take(&mut *private_key),
            public_key,
            key_type: seed.key_type,
        })
    }

    /// A keypair from a 32-byte secret key; the public key is derived from it.
    pub fn from_private_key(key_type: KeyType, private_key: &[u8]) -> Result<Self, CoreError> {
        let public_key = match key_type {
            KeyType::Ed25519 => super::ed25519::public_key(private_key)?,
            KeyType::Secp256k1 => super::secp256k1::public_key(private_key)?,
        };
        Ok(Self {
            private_key: private_key.to_vec(),
            public_key,
            key_type,
        })
    }

    /// The public key as it appears on the ledger: 33 bytes (0xED + point
    /// for Ed25519, compressed for secp256k1).
    pub fn public_key(&self) -> &[u8] {
        &self.public_key
    }

    /// The 32-byte secret key. Treat it as you treat the seed.
    pub fn private_key(&self) -> &[u8] {
        &self.private_key
    }

    /// The key type.
    pub fn key_type(&self) -> KeyType {
        self.key_type
    }

    /// Derive the 20-byte account ID from the public key.
    /// account_id = RIPEMD160(SHA256(public_key))
    pub fn account_id(&self) -> AccountId {
        AccountId::from_bytes(public_key_to_account_id(&self.public_key))
    }

    /// Get the classic address (base58check encoded account ID).
    pub fn classic_address(&self) -> String {
        self.account_id().to_address()
    }

    /// Sign `message` the way `rippled` verifies a signature from this key
    /// (`PublicKey.cpp` `verify`): Ed25519 signs the message itself;
    /// secp256k1 signs SHA-512Half(message). Use this for any message,
    /// including XRPL signing data (a hash prefix plus serialized fields).
    pub fn sign_message(&self, message: &[u8]) -> Result<Vec<u8>, CoreError> {
        match self.key_type {
            KeyType::Ed25519 => super::ed25519::sign(&self.private_key, message),
            KeyType::Secp256k1 => super::secp256k1::sign(&self.private_key, &sha512_half(message)),
        }
    }

    /// Whether `signature` is this key's [`Keypair::sign_message`] signature
    /// over `message`.
    pub fn verify_message(&self, message: &[u8], signature: &[u8]) -> Result<bool, CoreError> {
        verify_signing_data(&self.public_key, message, signature)
    }

    /// Sign XRPL signing data (a hash prefix plus serialized fields) the way
    /// rippled verifies it: the same as [`Keypair::sign_message`].
    pub fn sign_signing_data(&self, data: &[u8]) -> Result<Vec<u8>, CoreError> {
        self.sign_message(data)
    }

    /// The raw primitive: Ed25519 signs `message`; secp256k1 signs
    /// `message` as an already-computed 32-byte hash (any other length is an
    /// error). The two key types mean different things by the same call, so
    /// prefer [`Keypair::sign_message`].
    #[deprecated(
        since = "0.6.0",
        note = "use sign_message (hashes for secp256k1 as rippled verifies); sign signs a 32-byte hash for secp256k1"
    )]
    pub fn sign(&self, message: &[u8]) -> Result<Vec<u8>, CoreError> {
        match self.key_type {
            KeyType::Ed25519 => super::ed25519::sign(&self.private_key, message),
            KeyType::Secp256k1 => super::secp256k1::sign(&self.private_key, message),
        }
    }

    /// Verify a [`Keypair::sign`] signature (secp256k1: over a 32-byte hash).
    #[deprecated(since = "0.6.0", note = "use verify_message")]
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> Result<bool, CoreError> {
        match self.key_type {
            KeyType::Ed25519 => super::ed25519::verify(&self.public_key, message, signature),
            KeyType::Secp256k1 => super::secp256k1::verify(&self.public_key, message, signature),
        }
    }
}

impl fmt::Debug for Keypair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Keypair")
            .field("key_type", &self.key_type)
            .field("public_key", &hex::encode_upper(&self.public_key))
            .field("private_key", &"[REDACTED]")
            .finish()
    }
}

/// Whether `signature` is `public_key`'s signature over XRPL signing data
/// (or any message), as `rippled` checks it: the key type is the key's
/// first byte (0xED for Ed25519), Ed25519 over the data itself, secp256k1
/// over SHA-512Half of it.
pub fn verify_signing_data(
    public_key: &[u8],
    data: &[u8],
    signature: &[u8],
) -> Result<bool, CoreError> {
    if public_key.first() == Some(&0xED) {
        super::ed25519::verify(public_key, data, signature)
    } else {
        super::secp256k1::verify(public_key, &sha512_half(data), signature)
    }
}

/// 16 random bytes from the operating system (`getrandom`), never a
/// user-space generator: a forked process must not repeat its parent's seeds.
#[cfg(all(feature = "std", not(target_arch = "wasm32")))]
#[allow(
    clippy::expect_used,
    reason = "no OS random source means no safe key; there is nothing to fall back to"
)]
fn generate_random_bytes_16() -> [u8; 16] {
    use rand::TryRngCore;
    let mut bytes = [0u8; 16];
    rand::rngs::OsRng
        .try_fill_bytes(&mut bytes)
        .expect("the operating system's random source failed");
    bytes
}

/// Generate 16 random bytes using browser WebCrypto (WASM).
#[cfg(all(feature = "std", target_arch = "wasm32"))]
#[allow(
    clippy::expect_used,
    reason = "no CSPRNG means no safe key; there is nothing to fall back to"
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

    // A multi-signed transaction is signed by each signer, not by one key.
    if obj
        .get("Signers")
        .is_some_and(|s| !s.as_array().is_some_and(Vec::is_empty))
    {
        return Err(CoreError::ValidationError(
            "the transaction has Signers, so it is multi-signed: each signer signs it \
             for multi-signing instead"
                .to_string(),
        ));
    }

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
        assert_eq!(decoded.key_type(), KeyType::Secp256k1);
        assert_eq!(decoded.as_bytes(), seed.as_bytes());
        assert_eq!(decoded.to_base58(), encoded);
    }

    #[test]
    fn seed_roundtrip_ed25519() {
        let seed = Seed::generate();
        let encoded = seed.to_base58();
        let decoded = Seed::from_base58(&encoded).unwrap();
        assert_eq!(decoded.key_type(), KeyType::Ed25519);
        assert_eq!(decoded.as_bytes(), seed.as_bytes());
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

        let message = b"test transaction data";
        let sig = keypair.sign_message(message).unwrap();
        assert!(keypair.verify_message(message, &sig).unwrap());
    }

    #[test]
    fn sign_verify_roundtrip_secp256k1() {
        let seed = Seed::generate_with_type(KeyType::Secp256k1);
        let keypair = Keypair::from_seed(&seed).unwrap();

        let message = b"test transaction data";
        let sig = keypair.sign_message(message).unwrap();
        assert!(keypair.verify_message(message, &sig).unwrap());
        assert!(!keypair.verify_message(b"other data", &sig).unwrap());
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
        assert_eq!(seed.key_type(), key_type);
        let keypair = Keypair::from_seed(&seed).unwrap();
        assert_eq!(hex::encode_upper(keypair.private_key()), private_key[2..]);
        assert_eq!(hex::encode_upper(keypair.public_key()), public_key);
        // The secret key alone gives the same pair.
        let again = Keypair::from_private_key(key_type, keypair.private_key()).unwrap();
        assert_eq!(again.public_key(), keypair.public_key());
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
        let sig = "30440220583A91C95E54E6A651C47BEC22744E0B101E2C4060E7B08F6341657DAD9BC3EE02207D1489C7395DB0188D3A56A977ECBA54B36FA9371B40319655B1B4429E33EF2D";
        assert_eq!(
            hex::encode_upper(keypair.sign_message(b"test message").unwrap()),
            sig
        );
        assert!(keypair
            .verify_message(b"test message", &hex::decode(sig).unwrap())
            .unwrap());
        // The raw primitive over the hash is the same signature.
        let hash = sha512_half(b"test message");
        assert_eq!(
            hex::encode_upper(super::super::secp256k1::sign(keypair.private_key(), &hash).unwrap()),
            sig
        );
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
            hex::encode_upper(keypair.sign_message(b"test message").unwrap()),
            sig
        );
        assert!(keypair
            .verify_message(b"test message", &hex::decode(sig).unwrap())
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

    /// `{:?}` on a seed or key pair never shows a secret.
    #[test]
    fn debug_shows_no_secret() {
        let seed = Seed::from_base58(SECP_SEED).unwrap();
        let keypair = Keypair::from_seed(&seed).unwrap();
        let (s, k) = (format!("{seed:?}"), format!("{keypair:?}"));
        assert!(s.contains("REDACTED") && !s.contains("1, 2, 3"), "{s}");
        let secret = hex::encode_upper(keypair.private_key());
        assert!(k.contains("REDACTED") && !k.contains(&secret), "{k}");
        assert!(!k.contains("215, 139"), "{k}");
    }

    /// secp256k1 signs exactly a 32-byte hash. The ECDSA primitive would
    /// use only the first 32 bytes of a longer input, so a signature over
    /// one message would verify for any message sharing those bytes.
    #[test]
    fn secp256k1_refuses_anything_but_a_32_byte_hash() {
        use super::super::secp256k1;
        let keypair = Keypair::from_seed(&Seed::from_base58(SECP_SEED).unwrap()).unwrap();
        let mut m1 = vec![0xAAu8; 32];
        m1.extend_from_slice(b"pay 1 XRP");
        let mut m2 = vec![0xAAu8; 32];
        m2.extend_from_slice(b"pay 1000000 XRP to mallory");
        assert!(secp256k1::sign(keypair.private_key(), &m1).is_err());
        assert!(secp256k1::sign(keypair.private_key(), &m1[..20]).is_err());
        let sig = keypair.sign_message(&m1).unwrap();
        assert!(keypair.verify_message(&m1, &sig).unwrap());
        assert!(!keypair.verify_message(&m2, &sig).unwrap());
        assert!(secp256k1::verify(keypair.public_key(), &m2, &sig).is_err());
        // Only rippled's 33-byte compressed public keys.
        let uncompressed = {
            use k256::elliptic_curve::sec1::ToEncodedPoint;
            k256::PublicKey::from_sec1_bytes(keypair.public_key())
                .unwrap()
                .to_encoded_point(false)
                .as_bytes()
                .to_vec()
        };
        assert!(secp256k1::verify(&uncompressed, &sha512_half(&m1), &sig).is_err());
    }

    #[test]
    fn a_keypair_from_a_secret_key_has_its_public_key() {
        for key_type in [KeyType::Ed25519, KeyType::Secp256k1] {
            let keypair = Keypair::from_seed(&Seed::generate_with_type(key_type)).unwrap();
            let again = Keypair::from_private_key(key_type, keypair.private_key()).unwrap();
            assert_eq!(again.public_key(), keypair.public_key());
            assert_eq!(again.classic_address(), keypair.classic_address());
        }
        assert!(Keypair::from_private_key(KeyType::Secp256k1, &[0u8; 32]).is_err());
        assert!(Keypair::from_private_key(KeyType::Ed25519, &[1u8; 31]).is_err());
    }

    #[test]
    fn a_multi_signed_transaction_is_not_single_signed() {
        let keypair = Keypair::from_seed(&Seed::from_base58(SECP_SEED).unwrap()).unwrap();
        let tx = serde_json::json!({"TransactionType": "AccountSet", "Account": SECP_ADDRESS,
            "Fee": "12", "Sequence": 1, "SigningPubKey": "",
            "Signers": [{"Signer": {"Account": ED_ADDRESS, "SigningPubKey": "ED01", "TxnSignature": "AA"}}]});
        assert!(sign_transaction(&tx, &keypair).is_err());
    }
}
