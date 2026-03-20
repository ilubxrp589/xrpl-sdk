use crate::codec::amount;
use crate::codec::definitions::{
    ledger_entry_type_code, lookup_field_def, permission_value_code, transaction_type_code,
};
use crate::codec::field::{encode_vl, FieldId, TypeCode};
use crate::types::AccountId;
use crate::CoreError;
use serde_json::Value;

/// Non-serialized field names that appear in JSON but shouldn't be encoded.
const SKIP_FIELDS: &[&str] = &[
    "hash",
    "ctid",
    "date",
    "inLedger",
    "ledger_index",
    "validated",
    "meta",
    "metaData",
    "status",
    "close_time_iso",
];

/// Encode a transaction from JSON to canonical binary format.
pub fn encode_transaction_json(json: &Value, for_signing: bool) -> Result<Vec<u8>, CoreError> {
    let obj = json
        .as_object()
        .ok_or_else(|| CoreError::CodecError("transaction must be a JSON object".to_string()))?;

    let mut fields: Vec<(FieldId, Vec<u8>)> = Vec::new();

    for (name, value) in obj {
        if SKIP_FIELDS.contains(&name.as_str()) {
            continue;
        }

        let field_def = match lookup_field_def(name) {
            Some(f) => f,
            None => continue,
        };

        if !field_def.is_serialized {
            continue;
        }
        if for_signing && !field_def.is_signing {
            continue;
        }

        let field_id = FieldId::new(field_def.type_code, field_def.field_code);

        // Handle string-based TransactionType / LedgerEntryType → numeric
        let resolved_value = resolve_type_field(name, value)?;
        let value_ref = resolved_value.as_ref().unwrap_or(value);

        let encoded_data = encode_field_value(field_def.type_code, name, value_ref, for_signing)?;

        let mut field_bytes = field_id.encode();
        field_bytes.extend_from_slice(&encoded_data);
        fields.push((field_id, field_bytes));
    }

    // Sort by canonical field order: (type_code, field_code)
    fields.sort_by_key(|(fid, _)| fid.sort_key());

    let mut result = Vec::new();
    for (_, bytes) in fields {
        result.extend_from_slice(&bytes);
    }

    Ok(result)
}

/// If the field is TransactionType or LedgerEntryType and value is a string,
/// convert to the numeric code. Returns None if no conversion needed.
fn resolve_type_field(name: &str, value: &Value) -> Result<Option<Value>, CoreError> {
    match name {
        "TransactionType" => {
            if let Some(s) = value.as_str() {
                let code = transaction_type_code(s)?;
                return Ok(Some(Value::Number(serde_json::Number::from(code))));
            }
            Ok(None)
        }
        "LedgerEntryType" => {
            if let Some(s) = value.as_str() {
                let code = ledger_entry_type_code(s)?;
                return Ok(Some(Value::Number(serde_json::Number::from(code))));
            }
            Ok(None)
        }
        _ => Ok(None),
    }
}

/// Encode a single field value based on its type code.
fn encode_field_value(
    type_code: u16,
    field_name: &str,
    value: &Value,
    for_signing: bool,
) -> Result<Vec<u8>, CoreError> {
    let tc = TypeCode::from_u16(type_code);

    match tc {
        Some(TypeCode::UInt8) => encode_uint8(value),
        Some(TypeCode::UInt16) => encode_uint16(value),
        Some(TypeCode::UInt32) => encode_uint32(value, field_name),
        Some(TypeCode::UInt64) => encode_uint64(value),
        Some(TypeCode::Hash128) => encode_hash(value, 16),
        Some(TypeCode::Hash160) => encode_hash(value, 20),
        Some(TypeCode::Hash256) => encode_hash(value, 32),
        Some(TypeCode::Amount) => encode_amount_field(value),
        Some(TypeCode::Blob) => encode_blob(value),
        Some(TypeCode::AccountId) => encode_account_id_field(value),
        Some(TypeCode::Number) => encode_number(value),
        Some(TypeCode::StObject) => encode_st_object(value, for_signing),
        Some(TypeCode::StArray) => encode_st_array(value, for_signing),
        Some(TypeCode::PathSet) => encode_pathset(value),
        Some(TypeCode::Vector256) => encode_vector256(value),
        Some(TypeCode::Issue) => encode_issue(value),
        Some(TypeCode::XChainBridge) => encode_xchain_bridge(value),
        Some(TypeCode::Currency) => encode_currency_type(value),
        Some(TypeCode::UInt96) => encode_raw_hex(value, 12, "UInt96"),
        Some(TypeCode::Hash192) => encode_raw_hex(value, 24, "Hash192"),
        Some(TypeCode::UInt384) => encode_raw_hex(value, 48, "UInt384"),
        Some(TypeCode::UInt512) => encode_raw_hex(value, 64, "UInt512"),
        _ => {
            if value.is_object() {
                encode_st_object(value, for_signing)
            } else {
                Err(CoreError::CodecError(format!(
                    "unsupported type code {type_code} for field '{field_name}'"
                )))
            }
        }
    }
}

fn encode_uint8(value: &Value) -> Result<Vec<u8>, CoreError> {
    let n = value
        .as_u64()
        .ok_or_else(|| CoreError::CodecError("UInt8 must be a number".to_string()))?;
    Ok(vec![n as u8])
}

fn encode_uint16(value: &Value) -> Result<Vec<u8>, CoreError> {
    let n = value
        .as_u64()
        .ok_or_else(|| CoreError::CodecError("UInt16 must be a number".to_string()))?;
    Ok((n as u16).to_be_bytes().to_vec())
}

fn encode_uint32(value: &Value, field_name: &str) -> Result<Vec<u8>, CoreError> {
    // If value is a number, encode directly
    if let Some(n) = value.as_u64() {
        return Ok((n as u32).to_be_bytes().to_vec());
    }
    // Some UInt32 fields accept string values that need special resolution
    // (e.g., PermissionValue uses transaction type names or composite codes)
    if let Some(s) = value.as_str() {
        // Try parsing as decimal number first
        if let Ok(n) = s.parse::<u32>() {
            return Ok(n.to_be_bytes().to_vec());
        }
        // Try parsing as hex (0x prefix)
        if let Some(hex_str) = s.strip_prefix("0x") {
            if let Ok(n) = u32::from_str_radix(hex_str, 16) {
                return Ok(n.to_be_bytes().to_vec());
            }
        }
        // For PermissionValue, resolve via granular permissions + transaction types
        if field_name == "PermissionValue" {
            let code = permission_value_code(s)?;
            return Ok(code.to_be_bytes().to_vec());
        }
    }
    Err(CoreError::CodecError(format!(
        "UInt32 field '{field_name}' must be a number, got: {value}"
    )))
}

fn encode_uint64(value: &Value) -> Result<Vec<u8>, CoreError> {
    let s = value
        .as_str()
        .ok_or_else(|| CoreError::CodecError("UInt64 must be a hex string".to_string()))?;
    let bytes = hex::decode(s).map_err(|e| CoreError::InvalidHex(e.to_string()))?;
    if bytes.len() != 8 {
        return Err(CoreError::CodecError(format!(
            "UInt64 hex must be 8 bytes, got {}",
            bytes.len()
        )));
    }
    Ok(bytes)
}

fn encode_hash(value: &Value, expected_len: usize) -> Result<Vec<u8>, CoreError> {
    let s = value
        .as_str()
        .ok_or_else(|| CoreError::CodecError("Hash must be a hex string".to_string()))?;
    let bytes = hex::decode(s).map_err(|e| CoreError::InvalidHex(e.to_string()))?;
    if bytes.len() != expected_len {
        return Err(CoreError::CodecError(format!(
            "Hash expected {expected_len} bytes, got {}",
            bytes.len()
        )));
    }
    Ok(bytes)
}

fn encode_amount_field(value: &Value) -> Result<Vec<u8>, CoreError> {
    match value {
        Value::String(s) => {
            let drops: u64 = s
                .parse()
                .map_err(|_| CoreError::InvalidAmount(format!("invalid XRP drops: {s}")))?;
            Ok(amount::encode_amount_xrp(drops)?.to_vec())
        }
        Value::Object(obj) => {
            // Check for MPT amount (has mpt_issuance_id)
            if obj.contains_key("mpt_issuance_id") {
                return encode_mpt_amount(obj);
            }

            let value_str = obj
                .get("value")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CoreError::InvalidAmount("IOU missing 'value'".to_string()))?;
            let currency_str = obj
                .get("currency")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CoreError::InvalidAmount("IOU missing 'currency'".to_string()))?;
            let issuer_str = obj
                .get("issuer")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CoreError::InvalidAmount("IOU missing 'issuer'".to_string()))?;

            let iou = crate::types::IouAmount::from_decimal(value_str)?;
            let currency = if currency_str.len() == 3 {
                crate::types::Currency::from_ascii(currency_str)?
            } else if currency_str.len() == 40 {
                crate::types::Currency::from_hex(currency_str)?
            } else {
                return Err(CoreError::InvalidCurrency(format!(
                    "invalid currency: {currency_str}"
                )));
            };
            let issuer = AccountId::from_address(issuer_str)?;

            Ok(amount::encode_amount_iou(&iou, &currency, &issuer).to_vec())
        }
        _ => Err(CoreError::InvalidAmount(
            "Amount must be string (XRP) or object (IOU/MPT)".to_string(),
        )),
    }
}

/// Encode MPT (Multi-Purpose Token) amount.
/// Format: 1 byte (0x60 positive, 0x00 negative/zero) + 8 bytes value + 32 bytes issuance_id
fn encode_mpt_amount(obj: &serde_json::Map<String, Value>) -> Result<Vec<u8>, CoreError> {
    let value_str = obj
        .get("value")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CoreError::InvalidAmount("MPT missing 'value'".to_string()))?;
    let issuance_id = obj
        .get("mpt_issuance_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CoreError::InvalidAmount("MPT missing 'mpt_issuance_id'".to_string()))?;

    let value: i64 = value_str
        .parse()
        .map_err(|_| CoreError::InvalidAmount(format!("invalid MPT value: {value_str}")))?;

    let id_bytes = hex::decode(issuance_id).map_err(|e| CoreError::InvalidHex(e.to_string()))?;

    let mut result = Vec::with_capacity(41);

    // First byte: 0x60 for positive, 0x00 for zero/negative
    if value > 0 {
        result.push(0x60);
    } else {
        result.push(0x00);
    }

    // 8 bytes big-endian value (unsigned magnitude)
    result.extend_from_slice(&(value.unsigned_abs()).to_be_bytes());

    // 32 bytes issuance ID (or however many bytes it is — usually 32)
    result.extend_from_slice(&id_bytes);

    Ok(result)
}

fn encode_blob(value: &Value) -> Result<Vec<u8>, CoreError> {
    let s = value
        .as_str()
        .ok_or_else(|| CoreError::CodecError("Blob must be a hex string".to_string()))?;
    let bytes = hex::decode(s).map_err(|e| CoreError::InvalidHex(e.to_string()))?;
    let mut result = encode_vl(bytes.len());
    result.extend_from_slice(&bytes);
    Ok(result)
}

fn encode_account_id_field(value: &Value) -> Result<Vec<u8>, CoreError> {
    let s = value
        .as_str()
        .ok_or_else(|| CoreError::CodecError("AccountID must be a string".to_string()))?;
    let account = AccountId::from_address(s)?;
    let mut result = encode_vl(20);
    result.extend_from_slice(account.as_bytes());
    Ok(result)
}

fn encode_st_object(value: &Value, for_signing: bool) -> Result<Vec<u8>, CoreError> {
    let obj = value
        .as_object()
        .ok_or_else(|| CoreError::CodecError("STObject must be a JSON object".to_string()))?;

    let mut fields: Vec<(FieldId, Vec<u8>)> = Vec::new();

    for (name, val) in obj {
        if SKIP_FIELDS.contains(&name.as_str()) {
            continue;
        }

        let field_def = match lookup_field_def(name) {
            Some(f) => f,
            None => continue,
        };
        if !field_def.is_serialized {
            continue;
        }
        if for_signing && !field_def.is_signing {
            continue;
        }

        let field_id = FieldId::new(field_def.type_code, field_def.field_code);

        let resolved_value = resolve_type_field(name, val)?;
        let val_ref = resolved_value.as_ref().unwrap_or(val);

        let encoded_data = encode_field_value(field_def.type_code, name, val_ref, for_signing)?;
        let mut field_bytes = field_id.encode();
        field_bytes.extend_from_slice(&encoded_data);
        fields.push((field_id, field_bytes));
    }

    fields.sort_by_key(|(fid, _)| fid.sort_key());

    let mut result = Vec::new();
    for (_, bytes) in fields {
        result.extend_from_slice(&bytes);
    }
    // Object end marker
    result.push(0xE1);
    Ok(result)
}

fn encode_st_array(value: &Value, for_signing: bool) -> Result<Vec<u8>, CoreError> {
    let arr = value
        .as_array()
        .ok_or_else(|| CoreError::CodecError("STArray must be a JSON array".to_string()))?;

    let mut result = Vec::new();

    for element in arr {
        let obj = element.as_object().ok_or_else(|| {
            CoreError::CodecError("STArray element must be a JSON object".to_string())
        })?;

        // Each array element is wrapped: { "Memo": { ... } }
        for (wrapper_name, inner_value) in obj {
            let field_def = match lookup_field_def(wrapper_name) {
                Some(f) => f,
                None => continue,
            };

            let field_id = FieldId::new(field_def.type_code, field_def.field_code);
            let encoded_data = encode_st_object(inner_value, for_signing)?;

            let header = field_id.encode();
            result.extend_from_slice(&header);
            result.extend_from_slice(&encoded_data);
        }
    }

    // Array end marker
    result.push(0xF1);
    Ok(result)
}

fn encode_pathset(value: &Value) -> Result<Vec<u8>, CoreError> {
    let paths = value
        .as_array()
        .ok_or_else(|| CoreError::CodecError("PathSet must be a JSON array".to_string()))?;

    let mut result = Vec::new();

    for (i, path) in paths.iter().enumerate() {
        if i > 0 {
            result.push(0xFF); // path separator
        }

        let steps = path
            .as_array()
            .ok_or_else(|| CoreError::CodecError("Path must be a JSON array".to_string()))?;

        for step in steps {
            let obj = step
                .as_object()
                .ok_or_else(|| CoreError::CodecError("PathStep must be an object".to_string()))?;

            let has_account = obj.contains_key("account");
            let has_currency = obj.contains_key("currency");
            let has_issuer = obj.contains_key("issuer");

            let mut type_byte: u8 = 0;
            if has_account {
                type_byte |= 0x01;
            }
            if has_currency {
                type_byte |= 0x10;
            }
            if has_issuer {
                type_byte |= 0x20;
            }

            result.push(type_byte);

            if has_account {
                let s = obj["account"]
                    .as_str()
                    .ok_or_else(|| CoreError::CodecError("account must be string".to_string()))?;
                let account = AccountId::from_address(s)?;
                result.extend_from_slice(account.as_bytes());
            }

            if has_currency {
                let s = obj["currency"]
                    .as_str()
                    .ok_or_else(|| CoreError::CodecError("currency must be string".to_string()))?;
                let currency = if s == "XRP" {
                    crate::types::Currency::xrp()
                } else if s.len() == 3 {
                    crate::types::Currency::from_ascii(s)?
                } else {
                    crate::types::Currency::from_hex(s)?
                };
                result.extend_from_slice(&currency.to_bytes());
            }

            if has_issuer {
                let s = obj["issuer"]
                    .as_str()
                    .ok_or_else(|| CoreError::CodecError("issuer must be string".to_string()))?;
                let issuer = AccountId::from_address(s)?;
                result.extend_from_slice(issuer.as_bytes());
            }
        }
    }

    result.push(0x00); // pathset end marker
    Ok(result)
}

fn encode_vector256(value: &Value) -> Result<Vec<u8>, CoreError> {
    let arr = value
        .as_array()
        .ok_or_else(|| CoreError::CodecError("Vector256 must be a JSON array".to_string()))?;

    let mut data = Vec::new();
    for item in arr {
        let s = item
            .as_str()
            .ok_or_else(|| CoreError::CodecError("Vector256 element must be hex".to_string()))?;
        let bytes = hex::decode(s).map_err(|e| CoreError::InvalidHex(e.to_string()))?;
        if bytes.len() != 32 {
            return Err(CoreError::CodecError(
                "Vector256 element must be 32 bytes".to_string(),
            ));
        }
        data.extend_from_slice(&bytes);
    }

    let mut result = encode_vl(data.len());
    result.extend_from_slice(&data);
    Ok(result)
}

/// Encode an Issue type (type 24).
/// XRP: 20 zero bytes.
/// IOU: 20 bytes currency + 20 bytes issuer.
/// MPT: 0x99 prefix + 24 bytes mpt_issuance_id.
fn encode_issue(value: &Value) -> Result<Vec<u8>, CoreError> {
    let obj = value
        .as_object()
        .ok_or_else(|| CoreError::CodecError("Issue must be a JSON object".to_string()))?;

    // MPT Issue format: { "mpt_issuance_id": "hex..." }
    // Wire: issuer (20B) + NO_ACCOUNT (20B) + sequence_LE (4B) = 44 bytes
    if let Some(mpt_id) = obj.get("mpt_issuance_id").and_then(|v| v.as_str()) {
        let id_bytes = hex::decode(mpt_id).map_err(|e| CoreError::InvalidHex(e.to_string()))?;
        if id_bytes.len() != 24 {
            return Err(CoreError::CodecError(format!(
                "mpt_issuance_id must be 24 bytes, got {}",
                id_bytes.len()
            )));
        }

        // mpt_issuance_id = sequence (4 bytes BE) + issuer (20 bytes)
        // Length was already checked to be 24
        let id_arr: [u8; 24] = id_bytes
            .try_into()
            .map_err(|_| CoreError::CodecError("mpt_issuance_id must be 24 bytes".to_string()))?;
        // id_arr is [u8; 24], so these indices are always valid
        let sequence_be: [u8; 4] = [id_arr[0], id_arr[1], id_arr[2], id_arr[3]];
        let issuer = &id_arr[4..24];

        // Convert sequence to little-endian for wire format
        let sequence_le = [
            sequence_be[3],
            sequence_be[2],
            sequence_be[1],
            sequence_be[0],
        ];

        let no_account: [u8; 20] = {
            let mut a = [0u8; 20];
            a[19] = 0x01;
            a
        };

        let mut result = Vec::with_capacity(44);
        result.extend_from_slice(issuer); // 20 bytes issuer
        result.extend_from_slice(&no_account); // 20 bytes NO_ACCOUNT placeholder
        result.extend_from_slice(&sequence_le); // 4 bytes sequence LE
        return Ok(result);
    }

    let currency_str = obj
        .get("currency")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            CoreError::CodecError("Issue missing 'currency' or 'mpt_issuance_id'".to_string())
        })?;

    if currency_str == "XRP" {
        // XRP issue: 20 zero bytes (no issuer)
        return Ok(vec![0u8; 20]);
    }

    // Non-XRP: 20 bytes currency + 20 bytes issuer
    let currency = if currency_str.len() == 3 {
        crate::types::Currency::from_ascii(currency_str)?
    } else {
        crate::types::Currency::from_hex(currency_str)?
    };

    let issuer_str = obj
        .get("issuer")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CoreError::CodecError("Non-XRP Issue missing 'issuer'".to_string()))?;
    let issuer = AccountId::from_address(issuer_str)?;

    let mut result = Vec::with_capacity(40);
    result.extend_from_slice(&currency.to_bytes());
    result.extend_from_slice(issuer.as_bytes());
    Ok(result)
}

/// Encode XChainBridge type (type 25).
/// Fixed format: VL(20) + LockingChainDoor + LockingChainIssue + VL(20) + IssuingChainDoor + IssuingChainIssue
fn encode_xchain_bridge(value: &Value) -> Result<Vec<u8>, CoreError> {
    let obj = value
        .as_object()
        .ok_or_else(|| CoreError::CodecError("XChainBridge must be a JSON object".to_string()))?;

    let mut result = Vec::new();

    // LockingChainDoor (AccountID, VL-prefixed)
    let door1 = obj
        .get("LockingChainDoor")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CoreError::CodecError("missing LockingChainDoor".to_string()))?;
    let door1_id = AccountId::from_address(door1)?;
    result.extend_from_slice(&encode_vl(20));
    result.extend_from_slice(door1_id.as_bytes());

    // LockingChainIssue (Issue type, raw bytes)
    let issue1 = obj
        .get("LockingChainIssue")
        .ok_or_else(|| CoreError::CodecError("missing LockingChainIssue".to_string()))?;
    result.extend_from_slice(&encode_issue(issue1)?);

    // IssuingChainDoor (AccountID, VL-prefixed)
    let door2 = obj
        .get("IssuingChainDoor")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CoreError::CodecError("missing IssuingChainDoor".to_string()))?;
    let door2_id = AccountId::from_address(door2)?;
    result.extend_from_slice(&encode_vl(20));
    result.extend_from_slice(door2_id.as_bytes());

    // IssuingChainIssue (Issue type, raw bytes)
    let issue2 = obj
        .get("IssuingChainIssue")
        .ok_or_else(|| CoreError::CodecError("missing IssuingChainIssue".to_string()))?;
    result.extend_from_slice(&encode_issue(issue2)?);

    Ok(result)
}

/// Encode a Currency type (type 26).
/// Just 20 bytes of currency code.
fn encode_currency_type(value: &Value) -> Result<Vec<u8>, CoreError> {
    let s = value
        .as_str()
        .ok_or_else(|| CoreError::CodecError("Currency must be a string".to_string()))?;

    if s == "XRP" {
        return Ok(vec![0u8; 20]);
    }

    if s.len() == 3 {
        let c = crate::types::Currency::from_ascii(s)?;
        Ok(c.to_bytes().to_vec())
    } else if s.len() == 40 {
        let c = crate::types::Currency::from_hex(s)?;
        Ok(c.to_bytes().to_vec())
    } else {
        Err(CoreError::InvalidCurrency(format!(
            "invalid currency string: {s}"
        )))
    }
}

/// Encode a Number type (type 9) — STNumber.
/// 12 bytes: 8-byte signed i64 mantissa (BE) + 4-byte signed i32 exponent (BE).
/// Mantissa normalized to [10^18, 2^63-1]. Zero = mantissa 0, exponent -2147483648.
fn encode_number(value: &Value) -> Result<Vec<u8>, CoreError> {
    let s = value
        .as_str()
        .ok_or_else(|| CoreError::CodecError("Number must be a string".to_string()))?;

    const MIN_MANTISSA: i128 = 1_000_000_000_000_000_000; // 10^18
    const MAX_INT64: i128 = 9_223_372_036_854_775_807; // 2^63 - 1
    const ZERO_EXPONENT: i32 = -2_147_483_648_i32;

    // Parse the string
    let (is_negative, base_str) = if let Some(stripped) = s.strip_prefix('-') {
        (true, stripped)
    } else {
        (false, s)
    };

    // Handle scientific notation
    let (base, sci_exp) = if let Some((b, e)) = base_str
        .split_once('e')
        .or_else(|| base_str.split_once('E'))
    {
        let exp: i32 = e
            .parse()
            .map_err(|_| CoreError::InvalidAmount(format!("invalid exponent: {e}")))?;
        (b, exp)
    } else {
        (base_str, 0i32)
    };

    let (integer_part, decimal_part) = match base.split_once('.') {
        Some((i, d)) => (i, d),
        None => (base, ""),
    };

    let combined = format!("{integer_part}{decimal_part}");
    let combined = combined.trim_start_matches('0');
    if combined.is_empty() || s == "0" {
        // Zero: mantissa = 0, exponent = DEFAULT_VALUE_EXPONENT
        let mut buf = vec![0u8; 8];
        buf.extend_from_slice(&ZERO_EXPONENT.to_be_bytes());
        return Ok(buf);
    }

    let mut mantissa: i128 = combined
        .parse()
        .map_err(|e: std::num::ParseIntError| CoreError::InvalidAmount(e.to_string()))?;
    let mut exponent: i32 = -(decimal_part.len() as i32) + sci_exp;

    if is_negative {
        mantissa = -mantissa;
    }

    // Strip trailing zeros from mantissa
    while mantissa != 0 && mantissa % 10 == 0 {
        mantissa /= 10;
        exponent += 1;
    }

    // Normalize: mantissa into [MIN_MANTISSA, MAX_INT64]
    let abs_m = mantissa.unsigned_abs();
    let mut m = abs_m;
    let mut e = exponent;

    while (m as i128) < MIN_MANTISSA && e > -32768 {
        m *= 10;
        e -= 1;
    }
    while m as i128 > MAX_INT64 && e < 32768 {
        m /= 10;
        e += 1;
    }

    let signed_m: i64 = if is_negative { -(m as i64) } else { m as i64 };

    let mut buf = Vec::with_capacity(12);
    buf.extend_from_slice(&signed_m.to_be_bytes());
    buf.extend_from_slice(&e.to_be_bytes());
    Ok(buf)
}

/// Encode a raw hex value of fixed size.
fn encode_raw_hex(
    value: &Value,
    expected_len: usize,
    type_name: &str,
) -> Result<Vec<u8>, CoreError> {
    let s = value
        .as_str()
        .ok_or_else(|| CoreError::CodecError(format!("{type_name} must be a hex string")))?;
    let bytes = hex::decode(s).map_err(|e| CoreError::InvalidHex(e.to_string()))?;
    if bytes.len() != expected_len {
        return Err(CoreError::CodecError(format!(
            "{type_name} expected {expected_len} bytes, got {}",
            bytes.len()
        )));
    }
    Ok(bytes)
}

/// Encode a transaction for multi-signing by a specific account.
/// Byte layout: SMT\0 prefix + encoded_tx_bytes (excluding TxnSignature and Signers) + signing_account 20 bytes.
pub fn encode_for_multisigning(tx: &Value, signing_account: &str) -> Result<Vec<u8>, CoreError> {
    let obj = tx
        .as_object()
        .ok_or_else(|| CoreError::CodecError("transaction must be a JSON object".to_string()))?;

    // Encode fields, excluding non-signing fields AND Signers array
    let mut fields: Vec<(FieldId, Vec<u8>)> = Vec::new();

    for (name, value) in obj {
        if SKIP_FIELDS.contains(&name.as_str()) {
            continue;
        }
        // Skip Signers array during multisign encoding
        if name == "Signers" {
            continue;
        }

        let field_def = match lookup_field_def(name) {
            Some(f) => f,
            None => continue,
        };

        if !field_def.is_serialized {
            continue;
        }
        // Use for_signing=true to exclude TxnSignature
        if !field_def.is_signing {
            continue;
        }

        let field_id = FieldId::new(field_def.type_code, field_def.field_code);
        let resolved_value = resolve_type_field(name, value)?;
        let value_ref = resolved_value.as_ref().unwrap_or(value);
        let encoded_data = encode_field_value(field_def.type_code, name, value_ref, true)?;

        let mut field_bytes = field_id.encode();
        field_bytes.extend_from_slice(&encoded_data);
        fields.push((field_id, field_bytes));
    }

    fields.sort_by_key(|(fid, _)| fid.sort_key());

    // SMT\0 prefix
    let mut result = vec![0x53, 0x4D, 0x54, 0x00];

    for (_, bytes) in fields {
        result.extend_from_slice(&bytes);
    }

    // Append signing account's 20-byte AccountID
    let account_id = AccountId::from_address(signing_account)?;
    result.extend_from_slice(account_id.as_bytes());

    Ok(result)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn encode_simple_payment() {
        let tx = json!({
            "TransactionType": "Payment",
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
            "Amount": "1000000",
            "Fee": "12",
            "Sequence": 1,
            "Flags": 0,
            "SigningPubKey": "",
            "LastLedgerSequence": 100
        });

        let result = encode_transaction_json(&tx, false);
        assert!(result.is_ok(), "encode failed: {:?}", result.err());
        let bytes = result.unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn encode_preserves_canonical_order() {
        let tx = json!({
            "Fee": "12",
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "TransactionType": "Payment",
            "Sequence": 1,
            "Flags": 0,
            "Amount": "1000000",
            "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
            "SigningPubKey": ""
        });

        let result = encode_transaction_json(&tx, false).unwrap();
        // First field header should be TransactionType (type=1, field=2) = 0x12
        assert_eq!(result[0], 0x12);
    }

    #[test]
    fn encode_for_signing_excludes_signature() {
        let tx = json!({
            "TransactionType": "Payment",
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
            "Amount": "1000000",
            "Fee": "12",
            "Sequence": 1,
            "Flags": 0,
            "SigningPubKey": "ED5F5AC43F527AE97194FF17FDA5E7F9B36D9BCE3E539FCB987B8E6CCD40C4DE47",
            "TxnSignature": "AABBCCDD"
        });

        let with_sig = encode_transaction_json(&tx, false).unwrap();
        let without_sig = encode_transaction_json(&tx, true).unwrap();
        assert!(without_sig.len() < with_sig.len());
    }
}
