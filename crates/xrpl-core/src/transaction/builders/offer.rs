use super::{merge_common, Transaction, TxCommon};
use crate::CoreError;
use serde_json::{json, Value};

/// OfferCreate transaction: place an order on the DEX.
#[derive(Debug, Clone)]
pub struct OfferCreate {
    /// The fields every transaction has.
    pub common: TxCommon,
    /// What the offer wants in return (`TakerPays`): XRP as a string of **drops** (`"1000000"` is 1 XRP; `utils::xrp_to_drops`), or a token `{"currency", "issuer", "value"}`.
    pub taker_pays: Value,
    /// What the offer gives (`TakerGets`): XRP as a string of **drops** (`"1000000"` is 1 XRP; `utils::xrp_to_drops`), or a token `{"currency", "issuer", "value"}`.
    pub taker_gets: Value,
    /// When it expires (`Expiration`). Ledger time: seconds since 2000-01-01 UTC, not Unix time (`utils::unix_to_ripple_time`).
    pub expiration: Option<u32>,
    /// The sequence of the transaction that created the entry (`OfferSequence`).
    pub offer_sequence: Option<u32>,
}

impl Transaction for OfferCreate {
    fn transaction_type(&self) -> &'static str {
        "OfferCreate"
    }

    fn to_json(&self) -> Value {
        let mut obj = serde_json::Map::new();
        obj.insert("TransactionType".into(), json!("OfferCreate"));
        obj.insert("TakerPays".into(), self.taker_pays.clone());
        obj.insert("TakerGets".into(), self.taker_gets.clone());
        if let Some(exp) = self.expiration {
            obj.insert("Expiration".into(), json!(exp));
        }
        if let Some(os) = self.offer_sequence {
            obj.insert("OfferSequence".into(), json!(os));
        }
        merge_common(&mut obj, &self.common);
        Value::Object(obj)
    }

    fn validate(&self) -> Result<(), CoreError> {
        Ok(())
    }
}

/// Builds a [`OfferCreate`] (from [`OfferCreate::builder`]); `build` checks the required fields.
pub struct OfferCreateBuilder {
    common: TxCommon,
    taker_pays: Option<Value>,
    taker_gets: Option<Value>,
    expiration: Option<u32>,
    offer_sequence: Option<u32>,
}

impl OfferCreateBuilder {
    /// A builder for the transaction from `account`.
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: TxCommon::new(account),
            taker_pays: None,
            taker_gets: None,
            expiration: None,
            offer_sequence: None,
        }
    }
    /// Set `TakerPays`.
    pub fn taker_pays(mut self, v: Value) -> Self {
        self.taker_pays = Some(v);
        self
    }
    /// Set `TakerGets`.
    pub fn taker_gets(mut self, v: Value) -> Self {
        self.taker_gets = Some(v);
        self
    }
    /// Set `Expiration`.
    pub fn expiration(mut self, e: u32) -> Self {
        self.expiration = Some(e);
        self
    }
    /// Set `OfferSequence`.
    pub fn offer_sequence(mut self, s: u32) -> Self {
        self.offer_sequence = Some(s);
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
    /// Set `Flags`.
    pub fn flags(mut self, f: u32) -> Self {
        self.common.flags = Some(f);
        self
    }

    /// Check the required fields and build the transaction.
    pub fn build(self) -> Result<OfferCreate, CoreError> {
        let taker_pays = self.taker_pays.ok_or_else(|| {
            CoreError::ValidationError("OfferCreate: taker_pays is required".into())
        })?;
        let taker_gets = self.taker_gets.ok_or_else(|| {
            CoreError::ValidationError("OfferCreate: taker_gets is required".into())
        })?;
        Ok(OfferCreate {
            common: self.common,
            taker_pays,
            taker_gets,
            expiration: self.expiration,
            offer_sequence: self.offer_sequence,
        })
    }
}

impl OfferCreate {
    /// Start building one from `account`.
    pub fn builder(account: impl Into<String>) -> OfferCreateBuilder {
        OfferCreateBuilder::new(account)
    }
}

/// OfferCancel transaction: cancel an existing offer.
#[derive(Debug, Clone)]
pub struct OfferCancel {
    /// The fields every transaction has.
    pub common: TxCommon,
    /// The sequence of the transaction that created the entry (`OfferSequence`).
    pub offer_sequence: u32,
}

impl Transaction for OfferCancel {
    fn transaction_type(&self) -> &'static str {
        "OfferCancel"
    }

    fn to_json(&self) -> Value {
        let mut obj = serde_json::Map::new();
        obj.insert("TransactionType".into(), json!("OfferCancel"));
        obj.insert("OfferSequence".into(), json!(self.offer_sequence));
        merge_common(&mut obj, &self.common);
        Value::Object(obj)
    }

    fn validate(&self) -> Result<(), CoreError> {
        Ok(())
    }
}

/// Builds a [`OfferCancel`] (from [`OfferCancel::builder`]); `build` checks the required fields.
pub struct OfferCancelBuilder {
    common: TxCommon,
    offer_sequence: Option<u32>,
}

impl OfferCancelBuilder {
    /// A builder for the transaction from `account`.
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: TxCommon::new(account),
            offer_sequence: None,
        }
    }
    /// Set `OfferSequence`.
    pub fn offer_sequence(mut self, s: u32) -> Self {
        self.offer_sequence = Some(s);
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
    pub fn build(self) -> Result<OfferCancel, CoreError> {
        let offer_sequence = self.offer_sequence.ok_or_else(|| {
            CoreError::ValidationError("OfferCancel: offer_sequence is required".into())
        })?;
        Ok(OfferCancel {
            common: self.common,
            offer_sequence,
        })
    }
}

impl OfferCancel {
    /// Start building one from `account`.
    pub fn builder(account: impl Into<String>) -> OfferCancelBuilder {
        OfferCancelBuilder::new(account)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn offer_create_build_success() {
        let tx = OfferCreate::builder("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")
            .taker_pays(json!("1000000"))
            .taker_gets(json!({"value": "100", "currency": "USD", "issuer": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe"}))
            .build()
            .unwrap();
        let j = tx.to_json();
        assert_eq!(j["TransactionType"], "OfferCreate");
    }

    #[test]
    fn offer_create_missing_taker_pays() {
        let r = OfferCreate::builder("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")
            .taker_gets(json!("1000000"))
            .build();
        assert!(r.is_err());
    }

    #[test]
    fn offer_cancel_build_success() {
        let tx = OfferCancel::builder("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")
            .offer_sequence(5)
            .build()
            .unwrap();
        assert_eq!(tx.to_json()["OfferSequence"], 5);
    }
}
