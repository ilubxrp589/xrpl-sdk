//! What `rippled` charges for a transaction: its base cost, as `rippled`
//! 3.4.1's `calculateBaseFee` computes it, before load and open-ledger
//! escalation scale it.
//!
//! | Transaction | Base cost |
//! |---|---|
//! | Most | one base fee, plus one per multi-signer (`Signers`) and per signer of a multi-signed `SponsorSignature` |
//! | `AccountDelete`, `AMMCreate`, `LedgerStateFix` | one owner reserve (signers add nothing) |
//! | `EscrowFinish` with a `Fulfillment` of *n* bytes | as most, plus 32 + *n* / 16 base fees |
//! | `ConfidentialMPTConvert`, `…ConvertBack`, `…Send`, `…Clawback`, `…MergeInbox` | as most, plus 9 base fees |
//! | `LoanSet` | as most, plus one base fee per signature in `CounterpartySignature` |
//! | `Batch` | two base fees, one per outer multi-signer, each inner transaction's own cost, one per `BatchSigners` signature |
//!
//! Two costs depend on the ledger and are not computed here:
//! `SetRegularKey` signed with the master key is free once (until the
//! account's `lsfPasswordSpent` flag is set; paying the usual fee is still
//! accepted), and `LoanPay` costs one more base fee for every five loan
//! payments beyond the first five that the payment covers.
//!
//! On mainnet in October 2026 a base fee is 10 drops and an owner reserve
//! 200,000 drops (0.2 XRP); read the current values from `server_state`
//! (`validated_ledger.base_fee` and `reserve_inc`) or the `FeeSettings`
//! ledger entry.

use crate::CoreError;
use serde_json::Value;

/// A transaction's base cost: so many base fees and so many owner reserves.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct TransactionCost {
    /// Base fees (`FeeSettings.BaseFeeDrops`; the reference transaction costs one).
    pub base_fees: u64,
    /// Owner reserves (`FeeSettings.ReserveIncrementDrops`).
    pub owner_reserves: u64,
}

impl TransactionCost {
    /// The cost in drops, at `base_fee` drops per base fee and
    /// `owner_reserve` drops per owner reserve. Saturates instead of
    /// overflowing.
    ///
    /// ```
    /// use xrpl_core::transaction::cost::TransactionCost;
    ///
    /// let delete = TransactionCost { base_fees: 0, owner_reserves: 1 };
    /// assert_eq!(delete.drops(10, 200_000), 200_000);
    /// ```
    #[must_use]
    pub fn drops(&self, base_fee: u64, owner_reserve: u64) -> u64 {
        self.base_fees
            .saturating_mul(base_fee)
            .saturating_add(self.owner_reserves.saturating_mul(owner_reserve))
    }
}

/// The transaction types whose base cost is one owner reserve.
const OWNER_RESERVE_TYPES: [&str; 3] = ["AccountDelete", "AMMCreate", "LedgerStateFix"];

/// The Confidential MPT types, which cost 9 base fees more
/// (`kConfidentialFeeMultiplier`).
const CONFIDENTIAL_TYPES: [&str; 5] = [
    "ConfidentialMPTConvert",
    "ConfidentialMPTConvertBack",
    "ConfidentialMPTSend",
    "ConfidentialMPTClawback",
    "ConfidentialMPTMergeInbox",
];

/// The base cost of `tx` (JSON) as `rippled` 3.4.1 computes it, for the
/// transaction as it will be submitted. The multi-signers are counted from
/// its `Signers`; when they are not on it yet, pass how many there will be
/// as `signers` (0 for a single-signed transaction). The larger count is
/// used.
///
/// ```
/// use serde_json::json;
/// use xrpl_core::transaction::cost::{transaction_cost, TransactionCost};
///
/// let payment = json!({"TransactionType": "Payment", "Account": "rA"});
/// assert_eq!(transaction_cost(&payment, 0)?, TransactionCost { base_fees: 1, owner_reserves: 0 });
/// assert_eq!(transaction_cost(&payment, 3)?.base_fees, 4);
///
/// let delete = json!({"TransactionType": "AccountDelete", "Account": "rA"});
/// assert_eq!(transaction_cost(&delete, 3)?, TransactionCost { base_fees: 0, owner_reserves: 1 });
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
///
/// # Errors
///
/// [`CoreError::ValidationError`] if `tx` has no `TransactionType`, or a
/// `Batch` has no `RawTransactions`; [`CoreError::InvalidHex`] for a
/// `Fulfillment` that is not hex.
pub fn transaction_cost(tx: &Value, signers: usize) -> Result<TransactionCost, CoreError> {
    let invalid = |m: &str| CoreError::ValidationError(m.to_string());
    let kind = tx
        .get("TransactionType")
        .and_then(Value::as_str)
        .ok_or_else(|| invalid("the transaction has no TransactionType"))?;
    if OWNER_RESERVE_TYPES.contains(&kind) {
        return Ok(TransactionCost {
            base_fees: 0,
            owner_reserves: 1,
        });
    }
    let own_signers = count(
        tx.get("Signers")
            .and_then(Value::as_array)
            .map_or(0, Vec::len),
    );
    let sponsor_signers = count(
        tx.get("SponsorSignature")
            .and_then(|s| s.get("Signers"))
            .and_then(Value::as_array)
            .map_or(0, Vec::len),
    );
    // Transactor::calculateBaseFee: one, plus one per signer and sponsor signer.
    let mut cost = TransactionCost {
        base_fees: 1u64
            .saturating_add(own_signers.max(count(signers)))
            .saturating_add(sponsor_signers),
        owner_reserves: 0,
    };
    let add =
        |cost: &mut TransactionCost, n: u64| cost.base_fees = cost.base_fees.saturating_add(n);
    match kind {
        "EscrowFinish" => {
            if let Some(fulfillment) = tx.get("Fulfillment").and_then(Value::as_str) {
                if fulfillment.len() % 2 != 0 || !fulfillment.bytes().all(|b| b.is_ascii_hexdigit())
                {
                    return Err(CoreError::InvalidHex(
                        "Fulfillment is not an even number of hex digits".to_string(),
                    ));
                }
                let bytes = count(fulfillment.len() / 2);
                add(&mut cost, 32u64.saturating_add(bytes / 16));
            }
        }
        k if CONFIDENTIAL_TYPES.contains(&k) => add(&mut cost, 9),
        "LoanSet" => {
            if let Some(signature) = tx.get("CounterpartySignature") {
                let n = match signature.get("Signers").and_then(Value::as_array) {
                    Some(list) => count(list.len()),
                    None => u64::from(signature.get("TxnSignature").is_some()),
                };
                add(&mut cost, n);
            }
        }
        "Batch" => {
            // Batch::calculateBaseFee: one more base fee for the batch, each
            // inner transaction's own cost, one per BatchSigners signature.
            add(&mut cost, 1);
            let raw = tx
                .get("RawTransactions")
                .and_then(Value::as_array)
                .ok_or_else(|| invalid("a Batch has RawTransactions"))?;
            for wrapped in raw {
                let inner = wrapped.get("RawTransaction").unwrap_or(wrapped);
                let inner_cost = transaction_cost(inner, 0)?;
                add(&mut cost, inner_cost.base_fees);
                cost.owner_reserves = cost
                    .owner_reserves
                    .saturating_add(inner_cost.owner_reserves);
            }
            for signer in tx
                .get("BatchSigners")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                let signer = signer.get("BatchSigner").unwrap_or(signer);
                let n = if signer.get("TxnSignature").is_some() {
                    1
                } else {
                    count(
                        signer
                            .get("Signers")
                            .and_then(Value::as_array)
                            .map_or(0, Vec::len),
                    )
                };
                add(&mut cost, n);
            }
        }
        _ => {}
    }
    Ok(cost)
}

fn count(n: usize) -> u64 {
    u64::try_from(n).unwrap_or(u64::MAX)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;
    use serde_json::json;

    fn cost(tx: Value) -> (u64, u64) {
        let c = transaction_cost(&tx, 0).unwrap();
        (c.base_fees, c.owner_reserves)
    }

    #[test]
    fn the_rules_of_rippled_3_4_1() {
        assert_eq!(cost(json!({"TransactionType": "Payment"})), (1, 0));
        assert_eq!(
            cost(json!({"TransactionType": "Payment", "Signers": [{}, {}, {}]})),
            (4, 0)
        );
        assert_eq!(
            cost(json!({"TransactionType": "Payment", "SponsorSignature": {"Signers": [{}, {}]}})),
            (3, 0)
        );
        for kind in ["AccountDelete", "AMMCreate", "LedgerStateFix"] {
            assert_eq!(
                cost(json!({"TransactionType": kind, "Signers": [{}, {}]})),
                (0, 1),
                "{kind}"
            );
        }
        // 100 bytes of fulfillment: 1 + 32 + 100/16 = 39.
        let fulfillment = "A0".repeat(100);
        assert_eq!(
            cost(json!({"TransactionType": "EscrowFinish", "Fulfillment": fulfillment})),
            (39, 0)
        );
        assert_eq!(cost(json!({"TransactionType": "EscrowFinish"})), (1, 0));
        assert!(transaction_cost(
            &json!({"TransactionType": "EscrowFinish", "Fulfillment": "A0Z"}),
            0
        )
        .is_err());
        assert_eq!(
            cost(json!({"TransactionType": "ConfidentialMPTSend"})),
            (10, 0)
        );
        assert_eq!(
            cost(
                json!({"TransactionType": "LoanSet", "CounterpartySignature": {"Signers": [{}, {}]}})
            ),
            (3, 0)
        );
        assert_eq!(
            cost(
                json!({"TransactionType": "LoanSet", "CounterpartySignature": {"TxnSignature": "AB"}})
            ),
            (2, 0)
        );
        assert_eq!(cost(json!({"TransactionType": "LoanSet"})), (1, 0));
        assert!(transaction_cost(&json!({"Account": "rA"}), 0).is_err());
    }

    #[test]
    fn expected_signers_count_when_not_on_the_transaction_yet() {
        let tx = json!({"TransactionType": "Payment"});
        assert_eq!(transaction_cost(&tx, 2).unwrap().base_fees, 3);
        let signed = json!({"TransactionType": "Payment", "Signers": [{}, {}, {}]});
        assert_eq!(transaction_cost(&signed, 2).unwrap().base_fees, 4);
    }

    #[test]
    fn a_batch_counts_its_inner_transactions_and_signers() {
        let batch = json!({
            "TransactionType": "Batch",
            "RawTransactions": [
                {"RawTransaction": {"TransactionType": "Payment"}},
                {"RawTransaction": {"TransactionType": "AccountDelete"}},
                {"RawTransaction": {"TransactionType": "EscrowFinish", "Fulfillment": "A0".repeat(32)}},
            ],
            "BatchSigners": [
                {"BatchSigner": {"TxnSignature": "AB"}},
                {"BatchSigner": {"Signers": [{}, {}, {}]}},
            ],
        });
        // 2 + Payment 1 + EscrowFinish (1 + 32 + 2) + signatures 1 + 3; one reserve.
        assert_eq!(cost(batch), (42, 1));
        assert!(transaction_cost(&json!({"TransactionType": "Batch"}), 0).is_err());
    }

    #[test]
    fn drops() {
        let c = TransactionCost {
            base_fees: 3,
            owner_reserves: 1,
        };
        assert_eq!(c.drops(10, 200_000), 200_030);
        assert_eq!(c.drops(u64::MAX, 1), u64::MAX);
    }
}
