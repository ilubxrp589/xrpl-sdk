//! Typed ledger entries, as rippled's JSON spells them (`ledger_entry`,
//! `account_objects`, `ledger_data`). One struct per `LedgerEntryType`,
//! generated from rippled 3.4.1's `LEDGER_ENTRY_FORMATS` (see `formats.rs`).

mod formats;

pub use formats::*;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

macro_rules! ledger_object_enum {
    ($($name:ident),*) => {
        /// A ledger entry, typed by its `LedgerEntryType`.
        #[allow(clippy::upper_case_acronyms, clippy::large_enum_variant)]
        #[derive(Debug, Clone, PartialEq)]
        pub enum LedgerObject {
            $(
                #[doc = concat!("`", stringify!($name), "`")]
                $name($name),
            )*
            /// A `LedgerEntryType` this crate does not know, or (when deserialized as
            /// part of a response) an entry that does not fit its type: as rippled sent it.
            Other(Value),
        }

        impl LedgerObject {
            /// The `LedgerEntryType` name.
            pub fn entry_type(&self) -> &str {
                match self {
                    $(LedgerObject::$name(_) => stringify!($name),)*
                    LedgerObject::Other(v) => v
                        .get("LedgerEntryType")
                        .and_then(Value::as_str)
                        .unwrap_or_default(),
                }
            }

            /// Parse rippled's JSON for one ledger entry. A known type whose
            /// fields do not fit is an error; an unknown type is `Other`.
            pub fn from_value(v: Value) -> Result<Self, serde_json::Error> {
                let ty = v.get("LedgerEntryType").and_then(Value::as_str).map(str::to_owned);
                match ty.as_deref() {
                    $(Some(stringify!($name)) => Ok(LedgerObject::$name(serde_json::from_value(v)?)),)*
                    _ => Ok(LedgerObject::Other(v)),
                }
            }

            /// Back to rippled's JSON, `LedgerEntryType` included.
            pub fn to_value(&self) -> Result<Value, serde_json::Error> {
                let mut v = match self {
                    $(LedgerObject::$name(o) => serde_json::to_value(o)?,)*
                    LedgerObject::Other(v) => return Ok(v.clone()),
                };
                if let Some(obj) = v.as_object_mut() {
                    obj.insert("LedgerEntryType".into(), Value::String(self.entry_type().into()));
                }
                Ok(v)
            }
        }
    };
}
each_entry_type!(ledger_object_enum);

/// Lenient: an entry that does not fit its type becomes `Other` with the raw
/// JSON, so one odd entry never fails a whole response. `from_value` is strict.
impl<'de> Deserialize<'de> for LedgerObject {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let v = Value::deserialize(deserializer)?;
        Ok(LedgerObject::from_value(v.clone()).unwrap_or(LedgerObject::Other(v)))
    }
}

impl Serialize for LedgerObject {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_value()
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}
