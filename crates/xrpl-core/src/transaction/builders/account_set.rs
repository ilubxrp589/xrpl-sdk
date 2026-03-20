use super::{merge_common, Transaction, TxCommon};
use crate::CoreError;
use serde_json::{json, Value};

/// AccountSet transaction: modify account settings.
#[derive(Debug, Clone)]
pub struct AccountSet {
    pub common: TxCommon,
    pub set_flag: Option<u32>,
    pub clear_flag: Option<u32>,
    pub domain: Option<String>,
    pub email_hash: Option<String>,
    pub message_key: Option<String>,
    pub transfer_rate: Option<u32>,
    pub tick_size: Option<u8>,
}

impl Transaction for AccountSet {
    fn transaction_type(&self) -> &'static str {
        "AccountSet"
    }

    fn to_json(&self) -> Value {
        let mut obj = serde_json::Map::new();
        obj.insert("TransactionType".into(), json!("AccountSet"));
        if let Some(sf) = self.set_flag {
            obj.insert("SetFlag".into(), json!(sf));
        }
        if let Some(cf) = self.clear_flag {
            obj.insert("ClearFlag".into(), json!(cf));
        }
        if let Some(ref d) = self.domain {
            obj.insert("Domain".into(), json!(d));
        }
        if let Some(ref eh) = self.email_hash {
            obj.insert("EmailHash".into(), json!(eh));
        }
        if let Some(ref mk) = self.message_key {
            obj.insert("MessageKey".into(), json!(mk));
        }
        if let Some(tr) = self.transfer_rate {
            obj.insert("TransferRate".into(), json!(tr));
        }
        if let Some(ts) = self.tick_size {
            obj.insert("TickSize".into(), json!(ts));
        }
        merge_common(&mut obj, &self.common);
        Value::Object(obj)
    }

    fn validate(&self) -> Result<(), CoreError> {
        Ok(())
    }
}

pub struct AccountSetBuilder {
    common: TxCommon,
    set_flag: Option<u32>,
    clear_flag: Option<u32>,
    domain: Option<String>,
    email_hash: Option<String>,
    message_key: Option<String>,
    transfer_rate: Option<u32>,
    tick_size: Option<u8>,
}

impl AccountSetBuilder {
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: TxCommon::new(account),
            set_flag: None,
            clear_flag: None,
            domain: None,
            email_hash: None,
            message_key: None,
            transfer_rate: None,
            tick_size: None,
        }
    }
    pub fn set_flag(mut self, f: u32) -> Self {
        self.set_flag = Some(f);
        self
    }
    pub fn clear_flag(mut self, f: u32) -> Self {
        self.clear_flag = Some(f);
        self
    }
    pub fn domain(mut self, d: impl Into<String>) -> Self {
        self.domain = Some(d.into());
        self
    }
    pub fn email_hash(mut self, h: impl Into<String>) -> Self {
        self.email_hash = Some(h.into());
        self
    }
    pub fn message_key(mut self, k: impl Into<String>) -> Self {
        self.message_key = Some(k.into());
        self
    }
    pub fn transfer_rate(mut self, r: u32) -> Self {
        self.transfer_rate = Some(r);
        self
    }
    pub fn tick_size(mut self, t: u8) -> Self {
        self.tick_size = Some(t);
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

    pub fn build(self) -> Result<AccountSet, CoreError> {
        Ok(AccountSet {
            common: self.common,
            set_flag: self.set_flag,
            clear_flag: self.clear_flag,
            domain: self.domain,
            email_hash: self.email_hash,
            message_key: self.message_key,
            transfer_rate: self.transfer_rate,
            tick_size: self.tick_size,
        })
    }
}

impl AccountSet {
    pub fn builder(account: impl Into<String>) -> AccountSetBuilder {
        AccountSetBuilder::new(account)
    }
}
