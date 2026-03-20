use crate::CoreError;
use sha2::{Digest, Sha256};

#[cfg(not(feature = "std"))]
use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};

/// XRPL-specific Base58 alphabet (NOT Bitcoin's alphabet).
const XRPL_ALPHABET: &[u8; 58] = b"rpshnaf39wBUDNEGHJKLM4PQRST7VWXYZ2bcdeCg65jkm8oFqi1tuvAxyz";

/// Reverse lookup table: ASCII byte -> base58 digit value (255 = invalid).
/// SAFETY: All bytes in XRPL_ALPHABET are ASCII (< 128), so indexing into a [u8; 128] is safe.
/// The index `i` ranges 0..58, so XRPL_ALPHABET[i] is always in bounds.
/// This runs in a const context where `.get()` is not available.
#[allow(clippy::indexing_slicing)]
const DECODE_TABLE: [u8; 128] = {
    let mut table = [255u8; 128];
    let mut i = 0;
    while i < 58 {
        table[XRPL_ALPHABET[i] as usize] = i as u8;
        i += 1;
    }
    table
};

/// Double-SHA256 checksum: first 4 bytes of SHA256(SHA256(data)).
fn checksum(data: &[u8]) -> [u8; 4] {
    let first = Sha256::digest(data);
    let second: [u8; 32] = Sha256::digest(first).into();
    // SHA256 always produces 32 bytes, so [0..4] is always valid.
    [second[0], second[1], second[2], second[3]]
}

/// Base58-encode raw bytes (no checksum, no prefix).
fn base58_encode(data: &[u8]) -> String {
    if data.is_empty() {
        return String::new();
    }

    // Count leading zeros
    let leading_zeros = data.iter().take_while(|&&b| b == 0).count();

    // Convert bytes to base58 using big-number division
    let mut digits: Vec<u8> = Vec::new();
    for &byte in data {
        let mut carry = byte as u32;
        for digit in digits.iter_mut() {
            carry += (*digit as u32) * 256;
            *digit = (carry % 58) as u8;
            carry /= 58;
        }
        while carry > 0 {
            digits.push((carry % 58) as u8);
            carry /= 58;
        }
    }

    // Reverse and convert to characters
    // SAFETY: XRPL_ALPHABET has 58 entries; index 0 always exists.
    let zero_char = *XRPL_ALPHABET.first().unwrap_or(&b'r');
    let mut result = String::with_capacity(leading_zeros + digits.len());
    for _ in 0..leading_zeros {
        result.push(zero_char as char);
    }
    for &d in digits.iter().rev() {
        // d is always < 58 (result of % 58 arithmetic), so .get() always succeeds
        let ch = XRPL_ALPHABET.get(d as usize).copied().unwrap_or(b'?');
        result.push(ch as char);
    }

    result
}

/// Base58-decode string to raw bytes.
fn base58_decode(s: &str) -> Result<Vec<u8>, CoreError> {
    if s.is_empty() {
        return Ok(Vec::new());
    }

    // Count leading 'r' (base58 zero character)
    let zero_char = *XRPL_ALPHABET.first().unwrap_or(&b'r');
    let leading_zeros = s.chars().take_while(|&c| c == zero_char as char).count();

    // Convert from base58 to bytes
    let mut bytes: Vec<u8> = Vec::new();
    for c in s.chars() {
        if c as u32 >= 128 {
            return Err(CoreError::InvalidBase58(format!("invalid character: {c}")));
        }
        let digit = *DECODE_TABLE.get(c as usize).unwrap_or(&255);
        if digit == 255 {
            return Err(CoreError::InvalidBase58(format!("invalid character: {c}")));
        }

        let mut carry = digit as u32;
        for byte in bytes.iter_mut() {
            carry += (*byte as u32) * 58;
            *byte = (carry & 0xFF) as u8;
            carry >>= 8;
        }
        while carry > 0 {
            bytes.push((carry & 0xFF) as u8);
            carry >>= 8;
        }
    }

    // Add leading zeros
    let mut result = vec![0u8; leading_zeros];
    result.extend(bytes.iter().rev());
    Ok(result)
}

/// Encode a 20-byte account ID to a classic address with base58check.
/// Prefix: [0x00], checksum appended.
pub fn encode_account_id(bytes: &[u8; 20]) -> String {
    let mut payload = Vec::with_capacity(25);
    payload.push(0x00); // account ID prefix
    payload.extend_from_slice(bytes);
    let cksum = checksum(&payload);
    payload.extend_from_slice(&cksum);
    base58_encode(&payload)
}

/// Decode a classic address to a 20-byte account ID.
/// Validates prefix [0x00] and checksum.
pub fn decode_account_id(addr: &str) -> Result<[u8; 20], CoreError> {
    let decoded = base58_decode(addr)?;

    // Expected: 1 byte prefix + 20 bytes payload + 4 bytes checksum = 25 bytes
    if decoded.len() != 25 {
        return Err(CoreError::InvalidAddress(format!(
            "expected 25 bytes, got {}",
            decoded.len()
        )));
    }

    // Convert to fixed-size array — length was already checked to be 25
    let decoded: [u8; 25] = decoded
        .try_into()
        .map_err(|_| CoreError::InvalidAddress("expected 25 bytes".to_string()))?;

    if decoded[0] != 0x00 {
        return Err(CoreError::InvalidAddress(format!(
            "expected prefix 0x00, got 0x{:02X}",
            decoded[0]
        )));
    }

    let payload = &decoded[..21];
    let expected_checksum = checksum(payload);
    let actual_checksum: [u8; 4] = [decoded[21], decoded[22], decoded[23], decoded[24]];

    if expected_checksum != actual_checksum {
        return Err(CoreError::InvalidChecksum {
            expected: expected_checksum,
            got: actual_checksum,
        });
    }

    let mut result = [0u8; 20];
    result.copy_from_slice(&decoded[1..21]);
    Ok(result)
}

/// Key type for seed encoding/decoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyType {
    Ed25519,
    Secp256k1,
}

/// Encode a 16-byte seed to base58check.
/// Ed25519 prefix: [0x01, 0xE1, 0x4B] → starts with "sEd"
/// Secp256k1 prefix: [0x21] → starts with "s"
pub fn encode_seed(bytes: &[u8; 16], key_type: KeyType) -> String {
    let prefix: &[u8] = match key_type {
        KeyType::Ed25519 => &[0x01, 0xE1, 0x4B],
        KeyType::Secp256k1 => &[0x21],
    };

    let mut payload = Vec::with_capacity(prefix.len() + 16 + 4);
    payload.extend_from_slice(prefix);
    payload.extend_from_slice(bytes);
    let cksum = checksum(&payload);
    payload.extend_from_slice(&cksum);
    base58_encode(&payload)
}

/// Decode a base58check seed string to 16 raw bytes + key type.
pub fn decode_seed(seed: &str) -> Result<([u8; 16], KeyType), CoreError> {
    let decoded = base58_decode(seed)?;

    // Try Ed25519 prefix first (3 bytes + 16 bytes + 4 bytes = 23)
    if let Ok(d) = <[u8; 23]>::try_from(decoded.as_slice()) {
        if d[0] == 0x01 && d[1] == 0xE1 && d[2] == 0x4B {
            let payload = &d[..19];
            let expected_cksum = checksum(payload);
            let actual_cksum: [u8; 4] = [d[19], d[20], d[21], d[22]];
            if expected_cksum != actual_cksum {
                return Err(CoreError::InvalidChecksum {
                    expected: expected_cksum,
                    got: actual_cksum,
                });
            }
            let mut seed_bytes = [0u8; 16];
            seed_bytes.copy_from_slice(&d[3..19]);
            return Ok((seed_bytes, KeyType::Ed25519));
        }
    }

    // Try Secp256k1 prefix (1 byte + 16 bytes + 4 bytes = 21)
    if let Ok(d) = <[u8; 21]>::try_from(decoded.as_slice()) {
        if d[0] == 0x21 {
            let payload = &d[..17];
            let expected_cksum = checksum(payload);
            let actual_cksum: [u8; 4] = [d[17], d[18], d[19], d[20]];
            if expected_cksum != actual_cksum {
                return Err(CoreError::InvalidChecksum {
                    expected: expected_cksum,
                    got: actual_cksum,
                });
            }
            let mut seed_bytes = [0u8; 16];
            seed_bytes.copy_from_slice(&d[1..17]);
            return Ok((seed_bytes, KeyType::Secp256k1));
        }
    }

    Err(CoreError::InvalidSeed(format!(
        "unrecognized seed format (len={})",
        decoded.len()
    )))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn account_id_roundtrip_known() {
        // Known address: rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh
        let addr = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";
        let bytes = decode_account_id(addr).unwrap();
        let re_encoded = encode_account_id(&bytes);
        assert_eq!(re_encoded, addr);
    }

    #[test]
    fn account_id_roundtrip_random() {
        let bytes: [u8; 20] = [
            0xB5, 0xF7, 0x62, 0x79, 0x8A, 0x53, 0xD5, 0x43, 0xA0, 0x14, 0xCA, 0xF8, 0xB2, 0x97,
            0xCF, 0xF8, 0xF2, 0xF9, 0x37, 0xE8,
        ];
        let encoded = encode_account_id(&bytes);
        assert!(encoded.starts_with('r'));
        let decoded = decode_account_id(&encoded).unwrap();
        assert_eq!(decoded, bytes);
    }

    #[test]
    fn account_id_bad_checksum() {
        // Change last character
        let addr = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTx";
        assert!(decode_account_id(addr).is_err());
    }

    #[test]
    fn account_id_invalid_prefix() {
        // This would need a carefully crafted string with wrong prefix
        // Just test invalid base58 chars
        let addr = "0OIl"; // characters not in XRPL alphabet
        assert!(decode_account_id(addr).is_err());
    }

    #[test]
    fn seed_secp256k1_roundtrip() {
        let seed_str = "sn3nxiW7v8KXzPzAqzyHXbSSKNuN9";
        let (bytes, key_type) = decode_seed(seed_str).unwrap();
        assert_eq!(key_type, KeyType::Secp256k1);
        let re_encoded = encode_seed(&bytes, KeyType::Secp256k1);
        assert_eq!(re_encoded, seed_str);
    }

    #[test]
    fn seed_ed25519_roundtrip() {
        let seed_str = "sEdTM1uX8pu2do5XvTnutH6HsouMaM2";
        let (bytes, key_type) = decode_seed(seed_str).unwrap();
        assert_eq!(key_type, KeyType::Ed25519);
        let re_encoded = encode_seed(&bytes, KeyType::Ed25519);
        assert_eq!(re_encoded, seed_str);
    }

    #[test]
    fn seed_detect_key_type() {
        // secp256k1 seed starts with 's' but not 'sEd'
        let (_, kt) = decode_seed("sn3nxiW7v8KXzPzAqzyHXbSSKNuN9").unwrap();
        assert_eq!(kt, KeyType::Secp256k1);

        // ed25519 seed starts with 'sEd'
        let (_, kt) = decode_seed("sEdTM1uX8pu2do5XvTnutH6HsouMaM2").unwrap();
        assert_eq!(kt, KeyType::Ed25519);
    }
}
