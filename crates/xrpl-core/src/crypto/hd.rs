//! BIP-32 key derivation (secp256k1): the private key at a path such as
//! `m/44'/144'/0'/0/0` below a 64-byte BIP-39 seed.
//!
//! XRP is coin type 144 (SLIP-44). [`DerivationPath::xrp_account`] gives
//! `m/44'/144'/{account}'/0/0`, the path Ledger devices and xrpl.js use for
//! XRP account `account`; the [`Default`] path is account 0.
//!
//! Only private derivation is offered (no extended public keys): the result
//! is a [`Keypair`] to sign with. Hardened steps may be written `'`, `h` or
//! `H`.

use super::Keypair;
use crate::address::KeyType;
use crate::CoreError;
use core::fmt;
use core::str::FromStr;
use hmac::{Hmac, Mac};
use k256::elliptic_curve::sec1::ToSec1Point;
use k256::elliptic_curve::PrimeField;
use k256::{FieldBytes, Scalar, SecretKey};
use sha2::Sha512;
use zeroize::{Zeroize, Zeroizing};

#[cfg(not(feature = "std"))]
use alloc::{format, string::String, vec::Vec};

/// The bit that marks a hardened step (`2^31`).
///
/// ```
/// use xrpl_core::crypto::hd::{DerivationPath, HARDENED};
///
/// assert_eq!(HARDENED, 1 << 31);
/// let path = DerivationPath::from_indices(&[44 | HARDENED, 144 | HARDENED, 7])?;
/// assert_eq!(path.to_string(), "m/44'/144'/7");
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
pub const HARDENED: u32 = 0x8000_0000;

/// A BIP-32 derivation path: the child index of each step, hardened ones
/// with [`HARDENED`] set.
///
/// ```
/// use xrpl_core::crypto::hd::{DerivationPath, HARDENED};
///
/// let path: DerivationPath = "m/44'/144'/0'/0/0".parse()?;
/// assert_eq!(path, DerivationPath::default());
/// assert_eq!(path.indices(), &[44 | HARDENED, 144 | HARDENED, HARDENED, 0, 0]);
/// assert_eq!(DerivationPath::xrp_account(3)?.to_string(), "m/44'/144'/3'/0/0");
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DerivationPath {
    indices: Vec<u32>,
}

impl DerivationPath {
    /// `m/44'/144'/{account}'/0/0`: XRP account `account` (0 for the first).
    ///
    /// ```
    /// use xrpl_core::crypto::hd::{DerivationPath, HARDENED};
    ///
    /// assert_eq!(DerivationPath::xrp_account(0)?, DerivationPath::default());
    /// assert_eq!(DerivationPath::xrp_account(1)?.to_string(), "m/44'/144'/1'/0/0");
    /// assert!(DerivationPath::xrp_account(HARDENED).is_err());
    /// # Ok::<(), xrpl_core::CoreError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// [`CoreError::InvalidDerivationPath`] if `account` is `2^31` or more.
    pub fn xrp_account(account: u32) -> Result<Self, CoreError> {
        if account >= HARDENED {
            return Err(CoreError::InvalidDerivationPath(format!(
                "an account index is below 2^31, not {account}"
            )));
        }
        Ok(Self {
            indices: [44 | HARDENED, 144 | HARDENED, account | HARDENED, 0, 0].to_vec(),
        })
    }

    /// A path from its child indices (hardened ones with [`HARDENED`] set).
    ///
    /// ```
    /// use xrpl_core::crypto::hd::{DerivationPath, HARDENED};
    ///
    /// let path = DerivationPath::from_indices(&[44 | HARDENED, 144 | HARDENED, HARDENED, 0, 0])?;
    /// assert_eq!(path, DerivationPath::default());
    /// assert!(DerivationPath::from_indices(&[0; 256]).is_err());
    /// # Ok::<(), xrpl_core::CoreError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// [`CoreError::InvalidDerivationPath`] for more than 255 steps (a BIP-32
    /// depth is one byte).
    pub fn from_indices(indices: &[u32]) -> Result<Self, CoreError> {
        if indices.len() > 255 {
            return Err(CoreError::InvalidDerivationPath(format!(
                "a path has at most 255 steps, not {}",
                indices.len()
            )));
        }
        Ok(Self {
            indices: indices.to_vec(),
        })
    }

    /// The child index of each step.
    ///
    /// ```
    /// use xrpl_core::crypto::hd::{DerivationPath, HARDENED};
    ///
    /// let path: DerivationPath = "m/0'/1".parse()?;
    /// assert_eq!(path.indices(), &[HARDENED, 1]);
    /// # Ok::<(), xrpl_core::CoreError>(())
    /// ```
    pub fn indices(&self) -> &[u32] {
        &self.indices
    }

    /// The secp256k1 key pair at this path below `seed` (16 to 64 bytes: a
    /// BIP-39 seed is 64, from [`Mnemonic::to_seed`](super::mnemonic::Mnemonic::to_seed)).
    ///
    /// ```
    /// use xrpl_core::crypto::hd::DerivationPath;
    /// use xrpl_core::crypto::mnemonic::Mnemonic;
    ///
    /// let phrase: Mnemonic =
    ///     "assault rare scout seed design extend noble drink talk control guitar quote".parse()?;
    /// let keypair = DerivationPath::default().derive(phrase.to_seed("").as_slice())?;
    /// assert_eq!(keypair.classic_address(), "rJUr92pXcNtH6PtFfbRhHCo469XmPrbTJU");
    /// assert!(DerivationPath::default().derive(&[0; 15]).is_err());
    /// # Ok::<(), xrpl_core::CoreError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// [`CoreError::ValidationError`] for a seed of another length;
    /// [`CoreError::KeyDerivationFailed`] in the case BIP-32 calls invalid
    /// (probability below 2^-127), where this path has no key.
    pub fn derive(&self, seed: &[u8]) -> Result<Keypair, CoreError> {
        let (key, _chain) = derive_extended(seed, &self.indices)?;
        let bytes = Zeroizing::new(key.to_bytes().to_vec());
        Keypair::from_private_key(KeyType::Secp256k1, &bytes)
    }
}

impl Default for DerivationPath {
    /// `m/44'/144'/0'/0/0`, the first XRP account.
    fn default() -> Self {
        Self {
            indices: [44 | HARDENED, 144 | HARDENED, HARDENED, 0, 0].to_vec(),
        }
    }
}

impl FromStr for DerivationPath {
    type Err = CoreError;

    /// Read `m` followed by `/`-separated steps, each a decimal index below
    /// 2^31 with `'`, `h` or `H` for a hardened step.
    fn from_str(path: &str) -> Result<Self, CoreError> {
        let invalid = |m: String| CoreError::InvalidDerivationPath(format!("{m} (in \"{path}\")"));
        let mut steps = path.trim().split('/');
        if steps.next() != Some("m") {
            return Err(invalid("a path starts with \"m\"".into()));
        }
        let mut indices = Vec::new();
        for (n, step) in steps.enumerate() {
            let (digits, hardened) = match step.strip_suffix(['\'', 'h', 'H']) {
                Some(digits) => (digits, true),
                None => (step, false),
            };
            let index = (!digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()))
                .then(|| digits.parse::<u32>().ok())
                .flatten()
                .filter(|i| *i < HARDENED)
                .ok_or_else(|| {
                    invalid(format!(
                        "step {} is not an index below 2^31 (with ' for hardened)",
                        n + 1
                    ))
                })?;
            indices.push(if hardened { index | HARDENED } else { index });
        }
        Self::from_indices(&indices)
    }
}

impl fmt::Display for DerivationPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("m")?;
        for index in &self.indices {
            if index & HARDENED == 0 {
                write!(f, "/{index}")?;
            } else {
                write!(f, "/{}'", index & !HARDENED)?;
            }
        }
        Ok(())
    }
}

type HmacSha512 = Hmac<Sha512>;

/// HMAC-SHA512 of `parts` under `key`, in a buffer wiped on drop.
fn hmac_sha512(key: &[u8], parts: &[&[u8]]) -> Zeroizing<[u8; 64]> {
    #[allow(
        clippy::expect_used,
        reason = "HMAC takes a key of any length; new_from_slice cannot fail"
    )]
    let mut mac = HmacSha512::new_from_slice(key).expect("HMAC accepts any key length");
    for part in parts {
        mac.update(part);
    }
    Zeroizing::new(mac.finalize().into_bytes().into())
}

/// The private key and chain code at `indices` below `seed`.
pub(crate) fn derive_extended(
    seed: &[u8],
    indices: &[u32],
) -> Result<(SecretKey, Zeroizing<[u8; 32]>), CoreError> {
    if !(16..=64).contains(&seed.len()) {
        return Err(CoreError::ValidationError(format!(
            "a BIP-32 seed is 16 to 64 bytes, not {}",
            seed.len()
        )));
    }
    let i = hmac_sha512(b"Bitcoin seed", &[seed]);
    let (left, right) = i.split_at(32);
    let mut key = SecretKey::from_slice(left).map_err(|_| CoreError::KeyDerivationFailed)?;
    let mut chain = Zeroizing::new([0u8; 32]);
    chain.copy_from_slice(right);
    for index in indices {
        let mut parent = Zeroizing::new([0u8; 32]);
        parent.copy_from_slice(&key.to_bytes());
        let i = if index & HARDENED == 0 {
            let point = key.public_key().to_sec1_point(true);
            hmac_sha512(chain.as_slice(), &[point.as_bytes(), &index.to_be_bytes()])
        } else {
            hmac_sha512(
                chain.as_slice(),
                &[&[0], parent.as_slice(), &index.to_be_bytes()],
            )
        };
        // k_child = I_L + k_parent (mod n); I_L ≥ n or a zero sum has no key.
        let (i_left, i_right) = i.split_at(32);
        let mut left = Zeroizing::new([0u8; 32]);
        left.copy_from_slice(i_left);
        let mut tweak = Option::<Scalar>::from(Scalar::from_repr(FieldBytes::from(*left)))
            .ok_or(CoreError::KeyDerivationFailed)?;
        let mut sum = tweak + *key.to_nonzero_scalar();
        let child = SecretKey::from_bytes(&sum.to_bytes());
        tweak.zeroize();
        sum.zeroize();
        key = child.map_err(|_| CoreError::KeyDerivationFailed)?;
        chain.copy_from_slice(i_right);
    }
    Ok((key, chain))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;
    use crate::crypto::mnemonic::{tests::trezor_vectors, Mnemonic};

    /// The chain code and private key of an `xprv…` string.
    fn parse_xprv(xprv: &str) -> ([u8; 32], [u8; 32]) {
        let raw = bs58::decode(xprv).into_vec().unwrap();
        assert_eq!(raw.len(), 82);
        assert_eq!(&raw[..4], &[0x04, 0x88, 0xAD, 0xE4]);
        assert_eq!(raw[45], 0);
        (
            raw[13..45].try_into().unwrap(),
            raw[46..78].try_into().unwrap(),
        )
    }

    #[test]
    fn bip32_reference_vectors() {
        let file: serde_json::Value =
            serde_json::from_str(include_str!("../../tests/vectors/bip32.json")).unwrap();
        let mut checked = 0;
        for vector in file["vectors"].as_array().unwrap() {
            let seed = hex::decode(vector["seed"].as_str().unwrap()).unwrap();
            for chain in vector["chains"].as_array().unwrap() {
                let path: DerivationPath = chain["path"].as_str().unwrap().parse().unwrap();
                assert_eq!(path.to_string(), chain["path"].as_str().unwrap());
                let (key, code) = derive_extended(&seed, path.indices()).unwrap();
                let (want_code, want_key) = parse_xprv(chain["xprv"].as_str().unwrap());
                assert_eq!(key.to_bytes()[..], want_key, "{path}");
                assert_eq!(*code, want_code, "{path}");
                checked += 1;
            }
        }
        assert_eq!(checked, 17);
    }

    #[test]
    fn bip39_master_keys() {
        for [_, phrase, _, xprv] in trezor_vectors() {
            let seed = Mnemonic::parse(&phrase).unwrap().to_seed("TREZOR");
            let (key, code) = derive_extended(seed.as_slice(), &[]).unwrap();
            let (want_code, want_key) = parse_xprv(&xprv);
            assert_eq!(key.to_bytes()[..], want_key);
            assert_eq!(*code, want_code);
        }
    }

    #[test]
    fn xrpl_js_known_answer() {
        // xrpl.js `Wallet.fromMnemonic` test: the default path m/44'/144'/0'/0/0.
        let mnemonic = Mnemonic::parse(
            "assault rare scout seed design extend noble drink talk control guitar quote",
        )
        .unwrap();
        let keypair = DerivationPath::default()
            .derive(mnemonic.to_seed("").as_slice())
            .unwrap();
        assert_eq!(
            hex::encode_upper(keypair.public_key()),
            "035953FCD81D001CF634EB44A87940F3F98ADF2483D09C914BAED0539BE50F385D"
        );
        assert_eq!(
            hex::encode_upper(keypair.private_key()),
            "13FC461CA5799F1357C8130AF703CBA7E9C28E072C6CA8F7DEF8601CDE98F394"
        );
    }

    #[test]
    fn paths() {
        for (text, shown) in [
            ("m", "m"),
            ("m/0h/1H/2'", "m/0'/1'/2'"),
            (" m/44'/144'/0'/0/0 ", "m/44'/144'/0'/0/0"),
            ("m/2147483647'", "m/2147483647'"),
        ] {
            assert_eq!(text.parse::<DerivationPath>().unwrap().to_string(), shown);
        }
        for bad in [
            "",
            "44'/144'",
            "M/0",
            "m/",
            "m//0",
            "m/2147483648",
            "m/-1",
            "m/+1",
            "m/0''",
            "m/x",
        ] {
            assert!(bad.parse::<DerivationPath>().is_err(), "{bad}");
        }
        assert!(DerivationPath::xrp_account(HARDENED).is_err());
        assert!(DerivationPath::from_indices(&[0; 256]).is_err());
        assert!(DerivationPath::default().derive(&[0; 15]).is_err());
        assert!(DerivationPath::default().derive(&[0; 65]).is_err());
    }
}
