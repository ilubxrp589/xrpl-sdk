use crate::codec::amount;
use crate::codec::definitions::{granular_permission_name, lookup_field_def_by_id};
use crate::codec::field::{decode_vl, FieldId, TypeCode};
use crate::types::{AccountId, Currency};
use crate::CoreError;
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::sync::LazyLock;

/// Parse the embedded definitions.json.
fn parse_definitions() -> Value {
    serde_json::from_str(DEFINITIONS_JSON).unwrap_or_default()
}

/// Helper: get a sub-slice or return a codec error.
fn get_slice<'a>(
    buf: &'a [u8],
    range: std::ops::Range<usize>,
    msg: &str,
) -> Result<&'a [u8], CoreError> {
    buf.get(range)
        .ok_or_else(|| CoreError::CodecError(msg.to_string()))
}

/// Helper: get a sub-slice from start or return a codec error.
fn get_slice_from<'a>(buf: &'a [u8], start: usize, msg: &str) -> Result<&'a [u8], CoreError> {
    buf.get(start..)
        .ok_or_else(|| CoreError::CodecError(msg.to_string()))
}

/// Helper: copy exactly 20 bytes from a slice into a fixed array.
fn copy_20(src: &[u8]) -> Result<[u8; 20], CoreError> {
    <[u8; 20]>::try_from(src).map_err(|_| CoreError::CodecError("expected 20 bytes".to_string()))
}

/// Reverse map: transaction type code → name.
static TRANSACTION_TYPE_NAMES: LazyLock<HashMap<u16, String>> = LazyLock::new(|| {
    let defs = parse_definitions();
    let Some(tt) = defs.get("TRANSACTION_TYPES").and_then(|v| v.as_object()) else {
        return HashMap::new();
    };
    tt.iter()
        .filter_map(|(name, val)| {
            let code = val.as_i64()?;
            if code < 0 {
                return None;
            }
            Some((u16::try_from(code).ok()?, name.clone()))
        })
        .collect()
});

/// The low 16 bits, as rippled's `static_cast<std::uint16_t>` takes them.
#[allow(
    clippy::cast_possible_truncation,
    reason = "rippled truncates to 16 bits here; so must the decoder"
)]
fn low_16_bits(v: u32) -> u16 {
    v as u16
}

/// Reverse map: ledger entry type code → name.
static LEDGER_ENTRY_TYPE_NAMES: LazyLock<HashMap<u16, String>> = LazyLock::new(|| {
    let defs = parse_definitions();
    let Some(tt) = defs.get("LEDGER_ENTRY_TYPES").and_then(|v| v.as_object()) else {
        return HashMap::new();
    };
    tt.iter()
        .filter_map(|(name, val)| {
            let code = val.as_i64()?;
            if code < 0 {
                return None;
            }
            Some((u16::try_from(code).ok()?, name.clone()))
        })
        .collect()
});

use super::definitions::{DEFINITIONS_JSON, UINT64_BASE_TEN};

/// Look up transaction type name by numeric code.
fn transaction_type_name(code: u16) -> Option<&'static str> {
    TRANSACTION_TYPE_NAMES.get(&code).map(|s| s.as_str())
}

/// Look up ledger entry type name by numeric code.
fn ledger_entry_type_name(code: u16) -> Option<&'static str> {
    LEDGER_ENTRY_TYPE_NAMES.get(&code).map(|s| s.as_str())
}

/// Decode a binary-encoded XRPL transaction (or ledger object) back to JSON.
///
/// This is the inverse of `encode_transaction_json`. It reads the canonical
/// binary format and reconstructs the JSON representation, converting numeric
/// type codes back to string names where appropriate.
/// Maximum recursion depth for nested STObject/STArray decoding.
/// Prevents stack overflow from malicious or malformed deeply-nested input.
const MAX_DECODE_DEPTH: usize = 16;

/// Decode a serialized transaction or ledger entry into `rippled`'s JSON.
///
/// ```
/// use xrpl_core::codec::{decode_transaction_binary, encode_transaction_json};
///
/// let entry = serde_json::json!({"LedgerEntryType": "AccountRoot",
///     "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", "Balance": "148446663", "Flags": 0,
///     "OwnerCount": 0, "Sequence": 336, "PreviousTxnLgrSeq": 14091160,
///     "PreviousTxnID": "0D5FB50FA65C9FE1538FD7E398FFFE9D1908DFA4576D8D7A020040686F93C77D"});
/// let bytes = encode_transaction_json(&entry)?;
/// assert_eq!(decode_transaction_binary(&bytes)?, entry);
/// // A truncated entry is refused.
/// assert!(decode_transaction_binary(&bytes[..bytes.len() - 1]).is_err());
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
///
/// # Errors
///
/// [`CoreError::UnknownField`] for a field code the definitions do not
/// hold; [`CoreError::InvalidAmount`] for an amount not in `rippled`'s
/// canonical form; [`CoreError::CodecError`] for any other bytes `rippled`
/// would refuse: a truncated or non-canonical field, nesting deeper than 16
/// levels, or bytes left over after the object.
pub fn decode_transaction_binary(bytes: &[u8]) -> Result<Value, CoreError> {
    let (obj, consumed) = decode_fields(bytes, false, 0)?;
    if consumed != bytes.len() {
        return Err(CoreError::CodecError(format!(
            "decoded {} bytes but input has {} bytes",
            consumed,
            bytes.len()
        )));
    }
    Ok(Value::Object(obj))
}

/// Decode fields from the buffer. If `is_inner` is true, stop at the STObject
/// end marker (0xE1) instead of consuming the entire buffer.
/// Returns the decoded JSON object and the number of bytes consumed.
fn decode_fields(
    bytes: &[u8],
    is_inner: bool,
    depth: usize,
) -> Result<(Map<String, Value>, usize), CoreError> {
    if depth > MAX_DECODE_DEPTH {
        return Err(CoreError::CodecError(
            "decode depth limit exceeded".to_string(),
        ));
    }
    let mut obj = Map::new();
    let mut pos = 0;

    while pos < bytes.len() {
        // Decode field header
        let remaining = get_slice_from(bytes, pos, "buffer underflow in field header")?;
        let (field_id, header_len) = FieldId::decode(remaining)?;
        pos += header_len;

        // The end markers, as rippled's STObject::set treats them.
        match (field_id.type_code, field_id.field_code) {
            (14, 1) if is_inner || pos == bytes.len() => return Ok((obj, pos)),
            (14, 1) => {
                // rippled stops reading here and ignores the rest, which
                // would silently drop fields; refuse instead.
                return Err(CoreError::CodecError(
                    "bytes follow the transaction's end-of-object marker".to_string(),
                ));
            }
            (15, 1) => {
                return Err(CoreError::CodecError(
                    "an end-of-array marker inside an object; rippled refuses it".to_string(),
                ))
            }
            _ => {}
        }

        // Look up field definition
        let field_def = lookup_field_def_by_id(field_id.type_code, field_id.field_code);
        let field_name = match &field_def {
            Some(fd) => fd.name.clone(),
            None => {
                // rippled refuses a field it does not know (the SField lookup
                // throws). Skipping one would drop data from the JSON, and its
                // length cannot be known in general (objects, arrays, PathSet).
                return Err(CoreError::UnknownField(format!(
                    "type {} nth {}",
                    field_id.type_code, field_id.field_code
                )));
            }
        };

        let tc = TypeCode::from_u16(field_id.type_code);

        // Decode value based on type code
        let remaining = get_slice_from(bytes, pos, "buffer underflow in field value")?;
        let (value, value_len) = decode_field_value(tc, remaining, &field_name, depth)?;
        pos += value_len;

        if obj.contains_key(&field_name) {
            return Err(CoreError::CodecError(format!(
                "{field_name} appears twice; rippled refuses duplicate fields"
            )));
        }
        obj.insert(field_name, value);
    }

    if is_inner {
        return Err(CoreError::CodecError(
            "unexpected end of buffer while decoding inner object (no 0xE1 marker)".to_string(),
        ));
    }

    Ok((obj, pos))
}

/// Decode a single field value based on its type code.
/// Returns (decoded_value, bytes_consumed).
fn decode_field_value(
    tc: Option<TypeCode>,
    buf: &[u8],
    field_name: &str,
    depth: usize,
) -> Result<(Value, usize), CoreError> {
    match tc {
        Some(TypeCode::UInt8) => decode_uint8(buf),
        Some(TypeCode::UInt16) => decode_uint16(buf, field_name),
        Some(TypeCode::UInt32) => decode_uint32(buf, field_name),
        Some(TypeCode::UInt64) => decode_uint64(buf, field_name),
        Some(TypeCode::Hash128) => decode_hash(buf, 16),
        Some(TypeCode::Hash160) => decode_hash(buf, 20),
        Some(TypeCode::Hash256) => decode_hash(buf, 32),
        Some(TypeCode::Amount) => decode_amount_field(buf),
        Some(TypeCode::Blob) => decode_blob(buf),
        Some(TypeCode::AccountId) => decode_account_id_field(buf),
        Some(TypeCode::Number) => decode_number(buf),
        Some(TypeCode::Int32) => decode_int32(buf),
        Some(TypeCode::StObject) => decode_st_object(buf, depth),
        Some(TypeCode::StArray) => decode_st_array(buf, depth),
        Some(TypeCode::PathSet) => decode_pathset(buf),
        Some(TypeCode::Vector256) => decode_vector256(buf),
        Some(TypeCode::Issue) => decode_issue(buf),
        Some(TypeCode::XChainBridge) => decode_xchain_bridge(buf),
        Some(TypeCode::Currency) => decode_currency_type(buf),
        Some(TypeCode::UInt96) => decode_raw_hex(buf, 12),
        Some(TypeCode::Hash192) => decode_raw_hex(buf, 24),
        Some(TypeCode::UInt384) => decode_raw_hex(buf, 48),
        Some(TypeCode::UInt512) => decode_raw_hex(buf, 64),
        _ => Err(CoreError::CodecError(format!(
            "unsupported type code {:?} for field '{field_name}'",
            tc
        ))),
    }
}

fn decode_uint8(buf: &[u8]) -> Result<(Value, usize), CoreError> {
    let &b = buf
        .first()
        .ok_or_else(|| CoreError::CodecError("buffer too short for UInt8".to_string()))?;
    Ok((Value::Number(b.into()), 1))
}

fn decode_uint16(buf: &[u8], field_name: &str) -> Result<(Value, usize), CoreError> {
    if buf.len() < 2 {
        return Err(CoreError::CodecError(
            "buffer too short for UInt16".to_string(),
        ));
    }
    let b0 = *buf.first().unwrap_or(&0);
    let b1 = *buf.get(1).unwrap_or(&0);
    let val = u16::from_be_bytes([b0, b1]);

    // Convert TransactionType and LedgerEntryType codes back to string names
    match field_name {
        "TransactionType" => {
            if let Some(name) = transaction_type_name(val) {
                return Ok((Value::String(name.to_string()), 2));
            }
            // Fall through to numeric if unknown
        }
        "LedgerEntryType" => {
            if let Some(name) = ledger_entry_type_name(val) {
                return Ok((Value::String(name.to_string()), 2));
            }
        }
        _ => {}
    }

    Ok((Value::Number(val.into()), 2))
}

fn decode_uint32(buf: &[u8], field_name: &str) -> Result<(Value, usize), CoreError> {
    let bytes = buf
        .get(..4)
        .ok_or_else(|| CoreError::CodecError("buffer too short for UInt32".to_string()))?;
    let arr: [u8; 4] = bytes
        .try_into()
        .map_err(|_| CoreError::CodecError("buffer too short for UInt32".to_string()))?;
    let val = u32::from_be_bytes(arr);
    // rippled names a PermissionValue (Permissions.cpp getPermissionName): a
    // granular permission's name, else the name of TxType (value - 1), cast to
    // the 16-bit TxType as rippled's static_cast does.
    if field_name == "PermissionValue" {
        let name = granular_permission_name(val)
            .or_else(|| transaction_type_name(low_16_bits(val.wrapping_sub(1))));
        if let Some(name) = name {
            return Ok((Value::String(name.to_string()), 4));
        }
    }
    Ok((Value::Number(val.into()), 4))
}

/// Int32: big-endian two's complement; rippled's JSON is the number.
fn decode_int32(buf: &[u8]) -> Result<(Value, usize), CoreError> {
    let arr: [u8; 4] = buf
        .get(..4)
        .and_then(|b| b.try_into().ok())
        .ok_or_else(|| CoreError::CodecError("buffer too short for Int32".to_string()))?;
    Ok((Value::Number(i32::from_be_bytes(arr).into()), 4))
}

/// `decode_uint64` for the encoder's round-trip test.
#[cfg(test)]
#[allow(clippy::unwrap_used)]
pub(crate) fn tests_decode_uint64(buf: &[u8], field_name: &str) -> (Value, usize) {
    decode_uint64(buf, field_name).unwrap()
}

fn decode_uint64(buf: &[u8], field_name: &str) -> Result<(Value, usize), CoreError> {
    let bytes = buf
        .get(..8)
        .ok_or_else(|| CoreError::CodecError("buffer too short for UInt64".to_string()))?;
    // kSmdBaseTen UInt64 fields render as base-10 strings in rippled JSON
    // (sfields.macro); every other UInt64 renders as padded hex.
    if UINT64_BASE_TEN.contains(&field_name) {
        let arr: [u8; 8] = bytes
            .try_into()
            .map_err(|_| CoreError::CodecError("buffer too short for UInt64".to_string()))?;
        return Ok((Value::String(u64::from_be_bytes(arr).to_string()), 8));
    }
    let hex_str = hex::encode_upper(bytes);
    Ok((Value::String(hex_str), 8))
}

fn decode_hash(buf: &[u8], len: usize) -> Result<(Value, usize), CoreError> {
    let bytes = buf.get(..len).ok_or_else(|| {
        CoreError::CodecError(format!(
            "buffer too short for Hash{}: need {len}, have {}",
            len * 8,
            buf.len()
        ))
    })?;
    let hex_str = hex::encode_upper(bytes);
    Ok((Value::String(hex_str), len))
}

fn decode_amount_field(buf: &[u8]) -> Result<(Value, usize), CoreError> {
    let &first_byte = buf
        .first()
        .ok_or_else(|| CoreError::CodecError("buffer too short for Amount".to_string()))?;

    if first_byte & 0x80 == 0 && first_byte & 0x20 != 0 {
        // MPT amount (STAmount::add): lead byte (0x20 marker, 0x40 positive)
        // + 8-byte value + 192-bit MPTokenIssuanceID = 33 bytes.
        let (value_bytes, issuance_id) = buf
            .get(1..33)
            .and_then(|b| b.split_first_chunk::<8>())
            .ok_or_else(|| CoreError::CodecError("buffer too short for MPT amount".to_string()))?;
        let value = u64::from_be_bytes(*value_bytes);
        // rippled reads a larger magnitude but cannot write it as JSON
        // (maxMPTokenAmount is i64::MAX), so no ledger holds one.
        if i64::try_from(value).is_err() {
            return Err(CoreError::InvalidAmount(format!(
                "MPT amount {value} exceeds i64::MAX"
            )));
        }
        let sign = if first_byte & 0x40 == 0 && value != 0 {
            "-"
        } else {
            ""
        };
        let obj = json!({
            "mpt_issuance_id": hex::encode_upper(issuance_id),
            "value": format!("{sign}{value}"),
        });
        return Ok((obj, 33));
    }

    if first_byte & 0x80 == 0 {
        // XRP amount: 8 bytes
        let bytes = buf
            .get(..8)
            .ok_or_else(|| CoreError::CodecError("buffer too short for XRP amount".to_string()))?;
        let arr: [u8; 8] = bytes
            .try_into()
            .map_err(|_| CoreError::CodecError("buffer too short for XRP amount".to_string()))?;
        let (negative, drops) = amount::decode_xrp_signed(&arr)?;
        let sign = if negative { "-" } else { "" };
        Ok((Value::String(format!("{sign}{drops}")), 8))
    } else {
        // IOU amount: 48 bytes
        let bytes = buf
            .get(..48)
            .ok_or_else(|| CoreError::CodecError("buffer too short for IOU amount".to_string()))?;
        let arr: [u8; 48] = bytes
            .try_into()
            .map_err(|_| CoreError::CodecError("buffer too short for IOU amount".to_string()))?;
        let (iou, currency, issuer) = amount::decode_amount_iou(&arr)?;

        let currency_str = currency.to_string();
        let issuer_str = issuer.to_address();
        let value_str = iou.to_decimal();

        let obj = json!({
            "value": value_str,
            "currency": currency_str,
            "issuer": issuer_str
        });
        Ok((obj, 48))
    }
}

fn decode_blob(buf: &[u8]) -> Result<(Value, usize), CoreError> {
    let (len, vl_consumed) = decode_vl(buf)?;
    let total = vl_consumed + len;
    let data = get_slice(
        buf,
        vl_consumed..total,
        &format!(
            "buffer too short for Blob: need {total}, have {}",
            buf.len()
        ),
    )?;
    let hex_str = hex::encode_upper(data);
    Ok((Value::String(hex_str), total))
}

fn decode_account_id_field(buf: &[u8]) -> Result<(Value, usize), CoreError> {
    let (len, vl_consumed) = decode_vl(buf)?;
    // An account field never set is an empty VL (rippled's default STAccount: a UNLModify
    // pseudo-transaction's Account), which rippled's JSON shows as "".
    if len == 0 {
        return Ok((Value::String(String::new()), vl_consumed));
    }
    if len != 20 {
        return Err(CoreError::CodecError(format!(
            "AccountID VL length must be 20, got {len}"
        )));
    }
    let total = vl_consumed + 20;
    let bytes = get_slice(buf, vl_consumed..total, "buffer too short for AccountID")?;
    let arr = copy_20(bytes)?;
    let account = AccountId::from_bytes(arr);
    Ok((Value::String(account.to_address()), total))
}

fn decode_st_object(buf: &[u8], depth: usize) -> Result<(Value, usize), CoreError> {
    let (obj, consumed) = decode_fields(buf, true, depth + 1)?;
    Ok((Value::Object(obj), consumed))
}

fn decode_st_array(buf: &[u8], depth: usize) -> Result<(Value, usize), CoreError> {
    let mut arr = Vec::new();
    let mut pos = 0;

    while pos < buf.len() {
        // Each array element is a wrapped object: field header + inner object fields + 0xE1
        // The field header names the wrapper (e.g., "Memo", "SignerEntry")
        let remaining = get_slice_from(buf, pos, "buffer underflow in STArray element")?;
        let (field_id, header_len) = FieldId::decode(remaining)?;
        pos += header_len;

        // As rippled's STArray reads them: the array's end, a misplaced
        // object end, and elements that are not objects.
        match (field_id.type_code, field_id.field_code) {
            (15, 1) => return Ok((Value::Array(arr), pos)),
            (14, 1) => {
                return Err(CoreError::CodecError(
                    "an end-of-object marker inside an array; rippled refuses it".to_string(),
                ))
            }
            _ => {}
        }
        if TypeCode::from_u16(field_id.type_code) != Some(TypeCode::StObject) {
            return Err(CoreError::CodecError(format!(
                "array element of type {} is not an object; rippled refuses it",
                field_id.type_code
            )));
        }

        let field_def = lookup_field_def_by_id(field_id.type_code, field_id.field_code);
        let wrapper_name = match &field_def {
            Some(fd) => fd.name.clone(),
            None => {
                return Err(CoreError::CodecError(format!(
                    "unknown array wrapper field: type_code={}, field_code={}",
                    field_id.type_code, field_id.field_code
                )));
            }
        };

        // Decode the inner object (reads until 0xE1)
        let remaining = get_slice_from(buf, pos, "buffer underflow in STArray inner object")?;
        let (inner_obj, inner_consumed) = decode_fields(remaining, true, depth + 1)?;
        pos += inner_consumed;

        // Wrap: { "Memo": { ... } }
        let mut wrapper = Map::new();
        wrapper.insert(wrapper_name, Value::Object(inner_obj));
        arr.push(Value::Object(wrapper));
    }

    Err(CoreError::CodecError(
        "unexpected end of buffer while decoding STArray (no 0xF1 marker)".to_string(),
    ))
}

/// Decode a PathSet as `rippled` reads one (`STPathSet(SerialIter&)`): paths
/// separated by 0xFF and ended by 0x00, each step a type byte (0x01 account,
/// 0x10 currency, 0x40 MPT, 0x20 issuer) followed by those fields. An empty
/// path and unknown type bits are refused, as `rippled` refuses them. Steps
/// are written without `rippled`'s derived `type` field, as in the XRPL codec
/// fixtures.
fn decode_pathset(buf: &[u8]) -> Result<(Value, usize), CoreError> {
    const ACCOUNT: u8 = 0x01;
    const CURRENCY: u8 = 0x10;
    const ISSUER: u8 = 0x20;
    const MPT: u8 = 0x40;
    let mut paths: Vec<Value> = Vec::new();
    let mut current_path: Vec<Value> = Vec::new();
    let mut pos = 0;

    while let Some(&type_byte) = buf.get(pos) {
        pos += 1;
        if type_byte == 0x00 || type_byte == 0xFF {
            if current_path.is_empty() {
                return Err(CoreError::CodecError(
                    "PathSet has an empty path; rippled refuses it".to_string(),
                ));
            }
            paths.push(Value::Array(std::mem::take(&mut current_path)));
            if type_byte == 0x00 {
                return Ok((Value::Array(paths), pos));
            }
            continue;
        }
        if type_byte & !(ACCOUNT | CURRENCY | ISSUER | MPT) != 0 {
            return Err(CoreError::CodecError(format!(
                "path step type 0x{type_byte:02X} has unknown bits; rippled refuses it"
            )));
        }
        if type_byte & CURRENCY != 0 && type_byte & MPT != 0 {
            return Err(CoreError::CodecError(
                "path step has both a currency and an MPT".to_string(),
            ));
        }
        // rippled reads it, but its JSON refuses an issuer beside an MPT.
        if type_byte & MPT != 0 && type_byte & ISSUER != 0 {
            return Err(CoreError::CodecError(
                "path step has an MPT with an issuer".to_string(),
            ));
        }

        let mut step = Map::new();
        if type_byte & ACCOUNT != 0 {
            let bytes = get_slice(buf, pos..pos + 20, "buffer too short for path step account")?;
            let account = AccountId::from_bytes(copy_20(bytes)?);
            step.insert("account".to_string(), Value::String(account.to_address()));
            pos += 20;
        }
        if type_byte & CURRENCY != 0 {
            let bytes = get_slice(
                buf,
                pos..pos + 20,
                "buffer too short for path step currency",
            )?;
            let currency = Currency::from_bytes(copy_20(bytes)?);
            step.insert("currency".to_string(), Value::String(currency.to_string()));
            pos += 20;
        }
        if type_byte & MPT != 0 {
            let bytes = get_slice(buf, pos..pos + 24, "buffer too short for path step MPT")?;
            step.insert(
                "mpt_issuance_id".to_string(),
                Value::String(hex::encode_upper(bytes)),
            );
            pos += 24;
        }
        if type_byte & ISSUER != 0 {
            let bytes = get_slice(buf, pos..pos + 20, "buffer too short for path step issuer")?;
            let issuer = AccountId::from_bytes(copy_20(bytes)?);
            step.insert("issuer".to_string(), Value::String(issuer.to_address()));
            pos += 20;
        }
        current_path.push(Value::Object(step));
    }

    Err(CoreError::CodecError(
        "unexpected end of buffer while decoding PathSet (no 0x00 marker)".to_string(),
    ))
}

fn decode_vector256(buf: &[u8]) -> Result<(Value, usize), CoreError> {
    let (len, vl_consumed) = decode_vl(buf)?;
    if len % 32 != 0 {
        return Err(CoreError::CodecError(format!(
            "Vector256 length must be a multiple of 32, got {len}"
        )));
    }
    let total = vl_consumed + len;
    if buf.len() < total {
        return Err(CoreError::CodecError(format!(
            "buffer too short for Vector256: need {total}, have {}",
            buf.len()
        )));
    }

    let count = len / 32;
    let mut arr = Vec::with_capacity(count);
    for i in 0..count {
        let start = vl_consumed + i * 32;
        let bytes = get_slice(
            buf,
            start..start + 32,
            "buffer too short for Vector256 entry",
        )?;
        let hex_str = hex::encode_upper(bytes);
        arr.push(Value::String(hex_str));
    }

    Ok((Value::Array(arr), total))
}

fn decode_issue(buf: &[u8]) -> Result<(Value, usize), CoreError> {
    // Issue type can be:
    // - XRP: 20 zero bytes → { "currency": "XRP" }
    // - IOU: 20 bytes currency + 20 bytes issuer → { "currency": "...", "issuer": "r..." }
    // - MPT: 20 bytes issuer + 20 bytes NO_ACCOUNT + 4 bytes sequence_LE = 44 bytes

    if buf.len() < 20 {
        return Err(CoreError::CodecError(
            "buffer too short for Issue type".to_string(),
        ));
    }

    // Check if first 20 bytes are all zeros (XRP)
    let first20 = buf
        .get(..20)
        .ok_or_else(|| CoreError::CodecError("buffer too short for Issue type".to_string()))?;
    if first20 == [0u8; 20] {
        return Ok((json!({"currency": "XRP"}), 20));
    }

    // Try to detect MPT vs IOU: rippled's STIssue reads an MPT whenever the
    // second 20 bytes are NO_ACCOUNT (and then needs the 4-byte sequence).
    if buf.len() >= 40 {
        // Check for NO_ACCOUNT pattern in bytes 20..40
        let no_account: [u8; 20] = {
            let mut a = [0u8; 20];
            a[19] = 0x01;
            a
        };
        let second20 = buf.get(20..40).unwrap_or_default();
        if second20 == no_account {
            // MPT Issue: issuer(20) + NO_ACCOUNT(20) + sequence_LE(4)
            let issuer_bytes = copy_20(first20)?;
            let issuer = AccountId::from_bytes(issuer_bytes);

            // sequence is LE, convert to BE for the mpt_issuance_id
            let seq_le = buf.get(40..44).ok_or_else(|| {
                CoreError::CodecError("buffer too short for MPT sequence".to_string())
            })?;
            let seq_be = [
                *seq_le.get(3).unwrap_or(&0),
                *seq_le.get(2).unwrap_or(&0),
                *seq_le.get(1).unwrap_or(&0),
                *seq_le.first().unwrap_or(&0),
            ];

            // mpt_issuance_id = sequence_BE(4) + issuer(20) = 24 bytes hex
            let mut mpt_id = Vec::with_capacity(24);
            mpt_id.extend_from_slice(&seq_be);
            mpt_id.extend_from_slice(issuer.as_bytes());
            let mpt_hex = hex::encode_upper(&mpt_id);

            return Ok((json!({"mpt_issuance_id": mpt_hex}), 44));
        }
    }

    // IOU: currency(20) + issuer(20) = 40 bytes
    if buf.len() < 40 {
        return Err(CoreError::CodecError(
            "buffer too short for IOU Issue type".to_string(),
        ));
    }

    let currency_slice = buf
        .get(..20)
        .ok_or_else(|| CoreError::CodecError("buffer too short for currency".to_string()))?;
    let currency_bytes = copy_20(currency_slice)?;
    let currency = Currency::from_bytes(currency_bytes);

    let issuer_slice = buf
        .get(20..40)
        .ok_or_else(|| CoreError::CodecError("buffer too short for issuer".to_string()))?;
    let issuer_bytes = copy_20(issuer_slice)?;
    if issuer_bytes == [0u8; 20] {
        // rippled: "invalid issue: currency and account native mismatch".
        return Err(CoreError::CodecError(
            "a token Issue cannot have XRP's issuer (all zeros)".to_string(),
        ));
    }
    let issuer = AccountId::from_bytes(issuer_bytes);

    Ok((
        json!({
            "currency": currency.to_string(),
            "issuer": issuer.to_address()
        }),
        40,
    ))
}

fn decode_xchain_bridge(buf: &[u8]) -> Result<(Value, usize), CoreError> {
    let mut pos = 0;

    // LockingChainDoor (AccountID, VL-prefixed)
    let remaining = get_slice_from(buf, pos, "buffer underflow in XChainBridge door1 VL")?;
    let (door1_len, door1_vl) = decode_vl(remaining)?;
    pos += door1_vl;
    if door1_len != 20 || pos + 20 > buf.len() {
        return Err(CoreError::CodecError(
            "invalid LockingChainDoor in XChainBridge".to_string(),
        ));
    }
    let door1_slice = get_slice(buf, pos..pos + 20, "buffer underflow for LockingChainDoor")?;
    let door1 = AccountId::from_bytes(copy_20(door1_slice)?);
    pos += 20;

    // LockingChainIssue (Issue type)
    let remaining = get_slice_from(buf, pos, "buffer underflow in XChainBridge issue1")?;
    let (issue1, issue1_len) = decode_issue(remaining)?;
    pos += issue1_len;

    // IssuingChainDoor (AccountID, VL-prefixed)
    let remaining = get_slice_from(buf, pos, "buffer underflow in XChainBridge door2 VL")?;
    let (door2_len, door2_vl) = decode_vl(remaining)?;
    pos += door2_vl;
    if door2_len != 20 || pos + 20 > buf.len() {
        return Err(CoreError::CodecError(
            "invalid IssuingChainDoor in XChainBridge".to_string(),
        ));
    }
    let door2_slice = get_slice(buf, pos..pos + 20, "buffer underflow for IssuingChainDoor")?;
    let door2 = AccountId::from_bytes(copy_20(door2_slice)?);
    pos += 20;

    // IssuingChainIssue (Issue type)
    let remaining = get_slice_from(buf, pos, "buffer underflow in XChainBridge issue2")?;
    let (issue2, issue2_len) = decode_issue(remaining)?;
    pos += issue2_len;

    let obj = json!({
        "LockingChainDoor": door1.to_address(),
        "LockingChainIssue": issue1,
        "IssuingChainDoor": door2.to_address(),
        "IssuingChainIssue": issue2
    });

    Ok((obj, pos))
}

fn decode_currency_type(buf: &[u8]) -> Result<(Value, usize), CoreError> {
    let bytes = buf
        .get(..20)
        .ok_or_else(|| CoreError::CodecError("buffer too short for Currency type".to_string()))?;
    let arr = copy_20(bytes)?;
    let currency = Currency::from_bytes(arr);
    Ok((Value::String(currency.to_string()), 20))
}

/// Decode a Number (type 9, `STNumber`): see [`super::number`].
fn decode_number(buf: &[u8]) -> Result<(Value, usize), CoreError> {
    let bytes: [u8; 12] = buf
        .get(..12)
        .and_then(|b| b.try_into().ok())
        .ok_or_else(|| CoreError::CodecError("buffer too short for Number type".to_string()))?;
    Ok((Value::String(super::number::decode(bytes)?), 12))
}

fn decode_raw_hex(buf: &[u8], len: usize) -> Result<(Value, usize), CoreError> {
    let bytes = buf.get(..len).ok_or_else(|| {
        CoreError::CodecError(format!(
            "buffer too short for raw hex: need {len}, have {}",
            buf.len()
        ))
    })?;
    let hex_str = hex::encode_upper(bytes);
    Ok((Value::String(hex_str), len))
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic
)]
mod tests {
    use super::*;
    use crate::codec::encode_transaction_json;
    use serde_json::json;

    /// Objects and arrays as rippled 3.4.1 reads them (`simulate` of a
    /// Payment with these bytes): whether it read the transaction.
    #[test]
    fn reads_structure_as_rippled_does() {
        let (t, f, s, s2) = ("120000", "2200000000", "2400000001", "2400000002");
        let rest = "614000000000000064\
                    68400000000000000C\
                    7300\
                    8114B5F762798A53D543A014CAF8B297CFF8F2F937E8\
                    8314F667B0CA50CC7709A220B0561B85E53A48461FA8";
        let base = format!("{t}{f}{s}{rest}");
        let (fee_and_before, after_fee) = base.split_at(base.find("7300").unwrap());
        for (case, bytes, read) in [
            ("base", base.clone(), true),
            ("fields out of order", format!("{t}{s}{f}{rest}"), true),
            ("a duplicate field", format!("{t}{f}{s}{s2}{rest}"), false),
            ("end-of-object marker last", format!("{base}E1"), true),
            (
                "end-of-array marker in the object",
                format!("{base}F1"),
                false,
            ),
            ("a Memo", format!("{base}F9EA7D01AAE1F1"), true),
            (
                "a Memo holding a Fee",
                format!("{base}F9EA684000000000000005E1F1"),
                true,
            ),
            (
                "a Memo with a duplicate field",
                format!("{base}F9EA7D01AA7D01BBE1F1"),
                false,
            ),
            (
                "an end-of-object marker as an element",
                format!("{base}F9E1F1"),
                false,
            ),
            (
                "an Amount as an element",
                format!("{base}F961400000000000000005F1"),
                false,
            ),
            // rippled stops at the marker and ignores the rest (then misses
            // Account); the SDK refuses instead of dropping fields.
            (
                "end-of-object marker mid-way",
                format!("{fee_and_before}E1{after_fee}"),
                false,
            ),
        ] {
            let got = decode_transaction_binary(&hex::decode(&bytes).unwrap());
            assert_eq!(got.is_ok(), read, "{case}: {got:?}");
        }
        let with_marker = decode_transaction_binary(&hex::decode(format!("{base}E1")).unwrap());
        let plain = decode_transaction_binary(&hex::decode(&base).unwrap());
        assert_eq!(with_marker.unwrap(), plain.unwrap());
    }

    /// Issues as rippled 3.4.1 reads them (`simulate` of a VaultCreate whose
    /// Asset held these bytes): the Asset it showed, or None where it refused.
    #[test]
    fn reads_issues_as_rippled_does() {
        const ISSUER: &str = "F667B0CA50CC7709A220B0561B85E53A48461FA8";
        const USD: &str = "0000000000000000000000005553440000000000";
        const NO_ACCOUNT: &str = "0000000000000000000000000000000000000001";
        let zeros = "0".repeat(40);
        for (bytes, shown) in [
            (zeros.clone(), Some(json!({"currency": "XRP"}))),
            (
                format!("{USD}{ISSUER}"),
                Some(json!({"currency": "USD", "issuer": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe"})),
            ),
            (format!("{USD}{zeros}"), None),
            (
                format!("{ISSUER}{NO_ACCOUNT}00000005"),
                Some(json!({"mpt_issuance_id": format!("05000000{ISSUER}")})),
            ),
            (format!("{ISSUER}{NO_ACCOUNT}"), None),
            (
                format!("{USD}{NO_ACCOUNT}00000005"),
                Some(json!({"mpt_issuance_id": format!("05000000{USD}")})),
            ),
        ] {
            let got = decode_issue(&hex::decode(&bytes).unwrap())
                .ok()
                .map(|(v, _)| v);
            assert_eq!(got, shown, "{bytes}");
        }
    }

    /// PathSets as rippled 3.4.1 reads them (`simulate` of a Payment whose
    /// Paths held these bytes): the paths it showed (less its derived `type`
    /// field), or None where it refused the blob.
    #[test]
    fn reads_paths_as_rippled_does() {
        const ACCT: &str = "F667B0CA50CC7709A220B0561B85E53A48461FA8";
        const ADDR: &str = "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe";
        const USD: &str = "0000000000000000000000005553440000000000";
        const MPT: &str = "000000000000555344000000007D0748616C63796F6EE1F1";
        let xrp = "0".repeat(40);
        for (bytes, shown) in [
            (format!("01{ACCT}00"), Some(json!([[{"account": ADDR}]]))),
            (
                format!("40{MPT}00"),
                Some(json!([[{"mpt_issuance_id": MPT}]])),
            ),
            (format!("10{xrp}00"), Some(json!([[{"currency": "XRP"}]]))),
            (
                format!("30{USD}{ACCT}00"),
                Some(json!([[{"currency": "USD", "issuer": ADDR}]])),
            ),
            (format!("FF01{ACCT}00"), None), // empty first path
            (format!("01{ACCT}FF00"), None), // empty last path
            ("00".to_string(), None),        // no path at all
            (format!("01{ACCT}FFFF01{ACCT}00"), None),
            (format!("02{ACCT}00"), None), // unknown type bit
            ("0200".to_string(), None),
            (format!("50{USD}{MPT}00"), None), // currency and MPT
            ("8000".to_string(), None),
            // rippled reads this one, but its JSON refuses an issuer beside an
            // MPT, so the SDK refuses it outright.
            (format!("60{MPT}{ACCT}00"), None),
        ] {
            let got = decode_pathset(&hex::decode(&bytes).unwrap())
                .ok()
                .map(|(v, _)| v);
            assert_eq!(got, shown, "{bytes}");
        }
    }

    /// MPT amounts as rippled 3.4.1 reads them (`simulate` of a Payment whose
    /// Amount held these 9 bytes before the issuance ID): the value it
    /// showed, or None where it could not.
    #[test]
    fn reads_mpt_amounts_as_rippled_does() {
        const ID: &str = "000000000000555344000000007D0748616C63796F6EE1F1";
        for (lead_and_value, shown) in [
            ("600000000000000064", Some("100")),
            ("200000000000000064", Some("-100")),
            ("607FFFFFFFFFFFFFFF", Some("9223372036854775807")),
            ("200000000000000000", Some("0")),
            ("600000000000000000", Some("0")),
            ("7F0000000000000005", Some("5")), // other lead-byte bits are ignored
            ("610000000000000005", Some("5")),
            ("608000000000000000", None),
            ("60856B3DF729C00000", None), // what the fuzzer found
        ] {
            let bytes = hex::decode(format!("{lead_and_value}{ID}")).unwrap();
            let got = decode_amount_field(&bytes).ok().map(|(v, n)| {
                assert_eq!(n, 33);
                assert_eq!(v["mpt_issuance_id"], ID);
                v["value"].as_str().unwrap().to_string()
            });
            assert_eq!(got.as_deref(), shown, "{lead_and_value}");
        }
    }

    #[test]
    fn decodes_a_mainnet_delegate_set_as_rippled_does() {
        // 4A242D3BBE512789… in ledger #107804444: rippled 3.4.1's `tx` (binary)
        // and its JSON, which names each PermissionValue (TxType + 1).
        let blob = "12004024066B3B57201B066CF72D68400000000000000C7321ED35D4A0AB1968E2C9B104D243574E7867A58F23B0D3B4F5491DEE3094FD740C9E7440C80B1A0C044BF8BC45DB69CC2B109AFF2234CB0711BA41308BF1AE24D9A7D54A97ABBBB2F01F67DAF62B96FDF203484646BCE52C984C8BD5729B137CC01BB20B811476D532936E2AC465EDB25E40566165EB170C8B6185147D1CBA3091C51F2B0A33C9298C32229DB887AD1BF01DEF20340000001CE1EF20340000001DE1F1";
        let rippled = json!({
            "Account": "rBqL31rm5w2kW8VQFF6WXXQaG8avSym1ij",
            "Authorize": "rUQX1zNmDTKyaM1tCbY84J7eumKXtN9b6F",
            "Fee": "12",
            "LastLedgerSequence": 107804461,
            "Permissions": [
                {"Permission": {"PermissionValue": "NFTokenCreateOffer"}},
                {"Permission": {"PermissionValue": "NFTokenCancelOffer"}}
            ],
            "Sequence": 107690839,
            "SigningPubKey": "ED35D4A0AB1968E2C9B104D243574E7867A58F23B0D3B4F5491DEE3094FD740C9E",
            "TransactionType": "DelegateSet",
            "TxnSignature": "C80B1A0C044BF8BC45DB69CC2B109AFF2234CB0711BA41308BF1AE24D9A7D54A97ABBBB2F01F67DAF62B96FDF203484646BCE52C984C8BD5729B137CC01BB20B"
        });
        let bytes = hex::decode(blob).unwrap();
        assert_eq!(decode_transaction_binary(&bytes).unwrap(), rippled);
        assert_eq!(encode_transaction_json(&rippled).unwrap(), bytes);
    }

    #[test]
    fn permission_value_names_follow_rippled() {
        // Permissions.cpp getPermissionName: a granular permission's name, else
        // the name of TxType (value - 1), else the number.
        let tx = json!({"Permissions": [
            {"Permission": {"PermissionValue": 1}},
            {"Permission": {"PermissionValue": 65_537}},
            {"Permission": {"PermissionValue": 65_548}},
            {"Permission": {"PermissionValue": 70_000}}
        ]});
        let back = decode_transaction_binary(&encode_transaction_json(&tx).unwrap()).unwrap();
        let names: Vec<&Value> = (0..4)
            .map(|i| &back["Permissions"][i]["Permission"]["PermissionValue"])
            .collect();
        assert_eq!(
            names,
            [
                &json!("Payment"),
                &json!("TrustlineAuthorize"),
                &json!("MPTokenIssuanceUnlock"),
                &json!(70_000)
            ]
        );
    }

    #[test]
    fn int32_is_signed_big_endian() {
        // STInteger<int32_t>: add32 (big-endian two's complement); JSON is the
        // number, and rippled also accepts a decimal string (STParsedJSON).
        let bytes = encode_transaction_json(&json!({"LoanScale": -5})).unwrap();
        assert_eq!(bytes, [0xA1, 0xFF, 0xFF, 0xFF, 0xFB]);
        assert_eq!(
            decode_transaction_binary(&bytes).unwrap(),
            json!({"LoanScale": -5})
        );
        assert_eq!(
            encode_transaction_json(&json!({"LoanScale": "-5"})).unwrap(),
            bytes
        );
        assert!(encode_transaction_json(&json!({"LoanScale": 2_147_483_648_i64})).is_err());
    }

    #[test]
    fn x_addresses_become_classic_addresses_and_tags() {
        // As xrpl.js's codec does: Account's tag → SourceTag, Destination's → DestinationTag.
        let classic = json!({
            "TransactionType": "Payment",
            "Account": "rGWrZyQqhTp9Xu7G5Pkayo7bXjH4k4QYpf",
            "SourceTag": 1,
            "Destination": "r9cZA1mLK5R5Am25ArfXFmqgNwjZgnfk59",
            "DestinationTag": 11_747,
            "Amount": "1000000"
        });
        let x = json!({
            "TransactionType": "Payment",
            "Account": "XVLhHMPHU98es4dbozjVtdWzVrDjtV8xvjGQTYPiAx6gwDC",
            "Destination": "X7AcgcsBL6XDcUb289X4mJ8djcdyKaLFuhLRuNXPrDeJd9A",
            "Amount": "1000000"
        });
        let want = encode_transaction_json(&classic).unwrap();
        assert_eq!(encode_transaction_json(&x).unwrap(), want);
        // An untagged X-address is just the classic address.
        let untagged = json!({"Account": "XVLhHMPHU98es4dbozjVtdWzVrDjtV5fdx1mHp98tDMoQXb"});
        assert_eq!(
            encode_transaction_json(&untagged).unwrap(),
            encode_transaction_json(&json!({"Account": "rGWrZyQqhTp9Xu7G5Pkayo7bXjH4k4QYpf"}))
                .unwrap()
        );
        // A tag twice, or a tag on a field that has no tag field, is an error.
        let twice =
            json!({"Account": "XVLhHMPHU98es4dbozjVtdWzVrDjtV8xvjGQTYPiAx6gwDC", "SourceTag": 1});
        assert!(encode_transaction_json(&twice).is_err());
        let elsewhere = json!({"Authorize": "XVLhHMPHU98es4dbozjVtdWzVrDjtV8xvjGQTYPiAx6gwDC"});
        assert!(encode_transaction_json(&elsewhere).is_err());
    }

    #[test]
    fn an_unknown_field_is_an_error_not_skipped() {
        // STObject nth 14 was Hooks' "Hook"; rippled 3.4.1 defines no such field.
        let err = decode_transaction_binary(&[0xEE, 0xE1]).unwrap_err();
        assert!(matches!(err, CoreError::UnknownField(_)), "{err}");
    }

    #[test]
    fn decode_simple_payment() {
        // A known simple Payment transaction
        let tx_json = serde_json::json!({
            "TransactionType": "Payment",
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
            "Amount": "1000000",
            "Fee": "12",
            "Sequence": 1,
            "Flags": 0,
            "SigningPubKey": ""
        });

        // Encode to binary
        let encoded = encode_transaction_json(&tx_json).expect("encode failed");

        // Decode back to JSON
        let decoded = decode_transaction_binary(&encoded).expect("decode failed");

        // Verify key fields
        assert_eq!(decoded["TransactionType"], "Payment");
        assert_eq!(decoded["Account"], "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
        assert_eq!(decoded["Destination"], "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe");
        assert_eq!(decoded["Amount"], "1000000");
        assert_eq!(decoded["Fee"], "12");
        assert_eq!(decoded["Sequence"], 1);
        assert_eq!(decoded["Flags"], 0);
    }

    #[test]
    fn roundtrip_transaction_fixtures() {
        let fixtures: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../tests/vectors/codec-fixtures.json"
        ))
        .expect("invalid fixtures");

        let transactions = fixtures["transactions"].as_array().unwrap();
        let mut passed = 0;

        for (i, entry) in transactions.iter().enumerate() {
            let expected_hex = entry["binary"].as_str().unwrap();
            let expected_json = &entry["json"];
            let tx_type = expected_json["TransactionType"].as_str().unwrap_or("?");

            // Decode binary -> JSON
            let binary = hex::decode(expected_hex).unwrap();
            let decoded = match decode_transaction_binary(&binary) {
                Ok(v) => v,
                Err(e) => {
                    panic!("decode failed for tx[{i}] {tx_type}: {e}");
                }
            };

            // Re-encode decoded JSON -> binary
            let re_encoded = match encode_transaction_json(&decoded) {
                Ok(v) => v,
                Err(e) => {
                    panic!("re-encode failed for tx[{i}] {tx_type}: {e}");
                }
            };

            let re_encoded_hex = hex::encode_upper(&re_encoded);
            assert_eq!(
                re_encoded_hex,
                expected_hex.to_uppercase(),
                "roundtrip mismatch for tx[{i}] {tx_type}"
            );
            passed += 1;
        }

        eprintln!(
            "\nDecode roundtrip: {passed} passed out of {} total",
            transactions.len()
        );
        assert!(passed > 0, "no transactions passed roundtrip test");
    }

    #[test]
    fn roundtrip_account_state_fixtures() {
        let fixtures: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../tests/vectors/codec-fixtures.json"
        ))
        .expect("invalid fixtures");

        let entries = fixtures["accountState"].as_array().unwrap();
        let mut passed = 0;
        let mut skipped = 0;

        for entry in entries.iter() {
            let expected_hex = entry["binary"].as_str().unwrap();
            let binary = hex::decode(expected_hex).unwrap();

            let decoded = match decode_transaction_binary(&binary) {
                Ok(v) => v,
                Err(_) => {
                    skipped += 1;
                    continue;
                }
            };

            let re_encoded = match encode_transaction_json(&decoded) {
                Ok(v) => v,
                Err(_) => {
                    skipped += 1;
                    continue;
                }
            };

            let re_encoded_hex = hex::encode_upper(&re_encoded);
            if re_encoded_hex == expected_hex.to_uppercase() {
                passed += 1;
            } else {
                skipped += 1;
            }
        }

        eprintln!(
            "\nAccount state decode roundtrip: {passed} passed, {skipped} skipped out of {} total",
            entries.len()
        );
        assert!(passed > 0, "no account state entries passed roundtrip test");
    }

    #[test]
    fn fuzz_regression_deeply_nested_objects() {
        // Fuzzer-found crash: deeply nested STObjects causing stack overflow.
        // 0xEE is the STArray type marker; repeated bytes create pathological nesting.
        let data = vec![0xEE; 1024];
        let result = decode_transaction_binary(&data);
        assert!(
            result.is_err(),
            "deeply nested input must return Err, not stack overflow"
        );
    }
}
