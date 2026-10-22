use xrpl_core::address::KeyType;
use xrpl_core::codec::encode_transaction_json;
use xrpl_core::crypto::{sign_transaction, Keypair, Seed};
use xrpl_core::types::AccountId;
use xrpl_core::CoreError;

/// High-level wallet for XRPL transactions.
/// Wraps a keypair with convenient methods for signing and address derivation.
pub struct Wallet {
    pub keypair: Keypair,
    pub address: String,
    pub account_id: AccountId,
}

impl Wallet {
    /// Generate a new random wallet (Ed25519 by default).
    ///
    /// A freshly generated random seed always derives a valid keypair, so this
    /// is infallible; use [`Wallet::from_seed`] for seeds from elsewhere.
    #[allow(
        clippy::expect_used,
        reason = "a random 16-byte seed always derives a keypair"
    )]
    pub fn generate() -> Self {
        let seed = Seed::generate();
        Self::from_keypair(
            Keypair::from_seed(&seed).expect("freshly generated seed must produce valid keypair"),
        )
    }

    /// Generate a new random wallet with a specific key type.
    #[allow(
        clippy::expect_used,
        reason = "a random 16-byte seed always derives a keypair"
    )]
    pub fn generate_with_type(key_type: KeyType) -> Self {
        let seed = Seed::generate_with_type(key_type);
        Self::from_keypair(
            Keypair::from_seed(&seed).expect("freshly generated seed must produce valid keypair"),
        )
    }

    /// Create a wallet from a base58-encoded seed string.
    pub fn from_seed(seed_str: &str) -> Result<Self, CoreError> {
        let seed = Seed::from_base58(seed_str)?;
        let keypair = Keypair::from_seed(&seed)?;
        Ok(Self::from_keypair(keypair))
    }

    /// Alias for `from_seed` — XRPL often uses "secret" to mean seed.
    pub fn from_secret(secret: &str) -> Result<Self, CoreError> {
        Self::from_seed(secret)
    }

    /// Create a wallet from an existing keypair.
    pub fn from_keypair(keypair: Keypair) -> Self {
        let account_id = keypair.account_id();
        let address = keypair.classic_address();
        Self {
            keypair,
            address,
            account_id,
        }
    }

    /// Sign a transaction (JSON). Returns the signed tx JSON with
    /// SigningPubKey and TxnSignature fields set.
    pub fn sign_transaction(&self, tx: &serde_json::Value) -> Result<serde_json::Value, CoreError> {
        sign_transaction(tx, &self.keypair)
    }

    /// Sign a transaction and return the hex-encoded blob ready for submission.
    pub fn sign_and_encode(&self, tx: &serde_json::Value) -> Result<String, CoreError> {
        let signed = sign_transaction(tx, &self.keypair)?;
        let encoded = encode_transaction_json(&signed, false)?;
        Ok(hex::encode_upper(&encoded))
    }

    /// Get the classic address (starts with 'r').
    pub fn classic_address(&self) -> &str {
        &self.address
    }

    /// Get the public key as hex string.
    pub fn public_key_hex(&self) -> String {
        hex::encode_upper(&self.keypair.public_key)
    }

    /// Sign a transaction for multi-signing.
    /// Returns a Signer entry ready to be added to the Signers array.
    pub fn sign_for_multisigning(
        &self,
        tx: &serde_json::Value,
    ) -> Result<serde_json::Value, CoreError> {
        let encoded = xrpl_core::codec::encode_for_multisigning(tx, &self.address)?;
        let signature = self.keypair.sign_signing_data(&encoded)?;

        Ok(serde_json::json!({
            "Signer": {
                "Account": self.address,
                "SigningPubKey": self.public_key_hex(),
                "TxnSignature": hex::encode_upper(&signature)
            }
        }))
    }
}

/// Combine multiple Signer entries into the Signers array on a transaction.
/// Sorts signers by 20-byte AccountID, the order rippled requires. Base58
/// text order is not the same order, so every Account is decoded first.
pub fn collect_signers(
    tx: &mut serde_json::Value,
    signers: Vec<serde_json::Value>,
) -> Result<(), CoreError> {
    let mut keyed = signers
        .into_iter()
        .map(|signer| {
            let account = signer
                .get("Signer")
                .and_then(|s| s.get("Account"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    CoreError::InvalidAddress("Signer entry has no Account".to_string())
                })?;
            Ok((xrpl_core::address::decode_account_id(account)?, signer))
        })
        .collect::<Result<Vec<_>, CoreError>>()?;
    keyed.sort_by_key(|a| a.0);

    let obj = tx
        .as_object_mut()
        .ok_or_else(|| CoreError::CodecError("transaction must be a JSON object".to_string()))?;
    obj.insert(
        "Signers".to_string(),
        serde_json::Value::Array(keyed.into_iter().map(|(_, signer)| signer).collect()),
    );
    obj.insert(
        "SigningPubKey".to_string(),
        serde_json::Value::String(String::new()),
    );
    Ok(())
}

impl std::fmt::Debug for Wallet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Wallet")
            .field("address", &self.address)
            .field("key_type", &self.keypair.key_type)
            .finish()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn generate_wallet() {
        let wallet = Wallet::generate();
        assert!(wallet.address.starts_with('r'));
        assert!(!wallet.public_key_hex().is_empty());
    }

    #[test]
    fn wallet_from_seed_round_trip() {
        let seed = Seed::generate_with_type(KeyType::Secp256k1);
        let seed_str = seed.to_base58();
        let wallet = Wallet::from_seed(&seed_str).unwrap();
        assert!(wallet.address.starts_with('r'));
        assert_eq!(wallet.keypair.key_type, KeyType::Secp256k1);
        // Round-trip: recreate from same seed and verify identical address
        let wallet2 = Wallet::from_seed(&seed_str).unwrap();
        assert_eq!(wallet.address, wallet2.address);
        assert_eq!(wallet.public_key_hex(), wallet2.public_key_hex());
    }

    #[test]
    fn wallet_sign_transaction_round_trip() {
        let seed = Seed::generate_with_type(KeyType::Secp256k1);
        let seed_str = seed.to_base58();
        let wallet = Wallet::from_seed(&seed_str).unwrap();
        let tx = serde_json::json!({
            "TransactionType": "Payment",
            "Account": wallet.classic_address(),
            "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
            "Amount": "1000000",
            "Fee": "12",
            "Sequence": 1,
            "Flags": 0
        });

        let signed = wallet.sign_transaction(&tx).unwrap();
        assert!(signed["TxnSignature"].is_string());
        assert!(signed["SigningPubKey"].is_string());
        // Round-trip: same seed produces identical signature
        let wallet2 = Wallet::from_seed(&seed_str).unwrap();
        let signed2 = wallet2.sign_transaction(&tx).unwrap();
        assert_eq!(
            signed["TxnSignature"].as_str().unwrap(),
            signed2["TxnSignature"].as_str().unwrap()
        );
    }

    #[test]
    fn wallet_sign_and_encode() {
        let wallet = Wallet::generate();
        let tx = serde_json::json!({
            "TransactionType": "Payment",
            "Account": wallet.classic_address(),
            "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
            "Amount": "500000",
            "Fee": "12",
            "Sequence": 1,
            "Flags": 0
        });

        let blob = wallet.sign_and_encode(&tx).unwrap();
        assert!(!blob.is_empty());
        // Should be valid hex
        assert!(hex::decode(&blob).is_ok());
    }

    #[test]
    fn multisign_creates_signer_entry_round_trip() {
        let seed = Seed::generate_with_type(KeyType::Secp256k1);
        let seed_str = seed.to_base58();
        let wallet = Wallet::from_seed(&seed_str).unwrap();
        let tx = serde_json::json!({
            "TransactionType": "Payment",
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
            "Amount": "1000000",
            "Fee": "12",
            "Sequence": 1,
            "Flags": 0,
            "SigningPubKey": ""
        });
        let signer = wallet.sign_for_multisigning(&tx).unwrap();
        assert!(signer["Signer"]["Account"].is_string());
        assert!(signer["Signer"]["SigningPubKey"].is_string());
        assert!(signer["Signer"]["TxnSignature"].is_string());
        // Round-trip: same seed produces identical multisig signer entry
        let wallet2 = Wallet::from_seed(&seed_str).unwrap();
        let signer2 = wallet2.sign_for_multisigning(&tx).unwrap();
        assert_eq!(
            signer["Signer"]["TxnSignature"].as_str().unwrap(),
            signer2["Signer"]["TxnSignature"].as_str().unwrap()
        );
        assert_eq!(
            signer["Signer"]["Account"].as_str().unwrap(),
            signer2["Signer"]["Account"].as_str().unwrap()
        );
    }

    #[test]
    fn collect_signers_rejects_signer_without_valid_account() {
        let mut tx = serde_json::json!({"TransactionType": "Payment"});
        let bad = vec![
            serde_json::json!({"Signer": {"Account": "rZZZ", "SigningPubKey": "AA", "TxnSignature": "BB"}}),
        ];
        assert!(super::collect_signers(&mut tx, bad).is_err());
        let missing = vec![serde_json::json!({"Signer": {"SigningPubKey": "AA"}})];
        assert!(super::collect_signers(&mut tx, missing).is_err());
        assert!(tx.get("Signers").is_none());
    }

    #[test]
    fn collect_signers_orders_by_account_id_bytes_not_base58() {
        use xrpl_core::address::decode_account_id;
        // rippled (STTx.cpp) rejects "Unsorted Signers array" unless the
        // signers ascend by 20-byte AccountID. The base58 alphabet is not in
        // ASCII order: 'P' < 'w' as text, but 'w' is the lower base58 digit.
        let low = "rw2ciyaNshpHe7bCHo4bRWq6pqqynnWKQg";
        let high = "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe";
        assert!(high < low, "precondition: text order puts `high` first");
        assert!(
            decode_account_id(low).unwrap() < decode_account_id(high).unwrap(),
            "precondition: AccountID order puts `low` first"
        );
        let mut tx = serde_json::json!({
            "TransactionType": "Payment",
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
            "Amount": "1000000",
            "Fee": "12",
            "Sequence": 1
        });
        let signers = vec![
            serde_json::json!({"Signer": {"Account": high, "SigningPubKey": "AA", "TxnSignature": "BB"}}),
            serde_json::json!({"Signer": {"Account": low, "SigningPubKey": "CC", "TxnSignature": "DD"}}),
        ];
        super::collect_signers(&mut tx, signers).unwrap();
        assert_eq!(tx["SigningPubKey"], "");
        let arr = tx["Signers"].as_array().unwrap();
        assert_eq!(arr[0]["Signer"]["Account"], low);
        assert_eq!(arr[1]["Signer"]["Account"], high);
    }

    #[test]
    fn multisign_two_wallets() {
        let w1 = Wallet::generate();
        let w2 = Wallet::generate();
        let tx = serde_json::json!({
            "TransactionType": "Payment",
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
            "Amount": "1000000",
            "Fee": "12",
            "Sequence": 1,
            "Flags": 0,
            "SigningPubKey": ""
        });
        let s1 = w1.sign_for_multisigning(&tx).unwrap();
        let s2 = w2.sign_for_multisigning(&tx).unwrap();
        // Signatures should be different
        assert_ne!(
            s1["Signer"]["TxnSignature"].as_str().unwrap(),
            s2["Signer"]["TxnSignature"].as_str().unwrap()
        );
    }

    #[test]
    fn sign_for_multisigning_matches_rippled() {
        // rippled 3.4.1 `sign_for` (offline) with xrpl.js's keypair fixture
        // seeds; both schemes are deterministic.
        let tx = serde_json::json!({
            "TransactionType": "Payment",
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
            "Amount": "1000000",
            "Fee": "30",
            "Sequence": 1,
            "Flags": 0,
            "SigningPubKey": ""
        });
        for (seed, sig) in [
            ("sp5fghtJtpUorTwvof1NpDXAzNwf5", "30450221009A24EEBC327296E293759F66713325FD9FDF60CF8D7FD9F31D9792CE3281709902203F5FEE128F51892B6A7278E28E8724F05ED55D9B062967B6FF866B53CA67A52B"),
            ("sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r", "1209DCDAE666452AB21CA541F6653A80290F96336C52872463AD1D2A9B9D69CC6480E4397FFC9EA0678E45AB93E85D2AFE2AD36D86898387270F851296408406"),
        ] {
            let signer = Wallet::from_seed(seed).unwrap().sign_for_multisigning(&tx).unwrap();
            assert_eq!(signer["Signer"]["TxnSignature"], sig, "{seed}");
        }
    }

    #[test]
    fn wallet_ed25519_vs_secp256k1() {
        let ed = Wallet::generate_with_type(KeyType::Ed25519);
        let sec = Wallet::generate_with_type(KeyType::Secp256k1);
        assert_ne!(ed.address, sec.address);
        // Ed25519 public key starts with 0xED
        assert!(ed.public_key_hex().starts_with("ED"));
    }
}
