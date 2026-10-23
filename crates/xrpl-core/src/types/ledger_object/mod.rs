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
        #[non_exhaustive]
        pub enum LedgerObject {
            $(
                #[doc = concat!("`", stringify!($name), "`")]
                $name($name),
            )*
            /// A `LedgerEntryType` this crate does not know: as rippled sent it.
            Other(Value),
            /// An entry of a type this crate knows that did not parse as that
            /// type (a field a newer `rippled` added, or a value of the wrong
            /// shape). Only lenient parsing (inside a response) makes one;
            /// [`LedgerObject::from_value`] returns the error instead.
            Unparsed {
                /// The `LedgerEntryType`.
                entry_type: String,
                /// Why it did not parse.
                error: String,
                /// The entry, as rippled sent it.
                value: Value,
            },
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
                    LedgerObject::Unparsed { entry_type, .. } => entry_type,
                }
            }

            /// Parse rippled's JSON for one ledger entry. Strict: a known type
            /// with a field its struct does not have (which would be lost on
            /// the way back), or a field of the wrong shape, is an error; an
            /// unknown type is `Other`.
            ///
            /// # Errors
            ///
            /// A [`serde_json::Error`] for an entry of a known type with a
            /// field its struct does not have, a field of the wrong shape, or a
            /// required field missing.
            pub fn from_value(v: Value) -> Result<Self, serde_json::Error> {
                let ty = v.get("LedgerEntryType").and_then(Value::as_str).map(str::to_owned);
                let entry = match ty.as_deref() {
                    $(Some(stringify!($name)) => LedgerObject::$name(serde_json::from_value(v.clone())?),)*
                    _ => return Ok(LedgerObject::Other(v)),
                };
                let dropped = crate::transaction::models::dropped_paths(&v, &entry.to_value()?);
                if dropped.is_empty() {
                    Ok(entry)
                } else {
                    Err(<serde_json::Error as serde::de::Error>::custom(format!(
                        "unknown field in {}: {}",
                        entry.entry_type(),
                        dropped.join(", ")
                    )))
                }
            }

            /// Back to rippled's JSON, `LedgerEntryType` included.
            ///
            /// # Errors
            ///
            /// A [`serde_json::Error`] if a field of a typed entry does not
            /// serialize (for example a hand-built `Currency::Standard` code
            /// that is not UTF-8); `Other` and `Unparsed` never fail.
            pub fn to_value(&self) -> Result<Value, serde_json::Error> {
                let mut v = match self {
                    $(LedgerObject::$name(o) => serde_json::to_value(o)?,)*
                    LedgerObject::Other(v) | LedgerObject::Unparsed { value: v, .. } => {
                        return Ok(v.clone())
                    }
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

/// Lenient: an entry that does not fit its type becomes
/// [`LedgerObject::Unparsed`] (carrying the error and the raw JSON), so one
/// odd entry never fails a whole response and is never mistaken for an entry
/// of an unknown type. `from_value` is strict.
impl<'de> Deserialize<'de> for LedgerObject {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let v = Value::deserialize(deserializer)?;
        Ok(
            LedgerObject::from_value(v.clone()).unwrap_or_else(|e| LedgerObject::Unparsed {
                entry_type: v
                    .get("LedgerEntryType")
                    .and_then(Value::as_str)
                    .unwrap_or("?")
                    .to_string(),
                error: e.to_string(),
                value: v,
            }),
        )
    }
}

impl Serialize for LedgerObject {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_value()
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}
