//! WASM bindings for xrpl-core — exposes core functions to JavaScript.
//! Only compiled when the `wasm` feature is enabled.

use wasm_bindgen::prelude::*;

use crate::CoreError;

fn to_js_error(e: CoreError) -> JsValue {
    JsValue::from_str(&e.to_string())
}

/// Encode a transaction JSON string to a hex blob.
///
/// For a `wasm32` build (the `wasm` feature): JavaScript calls it as
/// `encode_transaction(json)`.
///
/// ```no_run
/// use xrpl_core::wasm_bindgen_exports::encode_transaction;
///
/// let blob = encode_transaction(
///     r#"{"TransactionType": "AccountSet", "Account": "rU6K7V3Po4snVhBBaU29sesqs2qTQJWDw1",
///         "Fee": "12", "Sequence": 1}"#,
/// )?;
/// assert!(blob.starts_with("120003")); // TransactionType: AccountSet (3)
/// # Ok::<(), wasm_bindgen::JsValue>(())
/// ```
///
/// # Errors
///
/// A `JsValue` string if `json_str` is not JSON, or holding the [`CoreError`]
/// message if
/// [`encode_transaction_json`](crate::codec::encode_transaction_json) refuses
/// the transaction.
#[wasm_bindgen]
pub fn encode_transaction(json_str: &str) -> Result<String, JsValue> {
    let value: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| JsValue::from_str(&format!("invalid JSON: {e}")))?;
    let bytes = crate::codec::encode_transaction_json(&value).map_err(to_js_error)?;
    Ok(hex::encode_upper(bytes))
}

/// What a single signature covers, as hex: the prefix `"STX\0"`
/// (`53545800`) and the transaction's signed fields, as xrpl.js's
/// `encodeForSigning` returns it.
///
/// For a `wasm32` build (the `wasm` feature): JavaScript calls it as
/// `encode_for_signing(json)`.
///
/// ```no_run
/// use xrpl_core::wasm_bindgen_exports::encode_for_signing;
///
/// let data = encode_for_signing(
///     r#"{"TransactionType": "AccountSet", "Account": "rU6K7V3Po4snVhBBaU29sesqs2qTQJWDw1",
///         "Fee": "12", "Sequence": 1, "SigningPubKey": ""}"#,
/// )?;
/// assert!(data.starts_with("53545800"));
/// # Ok::<(), wasm_bindgen::JsValue>(())
/// ```
///
/// # Errors
///
/// A `JsValue` string if `json_str` is not JSON, or holding the [`CoreError`]
/// message if [`encode_for_signing`](crate::codec::encode_for_signing)
/// refuses the transaction.
#[wasm_bindgen]
pub fn encode_for_signing(json_str: &str) -> Result<String, JsValue> {
    let value: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| JsValue::from_str(&format!("invalid JSON: {e}")))?;
    let bytes = crate::codec::encode_for_signing(&value).map_err(to_js_error)?;
    Ok(hex::encode_upper(bytes))
}

/// Decode a hex blob back to a transaction JSON string.
///
/// For a `wasm32` build (the `wasm` feature): JavaScript calls it as
/// `decode_transaction(hex)`.
///
/// ```no_run
/// use xrpl_core::wasm_bindgen_exports::{decode_transaction, encode_transaction};
///
/// // A payment rippled 3.4.1 signed.
/// let blob = "120000220000000024000000016140000000000F424068400000000000000C7321030D58EB48B4420B1F7B9DF55087E0E29FEF0E8468F9A6825B01CA2C361042D43574473045022100BC95504DC72DDBDAA0F5EAA973EC07E057FD9A3E7500AC3CF07454C68219985E02200B0992C47857A867F5664BB6F74E62F086A59A0C31848E0FB3819127E8EC0A1281148049717CC948789F32F267ADC2582484E3DFA6988314B5F762798A53D543A014CAF8B297CFF8F2F937E8";
/// let json = decode_transaction(blob)?;
/// assert!(json.contains(r#""TransactionType":"Payment""#));
/// assert_eq!(encode_transaction(&json)?, blob);
/// # Ok::<(), wasm_bindgen::JsValue>(())
/// ```
///
/// # Errors
///
/// A `JsValue` string if `hex_str` is not hex, or holding the [`CoreError`]
/// message if
/// [`decode_transaction_binary`](crate::codec::decode_transaction_binary)
/// refuses the bytes.
#[wasm_bindgen]
pub fn decode_transaction(hex_str: &str) -> Result<String, JsValue> {
    let bytes =
        hex::decode(hex_str).map_err(|e| JsValue::from_str(&format!("invalid hex: {e}")))?;
    let value = crate::codec::decode_transaction_binary(&bytes).map_err(to_js_error)?;
    serde_json::to_string(&value)
        .map_err(|e| JsValue::from_str(&format!("JSON serialization failed: {e}")))
}

/// Encode a classic XRPL address to its 20-byte AccountID hex.
///
/// For a `wasm32` build (the `wasm` feature): JavaScript calls it as
/// `address_to_account_id(address)`.
///
/// ```no_run
/// use xrpl_core::wasm_bindgen_exports::address_to_account_id;
///
/// assert_eq!(
///     address_to_account_id("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")?,
///     "B5F762798A53D543A014CAF8B297CFF8F2F937E8",
/// );
/// # Ok::<(), wasm_bindgen::JsValue>(())
/// ```
///
/// # Errors
///
/// A `JsValue` string holding the [`CoreError`] message if `address` is not a
/// classic address.
#[wasm_bindgen]
pub fn address_to_account_id(address: &str) -> Result<String, JsValue> {
    let bytes = crate::address::decode_account_id(address).map_err(to_js_error)?;
    Ok(hex::encode_upper(bytes))
}

/// Encode a 20-byte AccountID hex to a classic XRPL address.
///
/// For a `wasm32` build (the `wasm` feature): JavaScript calls it as
/// `account_id_to_address(hex)`.
///
/// ```no_run
/// use xrpl_core::wasm_bindgen_exports::account_id_to_address;
///
/// assert_eq!(
///     account_id_to_address("B5F762798A53D543A014CAF8B297CFF8F2F937E8")?,
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
/// );
/// assert!(account_id_to_address("B5F7").is_err());
/// # Ok::<(), wasm_bindgen::JsValue>(())
/// ```
///
/// # Errors
///
/// A `JsValue` string if `hex_str` is not hex or not 20 bytes.
#[wasm_bindgen]
pub fn account_id_to_address(hex_str: &str) -> Result<String, JsValue> {
    let bytes =
        hex::decode(hex_str).map_err(|e| JsValue::from_str(&format!("invalid hex: {e}")))?;
    if bytes.len() != 20 {
        return Err(JsValue::from_str("AccountID must be exactly 20 bytes"));
    }
    // Length was checked to be 20 above, so this conversion always succeeds
    let arr: [u8; 20] = bytes
        .try_into()
        .map_err(|_| JsValue::from_str("AccountID must be exactly 20 bytes"))?;
    Ok(crate::address::encode_account_id(&arr))
}

/// Validate that a string is a valid classic XRPL address.
///
/// For a `wasm32` build (the `wasm` feature): JavaScript calls it as
/// `is_valid_address(address)`.
///
/// ```no_run
/// use xrpl_core::wasm_bindgen_exports::is_valid_address;
///
/// assert!(is_valid_address("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe"));
/// assert!(!is_valid_address("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYf"));
/// ```
#[wasm_bindgen]
pub fn is_valid_address(address: &str) -> bool {
    crate::address::decode_account_id(address).is_ok()
}

/// Generate a new random wallet seed (base58check encoded).
///
/// For a `wasm32` build (the `wasm` feature): JavaScript calls it as
/// `generate_seed("ed25519")`.
///
/// ```no_run
/// use xrpl_core::wasm_bindgen_exports::generate_seed;
///
/// assert!(generate_seed("ed25519")?.starts_with("sEd"));
/// assert!(generate_seed("rsa").is_err());
/// # Ok::<(), wasm_bindgen::JsValue>(())
/// ```
///
/// # Errors
///
/// A `JsValue` string if `key_type` is not `ed25519` or `secp256k1` (or
/// `Ed25519`, `Secp256k1`).
#[wasm_bindgen]
pub fn generate_seed(key_type: &str) -> Result<String, JsValue> {
    let kt = match key_type {
        "ed25519" | "Ed25519" => crate::address::KeyType::Ed25519,
        "secp256k1" | "Secp256k1" => crate::address::KeyType::Secp256k1,
        _ => {
            return Err(JsValue::from_str(
                "key_type must be 'ed25519' or 'secp256k1'",
            ))
        }
    };
    let seed = crate::crypto::Seed::generate_with_type(kt);
    Ok(seed.to_base58().to_string())
}

/// Derive the classic address from a seed string.
///
/// For a `wasm32` build (the `wasm` feature): JavaScript calls it as
/// `seed_to_address(seed)`.
///
/// ```no_run
/// use xrpl_core::wasm_bindgen_exports::seed_to_address;
///
/// assert_eq!(
///     seed_to_address("sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r")?,
///     "rLUEXYuLiQptky37CqLcm9USQpPiz5rkpD",
/// );
/// # Ok::<(), wasm_bindgen::JsValue>(())
/// ```
///
/// # Errors
///
/// A `JsValue` string holding the [`CoreError`] message if `seed` is not a
/// seed (or, for secp256k1, no key derives from it).
#[wasm_bindgen]
pub fn seed_to_address(seed: &str) -> Result<String, JsValue> {
    let keypair = crate::crypto::Seed::from_base58(seed)
        .and_then(|s| s.derive_keypair())
        .map_err(to_js_error)?;
    Ok(keypair.classic_address())
}

/// Sign `message_hex` with a seed's key: Ed25519 signs the bytes;
/// secp256k1 signs them as an already-computed 32-byte hash, and any other
/// length is an error (for XRPL data, SHA-512Half it first). The key is
/// wiped when done.
///
/// For a `wasm32` build (the `wasm` feature): JavaScript calls it as
/// `sign_message(seed, message_hex)`.
///
/// ```no_run
/// use xrpl_core::wasm_bindgen_exports::sign_message;
///
/// // An Ed25519 key signs the bytes themselves: 64 bytes of signature.
/// let signature = sign_message("sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r", "68656C6C6F")?;
/// assert_eq!(signature.len(), 128);
/// // A secp256k1 key signs a 32-byte hash only.
/// assert!(sign_message("sp5fghtJtpUorTwvof1NpDXAzNwf5", "68656C6C6F").is_err());
/// # Ok::<(), wasm_bindgen::JsValue>(())
/// ```
///
/// # Errors
///
/// A `JsValue` string if `message_hex` is not hex, or holding the
/// [`CoreError`] message if `seed` is not a seed or the key is secp256k1 and
/// the message is not 32 bytes.
#[wasm_bindgen]
pub fn sign_message(seed: &str, message_hex: &str) -> Result<String, JsValue> {
    let message = hex::decode(message_hex)
        .map_err(|e| JsValue::from_str(&format!("invalid message hex: {e}")))?;
    let keypair = crate::crypto::Seed::from_base58(seed)
        .and_then(|s| s.derive_keypair())
        .map_err(to_js_error)?;
    let signature = match keypair.key_type() {
        crate::address::KeyType::Ed25519 => {
            crate::crypto::ed25519::sign(keypair.private_key(), &message)
        }
        crate::address::KeyType::Secp256k1 => {
            crate::crypto::secp256k1::sign(keypair.private_key(), &message)
        }
    }
    .map_err(to_js_error)?;
    Ok(hex::encode_upper(signature))
}
