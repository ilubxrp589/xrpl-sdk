//! WASM bindings for xrpl-core — exposes core functions to JavaScript.
//! Only compiled when the `wasm` feature is enabled.

use wasm_bindgen::prelude::*;

use crate::CoreError;

fn to_js_error(e: CoreError) -> JsValue {
    JsValue::from_str(&e.to_string())
}

/// Encode a transaction JSON string to a hex blob.
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
    let bytes = crate::codec::encode_transaction_json(&value, false).map_err(to_js_error)?;
    Ok(hex::encode_upper(bytes))
}

/// Encode a transaction JSON string for signing (excludes non-signing fields).
///
/// # Errors
///
/// A `JsValue` string if `json_str` is not JSON, or holding the [`CoreError`]
/// message if
/// [`encode_transaction_json`](crate::codec::encode_transaction_json) refuses
/// the transaction.
#[wasm_bindgen]
pub fn encode_transaction_for_signing(json_str: &str) -> Result<String, JsValue> {
    let value: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| JsValue::from_str(&format!("invalid JSON: {e}")))?;
    let bytes = crate::codec::encode_transaction_json(&value, true).map_err(to_js_error)?;
    Ok(hex::encode_upper(bytes))
}

/// Decode a hex blob back to a transaction JSON string.
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
#[wasm_bindgen]
pub fn is_valid_address(address: &str) -> bool {
    crate::address::decode_account_id(address).is_ok()
}

/// Generate a new random wallet seed (base58check encoded).
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
