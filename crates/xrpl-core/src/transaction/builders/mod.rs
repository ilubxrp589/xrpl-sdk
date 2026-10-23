mod account_set;
mod amm;
mod check;
mod clawback;
mod delegate_set;
mod escrow;
mod nft;
mod offer;
mod payment;
mod payment_channel;
mod set_regular_key;
mod signer_list;
mod trust_set;

use crate::CoreError;
use serde_json::Value;

/// Trait implemented by the hand-written transaction builders below.
///
/// These builders cover common transactions with `serde_json::Value`
/// amounts (XRP as a string of drops; a token as an object) and check the
/// rules `rippled` checks before applying (`build` calls `validate`). For
/// every transaction type, typed field by field and generated from
/// `rippled`'s definitions, use [`crate::transaction::models`] (its
/// `Payment` is a different type from this module's `Payment`).
pub trait Transaction {
    /// Returns the XRPL transaction type string (e.g. "Payment").
    fn transaction_type(&self) -> &'static str;

    /// Serialize the transaction to a JSON Value suitable for signing/submission.
    fn to_json(&self) -> Value;

    /// Validate that all required fields are present and valid.
    fn validate(&self) -> Result<(), CoreError>;
}

/// Common fields shared by all transactions.
/// Fields that can be autofilled (fee, sequence, last_ledger_sequence) are Optional.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct TxCommon {
    /// The account that sends the transaction (`Account`).
    pub account: String,
    /// The transaction cost, in drops (`Fee`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee: Option<String>,
    /// The sequence number (`Sequence`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sequence: Option<u32>,
    /// The last ledger the transaction may appear in (`LastLedgerSequence`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_ledger_sequence: Option<u32>,
    /// Flag bits (`Flags`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<u32>,
    /// Notes attached to the transaction (`Memos`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memos: Option<Vec<Value>>,
    /// A tag the sender uses to tell its own payments apart (`SourceTag`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_tag: Option<u32>,
    /// Valid only if the account's last transaction had this hash (`AccountTxnID`).
    #[serde(rename = "AccountTxnID", skip_serializing_if = "Option::is_none")]
    pub account_txn_id: Option<String>,
    /// The network's ID, required on networks above 1024 and refused on
    /// mainnet, Testnet and Devnet (`NetworkID`; autofill sets it).
    #[serde(rename = "NetworkID", skip_serializing_if = "Option::is_none")]
    pub network_id: Option<u32>,
    /// The account acting for `account` through a delegation (`Delegate`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delegate: Option<String>,
    /// A ticket to use instead of a sequence number (`TicketSequence`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticket_sequence: Option<u32>,
    /// The signing public key, hex; empty when multi-signed (`SigningPubKey`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signing_pub_key: Option<String>,
    /// The signature, hex (`TxnSignature`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub txn_signature: Option<String>,
}

impl TxCommon {
    /// A builder for the transaction from `account`.
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            account: account.into(),
            fee: None,
            sequence: None,
            last_ledger_sequence: None,
            flags: None,
            memos: None,
            source_tag: None,
            account_txn_id: None,
            network_id: None,
            delegate: None,
            ticket_sequence: None,
            signing_pub_key: None,
            txn_signature: None,
        }
    }
}

/// A builder's validation error: `"{tx}: {why}"`.
fn invalid(tx: &str, why: &str) -> CoreError {
    CoreError::ValidationError(format!("{tx}: {why}"))
}

/// `value`, when set, must be hex of at most `max_bytes` bytes.
fn check_hex(value: Option<&str>, field: &str, max_bytes: usize) -> Result<(), CoreError> {
    match value.map(hex::decode) {
        Some(Ok(bytes)) if bytes.len() > max_bytes => Err(CoreError::ValidationError(format!(
            "{field} is at most {max_bytes} bytes ({} hex digits)",
            2 * max_bytes
        ))),
        Some(Err(_)) => Err(CoreError::ValidationError(format!("{field} must be hex"))),
        _ => Ok(()),
    }
}

/// Helper: merge TxCommon fields into a JSON object.
fn merge_common(base: &mut serde_json::Map<String, Value>, common: &TxCommon) {
    let common_val = serde_json::to_value(common).unwrap_or_default();
    if let Value::Object(map) = common_val {
        for (k, v) in map {
            base.entry(k).or_insert(v);
        }
    }
}

pub use account_set::{AccountSet, AccountSetBuilder};
pub use amm::{
    AMMBid, AMMBidBuilder, AMMCreate, AMMCreateBuilder, AMMDeposit, AMMDepositBuilder, AMMVote,
    AMMVoteBuilder, AMMWithdraw, AMMWithdrawBuilder,
};
pub use check::{
    CheckCancel, CheckCancelBuilder, CheckCash, CheckCashBuilder, CheckCreate, CheckCreateBuilder,
};
pub use clawback::{Clawback, ClawbackBuilder};
pub use delegate_set::{DelegateSet, DelegateSetBuilder};
pub use escrow::{
    EscrowCancel, EscrowCancelBuilder, EscrowCreate, EscrowCreateBuilder, EscrowFinish,
    EscrowFinishBuilder,
};
pub use nft::{
    NFTokenAcceptOffer, NFTokenAcceptOfferBuilder, NFTokenBurn, NFTokenBurnBuilder,
    NFTokenCancelOffer, NFTokenCancelOfferBuilder, NFTokenCreateOffer, NFTokenCreateOfferBuilder,
    NFTokenMint, NFTokenMintBuilder,
};
pub use offer::{OfferCancel, OfferCancelBuilder, OfferCreate, OfferCreateBuilder};
pub use payment::{Payment, PaymentBuilder};
pub use payment_channel::{
    PaymentChannelClaim, PaymentChannelClaimBuilder, PaymentChannelCreate,
    PaymentChannelCreateBuilder, PaymentChannelFund, PaymentChannelFundBuilder,
};
pub use set_regular_key::{SetRegularKey, SetRegularKeyBuilder};
pub use signer_list::{SignerListSet, SignerListSetBuilder};
pub use trust_set::{TrustSet, TrustSetBuilder};

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;
    use crate::transaction::models::{AMMDeposit as D, AMMWithdraw as W};
    use serde_json::json;

    const A: &str = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";
    const B: &str = "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe";
    const C: &str = "rU6K7V3Po4snVhBBaU29sesqs2qTQJWDw1";

    fn usd(value: &str) -> Value {
        json!({"currency": "USD", "issuer": C, "value": value})
    }

    /// With no mode flag, the AMM builders set the one their fields
    /// describe; a combination rippled refuses is refused here.
    #[test]
    fn amm_modes_are_inferred_and_checked() {
        let deposit = || {
            AMMDeposit::builder(A)
                .asset(json!({"currency": "XRP"}))
                .asset2(json!({"currency": "USD", "issuer": C}))
        };
        let flags = |tx: &AMMDeposit| tx.common.flags.unwrap();
        assert_eq!(
            flags(&deposit().amount(json!("1000")).build().unwrap()),
            D::TF_SINGLE_ASSET
        );
        assert_eq!(
            flags(
                &deposit()
                    .amount(json!("1000"))
                    .amount2(usd("1"))
                    .build()
                    .unwrap()
            ),
            D::TF_TWO_ASSET
        );
        assert_eq!(
            flags(&deposit().lp_token_out(usd("1")).build().unwrap()),
            D::TF_LP_TOKEN
        );
        assert_eq!(
            flags(
                &deposit()
                    .amount(json!("1"))
                    .e_price(json!("2"))
                    .build()
                    .unwrap()
            ),
            D::TF_LIMIT_LP_TOKEN
        );
        assert!(deposit().build().is_err(), "nothing to deposit");
        assert!(
            deposit()
                .amount(json!("1"))
                .flags(D::TF_TWO_ASSET)
                .build()
                .is_err(),
            "flag and fields disagree"
        );
        assert!(deposit()
            .amount(json!("1"))
            .flags(D::TF_SINGLE_ASSET | D::TF_LP_TOKEN)
            .build()
            .is_err());
        assert!(
            deposit()
                .amount(json!("1"))
                .trading_fee(10)
                .build()
                .is_err(),
            "TradingFee needs tfTwoAssetIfEmpty"
        );

        let withdraw = || {
            AMMWithdraw::builder(A)
                .asset(json!({"currency": "XRP"}))
                .asset2(json!({"currency": "USD", "issuer": C}))
        };
        assert_eq!(
            withdraw().build().unwrap().common.flags,
            Some(W::TF_WITHDRAW_ALL)
        );
        assert_eq!(
            withdraw().amount(json!("5")).build().unwrap().common.flags,
            Some(W::TF_SINGLE_ASSET)
        );
        assert!(withdraw().amount2(usd("1")).build().is_err());
    }

    #[test]
    fn account_set_values_are_checked() {
        assert!(AccountSet::builder(A).tick_size(2).build().is_err());
        assert!(AccountSet::builder(A).tick_size(5).build().is_ok());
        assert!(AccountSet::builder(A).transfer_rate(999).build().is_err());
        assert!(AccountSet::builder(A)
            .transfer_rate(1_005_000_000)
            .build()
            .is_ok());
        assert!(AccountSet::builder(A)
            .set_flag(8)
            .clear_flag(8)
            .build()
            .is_err());
    }

    #[test]
    fn signer_lists_are_checked() {
        let entry = |account: &str, weight: u64| json!({"SignerEntry": {"Account": account, "SignerWeight": weight}});
        let list = |quorum, entries| {
            SignerListSet::builder(A)
                .signer_quorum(quorum)
                .signer_entries(entries)
                .build()
        };
        assert!(list(2, vec![entry(B, 1), entry(C, 1)]).is_ok());
        assert!(
            list(3, vec![entry(B, 1), entry(C, 1)]).is_err(),
            "weights below the quorum"
        );
        assert!(
            list(1, vec![entry(B, 1), entry(B, 1)]).is_err(),
            "listed twice"
        );
        assert!(list(1, vec![entry(A, 1)]).is_err(), "itself");
        assert!(list(1, vec![entry(B, 0)]).is_err(), "zero weight");
        assert!(
            list(0, vec![entry(B, 1)]).is_err(),
            "quorum 0 deletes the list"
        );
        assert!(SignerListSet::builder(A).signer_quorum(0).build().is_ok());
    }

    #[test]
    fn regular_keys_delegates_and_clawbacks_are_checked() {
        assert!(SetRegularKey::builder(A).regular_key(A).build().is_err());
        assert!(SetRegularKey::builder(A).regular_key(B).build().is_ok());
        assert!(DelegateSet::builder(A)
            .authorize(A)
            .permissions(vec![json!({"Permission": {"PermissionValue": "Payment"}})])
            .build()
            .is_err());
        assert!(
            Clawback::builder(A).amount(json!("100")).build().is_err(),
            "XRP"
        );
        assert!(
            Clawback::builder(C).amount(usd("1")).build().is_err(),
            "the holder is the issuer"
        );
        assert!(Clawback::builder(A).amount(usd("1")).build().is_ok());
        let mpt = json!({"mpt_issuance_id": "00000001B5F762798A53D543A014CAF8B297CFF8F2F937E8", "value": "5"});
        assert!(
            Clawback::builder(A).amount(mpt.clone()).build().is_err(),
            "no Holder"
        );
        assert!(Clawback::builder(A).amount(mpt).holder(B).build().is_ok());
    }

    /// Every name the common fields write is a field rippled knows (the
    /// codec would refuse it otherwise; AccountTxnID was AccountTxnId).
    #[test]
    fn common_fields_use_rippleds_names() {
        let mut common = TxCommon::new(A);
        common.account_txn_id = Some("AB".repeat(32));
        common.network_id = Some(21_338);
        common.delegate = Some(B.to_string());
        let mut tx = serde_json::Map::new();
        tx.insert("TransactionType".into(), json!("AccountSet"));
        merge_common(&mut tx, &common);
        assert!(tx.contains_key("AccountTxnID") && tx.contains_key("NetworkID"));
        assert!(crate::codec::encode_transaction_json(&Value::Object(tx), false).is_ok());
    }
}
