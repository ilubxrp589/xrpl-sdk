//! What a transaction did to each account's balances, from its metadata.

use super::decimal::Decimal;
use crate::CoreError;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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

/// The balance changes a transaction made, per account, from its metadata:
/// XRP from each `AccountRoot`, tokens from each `RippleState` (listed for both
/// accounts on the trust line, each from its own side). The same result as
/// xrpl.js's `getBalanceChanges`, in the same order, but exact: values are
/// computed in decimal, never in floating point.
pub fn get_balance_changes(meta: &Value) -> Result<Vec<AccountBalanceChanges>, CoreError> {
    let nodes = meta
        .get("AffectedNodes")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            CoreError::ValidationError(
                "expected transaction metadata (an object with AffectedNodes)".into(),
            )
        })?;
    let mut changes: Vec<(String, Balance)> = Vec::new();
    for node in nodes {
        let Some(node) = ["CreatedNode", "ModifiedNode", "DeletedNode"]
            .iter()
            .find_map(|kind| node.get(*kind))
        else {
            continue;
        };
        match node.get("LedgerEntryType").and_then(Value::as_str) {
            Some("AccountRoot") => {
                if let Some(change) = xrp_change(node)? {
                    changes.push(change);
                }
            }
            Some("RippleState") => changes.extend(trust_line_changes(node)?),
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
fn balance_change(node: &Value) -> Result<Option<Decimal>, CoreError> {
    let value = |v: &Value| -> Result<Decimal, CoreError> {
        match v {
            Value::String(drops) => Decimal::parse(drops),
            Value::Object(_) => {
                Decimal::parse(v.get("value").and_then(Value::as_str).unwrap_or(""))
            }
            _ => Err(CoreError::InvalidAmount(format!("not a Balance: {v}"))),
        }
    };
    let change = if let Some(new) = node.pointer("/NewFields/Balance") {
        value(new)?
    } else if let (Some(prev), Some(fin)) = (
        node.pointer("/PreviousFields/Balance"),
        node.pointer("/FinalFields/Balance"),
    ) {
        value(fin)?.sub(&value(prev)?)
    } else {
        return Ok(None);
    };
    Ok((!change.is_zero()).then_some(change))
}

fn xrp_change(node: &Value) -> Result<Option<(String, Balance)>, CoreError> {
    let Some(drops) = balance_change(node)? else {
        return Ok(None);
    };
    let account = node
        .pointer("/FinalFields/Account")
        .or_else(|| node.pointer("/NewFields/Account"))
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
fn trust_line_changes(node: &Value) -> Result<Vec<(String, Balance)>, CoreError> {
    let Some(change) = balance_change(node)? else {
        return Ok(Vec::new());
    };
    let fields = node.get("NewFields").or_else(|| node.get("FinalFields"));
    let field = |path: &str| {
        fields
            .and_then(|f| f.pointer(path))
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| CoreError::ValidationError(format!("RippleState without {path}")))
    };
    let (low, high, currency) = (
        field("/LowLimit/issuer")?,
        field("/HighLimit/issuer")?,
        field("/Balance/currency")?,
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
            let got = get_balance_changes(meta).unwrap();
            let accounts = |v: &[AccountBalanceChanges]| {
                v.iter().map(|a| a.account.clone()).collect::<Vec<_>>()
            };
            assert_eq!(accounts(&got), accounts(&expected), "{}", case["name"]);
            assert_eq!(sorted(got), sorted(expected), "{}", case["name"]);
        }
    }

    #[test]
    fn rejects_what_is_not_metadata() {
        assert!(get_balance_changes(&serde_json::json!({"hash": "00"})).is_err());
        assert!(
            get_balance_changes(&serde_json::json!({"AffectedNodes": []}))
                .unwrap()
                .is_empty()
        );
    }
}
