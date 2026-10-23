use super::{merge_common, Transaction, TxCommon};
use crate::CoreError;
use serde_json::{json, Value};

/// SignerListSet transaction.
#[derive(Debug, Clone)]
pub struct SignerListSet {
    /// The fields every transaction has.
    pub common: TxCommon,
    /// The total signer weight a multi-signature needs; 0 deletes the list (`SignerQuorum`).
    pub signer_quorum: u32,
    /// The signers and their weights (`SignerEntries`).
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
        let bad = |m: String| super::invalid("SignerListSet", &m);
        let entries = self.signer_entries.as_deref().unwrap_or_default();
        if self.signer_quorum == 0 {
            // A quorum of 0 deletes the list, which then has no entries.
            return if entries.is_empty() {
                Ok(())
            } else {
                Err(bad(
                    "a SignerQuorum of 0 deletes the list: leave SignerEntries out".into(),
                ))
            };
        }
        if entries.is_empty() || entries.len() > 32 {
            return Err(bad(format!("1 to 32 SignerEntries, got {}", entries.len())));
        }
        let mut seen = Vec::with_capacity(entries.len());
        let mut total: u64 = 0;
        for entry in entries {
            let inner = entry
                .get("SignerEntry")
                .ok_or_else(|| bad("each entry is {\"SignerEntry\": {…}}".into()))?;
            let account = inner
                .get("Account")
                .and_then(Value::as_str)
                .ok_or_else(|| bad("a SignerEntry has no Account".into()))?;
            let id = crate::address::decode_account_id(account)?;
            if account == self.common.account {
                return Err(bad("the account cannot be on its own signer list".into()));
            }
            if seen.contains(&id) {
                return Err(bad(format!("{account} is listed twice")));
            }
            seen.push(id);
            let weight = inner
                .get("SignerWeight")
                .and_then(Value::as_u64)
                .filter(|w| (1..=u64::from(u16::MAX)).contains(w))
                .ok_or_else(|| bad(format!("{account}'s SignerWeight is 1 to 65535")))?;
            total += weight;
        }
        if total < u64::from(self.signer_quorum) {
            return Err(bad(format!(
                "the weights add up to {total}, less than the quorum {}",
                self.signer_quorum
            )));
        }
        Ok(())
    }
}
/// Builds a [`SignerListSet`] (from [`SignerListSet::builder`]); `build` checks the required fields.
#[derive(Debug)]
pub struct SignerListSetBuilder {
    common: TxCommon,
    signer_quorum: Option<u32>,
    signer_entries: Option<Vec<Value>>,
}
impl SignerListSetBuilder {
    /// A builder for the transaction from `account`.
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: TxCommon::new(account),
            signer_quorum: None,
            signer_entries: None,
        }
    }
    /// Set `SignerQuorum`.
    pub fn signer_quorum(mut self, q: u32) -> Self {
        self.signer_quorum = Some(q);
        self
    }
    /// Set `SignerEntries`.
    pub fn signer_entries(mut self, entries: Vec<Value>) -> Self {
        self.signer_entries = Some(entries);
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
    pub fn build(self) -> Result<SignerListSet, CoreError> {
        let signer_quorum = self.signer_quorum.ok_or_else(|| {
            CoreError::ValidationError("SignerListSet: signer_quorum is required".into())
        })?;
        let tx = SignerListSet {
            common: self.common,
            signer_quorum,
            signer_entries: self.signer_entries,
        };
        tx.validate()?;
        Ok(tx)
    }
}
impl SignerListSet {
    /// Start building one from `account`.
    pub fn builder(account: impl Into<String>) -> SignerListSetBuilder {
        SignerListSetBuilder::new(account)
    }
}
