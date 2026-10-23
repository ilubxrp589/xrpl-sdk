//! Secret numbers (XLS-12): a 16-byte seed as eight blocks of six digits,
//! the form Xaman shows for an account's secret
//! (`"399150 474506 009147 088773 432160 282843 253738 605430"`).
//!
//! Block `n` (counting from 0) holds two bytes of the seed: their value as
//! five digits (`00000`–`65535`), then a check digit,
//! `value × (2n + 1) mod 9`. The same algorithm as `@xrplf/secret-numbers`,
//! which xrpl.js uses.
//!
//! The numbers carry no key type. Xaman makes secp256k1 accounts from them,
//! so [`Seed::from_secret_numbers`](super::Seed::from_secret_numbers) takes
//! the key type and the wallet helpers default to secp256k1, as xrpl.js does.

use crate::CoreError;
use zeroize::{Zeroize, Zeroizing};

#[cfg(not(feature = "std"))]
use alloc::{format, string::String};

/// The eight blocks of `seed`, separated by single spaces. The string is
/// wiped on drop.
pub fn encode(seed: &[u8; 16]) -> Zeroizing<String> {
    let mut out = Zeroizing::new(String::with_capacity(8 * 7));
    for (n, pair) in (0u32..).zip(seed.chunks_exact(2)) {
        let mut value = pair
            .iter()
            .fold(0u32, |acc, byte| (acc << 8) | u32::from(*byte));
        let mut digits = [b'0'; 6];
        let mut rest = value;
        for digit in digits.iter_mut().take(5).rev() {
            *digit = digit_char(rest % 10);
            rest /= 10;
        }
        if let Some(last) = digits.last_mut() {
            *last = digit_char(check_digit(n, value));
        }
        if n > 0 {
            out.push(' ');
        }
        out.extend(digits.iter().map(|d| char::from(*d)));
        value.zeroize();
        digits.zeroize();
    }
    out
}

/// The 16-byte seed that `text` holds: 48 digits in eight blocks of six.
/// Every character that is not a digit is skipped, so spaces, dashes, line
/// breaks and Xaman's row letters (`A 399150 B 474506 …`) may stay. The
/// bytes are wiped on drop.
///
/// # Errors
///
/// [`CoreError::InvalidSeed`] if there are not 48 digits, a block's value is
/// above 65535, or a block's check digit does not match. The error names the
/// block, never its digits.
pub fn decode(text: &str) -> Result<Zeroizing<[u8; 16]>, CoreError> {
    let invalid = |m: String| CoreError::InvalidSeed(format!("secret numbers: {m}"));
    let mut digits = Zeroizing::new([0u32; 48]);
    let mut count = 0usize;
    for c in text.chars() {
        if c.is_ascii_digit() {
            if let Some(slot) = digits.get_mut(count) {
                *slot = u32::from(c) - u32::from('0');
            }
            count += 1;
        }
    }
    if count != 48 {
        return Err(invalid(format!(
            "there are 48 digits (eight blocks of six), not {count}"
        )));
    }
    let mut seed = Zeroizing::new([0u8; 16]);
    for ((n, block), out) in (0u32..)
        .zip(digits.chunks_exact(6))
        .zip(seed.chunks_exact_mut(2))
    {
        let label = n + 1;
        let &[d0, d1, d2, d3, d4, check] = block else {
            return Err(invalid(format!("block {label} is not six digits")));
        };
        let mut value = [d0, d1, d2, d3, d4]
            .iter()
            .fold(0u32, |acc, d| acc * 10 + d);
        if value > 0xFFFF {
            value.zeroize();
            return Err(invalid(format!(
                "block {label} is above 65535, so it is not a secret number"
            )));
        }
        if check_digit(n, value) != check {
            value.zeroize();
            return Err(invalid(format!(
                "block {label} does not match its check digit (its last digit): \
                 a digit in it is wrong"
            )));
        }
        let [_, _, high, low] = value.to_be_bytes();
        out.copy_from_slice(&[high, low]);
        value.zeroize();
    }
    Ok(seed)
}

/// The check digit of block `n` holding `value`: `value × (2n + 1) mod 9`.
fn check_digit(n: u32, value: u32) -> u32 {
    value * (2 * n + 1) % 9
}

/// The ASCII digit for `d % 10`.
fn digit_char(d: u32) -> u8 {
    b'0' + u8::try_from(d % 10).unwrap_or_default()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    // The xrpl.js vector; `rippled`'s `wallet_propose` with this seed gives
    // the public keys the wallet tests check.
    const NUMBERS: &str = "399150 474506 009147 088773 432160 282843 253738 605430";
    const SEED_HEX: &str = "9BEBB95A039222ADA8D06E7C631DEC7F";

    #[test]
    fn known_answer() {
        let seed: [u8; 16] = hex::decode(SEED_HEX).unwrap().try_into().unwrap();
        assert_eq!(encode(&seed).as_str(), NUMBERS);
        assert_eq!(*decode(NUMBERS).unwrap(), seed);
        assert_eq!(
            encode(&[0; 16]).as_str(),
            "000000 000000 000000 000000 000000 000000 000000 000000"
        );
        // 65535 × (2n + 1) mod 9 = 6, 0, 3, 6, 0, 3, 6, 0
        assert_eq!(
            encode(&[0xFF; 16]).as_str(),
            "655356 655350 655353 655356 655350 655353 655356 655350"
        );
        assert_eq!(
            *decode("655356 655350 655353 655356 655350 655353 655356 655350").unwrap(),
            [0xFF; 16]
        );
    }

    #[test]
    fn separators_are_skipped() {
        let expected = *decode(NUMBERS).unwrap();
        for text in [
            "399150474506009147088773432160282843253738605430",
            "A 399150\nB 474506\nC 009147\nD 088773\nE 432160\nF 282843\nG 253738\nH 605430",
            "399150-474506-009147-088773-432160-282843-253738-605430",
        ] {
            assert_eq!(*decode(text).unwrap(), expected, "{text}");
        }
    }

    #[test]
    fn round_trip() {
        for i in 0u8..=255 {
            let seed: [u8; 16] = core::array::from_fn(|j| {
                i.wrapping_mul(31)
                    .wrapping_add(u8::try_from(j).unwrap() * 17)
            });
            assert_eq!(*decode(&encode(&seed)).unwrap(), seed);
        }
    }

    #[test]
    fn refusals_name_blocks_not_digits() {
        let err = decode("399150 474506 009147").unwrap_err();
        assert!(err.to_string().contains("not 18"), "{err}");
        // A changed digit breaks its block's check digit.
        let err = decode("399150 474506 009147 088773 432160 282843 253738 605431").unwrap_err();
        assert!(err.to_string().contains("block 8 "), "{err}");
        assert!(!err.to_string().contains("605431"), "{err}");
        // 99999 does not fit two bytes (xrpl.js reads it as 0x869F).
        let err = decode("999990 474506 009147 088773 432160 282843 253738 605430").unwrap_err();
        assert!(err.to_string().contains("block 1 is above 65535"), "{err}");
        // Non-ASCII digits are not digits.
        assert!(decode("٣99150 474506 009147 088773 432160 282843 253738 605430").is_err());
    }
}
