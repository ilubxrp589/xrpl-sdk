use super::{merge_common, Transaction, TxCommon};
use crate::CoreError;
use serde_json::{json, Value};

/// Clawback transaction.
#[derive(Debug, Clone)]
pub struct Clawback {
    /// The fields every transaction has.
    pub common: TxCommon,
    /// How much to take back (`Amount`). A token: `{"currency", "issuer",
    /// "value"}` where `issuer` names the **holder** (rippled's convention
    /// for Clawback). An MPT: `{"mpt_issuance_id", "value"}`, with the
    /// holder in `holder`.
    pub amount: Value,
    /// The holder of an MPT to claw back from (`Holder`); not set for a token.
    pub holder: Option<String>,
}
impl Transaction for Clawback {
    fn transaction_type(&self) -> &'static str {
        "Clawback"
    }
    fn to_json(&self) -> Value {
        let mut obj = serde_json::Map::new();
        obj.insert("TransactionType".into(), json!("Clawback"));
        obj.insert("Amount".into(), self.amount.clone());
        if let Some(h) = &self.holder {
            obj.insert("Holder".into(), json!(h));
        }
        merge_common(&mut obj, &self.common);
        Value::Object(obj)
    }
    fn validate(&self) -> Result<(), CoreError> {
        let bad = |m: &str| super::invalid("Clawback", m);
        let value = self
            .amount
            .get("value")
            .and_then(Value::as_str)
            .ok_or_else(|| bad("Amount is a token or an MPT amount (XRP cannot be clawed back)"))?;
        let parsed = crate::types::IouAmount::from_decimal(value)?;
        if parsed.is_zero() || parsed.is_negative {
            return Err(bad("the amount must be positive"));
        }
        let holder = if self.amount.get("mpt_issuance_id").is_some() {
            self.holder
                .as_deref()
                .ok_or_else(|| bad("an MPT clawback names the holder in Holder"))?
        } else {
            if self.holder.is_some() {
                return Err(bad(
                    "a token clawback names the holder in Amount's issuer, not Holder",
                ));
            }
            self.amount
                .get("issuer")
                .and_then(Value::as_str)
                .ok_or_else(|| bad("a token amount names the holder in its issuer"))?
        };
        crate::address::decode_account_id(holder)?;
        if holder == self.common.account {
            return Err(bad("the holder cannot be the issuer itself"));
        }
        Ok(())
    }
}
/// Builds a [`Clawback`] (from [`Clawback::builder`]); `build` checks the required fields.
#[derive(Debug)]
pub struct ClawbackBuilder {
    common: TxCommon,
    amount: Option<Value>,
    holder: Option<String>,
}
impl ClawbackBuilder {
    /// A builder for the transaction from `account`.
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: TxCommon::new(account),
            amount: None,
            holder: None,
        }
    }
    /// Set `Amount`.
    pub fn amount(mut self, a: Value) -> Self {
        self.amount = Some(a);
        self
    }
    /// Set `Holder` (MPT clawbacks only).
    pub fn holder(mut self, h: impl Into<String>) -> Self {
        self.holder = Some(h.into());
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
    ///
    /// # Errors
    ///
    /// [`CoreError::ValidationError`] if `amount` is missing, has no `value`
    /// (XRP cannot be clawed back) or is not positive, the holder is missing or
    /// in the wrong field (a token's is its issuer, an MPT's is `Holder`), or
    /// the holder is the account itself; [`CoreError::InvalidAmount`] for a
    /// `value` that does not parse; an address error
    /// ([`CoreError::InvalidAddress`], `InvalidBase58` or `InvalidChecksum`)
    /// for a holder that is not a classic address.
    pub fn build(self) -> Result<Clawback, CoreError> {
        let amount = self
            .amount
            .ok_or_else(|| CoreError::ValidationError("Clawback: amount is required".into()))?;
        let tx = Clawback {
            common: self.common,
            amount,
            holder: self.holder,
        };
        tx.validate()?;
        Ok(tx)
    }
}
impl Clawback {
    /// Start building one from `account`.
    pub fn builder(account: impl Into<String>) -> ClawbackBuilder {
        ClawbackBuilder::new(account)
    }
}
