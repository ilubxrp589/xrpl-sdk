use super::{merge_common, Transaction, TxCommon};
use crate::CoreError;
use serde_json::{json, Value};

/// Payment transaction: send XRP or IOU between accounts.
#[derive(Debug, Clone)]
pub struct Payment {
    pub common: TxCommon,
    pub amount: Value,
    pub destination: String,
    pub destination_tag: Option<u32>,
    pub send_max: Option<Value>,
    pub deliver_min: Option<Value>,
    pub paths: Option<Vec<Vec<Value>>>,
    pub invoice_id: Option<String>,
}

impl Transaction for Payment {
    fn transaction_type(&self) -> &'static str {
        "Payment"
    }

    fn to_json(&self) -> Value {
        let mut obj = serde_json::Map::new();
        obj.insert("TransactionType".into(), json!("Payment"));
        obj.insert("Amount".into(), self.amount.clone());
        obj.insert("Destination".into(), json!(self.destination));
        if let Some(dt) = self.destination_tag {
            obj.insert("DestinationTag".into(), json!(dt));
        }
        if let Some(ref sm) = self.send_max {
            obj.insert("SendMax".into(), sm.clone());
        }
        if let Some(ref dm) = self.deliver_min {
            obj.insert("DeliverMin".into(), dm.clone());
        }
        if let Some(ref p) = self.paths {
            obj.insert("Paths".into(), json!(p));
        }
        if let Some(ref id) = self.invoice_id {
            obj.insert("InvoiceID".into(), json!(id));
        }
        merge_common(&mut obj, &self.common);
        Value::Object(obj)
    }

    fn validate(&self) -> Result<(), CoreError> {
        if self.destination.is_empty() {
            return Err(CoreError::ValidationError(
                "Payment: destination is required".into(),
            ));
        }
        Ok(())
    }
}

pub struct PaymentBuilder {
    common: TxCommon,
    amount: Option<Value>,
    destination: Option<String>,
    destination_tag: Option<u32>,
    send_max: Option<Value>,
    deliver_min: Option<Value>,
    paths: Option<Vec<Vec<Value>>>,
    invoice_id: Option<String>,
}

impl PaymentBuilder {
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: TxCommon::new(account),
            amount: None,
            destination: None,
            destination_tag: None,
            send_max: None,
            deliver_min: None,
            paths: None,
            invoice_id: None,
        }
    }

    pub fn amount(mut self, amount: Value) -> Self {
        self.amount = Some(amount);
        self
    }
    pub fn destination(mut self, dest: impl Into<String>) -> Self {
        self.destination = Some(dest.into());
        self
    }
    pub fn destination_tag(mut self, tag: u32) -> Self {
        self.destination_tag = Some(tag);
        self
    }
    pub fn send_max(mut self, v: Value) -> Self {
        self.send_max = Some(v);
        self
    }
    pub fn deliver_min(mut self, v: Value) -> Self {
        self.deliver_min = Some(v);
        self
    }
    pub fn paths(mut self, p: Vec<Vec<Value>>) -> Self {
        self.paths = Some(p);
        self
    }
    pub fn invoice_id(mut self, id: impl Into<String>) -> Self {
        self.invoice_id = Some(id.into());
        self
    }
    pub fn fee(mut self, fee: impl Into<String>) -> Self {
        self.common.fee = Some(fee.into());
        self
    }
    pub fn sequence(mut self, seq: u32) -> Self {
        self.common.sequence = Some(seq);
        self
    }
    pub fn last_ledger_sequence(mut self, lls: u32) -> Self {
        self.common.last_ledger_sequence = Some(lls);
        self
    }
    pub fn flags(mut self, f: u32) -> Self {
        self.common.flags = Some(f);
        self
    }
    pub fn memos(mut self, m: Vec<Value>) -> Self {
        self.common.memos = Some(m);
        self
    }
    pub fn source_tag(mut self, t: u32) -> Self {
        self.common.source_tag = Some(t);
        self
    }

    pub fn build(self) -> Result<Payment, CoreError> {
        let amount = self
            .amount
            .ok_or_else(|| CoreError::ValidationError("Payment: amount is required".into()))?;
        let destination = self
            .destination
            .ok_or_else(|| CoreError::ValidationError("Payment: destination is required".into()))?;
        let tx = Payment {
            common: self.common,
            amount,
            destination,
            destination_tag: self.destination_tag,
            send_max: self.send_max,
            deliver_min: self.deliver_min,
            paths: self.paths,
            invoice_id: self.invoice_id,
        };
        tx.validate()?;
        Ok(tx)
    }
}

impl Payment {
    pub fn builder(account: impl Into<String>) -> PaymentBuilder {
        PaymentBuilder::new(account)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payment_build_success() {
        let tx = Payment::builder("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")
            .amount(json!("1000000"))
            .destination("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe")
            .fee("12")
            .sequence(1)
            .build()
            .unwrap();

        let j = tx.to_json();
        assert_eq!(j["TransactionType"], "Payment");
        assert_eq!(j["Amount"], "1000000");
        assert_eq!(j["Destination"], "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe");
        assert_eq!(j["Fee"], "12");
        assert_eq!(j["Sequence"], 1);
    }

    #[test]
    fn payment_missing_amount() {
        let result = Payment::builder("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")
            .destination("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe")
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn payment_missing_destination() {
        let result = Payment::builder("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")
            .amount(json!("1000000"))
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn payment_to_json_roundtrip() {
        let tx = Payment::builder("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")
            .amount(json!({"value": "100", "currency": "USD", "issuer": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe"}))
            .destination("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe")
            .destination_tag(42)
            .fee("12")
            .sequence(1)
            .build()
            .unwrap();

        let j = tx.to_json();
        assert_eq!(j["DestinationTag"], 42);
        assert_eq!(j["Amount"]["currency"], "USD");
    }
}
