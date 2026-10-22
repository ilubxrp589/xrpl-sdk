use crate::address::x_address_to_classic;
use crate::codec::amount;
use crate::codec::definitions::{
    ledger_entry_type_code, lookup_field_def, permission_value_code, transaction_type_code,
};
use crate::codec::field::{encode_vl, FieldId, TypeCode};
use crate::types::AccountId;
use crate::CoreError;
use serde_json::Value;
use std::borrow::Cow;

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

/// X-addresses (XLS-5d) in AccountID fields become classic addresses, as
/// xrpl.js's codec does: an Account's tag becomes SourceTag and a
/// Destination's tag DestinationTag. A tag on any other field, or one that
/// meets an explicit tag field, is an error.
fn with_classic_addresses(
    obj: &serde_json::Map<String, Value>,
) -> Result<Cow<'_, serde_json::Map<String, Value>>, CoreError> {
    let x_address = |name: &str, value: &Value| -> Option<String> {
        let s = value.as_str().filter(|s| s.starts_with(['X', 'T']))?;
        lookup_field_def(name)
            .filter(|f| f.type_code == TypeCode::AccountId as u16)
            .map(|_| s.to_string())
    };
    if !obj.iter().any(|(n, v)| x_address(n, v).is_some()) {
        return Ok(Cow::Borrowed(obj));
    }
    let mut out = obj.clone();
    for (name, value) in obj {
        let Some(x) = x_address(name, value) else {
            continue;
        };
        let (classic, tag, _) = x_address_to_classic(&x)?;
        out.insert(name.clone(), Value::String(classic));
        let Some(tag) = tag else {
            continue;
        };
        let tag_field = match name.as_str() {
            "Account" => "SourceTag",
            "Destination" => "DestinationTag",
            _ => {
                return Err(CoreError::CodecError(format!(
                    "{name} is an X-address with a tag, and {name} has no tag field"
                )))
            }
        };
        if obj.contains_key(tag_field) {
            return Err(CoreError::CodecError(format!(
                "{name} is an X-address with a tag and {tag_field} is also set"
            )));
        }
        out.insert(tag_field.to_string(), Value::from(tag));
    }
    Ok(Cow::Owned(out))
}

/// Encode a transaction from JSON to canonical binary format.
pub fn encode_transaction_json(json: &Value, for_signing: bool) -> Result<Vec<u8>, CoreError> {
    let obj = json
        .as_object()
        .ok_or_else(|| CoreError::CodecError("transaction must be a JSON object".to_string()))?;
    let obj = with_classic_addresses(obj)?;

    let mut fields: Vec<(FieldId, Vec<u8>)> = Vec::new();

    for (name, value) in obj.iter() {
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
        Some(TypeCode::UInt64) => encode_uint64(value, field_name),
        Some(TypeCode::Int32) => encode_int32(value, field_name),
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

/// Int32: big-endian two's complement. rippled (STParsedJSON) takes a JSON
/// integer or a decimal string.
fn encode_int32(value: &Value, field_name: &str) -> Result<Vec<u8>, CoreError> {
    let n = match value {
        Value::Number(n) => n.as_i64().and_then(|n| i32::try_from(n).ok()),
        Value::String(s) => s.parse::<i32>().ok(),
        _ => None,
    };
    n.map(|n| n.to_be_bytes().to_vec()).ok_or_else(|| {
        CoreError::CodecError(format!(
            "Int32 field '{field_name}' must be a 32-bit signed integer, got: {value}"
        ))
    })
}

/// UInt64 as rippled's STParsedJSON takes it: a JSON integer, or a string of
/// hex digits (any length that fits), base 10 for the `kSmdBaseTen` fields
/// (`MaximumAmount`, `MPTAmount`, …).
fn encode_uint64(value: &Value, field_name: &str) -> Result<Vec<u8>, CoreError> {
    let base_ten = super::definitions::UINT64_BASE_TEN.contains(&field_name);
    let n = match value {
        Value::Number(n) => n.as_u64(),
        Value::String(s) => {
            let radix = if base_ten { 10 } else { 16 };
            let digits = !s.is_empty() && s.chars().all(|c| c.is_digit(radix));
            digits.then(|| u64::from_str_radix(s, radix).ok()).flatten()
        }
        _ => None,
    };
    n.map(|n| n.to_be_bytes().to_vec()).ok_or_else(|| {
        CoreError::CodecError(format!(
            "UInt64 field '{field_name}' must be a non-negative integer or a {} string that fits 64 bits, got: {value}",
            if base_ten { "base-10" } else { "hex" }
        ))
    })
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

/// XRP drops written as `rippled` reads them (`amountFromString`): its
/// number syntax (`"100"`, `"-100"`, `"+100"`, `"1e2"`; not `"0100"` or
/// `"1.5"`), whole drops only. Up to 2^61 - 1 drops are accepted, beyond
/// `rippled`'s JSON limit of 10^17, because mainnet ledger entries hold such
/// amounts and the codec must reproduce them.
fn encode_xrp_string(s: &str) -> Result<Vec<u8>, CoreError> {
    let (negative, mantissa, exponent) = super::number::parse(s)?;
    let whole = || CoreError::InvalidAmount(format!("XRP amount {s:?}: XRP is whole drops"));
    if exponent < 0 {
        return Err(whole());
    }
    let drops = if mantissa == 0 {
        0
    } else {
        u32::try_from(exponent)
            .ok()
            .and_then(|e| 10u64.checked_pow(e))
            .and_then(|scale| mantissa.checked_mul(scale))
            .ok_or_else(|| CoreError::InvalidAmount(format!("XRP amount {s:?} is too large")))?
    };
    Ok(amount::encode_xrp_signed(negative, drops)?.to_vec())
}

fn encode_amount_field(value: &Value) -> Result<Vec<u8>, CoreError> {
    match value {
        Value::String(s) => encode_xrp_string(s),
        // rippled's JSON integers are 32-bit (`amountFromJson`).
        Value::Number(n) => match (n.as_i64(), n.as_u64()) {
            (Some(i), _) if i32::try_from(i).is_ok() => {
                Ok(amount::encode_xrp_signed(i < 0, i.unsigned_abs())?.to_vec())
            }
            (_, Some(u)) if u32::try_from(u).is_ok() => Ok(amount::encode_amount_xrp(u)?.to_vec()),
            _ => Err(CoreError::InvalidAmount(format!(
                "XRP amount {n}: rippled takes only 32-bit integers in JSON; write drops as a string"
            ))),
        },
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

    // rippled's `amountFromString`: its number syntax, whole units, at most
    // i64::MAX (maxMPTokenAmount). "1e3" is 1000; "-0" is 0.
    let (negative, mantissa, exponent) = super::number::parse(value_str)?;
    if exponent < 0 {
        return Err(CoreError::InvalidAmount(format!(
            "MPT amount {value_str:?}: MPT amounts are whole units"
        )));
    }
    let magnitude = if mantissa == 0 {
        0
    } else {
        u32::try_from(exponent)
            .ok()
            .and_then(|e| 10u64.checked_pow(e))
            .and_then(|scale| mantissa.checked_mul(scale))
            .filter(|m| i64::try_from(*m).is_ok())
            .ok_or_else(|| {
                CoreError::InvalidAmount(format!("MPT amount {value_str:?} exceeds i64::MAX"))
            })?
    };

    let id_bytes = hex::decode(issuance_id).map_err(|e| CoreError::InvalidHex(e.to_string()))?;
    if id_bytes.len() != 24 {
        return Err(CoreError::InvalidAmount(format!(
            "mpt_issuance_id must be 24 bytes (48 hex characters), got {}",
            id_bytes.len()
        )));
    }

    // STAmount::add: lead byte 0x20 (MPT), plus 0x40 unless negative (zero
    // is never negative), the 8-byte magnitude, then the 24-byte issuance ID.
    let mut result = Vec::with_capacity(33);
    result.push(if negative && magnitude != 0 {
        0x20
    } else {
        0x60
    });
    result.extend_from_slice(&magnitude.to_be_bytes());
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
    // "" is an account never set: an empty VL (rippled's default STAccount, e.g. a UNLModify
    // pseudo-transaction's Account). The zero account, set, is rrrrrrrrrrrrrrrrrrrrrhoLvTp.
    if s.is_empty() {
        return Ok(encode_vl(0));
    }
    let account = AccountId::from_address(s)?;
    let mut result = encode_vl(20);
    result.extend_from_slice(account.as_bytes());
    Ok(result)
}

fn encode_st_object(value: &Value, for_signing: bool) -> Result<Vec<u8>, CoreError> {
    let obj = value
        .as_object()
        .ok_or_else(|| CoreError::CodecError("STObject must be a JSON object".to_string()))?;
    let obj = with_classic_addresses(obj)?;

    let mut fields: Vec<(FieldId, Vec<u8>)> = Vec::new();

    for (name, val) in obj.iter() {
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

/// Encode a PathSet as `rippled` writes one (`STPathSet::add`), from the
/// steps' `account`, `currency`, `mpt_issuance_id` and `issuer` (a `type`
/// field, which `rippled` derives, is ignored). An empty PathSet, an empty
/// path and an empty step are refused: `rippled`'s JSON parser accepts an
/// empty path, but writes bytes its binary reader then refuses.
fn encode_pathset(value: &Value) -> Result<Vec<u8>, CoreError> {
    let paths = value
        .as_array()
        .ok_or_else(|| CoreError::CodecError("PathSet must be a JSON array".to_string()))?;
    if paths.is_empty() {
        return Err(CoreError::CodecError(
            "an empty PathSet; leave the field out instead".to_string(),
        ));
    }
    let string =
        |obj: &serde_json::Map<String, Value>, key: &str| -> Result<Option<String>, CoreError> {
            match obj.get(key) {
                None => Ok(None),
                Some(Value::String(s)) => Ok(Some(s.clone())),
                Some(_) => Err(CoreError::CodecError(format!(
                    "path step {key} must be a string"
                ))),
            }
        };

    let mut result = Vec::new();
    for (i, path) in paths.iter().enumerate() {
        if i > 0 {
            result.push(0xFF); // path separator
        }
        let steps = path
            .as_array()
            .ok_or_else(|| CoreError::CodecError("Path must be a JSON array".to_string()))?;
        if steps.is_empty() {
            return Err(CoreError::CodecError(
                "a path must have at least one step (rippled cannot read an empty path)"
                    .to_string(),
            ));
        }
        for step in steps {
            let obj = step
                .as_object()
                .ok_or_else(|| CoreError::CodecError("PathStep must be an object".to_string()))?;
            let account = string(obj, "account")?;
            let currency = string(obj, "currency")?;
            let mpt = string(obj, "mpt_issuance_id")?;
            let issuer = string(obj, "issuer")?;
            if currency.is_some() && mpt.is_some() {
                return Err(CoreError::CodecError(
                    "a path step cannot have both a currency and an mpt_issuance_id".to_string(),
                ));
            }
            if mpt.is_some() && issuer.is_some() {
                return Err(CoreError::CodecError(
                    "a path step with an mpt_issuance_id cannot have an issuer".to_string(),
                ));
            }

            let mut type_byte: u8 = 0;
            let mut fields = Vec::new();
            if let Some(s) = &account {
                type_byte |= 0x01;
                fields.extend_from_slice(AccountId::from_address(s)?.as_bytes());
            }
            if let Some(s) = &currency {
                type_byte |= 0x10;
                let currency = if s == "XRP" {
                    crate::types::Currency::xrp()
                } else if s.len() == 3 {
                    crate::types::Currency::from_ascii(s)?
                } else {
                    crate::types::Currency::from_hex(s)?
                };
                fields.extend_from_slice(&currency.to_bytes());
            }
            if let Some(s) = &mpt {
                type_byte |= 0x40;
                let id = hex::decode(s).map_err(|e| CoreError::InvalidHex(e.to_string()))?;
                if id.len() != 24 {
                    return Err(CoreError::CodecError(format!(
                        "path step mpt_issuance_id must be 24 bytes, got {}",
                        id.len()
                    )));
                }
                fields.extend_from_slice(&id);
            }
            if let Some(s) = &issuer {
                type_byte |= 0x20;
                fields.extend_from_slice(AccountId::from_address(s)?.as_bytes());
            }
            if type_byte == 0 {
                return Err(CoreError::CodecError(
                    "a path step needs an account, currency, mpt_issuance_id or issuer".to_string(),
                ));
            }
            result.push(type_byte);
            result.extend_from_slice(&fields);
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

    // rippled's to_string(Currency) spells noCurrency (0x00…01) "1"
    // (UintTypes.cpp) — an Oracle BaseAsset/QuoteAsset can carry it
    // (xrpl-validator finding 369).
    if s == "1" {
        let mut b = vec![0u8; 19];
        b.push(1);
        return Ok(b);
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

/// Encode a Number (type 9, `STNumber`): see [`super::number`].
fn encode_number(value: &Value) -> Result<Vec<u8>, CoreError> {
    super::number::encode(value).map(|bytes| bytes.to_vec())
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
    let obj = with_classic_addresses(obj)?;

    // Encode fields, excluding non-signing fields AND Signers array
    let mut fields: Vec<(FieldId, Vec<u8>)> = Vec::new();

    for (name, value) in obj.iter() {
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

    /// XRP as rippled 3.4.1 writes it (`simulate` of a Payment with
    /// `binary: true`): the Amount bytes for each JSON value, or None where
    /// rippled refused the value.
    #[test]
    fn writes_xrp_as_rippled_does() {
        for (value, bytes) in [
            (json!("100"), Some("4000000000000064")),
            (json!("-100"), Some("0000000000000064")),
            (json!("+100"), Some("4000000000000064")),
            (json!("1e2"), Some("4000000000000064")),
            (json!(100), Some("4000000000000064")),
            (json!("0"), Some("4000000000000000")),
            (json!("-0"), Some("4000000000000000")),
            (json!("100000000000000000"), Some("416345785D8A0000")),
            (json!("1.5"), None),
            (json!("00100"), None),
            (json!(" 100"), None),
        ] {
            let got = encode_amount_field(&value).ok().map(hex::encode_upper);
            assert_eq!(got.as_deref(), bytes, "{value}");
        }
        // rippled's JSON refuses more than 10^17 drops; the codec writes them,
        // because mainnet ledger entries hold such amounts.
        assert_eq!(
            hex::encode_upper(encode_amount_field(&json!("100000000000000001")).unwrap()),
            "416345785D8A0001"
        );
        assert!(encode_amount_field(&json!(4_294_967_296u64)).is_err());
    }

    /// PathSets as rippled 3.4.1 writes them (`simulate` with `binary: true`
    /// of a Payment with these Paths): the bytes, or None where the SDK
    /// refuses the value.
    #[test]
    fn writes_paths_as_rippled_does() {
        const ACCT: &str = "F667B0CA50CC7709A220B0561B85E53A48461FA8";
        const ADDR: &str = "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe";
        const USD: &str = "0000000000000000000000005553440000000000";
        const MPT: &str = "000000000000555344000000007D0748616C63796F6EE1F1";
        let xrp = "0".repeat(40);
        for (paths, bytes) in [
            (json!([[{"account": ADDR}]]), Some(format!("01{ACCT}00"))),
            (
                json!([[{"account": ADDR, "type": 48}]]),
                Some(format!("01{ACCT}00")),
            ),
            (
                json!([[{"mpt_issuance_id": MPT}]]),
                Some(format!("40{MPT}00")),
            ),
            (json!([[{"currency": "XRP"}]]), Some(format!("10{xrp}00"))),
            (
                json!([[{"account": ADDR}], [{"currency": "USD", "issuer": ADDR}]]),
                Some(format!("01{ACCT}FF30{USD}{ACCT}00")),
            ),
            // rippled refuses these too.
            (json!([]), None),
            (json!([[{}]]), None),
            (json!([[{"mpt_issuance_id": MPT, "issuer": ADDR}]]), None),
            (json!([[{"currency": "USD", "mpt_issuance_id": MPT}]]), None),
            // rippled writes these (as 00, and as ...FF00), but cannot read
            // what it wrote; the SDK refuses them up front.
            (json!([[]]), None),
            (json!([[{"account": ADDR}], []]), None),
        ] {
            let got = encode_pathset(&paths).ok().map(hex::encode_upper);
            assert_eq!(got, bytes, "{paths}");
        }
    }

    /// MPT amounts as rippled 3.4.1 writes them (as above): the 9 bytes
    /// before the issuance ID, or None where rippled refused the value.
    #[test]
    fn writes_mpt_as_rippled_does() {
        const ID: &str = "000000000000555344000000007D0748616C63796F6EE1F1";
        for (value, bytes) in [
            ("9223372036854775807", Some("607FFFFFFFFFFFFFFF")),
            ("-5", Some("200000000000000005")),
            ("1e3", Some("6000000000000003E8")),
            ("-0", Some("600000000000000000")),
            ("9223372036854775808", None),
            ("1.5", None),
            ("0100", None),
        ] {
            let got = encode_amount_field(&json!({"mpt_issuance_id": ID, "value": value}))
                .ok()
                .map(hex::encode_upper);
            let want = bytes.map(|b| format!("{b}{ID}"));
            assert_eq!(got, want, "{value}");
        }
        assert!(encode_amount_field(&json!({"mpt_issuance_id": "00", "value": "1"})).is_err());
    }

    /// UInt64 in every form rippled's STParsedJSON accepts (found by the live
    /// testnet test: OracleSet's AssetPrice as a number, MaximumAmount in base 10).
    #[test]
    fn uint64_takes_what_rippled_takes() {
        let enc = |v: Value, field: &str| encode_uint64(&v, field).map(hex::encode_upper);
        assert_eq!(enc(json!(74), "AssetPrice").unwrap(), "000000000000004A");
        assert_eq!(enc(json!("2E4"), "AssetPrice").unwrap(), "00000000000002E4");
        assert_eq!(
            enc(json!("00000000000002e4"), "AssetPrice").unwrap(),
            "00000000000002E4"
        );
        assert_eq!(
            enc(json!("100000"), "MaximumAmount").unwrap(),
            "00000000000186A0"
        );
        assert_eq!(
            enc(json!("18446744073709551615"), "MPTAmount").unwrap(),
            "FFFFFFFFFFFFFFFF"
        );
        for (bad, field) in [
            (json!(-1), "AssetPrice"),
            (json!("10000000000000000"), "AssetPrice"),
            (json!("+5"), "AssetPrice"),
            (json!(""), "AssetPrice"),
            (json!("1A"), "MaximumAmount"),
            (json!(1.5), "AssetPrice"),
        ] {
            assert!(enc(bad.clone(), field).is_err(), "{bad} for {field}");
        }
        // What the decoder writes, the encoder reads back.
        for (field, hex) in [
            ("MaximumAmount", "00000000000186A0"),
            ("AssetPrice", "00000000000002E4"),
        ] {
            let (json, _) =
                crate::codec::decode::tests_decode_uint64(&hex::decode(hex).unwrap(), field);
            assert_eq!(enc(json, field).unwrap(), hex);
        }
    }

    /// Mainnet pseudo-transactions round-trip byte for byte. A UNLModify's Account is never set: an empty
    /// VL (`81 00`), "" in JSON (#107068416). An EnableAmendment's is set to the zero account: a 20-byte VL
    /// (`81 14` and twenty zeros), rrrrrrrrrrrrrrrrrrrrrhoLvTp (#106911489).
    #[test]
    fn pseudo_transaction_accounts_round_trip_byte_exact() {
        let cases = [
            ("1200662400000000260661BC006840000000000000007300701321ED9AE4F5887BA029EB7C0884486D23CF\
              281975F773F44BD213054219882C411CC7810000101101", ""),
            ("120064240000000026065F570150133298D47E1F3A8A24FECAA30F699B8FE1DD234E072834BA099AD8180F\
              FCE0FEC4684000000000000000730081140000000000000000000000000000000000000000",
             "rrrrrrrrrrrrrrrrrrrrrhoLvTp"),
        ];
        for (blob_hex, account) in cases {
            let blob = hex::decode(blob_hex).unwrap();
            let json = crate::codec::decode::decode_transaction_binary(&blob).unwrap();
            assert_eq!(json["Account"], account, "{json}");
            assert_eq!(
                encode_transaction_json(&json, false).unwrap(),
                blob,
                "{json}"
            );
        }
        assert_eq!(encode_account_id_field(&json!("")).unwrap(), vec![0x00]);
    }

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
