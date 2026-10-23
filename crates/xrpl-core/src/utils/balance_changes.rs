//! What a transaction did to each account's balances, from its metadata.

use super::decimal::Decimal;
use crate::transaction::metadata::{AffectedNode, DeliveredAmount, TransactionMetadata};
use crate::types::Amount;
use crate::CoreError;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// One balance change: XRP (`currency` "XRP", no issuer) or a token, from the
/// point of view of the account it is listed under.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Balance {
    /// `"XRP"` or the token's currency code.
    pub currency: String,
    /// The issuer (`Issuer`).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub issuer: Option<String>,
    /// The change, in plain decimal (`"-100.012"`); XRP in XRP, not drops.
    pub value: String,
}

/// Every balance change of one account.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountBalanceChanges {
    /// The account that sends the transaction or owns the entry (`Account`).
    pub account: String,
    /// Each balance change.
    pub balances: Vec<Balance>,
}

/// What a transaction actually delivered, from its metadata: `rippled`'s
/// `delivered_amount` (or the stored `DeliveredAmount`). Use it, never the
/// Payment's `Amount`, to see what arrived: a partial payment
/// (`tfPartialPayment`) can deliver far less than its `Amount`. `None` when
/// the metadata has neither, or says `"unavailable"` (ledgers from before
/// 2014).
///
/// ```
/// use xrpl_core::transaction::TransactionMetadata;
/// use xrpl_core::utils::delivered_amount;
///
/// let meta: TransactionMetadata = serde_json::from_value(serde_json::json!({
///     "TransactionResult": "tesSUCCESS", "TransactionIndex": 0, "AffectedNodes": [],
///     "delivered_amount": "15",
/// }))?;
/// assert_eq!(delivered_amount(&meta), Some(xrpl_core::types::Amount::Xrp(15)));
/// # Ok::<(), serde_json::Error>(())
/// ```
pub fn delivered_amount(meta: &TransactionMetadata) -> Option<Amount> {
    match &meta.delivered_amount {
        Some(DeliveredAmount::Amount(amount)) => Some(amount.clone()),
        Some(DeliveredAmount::Unavailable) => None,
        _ => meta.stored_delivered_amount.clone(),
    }
}

/// The balance changes a transaction made, per account, from its metadata:
/// XRP from each `AccountRoot`, tokens from each `RippleState` (listed for both
/// accounts on the trust line, each from its own side). The same result as
/// xrpl.js's `getBalanceChanges`, in the same order, but exact: values are
/// computed in decimal, never in floating point.
///
/// Not included: MPT balances (`MPToken` and `MPTokenIssuance` entries), and
/// XRP held in escrows, payment channels and checks (they are not account
/// balances). For what a Payment delivered, see [`delivered_amount`], never
/// the transaction's `Amount` (a partial payment delivers less).
///
/// # Errors
///
/// [`CoreError::ValidationError`] if a changed `RippleState` has no
/// `LowLimit`/`HighLimit` issuer or `Balance` currency;
/// [`CoreError::InvalidAmount`] for a `Balance` that is not a number.
pub fn get_balance_changes(
    meta: &TransactionMetadata,
) -> Result<Vec<AccountBalanceChanges>, CoreError> {
    let mut changes: Vec<(String, Balance)> = Vec::new();
    for node in &meta.affected_nodes {
        match node.ledger_entry_type() {
            "AccountRoot" => {
                if let Some(change) = xrp_change(node)? {
                    changes.push(change);
                }
            }
            "RippleState" => changes.extend(trust_line_changes(node)?),
            _ => {}
        }
    }
    let mut grouped: Vec<AccountBalanceChanges> = Vec::new();
    for (account, balance) in changes {
        match grouped.iter_mut().find(|g| g.account == account) {
            Some(g) => g.balances.push(balance),
            None => grouped.push(AccountBalanceChanges {
                account,
                balances: vec![balance],
            }),
        }
    }
    Ok(grouped)
}

/// `Balance` after minus before (or the new balance of a created entry); `None`
/// if it did not change.
fn balance_change(node: &AffectedNode) -> Result<Option<Decimal>, CoreError> {
    let value = |v: &Value| -> Result<Decimal, CoreError> {
        match v {
            Value::String(drops) => Decimal::parse(drops),
            Value::Object(_) => {
                Decimal::parse(v.get("value").and_then(Value::as_str).unwrap_or(""))
            }
            _ => Err(CoreError::InvalidAmount(format!("not a Balance: {v}"))),
        }
    };
    let change = if let Some(new) = balance(node.new_fields()) {
        value(new)?
    } else if let (Some(prev), Some(fin)) = (
        balance(node.previous_fields()),
        balance(node.final_fields()),
    ) {
        value(fin)?.sub(&value(prev)?)
    } else {
        return Ok(None);
    };
    Ok((!change.is_zero()).then_some(change))
}

/// The `Balance` among `fields`.
fn balance(fields: Option<&Map<String, Value>>) -> Option<&Value> {
    fields.and_then(|f| f.get("Balance"))
}

/// The entry's fields: those it was created with, or those after the
/// transaction.
fn fields(node: &AffectedNode) -> Option<&Map<String, Value>> {
    node.new_fields().or_else(|| node.final_fields())
}

fn xrp_change(node: &AffectedNode) -> Result<Option<(String, Balance)>, CoreError> {
    let Some(drops) = balance_change(node)? else {
        return Ok(None);
    };
    let account = node
        .final_fields()
        .and_then(|f| f.get("Account"))
        .or_else(|| node.new_fields().and_then(|f| f.get("Account")))
        .and_then(Value::as_str);
    Ok(account.map(|a| {
        (
            a.to_string(),
            Balance {
                currency: "XRP".into(),
                issuer: None,
                value: drops.scaled(-6).to_string(),
            },
        )
    }))
}

/// A trust line's `Balance` is the low account's: positive when the high
/// account owes it. List the change for the low account, then for the high.
fn trust_line_changes(node: &AffectedNode) -> Result<Vec<(String, Balance)>, CoreError> {
    let Some(change) = balance_change(node)? else {
        return Ok(Vec::new());
    };
    let fields = fields(node);
    let field = |entry: &str, key: &str| {
        fields
            .and_then(|f| f.get(entry))
            .and_then(|v| v.get(key))
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| CoreError::ValidationError(format!("RippleState without {entry}.{key}")))
    };
    let (low, high, currency) = (
        field("LowLimit", "issuer")?,
        field("HighLimit", "issuer")?,
        field("Balance", "currency")?,
    );
    Ok(vec![
        (
            low.clone(),
            Balance {
                currency: currency.clone(),
                issuer: Some(high.clone()),
                value: change.to_string(),
            },
        ),
        (
            high,
            Balance {
                currency,
                issuer: Some(low),
                value: change.negated().to_string(),
            },
        ),
    ])
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    fn read(path: &str) -> Value {
        let path = format!("{}/tests/fixtures/{path}", env!("CARGO_MANIFEST_DIR"));
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    /// Order-insensitive within an account's balances: xrpl.js lists fields
    /// in whatever order its object literals had, which carries no meaning.
    fn sorted(mut v: Vec<AccountBalanceChanges>) -> Vec<AccountBalanceChanges> {
        for a in &mut v {
            a.balances.sort_by(|x, y| {
                (&x.currency, &x.issuer, &x.value).cmp(&(&y.currency, &y.issuer, &y.value))
            });
        }
        v
    }

    // xrpl.js getBalanceChanges.test.ts: its 12 fixtures and expected results.
    #[test]
    fn matches_xrpl_js() {
        let cases = read("xrpljs/balance_changes_expected.json");
        let cases = cases.as_array().unwrap();
        assert_eq!(cases.len(), 12);
        for case in cases {
            let fixture = read(&format!(
                "xrpljs/{}.json",
                case["fixture"].as_str().unwrap()
            ));
            let meta = match case["path"].as_str().unwrap() {
                "metadata" => &fixture["metadata"],
                _ => &fixture["result"]["meta"],
            };
            let expected: Vec<AccountBalanceChanges> =
                serde_json::from_value(case["expected"].clone()).unwrap();
            let meta: TransactionMetadata = serde_json::from_value(meta.clone())
                .unwrap_or_else(|e| panic!("{}: {e}", case["name"]));
            let got = get_balance_changes(&meta).unwrap();
            let accounts = |v: &[AccountBalanceChanges]| {
                v.iter().map(|a| a.account.clone()).collect::<Vec<_>>()
            };
            assert_eq!(accounts(&got), accounts(&expected), "{}", case["name"]);
            assert_eq!(sorted(got), sorted(expected), "{}", case["name"]);
        }
    }

    #[test]
    fn rejects_what_is_not_metadata() {
        assert!(
            serde_json::from_value::<TransactionMetadata>(serde_json::json!({"hash": "00"}))
                .is_err()
        );
        assert!(get_balance_changes(&meta(serde_json::json!({})))
            .unwrap()
            .is_empty());
    }

    /// Metadata with no affected entries and the fields of `extra`.
    fn meta(extra: Value) -> TransactionMetadata {
        let mut m = serde_json::json!({"TransactionResult": "tesSUCCESS", "TransactionIndex": 0, "AffectedNodes": []});
        for (k, v) in extra.as_object().unwrap() {
            m[k] = v.clone();
        }
        serde_json::from_value(m).unwrap()
    }

    /// A partial payment's delivered amount, not its Amount; "unavailable"
    /// and absent are None.
    #[test]
    fn delivered_amount_is_what_arrived() {
        let token = serde_json::json!({"currency": "USD", "issuer": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", "value": "0.5"});
        assert_eq!(
            delivered_amount(&meta(serde_json::json!({"delivered_amount": token}))),
            Some(serde_json::from_value(token).unwrap())
        );
        assert_eq!(
            delivered_amount(&meta(serde_json::json!({"DeliveredAmount": "7"}))),
            Some(crate::types::Amount::Xrp(7))
        );
        assert_eq!(
            delivered_amount(&meta(
                serde_json::json!({"delivered_amount": "unavailable", "DeliveredAmount": "7"})
            )),
            None
        );
        assert_eq!(delivered_amount(&meta(serde_json::json!({}))), None);
    }
}
