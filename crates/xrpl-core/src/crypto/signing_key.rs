//! [`SigningKey`] and [`AsyncSigningKey`]: what signs, in memory or not.

#[cfg(feature = "std")]
use super::signing::verify_signing_data;
use super::signing::Keypair;
use crate::{CoreError, MaybeSend};
use core::future::Future;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// A key that signs XRPL signing data, synchronously: a [`Keypair`] (and
/// the SDK's `Wallet`), or a key of your own that never leaves an HSM or a
/// hardware wallet. Every signing function here and in `halcyon-xrpl-sdk`
/// takes one, and every `SigningKey` is an [`AsyncSigningKey`] too.
///
/// A key signs XRPL signing data (a hash prefix and serialized fields) as
/// `rippled` verifies it: an Ed25519 key signs the data itself; a
/// secp256k1 key signs SHA-512Half of the data
/// ([`sha512_half`](super::sha512_half)) and returns a DER signature. A
/// secp256k1 signature with a high S is fine: the signing functions turn it
/// into the low-S form `rippled` requires.
///
/// ```
/// use xrpl_core::crypto::{secp256k1, sha512_half, SigningKey};
/// use xrpl_core::CoreError;
///
/// /// A key on a device. (Here a stand-in holds the secret, to keep the
/// /// example self-contained.)
/// struct Device {
///     public_key: Vec<u8>,
///     secret: Vec<u8>,
/// }
///
/// impl SigningKey for Device {
///     fn public_key(&self) -> &[u8] {
///         &self.public_key
///     }
///
///     fn sign_signing_data(&self, data: &[u8]) -> Result<Vec<u8>, CoreError> {
///         // A secp256k1 device signs the 32-byte digest and returns DER.
///         secp256k1::sign(&self.secret, &sha512_half(data))
///     }
/// }
///
/// let keys = xrpl_core::crypto::Keypair::from_seed(
///     &xrpl_core::crypto::Seed::from_base58("sp5fghtJtpUorTwvof1NpDXAzNwf5")?,
/// )?;
/// let device = Device { public_key: keys.public_key().to_vec(), secret: keys.private_key().to_vec() };
/// let tx = serde_json::json!({
///     "TransactionType": "Payment",
///     "Account": "rU6K7V3Po4snVhBBaU29sesqs2qTQJWDw1",
///     "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
///     "Amount": "1000000", "Fee": "12", "Sequence": 1,
/// });
/// let signed = xrpl_core::crypto::sign_transaction(&tx, &device)?;
/// assert!(xrpl_core::crypto::verify_transaction(&signed)?);
/// # Ok::<(), CoreError>(())
/// ```
pub trait SigningKey {
    /// The public key as the XRP Ledger writes it (`SigningPubKey`): 33
    /// bytes, `0xED` and the key for Ed25519, the compressed point (`0x02`
    /// or `0x03`, then X) for secp256k1.
    fn public_key(&self) -> &[u8];

    /// Sign XRPL signing data: Ed25519 over `data` itself, secp256k1 over
    /// SHA-512Half of `data` (DER).
    ///
    /// # Errors
    ///
    /// Whatever the key fails with; an error of your own goes in
    /// [`CoreError::SigningKeyFailed`] (`CoreError::SigningKeyFailed(e.into())`).
    fn sign_signing_data(&self, data: &[u8]) -> Result<Vec<u8>, CoreError>;
}

/// A key that signs XRPL signing data asynchronously: a remote key (a cloud
/// KMS, a signing service). It signs as a [`SigningKey`] does, and every
/// `SigningKey` is one. Implement it with `async fn sign_signing_data`; on
/// native targets its future must be `Send`.
///
/// A signature from an `AsyncSigningKey` is checked against its public key
/// before it is used, so a key that signs the wrong thing (the data where
/// it should sign the digest, or with another key) fails here, with
/// [`CoreError::SigningKeyFailed`], not on the network.
///
/// ```
/// use xrpl_core::crypto::{secp256k1, sha512_half, AsyncSigningKey};
/// use xrpl_core::CoreError;
///
/// /// A key held by a signing service. (Here a stand-in holds the secret,
/// /// to keep the example self-contained.)
/// struct RemoteKey {
///     public_key: Vec<u8>, // fetched once, when the key is set up
///     secret: Vec<u8>,
/// }
///
/// impl AsyncSigningKey for RemoteKey {
///     fn public_key(&self) -> &[u8] {
///         &self.public_key
///     }
///
///     async fn sign_signing_data(&self, data: &[u8]) -> Result<Vec<u8>, CoreError> {
///         // A secp256k1 KMS signs the 32-byte digest (message type
///         // "DIGEST") and returns DER; its own errors go in with `.into()`.
///         let digest = sha512_half(data);
///         secp256k1::sign(&self.secret, &digest)
///             .map_err(|e| CoreError::SigningKeyFailed(e.to_string().into()))
///     }
/// }
///
/// # async fn run() -> Result<(), CoreError> {
/// # let keys = xrpl_core::crypto::Keypair::from_seed(
/// #     &xrpl_core::crypto::Seed::from_base58("sp5fghtJtpUorTwvof1NpDXAzNwf5")?,
/// # )?;
/// # let key = RemoteKey { public_key: keys.public_key().to_vec(), secret: keys.private_key().to_vec() };
/// let tx = serde_json::json!({
///     "TransactionType": "Payment",
///     "Account": "rU6K7V3Po4snVhBBaU29sesqs2qTQJWDw1",
///     "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
///     "Amount": "1000000", "Fee": "12", "Sequence": 1,
/// });
/// let signed = xrpl_core::crypto::sign_transaction_async(&tx, &key).await?;
/// assert!(xrpl_core::crypto::verify_transaction(&signed)?);
/// # Ok(())
/// # }
/// # fn block_on<F: core::future::Future>(f: F) -> F::Output {
/// #     use core::task::{Context, Poll, Waker};
/// #     let mut f = core::pin::pin!(f);
/// #     loop {
/// #         if let Poll::Ready(v) = f.as_mut().poll(&mut Context::from_waker(Waker::noop())) {
/// #             return v;
/// #         }
/// #     }
/// # }
/// # block_on(run()).unwrap();
/// ```
pub trait AsyncSigningKey {
    /// The public key as the XRP Ledger writes it (see
    /// [`SigningKey::public_key`]). Fetch it once, when the key is set up.
    fn public_key(&self) -> &[u8];

    /// Sign XRPL signing data: Ed25519 over `data` itself, secp256k1 over
    /// SHA-512Half of `data` (DER).
    ///
    /// # Errors
    ///
    /// Whatever the key fails with; an error of your own goes in
    /// [`CoreError::SigningKeyFailed`] (`CoreError::SigningKeyFailed(e.into())`).
    fn sign_signing_data(
        &self,
        data: &[u8],
    ) -> impl Future<Output = Result<Vec<u8>, CoreError>> + MaybeSend;
}

impl<K: SigningKey + ?Sized> AsyncSigningKey for K {
    fn public_key(&self) -> &[u8] {
        SigningKey::public_key(self)
    }

    fn sign_signing_data(
        &self,
        data: &[u8],
    ) -> impl Future<Output = Result<Vec<u8>, CoreError>> + MaybeSend {
        core::future::ready(SigningKey::sign_signing_data(self, data))
    }
}

impl SigningKey for Keypair {
    fn public_key(&self) -> &[u8] {
        Keypair::public_key(self)
    }

    fn sign_signing_data(&self, data: &[u8]) -> Result<Vec<u8>, CoreError> {
        Keypair::sign_signing_data(self, data)
    }
}

/// The key's public key, if it is one the XRP Ledger takes: 33 bytes,
/// starting `0xED` (Ed25519) or `0x02` / `0x03` (secp256k1, compressed).
#[cfg(feature = "std")]
pub(crate) fn checked_public_key(public_key: &[u8]) -> Result<&[u8], CoreError> {
    match public_key {
        [0xED | 0x02 | 0x03, ..] if public_key.len() == 33 => Ok(public_key),
        _ => Err(CoreError::SigningKeyFailed(
            "its public key is not 33 bytes starting 0xED (Ed25519) or 0x02 / 0x03 \
             (a compressed secp256k1 key)"
                .into(),
        )),
    }
}

/// A key's signature over `data`, made ready to use: a secp256k1 signature
/// in the low-S form `rippled` requires (some KMS return a high S). With
/// `check`, it must also verify against `public_key`.
#[cfg(feature = "std")]
pub(crate) fn finished_signature(
    public_key: &[u8],
    data: &[u8],
    signature: Vec<u8>,
    check: bool,
) -> Result<Vec<u8>, CoreError> {
    let signature = if public_key.first() == Some(&0xED) {
        signature
    } else {
        super::secp256k1::low_s(&signature)
            .map_err(|e| CoreError::SigningKeyFailed(e.to_string().into()))?
    };
    if check && !verify_signing_data(public_key, data, &signature)? {
        return Err(CoreError::SigningKeyFailed(
            "its signature does not verify against its public key (an Ed25519 key signs \
             the data itself; a secp256k1 key signs SHA-512Half of the data)"
                .into(),
        ));
    }
    Ok(signature)
}

/// Sign `data` with `key` (synchronously): the finished signature.
#[cfg(feature = "std")]
pub(crate) fn sign_with<K: SigningKey + ?Sized>(
    key: &K,
    data: &[u8],
) -> Result<Vec<u8>, CoreError> {
    let public_key = checked_public_key(SigningKey::public_key(key))?;
    finished_signature(public_key, data, key.sign_signing_data(data)?, false)
}

/// Sign `data` with `key` (asynchronously): the finished signature, checked
/// against the key's public key.
#[cfg(feature = "std")]
pub(crate) async fn sign_with_async<K: AsyncSigningKey + ?Sized>(
    key: &K,
    data: &[u8],
) -> Result<Vec<u8>, CoreError> {
    let public_key = checked_public_key(AsyncSigningKey::public_key(key))?;
    let signature = AsyncSigningKey::sign_signing_data(key, data).await?;
    finished_signature(public_key, data, signature, true)
}

#[cfg(all(test, feature = "std"))]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::crypto::{sign_transaction, verify_transaction, Seed};

    /// A secp256k1 key that answers with the high-S twin of each signature,
    /// as a KMS with a random nonce does about half the time.
    struct HighS(Keypair);

    impl SigningKey for HighS {
        fn public_key(&self) -> &[u8] {
            self.0.public_key()
        }

        fn sign_signing_data(&self, data: &[u8]) -> Result<Vec<u8>, CoreError> {
            use k256::ecdsa::Signature;
            use k256::elliptic_curve::ops::Neg;
            let low = Signature::from_der(&self.0.sign_signing_data(data)?).unwrap();
            let (r, s) = low.split_scalars();
            let high = Signature::from_scalars(r, s.neg()).unwrap();
            Ok(high.to_der().as_bytes().to_vec())
        }
    }

    #[test]
    fn a_high_s_key_signs_what_rippled_takes() {
        let keys = Keypair::from_seed(&Seed::from_base58("sp5fghtJtpUorTwvof1NpDXAzNwf5").unwrap())
            .unwrap();
        let tx = serde_json::json!({
            "TransactionType": "Payment",
            "Account": keys.classic_address(),
            "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
            "Amount": "1000000", "Fee": "12", "Sequence": 1,
        });
        let high = HighS(keys.clone());
        let signed = sign_transaction(&tx, &high).unwrap();
        assert!(verify_transaction(&signed).unwrap());
        assert_eq!(signed, sign_transaction(&tx, &keys).unwrap());
    }

    /// Drive a future that never waits (every key here answers at once).
    fn block_on<F: core::future::Future>(f: F) -> F::Output {
        use core::task::{Context, Poll, Waker};
        let mut f = core::pin::pin!(f);
        loop {
            if let Poll::Ready(v) = f.as_mut().poll(&mut Context::from_waker(Waker::noop())) {
                return v;
            }
        }
    }

    /// Every signing function has an async twin that signs the same bytes.
    #[test]
    fn async_signing_signs_what_sync_signing_does() {
        use crate::crypto::{sign_payment_channel_claim, sign_payment_channel_claim_async};
        use crate::transaction::batch;
        let keys = Keypair::from_seed(&Seed::from_base58("sp5fghtJtpUorTwvof1NpDXAzNwf5").unwrap())
            .unwrap();
        let other = "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe";
        let pay = |from: &str, seq: u32| {
            serde_json::json!({"RawTransaction": {"TransactionType": "Payment", "Account": from,
                "Destination": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", "Amount": "1", "Fee": "0",
                "Sequence": seq, "SigningPubKey": "", "Flags": 0x4000_0000}})
        };
        let tx = serde_json::json!({"TransactionType": "Batch", "Account": other,
            "Sequence": 3, "Flags": 0x0001_0000, "Fee": "40",
            "RawTransactions": [pay(other, 4), pay(&keys.classic_address(), 9)]});
        assert_eq!(
            block_on(batch::sign_batch_async(&tx, &keys)).unwrap(),
            batch::sign_batch(&tx, &keys).unwrap()
        );
        assert_eq!(
            block_on(batch::sign_batch_for_async(
                &tx,
                other,
                &keys.classic_address(),
                &keys
            ))
            .unwrap(),
            batch::sign_batch_for(&tx, other, &keys.classic_address(), &keys).unwrap()
        );
        let channel = "5DB01B7FFED6B67E6B0414DED11E051D2EE2B7619CE0EAA6286D67A3A4D5BDB3";
        assert_eq!(
            block_on(sign_payment_channel_claim_async(channel, 7, &keys)).unwrap(),
            sign_payment_channel_claim(channel, 7, &keys).unwrap()
        );
        let single = serde_json::json!({"TransactionType": "Payment", "Account": keys.classic_address(),
            "Destination": other, "Amount": "1000000", "Fee": "12", "Sequence": 1});
        assert_eq!(
            block_on(crate::crypto::sign_transaction_async(&single, &keys)).unwrap(),
            sign_transaction(&single, &keys).unwrap()
        );
    }

    #[test]
    fn a_public_key_the_ledger_does_not_take_is_refused() {
        for key in [&[0x04; 65][..], &[0xED; 32], &[0x05; 33], &[]] {
            assert!(matches!(
                checked_public_key(key),
                Err(CoreError::SigningKeyFailed(_))
            ));
        }
        assert!(checked_public_key(&[0x02; 33]).is_ok());
    }
}
