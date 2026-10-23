//! The XRP Ledger's binary format. [`encode_transaction_json`] and
//! [`decode_transaction_binary`] convert between `rippled`'s JSON and its
//! canonical binary form, for transactions and ledger entries alike. Field
//! and type definitions come from `rippled` 3.4.1's `server_definitions`
//! ([`definitions::DEFINITIONS_JSON`]); a field they do not hold is an
//! error, as in `rippled`.

/// Amount encoding.
pub mod amount;
/// Binary → JSON.
pub mod decode;
/// Field and type definitions, from `rippled` 3.4.1's `server_definitions`.
pub mod definitions;
/// JSON → binary.
pub mod encode;
/// Field IDs and length prefixes.
pub mod field;
mod number;

pub use decode::decode_transaction_binary;
pub use definitions::{lookup_field_def, lookup_field_def_by_id, transaction_type_code};
pub use encode::{encode_for_multisigning, encode_for_signing, encode_transaction_json};
pub use field::{decode_vl, encode_vl, FieldId};
