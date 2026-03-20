use super::{merge_common, Transaction, TxCommon};
use crate::CoreError;
use serde_json::{json, Value};

/// SignerListSet transaction.
#[derive(Debug, Clone)]
pub struct SignerListSet {
    pub common: TxCommon,
    pub signer_quorum: u32,
    pub signer_entries: Option<Vec<Value>>,
}
impl Transaction for SignerListSet {
    fn transaction_type(&self) -> &'static str {
        "SignerListSet"
    }
    fn to_json(&self) -> Value {
        let mut obj = serde_json::Map::new();
        obj.insert("TransactionType".into(), json!("SignerListSet"));
        obj.insert("SignerQuorum".into(), json!(self.signer_quorum));
        if let Some(ref se) = self.signer_entries {
            obj.insert("SignerEntries".into(), json!(se));
        }
        merge_common(&mut obj, &self.common);
        Value::Object(obj)
    }
    fn validate(&self) -> Result<(), CoreError> {
        Ok(())
    }
}
pub struct SignerListSetBuilder {
    common: TxCommon,
    signer_quorum: Option<u32>,
    signer_entries: Option<Vec<Value>>,
}
impl SignerListSetBuilder {
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: TxCommon::new(account),
            signer_quorum: None,
            signer_entries: None,
        }
    }
    pub fn signer_quorum(mut self, q: u32) -> Self {
        self.signer_quorum = Some(q);
        self
    }
    pub fn signer_entries(mut self, entries: Vec<Value>) -> Self {
        self.signer_entries = Some(entries);
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
    pub fn build(self) -> Result<SignerListSet, CoreError> {
        let signer_quorum = self.signer_quorum.ok_or_else(|| {
            CoreError::ValidationError("SignerListSet: signer_quorum is required".into())
        })?;
        Ok(SignerListSet {
            common: self.common,
            signer_quorum,
            signer_entries: self.signer_entries,
        })
    }
}
impl SignerListSet {
    pub fn builder(account: impl Into<String>) -> SignerListSetBuilder {
        SignerListSetBuilder::new(account)
    }
}
