use xrpl_core::address::KeyType;
use xrpl_core::codec::encode_transaction_json;
use xrpl_core::crypto::hd::DerivationPath;
use xrpl_core::crypto::mnemonic::Mnemonic;
use xrpl_core::crypto::{verify_signing_data, AsyncSigningKey, Keypair, Seed, SigningKey};
use xrpl_core::transaction::ToTxJson;
use xrpl_core::types::AccountId;
use xrpl_core::types::Hash256;
use xrpl_core::CoreError;
use zeroize::Zeroizing;

/// The most signers a multi-signed transaction may carry (`rippled`'s
/// `STTx::maxMultiSigners`).
pub const MAX_MULTISIGNERS: usize = 32;

/// A signed transaction: what [`Wallet::sign`], [`sign`] and [`sign_async`]
/// return.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct SignedTransaction {
    /// The signed transaction as JSON (`SigningPubKey` and `TxnSignature`
    /// set).
    pub tx_json: serde_json::Value,
    /// The blob to submit, in upper-case hex.
    pub tx_blob: String,
    /// The transaction's ID: the hash it has on the ledger, known before it
    /// is submitted (look it up with `tx` to learn its outcome).
    pub hash: Hash256,
}

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
    ///
    /// # Errors
    ///
    /// [`CoreError::InvalidSeed`] if `seed_str` is not a seed: not base58, a
    /// checksum that does not match, or the wrong length or prefix.
    pub fn from_seed(seed_str: &str) -> Result<Self, CoreError> {
        Self::from_seed_value(Seed::from_base58(seed_str)?)
    }

    /// Alias for `from_seed` — XRPL often uses "secret" to mean seed.
    ///
    /// # Errors
    ///
    /// As [`Wallet::from_seed`].
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
    ///
    /// # Errors
    ///
    /// The errors of
    /// [`batch_signing_data`](xrpl_core::transaction::batch::batch_signing_data):
    /// [`CoreError::ValidationError`] for a `batch` with no `Account`, no
    /// `Sequence` or `TicketSequence`, no `RawTransactions`, or an inner
    /// transaction neither signed nor marked `tfInnerBatchTxn`; an address
    /// error for an `Account` that is not a classic address; and a codec
    /// error for an inner transaction that does not encode.
    pub fn sign_batch(
        &self,
        batch: &serde_json::Value,
    ) -> Result<xrpl_core::transaction::models::BatchSigner, CoreError> {
        xrpl_core::transaction::batch::sign_batch(batch, &self.keypair)
    }

    /// Sign a `Batch` for `account`, an account whose regular key this
    /// wallet holds: the `BatchSigner` to add with
    /// `transaction::batch::add_batch_signers`.
    ///
    /// # Errors
    ///
    /// As [`Wallet::sign_batch`], and an address error
    /// ([`CoreError::InvalidAddress`], `InvalidBase58` or `InvalidChecksum`)
    /// for an `account` that is not a classic address.
    pub fn sign_batch_as(
        &self,
        batch: &serde_json::Value,
        account: &str,
    ) -> Result<xrpl_core::transaction::models::BatchSigner, CoreError> {
        xrpl_core::transaction::batch::sign_batch_as(batch, account, &self.keypair)
    }

    /// Sign a `Batch` for `batch_signer`, an account whose signer list this
    /// wallet is on: one of the `Signers` of its multi-signed `BatchSigner`.
    ///
    /// # Errors
    ///
    /// As [`Wallet::sign_batch`], and an address error
    /// ([`CoreError::InvalidAddress`], `InvalidBase58` or `InvalidChecksum`)
    /// for a `batch_signer` that is not a classic address.
    pub fn sign_batch_for(
        &self,
        batch: &serde_json::Value,
        batch_signer: &str,
    ) -> Result<xrpl_core::transaction::models::Signer, CoreError> {
        xrpl_core::transaction::batch::sign_batch_for(batch, batch_signer, &self.keypair)
    }

    /// Sign a payment channel claim for `amount_drops` (the channel's total
    /// paid out, not an increment) with this wallet's key.
    ///
    /// # Errors
    ///
    /// [`CoreError::InvalidHex`] if `channel_id` is not 32 bytes of hex.
    pub fn sign_payment_channel_claim(
        &self,
        channel_id: &str,
        amount_drops: u64,
    ) -> Result<String, CoreError> {
        xrpl_core::crypto::sign_payment_channel_claim(channel_id, amount_drops, &self.keypair)
    }

    /// Sign `tx` as it is: a typed transaction
    /// ([`models`](xrpl_core::transaction::models), checked first) or JSON.
    /// Fill in `Fee`, `Sequence` and `LastLedgerSequence` before
    /// (`Client::autofill`), or use [`crate::sign_and_submit`], which does it
    /// all. Returns the signed JSON, the blob to submit and the transaction's
    /// ID. (The same as [`sign`]`(tx, &wallet)`.)
    ///
    /// ```
    /// use xrpl_sdk::{models, Amount, TxModel, Wallet};
    ///
    /// let wallet = Wallet::from_seed("sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r")?;
    /// let payment = models::Payment::new(
    ///     wallet.classic_address(),
    ///     "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
    ///     Amount::xrp(1_000_000)?,
    /// )
    /// .with_fee(12)
    /// .with_sequence(1);
    /// let signed = wallet.sign(&payment)?;
    /// assert_eq!(signed.hash, xrpl_sdk::hash_signed_tx_blob(&signed.tx_blob)?);
    /// # Ok::<(), xrpl_sdk::CoreError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A [`CoreError`] when the transaction fails its checks, does not
    /// encode (an unknown field, a value that does not fit) or is
    /// multi-signed (`Signers` set: see [`Wallet::sign_for_multisigning`]).
    pub fn sign(&self, tx: &impl ToTxJson) -> Result<SignedTransaction, CoreError> {
        sign(tx, self)
    }

    /// Get the classic address (starts with 'r').
    pub fn classic_address(&self) -> &str {
        &self.address
    }

    /// The public key as the XRP Ledger writes it (33 bytes).
    pub fn public_key(&self) -> &[u8] {
        self.keypair.public_key()
    }

    /// The public key in upper-case hex, as `SigningPubKey` holds it.
    pub fn public_key_hex(&self) -> String {
        hex::encode_upper(self.keypair.public_key())
    }

    /// Sign `tx` (a typed transaction or JSON) as one of its multi-signers:
    /// this wallet's [`Signer`](xrpl_core::transaction::models::Signer) for
    /// [`collect_signers`]. The transaction is signed with an empty
    /// `SigningPubKey` (a missing one counts as empty; a non-empty one is an
    /// error), as `rippled`'s `sign_for` does.
    ///
    /// # Errors
    ///
    /// The errors of [`ToTxJson::to_tx_json`] (a model that fails its
    /// checks) and of
    /// [`encode_for_multisigning`](xrpl_core::codec::encode_for_multisigning):
    /// [`CoreError::ValidationError`] for a non-empty `SigningPubKey`, and a
    /// codec error for a field that does not encode.
    pub fn sign_for_multisigning(
        &self,
        tx: &impl ToTxJson,
    ) -> Result<xrpl_core::transaction::models::Signer, CoreError> {
        xrpl_core::crypto::sign_for_multisigning(&tx.to_tx_json()?, &self.address, self)
    }
}

impl SigningKey for Wallet {
    fn public_key(&self) -> &[u8] {
        self.keypair.public_key()
    }

    fn sign_signing_data(&self, data: &[u8]) -> Result<Vec<u8>, CoreError> {
        self.keypair.sign_signing_data(data)
    }
}

/// Sign `tx` (a typed transaction, checked first, or JSON) with `key`: a
/// [`Wallet`], a [`Keypair`], or a key of your own that never leaves an HSM
/// or a hardware wallet ([`SigningKey`]). Returns the signed JSON, the blob
/// to submit and the transaction's ID.
///
/// # Errors
///
/// As [`Wallet::sign`], and the key's own errors
/// ([`CoreError::SigningKeyFailed`]).
pub fn sign<K: SigningKey + ?Sized>(
    tx: &impl ToTxJson,
    key: &K,
) -> Result<SignedTransaction, CoreError> {
    signed(xrpl_core::crypto::sign_transaction(&tx.to_tx_json()?, key)?)
}

/// [`sign`] with an [`AsyncSigningKey`]: a remote key (a cloud KMS, a
/// signing service). The signature is checked against the key's public key
/// before it is used, so a key that signs the wrong thing is an error here,
/// not a refusal from the network.
///
/// # Errors
///
/// As [`sign`], and [`CoreError::SigningKeyFailed`] for a signature that
/// does not verify.
pub async fn sign_async<K: AsyncSigningKey + ?Sized>(
    tx: &impl ToTxJson,
    key: &K,
) -> Result<SignedTransaction, CoreError> {
    signed(xrpl_core::crypto::sign_transaction_async(&tx.to_tx_json()?, key).await?)
}

/// A signed transaction's JSON with its blob and ID.
fn signed(tx_json: serde_json::Value) -> Result<SignedTransaction, CoreError> {
    let blob = encode_transaction_json(&tx_json)?;
    Ok(SignedTransaction {
        hash: xrpl_core::hashes::transaction_id(&blob),
        tx_blob: hex::encode_upper(&blob),
        tx_json,
    })
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
///
/// # Errors
///
/// - [`CoreError::ValidationError`] for no signers or more than
///   [`MAX_MULTISIGNERS`], a transaction with a `TxnSignature`, a `Signer`
///   entry without its `Account`, `SigningPubKey` or `TxnSignature`, a
///   signature that is not for this transaction, or an account that signs
///   twice.
/// - [`CoreError::InvalidHex`], [`CoreError::InvalidKeyLength`] or
///   [`CoreError::SigningFailed`] for a key or signature that is malformed.
/// - The errors of
///   [`encode_for_multisigning`](xrpl_core::codec::encode_for_multisigning)
///   for the transaction or a signer's `Account`.
pub fn collect_signers(
    tx: &mut serde_json::Value,
    signers: Vec<xrpl_core::transaction::models::Signer>,
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
            let account = signer.account.as_str();
            let public_key = hex::decode(&signer.signing_pub_key)
                .map_err(|e| CoreError::InvalidHex(e.to_string()))?;
            let signature = hex::decode(&signer.txn_signature)
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
    let signers = keyed
        .into_iter()
        .map(|(_, signer)| {
            serde_json::to_value(signer)
                .map(|signer| serde_json::json!({ "Signer": signer }))
                .map_err(|e| CoreError::CodecError(e.to_string()))
        })
        .collect::<Result<Vec<_>, CoreError>>()?;
    obj.insert("Signers".to_string(), serde_json::Value::Array(signers));
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

        let signed = wallet.sign(&tx).unwrap().tx_json;
        assert!(signed["TxnSignature"].is_string());
        assert!(signed["SigningPubKey"].is_string());
        // Round-trip: same seed produces identical signature
        let wallet2 = Wallet::from_seed(&seed_str).unwrap();
        let signed2 = wallet2.sign(&tx).unwrap().tx_json;
        assert_eq!(
            signed["TxnSignature"].as_str().unwrap(),
            signed2["TxnSignature"].as_str().unwrap()
        );
    }

    /// A key of one's own, synchronous or remote, signs as the wallet does;
    /// a remote one that signs the wrong thing is refused before anything
    /// is submitted.
    #[tokio::test]
    async fn any_signing_key_signs_as_the_wallet_does() {
        use xrpl_core::crypto::{secp256k1, sha512_half};

        struct Remote {
            public_key: Vec<u8>,
            secret: Vec<u8>,
            hash_first: bool,
        }
        impl AsyncSigningKey for Remote {
            fn public_key(&self) -> &[u8] {
                &self.public_key
            }
            async fn sign_signing_data(&self, data: &[u8]) -> Result<Vec<u8>, CoreError> {
                tokio::task::yield_now().await;
                if self.hash_first {
                    secp256k1::sign(&self.secret, &sha512_half(data))
                } else {
                    // The mistake a KMS integration makes: no SHA-512Half.
                    secp256k1::sign(&self.secret, &data[data.len() - 32..])
                }
            }
        }

        let wallet = Wallet::generate_with_type(KeyType::Secp256k1);
        let tx = serde_json::json!({
            "TransactionType": "Payment",
            "Account": wallet.classic_address(),
            "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
            "Amount": "500000",
            "Fee": "12",
            "Sequence": 1,
        });
        let expected = wallet.sign(&tx).unwrap();
        assert_eq!(sign(&tx, wallet.keypair()).unwrap(), expected);
        assert_eq!(sign_async(&tx, &wallet).await.unwrap(), expected);
        let remote = |hash_first| Remote {
            public_key: wallet.public_key().to_vec(),
            secret: wallet.keypair().private_key().to_vec(),
            hash_first,
        };
        assert_eq!(sign_async(&tx, &remote(true)).await.unwrap(), expected);
        let refused = sign_async(&tx, &remote(false)).await.unwrap_err();
        assert!(
            matches!(refused, CoreError::SigningKeyFailed(_)),
            "{refused}"
        );
        // A key whose public key the XRP Ledger does not take is refused.
        let mut uncompressed = remote(true);
        uncompressed.public_key = vec![0x04; 65];
        assert!(matches!(
            sign_async(&tx, &uncompressed).await,
            Err(CoreError::SigningKeyFailed(_))
        ));
        // A boxed key works too.
        let boxed: Box<dyn SigningKey> = Box::new(wallet.keypair().clone());
        assert_eq!(sign(&tx, &*boxed).unwrap(), expected);
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
        assert_eq!(signer.account, wallet.classic_address());
        assert_eq!(signer.signing_pub_key, wallet.public_key_hex());
        // The same seed signs the same transaction the same way.
        let wallet2 = Wallet::from_seed(&seed_str).unwrap();
        assert_eq!(wallet2.sign_for_multisigning(&tx).unwrap(), signer);
    }

    #[test]
    fn collect_signers_rejects_signer_without_valid_account() {
        let mut tx = serde_json::json!({"TransactionType": "Payment"});
        let bad = vec![xrpl_core::transaction::models::Signer::new(
            "rZZZ", "AA", "BB",
        )];
        assert!(super::collect_signers(&mut tx, bad).is_err());
        let not_hex = vec![xrpl_core::transaction::models::Signer::new(
            "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
            "ZZ",
            "BB",
        )];
        assert!(super::collect_signers(&mut tx, not_hex).is_err());
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
        assert_ne!(s1.txn_signature, s2.txn_signature);
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
            assert_eq!(signer.txn_signature, sig, "{seed}");
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
    /// `sign` takes a typed model, a `Tx` or JSON and signs the same
    /// transaction from each; the hash is the blob's ID; a model `rippled`
    /// would refuse is refused before signing.
    #[test]
    fn sign_takes_every_form_of_a_transaction() {
        use xrpl_core::transaction::models::{self, TxModel};
        use xrpl_core::types::Amount;
        let wallet = Wallet::from_seed("sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r").unwrap();
        let destination = "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe";
        let json = serde_json::json!({
            "TransactionType": "Payment",
            "Account": wallet.classic_address(),
            "Destination": destination,
            "Amount": "1000000",
            "Fee": "12",
            "Sequence": 1,
        });
        let model = models::Payment::new(
            wallet.classic_address(),
            destination,
            Amount::xrp(1_000_000).unwrap(),
        )
        .with_fee(12)
        .with_sequence(1);
        let from_json = wallet.sign(&json).unwrap();
        assert_eq!(wallet.sign(&model).unwrap(), from_json);
        let refused = models::AMMVote::new(
            wallet.classic_address(),
            models::Asset::xrp(),
            models::Asset::token("USD", destination).unwrap(),
            1500,
        )
        .with_fee(12)
        .with_sequence(1);
        let err = wallet.sign(&refused).unwrap_err();
        assert!(err.to_string().contains("TradingFee"), "{err}");
        assert_eq!(wallet.sign(&models::Tx::Payment(model)).unwrap(), from_json);
        assert_eq!(
            from_json.hash,
            xrpl_core::hashes::hash_signed_tx_blob(&from_json.tx_blob).unwrap()
        );
        assert_eq!(
            from_json.tx_json,
            xrpl_core::crypto::sign_transaction(&json, &wallet).unwrap()
        );
        // A multi-signed transaction is refused, as sign_transaction refuses it.
        let mut multi = json.clone();
        multi["Signers"] = serde_json::json!([{"Signer": {}}]);
        assert!(wallet.sign(&multi).is_err());
    }
}
