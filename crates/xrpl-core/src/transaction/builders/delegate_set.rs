use super::{merge_common, Transaction, TxCommon};
use crate::CoreError;
use serde_json::{json, Value};

/// DelegateSet transaction.
#[derive(Debug, Clone)]
pub struct DelegateSet {
    /// The fields every transaction has.
    pub common: TxCommon,
    /// The account to authorize (`Authorize`).
    pub authorize: String,
    /// The transaction types or granular permissions to grant (`Permissions`).
    pub permissions: Vec<Value>,
}
impl Transaction for DelegateSet {
    fn transaction_type(&self) -> &'static str {
        "DelegateSet"
    }
    fn to_json(&self) -> Value {
        let mut obj = serde_json::Map::new();
        obj.insert("TransactionType".into(), json!("DelegateSet"));
        obj.insert("Authorize".into(), json!(self.authorize));
        obj.insert("Permissions".into(), json!(self.permissions));
        merge_common(&mut obj, &self.common);
        Value::Object(obj)
    }
    fn validate(&self) -> Result<(), CoreError> {
        Ok(())
    }
}
/// Builds a [`DelegateSet`] (from [`DelegateSet::builder`]); `build` checks the required fields.
pub struct DelegateSetBuilder {
    common: TxCommon,
    authorize: Option<String>,
    permissions: Option<Vec<Value>>,
}
impl DelegateSetBuilder {
    /// A builder for the transaction from `account`.
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: TxCommon::new(account),
            authorize: None,
            permissions: None,
        }
    }
    /// Set `Authorize`.
    pub fn authorize(mut self, a: impl Into<String>) -> Self {
        self.authorize = Some(a.into());
        self
    }
    /// Set `Permissions`.
    pub fn permissions(mut self, p: Vec<Value>) -> Self {
        self.permissions = Some(p);
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
    pub fn build(self) -> Result<DelegateSet, CoreError> {
        let authorize = self.authorize.ok_or_else(|| {
            CoreError::ValidationError("DelegateSet: authorize is required".into())
        })?;
        let permissions = self.permissions.ok_or_else(|| {
            CoreError::ValidationError("DelegateSet: permissions is required".into())
        })?;
        Ok(DelegateSet {
            common: self.common,
            authorize,
            permissions,
        })
    }
}
impl DelegateSet {
    /// Start building one from `account`.
    pub fn builder(account: impl Into<String>) -> DelegateSetBuilder {
        DelegateSetBuilder::new(account)
    }
}
