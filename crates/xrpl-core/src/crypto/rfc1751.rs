//! RFC 1751 keys: a 16-byte seed as twelve short English words, the form
//! `rippled`'s `wallet_propose` returns as `master_key`
//! (`"YOKE DREW DIME DRUM LAIN NIBS DUAL PEA BAIT CHOW JOY FILE"`).
//!
//! Each half of the key (8 bytes) becomes six words of 11 bits: its 64 bits
//! and 2 parity bits (the sum of its 2-bit groups). `rippled` encodes the
//! seed's bytes in reverse order (`Seed.cpp`, `seedAs1751`), and so does this
//! module, so the words match `wallet_propose` byte for byte.
//!
//! The words carry no key type: the same twelve words are the
//! `master_key` of a secp256k1 seed and of an Ed25519 seed. Say which with
//! [`Seed::from_rfc1751`](super::Seed::from_rfc1751).
//!
//! Reading the words back, [`decode`] accepts what `rippled` accepts: any
//! case, any whitespace between words, and `1`, `0`, `5` for `L`, `O`, `S`.
//! It also reads `YOU`, the last short word of the dictionary, which
//! `rippled` 3.4.1 cannot: its dictionary search stops one word short
//! (`RFC1751::etob`), so about one key in 170 that `wallet_propose` writes
//! does not read back there. `wallet_propose` then takes the words as a
//! passphrase and returns a different seed without an error.

use super::wordlists::RFC1751_WORDS;
use crate::CoreError;
use zeroize::{Zeroize, Zeroizing};

#[cfg(not(feature = "std"))]
use alloc::{format, string::String, vec::Vec};

/// The twelve words of `seed`, upper case and separated by single spaces,
/// as `wallet_propose` writes its `master_key`. The string is wiped on drop.
pub fn encode(seed: &[u8; 16]) -> Zeroizing<String> {
    let mut key = Zeroizing::new(*seed);
    key.reverse();
    let mut out = Zeroizing::new(String::with_capacity(12 * 5));
    for half in key.chunks_exact(8) {
        let mut block = 0u64;
        for byte in half {
            block = (block << 8) | u64::from(*byte);
        }
        // 66 bits: the 64 key bits, then the 2 parity bits.
        let mut bits = (u128::from(block) << 2) | u128::from(parity(block));
        for shift in (0..6).rev().map(|n| n * 11) {
            if !out.is_empty() {
                out.push(' ');
            }
            out.push_str(word(bits, shift));
        }
        block.zeroize();
        bits.zeroize();
    }
    out
}

/// The 16-byte seed that `words` encode (twelve words, as [`encode`] writes
/// them; any case and spacing). The bytes are wiped on drop.
///
/// # Errors
///
/// [`CoreError::InvalidSeed`] if there are not twelve words, a word is not
/// in the dictionary, or a half fails its parity check. The error names the
/// word's position, never the word.
pub fn decode(words: &str) -> Result<Zeroizing<[u8; 16]>, CoreError> {
    let invalid = |m: String| CoreError::InvalidSeed(format!("RFC 1751 key: {m}"));
    let list: Vec<&str> = words.split_whitespace().collect();
    if list.len() != 12 {
        return Err(invalid(format!("it has twelve words, not {}", list.len())));
    }
    let mut key = Zeroizing::new([0u8; 16]);
    for ((half, chunk), out) in list
        .chunks_exact(6)
        .enumerate()
        .zip(key.chunks_exact_mut(8))
    {
        let mut bits = 0u128;
        for (n, word) in chunk.iter().enumerate() {
            let position = half * 6 + n + 1;
            let index = lookup(word)
                .ok_or_else(|| invalid(format!("word {position} is not in the dictionary")))?;
            bits = (bits << 11) | u128::from(index);
        }
        let mut block = high_64(bits);
        let check = bits & 3;
        bits.zeroize();
        if u128::from(parity(block)) != check {
            block.zeroize();
            return Err(invalid(format!(
                "words {}-{} fail their parity check: one of them is wrong",
                half * 6 + 1,
                half * 6 + 6
            )));
        }
        out.copy_from_slice(&block.to_be_bytes());
        block.zeroize();
    }
    key.reverse();
    Ok(key)
}

/// The sum of the 32 two-bit groups of `block`, modulo 4.
fn parity(block: u64) -> u64 {
    (0..32).map(|n| (block >> (n * 2)) & 3).sum::<u64>() & 3
}

/// The word for the 11 bits of `bits` starting `shift` bits from the bottom.
#[allow(
    clippy::cast_possible_truncation,
    clippy::indexing_slicing,
    reason = "masked to 11 bits: below 2048, the dictionary's length"
)]
fn word(bits: u128, shift: u32) -> &'static str {
    RFC1751_WORDS[((bits >> shift) & 0x7FF) as usize]
}

/// The 64 key bits of a 66-bit block (key bits, then 2 parity bits).
#[allow(
    clippy::cast_possible_truncation,
    reason = "a 66-bit value shifted right by 2 has 64 bits"
)]
fn high_64(bits: u128) -> u64 {
    (bits >> 2) as u64
}

/// The dictionary index of `word`, read as `rippled` reads it (`standard`):
/// upper case, with `1`, `0`, `5` for `L`, `O`, `S`. Words of one to three
/// letters sit at `0..571`, four-letter words at `571..2048`.
fn lookup(word: &str) -> Option<u16> {
    let bytes = word.as_bytes();
    if bytes.is_empty() || bytes.len() > 4 {
        return None;
    }
    let mut buf = [0u8; 4];
    for (out, byte) in buf.iter_mut().zip(bytes) {
        *out = match byte.to_ascii_uppercase() {
            b'1' => b'L',
            b'0' => b'O',
            b'5' => b'S',
            other => other,
        };
    }
    let standard = core::str::from_utf8(buf.get(..bytes.len())?).ok()?;
    let range = if bytes.len() < 4 { 0..571 } else { 571..2048 };
    let start = range.start;
    let found = RFC1751_WORDS.get(range)?.binary_search(&standard).ok()?;
    u16::try_from(start + found).ok()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn known_answers() {
        let seed: [u8; 16] = hex::decode("00112233445566778899AABBCCDDEEFF")
            .unwrap()
            .try_into()
            .unwrap();
        let words = "YOKE DREW DIME DRUM LAIN NIBS DUAL PEA BAIT CHOW JOY FILE";
        assert_eq!(encode(&seed).as_str(), words);
        assert_eq!(*decode(words).unwrap(), seed);
        assert_eq!(*decode(&words.to_lowercase()).unwrap(), seed);
        assert_eq!(
            *decode(" yoke  drew\tdime drum lain nibs\ndual pea bait chow joy file ").unwrap(),
            seed
        );
    }

    #[test]
    fn rippled_digit_spellings() {
        // `standard` reads 1, 0, 5 as L, O, S: "SILT" may be typed "5I1T".
        let words = "OWLY ANNA KLAN OR TRIO MIKE TEST YOU OUR MACE WAVE SILT";
        let typed = "OWLY ANNA K1AN 0R TRIO MIKE TE5T YOU OUR MACE WAVE 5I1T";
        assert_eq!(*decode(typed).unwrap(), *decode(words).unwrap());
    }

    #[test]
    fn refusals_name_positions_not_words() {
        let words = "YOKE DREW DIME DRUM LAIN NIBS DUAL PEA BAIT CHOW JOY FILE";
        let err = decode("YOKE DREW DIME DRUM LAIN NIBS DUAL PEA BAIT CHOW JOY").unwrap_err();
        assert!(err.to_string().contains("not 11"), "{err}");
        let err = decode(&words.replace("CHOW", "XQZW")).unwrap_err();
        assert!(err.to_string().contains("word 10 "), "{err}");
        assert!(!err.to_string().contains("XQZW"), "{err}");
        let err = decode(&words.replace("CHOW", "CHOWDER")).unwrap_err();
        assert!(err.to_string().contains("word 10 "), "{err}");
        // A changed word gives a different key or fails the parity check of
        // its half (2 bits: about three changes in four are caught).
        let seed = *decode(words).unwrap();
        let mut caught = 0;
        for other in RFC1751_WORDS.iter().filter(|w| w.len() == 4) {
            match decode(&words.replacen("DREW", other, 1)) {
                Ok(key) => assert!(*other == "DREW" || *key != seed),
                Err(err) => {
                    assert!(err.to_string().contains("words 1-6 fail"), "{err}");
                    caught += 1;
                }
            }
        }
        assert_eq!(caught, 1108);
    }
}
