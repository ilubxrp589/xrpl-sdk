//! Classic addresses (`r…`), X-addresses (XLS-5d: an address with its
//! destination tag) and seeds (`s…`): base58check with the XRP Ledger's
//! alphabet, as `rippled` encodes them.

use crate::CoreError;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

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
#[allow(
    clippy::indexing_slicing,
    clippy::cast_possible_truncation,
    reason = "const context: i < 58 and every alphabet byte < 128"
)]
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
/// The longest base58 string decoded (every XRPL base58 value — address,
/// X-address, seed, public key — is far shorter). Decoding takes time
/// quadratic in the length, so a server cannot make an "address" in a
/// response cost minutes of CPU.
const MAX_BASE58_LEN: usize = 128;

fn base58_decode(s: &str) -> Result<Vec<u8>, CoreError> {
    if s.is_empty() {
        return Ok(Vec::new());
    }
    if s.len() > MAX_BASE58_LEN {
        return Err(CoreError::InvalidBase58(format!(
            "longer than {MAX_BASE58_LEN} characters"
        )));
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
/// Prefix: `0x00`, checksum appended.
pub fn encode_account_id(bytes: &[u8; 20]) -> String {
    let mut payload = Vec::with_capacity(25);
    payload.push(0x00); // account ID prefix
    payload.extend_from_slice(bytes);
    let cksum = checksum(&payload);
    payload.extend_from_slice(&cksum);
    base58_encode(&payload)
}

/// Decode a classic address to a 20-byte account ID.
/// Validates prefix `0x00` and checksum.
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

    if expected_checksum.ct_eq(&actual_checksum).unwrap_u8() != 1 {
        return Err(CoreError::InvalidChecksum {
            expected: expected_checksum,
            got: actual_checksum,
        });
    }

    let mut result = [0u8; 20];
    result.copy_from_slice(&decoded[1..21]);
    Ok(result)
}

/// X-address prefixes (XLS-5d): mainnet addresses start with "X", testnet with "T".
const X_ADDRESS_MAIN: [u8; 2] = [0x05, 0x44];
const X_ADDRESS_TEST: [u8; 2] = [0x04, 0x93];

/// Encode an account ID and optional tag as an X-address (XLS-5d): prefix,
/// account ID, flag (1 = tag present), the tag as 4 little-endian bytes,
/// 4 reserved zero bytes (64-bit tags), then the base58check checksum.
pub fn encode_x_address(account_id: &[u8; 20], tag: Option<u32>, test: bool) -> String {
    let mut payload = Vec::with_capacity(35);
    payload.extend_from_slice(if test {
        &X_ADDRESS_TEST
    } else {
        &X_ADDRESS_MAIN
    });
    payload.extend_from_slice(account_id);
    payload.push(u8::from(tag.is_some()));
    payload.extend_from_slice(&tag.unwrap_or(0).to_le_bytes());
    payload.extend_from_slice(&[0; 4]);
    let cksum = checksum(&payload);
    payload.extend_from_slice(&cksum);
    base58_encode(&payload)
}

/// Decode an X-address to (account ID, tag, is_test). Rejects unknown
/// prefixes and flags, 64-bit tags, and tag bytes without the tag flag.
pub fn decode_x_address(x_address: &str) -> Result<([u8; 20], Option<u32>, bool), CoreError> {
    let decoded: [u8; 35] = base58_decode(x_address)?.try_into().map_err(|d: Vec<u8>| {
        CoreError::InvalidAddress(format!("X-address: expected 35 bytes, got {}", d.len()))
    })?;
    let expected_checksum = checksum(&decoded[..31]);
    let actual_checksum: [u8; 4] = [decoded[31], decoded[32], decoded[33], decoded[34]];
    if expected_checksum.ct_eq(&actual_checksum).unwrap_u8() != 1 {
        return Err(CoreError::InvalidChecksum {
            expected: expected_checksum,
            got: actual_checksum,
        });
    }
    let test = match [decoded[0], decoded[1]] {
        X_ADDRESS_MAIN => false,
        X_ADDRESS_TEST => true,
        [a, b] => {
            return Err(CoreError::InvalidAddress(format!(
                "X-address: unknown prefix {a:02X}{b:02X}"
            )))
        }
    };
    if decoded[27..31] != [0; 4] {
        return Err(CoreError::InvalidAddress(
            "X-address: 64-bit tags are not supported".to_string(),
        ));
    }
    let tag_bytes = [decoded[23], decoded[24], decoded[25], decoded[26]];
    let tag = match decoded[22] {
        0 if tag_bytes == [0; 4] => None,
        0 => {
            return Err(CoreError::InvalidAddress(
                "X-address: tag bytes set without the tag flag".to_string(),
            ))
        }
        1 => Some(u32::from_le_bytes(tag_bytes)),
        flag => {
            return Err(CoreError::InvalidAddress(format!(
                "X-address: unknown flag {flag}"
            )))
        }
    };
    let mut account_id = [0u8; 20];
    account_id.copy_from_slice(&decoded[2..22]);
    Ok((account_id, tag, test))
}

/// Classic address (+ optional tag) → X-address.
pub fn classic_to_x_address(
    classic: &str,
    tag: Option<u32>,
    test: bool,
) -> Result<String, CoreError> {
    Ok(encode_x_address(&decode_account_id(classic)?, tag, test))
}

/// X-address → (classic address, tag, is_test).
pub fn x_address_to_classic(x_address: &str) -> Result<(String, Option<u32>, bool), CoreError> {
    let (account_id, tag, test) = decode_x_address(x_address)?;
    Ok((encode_account_id(&account_id), tag, test))
}

/// Key type for seed encoding/decoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyType {
    /// Ed25519: `rippled`'s default for new keys.
    Ed25519,
    /// secp256k1 (ECDSA).
    Secp256k1,
}

/// Encode a 16-byte seed to base58check.
/// Ed25519 prefix: [0x01, 0xE1, 0x4B] → starts with "sEd"
/// Secp256k1 prefix: `0x21` → starts with "s"
pub fn encode_seed(bytes: &[u8; 16], key_type: KeyType) -> String {
    let prefix: &[u8] = match key_type {
        KeyType::Ed25519 => &[0x01, 0xE1, 0x4B],
        KeyType::Secp256k1 => &[0x21],
    };

    // The payload holds the seed: wiped when done.
    let mut payload = zeroize::Zeroizing::new(Vec::with_capacity(prefix.len() + 16 + 4));
    payload.extend_from_slice(prefix);
    payload.extend_from_slice(bytes);
    let cksum = checksum(&payload);
    payload.extend_from_slice(&cksum);
    base58_encode(&payload)
}

/// Decode a base58check seed string to 16 raw bytes + key type.
///
/// Errors say only what kind of mistake the string has: never a character
/// of it or a checksum computed from it, which would put part of a secret
/// into a log.
pub fn decode_seed(seed: &str) -> Result<([u8; 16], KeyType), CoreError> {
    decode_seed_bytes(seed).map_err(|e| {
        CoreError::InvalidSeed(
            match e {
                CoreError::InvalidBase58(_) => "not base58 (a character outside the XRPL alphabet)",
                CoreError::InvalidChecksum { .. } => "the checksum does not match (a typo?)",
                _ => "not a seed (wrong length or prefix)",
            }
            .to_string(),
        )
    })
}

fn decode_seed_bytes(seed: &str) -> Result<([u8; 16], KeyType), CoreError> {
    let decoded = zeroize::Zeroizing::new(base58_decode(seed)?);

    // Try Ed25519 prefix first (3 bytes + 16 bytes + 4 bytes = 23)
    if let Ok(d) = <[u8; 23]>::try_from(decoded.as_slice()) {
        if d[0] == 0x01 && d[1] == 0xE1 && d[2] == 0x4B {
            let payload = &d[..19];
            let expected_cksum = checksum(payload);
            let actual_cksum: [u8; 4] = [d[19], d[20], d[21], d[22]];
            if expected_cksum.ct_eq(&actual_cksum).unwrap_u8() != 1 {
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
            if expected_cksum.ct_eq(&actual_cksum).unwrap_u8() != 1 {
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

    // xrpl.js ripple-address-codec test/index.test.ts: classic, tag, mainnet X, testnet X.
    const X_ADDRESS_CASES: &[(&str, Option<u32>, &str, &str)] = &[
        (
            "r9cZA1mLK5R5Am25ArfXFmqgNwjZgnfk59",
            None,
            "X7AcgcsBL6XDcUb289X4mJ8djcdyKaB5hJDWMArnXr61cqZ",
            "T719a5UwUCnEs54UsxG9CJYYDhwmFCqkr7wxCcNcfZ6p5GZ",
        ),
        (
            "r9cZA1mLK5R5Am25ArfXFmqgNwjZgnfk59",
            Some(1),
            "X7AcgcsBL6XDcUb289X4mJ8djcdyKaGZMhc9YTE92ehJ2Fu",
            "T719a5UwUCnEs54UsxG9CJYYDhwmFCvbJNZbi37gBGkRkbE",
        ),
        (
            "r9cZA1mLK5R5Am25ArfXFmqgNwjZgnfk59",
            Some(14),
            "X7AcgcsBL6XDcUb289X4mJ8djcdyKaGo2K5VpXpmCqbV2gS",
            "T719a5UwUCnEs54UsxG9CJYYDhwmFCvqXVCALUGJGSbNV3x",
        ),
        (
            "r9cZA1mLK5R5Am25ArfXFmqgNwjZgnfk59",
            Some(11747),
            "X7AcgcsBL6XDcUb289X4mJ8djcdyKaLFuhLRuNXPrDeJd9A",
            "T719a5UwUCnEs54UsxG9CJYYDhwmFCziiNHtUukubF2Mg6t",
        ),
        (
            "rLczgQHxPhWtjkaQqn3Q6UM8AbRbbRvs5K",
            None,
            "XVZVpQj8YSVpNyiwXYSqvQoQqgBttTxAZwMcuJd4xteQHyt",
            "TVVrSWtmQQssgVcmoMBcFQZKKf56QscyWLKnUyiuZW8ALU4",
        ),
        (
            "rpZc4mVfWUif9CRoHRKKcmhu1nx2xktxBo",
            None,
            "X7YenJqxv3L66CwhBSfd3N8RzGXxYqPopMGMsCcpho79rex",
            "T77wVQzA8ntj9wvCTNiQpNYLT5hmhRsFyXDoMLqYC4BzQtV",
        ),
        (
            "rpZc4mVfWUif9CRoHRKKcmhu1nx2xktxBo",
            Some(58),
            "X7YenJqxv3L66CwhBSfd3N8RzGXxYqV56ZkTCa9UCzgaao1",
            "T77wVQzA8ntj9wvCTNiQpNYLT5hmhR9kej6uxm4jGcQD7rZ",
        ),
        (
            "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW",
            Some(23480),
            "X7d3eHCXzwBeWrZec1yT24iZerQjYL8m8zCJ16ACxu1BrBY",
            "T7YChPFWifjCAXLEtg5N74c7fSAYsvSokwcmBPBUZWhxH5P",
        ),
        (
            "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW",
            Some(11747),
            "X7d3eHCXzwBeWrZec1yT24iZerQjYLo2CJf8oVC5CMWey5m",
            "T7YChPFWifjCAXLEtg5N74c7fSAYsvTcc7nEfwuEEvn5Q4w",
        ),
        (
            "rGWrZyQqhTp9Xu7G5Pkayo7bXjH4k4QYpf",
            None,
            "XVLhHMPHU98es4dbozjVtdWzVrDjtV5fdx1mHp98tDMoQXb",
            "TVE26TYGhfLC7tQDno7G8dGtxSkYQn49b3qD26PK7FcGSKE",
        ),
        (
            "rGWrZyQqhTp9Xu7G5Pkayo7bXjH4k4QYpf",
            Some(0),
            "XVLhHMPHU98es4dbozjVtdWzVrDjtV8AqEL4xcZj5whKbmc",
            "TVE26TYGhfLC7tQDno7G8dGtxSkYQnSy8RHqGHoGJ59spi2",
        ),
        (
            "rGWrZyQqhTp9Xu7G5Pkayo7bXjH4k4QYpf",
            Some(1),
            "XVLhHMPHU98es4dbozjVtdWzVrDjtV8xvjGQTYPiAx6gwDC",
            "TVE26TYGhfLC7tQDno7G8dGtxSkYQnSz1uDimDdPYXzSpyw",
        ),
        (
            "rGWrZyQqhTp9Xu7G5Pkayo7bXjH4k4QYpf",
            Some(2),
            "XVLhHMPHU98es4dbozjVtdWzVrDjtV8zpDURx7DzBCkrQE7",
            "TVE26TYGhfLC7tQDno7G8dGtxSkYQnTryP9tG9TW8GeMBmd",
        ),
        (
            "rGWrZyQqhTp9Xu7G5Pkayo7bXjH4k4QYpf",
            Some(32),
            "XVLhHMPHU98es4dbozjVtdWzVrDjtVoYiC9UvKfjKar4LJe",
            "TVE26TYGhfLC7tQDno7G8dGtxSkYQnT2oqaCDzMEuCDAj1j",
        ),
        (
            "rGWrZyQqhTp9Xu7G5Pkayo7bXjH4k4QYpf",
            Some(276),
            "XVLhHMPHU98es4dbozjVtdWzVrDjtVoKj3MnFGMXEFMnvJV",
            "TVE26TYGhfLC7tQDno7G8dGtxSkYQnTMgJJYfAbsiPsc6Zg",
        ),
        (
            "rGWrZyQqhTp9Xu7G5Pkayo7bXjH4k4QYpf",
            Some(65591),
            "XVLhHMPHU98es4dbozjVtdWzVrDjtVozpjdhPQVdt3ghaWw",
            "TVE26TYGhfLC7tQDno7G8dGtxSkYQn7ryu2W6njw7mT1jmS",
        ),
        (
            "rGWrZyQqhTp9Xu7G5Pkayo7bXjH4k4QYpf",
            Some(16781933),
            "XVLhHMPHU98es4dbozjVtdWzVrDjtVqrDUk2vDpkTjPsY73",
            "TVE26TYGhfLC7tQDno7G8dGtxSkYQnVsw45sDtGHhLi27Qa",
        ),
        (
            "rGWrZyQqhTp9Xu7G5Pkayo7bXjH4k4QYpf",
            Some(4294967294),
            "XVLhHMPHU98es4dbozjVtdWzVrDjtV1kAsixQTdMjbWi39u",
            "TVE26TYGhfLC7tQDno7G8dGtxSkYQnX8tDFQ53itLNqs6vU",
        ),
        (
            "rGWrZyQqhTp9Xu7G5Pkayo7bXjH4k4QYpf",
            Some(4294967295),
            "XVLhHMPHU98es4dbozjVtdWzVrDjtV18pX8yuPT7y4xaEHi",
            "TVE26TYGhfLC7tQDno7G8dGtxSkYQnXoy6kSDh6rZzApc69",
        ),
        (
            "rPEPPER7kfTD9w2To4CQk6UCfuHM9c6GDY",
            None,
            "XV5sbjUmgPpvXv4ixFWZ5ptAYZ6PD2gYsjNFQLKYW33DzBm",
            "TVd2rqMkYL2AyS97NdELcpeiprNBjwLZzuUG5rZnaewsahi",
        ),
        (
            "rPEPPER7kfTD9w2To4CQk6UCfuHM9c6GDY",
            Some(0),
            "XV5sbjUmgPpvXv4ixFWZ5ptAYZ6PD2m4Er6SnvjVLpMWPjR",
            "TVd2rqMkYL2AyS97NdELcpeiprNBjwRQUBetPbyrvXSTuxU",
        ),
        (
            "rPEPPER7kfTD9w2To4CQk6UCfuHM9c6GDY",
            Some(13371337),
            "XV5sbjUmgPpvXv4ixFWZ5ptAYZ6PD2qwGkhgc48zzcx6Gkr",
            "TVd2rqMkYL2AyS97NdELcpeiprNBjwVUDvp3vhpXbNhLwJi",
        ),
    ];

    #[test]
    fn x_addresses_match_xrpl_js() {
        for &(classic, tag, main, test) in X_ADDRESS_CASES {
            assert_eq!(
                classic_to_x_address(classic, tag, false).unwrap(),
                main,
                "{classic} {tag:?}"
            );
            assert_eq!(
                classic_to_x_address(classic, tag, true).unwrap(),
                test,
                "{classic} {tag:?}"
            );
            assert_eq!(
                x_address_to_classic(main).unwrap(),
                (classic.to_string(), tag, false)
            );
            assert_eq!(
                x_address_to_classic(test).unwrap(),
                (classic.to_string(), tag, true)
            );
        }
    }

    #[test]
    fn x_address_rejects_what_xls5d_does_not_define() {
        let raw = |prefix: [u8; 2], flag: u8, tail: [u8; 8]| {
            let mut p = prefix.to_vec();
            p.extend_from_slice(&[0x11; 20]);
            p.push(flag);
            p.extend_from_slice(&tail);
            let c = checksum(&p);
            p.extend_from_slice(&c);
            base58_encode(&p)
        };
        assert!(decode_x_address(&raw(X_ADDRESS_MAIN, 1, [1, 0, 0, 0, 0, 0, 0, 0])).is_ok());
        assert!(
            decode_x_address(&raw(X_ADDRESS_MAIN, 1, [1, 0, 0, 0, 1, 0, 0, 0])).is_err(),
            "64-bit tag"
        );
        assert!(
            decode_x_address(&raw(X_ADDRESS_MAIN, 0, [1, 0, 0, 0, 0, 0, 0, 0])).is_err(),
            "tag bytes, no flag"
        );
        assert!(
            decode_x_address(&raw(X_ADDRESS_MAIN, 2, [0; 8])).is_err(),
            "unknown flag"
        );
        assert!(
            decode_x_address(&raw([0x05, 0x45], 0, [0; 8])).is_err(),
            "unknown prefix"
        );
        let good = X_ADDRESS_CASES[0].2;
        let bad_checksum = format!(
            "{}{}",
            &good[..good.len() - 1],
            if good.ends_with('Z') { 'Y' } else { 'Z' }
        );
        assert!(decode_x_address(&bad_checksum).is_err(), "checksum");
        assert!(
            decode_x_address("rGWrZyQqhTp9Xu7G5Pkayo7bXjH4k4QYpf").is_err(),
            "classic address"
        );
    }

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
        // Generate random 16 bytes and round-trip through encode/decode
        use rand::RngCore;
        let mut bytes = [0u8; 16];
        rand::rng().fill_bytes(&mut bytes);
        let encoded = encode_seed(&bytes, KeyType::Secp256k1);
        assert!(encoded.starts_with('s'));
        assert!(!encoded.starts_with("sEd"));
        let (decoded_bytes, key_type) = decode_seed(&encoded).unwrap();
        assert_eq!(key_type, KeyType::Secp256k1);
        assert_eq!(decoded_bytes, bytes);
        // Double round-trip
        let re_encoded = encode_seed(&decoded_bytes, KeyType::Secp256k1);
        assert_eq!(re_encoded, encoded);
    }

    #[test]
    fn seed_ed25519_roundtrip() {
        // Generate random 16 bytes and round-trip through encode/decode
        use rand::RngCore;
        let mut bytes = [0u8; 16];
        rand::rng().fill_bytes(&mut bytes);
        let encoded = encode_seed(&bytes, KeyType::Ed25519);
        assert!(encoded.starts_with("sEd"));
        let (decoded_bytes, key_type) = decode_seed(&encoded).unwrap();
        assert_eq!(key_type, KeyType::Ed25519);
        assert_eq!(decoded_bytes, bytes);
        // Double round-trip
        let re_encoded = encode_seed(&decoded_bytes, KeyType::Ed25519);
        assert_eq!(re_encoded, encoded);
    }

    #[test]
    fn seed_detect_key_type() {
        use rand::RngCore;
        let mut bytes = [0u8; 16];
        rand::rng().fill_bytes(&mut bytes);

        // secp256k1 seed starts with 's' but not 'sEd'
        let secp_encoded = encode_seed(&bytes, KeyType::Secp256k1);
        let (_, kt) = decode_seed(&secp_encoded).unwrap();
        assert_eq!(kt, KeyType::Secp256k1);

        // ed25519 seed starts with 'sEd'
        let ed_encoded = encode_seed(&bytes, KeyType::Ed25519);
        let (_, kt) = decode_seed(&ed_encoded).unwrap();
        assert_eq!(kt, KeyType::Ed25519);
    }

    /// A seed with a typo or a stray character is refused without its
    /// characters, or a checksum computed from it, in the message.
    #[test]
    fn seed_errors_carry_no_part_of_the_seed() {
        for (bad, why) in [
            ("sp5fghtJtpUorTwvof1NpDXAzNwf6", "checksum"),
            ("sp5fghtJtpUorTwvof1NpDXAzNwf0", "base58"),
            ("sp5fghtJtpUorTwvof1NpDXAz", "not a seed"),
        ] {
            let message = decode_seed(bad).unwrap_err().to_string();
            assert!(message.contains(why), "{message}");
            assert!(
                !message.contains("sp5f") && !message.contains('[') && !message.contains('0'),
                "{message}"
            );
        }
    }

    #[test]
    fn a_huge_string_is_refused_before_decoding() {
        let huge = "p".repeat(200_000);
        let started = std::time::Instant::now();
        assert!(decode_account_id(&huge).is_err());
        assert!(started.elapsed() < std::time::Duration::from_millis(50));
    }
}
