use xrpl_core::address::KeyType;
use xrpl_core::codec::encode_transaction_json;
use xrpl_core::crypto::hd::DerivationPath;
use xrpl_core::crypto::mnemonic::Mnemonic;
use xrpl_core::crypto::{sign_transaction, verify_signing_data, Keypair, Seed};
use xrpl_core::types::AccountId;
use xrpl_core::CoreError;
use zeroize::Zeroizing;

/// The most signers a multi-signed transaction may carry (`rippled`'s
/// `STTx::maxMultiSigners`).
pub const MAX_MULTISIGNERS: usize = 32;

/// High-level wallet for XRPL transactions: a key pair, its address and,
/// when it was made from one, its seed. `Debug` shows the address and key
/// type only; the secrets are wiped on drop.
#[derive(Clone)]
pub struct Wallet {
    keypair: Keypair,
    address: String,
    account_id: AccountId,
    seed: Option<Seed>,
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
        Self::generate_with_type(KeyType::Ed25519)
    }

    /// Generate a new random wallet with a specific key type.
    #[allow(
        clippy::expect_used,
        reason = "a random 16-byte seed always derives a keypair"
    )]
    pub fn generate_with_type(key_type: KeyType) -> Self {
        let seed = Seed::generate_with_type(key_type);
        let keypair =
            Keypair::from_seed(&seed).expect("freshly generated seed must produce valid keypair");
        Self::from_keypair(keypair).with_seed(seed)
    }

    /// Create a wallet from a base58-encoded seed string.
    pub fn from_seed(seed_str: &str) -> Result<Self, CoreError> {
        Self::from_seed_value(Seed::from_base58(seed_str)?)
    }

    /// Alias for `from_seed` — XRPL often uses "secret" to mean seed.
    pub fn from_secret(secret: &str) -> Result<Self, CoreError> {
        Self::from_seed(secret)
    }

    /// Open a wallet from a BIP-39 recovery phrase (12 to 24 words, as
    /// Ledger devices and multi-coin wallets show), with no passphrase, at
    /// the first XRP account: `m/44'/144'/0'/0/0`, the path Ledger and
    /// xrpl.js use. The key is secp256k1 and the wallet has no XRPL seed
    /// ([`Wallet::seed`] is `None`): the phrase is its backup.
    ///
    /// ```
    /// use xrpl_sdk::Wallet;
    ///
    /// let wallet = Wallet::from_mnemonic(
    ///     "assault rare scout seed design extend noble drink talk control guitar quote",
    /// )?;
    /// assert_eq!(wallet.classic_address(), "rJUr92pXcNtH6PtFfbRhHCo469XmPrbTJU");
    /// # Ok::<(), xrpl_sdk::CoreError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// [`CoreError::InvalidMnemonic`] if a word is not on the BIP-39 English
    /// list, the word count is wrong or the checksum does not match.
    pub fn from_mnemonic(phrase: &str) -> Result<Self, CoreError> {
        Self::from_mnemonic_with(phrase, "", &DerivationPath::default())
    }

    /// Open a wallet from a BIP-39 recovery phrase with a passphrase (the
    /// "25th word"; `""` for none) at `path`, for example
    /// [`DerivationPath::xrp_account`] for a later account. A wrong
    /// passphrase or path is not an error: it opens a different, empty
    /// wallet, so check the address.
    ///
    /// # Errors
    ///
    /// As [`Wallet::from_mnemonic`].
    pub fn from_mnemonic_with(
        phrase: &str,
        passphrase: &str,
        path: &DerivationPath,
    ) -> Result<Self, CoreError> {
        let mnemonic = Mnemonic::parse(phrase)?;
        Ok(Self::from_keypair(Keypair::from_mnemonic(
            &mnemonic, passphrase, path,
        )?))
    }

    /// Open a wallet from Xaman's secret numbers (eight blocks of six
    /// digits; separators and row letters are skipped). The key is
    /// secp256k1, as Xaman makes it and as xrpl.js reads it; use
    /// [`Wallet::from_secret_numbers_with_type`] for Ed25519.
    ///
    /// ```
    /// use xrpl_sdk::Wallet;
    ///
    /// let wallet = Wallet::from_secret_numbers(
    ///     "399150 474506 009147 088773 432160 282843 253738 605430",
    /// )?;
    /// assert_eq!(wallet.classic_address(), "rQKQsPeE3iTRyfUypLhuq74gZdcRdwWqDp");
    /// # Ok::<(), xrpl_sdk::CoreError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// [`CoreError::InvalidSeed`] if there are not 48 digits or a block
    /// fails its check digit.
    pub fn from_secret_numbers(numbers: &str) -> Result<Self, CoreError> {
        Self::from_secret_numbers_with_type(numbers, KeyType::Secp256k1)
    }

    /// Open a wallet from secret numbers with the given key type.
    ///
    /// # Errors
    ///
    /// As [`Wallet::from_secret_numbers`].
    pub fn from_secret_numbers_with_type(
        numbers: &str,
        key_type: KeyType,
    ) -> Result<Self, CoreError> {
        Self::from_seed_value(Seed::from_secret_numbers(numbers, key_type)?)
    }

    /// Open a wallet from an RFC 1751 key: the twelve words `rippled`'s
    /// `wallet_propose` returns as `master_key`. The words do not record the
    /// key type, so give the one `wallet_propose` was asked for (its
    /// `key_type`; `rippled`'s default is secp256k1).
    ///
    /// ```
    /// use xrpl_sdk::{KeyType, Wallet};
    ///
    /// let wallet = Wallet::from_rfc1751(
    ///     "YOKE DREW DIME DRUM LAIN NIBS DUAL PEA BAIT CHOW JOY FILE",
    ///     KeyType::Secp256k1,
    /// )?;
    /// assert_eq!(wallet.classic_address(), "rNiNSFyhVr5xfp8o8G5Ku81if8rdDrai5z");
    /// # Ok::<(), xrpl_sdk::CoreError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// [`CoreError::InvalidSeed`] if there are not twelve dictionary words or
    /// a half fails its parity check.
    pub fn from_rfc1751(words: &str, key_type: KeyType) -> Result<Self, CoreError> {
        Self::from_seed_value(Seed::from_rfc1751(words, key_type)?)
    }

    fn from_seed_value(seed: Seed) -> Result<Self, CoreError> {
        let keypair = Keypair::from_seed(&seed)?;
        Ok(Self::from_keypair(keypair).with_seed(seed))
    }

    /// Create a wallet from an existing keypair.
    pub fn from_keypair(keypair: Keypair) -> Self {
        let account_id = keypair.account_id();
        let address = keypair.classic_address();
        Self {
            keypair,
            address,
            account_id,
            seed: None,
        }
    }

    fn with_seed(mut self, seed: Seed) -> Self {
        self.seed = Some(seed);
        self
    }

    /// The seed (`s…`), to store and restore this wallet with
    /// [`Wallet::from_seed`], in a string wiped on drop. `None` for a wallet
    /// made from a bare keypair.
    pub fn seed(&self) -> Option<Zeroizing<String>> {
        self.seed.as_ref().map(Seed::to_base58)
    }

    /// The key pair.
    pub fn keypair(&self) -> &Keypair {
        &self.keypair
    }

    /// The key type.
    pub fn key_type(&self) -> KeyType {
        self.keypair.key_type()
    }

    /// The 20-byte account ID.
    pub fn account_id(&self) -> AccountId {
        self.account_id
    }

    /// Sign a `Batch` as one of its participants (an account with an inner
    /// transaction that is not the batch's own account): the `BatchSigner`
    /// to add with `transaction::batch::add_batch_signers`.
    pub fn sign_batch(
        &self,
        batch: &serde_json::Value,
    ) -> Result<xrpl_core::transaction::models::BatchSigner, CoreError> {
        xrpl_core::transaction::batch::sign_batch(batch, &self.keypair)
    }

    /// Sign a `Batch` for `account`, an account whose regular key this
    /// wallet holds: the `BatchSigner` to add with
    /// `transaction::batch::add_batch_signers`.
    pub fn sign_batch_as(
        &self,
        batch: &serde_json::Value,
        account: &str,
    ) -> Result<xrpl_core::transaction::models::BatchSigner, CoreError> {
        xrpl_core::transaction::batch::sign_batch_as(batch, account, &self.keypair)
    }

    /// Sign a `Batch` for `batch_signer`, an account whose signer list this
    /// wallet is on: one of the `Signers` of its multi-signed `BatchSigner`.
    pub fn sign_batch_for(
        &self,
        batch: &serde_json::Value,
        batch_signer: &str,
    ) -> Result<xrpl_core::transaction::models::Signer, CoreError> {
        xrpl_core::transaction::batch::sign_batch_for(batch, batch_signer, &self.keypair)
    }

    /// Sign a payment channel claim for `amount_drops` (the channel's total
    /// paid out, not an increment) with this wallet's key.
    pub fn sign_payment_channel_claim(
        &self,
        channel_id: &str,
        amount_drops: u64,
    ) -> Result<String, CoreError> {
        xrpl_core::crypto::sign_payment_channel_claim(channel_id, amount_drops, &self.keypair)
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
        hex::encode_upper(self.keypair.public_key())
    }

    /// Sign a transaction as one of its multi-signers: the `Signer` entry for
    /// [`collect_signers`]. The transaction is signed with an empty
    /// `SigningPubKey` (a missing one counts as empty; a non-empty one is an
    /// error), as `rippled`'s `sign_for` does.
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

/// Put the `Signer` entries from [`Wallet::sign_for_multisigning`] on a
/// transaction as its `Signers`, ready to encode and submit.
///
/// Every signature is checked against the transaction first, so one signed
/// for a different transaction, or a changed one, is an error here rather
/// than a refusal from the network. The signers are sorted by 20-byte
/// AccountID, the order rippled requires (base58 text order is not the same
/// order); an account may sign once; at most [`MAX_MULTISIGNERS`] may sign.
/// `SigningPubKey` is set empty; a `TxnSignature` (a single signature) is an
/// error.
pub fn collect_signers(
    tx: &mut serde_json::Value,
    signers: Vec<serde_json::Value>,
) -> Result<(), CoreError> {
    let invalid = |m: String| CoreError::ValidationError(m);
    if signers.is_empty() || signers.len() > MAX_MULTISIGNERS {
        return Err(invalid(format!(
            "a multi-signed transaction has 1 to {MAX_MULTISIGNERS} signers, got {}",
            signers.len()
        )));
    }
    if tx.get("TxnSignature").is_some() {
        return Err(invalid(
            "the transaction has a TxnSignature; a multi-signed one has none".to_string(),
        ));
    }
    let mut unsigned = tx.clone();
    if let Some(obj) = unsigned.as_object_mut() {
        obj.remove("Signers");
    }
    let mut keyed = signers
        .into_iter()
        .map(|signer| {
            let field = |name: &str| {
                signer
                    .get("Signer")
                    .and_then(|s| s.get(name))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| invalid(format!("a Signer entry has no {name}")))
            };
            let account = field("Account")?;
            let public_key = hex::decode(field("SigningPubKey")?)
                .map_err(|e| CoreError::InvalidHex(e.to_string()))?;
            let signature = hex::decode(field("TxnSignature")?)
                .map_err(|e| CoreError::InvalidHex(e.to_string()))?;
            let data = xrpl_core::codec::encode_for_multisigning(&unsigned, account)?;
            if !verify_signing_data(&public_key, &data, &signature)? {
                return Err(invalid(format!(
                    "{account}'s signature is not for this transaction"
                )));
            }
            Ok((xrpl_core::address::decode_account_id(account)?, signer))
        })
        .collect::<Result<Vec<_>, CoreError>>()?;
    keyed.sort_by_key(|a| a.0);
    if keyed.windows(2).any(|w| matches!(w, [a, b] if a.0 == b.0)) {
        return Err(invalid("an account signs a transaction once".to_string()));
    }

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
            .field("key_type", &self.keypair.key_type())
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
        assert_eq!(wallet.key_type(), KeyType::Secp256k1);
        // Round-trip: recreate from same seed and verify identical address
        let wallet2 = Wallet::from_seed(&seed_str).unwrap();
        assert_eq!(wallet.address, wallet2.address);
        assert_eq!(wallet.public_key_hex(), wallet2.public_key_hex());
        assert_eq!(wallet.seed().as_deref(), Some(&*seed_str));
    }

    #[test]
    fn a_generated_wallet_keeps_its_seed() {
        for wallet in [
            Wallet::generate(),
            Wallet::generate_with_type(KeyType::Secp256k1),
        ] {
            let restored = Wallet::from_seed(&wallet.seed().unwrap()).unwrap();
            assert_eq!(restored.address, wallet.address);
            assert_eq!(restored.key_type(), wallet.key_type());
        }
        let bare = Wallet::from_keypair(Wallet::generate().keypair().clone());
        assert!(bare.seed().is_none());
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

    fn multisigned_payment() -> serde_json::Value {
        serde_json::json!({
            "TransactionType": "Payment",
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
            "Amount": "1000000",
            "Fee": "36",
            "Sequence": 1
        })
    }

    /// Two wallets from fixed seeds whose addresses sort one way as text and
    /// the other way as 20-byte AccountIDs.
    fn text_and_byte_order_disagree() -> (Wallet, Wallet) {
        let wallets: Vec<Wallet> = (0u8..=255)
            .map(|i| {
                Wallet::from_keypair(
                    Seed::from_bytes([i; 16], KeyType::Ed25519)
                        .derive_keypair()
                        .unwrap(),
                )
            })
            .collect();
        for a in &wallets {
            for b in &wallets {
                if a.classic_address() < b.classic_address()
                    && a.account_id().as_bytes() > b.account_id().as_bytes()
                {
                    return (a.clone(), b.clone());
                }
            }
        }
        panic!("no such pair");
    }

    #[test]
    fn collect_signers_orders_by_account_id_bytes_not_base58() {
        // rippled (STTx.cpp) rejects "Unsorted Signers array" unless the
        // signers ascend by 20-byte AccountID; the base58 alphabet is not in
        // ASCII order.
        let (text_first, bytes_first) = text_and_byte_order_disagree();
        let mut tx = multisigned_payment();
        let signers = vec![
            text_first.sign_for_multisigning(&tx).unwrap(),
            bytes_first.sign_for_multisigning(&tx).unwrap(),
        ];
        super::collect_signers(&mut tx, signers).unwrap();
        assert_eq!(tx["SigningPubKey"], "");
        let arr = tx["Signers"].as_array().unwrap();
        assert_eq!(arr[0]["Signer"]["Account"], bytes_first.classic_address());
        assert_eq!(arr[1]["Signer"]["Account"], text_first.classic_address());
    }

    /// A signature over a different transaction, a repeated signer, too many
    /// signers or a single signature are refused before anything is sent.
    #[test]
    fn collect_signers_checks_what_it_collects() {
        let (a, b) = text_and_byte_order_disagree();
        let tx = multisigned_payment();
        let sign = |w: &Wallet, tx: &serde_json::Value| w.sign_for_multisigning(tx).unwrap();
        let mut other = tx.clone();
        other["Amount"] = serde_json::json!("2000000");
        assert!(
            super::collect_signers(&mut tx.clone(), vec![sign(&a, &tx), sign(&b, &other)]).is_err()
        );
        assert!(
            super::collect_signers(&mut tx.clone(), vec![sign(&a, &tx), sign(&a, &tx)]).is_err()
        );
        assert!(super::collect_signers(&mut tx.clone(), vec![]).is_err());
        let many = vec![sign(&a, &tx); MAX_MULTISIGNERS + 1];
        assert!(super::collect_signers(&mut tx.clone(), many).is_err());
        let mut single = tx.clone();
        single["TxnSignature"] = serde_json::json!("AA");
        assert!(super::collect_signers(&mut single, vec![sign(&a, &tx)]).is_err());
        // A signer handed the transaction without its empty SigningPubKey
        // signs the same data.
        let mut spelled = tx.clone();
        spelled["SigningPubKey"] = serde_json::json!("");
        assert_eq!(sign(&a, &spelled), sign(&a, &tx));
        let mut ok = tx.clone();
        super::collect_signers(&mut ok, vec![sign(&a, &spelled), sign(&b, &tx)]).unwrap();
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
        assert_ne!(ed.classic_address(), sec.classic_address());
        // Ed25519 public key starts with 0xED
        assert!(ed.public_key_hex().starts_with("ED"));
    }
}
