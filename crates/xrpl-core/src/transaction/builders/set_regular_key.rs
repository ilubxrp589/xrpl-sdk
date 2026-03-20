use super::{merge_common, Transaction, TxCommon};
use crate::CoreError;
use serde_json::{json, Value};

/// SetRegularKey transaction.
#[derive(Debug, Clone)]
pub struct SetRegularKey {
    pub common: TxCommon,
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
        Ok(())
    }
}
pub struct SetRegularKeyBuilder {
    common: TxCommon,
    regular_key: Option<String>,
}
impl SetRegularKeyBuilder {
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: TxCommon::new(account),
            regular_key: None,
        }
    }
    pub fn regular_key(mut self, k: impl Into<String>) -> Self {
        self.regular_key = Some(k.into());
        self
    }
    pub fn fee(mut self, f: impl Into<String>) -> Self {
        self.common.fee = Some(f.into());
        self
    }
    pub fn sequence(mut self, s: u32) -> Self {
        self.common.sequence = Some(s);
        self
    }
    pub fn last_ledger_sequence(mut self, l: u32) -> Self {
        self.common.last_ledger_sequence = Some(l);
        self
    }
    pub fn build(self) -> Result<SetRegularKey, CoreError> {
        Ok(SetRegularKey {
            common: self.common,
            regular_key: self.regular_key,
        })
    }
}
impl SetRegularKey {
    pub fn builder(account: impl Into<String>) -> SetRegularKeyBuilder {
        SetRegularKeyBuilder::new(account)
    }
}
