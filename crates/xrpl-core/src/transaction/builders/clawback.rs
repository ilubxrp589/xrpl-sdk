use super::{merge_common, Transaction, TxCommon};
use crate::CoreError;
use serde_json::{json, Value};

/// Clawback transaction.
#[derive(Debug, Clone)]
pub struct Clawback {
    /// The fields every transaction has.
    pub common: TxCommon,
    /// How much (`Amount`).
    pub amount: Value,
}
impl Transaction for Clawback {
    fn transaction_type(&self) -> &'static str {
        "Clawback"
    }
    fn to_json(&self) -> Value {
        let mut obj = serde_json::Map::new();
        obj.insert("TransactionType".into(), json!("Clawback"));
        obj.insert("Amount".into(), self.amount.clone());
        merge_common(&mut obj, &self.common);
        Value::Object(obj)
    }
    fn validate(&self) -> Result<(), CoreError> {
        Ok(())
    }
}
/// Builds a [`Clawback`] (from [`Clawback::builder`]); `build` checks the required fields.
pub struct ClawbackBuilder {
    common: TxCommon,
    amount: Option<Value>,
}
impl ClawbackBuilder {
    /// A builder for the transaction from `account`.
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: TxCommon::new(account),
            amount: None,
        }
    }
    /// Set `Amount`.
    pub fn amount(mut self, a: Value) -> Self {
        self.amount = Some(a);
        self
    }
    /// Set `Fee`.
    pub fn fee(mut self, f: impl Into<String>) -> Self {
        self.common.fee = Some(f.into());
        self
    }
    /// Set `Sequence`.
    pub fn sequence(mut self, s: u32) -> Self {
        self.common.sequence = Some(s);
        self
    }
    /// Set `LastLedgerSequence`.
    pub fn last_ledger_sequence(mut self, l: u32) -> Self {
        self.common.last_ledger_sequence = Some(l);
        self
    }
    /// Check the required fields and build the transaction.
    pub fn build(self) -> Result<Clawback, CoreError> {
        let amount = self
            .amount
            .ok_or_else(|| CoreError::ValidationError("Clawback: amount is required".into()))?;
        Ok(Clawback {
            common: self.common,
            amount,
        })
    }
}
impl Clawback {
    /// Start building one from `account`.
    pub fn builder(account: impl Into<String>) -> ClawbackBuilder {
        ClawbackBuilder::new(account)
    }
}
