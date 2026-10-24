//! Payment channel claims: what a channel's source signs so the destination can
//! redeem XRP from it later (rippled's `channel_authorize`), and how anyone
//! checks one (`channel_verify`, or PaymentChannelClaim's `Signature`).

use super::signing::verify_signing_data;
use super::signing_key::SigningKey;
use crate::types::Hash256;
use crate::CoreError;

/// `HashPrefix::paymentChannelClaim` ("CLM\0").
const PREFIX_CLAIM: [u8; 4] = *b"CLM\0";

/// "CLM\0" ‖ channel ID ‖ amount in drops (big-endian), as rippled signs it.
fn claim_data(channel_id: &str, amount_drops: u64) -> Result<Vec<u8>, CoreError> {
    let channel = Hash256::from_hex(channel_id)?;
    let mut data = Vec::with_capacity(4 + 32 + 8);
    data.extend_from_slice(&PREFIX_CLAIM);
    data.extend_from_slice(&channel.0);
    data.extend_from_slice(&amount_drops.to_be_bytes());
    Ok(data)
}

/// Sign a claim for `amount_drops` (the total the destination may have taken
/// from the channel, not an increment) with `key`, the channel's key (a
/// [`Keypair`](super::Keypair) or any [`SigningKey`]). Returns the signature
/// in hex, for PaymentChannelClaim's `Signature`.
///
/// ```
/// use xrpl_core::crypto::{sign_payment_channel_claim, verify_payment_channel_claim, Keypair, Seed};
///
/// let keys = Keypair::from_seed(&Seed::from_base58("sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r")?)?;
/// let channel = "5DB01B7FFED6B67E6B0414DED11E051D2EE2B7619CE0EAA6286D67A3A4D5BDB3";
/// let signature = sign_payment_channel_claim(channel, 1_000_000, &keys)?;
/// let public_key = hex::encode_upper(keys.public_key());
/// assert!(verify_payment_channel_claim(channel, 1_000_000, &signature, &public_key)?);
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
///
/// # Errors
///
/// [`CoreError::InvalidHex`] if `channel_id` is not 32 bytes in hex;
/// otherwise only the key's own errors ([`CoreError::SigningKeyFailed`] for
/// a public key the XRP Ledger does not take).
pub fn sign_payment_channel_claim<K: SigningKey + ?Sized>(
    channel_id: &str,
    amount_drops: u64,
    key: &K,
) -> Result<String, CoreError> {
    let signature = super::signing_key::sign_with(key, &claim_data(channel_id, amount_drops)?)?;
    Ok(hex::encode_upper(signature))
}

/// [`sign_payment_channel_claim`] with an
/// [`AsyncSigningKey`](super::AsyncSigningKey) (a remote key), whose
/// signature is checked against its public key.
///
/// ```
/// use xrpl_core::crypto::{sign_payment_channel_claim_async, verify_payment_channel_claim, Keypair, Seed};
///
/// # async fn run() -> Result<(), xrpl_core::CoreError> {
/// let key = Keypair::from_seed(&Seed::from_base58("sp5fghtJtpUorTwvof1NpDXAzNwf5")?)?;
/// let channel = "5DB01B7FFED6B67E6B0414DED11E051D2EE2B7619CE0EAA6286D67A3A4D5BDB3";
/// let signature = sign_payment_channel_claim_async(channel, 1_000_000, &key).await?;
/// let public_key = hex::encode_upper(key.public_key());
/// assert!(verify_payment_channel_claim(channel, 1_000_000, &signature, &public_key)?);
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
///
/// # Errors
///
/// As [`sign_payment_channel_claim`], and [`CoreError::SigningKeyFailed`]
/// for a signature that does not verify.
pub async fn sign_payment_channel_claim_async<K: super::AsyncSigningKey + ?Sized>(
    channel_id: &str,
    amount_drops: u64,
    key: &K,
) -> Result<String, CoreError> {
    let data = claim_data(channel_id, amount_drops)?;
    let signature = super::signing_key::sign_with_async(key, &data).await?;
    Ok(hex::encode_upper(signature))
}

/// Whether `signature` is the channel key's (`public_key`, hex) claim for
/// `amount_drops`. A well-formed signature that does not match is `Ok(false)`.
///
/// This checks the claim `rippled` 3.4.1's `channel_authorize` signed:
///
/// ```
/// use xrpl_core::crypto::verify_payment_channel_claim;
///
/// let channel = "5DB01B7FFED6B67E6B0414DED11E051D2EE2B7619CE0EAA6286D67A3A4D5BDB3";
/// let public_key = "030D58EB48B4420B1F7B9DF55087E0E29FEF0E8468F9A6825B01CA2C361042D435";
/// let signature = "3044022015DB5B0764337A0DB560A8841879EA4C21C8A3E58EDB691E8EFF61395619E69A02205C99B5784F95181820FE7771D707260CAA5DF4F0CF21E3F8E9396290718C40B2";
/// assert!(verify_payment_channel_claim(channel, 1_000_000, signature, public_key)?);
/// // The same signature does not cover one drop more.
/// assert!(!verify_payment_channel_claim(channel, 1_000_001, signature, public_key)?);
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
///
/// # Errors
///
/// [`CoreError::InvalidHex`] if `channel_id` is not 32 bytes in hex, or
/// `signature` or `public_key` is not hex; and the errors of
/// [`verify_signing_data`] for a malformed key or signature.
pub fn verify_payment_channel_claim(
    channel_id: &str,
    amount_drops: u64,
    signature: &str,
    public_key: &str,
) -> Result<bool, CoreError> {
    let data = claim_data(channel_id, amount_drops)?;
    let signature = hex::decode(signature).map_err(|e| CoreError::InvalidHex(e.to_string()))?;
    let public_key = hex::decode(public_key).map_err(|e| CoreError::InvalidHex(e.to_string()))?;
    verify_signing_data(&public_key, &data, &signature)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::address::KeyType;
    use crate::crypto::Keypair;
    use crate::crypto::Seed;

    const CHANNEL: &str = "5DB01B7FFED6B67E6B0414DED11E051D2EE2B7619CE0EAA6286D67A3A4D5BDB3";

    // rippled 3.4.1 `channel_authorize` (channel_id CHANNEL, amount "1000000")
    // with the public test seeds of crypto::signing's tests.
    #[test]
    fn matches_rippled_channel_authorize() {
        for (seed, public_key, rippled_signature) in [
            (
                "sp5fghtJtpUorTwvof1NpDXAzNwf5",
                "030D58EB48B4420B1F7B9DF55087E0E29FEF0E8468F9A6825B01CA2C361042D435",
                "3044022015DB5B0764337A0DB560A8841879EA4C21C8A3E58EDB691E8EFF61395619E69A02205C99B5784F95181820FE7771D707260CAA5DF4F0CF21E3F8E9396290718C40B2",
            ),
            (
                "sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r",
                "ED01FA53FA5A7E77798F882ECE20B1ABC00BB358A9E55A202D0D0676BD0CE37A63",
                "F86EFC14E70AAC691277620DA42C71906948C232187A68C75C0825D9AC2B5F2C3EA483AC6BD0FD6FDFC54F2B92E9CFFF82FEF541D6639B67DC5F66BE875CED01",
            ),
        ] {
            let keypair = Keypair::from_seed(&Seed::from_base58(seed).unwrap()).unwrap();
            assert_eq!(hex::encode_upper(keypair.public_key()), public_key);
            assert_eq!(sign_payment_channel_claim(CHANNEL, 1_000_000, &keypair).unwrap(), rippled_signature);
            assert!(verify_payment_channel_claim(CHANNEL, 1_000_000, rippled_signature, public_key).unwrap());
            assert!(!verify_payment_channel_claim(CHANNEL, 1_000_001, rippled_signature, public_key).unwrap());
        }
    }

    // xrpl.js signPaymentChannelClaim / verifyPaymentChannelClaim tests
    // (amount ".00001" XRP = 10 drops).
    #[test]
    fn matches_xrpl_js() {
        let channel = "3E18C05AD40319B809520F1A136370C4075321B285217323396D6FD9EE1E9037";
        let expected = "3045022100B5C54654221F154347679B97AE7791CBEF5E6772A3F894F9C781B8F1B400F89F022021E466D29DC5AEB5DFAFC76E8A88D2E388EBD25A84143B6AC3B647F479CB89B7";
        let keypair = Keypair::from_private_key(
            KeyType::Secp256k1,
            &hex::decode("ACCD3309DB14D1A4FC9B1DAE608031F4408C85C73EE05E035B7DC8B25840107A")
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            hex::encode_upper(keypair.public_key()),
            "02F89EAEC7667B30F33D0687BBA86C3FE2A08CCA40A9186C5BDE2DAA6FA97A37D8"
        );
        assert_eq!(
            sign_payment_channel_claim(channel, 10, &keypair).unwrap(),
            expected
        );
        assert!(verify_payment_channel_claim(
            channel,
            10,
            expected,
            "02F89EAEC7667B30F33D0687BBA86C3FE2A08CCA40A9186C5BDE2DAA6FA97A37D8"
        )
        .unwrap());
        assert!(!verify_payment_channel_claim(
            channel,
            10,
            expected,
            "03A6523FE4281DA48A6FD77FAF3CB77F5C7001ABA0B32BCEDE0369AC009758D7D9"
        )
        .unwrap());
    }
}
