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
    /// Freshly generated random seeds always produce valid keypairs, so
    /// `unwrap_or_else` is used as a defensive fallback that should never trigger.
    pub fn generate() -> Self {
        let seed = Seed::generate();
        Self::from_keypair(
            Keypair::from_seed(&seed).expect("freshly generated seed must produce valid keypair"),
        )
    }

    /// Generate a new random wallet with a specific key type.
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
        use sha2::{Digest, Sha512};

        let encoded = xrpl_core::codec::encode_for_multisigning(tx, &self.address)?;

        // SHA512Half
        let hash: [u8; 64] = Sha512::digest(&encoded).into();
        let hash_bytes = &hash[..32];

        // Sign the hash
        let signature = self.keypair.sign(hash_bytes)?;

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
/// Sorts signers by Account address (required by rippled for canonical order).
pub fn collect_signers(tx: &mut serde_json::Value, mut signers: Vec<serde_json::Value>) {
    signers.sort_by(|a, b| {
        let acct_a = a
            .get("Signer")
            .and_then(|s| s.get("Account"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let acct_b = b
            .get("Signer")
            .and_then(|s| s.get("Account"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        acct_a.cmp(acct_b)
    });
    tx["Signers"] = serde_json::Value::Array(signers);
    tx["SigningPubKey"] = serde_json::Value::String(String::new());
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
    fn collect_signers_sorts_by_account() {
        let mut tx = serde_json::json!({
            "TransactionType": "Payment",
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
            "Amount": "1000000",
            "Fee": "12",
            "Sequence": 1
        });
        let signers = vec![
            serde_json::json!({"Signer": {"Account": "rZZZ", "SigningPubKey": "AA", "TxnSignature": "BB"}}),
            serde_json::json!({"Signer": {"Account": "rAAA", "SigningPubKey": "CC", "TxnSignature": "DD"}}),
        ];
        super::collect_signers(&mut tx, signers);
        assert_eq!(tx["SigningPubKey"], "");
        let arr = tx["Signers"].as_array().unwrap();
        assert_eq!(arr[0]["Signer"]["Account"], "rAAA");
        assert_eq!(arr[1]["Signer"]["Account"], "rZZZ");
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
    fn wallet_ed25519_vs_secp256k1() {
        let ed = Wallet::generate_with_type(KeyType::Ed25519);
        let sec = Wallet::generate_with_type(KeyType::Secp256k1);
        assert_ne!(ed.address, sec.address);
        // Ed25519 public key starts with 0xED
        assert!(ed.public_key_hex().starts_with("ED"));
    }
}
