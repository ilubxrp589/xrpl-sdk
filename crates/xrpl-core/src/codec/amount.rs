use crate::types::{AccountId, Amount, Currency, IouAmount};
use crate::CoreError;

/// Maximum valid XRP supply in drops: 100 billion XRP * 10^6 drops per XRP.
/// rippled rejects any transaction with an XRP amount exceeding this value.
pub const MAX_XRP_DROPS: u64 = 100_000_000_000_000_000;

/// Encode XRP amount to 8-byte wire format.
/// Bit 63 = 0 (XRP), Bit 62 = 1 (positive), Bits 0-61 = drops.
///
/// The wire field holds 62 bits, and serialization must accept the full
/// range: mainnet ledger objects exist whose XRP amounts exceed the
/// nominal supply (e.g. an Offer asking 10× total supply). Supply-range
/// VALIDATION is a transaction-preflight concern, not a codec concern.
pub fn encode_amount_xrp(drops: u64) -> Result<[u8; 8], CoreError> {
    if drops > 0x3FFF_FFFF_FFFF_FFFF {
        return Err(CoreError::InvalidAmount(format!(
            "XRP amount {drops} drops exceeds the 62-bit wire field"
        )));
    }
    // Set bit 62 (positive flag). Bit 63 stays 0 (XRP marker).
    let value = 0x4000_0000_0000_0000u64 | drops;
    Ok(value.to_be_bytes())
}

/// Decode 8-byte wire format to XRP drops.
pub fn decode_amount_xrp(bytes: &[u8; 8]) -> Result<u64, CoreError> {
    let value = u64::from_be_bytes(*bytes);

    // Bit 63 must be 0 for XRP
    if value & 0x8000_0000_0000_0000 != 0 {
        return Err(CoreError::InvalidAmount(
            "bit 63 set — this is an IOU, not XRP".to_string(),
        ));
    }

    // Extract drops (bits 0-61, mask out bit 62 positive flag)
    let drops = value & 0x3FFF_FFFF_FFFF_FFFF;
    Ok(drops)
}

/// Encode IOU mantissa/exponent to 8-byte wire format.
/// Bit 63 = 1 (IOU), Bit 62 = positive flag, Bits 54-61 = exponent + 97, Bits 0-53 = mantissa.
pub fn encode_iou_value(iou: &IouAmount) -> [u8; 8] {
    if iou.is_zero() {
        // Special zero encoding: bit 63 set, everything else 0
        return 0x8000_0000_0000_0000u64.to_be_bytes();
    }

    let mut value: u64 = 0;

    // Bit 63 = 1 (IOU marker)
    value |= 0x8000_0000_0000_0000;

    // Bit 62 = positive flag
    if !iou.is_negative {
        value |= 0x4000_0000_0000_0000;
    }

    // Bits 54-61 = exponent + 97 (8-bit biased)
    let biased_exp = (iou.exponent as i16 + 97) as u64;
    value |= (biased_exp & 0xFF) << 54;

    // Bits 0-53 = mantissa
    value |= iou.mantissa & 0x003F_FFFF_FFFF_FFFF;

    value.to_be_bytes()
}

/// Decode 8-byte IOU value to IouAmount.
pub fn decode_iou_value(bytes: &[u8; 8]) -> Result<IouAmount, CoreError> {
    let value = u64::from_be_bytes(*bytes);

    // Bit 63 must be 1 for IOU
    if value & 0x8000_0000_0000_0000 == 0 {
        return Err(CoreError::InvalidAmount(
            "bit 63 not set — this is XRP, not IOU".to_string(),
        ));
    }

    // Check for zero encoding
    if value == 0x8000_0000_0000_0000 {
        return Ok(IouAmount::zero());
    }

    let is_negative = value & 0x4000_0000_0000_0000 == 0;
    let biased_exp = ((value >> 54) & 0xFF) as i16;
    let exponent = (biased_exp - 97) as i8;
    let mantissa = value & 0x003F_FFFF_FFFF_FFFF;

    Ok(IouAmount {
        mantissa,
        exponent,
        is_negative,
    })
}

/// Encode a full IOU Amount to 48 bytes (8B value + 20B currency + 20B issuer).
pub fn encode_amount_iou(iou: &IouAmount, currency: &Currency, issuer: &AccountId) -> [u8; 48] {
    let mut buf = [0u8; 48];
    let value_bytes = encode_iou_value(iou);
    buf[..8].copy_from_slice(&value_bytes);
    buf[8..28].copy_from_slice(&currency.to_bytes());
    buf[28..48].copy_from_slice(issuer.as_bytes());
    buf
}

/// Decode 48 bytes to (IouAmount, Currency, AccountId).
pub fn decode_amount_iou(bytes: &[u8; 48]) -> Result<(IouAmount, Currency, AccountId), CoreError> {
    let mut value_bytes = [0u8; 8];
    value_bytes.copy_from_slice(&bytes[..8]);
    let iou = decode_iou_value(&value_bytes)?;

    let mut currency_bytes = [0u8; 20];
    currency_bytes.copy_from_slice(&bytes[8..28]);
    let currency = Currency::from_bytes(currency_bytes);

    let mut issuer_bytes = [0u8; 20];
    issuer_bytes.copy_from_slice(&bytes[28..48]);
    let issuer = AccountId::from_bytes(issuer_bytes);

    Ok((iou, currency, issuer))
}

/// Encode an Amount (XRP or IOU) to wire format bytes.
pub fn encode_amount(amount: &Amount) -> Result<Vec<u8>, CoreError> {
    match amount {
        Amount::Xrp(drops) => Ok(encode_amount_xrp(*drops)?.to_vec()),
        Amount::Iou {
            value,
            currency,
            issuer,
        } => Ok(encode_amount_iou(value, currency, issuer).to_vec()),
        Amount::Mpt { value, issuance_id } => {
            // STAmount::add: lead byte 0x20 (MPT) | 0x40 unless negative,
            // the 8-byte magnitude, then the 24-byte MPTokenIssuanceID.
            let mut out = Vec::with_capacity(33);
            out.push(if *value >= 0 { 0x60 } else { 0x20 });
            out.extend_from_slice(&value.unsigned_abs().to_be_bytes());
            out.extend_from_slice(issuance_id.as_bytes());
            Ok(out)
        }
    }
}

/// Decode wire format bytes to Amount.
pub fn decode_amount(bytes: &[u8]) -> Result<Amount, CoreError> {
    if bytes.len() < 8 {
        return Err(CoreError::InvalidAmount(format!(
            "amount too short: {} bytes",
            bytes.len()
        )));
    }

    // Check bit 63 to determine XRP vs IOU
    let first_byte = *bytes
        .first()
        .ok_or_else(|| CoreError::InvalidAmount("amount buffer is empty".to_string()))?;
    if first_byte & 0x80 == 0 && first_byte & 0x20 != 0 {
        // MPT: lead byte + 8-byte magnitude + 24-byte MPTokenIssuanceID
        let arr: &[u8; 33] = bytes.try_into().map_err(|_| {
            CoreError::InvalidAmount(format!("MPT amount must be 33 bytes, got {}", bytes.len()))
        })?;
        let [lead, rest @ ..] = arr;
        let (magnitude, id) = rest
            .split_first_chunk::<8>()
            .ok_or_else(|| CoreError::InvalidAmount("MPT amount magnitude".to_string()))?;
        let magnitude = u64::from_be_bytes(*magnitude);
        let magnitude = i64::try_from(magnitude).map_err(|_| {
            CoreError::InvalidAmount(format!("MPT amount {magnitude} out of range"))
        })?;
        let mut issuance_id = [0u8; 24];
        issuance_id.copy_from_slice(id);
        let value = if lead & 0x40 == 0 {
            -magnitude
        } else {
            magnitude
        };
        Ok(Amount::Mpt {
            value,
            issuance_id: crate::types::Hash192(issuance_id),
        })
    } else if first_byte & 0x80 == 0 {
        // XRP: 8 bytes
        if bytes.len() != 8 {
            return Err(CoreError::InvalidAmount(format!(
                "XRP amount must be 8 bytes, got {}",
                bytes.len()
            )));
        }
        let mut arr = [0u8; 8];
        arr.copy_from_slice(bytes);
        let drops = decode_amount_xrp(&arr)?;
        Ok(Amount::Xrp(drops))
    } else {
        // IOU: 48 bytes
        if bytes.len() != 48 {
            return Err(CoreError::InvalidAmount(format!(
                "IOU amount must be 48 bytes, got {}",
                bytes.len()
            )));
        }
        let mut arr = [0u8; 48];
        arr.copy_from_slice(bytes);
        let (value, currency, issuer) = decode_amount_iou(&arr)?;
        Ok(Amount::Iou {
            value,
            currency,
            issuer,
        })
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic
)]
mod tests {
    use super::*;

    #[test]
    fn mpt_amount_matches_the_json_codec() {
        let json = serde_json::json!({"mpt_issuance_id": "0000012FFD9EE5DA93AC614B4DB94D7E0FCE415CA51BED47", "value": "-100"});
        let amount: Amount = serde_json::from_value(json.clone()).unwrap();
        let bytes = encode_amount(&amount).unwrap();
        let via_json =
            crate::codec::encode_transaction_json(&serde_json::json!({"Amount": json}), false)
                .unwrap();
        assert_eq!(via_json[1..], bytes[..]);
        assert_eq!(decode_amount(&bytes).unwrap(), amount);
    }

    #[test]
    fn encode_xrp_1_xrp() {
        // 1 XRP = 1,000,000 drops
        let encoded = encode_amount_xrp(1_000_000).unwrap();
        assert_eq!(encoded, [0x40, 0x00, 0x00, 0x00, 0x00, 0x0F, 0x42, 0x40]);
    }

    #[test]
    fn encode_xrp_zero() {
        let encoded = encode_amount_xrp(0).unwrap();
        assert_eq!(encoded, [0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn encode_xrp_10_xrp() {
        // 10 XRP = 10,000,000 drops = 0x989680
        let encoded = encode_amount_xrp(10_000_000).unwrap();
        assert_eq!(encoded, [0x40, 0x00, 0x00, 0x00, 0x00, 0x98, 0x96, 0x80]);
    }

    #[test]
    fn decode_xrp_roundtrip() {
        let drops = 12_345_678u64;
        let encoded = encode_amount_xrp(drops).unwrap();
        let decoded = decode_amount_xrp(&encoded).unwrap();
        assert_eq!(decoded, drops);
    }

    #[test]
    fn encode_iou_zero() {
        let zero = IouAmount::zero();
        let encoded = encode_iou_value(&zero);
        assert_eq!(encoded, [0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn decode_iou_zero() {
        let bytes = [0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00u8];
        let iou = decode_iou_value(&bytes).unwrap();
        assert!(iou.is_zero());
    }

    #[test]
    fn iou_value_roundtrip() {
        let iou = IouAmount::from_decimal("1.5").unwrap();
        let encoded = encode_iou_value(&iou);
        let decoded = decode_iou_value(&encoded).unwrap();
        assert_eq!(iou.mantissa, decoded.mantissa);
        assert_eq!(iou.exponent, decoded.exponent);
        assert_eq!(iou.is_negative, decoded.is_negative);
    }

    #[test]
    fn iou_negative_roundtrip() {
        let iou = IouAmount::from_decimal("-42.5").unwrap();
        let encoded = encode_iou_value(&iou);
        let decoded = decode_iou_value(&encoded).unwrap();
        assert!(decoded.is_negative);
        assert_eq!(decoded.to_decimal(), "-42.5");
    }

    #[test]
    fn full_iou_amount_roundtrip() {
        let iou = IouAmount::from_decimal("100").unwrap();
        let currency = Currency::from_ascii("USD").unwrap();
        let issuer = AccountId::from_address("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap();

        let encoded = encode_amount_iou(&iou, &currency, &issuer);
        assert_eq!(encoded.len(), 48);

        let (dec_iou, dec_currency, dec_issuer) = decode_amount_iou(&encoded).unwrap();
        assert_eq!(dec_iou.to_decimal(), "100");
        assert_eq!(dec_currency, currency);
        assert_eq!(dec_issuer, issuer);
    }

    #[test]
    fn amount_enum_xrp_roundtrip() {
        let amt = Amount::Xrp(1_000_000);
        let encoded = encode_amount(&amt).unwrap();
        let decoded = decode_amount(&encoded).unwrap();
        assert_eq!(decoded, amt);
    }

    #[test]
    fn amount_enum_iou_roundtrip() {
        let amt = Amount::Iou {
            value: IouAmount::from_decimal("50.25").unwrap(),
            currency: Currency::from_ascii("EUR").unwrap(),
            issuer: AccountId::from_address("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap(),
        };
        let encoded = encode_amount(&amt).unwrap();
        assert_eq!(encoded.len(), 48);
        let decoded = decode_amount(&encoded).unwrap();
        match &decoded {
            Amount::Iou { value, .. } => assert_eq!(value.to_decimal(), "50.25"),
            _ => panic!("expected IOU"),
        }
    }

    #[test]
    fn encode_xrp_above_supply_roundtrips() {
        // Mainnet ledger objects can carry XRP amounts above nominal supply
        // (an Offer asking 10× supply exists at #106455034); the codec must
        // round-trip the full 62-bit wire range and reject only overflow.
        let bytes = encode_amount_xrp(MAX_XRP_DROPS + 1).expect("wire range accepts above-supply");
        assert_eq!(decode_amount_xrp(&bytes).unwrap(), MAX_XRP_DROPS + 1);
        assert!(
            encode_amount_xrp(0x4000_0000_0000_0000).is_err(),
            "63rd bit must reject"
        );
    }

    #[test]
    fn encode_xrp_at_max_supply_accepted() {
        let result = encode_amount_xrp(MAX_XRP_DROPS);
        assert!(result.is_ok(), "must accept max supply exactly");
    }

    #[test]
    fn encode_xrp_zero_accepted() {
        let result = encode_amount_xrp(0);
        assert!(result.is_ok(), "zero drops is valid");
    }

    #[test]
    fn encode_xrp_u64_max_rejected() {
        let result = encode_amount_xrp(u64::MAX);
        assert!(result.is_err(), "u64::MAX must be rejected");
    }
}
