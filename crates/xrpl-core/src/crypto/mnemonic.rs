//! BIP-39 recovery phrases: 12 to 24 English words that hold a wallet's
//! entropy, the backup that Ledger devices and most multi-coin wallets show.
//!
//! A phrase is turned into keys in two steps, both standard:
//!
//! 1. [`Mnemonic::to_seed`]: PBKDF2-HMAC-SHA512 over the phrase, salted with
//!    `"mnemonic"` and an optional passphrase (2048 rounds, 64 bytes).
//! 2. [`DerivationPath::derive`](super::hd::DerivationPath::derive): BIP-32
//!    secp256k1 derivation along a path, `m/44'/144'/0'/0/0` for the first
//!    XRP account (coin type 144).
//!
//! [`Keypair::from_mnemonic`](super::Keypair::from_mnemonic) does both. A
//! key from a phrase is always secp256k1 and has no XRPL seed (`s…`): back it
//! up as the phrase.
//!
//! Only the English word list is read, as xrpl.js does. Words may be typed in
//! any case with any whitespace between them; the seed is computed from the
//! phrase in its standard form (lower case, single spaces), so it does not
//! depend on how the words were typed. The passphrase is normalized to
//! Unicode NFKD, as BIP-39 requires, and is otherwise used exactly as given.

use super::wordlists::BIP39_ENGLISH;
use crate::CoreError;
use core::fmt;
use core::str::FromStr;
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256, Sha512};
use unicode_normalization::UnicodeNormalization;
use zeroize::{Zeroize, Zeroizing};

#[cfg(not(feature = "std"))]
use alloc::{format, string::String, vec::Vec};

/// A BIP-39 recovery phrase with a valid checksum. The phrase and its
/// entropy are wiped on drop, and `Debug` shows only the word count.
///
/// ```
/// use xrpl_core::crypto::mnemonic::Mnemonic;
///
/// let phrase = "abandon abandon abandon abandon abandon abandon \
///               abandon abandon abandon abandon abandon about";
/// let mnemonic: Mnemonic = phrase.parse()?;
/// assert_eq!(mnemonic.word_count(), 12);
/// assert_eq!(mnemonic.entropy(), &[0u8; 16]);
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
#[derive(Clone)]
pub struct Mnemonic {
    phrase: Zeroizing<String>,
    entropy: Zeroizing<Vec<u8>>,
}

impl Mnemonic {
    /// Read a phrase: 12, 15, 18, 21 or 24 words from the BIP-39 English
    /// list, in any case, separated by any whitespace.
    ///
    /// # Errors
    ///
    /// [`CoreError::InvalidMnemonic`] if the word count is not one of those,
    /// a word is not on the list, or the checksum (the last bits of the last
    /// word) does not match. The error names a word's position, never the
    /// word.
    pub fn parse(phrase: &str) -> Result<Self, CoreError> {
        let normalized: Zeroizing<String> = Zeroizing::new(phrase.nfkd().collect());
        let mut indices: Zeroizing<Vec<u16>> = Zeroizing::new(Vec::with_capacity(24));
        let mut lower = Zeroizing::new(String::with_capacity(8));
        for (n, word) in normalized.split_whitespace().enumerate() {
            lower.clear();
            lower.extend(word.chars().flat_map(char::to_lowercase));
            let index = BIP39_ENGLISH
                .binary_search(&lower.as_str())
                .ok()
                .and_then(|i| u16::try_from(i).ok())
                .ok_or_else(|| {
                    invalid(format!(
                        "word {} is not in the BIP-39 English word list",
                        n + 1
                    ))
                })?;
            indices.push(index);
        }
        let words = indices.len();
        if !matches!(words, 12 | 15 | 18 | 21 | 24) {
            return Err(invalid(format!(
                "a phrase has 12, 15, 18, 21 or 24 words, not {words}"
            )));
        }
        // words × 11 bits = entropy (32 bits per 3 words) + checksum (1 bit per 3 words).
        let entropy_bytes = words / 3 * 4;
        let mut bits = Zeroizing::new([0u8; 33]);
        for (n, index) in indices.iter().enumerate() {
            for bit in 0..11 {
                if (index >> (10 - bit)) & 1 == 1 {
                    set_bit(bits.as_mut_slice(), n * 11 + bit);
                }
            }
        }
        let (entropy, rest) = bits.split_at(entropy_bytes);
        let entropy = Zeroizing::new(entropy.to_vec());
        let checksum_bits = words / 3;
        let expected = checksum_byte(&entropy, checksum_bits);
        let given = rest.first().copied().unwrap_or_default() & top_bits(checksum_bits);
        if expected != given {
            return Err(invalid(
                "the checksum does not match: a word is wrong or out of order".into(),
            ));
        }
        Ok(Self::from_indices(&indices, entropy))
    }

    /// The phrase for `entropy`: 16, 20, 24, 28 or 32 bytes give 12, 15, 18,
    /// 21 or 24 words.
    ///
    /// # Errors
    ///
    /// [`CoreError::InvalidMnemonic`] for any other length.
    pub fn from_entropy(entropy: &[u8]) -> Result<Self, CoreError> {
        if !matches!(entropy.len(), 16 | 20 | 24 | 28 | 32) {
            return Err(invalid(format!(
                "entropy is 16, 20, 24, 28 or 32 bytes, not {}",
                entropy.len()
            )));
        }
        let checksum_bits = entropy.len() / 4;
        let words = checksum_bits * 3;
        let mut bits = Zeroizing::new([0u8; 33]);
        let (head, tail) = bits.split_at_mut(entropy.len());
        head.copy_from_slice(entropy);
        if let Some(checksum) = tail.first_mut() {
            *checksum = checksum_byte(entropy, checksum_bits);
        }
        let mut indices: Zeroizing<Vec<u16>> = Zeroizing::new(Vec::with_capacity(words));
        for n in 0..words {
            let index = (0..11).fold(0u16, |acc, bit| {
                (acc << 1) | get_bit(bits.as_slice(), n * 11 + bit)
            });
            indices.push(index);
        }
        Ok(Self::from_indices(
            &indices,
            Zeroizing::new(entropy.to_vec()),
        ))
    }

    /// A new phrase of `word_count` words (12, 15, 18, 21 or 24) from the
    /// operating system's random source. 24 words hold 256 bits; 12 hold 128,
    /// the same as an XRPL seed.
    ///
    /// # Errors
    ///
    /// [`CoreError::InvalidMnemonic`] for any other word count.
    #[cfg(feature = "std")]
    pub fn generate(word_count: usize) -> Result<Self, CoreError> {
        if !matches!(word_count, 12 | 15 | 18 | 21 | 24) {
            return Err(invalid(format!(
                "a phrase has 12, 15, 18, 21 or 24 words, not {word_count}"
            )));
        }
        let mut entropy = Zeroizing::new(vec![0u8; word_count / 3 * 4]);
        super::signing::fill_random(&mut entropy);
        Self::from_entropy(&entropy)
    }

    fn from_indices(indices: &[u16], entropy: Zeroizing<Vec<u8>>) -> Self {
        let mut phrase = Zeroizing::new(String::with_capacity(indices.len() * 9));
        for index in indices {
            if !phrase.is_empty() {
                phrase.push(' ');
            }
            phrase.push_str(word(*index));
        }
        Self { phrase, entropy }
    }

    /// The phrase in its standard form: lower case, single spaces.
    pub fn phrase(&self) -> &str {
        &self.phrase
    }

    /// The number of words.
    pub fn word_count(&self) -> usize {
        self.entropy.len() / 4 * 3
    }

    /// The entropy the phrase holds (16 to 32 bytes).
    pub fn entropy(&self) -> &[u8] {
        &self.entropy
    }

    /// The 64-byte BIP-39 seed: PBKDF2-HMAC-SHA512 of the phrase, salted with
    /// `"mnemonic"` and `passphrase` (NFKD), 2048 rounds. Pass `""` for no
    /// passphrase. A different passphrase gives a different, equally valid
    /// wallet, so a mistyped one is not an error. The seed is wiped on drop.
    pub fn to_seed(&self, passphrase: &str) -> Zeroizing<[u8; 64]> {
        let mut salt = Zeroizing::new(String::from("mnemonic"));
        salt.extend(passphrase.nfkd());
        pbkdf2_hmac_sha512(self.phrase.as_bytes(), salt.as_bytes(), 2048)
    }
}

impl FromStr for Mnemonic {
    type Err = CoreError;

    fn from_str(phrase: &str) -> Result<Self, CoreError> {
        Self::parse(phrase)
    }
}

impl fmt::Debug for Mnemonic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Mnemonic")
            .field("word_count", &self.word_count())
            .field("phrase", &"[REDACTED]")
            .finish()
    }
}

impl PartialEq for Mnemonic {
    /// Equal when they hold the same entropy, compared in constant time.
    fn eq(&self, other: &Self) -> bool {
        use subtle::ConstantTimeEq;
        self.entropy.len() == other.entropy.len()
            && bool::from(self.entropy.as_slice().ct_eq(other.entropy.as_slice()))
    }
}

impl Eq for Mnemonic {}

fn invalid(message: String) -> CoreError {
    CoreError::InvalidMnemonic(message)
}

/// The word for an 11-bit index.
#[allow(
    clippy::indexing_slicing,
    reason = "indices are 11-bit values (below 2048) and the list has 2048 words"
)]
fn word(index: u16) -> &'static str {
    BIP39_ENGLISH[usize::from(index & 0x7FF)]
}

/// Bit `at` of `bytes`, counting from the top bit of the first byte.
fn get_bit(bytes: &[u8], at: usize) -> u16 {
    bytes
        .get(at / 8)
        .map_or(0, |byte| u16::from((byte >> (7 - at % 8)) & 1))
}

/// Set bit `at` of `bytes`, counting from the top bit of the first byte.
fn set_bit(bytes: &mut [u8], at: usize) {
    if let Some(byte) = bytes.get_mut(at / 8) {
        *byte |= 0x80 >> (at % 8);
    }
}

/// The first `bits` bits of SHA-256(`entropy`), in the top of a byte.
fn checksum_byte(entropy: &[u8], bits: usize) -> u8 {
    let mut hash: [u8; 32] = Sha256::digest(entropy).into();
    let byte = hash[0] & top_bits(bits);
    hash.zeroize();
    byte
}

/// A byte with its top `bits` bits set (`bits` up to 8).
fn top_bits(bits: usize) -> u8 {
    u32::try_from(bits)
        .ok()
        .and_then(|b| u8::MAX.checked_shr(b))
        .map_or(u8::MAX, |low| !low)
}

/// PBKDF2-HMAC-SHA512 with one 64-byte output block (RFC 8018 §5.2).
fn pbkdf2_hmac_sha512(password: &[u8], salt: &[u8], rounds: u32) -> Zeroizing<[u8; 64]> {
    type HmacSha512 = Hmac<Sha512>;
    #[allow(
        clippy::expect_used,
        reason = "HMAC takes a key of any length; new_from_slice cannot fail"
    )]
    let keyed = HmacSha512::new_from_slice(password).expect("HMAC accepts any key length");
    let mut mac = keyed.clone();
    mac.update(salt);
    mac.update(&1u32.to_be_bytes());
    let mut u = Zeroizing::new(<[u8; 64]>::from(mac.finalize().into_bytes()));
    let mut out = Zeroizing::new(*u);
    for _ in 1..rounds {
        let mut mac = keyed.clone();
        mac.update(u.as_slice());
        *u = mac.finalize().into_bytes().into();
        for (o, b) in out.iter_mut().zip(u.iter()) {
            *o ^= b;
        }
    }
    out
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
pub(crate) mod tests {
    use super::*;

    /// The 24 English vectors of the BIP-39 reference implementation
    /// (trezor/python-mnemonic `vectors.json`, passphrase `TREZOR`): entropy,
    /// phrase, seed, BIP-32 master key.
    pub(crate) fn trezor_vectors() -> Vec<[String; 4]> {
        let file: serde_json::Value =
            serde_json::from_str(include_str!("../../tests/vectors/bip39_trezor.json")).unwrap();
        serde_json::from_value(file["english"].clone()).unwrap()
    }

    #[test]
    fn reference_vectors() {
        let vectors = trezor_vectors();
        assert_eq!(vectors.len(), 24);
        for [entropy, phrase, seed, _] in &vectors {
            let entropy = hex::decode(entropy).unwrap();
            let from_entropy = Mnemonic::from_entropy(&entropy).unwrap();
            assert_eq!(from_entropy.phrase(), phrase);
            let parsed = Mnemonic::parse(phrase).unwrap();
            assert_eq!(parsed.entropy(), entropy.as_slice());
            assert_eq!(parsed, from_entropy);
            assert_eq!(&hex::encode(*parsed.to_seed("TREZOR")), seed);
        }
    }

    #[test]
    fn typed_forms_read_the_same() {
        let [_, phrase, seed, _] = &trezor_vectors()[1];
        let shouted = phrase.to_uppercase().replace(' ', "  \n ");
        let parsed = Mnemonic::parse(&format!("  {shouted}\t")).unwrap();
        assert_eq!(parsed.phrase(), phrase);
        assert_eq!(&hex::encode(*parsed.to_seed("TREZOR")), seed);
    }

    #[test]
    fn passphrase_is_nfkd() {
        // "é" typed as one code point (U+00E9) or as "e" + U+0301, and the
        // ligature "ﬁ" (U+FB01) or "fi": NFKD makes each pair the same.
        let m = Mnemonic::from_entropy(&[0x11; 16]).unwrap();
        assert_eq!(*m.to_seed("caf\u{00E9}"), *m.to_seed("cafe\u{0301}"));
        assert_eq!(*m.to_seed("\u{FB01}ve"), *m.to_seed("five"));
        assert_ne!(*m.to_seed("caf\u{00E9}"), *m.to_seed("cafe"));
    }

    #[test]
    fn refusals_name_positions_not_words() {
        let [_, phrase, _, _] = &trezor_vectors()[0];
        let words: Vec<&str> = phrase.split(' ').collect();
        let err = Mnemonic::parse(&words[..11].join(" ")).unwrap_err();
        assert!(err.to_string().contains("not 11"), "{err}");
        let mut bad = words.clone();
        bad[4] = "bitcoinz";
        let err = Mnemonic::parse(&bad.join(" ")).unwrap_err();
        assert!(err.to_string().contains("word 5 "), "{err}");
        assert!(!err.to_string().contains("bitcoinz"), "{err}");
        // "abandon" × 12 is on the list but fails the checksum.
        let err = Mnemonic::parse(&["abandon"; 12].join(" ")).unwrap_err();
        assert!(err.to_string().contains("checksum"), "{err}");
        assert!(Mnemonic::from_entropy(&[0; 15]).is_err());
    }

    #[cfg(feature = "std")]
    #[test]
    fn generate_round_trips() {
        assert!(Mnemonic::generate(13).is_err());
        for words in [12, 15, 18, 21, 24] {
            let m = Mnemonic::generate(words).unwrap();
            assert_eq!(m.word_count(), words);
            assert_eq!(Mnemonic::parse(m.phrase()).unwrap(), m);
            assert_eq!(
                format!("{m:?}"),
                format!("Mnemonic {{ word_count: {words}, phrase: \"[REDACTED]\" }}")
            );
        }
    }
}
