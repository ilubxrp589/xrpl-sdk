use super::{merge_common, Transaction, TxCommon};
use crate::CoreError;
use serde_json::{json, Value};

/// SetRegularKey transaction.
#[derive(Debug, Clone)]
pub struct SetRegularKey {
    /// The fields every transaction has.
    pub common: TxCommon,
    /// The regular key pair's account; absent to remove it (`RegularKey`).
    pub regular_key: Option<String>,
}
impl Transaction for SetRegularKey {
    fn transaction_type(&self) -> &'static str {
        "SetRegularKey"
    }
    fn to_json(&self) -> Value {
        let mut obj = serde_json::Map::new();
        obj.insert("TransactionType".into(), json!("SetRegularKey"));
        if let Some(ref rk) = self.regular_key {
            obj.insert("RegularKey".into(), json!(rk));
        }
        merge_common(&mut obj, &self.common);
        Value::Object(obj)
    }
    fn validate(&self) -> Result<(), CoreError> {
        if self.regular_key.as_deref() == Some(self.common.account.as_str()) {
            return Err(super::invalid(
                "SetRegularKey",
                "the regular key cannot be the account's own address",
            ));
        }
        if let Some(k) = &self.regular_key {
            crate::address::decode_account_id(k)?;
        }
        Ok(())
    }
}
/// Builds a [`SetRegularKey`] (from [`SetRegularKey::builder`]); `build` checks the required fields.
#[derive(Debug)]
pub struct SetRegularKeyBuilder {
    common: TxCommon,
    regular_key: Option<String>,
}
impl SetRegularKeyBuilder {
    /// A builder for the transaction from `account`.
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: TxCommon::new(account),
            regular_key: None,
        }
    }
    /// Set `RegularKey`.
    pub fn regular_key(mut self, k: impl Into<String>) -> Self {
        self.regular_key = Some(k.into());
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
    pub fn build(self) -> Result<SetRegularKey, CoreError> {
        let tx = SetRegularKey {
            common: self.common,
            regular_key: self.regular_key,
        };
        tx.validate()?;
        Ok(tx)
    }
}
impl SetRegularKey {
    /// Start building one from `account`.
    pub fn builder(account: impl Into<String>) -> SetRegularKeyBuilder {
        SetRegularKeyBuilder::new(account)
    }
}
