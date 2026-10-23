//! Ledger hashes, as `rippled` computes them: the hash of a ledger header,
//! and the roots of its transaction and state trees (SHAMaps).
//!
//! A ledger's hash covers its header, and the header holds the root hash of
//! the tree of its transactions (`transaction_hash`) and of its state
//! (`account_hash`). Rebuilding the transaction tree from a ledger's
//! transactions and comparing it with the header, then hashing the header
//! and comparing it with the ledger's hash, shows that the transactions are
//! exactly the ones that ledger holds: none added, removed or changed.
//! Whether the ledger hash itself is the network's is a separate question,
//! answered by comparing it with one from a source you trust (another
//! server, or the validations of trusted validators).
//!
//! ```text
//! ledger hash  = SHA-512Half("LWR\0" ‖ header, 118 bytes)
//! tx leaf      = SHA-512Half("SND\0" ‖ VL(tx) ‖ VL(meta) ‖ tx ID)
//! state leaf   = SHA-512Half("MLN\0" ‖ entry ‖ entry ID)
//! inner node   = SHA-512Half("MIN\0" ‖ 16 child hashes, zero for an empty branch)
//! ```
//!
//! A tree's root is always an inner node; below it, a branch that holds one
//! item is that item's leaf. An empty tree hashes to zero.

use crate::codec::field::encode_vl;
use crate::crypto::sha512_half;
use crate::types::Hash256;
use crate::CoreError;

/// A ledger header: what the ledger's hash covers.
///
/// ```
/// use xrpl_core::ledger::LedgerHeader;
///
/// // Mainnet ledger 107852951, as rippled's `ledger` method returns it.
/// let header = LedgerHeader::from_json(&serde_json::json!({
///     "ledger_index": "107852951",
///     "total_coins": "99985585808150845",
///     "parent_hash": "711923FB0F74D50DB0413DBD0B53CB555C8013F899A4B0D3F0293E005969C17C",
///     "transaction_hash": "6AEA36387F6EC51B3885D9CA6BD34017A7DC69B2EA1FFC3CD486ABE8D27D019D",
///     "account_hash": "8A981B51E1282D88C0D0E5EADDA02B4A3683F655F6C5464A5178F8104631BA39",
///     "parent_close_time": 846088132,
///     "close_time": 846088140,
///     "close_time_resolution": 10,
///     "close_flags": 0
/// }))?;
/// assert_eq!(
///     header.hash().to_string(),
///     "76768FFFD98D55385887C1018EB1074A9F8745512E84AB09250EE76B54BEDC93"
/// );
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LedgerHeader {
    /// The ledger's sequence number (`ledger_index`).
    pub ledger_index: u32,
    /// The XRP in existence, in drops (`total_coins`).
    pub total_coins: u64,
    /// The previous ledger's hash (`parent_hash`).
    pub parent_hash: Hash256,
    /// The root of the transaction tree (`transaction_hash`).
    pub transaction_hash: Hash256,
    /// The root of the state tree (`account_hash`).
    pub account_hash: Hash256,
    /// The previous ledger's close time, in seconds since 2000-01-01 UTC
    /// (`parent_close_time`).
    pub parent_close_time: u32,
    /// The close time, in seconds since 2000-01-01 UTC (`close_time`).
    pub close_time: u32,
    /// How finely the close time was rounded, in seconds
    /// (`close_time_resolution`).
    pub close_time_resolution: u8,
    /// Close flags (`close_flags`; 1 = the validators did not agree on a
    /// close time).
    pub close_flags: u8,
}

impl LedgerHeader {
    /// A header's length when serialized.
    pub const SIZE: usize = 118;

    /// The header's bytes, as `rippled` serializes it (and as its `ledger`
    /// method returns it in `ledger_data` with `binary: true`).
    #[must_use]
    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut out = [0u8; Self::SIZE];
        let parts: [&[u8]; 9] = [
            &self.ledger_index.to_be_bytes(),
            &self.total_coins.to_be_bytes(),
            &self.parent_hash.0,
            &self.transaction_hash.0,
            &self.account_hash.0,
            &self.parent_close_time.to_be_bytes(),
            &self.close_time.to_be_bytes(),
            &[self.close_time_resolution],
            &[self.close_flags],
        ];
        let mut rest: &mut [u8] = &mut out;
        for part in parts {
            let (head, tail) = rest.split_at_mut(part.len());
            head.copy_from_slice(part);
            rest = tail;
        }
        out
    }

    /// A header from its 118 bytes (`ledger_data` of a binary `ledger`).
    ///
    /// # Errors
    ///
    /// [`CoreError::ValidationError`] for any other length.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CoreError> {
        let bytes: &[u8; Self::SIZE] = bytes.try_into().map_err(|_| {
            CoreError::ValidationError(format!(
                "a ledger header is {} bytes, not {}",
                Self::SIZE,
                bytes.len()
            ))
        })?;
        let (ledger_index, rest) = take::<4>(bytes);
        let (total_coins, rest) = take::<8>(rest);
        let (parent_hash, rest) = take::<32>(rest);
        let (transaction_hash, rest) = take::<32>(rest);
        let (account_hash, rest) = take::<32>(rest);
        let (parent_close_time, rest) = take::<4>(rest);
        let (close_time, rest) = take::<4>(rest);
        let ([close_time_resolution], rest) = take::<1>(rest);
        let ([close_flags], _) = take::<1>(rest);
        Ok(Self {
            ledger_index: u32::from_be_bytes(ledger_index),
            total_coins: u64::from_be_bytes(total_coins),
            parent_hash: Hash256(parent_hash),
            transaction_hash: Hash256(transaction_hash),
            account_hash: Hash256(account_hash),
            parent_close_time: u32::from_be_bytes(parent_close_time),
            close_time: u32::from_be_bytes(close_time),
            close_time_resolution,
            close_flags,
        })
    }

    /// A header from `rippled`'s JSON for it (the `ledger` object of its
    /// `ledger` method): `ledger_index` and `total_coins` may be strings (API
    /// version 1) or numbers.
    ///
    /// # Errors
    ///
    /// [`CoreError::ValidationError`] for a missing or malformed field.
    pub fn from_json(header: &serde_json::Value) -> Result<Self, CoreError> {
        let field = |name: &str| {
            header.get(name).ok_or_else(|| {
                CoreError::ValidationError(format!("the ledger header has no {name}"))
            })
        };
        let number = |name: &str| -> Result<u64, CoreError> {
            let value = field(name)?;
            value
                .as_u64()
                .or_else(|| value.as_str().and_then(|s| s.parse().ok()))
                .ok_or_else(|| CoreError::ValidationError(format!("{name} is not a whole number")))
        };
        let small = |name: &str| -> Result<u32, CoreError> {
            u32::try_from(number(name)?)
                .map_err(|_| CoreError::ValidationError(format!("{name} is above 2^32")))
        };
        let byte = |name: &str| -> Result<u8, CoreError> {
            u8::try_from(number(name)?)
                .map_err(|_| CoreError::ValidationError(format!("{name} is above 255")))
        };
        let hash = |name: &str| -> Result<Hash256, CoreError> {
            Hash256::from_hex(field(name)?.as_str().unwrap_or_default())
        };
        Ok(Self {
            ledger_index: small("ledger_index")?,
            total_coins: number("total_coins")?,
            parent_hash: hash("parent_hash")?,
            transaction_hash: hash("transaction_hash")?,
            account_hash: hash("account_hash")?,
            parent_close_time: small("parent_close_time")?,
            close_time: small("close_time")?,
            close_time_resolution: byte("close_time_resolution")?,
            close_flags: byte("close_flags")?,
        })
    }

    /// The ledger's hash: SHA-512Half(`"LWR\0"` ‖ the header's bytes).
    #[must_use]
    pub fn hash(&self) -> Hash256 {
        prefixed_hash(b"LWR\0", &[&self.to_bytes()])
    }
}

/// A ledger whose transactions were checked against its hash: see
/// [`verify_ledger_response`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct VerifiedLedger {
    /// The ledger's hash, recomputed from its header.
    pub hash: Hash256,
    /// The header.
    pub header: LedgerHeader,
    /// Whether the server said the ledger is validated.
    pub validated: bool,
    /// Its transactions, in the order the server listed them.
    pub transactions: Vec<LedgerTransaction>,
}

/// One transaction of a [`VerifiedLedger`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct LedgerTransaction {
    /// The transaction's ID (its hash), recomputed from its bytes.
    pub id: Hash256,
    /// The signed transaction.
    pub tx_blob: Vec<u8>,
    /// Its metadata (what it did).
    pub meta: Vec<u8>,
}

impl LedgerTransaction {
    /// The transaction as `rippled`'s JSON.
    ///
    /// # Errors
    ///
    /// A codec error if it does not decode.
    pub fn tx_json(&self) -> Result<serde_json::Value, CoreError> {
        crate::codec::decode_transaction_binary(&self.tx_blob)
    }

    /// The metadata as `rippled`'s JSON.
    ///
    /// # Errors
    ///
    /// A codec error if it does not decode.
    pub fn meta_json(&self) -> Result<serde_json::Value, CoreError> {
        crate::codec::decode_transaction_binary(&self.meta)
    }
}

/// Check the answer of `rippled`'s `ledger` method (API version 2), asked
/// with `binary`, `transactions` and `expand` all `true`: the header
/// (`ledger_data`) hashes to the `ledger_hash` the server named, each
/// transaction (`tx_blob`) to the `hash` beside it, and the transactions
/// with their metadata (`meta_blob`) rebuild the header's
/// `transaction_hash`. Then the transactions are exactly the
/// ledger's. Compare [`VerifiedLedger::hash`] with a hash from a source you
/// trust to know the ledger is the network's.
///
/// # Errors
///
/// [`CoreError::ValidationError`] when the answer is not that shape, the
/// header does not hash to `ledger_hash`, or the transactions do not
/// rebuild `transaction_hash` (one is missing, added or changed).
pub fn verify_ledger_response(result: &serde_json::Value) -> Result<VerifiedLedger, CoreError> {
    let invalid = |m: &str| CoreError::ValidationError(format!("ledger: {m}"));
    let hex_field = |value: Option<&serde_json::Value>, name: &str| -> Result<Vec<u8>, CoreError> {
        let text = value.and_then(serde_json::Value::as_str).ok_or_else(|| {
            invalid(&format!(
                "no {name} (ask with binary, transactions and expand)"
            ))
        })?;
        hex::decode(text).map_err(|e| CoreError::InvalidHex(format!("{name}: {e}")))
    };
    let ledger = result.get("ledger").ok_or_else(|| invalid("no ledger"))?;
    let header = LedgerHeader::from_bytes(&hex_field(ledger.get("ledger_data"), "ledger_data")?)?;
    let hash = header.hash();
    let named = Hash256::from_hex(
        result
            .get("ledger_hash")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| invalid("no ledger_hash"))?,
    )?;
    if hash != named {
        return Err(invalid(&format!(
            "the header hashes to {hash}, not the ledger_hash {named} the server named"
        )));
    }
    let mut transactions = Vec::new();
    for entry in ledger
        .get("transactions")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| invalid("no transactions (ask with transactions and expand)"))?
    {
        let tx_blob = hex_field(entry.get("tx_blob"), "tx_blob")?;
        let meta = hex_field(entry.get("meta_blob"), "meta_blob")?;
        let id = crate::hashes::transaction_id(&tx_blob);
        // The ID the server wrote beside it must be the transaction's.
        if let Some(named) = entry.get("hash").and_then(serde_json::Value::as_str) {
            if Hash256::from_hex(named)? != id {
                return Err(invalid(&format!(
                    "a transaction hashes to {id}, not the hash {named} written beside it"
                )));
            }
        }
        transactions.push(LedgerTransaction { id, tx_blob, meta });
    }
    let root = transaction_tree_hash(
        transactions
            .iter()
            .map(|t| (t.tx_blob.as_slice(), t.meta.as_slice())),
    )?;
    if root != header.transaction_hash {
        return Err(invalid(&format!(
            "the transactions rebuild {root}, not the header's transaction_hash {}",
            header.transaction_hash
        )));
    }
    Ok(VerifiedLedger {
        hash,
        header,
        validated: result
            .get("validated")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        transactions,
    })
}

/// The first `N` bytes of `bytes` and the rest; `bytes` is a header of the
/// right length, so it always has them.
fn take<const N: usize>(bytes: &[u8]) -> ([u8; N], &[u8]) {
    let (head, rest) = bytes.split_at(N.min(bytes.len()));
    let mut out = [0u8; N];
    out.copy_from_slice(head);
    (out, rest)
}

/// The root of a ledger's transaction tree (its header's
/// `transaction_hash`) from each transaction and its metadata, as bytes
/// (`tx_blob` and `meta` of a binary `ledger` with expanded transactions).
/// The order of the transactions does not matter.
///
/// # Errors
///
/// [`CoreError::ValidationError`] when two transactions have the same ID
/// (no ledger holds a transaction twice), and a codec error for a blob too
/// long for a length prefix.
pub fn transaction_tree_hash<'a>(
    transactions: impl IntoIterator<Item = (&'a [u8], &'a [u8])>,
) -> Result<Hash256, CoreError> {
    let mut leaves = Vec::new();
    for (tx, meta) in transactions {
        let id = crate::hashes::transaction_id(tx);
        let leaf = prefixed_hash(
            b"SND\0",
            &[
                &encode_vl(tx.len())?,
                tx,
                &encode_vl(meta.len())?,
                meta,
                &id.0,
            ],
        );
        leaves.push((id.0, leaf.0));
    }
    tree_hash(leaves)
}

/// The root of a ledger's state tree (its header's `account_hash`) from
/// each ledger entry's ID and bytes. A whole ledger's state is large (tens
/// of millions of entries on mainnet); this is for a state you hold.
///
/// # Errors
///
/// [`CoreError::ValidationError`] when two entries have the same ID.
pub fn state_tree_hash<'a>(
    entries: impl IntoIterator<Item = (Hash256, &'a [u8])>,
) -> Result<Hash256, CoreError> {
    let leaves = entries
        .into_iter()
        .map(|(id, data)| (id.0, prefixed_hash(b"MLN\0", &[data, &id.0]).0))
        .collect();
    tree_hash(leaves)
}

/// The root over `(key, leaf hash)` pairs.
fn tree_hash(mut leaves: Vec<([u8; 32], [u8; 32])>) -> Result<Hash256, CoreError> {
    if leaves.is_empty() {
        return Ok(Hash256::ZERO);
    }
    leaves.sort_unstable_by_key(|leaf| leaf.0);
    if leaves.windows(2).any(|w| matches!(w, [a, b] if a.0 == b.0)) {
        return Err(CoreError::ValidationError(
            "two items of the tree have the same ID".into(),
        ));
    }
    Ok(Hash256(inner_hash(&leaves, 0)))
}

/// An inner node over `leaves` (sorted, distinct keys that share their first
/// `depth` nibbles): each of its 16 branches holds the leaves whose next
/// nibble is the branch's number.
fn inner_hash(leaves: &[([u8; 32], [u8; 32])], depth: usize) -> [u8; 32] {
    let mut children = [[0u8; 32]; 16];
    let mut rest = leaves;
    while let Some(first) = rest.first() {
        let branch = nibble(&first.0, depth);
        let count = rest
            .iter()
            .take_while(|leaf| nibble(&leaf.0, depth) == branch)
            .count();
        let (group, tail) = rest.split_at(count);
        if let Some(child) = children.get_mut(usize::from(branch)) {
            *child = match group {
                [only] => only.1,
                _ => inner_hash(group, depth + 1),
            };
        }
        rest = tail;
    }
    let flat: Vec<u8> = children.iter().flatten().copied().collect();
    prefixed_hash(b"MIN\0", &[&flat]).0
}

/// Nibble `depth` of `key`: the high half of byte `depth / 2` when `depth`
/// is even, the low half when odd (`rippled`'s `SHAMapNodeID::selectBranch`).
fn nibble(key: &[u8; 32], depth: usize) -> u8 {
    let byte = key.get(depth / 2).copied().unwrap_or_default();
    if depth % 2 == 0 {
        byte >> 4
    } else {
        byte & 0x0F
    }
}

/// SHA-512Half of `prefix` and `parts`, concatenated.
fn prefixed_hash(prefix: &[u8; 4], parts: &[&[u8]]) -> Hash256 {
    let mut data = Vec::with_capacity(4 + parts.iter().map(|p| p.len()).sum::<usize>());
    data.extend_from_slice(prefix);
    for part in parts {
        data.extend_from_slice(part);
    }
    Hash256(sha512_half(&data))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn header_bytes_round_trip() {
        let header = LedgerHeader {
            ledger_index: 7,
            total_coins: 99_985_585_808_150_845,
            parent_hash: Hash256([0xAA; 32]),
            transaction_hash: Hash256([0xBB; 32]),
            account_hash: Hash256([0xCC; 32]),
            parent_close_time: 1,
            close_time: 2,
            close_time_resolution: 10,
            close_flags: 1,
        };
        let bytes = header.to_bytes();
        assert_eq!(LedgerHeader::from_bytes(&bytes).unwrap(), header);
        assert!(LedgerHeader::from_bytes(&bytes[..117]).is_err());
    }

    #[test]
    fn trees() {
        assert_eq!(state_tree_hash([]).unwrap(), Hash256::ZERO);
        // One item: the root is still an inner node over it.
        let one = state_tree_hash([(Hash256([0x11; 32]), &b"x"[..])]).unwrap();
        assert_ne!(one, Hash256::ZERO);
        // Order does not matter; a repeated ID is refused.
        let (a, b) = (
            (Hash256([0x10; 32]), &b"a"[..]),
            (Hash256([0x1F; 32]), &b"b"[..]),
        );
        assert_eq!(
            state_tree_hash([a, b]).unwrap(),
            state_tree_hash([b, a]).unwrap()
        );
        assert!(state_tree_hash([a, a]).is_err());
    }
}
