use super::{merge_common, Transaction, TxCommon};
use crate::CoreError;
use serde_json::{json, Value};

/// AccountSet transaction: modify account settings.
#[derive(Debug, Clone)]
pub struct AccountSet {
    /// The fields every transaction has.
    pub common: TxCommon,
    /// An account flag to turn on (`ASF_*`) (`SetFlag`).
    pub set_flag: Option<u32>,
    /// An account flag to turn off (`ASF_*`) (`ClearFlag`).
    pub clear_flag: Option<u32>,
    /// A domain the account claims, hex (ASCII, lower case) (`Domain`).
    pub domain: Option<String>,
    /// The MD5 of an email address, for an avatar (`EmailHash`).
    pub email_hash: Option<String>,
    /// A public key for sending the account encrypted messages (`MessageKey`).
    pub message_key: Option<String>,
    /// The fee on transfers of the account's tokens, in billionths over 1,000,000,000 (`TransferRate`).
    pub transfer_rate: Option<u32>,
    /// Significant digits for offers of the account's tokens (3–15; 0 clears) (`TickSize`).
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
        let bad = |m: &str| super::invalid("AccountSet", m);
        if self.set_flag.is_some() && self.set_flag == self.clear_flag {
            return Err(bad("SetFlag and ClearFlag are the same flag"));
        }
        if let Some(t) = self.tick_size {
            if t != 0 && !(3..=15).contains(&t) {
                return Err(bad("TickSize is 0 (clear) or 3 to 15"));
            }
        }
        if let Some(r) = self.transfer_rate {
            if r != 0 && !(1_000_000_000..=2_000_000_000).contains(&r) {
                return Err(bad(
                    "TransferRate is 0 (none) or 1,000,000,000 to 2,000,000,000 (0% to 100%)",
                ));
            }
        }
        super::check_hex(self.domain.as_deref(), "Domain", 256)?;
        super::check_hex(self.message_key.as_deref(), "MessageKey", 33)?;
        if let Some(h) = &self.email_hash {
            if h.len() != 32 || hex::decode(h).is_err() {
                return Err(bad("EmailHash is 32 hex digits (an MD5 hash)"));
            }
        }
        Ok(())
    }
}

/// Builds a [`AccountSet`] (from [`AccountSet::builder`]); `build` checks the required fields.
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
    /// A builder for the transaction from `account`.
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
    /// Set `SetFlag`.
    pub fn set_flag(mut self, f: u32) -> Self {
        self.set_flag = Some(f);
        self
    }
    /// Set `ClearFlag`.
    pub fn clear_flag(mut self, f: u32) -> Self {
        self.clear_flag = Some(f);
        self
    }
    /// Set `Domain`.
    pub fn domain(mut self, d: impl Into<String>) -> Self {
        self.domain = Some(d.into());
        self
    }
    /// Set `EmailHash`.
    pub fn email_hash(mut self, h: impl Into<String>) -> Self {
        self.email_hash = Some(h.into());
        self
    }
    /// Set `MessageKey`.
    pub fn message_key(mut self, k: impl Into<String>) -> Self {
        self.message_key = Some(k.into());
        self
    }
    /// Set `TransferRate`.
    pub fn transfer_rate(mut self, r: u32) -> Self {
        self.transfer_rate = Some(r);
        self
    }
    /// Set `TickSize`.
    pub fn tick_size(mut self, t: u8) -> Self {
        self.tick_size = Some(t);
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
    pub fn build(self) -> Result<AccountSet, CoreError> {
        let tx = AccountSet {
            common: self.common,
            set_flag: self.set_flag,
            clear_flag: self.clear_flag,
            domain: self.domain,
            email_hash: self.email_hash,
            message_key: self.message_key,
            transfer_rate: self.transfer_rate,
            tick_size: self.tick_size,
        };
        tx.validate()?;
        Ok(tx)
    }
}

impl AccountSet {
    /// Start building one from `account`.
    pub fn builder(account: impl Into<String>) -> AccountSetBuilder {
        AccountSetBuilder::new(account)
    }
}
