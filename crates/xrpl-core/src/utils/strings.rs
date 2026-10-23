//! Text ↔ the hex rippled stores it as (`Domain`, `MemoData`, `MemoType`,
//! `URI`, DID and Oracle fields).

use crate::CoreError;

/// UTF-8 text → upper-case hex (`"example.com"` → `"6578616D706C652E636F6D"`).
pub fn str_to_hex(text: &str) -> String {
    hex::encode_upper(text.as_bytes())
}

/// Hex → UTF-8 text. Errors on bad hex or bytes that are not UTF-8.
///
/// # Errors
///
/// [`CoreError::InvalidHex`] if `hex_text` is not hex;
/// [`CoreError::ValidationError`] if the bytes are not UTF-8.
pub fn hex_to_str(hex_text: &str) -> Result<String, CoreError> {
    let bytes = hex::decode(hex_text).map_err(|e| CoreError::InvalidHex(e.to_string()))?;
    String::from_utf8(bytes).map_err(|e| CoreError::ValidationError(format!("not UTF-8: {e}")))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // xrpl.js hexConversion.test.ts.
    #[test]
    fn matches_xrpl_js() {
        assert_eq!(str_to_hex("example.com"), "6578616D706C652E636F6D");
        assert_eq!(hex_to_str("6578616D706C652E636F6D").unwrap(), "example.com");
        assert_eq!(str_to_hex("你好"), "E4BDA0E5A5BD");
        assert_eq!(hex_to_str("e4bda0e5a5bd").unwrap(), "你好");
        assert!(hex_to_str("ZZ").is_err());
        assert!(hex_to_str("FF").is_err());
    }
}
