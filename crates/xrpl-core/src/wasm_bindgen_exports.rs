//! WASM bindings for xrpl-core — exposes core functions to JavaScript.
//! Only compiled when the `wasm` feature is enabled.

use wasm_bindgen::prelude::*;

use crate::CoreError;

fn to_js_error(e: CoreError) -> JsValue {
    JsValue::from_str(&e.to_string())
}

/// Encode a transaction JSON string to a hex blob.
#[wasm_bindgen]
pub fn encode_transaction(json_str: &str) -> Result<String, JsValue> {
    let value: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| JsValue::from_str(&format!("invalid JSON: {e}")))?;
    let bytes = crate::codec::encode_transaction_json(&value, false).map_err(to_js_error)?;
    Ok(hex::encode_upper(bytes))
}

/// Encode a transaction JSON string for signing (excludes non-signing fields).
#[wasm_bindgen]
pub fn encode_transaction_for_signing(json_str: &str) -> Result<String, JsValue> {
    let value: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| JsValue::from_str(&format!("invalid JSON: {e}")))?;
    let bytes = crate::codec::encode_transaction_json(&value, true).map_err(to_js_error)?;
    Ok(hex::encode_upper(bytes))
}

/// Decode a hex blob back to a transaction JSON string.
#[wasm_bindgen]
pub fn decode_transaction(hex_str: &str) -> Result<String, JsValue> {
    let bytes =
        hex::decode(hex_str).map_err(|e| JsValue::from_str(&format!("invalid hex: {e}")))?;
    let value = crate::codec::decode_transaction_binary(&bytes).map_err(to_js_error)?;
    serde_json::to_string(&value)
        .map_err(|e| JsValue::from_str(&format!("JSON serialization failed: {e}")))
}

/// Encode a classic XRPL address to its 20-byte AccountID hex.
#[wasm_bindgen]
pub fn address_to_account_id(address: &str) -> Result<String, JsValue> {
    let bytes = crate::address::decode_account_id(address).map_err(to_js_error)?;
    Ok(hex::encode_upper(bytes))
}

/// Encode a 20-byte AccountID hex to a classic XRPL address.
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
    Ok(seed.to_base58())
}

/// Derive the classic address from a seed string.
#[wasm_bindgen]
pub fn seed_to_address(seed: &str) -> Result<String, JsValue> {
    let (seed_bytes, key_type) = crate::address::decode_seed(seed).map_err(to_js_error)?;
    let (_, public_key) = match key_type {
        crate::address::KeyType::Ed25519 => {
            crate::crypto::ed25519::derive_keypair(&seed_bytes).map_err(to_js_error)?
        }
        crate::address::KeyType::Secp256k1 => {
            crate::crypto::secp256k1::derive_keypair(&seed_bytes).map_err(to_js_error)?
        }
    };
    let account_id = crate::crypto::signing::public_key_to_account_id(&public_key);
    Ok(crate::address::encode_account_id(&account_id))
}

/// Sign a message hash with a seed.
#[wasm_bindgen]
pub fn sign_message(seed: &str, message_hex: &str) -> Result<String, JsValue> {
    let message = hex::decode(message_hex)
        .map_err(|e| JsValue::from_str(&format!("invalid message hex: {e}")))?;
    let (seed_bytes, key_type) = crate::address::decode_seed(seed).map_err(to_js_error)?;
    let signature = match key_type {
        crate::address::KeyType::Ed25519 => {
            let (private_key, _) =
                crate::crypto::ed25519::derive_keypair(&seed_bytes).map_err(to_js_error)?;
            crate::crypto::ed25519::sign(&private_key, &message).map_err(to_js_error)?
        }
        crate::address::KeyType::Secp256k1 => {
            let (private_key, _) =
                crate::crypto::secp256k1::derive_keypair(&seed_bytes).map_err(to_js_error)?;
            crate::crypto::secp256k1::sign(&private_key, &message).map_err(to_js_error)?
        }
    };
    Ok(hex::encode_upper(signature))
}
