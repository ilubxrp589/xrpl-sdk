use crate::CoreError;
use k256::ecdsa::signature::hazmat::PrehashSigner;
use k256::ecdsa::{Signature, SigningKey, VerifyingKey};
use k256::elliptic_curve::ops::Reduce;
use k256::elliptic_curve::sec1::ToEncodedPoint;
use k256::{Scalar, SecretKey, U256};
use sha2::{Digest, Sha512};
use zeroize::Zeroize;

#[cfg(not(feature = "std"))]
use alloc::{string::ToString, vec::Vec};

/// Derive an XRPL secp256k1 keypair from a 16-byte seed.
/// Uses the XRPL custom root keypair derivation (NOT BIP32).
pub fn derive_keypair(seed: &[u8; 16]) -> Result<(Vec<u8>, Vec<u8>), CoreError> {
    // Step 1: Root private key
    let mut root_private_bytes = {
        let mut result = None;
        for seq in 0u32.. {
            let mut payload = Vec::with_capacity(20);
            payload.extend_from_slice(seed);
            payload.extend_from_slice(&seq.to_be_bytes());
            let hash = Sha512::digest(&payload);
            // SHA512 always produces 64 bytes; first 32 always exist
            let bytes = hash.get(..32).ok_or(CoreError::KeyDerivationFailed)?;
            // SecretKey::from_slice validates: non-zero and < curve order
            if SecretKey::from_slice(bytes).is_ok() {
                result = Some(bytes.to_vec());
                break;
            }
        }
        result.ok_or(CoreError::KeyDerivationFailed)?
    };

    // Step 2: Root public key (33-byte compressed)
    let root_secret =
        SecretKey::from_slice(&root_private_bytes).map_err(|_| CoreError::KeyDerivationFailed)?;
    let root_public_point = root_secret.public_key();
    let root_public_compressed = root_public_point.to_encoded_point(true).as_bytes().to_vec();

    // Step 3: Account keypair derivation (sequence=0)
    let mut intermediate_bytes = {
        let seq = 0u32;
        let mut result = None;
        for sub_seq in 0u32.. {
            let mut payload = Vec::with_capacity(root_public_compressed.len() + 8);
            payload.extend_from_slice(&root_public_compressed);
            payload.extend_from_slice(&seq.to_be_bytes());
            payload.extend_from_slice(&sub_seq.to_be_bytes());
            let hash = Sha512::digest(&payload);
            // SHA512 always produces 64 bytes; first 32 always exist
            let bytes = hash.get(..32).ok_or(CoreError::KeyDerivationFailed)?;
            if SecretKey::from_slice(bytes).is_ok() {
                result = Some(bytes.to_vec());
                break;
            }
        }
        result.ok_or(CoreError::KeyDerivationFailed)?
    };

    // account_private = (root_private + intermediate) mod order
    let root_scalar = <Scalar as Reduce<U256>>::reduce(U256::from_be_slice(&root_private_bytes));
    let inter_scalar = <Scalar as Reduce<U256>>::reduce(U256::from_be_slice(&intermediate_bytes));
    let account_scalar = root_scalar + inter_scalar;

    let account_private_bytes = account_scalar.to_bytes();
    let account_secret = SecretKey::from_slice(&account_private_bytes)
        .map_err(|_| CoreError::KeyDerivationFailed)?;
    let account_public = account_secret.public_key();
    let account_public_compressed = account_public.to_encoded_point(true).as_bytes().to_vec();

    let result_private = account_private_bytes.to_vec();

    // Zeroize intermediate key material before drop
    root_private_bytes.zeroize();
    intermediate_bytes.zeroize();

    Ok((result_private, account_public_compressed))
}

/// Sign a 32-byte prehash with a secp256k1 private key.
/// Returns DER-encoded signature with low-S normalization.
pub fn sign(private_key: &[u8], prehash: &[u8]) -> Result<Vec<u8>, CoreError> {
    if private_key.len() != 32 {
        return Err(CoreError::InvalidKeyLength {
            expected: 32,
            got: private_key.len(),
        });
    }

    let signing_key =
        SigningKey::from_slice(private_key).map_err(|e| CoreError::SigningFailed(e.to_string()))?;

    let (signature, _): (Signature, _) = signing_key
        .sign_prehash(prehash)
        .map_err(|e| CoreError::SigningFailed(e.to_string()))?;

    let normalized = match signature.normalize_s() {
        Some(s) => s,
        None => signature,
    };
    Ok(normalized.to_der().as_bytes().to_vec())
}

/// Verify a DER-encoded secp256k1 signature.
pub fn verify(public_key: &[u8], prehash: &[u8], signature_der: &[u8]) -> Result<bool, CoreError> {
    use k256::ecdsa::signature::hazmat::PrehashVerifier;

    let verifying_key = VerifyingKey::from_sec1_bytes(public_key)
        .map_err(|e| CoreError::SigningFailed(e.to_string()))?;

    let signature =
        Signature::from_der(signature_der).map_err(|e| CoreError::SigningFailed(e.to_string()))?;

    Ok(verifying_key.verify_prehash(prehash, &signature).is_ok())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn secp256k1_derive_known_seed() {
        // Genesis seed: all zeros → should produce known root private key
        let seed = [0u8; 16];
        let (private_key, public_key) = derive_keypair(&seed).unwrap();
        assert_eq!(public_key.len(), 33);
        assert_eq!(private_key.len(), 32);
    }

    #[test]
    fn secp256k1_sign_verify() {
        let seed = [0u8; 16];
        let (private_key, public_key) = derive_keypair(&seed).unwrap();

        let hash = [0xABu8; 32];
        let sig = sign(&private_key, &hash).unwrap();
        assert!(!sig.is_empty());

        let valid = verify(&public_key, &hash, &sig).unwrap();
        assert!(valid);

        let mut bad_hash = hash;
        bad_hash[0] = 0xFF;
        let invalid = verify(&public_key, &bad_hash, &sig).unwrap();
        assert!(!invalid);
    }

    #[test]
    fn secp256k1_known_answer_xrpl_genesis_account() {
        let seed_b58 = "snoPBrXtMeMyMHUVTgbuqAfg1SUTb";
        let expected_pubkey_hex =
            "0330E7FC9D56BB25D6893BA3F317AE5BCF33B3291BD63DB32654A313222F7FD020";
        let expected_address = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";

        let (seed_bytes, key_type) = crate::address::decode_seed(seed_b58).unwrap();
        assert!(matches!(key_type, crate::address::KeyType::Secp256k1));

        let (_privkey, pubkey) = derive_keypair(&seed_bytes).unwrap();

        assert_eq!(
            hex::encode_upper(&pubkey),
            expected_pubkey_hex,
            "secp256k1 public key does not match XRPL known vector"
        );

        let account_id = crate::crypto::signing::public_key_to_account_id(&pubkey);
        let address = crate::address::encode_account_id(&account_id);

        assert_eq!(
            address, expected_address,
            "secp256k1 address does not match XRPL known vector"
        );
    }

    #[test]
    fn secp256k1_produces_low_s_signature() {
        let seed = [0x42u8; 16];
        let message = [0xABu8; 32];

        let (privkey, _) = derive_keypair(&seed).unwrap();
        let sig_bytes = sign(&privkey, &message).unwrap();

        // DER layout: 0x30 [total-len] 0x02 [r-len] [r...] 0x02 [s-len] [s...]
        assert_eq!(sig_bytes[0], 0x30, "DER signature must start with 0x30");
        assert!(sig_bytes.len() >= 8, "signature too short for valid DER");

        let r_len = sig_bytes[3] as usize;
        let s_len_offset = 4 + r_len + 1;
        let s_len = sig_bytes[s_len_offset] as usize;
        let s_start = s_len_offset + 1;
        let s_bytes = &sig_bytes[s_start..s_start + s_len];

        let s_significant = s_bytes.iter().find(|&&b| b != 0).copied().unwrap_or(0);

        assert!(
            s_significant < 0x80,
            "signature S value is high-S — XRPL validators will reject"
        );
    }
}
