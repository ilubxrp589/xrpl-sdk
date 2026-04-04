use crate::codec::amount;
use crate::codec::definitions::lookup_field_def_by_id;
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
            Some((code as u16, name.clone()))
        })
        .collect()
});

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
            Some((code as u16, name.clone()))
        })
        .collect()
});

const DEFINITIONS_JSON: &str = include_str!("../../../../tests/vectors/definitions.json");

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
        // Check for end markers
        let current_byte = *bytes
            .get(pos)
            .ok_or_else(|| CoreError::CodecError("unexpected end of buffer".to_string()))?;
        if is_inner && current_byte == 0xE1 {
            pos += 1; // consume the marker
            return Ok((obj, pos));
        }

        // Decode field header
        let remaining = get_slice_from(bytes, pos, "buffer underflow in field header")?;
        let (field_id, header_len) = FieldId::decode(remaining)?;
        pos += header_len;

        // Look up field definition
        let field_def = lookup_field_def_by_id(field_id.type_code, field_id.field_code);
        let field_name = match &field_def {
            Some(fd) => fd.name.clone(),
            None => {
                // Skip unknown fields instead of failing.
                // Determine byte length based on type code and skip.
                let remaining = get_slice_from(bytes, pos, "buffer underflow skipping unknown")?;
                let skip_len = match field_id.type_code {
                    // Fixed-size types
                    1 => 2,  // UINT16
                    2 => 4,  // UINT32
                    3 => 8,  // UINT64
                    4 => 16, // HASH128
                    5 => 32, // HASH256
                    6 => {
                        // AMOUNT: 8 bytes for XRP, 48 for IOU
                        // High bit: 0=XRP(8), 1=IOU(48)
                        if !remaining.is_empty() && (remaining[0] & 0x80) != 0 {
                            48
                        } else {
                            8
                        }
                    }
                    8 => 20, // ACCOUNTID
                    // Variable-length types
                    7 | 18 | 19 => {
                        // VL (blob), Transaction, Validation — length-prefixed
                        match decode_vl(remaining) {
                            Ok((vl_len, vl_header)) => vl_header + vl_len,
                            Err(_) => break, // can't determine length, stop parsing
                        }
                    }
                    // STObject end marker or STArray
                    14 | 15 => 0, // handled by markers
                    _ => break,   // unknown type, stop parsing
                };
                if skip_len > remaining.len() {
                    break;
                }
                pos += skip_len;
                continue;
            }
        };

        let tc = TypeCode::from_u16(field_id.type_code);

        // Decode value based on type code
        let remaining = get_slice_from(bytes, pos, "buffer underflow in field value")?;
        let (value, value_len) = decode_field_value(tc, remaining, &field_name, depth)?;
        pos += value_len;

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
        Some(TypeCode::UInt32) => decode_uint32(buf),
        Some(TypeCode::UInt64) => decode_uint64(buf),
        Some(TypeCode::Hash128) => decode_hash(buf, 16),
        Some(TypeCode::Hash160) => decode_hash(buf, 20),
        Some(TypeCode::Hash256) => decode_hash(buf, 32),
        Some(TypeCode::Amount) => decode_amount_field(buf),
        Some(TypeCode::Blob) => decode_blob(buf),
        Some(TypeCode::AccountId) => decode_account_id_field(buf),
        Some(TypeCode::Number) => decode_number(buf),
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

fn decode_uint32(buf: &[u8]) -> Result<(Value, usize), CoreError> {
    let bytes = buf
        .get(..4)
        .ok_or_else(|| CoreError::CodecError("buffer too short for UInt32".to_string()))?;
    let arr: [u8; 4] = bytes
        .try_into()
        .map_err(|_| CoreError::CodecError("buffer too short for UInt32".to_string()))?;
    let val = u32::from_be_bytes(arr);
    Ok((Value::Number(val.into()), 4))
}

fn decode_uint64(buf: &[u8]) -> Result<(Value, usize), CoreError> {
    let bytes = buf
        .get(..8)
        .ok_or_else(|| CoreError::CodecError("buffer too short for UInt64".to_string()))?;
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

    if first_byte & 0x80 == 0 {
        // XRP amount: 8 bytes
        let bytes = buf
            .get(..8)
            .ok_or_else(|| CoreError::CodecError("buffer too short for XRP amount".to_string()))?;
        let arr: [u8; 8] = bytes
            .try_into()
            .map_err(|_| CoreError::CodecError("buffer too short for XRP amount".to_string()))?;
        let drops = amount::decode_amount_xrp(&arr)?;
        Ok((Value::String(drops.to_string()), 8))
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
        // Check for array end marker
        let &current = buf.get(pos).ok_or_else(|| {
            CoreError::CodecError("unexpected end of buffer in STArray".to_string())
        })?;
        if current == 0xF1 {
            pos += 1;
            return Ok((Value::Array(arr), pos));
        }

        // Each array element is a wrapped object: field header + inner object fields + 0xE1
        // The field header names the wrapper (e.g., "Memo", "SignerEntry")
        let remaining = get_slice_from(buf, pos, "buffer underflow in STArray element")?;
        let (field_id, header_len) = FieldId::decode(remaining)?;
        pos += header_len;

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

fn decode_pathset(buf: &[u8]) -> Result<(Value, usize), CoreError> {
    let mut paths: Vec<Value> = Vec::new();
    let mut current_path: Vec<Value> = Vec::new();
    let mut pos = 0;

    while pos < buf.len() {
        let &byte = buf.get(pos).ok_or_else(|| {
            CoreError::CodecError("unexpected end of buffer in PathSet".to_string())
        })?;

        if byte == 0x00 {
            // PathSet end marker
            pos += 1;
            // Push current path if non-empty
            if !current_path.is_empty() {
                paths.push(Value::Array(current_path));
            }
            return Ok((Value::Array(paths), pos));
        }

        if byte == 0xFF {
            // Path separator
            pos += 1;
            paths.push(Value::Array(current_path));
            current_path = Vec::new();
            continue;
        }

        // Path step: type byte followed by data
        let type_byte = byte;
        pos += 1;

        let mut step = Map::new();

        // Bit 0x01: account (20 bytes)
        if type_byte & 0x01 != 0 {
            let bytes = get_slice(buf, pos..pos + 20, "buffer too short for path step account")?;
            let arr = copy_20(bytes)?;
            let account = AccountId::from_bytes(arr);
            step.insert("account".to_string(), Value::String(account.to_address()));
            pos += 20;
        }

        // Bit 0x10: currency (20 bytes)
        if type_byte & 0x10 != 0 {
            let bytes = get_slice(
                buf,
                pos..pos + 20,
                "buffer too short for path step currency",
            )?;
            let arr = copy_20(bytes)?;
            let currency = Currency::from_bytes(arr);
            step.insert("currency".to_string(), Value::String(currency.to_string()));
            pos += 20;
        }

        // Bit 0x20: issuer (20 bytes)
        if type_byte & 0x20 != 0 {
            let bytes = get_slice(buf, pos..pos + 20, "buffer too short for path step issuer")?;
            let arr = copy_20(bytes)?;
            let issuer = AccountId::from_bytes(arr);
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

    // Try to detect MPT vs IOU:
    if buf.len() >= 44 {
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

fn decode_number(buf: &[u8]) -> Result<(Value, usize), CoreError> {
    // Number type (STNumber): 12 bytes
    // 8-byte signed i64 mantissa (BE) + 4-byte signed i32 exponent (BE)
    if buf.len() < 12 {
        return Err(CoreError::CodecError(
            "buffer too short for Number type".to_string(),
        ));
    }

    let mantissa_bytes: [u8; 8] = buf
        .get(..8)
        .ok_or_else(|| CoreError::CodecError("buffer too short for Number mantissa".to_string()))?
        .try_into()
        .map_err(|_| CoreError::CodecError("buffer too short for Number mantissa".to_string()))?;
    let exponent_bytes: [u8; 4] = buf
        .get(8..12)
        .ok_or_else(|| CoreError::CodecError("buffer too short for Number exponent".to_string()))?
        .try_into()
        .map_err(|_| CoreError::CodecError("buffer too short for Number exponent".to_string()))?;

    let mantissa = i64::from_be_bytes(mantissa_bytes);
    let exponent = i32::from_be_bytes(exponent_bytes);

    const ZERO_EXPONENT: i32 = -2_147_483_648_i32;
    const MIN_MANTISSA: u64 = 1_000_000_000_000_000_000;

    if mantissa == 0 && exponent == ZERO_EXPONENT {
        return Ok((Value::String("0".to_string()), 12));
    }

    let is_negative = mantissa < 0;
    let mut abs_m = mantissa.unsigned_abs();
    let sign = if is_negative { "-" } else { "" };

    // If mantissa < MIN_MANTISSA, it was shrunk for int64 serialization; restore it
    let mut exp = exponent;
    if abs_m != 0 && abs_m < MIN_MANTISSA {
        abs_m = abs_m.saturating_mul(10);
        exp = exp.saturating_sub(1);
    }

    // Use scientific notation for exponents outside [-28, -8] or when exponent != 0 and outside range
    // Matches xrpl.js STNumber.toJSON() — rangeLog=18, so range is [-28, -8]
    if exp != 0 && !(-28..=-8).contains(&exp) {
        // Strip trailing zeros from mantissa (matches rippled behavior)
        while abs_m != 0 && abs_m % 10 == 0 && exp < 32768 {
            abs_m /= 10;
            exp = exp.saturating_add(1);
        }
        return Ok((Value::String(format!("{sign}{abs_m}e{exp}")), 12));
    }

    // Decimal rendering for exponents in [-28, -8] range or exponent == 0
    if exp == 0 {
        return Ok((Value::String(format!("{sign}{abs_m}")), 12));
    }

    // Negative exponent within decimal range
    let digits = abs_m.to_string();
    let neg_exp = (-exp) as usize;

    let result = if neg_exp >= digits.len() {
        let leading_zeros = neg_exp - digits.len();
        format!("{sign}0.{}{}", "0".repeat(leading_zeros), digits)
    } else {
        let (integer, decimal) = digits.split_at(digits.len() - neg_exp);
        let decimal = decimal.trim_end_matches('0');
        if decimal.is_empty() {
            format!("{sign}{integer}")
        } else {
            format!("{sign}{integer}.{decimal}")
        }
    };

    Ok((Value::String(result), 12))
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
        let encoded = encode_transaction_json(&tx_json, false).expect("encode failed");

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
            let re_encoded = match encode_transaction_json(&decoded, false) {
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

            let re_encoded = match encode_transaction_json(&decoded, false) {
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
