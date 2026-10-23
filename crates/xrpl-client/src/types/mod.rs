//! Typed `rippled` and Clio responses, grouped as xrpl.org groups the
//! methods, and [`LedgerIndex`] for choosing a ledger.
//!
//! Every response type here is checked against a response recorded from
//! `rippled` 3.4.1 (or Clio): it must parse, and serializing it back must
//! keep every field the server sent (`tests/rippled_responses.rs`).
//! Responses are read as `rippled`'s API version 1, the default when a
//! request does not name one.

mod account;
mod clio;
mod ledger;
mod path;
mod server;
mod transaction;
mod vault;

pub use account::*;
pub use clio::*;
pub use ledger::*;
pub use path::*;
pub use server::*;
pub use transaction::*;
pub use vault::*;
pub use xrpl_core::types::{Amount, LedgerObject};

use serde::Serialize;

/// Which ledger a request reads: serializes as `rippled` expects
/// (`"validated"`, `"current"`, `"closed"` or the number).
///
/// `Validated` (the default) is final; `Current` is the open ledger, whose
/// contents can still change; `Closed` is the latest closed ledger, not yet
/// necessarily validated.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LedgerIndex {
    /// The latest validated ledger: final.
    #[default]
    Validated,
    /// The open ledger: it can still change.
    Current,
    /// The latest closed ledger, not necessarily validated.
    Closed,
    /// A ledger by its index.
    Index(u32),
}

impl LedgerIndex {
    /// The JSON `rippled` takes for this ledger (what it serializes as).
    pub fn to_value(self) -> serde_json::Value {
        match self {
            LedgerIndex::Validated => serde_json::Value::String("validated".into()),
            LedgerIndex::Current => serde_json::Value::String("current".into()),
            LedgerIndex::Closed => serde_json::Value::String("closed".into()),
            LedgerIndex::Index(n) => serde_json::Value::Number(n.into()),
        }
    }
}

impl From<u32> for LedgerIndex {
    fn from(index: u32) -> Self {
        LedgerIndex::Index(index)
    }
}

impl Serialize for LedgerIndex {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            LedgerIndex::Validated => serializer.serialize_str("validated"),
            LedgerIndex::Current => serializer.serialize_str("current"),
            LedgerIndex::Closed => serializer.serialize_str("closed"),
            LedgerIndex::Index(n) => serializer.serialize_u32(*n),
        }
    }
}

/// A count `rippled` writes as a string in some responses (`fee`'s
/// `"current_queue_size": "28"`, API v1's `ledger.ledger_index`) and as a
/// number in others: read either; write it back as `rippled` did, a string.
pub(crate) mod count_as_string {
    use serde::{Deserialize, Deserializer, Serializer};

    pub(crate) fn serialize<S: Serializer>(v: &Option<u32>, s: S) -> Result<S::Ok, S::Error> {
        match v {
            Some(n) => s.serialize_str(&n.to_string()),
            None => s.serialize_none(),
        }
    }

    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<u32>, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Count {
            Number(u32),
            Text(String),
        }
        match Option::<Count>::deserialize(d)? {
            None => Ok(None),
            Some(Count::Number(n)) => Ok(Some(n)),
            Some(Count::Text(s)) => s.parse().map(Some).map_err(serde::de::Error::custom),
        }
    }
}
