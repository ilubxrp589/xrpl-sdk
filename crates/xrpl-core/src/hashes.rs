//! Hashes rippled derives: a transaction's ID, and the ID ("keylet") of every
//! kind of ledger entry, as `STTx::getTransactionID` and rippled's
//! `Indexes.cpp` compute them.

use crate::address::decode_account_id;
use crate::codec::{decode_transaction_binary, encode_transaction_json};
use crate::crypto::sha512_half;
use crate::types::{Currency, Hash192, Hash256};
use crate::CoreError;
use serde_json::Value;

/// `HashPrefix::transactionID` ("TXN\0").
const PREFIX_TRANSACTION_ID: [u8; 4] = *b"TXN\0";

/// `tfInnerBatchTxn`: an unsigned transaction inside a Batch.
const TF_INNER_BATCH_TXN: u64 = 0x4000_0000;

/// Pseudo-transactions: validators put them in a ledger; nobody signs them.
const PSEUDO_TRANSACTIONS: &[&str] = &["EnableAmendment", "SetFee", "UNLModify"];

/// The ID of a serialized transaction: SHA-512Half("TXN\0" ‖ blob). The blob
/// must be the signed transaction exactly as submitted.
pub fn transaction_id(blob: &[u8]) -> Hash256 {
    let mut data = Vec::with_capacity(PREFIX_TRANSACTION_ID.len() + blob.len());
    data.extend_from_slice(&PREFIX_TRANSACTION_ID);
    data.extend_from_slice(blob);
    Hash256(sha512_half(&data))
}

/// The ID (hash) of a signed transaction given as JSON. An unsigned one is an
/// error, except a pseudo-transaction or a Batch's inner transaction, which
/// never carry a signature.
///
/// ```
/// use xrpl_core::hashes::hash_signed_tx;
/// let tx = serde_json::json!({
///     "TransactionType": "EnableAmendment", "Account": "rrrrrrrrrrrrrrrrrrrrrhoLvTp",
///     "Amendment": "AE35ABDEFBDE520372B31C957020B34A7A4A9DC3115A69803A44016477C84D6E",
///     "Fee": "0", "LedgerSequence": 84206081, "Sequence": 0, "SigningPubKey": ""
/// });
/// assert_eq!(hash_signed_tx(&tx).unwrap().to_string(),
///     "CA4562711E4679FE9317DD767871E90A404C7A8B84FAFD35EC2CF0231F1F6DAF");
/// ```
pub fn hash_signed_tx(tx: &Value) -> Result<Hash256, CoreError> {
    require_signed(tx)?;
    Ok(transaction_id(&encode_transaction_json(tx, false)?))
}

/// The ID of a signed transaction blob (hex, as `submit` takes it).
///
/// `rippled` hashes the transaction as it re-serializes it (canonical field
/// order and forms), not the bytes it was sent, so this does the same: a
/// blob whose fields someone reordered on the way still has the ID the
/// ledger will show.
pub fn hash_signed_tx_blob(tx_blob: &str) -> Result<Hash256, CoreError> {
    let blob = hex::decode(tx_blob).map_err(|e| CoreError::InvalidHex(e.to_string()))?;
    hash_signed_tx(&decode_transaction_binary(&blob)?)
}

fn require_signed(tx: &Value) -> Result<(), CoreError> {
    let has = |field: &str| match tx.get(field) {
        Some(Value::String(s)) => !s.is_empty(),
        Some(Value::Array(a)) => !a.is_empty(),
        _ => false,
    };
    let pseudo = tx
        .get("TransactionType")
        .and_then(Value::as_str)
        .is_some_and(|t| PSEUDO_TRANSACTIONS.contains(&t));
    let inner_batch = tx
        .get("Flags")
        .and_then(crate::codec::encode::read_uint32)
        .is_some_and(|f| u64::from(f) & TF_INNER_BATCH_TXN != 0);
    if has("TxnSignature") || has("Signers") || pseudo || inner_batch {
        Ok(())
    } else {
        Err(CoreError::ValidationError(
            "the transaction is not signed: it has no TxnSignature or Signers".into(),
        ))
    }
}

/// Ledger entry IDs ("keylets"), as rippled computes them: SHA-512Half of a
/// two-byte namespace and the entry's identifying fields. Addresses are
/// classic addresses; IDs are hex.
///
/// ```
/// use xrpl_core::hashes::keylet;
/// assert_eq!(keylet::account("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap().to_string(),
///     "2B6AC232AA4C4BE41BF49D2459FA4A0347E1B543A4C92FCEE0821C0201E2E9A8");
/// ```
pub mod keylet {
    use super::*;

    /// One side of an AMM pool.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Issue<'a> {
        /// XRP.
        Xrp,
        /// A token: its currency code (3 letters or 40 hex digits) and issuer.
        Token {
            /// Its currency code (3 letters or 40 hex digits).
            currency: &'a str,
            /// Its issuer.
            issuer: &'a str,
        },
    }

    /// SHA-512Half(namespace ‖ parts), rippled's `indexHash`.
    fn index_hash(space: u8, parts: &[&[u8]]) -> Hash256 {
        let mut data = vec![0, space];
        for p in parts {
            data.extend_from_slice(p);
        }
        Hash256(sha512_half(&data))
    }

    fn id(address: &str) -> Result<[u8; 20], CoreError> {
        decode_account_id(address)
    }

    fn hash256(hex_id: &str) -> Result<[u8; 32], CoreError> {
        Ok(Hash256::from_hex(hex_id)?.0)
    }

    fn currency(code: &str) -> Result<[u8; 20], CoreError> {
        let c = if code.len() == 40 {
            Currency::from_hex(code)?
        } else {
            Currency::from_ascii(code)?
        };
        Ok(c.to_bytes())
    }

    fn account_seq(space: u8, account: &str, seq: u32) -> Result<Hash256, CoreError> {
        Ok(index_hash(space, &[&id(account)?, &seq.to_be_bytes()]))
    }

    /// `AccountRoot`.
    pub fn account(account: &str) -> Result<Hash256, CoreError> {
        Ok(index_hash(b'a', &[&id(account)?]))
    }

    /// An account's owner directory (its first page).
    pub fn owner_dir(account: &str) -> Result<Hash256, CoreError> {
        Ok(index_hash(b'O', &[&id(account)?]))
    }

    /// Page `page` of the directory whose first page is `root` (page 0 is `root`).
    pub fn dir_page(root: &Hash256, page: u64) -> Hash256 {
        if page == 0 {
            return *root;
        }
        index_hash(b'd', &[&root.0, &page.to_be_bytes()])
    }

    /// `RippleState`: the trust line between two accounts (in either order).
    pub fn trust_line(
        account_a: &str,
        account_b: &str,
        currency_code: &str,
    ) -> Result<Hash256, CoreError> {
        let (a, b) = (id(account_a)?, id(account_b)?);
        let (low, high) = if a <= b { (a, b) } else { (b, a) };
        Ok(index_hash(b'r', &[&low, &high, &currency(currency_code)?]))
    }

    /// `Offer`, by the sequence of the OfferCreate that placed it.
    pub fn offer(account: &str, sequence: u32) -> Result<Hash256, CoreError> {
        account_seq(b'o', account, sequence)
    }

    /// `Check`, by the sequence of its CheckCreate.
    pub fn check(account: &str, sequence: u32) -> Result<Hash256, CoreError> {
        account_seq(b'C', account, sequence)
    }

    /// `Escrow`, by the sequence of its EscrowCreate.
    pub fn escrow(account: &str, sequence: u32) -> Result<Hash256, CoreError> {
        account_seq(b'u', account, sequence)
    }

    /// `PayChannel`, by source, destination and the PaymentChannelCreate's sequence.
    pub fn payment_channel(
        source: &str,
        destination: &str,
        sequence: u32,
    ) -> Result<Hash256, CoreError> {
        Ok(index_hash(
            b'x',
            &[&id(source)?, &id(destination)?, &sequence.to_be_bytes()],
        ))
    }

    /// `Ticket`, by its TicketSequence.
    pub fn ticket(account: &str, ticket_sequence: u32) -> Result<Hash256, CoreError> {
        account_seq(b'T', account, ticket_sequence)
    }

    /// `SignerList` (rippled only ever uses list 0).
    pub fn signer_list(account: &str) -> Result<Hash256, CoreError> {
        account_seq(b'S', account, 0)
    }

    /// `DepositPreauth` of one account by an owner.
    pub fn deposit_preauth(owner: &str, authorized: &str) -> Result<Hash256, CoreError> {
        Ok(index_hash(b'p', &[&id(owner)?, &id(authorized)?]))
    }

    /// `NFTokenOffer`, by the sequence of its NFTokenCreateOffer.
    pub fn nftoken_offer(owner: &str, sequence: u32) -> Result<Hash256, CoreError> {
        account_seq(b'q', owner, sequence)
    }

    /// The directory of buy offers for an NFToken.
    pub fn nftoken_buy_offers(nftoken_id: &str) -> Result<Hash256, CoreError> {
        Ok(index_hash(b'h', &[&hash256(nftoken_id)?]))
    }

    /// The directory of sell offers for an NFToken.
    pub fn nftoken_sell_offers(nftoken_id: &str) -> Result<Hash256, CoreError> {
        Ok(index_hash(b'i', &[&hash256(nftoken_id)?]))
    }

    /// `AMM`: the pool for two assets (in either order).
    pub fn amm(asset1: Issue<'_>, asset2: Issue<'_>) -> Result<Hash256, CoreError> {
        // rippled orders Issues by currency, then issuer; XRP is all zeros.
        let key = |i: Issue<'_>| -> Result<([u8; 20], [u8; 20]), CoreError> {
            match i {
                Issue::Xrp => Ok(([0; 20], [0; 20])),
                Issue::Token {
                    currency: c,
                    issuer,
                } => Ok((currency(c)?, id(issuer)?)),
            }
        };
        let (a, b) = (key(asset1)?, key(asset2)?);
        let ((c1, i1), (c2, i2)) = if a <= b { (a, b) } else { (b, a) };
        Ok(index_hash(b'A', &[&i1, &c1, &i2, &c2]))
    }

    /// `DID`.
    pub fn did(account: &str) -> Result<Hash256, CoreError> {
        Ok(index_hash(b'I', &[&id(account)?]))
    }

    /// `Oracle`, by owner and OracleDocumentID.
    pub fn oracle(owner: &str, document_id: u32) -> Result<Hash256, CoreError> {
        account_seq(b'R', owner, document_id)
    }

    /// An MPT issuance's ID (MPTokenIssuanceID): the issuing transaction's
    /// sequence, then the issuer.
    pub fn mpt_issuance_id(sequence: u32, issuer: &str) -> Result<Hash192, CoreError> {
        let mut out = [0u8; 24];
        let (seq, acct) = out.split_at_mut(4);
        seq.copy_from_slice(&sequence.to_be_bytes());
        acct.copy_from_slice(&id(issuer)?);
        Ok(Hash192(out))
    }

    /// `MPTokenIssuance`, by MPTokenIssuanceID (48 hex digits).
    pub fn mpt_issuance(mpt_issuance_id: &str) -> Result<Hash256, CoreError> {
        Ok(index_hash(b'~', &[&Hash192::from_hex(mpt_issuance_id)?.0]))
    }

    /// `MPToken`: a holder's balance of an MPT issuance.
    pub fn mptoken(mpt_issuance_id: &str, holder: &str) -> Result<Hash256, CoreError> {
        Ok(index_hash(
            b't',
            &[&mpt_issuance(mpt_issuance_id)?.0, &id(holder)?],
        ))
    }

    /// `Credential`, by subject, issuer and CredentialType (hex, as in the transaction).
    pub fn credential(
        subject: &str,
        issuer: &str,
        credential_type: &str,
    ) -> Result<Hash256, CoreError> {
        let ty = hex::decode(credential_type).map_err(|e| CoreError::InvalidHex(e.to_string()))?;
        Ok(index_hash(b'D', &[&id(subject)?, &id(issuer)?, &ty]))
    }

    /// `Delegate`: the permissions `account` gave `authorized`.
    pub fn delegate(account: &str, authorized: &str) -> Result<Hash256, CoreError> {
        Ok(index_hash(b'E', &[&id(account)?, &id(authorized)?]))
    }

    /// `PermissionedDomain`, by owner and the PermissionedDomainSet's sequence.
    pub fn permissioned_domain(owner: &str, sequence: u32) -> Result<Hash256, CoreError> {
        account_seq(b'm', owner, sequence)
    }

    /// `Vault`, by owner and the VaultCreate's sequence.
    pub fn vault(owner: &str, sequence: u32) -> Result<Hash256, CoreError> {
        account_seq(b'V', owner, sequence)
    }

    /// `LoanBroker`, by owner and the LoanBrokerSet's sequence.
    pub fn loan_broker(owner: &str, sequence: u32) -> Result<Hash256, CoreError> {
        account_seq(b'l', owner, sequence)
    }

    /// `Loan`, by its LoanBroker's ID and the loan's sequence there.
    pub fn loan(loan_broker_id: &str, loan_sequence: u32) -> Result<Hash256, CoreError> {
        Ok(index_hash(
            b'L',
            &[&hash256(loan_broker_id)?, &loan_sequence.to_be_bytes()],
        ))
    }

    /// The `Amendments` singleton.
    pub fn amendments() -> Hash256 {
        index_hash(b'f', &[])
    }

    /// The `FeeSettings` singleton.
    pub fn fee_settings() -> Hash256 {
        index_hash(b'e', &[])
    }

    /// The `NegativeUNL` singleton.
    pub fn negative_unl() -> Hash256 {
        index_hash(b'N', &[])
    }

    /// The `LedgerHashes` entry holding the last 256 ledger hashes.
    pub fn ledger_hashes() -> Hash256 {
        index_hash(b's', &[])
    }

    /// The `LedgerHashes` entry holding every 256th hash for the 65,536
    /// ledgers around `ledger_sequence`.
    pub fn ledger_hashes_for(ledger_sequence: u32) -> Hash256 {
        index_hash(b's', &[&(ledger_sequence >> 16).to_be_bytes()])
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::keylet::{self, Issue};
    use super::*;

    fn s(h: Hash256) -> String {
        h.to_string()
    }

    // The keylet cases of xrpl.js hashes.test.ts.
    #[test]
    fn keylets_match_xrpl_js() {
        assert_eq!(
            s(keylet::trust_line(
                "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
                "rB5TihdPbKgMrkFqrqUC3yLdE8hhv4BdeY",
                "USD"
            )
            .unwrap()),
            "C683B5BB928F025F1E860D9D69D6C554C2202DE0D45877ADB3077DA4CB9E125C"
        );
        assert_eq!(
            s(keylet::trust_line(
                "rB5TihdPbKgMrkFqrqUC3yLdE8hhv4BdeY",
                "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
                "USD"
            )
            .unwrap()),
            "C683B5BB928F025F1E860D9D69D6C554C2202DE0D45877ADB3077DA4CB9E125C"
        );
        assert_eq!(
            s(keylet::trust_line(
                "r3kmLJN5D28dHuH8vZNUZpMC43pEHpaocV",
                "rUAMuQTfVhbfqUDuro7zzy4jj4Wq57MPTj",
                "UAM"
            )
            .unwrap()),
            "AE9ADDC584358E5847ADFC971834E471436FC3E9DE6EA1773DF49F419DC0F65E"
        );
        assert_eq!(
            s(keylet::offer("r32UufnaCGL82HubijgJGDmdE5hac7ZvLw", 137).unwrap()),
            "03F0AED09DEEE74CEF85CD57A0429D6113507CF759C597BABB4ADB752F734CE3"
        );
        assert_eq!(
            s(keylet::signer_list("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap()),
            "778365D5180F5DF3016817D1F318527AD7410D83F8636CF48C43E8AF72AB49BF"
        );
        assert_eq!(
            s(keylet::escrow("rDx69ebzbowuqztksVDmZXjizTd12BVr4x", 84).unwrap()),
            "61E8E8ED53FA2CEBE192B23897071E9A75217BF5A410E9CB5B45AAB7AECA567A"
        );
        assert_eq!(
            s(keylet::payment_channel(
                "rDx69ebzbowuqztksVDmZXjizTd12BVr4x",
                "rLFtVprxUEfsH54eCWKsZrEQzMDsx1wqso",
                82
            )
            .unwrap()),
            "E35708503B3C3143FB522D749AAFCC296E8060F0FB371A9A56FAE0B1ED127366"
        );
        assert_eq!(
            s(keylet::vault("rDcMtA1XpH5DGwiaqFif2cYCvgk5vxHraS", 18).unwrap()),
            "9C3208D7F99E5644643542518859401A96C93D80CC5F757AF0DF1781046C0A6A"
        );
        assert_eq!(
            s(keylet::loan_broker("rNTrjogemt4dZD13PaqphezBWSmiApNH4K", 84).unwrap()),
            "E799B84AC949CE2D8F27435C784F15C72E6A23ACA6841BA6D2F37A1E5DA4110F"
        );
        assert_eq!(
            s(keylet::loan(
                "AEB642A65066A6E6F03D312713475D958E0B320B74AD1A76B5B2EABB752E52AA",
                1
            )
            .unwrap()),
            "E93874AB62125DF2E86FB6C724B261F8E654E0334715C4D7160C0F148CDC9B47"
        );
    }

    /// Every entry in the mainnet sample whose ID its own fields determine must
    /// hash to the `index` rippled gave it.
    fn issue(a: &Value) -> Issue<'_> {
        match a["currency"].as_str().unwrap() {
            "XRP" => Issue::Xrp,
            c => Issue::Token {
                currency: c,
                issuer: a["issuer"].as_str().unwrap(),
            },
        }
    }

    #[test]
    fn keylets_match_mainnet_entries() {
        let path = format!(
            "{}/tests/fixtures/mainnet_ledger_objects.jsonl",
            env!("CARGO_MANIFEST_DIR")
        );
        let mut checked = std::collections::BTreeSet::new();
        for line in std::fs::read_to_string(path).unwrap().lines() {
            let o: Value = serde_json::from_str(line).unwrap();
            let str_of = |k: &str| o[k].as_str().unwrap();
            let u32_of = |k: &str| o[k].as_u64().unwrap() as u32;
            let ty = str_of("LedgerEntryType");
            let computed = match ty {
                "AccountRoot" => keylet::account(str_of("Account")).unwrap(),
                "Check" => keylet::check(str_of("Account"), u32_of("Sequence")).unwrap(),
                "Offer" => keylet::offer(str_of("Account"), u32_of("Sequence")).unwrap(),
                "Ticket" => keylet::ticket(str_of("Account"), u32_of("TicketSequence")).unwrap(),
                "Delegate" => keylet::delegate(str_of("Account"), str_of("Authorize")).unwrap(),
                "RippleState" => keylet::trust_line(
                    o["LowLimit"]["issuer"].as_str().unwrap(),
                    o["HighLimit"]["issuer"].as_str().unwrap(),
                    o["Balance"]["currency"].as_str().unwrap(),
                )
                .unwrap(),
                "AMM" => keylet::amm(issue(&o["Asset"]), issue(&o["Asset2"])).unwrap(),
                "Amendments" => keylet::amendments(),
                "FeeSettings" => keylet::fee_settings(),
                "LedgerHashes" => keylet::ledger_hashes(),
                "DirectoryNode" if o.get("Owner").is_some() && o["RootIndex"] == o["index"] => {
                    keylet::owner_dir(str_of("Owner")).unwrap()
                }
                "SignerList" if o.get("Owner").is_some() => {
                    keylet::signer_list(str_of("Owner")).unwrap()
                }
                _ => continue,
            };
            assert_eq!(s(computed), str_of("index"), "{ty}");
            checked.insert(ty.to_string());
        }
        assert!(checked.len() >= 10, "checked only {checked:?}");
    }

    // xrpl.js hashes.test.ts: an AccountRoot, a pseudo-transaction, a Batch inner transaction.
    #[test]
    fn transaction_hashes_match_xrpl_js() {
        assert_eq!(
            s(keylet::account("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap()),
            "2B6AC232AA4C4BE41BF49D2459FA4A0347E1B543A4C92FCEE0821C0201E2E9A8"
        );
        let inner = serde_json::json!({
            "Account": "rPMh7Pi9ct699iZUTWaytJUoHcJ7cgyziK", "Amount": "1000000",
            "Destination": "rJCxK2hX9tDMzbnn3cg1GU2g19Kfmhzxkp", "Fee": "0", "Flags": 0x4000_0000u32,
            "Sequence": 470, "SigningPubKey": "", "TransactionType": "Payment"
        });
        assert_eq!(
            s(hash_signed_tx(&inner).unwrap()),
            "9EDF5DB29F536DD3919037F1E8A72B040D075571A10C9000294C57B5ECEEA791"
        );
        let mut unsigned = inner.clone();
        unsigned["Flags"] = serde_json::json!(0);
        assert!(hash_signed_tx(&unsigned).is_err());
        let blob = hex::encode_upper(encode_transaction_json(&unsigned, false).unwrap());
        assert!(hash_signed_tx_blob(&blob).is_err());
    }

    // rippled 3.4.1 offline `sign` of the Payment in crypto::signing's tests:
    // its tx_json.hash for each key type.
    #[test]
    fn transaction_hashes_match_rippled() {
        for (blob, hash) in [
            (
                "120000220000000024000000016140000000000F424068400000000000000C7321030D58EB48B4420B1F7B9DF55087E0E29FEF0E8468F9A6825B01CA2C361042D43574473045022100BC95504DC72DDBDAA0F5EAA973EC07E057FD9A3E7500AC3CF07454C68219985E02200B0992C47857A867F5664BB6F74E62F086A59A0C31848E0FB3819127E8EC0A1281148049717CC948789F32F267ADC2582484E3DFA6988314B5F762798A53D543A014CAF8B297CFF8F2F937E8",
                "4A37F580F84306C73D3E38D9EB7C183A39154223CB173ACDD18BBB5C06BDD62C",
            ),
            (
                "120000220000000024000000016140000000000F424068400000000000000C7321ED01FA53FA5A7E77798F882ECE20B1ABC00BB358A9E55A202D0D0676BD0CE37A637440A7C427E6D84B1DB144466DF589D88C2F4CFC530743015EF5E5C31587FDA5334EB5A7E9C418DC17702A56860C28EEC2475098106330504543CEF0965D27EF570C8114D28B177E48D9A8D057E70F7E464B498367281B988314B5F762798A53D543A014CAF8B297CFF8F2F937E8",
                "05DD129B3CBA984E7DEB24FA2522C9DA9AAF4A518F1A01C78F2BD35B46298BF3",
            ),
        ] {
            assert_eq!(s(hash_signed_tx_blob(blob).unwrap()), hash);
            let json = decode_transaction_binary(&hex::decode(blob).unwrap()).unwrap();
            assert_eq!(s(hash_signed_tx(&json).unwrap()), hash);
        }
    }

    #[test]
    fn mpt_and_directory_ids() {
        let mptid = keylet::mpt_issuance_id(5, "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap();
        assert!(mptid.to_string().starts_with("00000005"));
        let root = keylet::owner_dir("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap();
        assert_eq!(keylet::dir_page(&root, 0), root);
        assert_ne!(keylet::dir_page(&root, 1), root);
        assert_eq!(
            keylet::amm(
                Issue::Xrp,
                Issue::Token {
                    currency: "USD",
                    issuer: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"
                }
            )
            .unwrap(),
            keylet::amm(
                Issue::Token {
                    currency: "USD",
                    issuer: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"
                },
                Issue::Xrp
            )
            .unwrap()
        );
    }

    /// rippled's ID is the hash of its own re-serialization: a signed blob
    /// with two fields swapped (which rippled still reads) has the ID of the
    /// canonical one, not the hash of the bytes as sent.
    #[test]
    fn a_reordered_blob_has_the_canonical_id() {
        let canonical = "120000220000000024000000016140000000000F424068400000000000000C7321030D58EB48B4420B1F7B9DF55087E0E29FEF0E8468F9A6825B01CA2C361042D43574473045022100BC95504DC72DDBDAA0F5EAA973EC07E057FD9A3E7500AC3CF07454C68219985E02200B0992C47857A867F5664BB6F74E62F086A59A0C31848E0FB3819127E8EC0A1281148049717CC948789F32F267ADC2582484E3DFA6988314B5F762798A53D543A014CAF8B297CFF8F2F937E8";
        let reordered = canonical.replacen(
            "1200002200000000240000000161",
            "1200002400000001220000000061",
            1,
        );
        assert_ne!(reordered, canonical);
        let id = hash_signed_tx_blob(canonical).unwrap();
        assert_eq!(id, transaction_id(&hex::decode(canonical).unwrap()));
        assert_eq!(hash_signed_tx_blob(&reordered).unwrap(), id);
        assert_ne!(transaction_id(&hex::decode(&reordered).unwrap()), id);
    }
}
