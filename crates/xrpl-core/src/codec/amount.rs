use crate::types::{AccountId, Amount, Currency, IouAmount};
use crate::types::{IOU_EXPONENT_MAX, IOU_EXPONENT_MIN, IOU_MANTISSA_MAX, IOU_MANTISSA_MIN};
use crate::CoreError;

/// Maximum valid XRP supply in drops: 100 billion XRP * 10^6 drops per XRP.
/// rippled rejects any transaction with an XRP amount exceeding this value.
pub const MAX_XRP_DROPS: u64 = 100_000_000_000_000_000;

/// Encode XRP amount to 8-byte wire format.
/// Bit 63 = 0 (XRP), Bit 62 = 1 (positive), Bit 61 = 0 (not MPT), Bits 0-60 = drops.
///
/// The wire field holds 61 bits, and serialization must accept the full
/// range: mainnet ledger objects exist whose XRP amounts exceed the
/// nominal supply (e.g. an Offer asking 10× total supply). Supply-range
/// VALIDATION is a transaction-preflight concern, not a codec concern.
pub fn encode_amount_xrp(drops: u64) -> Result<[u8; 8], CoreError> {
    encode_xrp_signed(false, drops)
}

/// Encode XRP with a sign, as `rippled` writes it: a negative amount has
/// bit 62 clear. Zero is always written positive.
pub fn encode_xrp_signed(negative: bool, drops: u64) -> Result<[u8; 8], CoreError> {
    // Bit 61 would mark the amount as an MPT.
    if drops > 0x1FFF_FFFF_FFFF_FFFF {
        return Err(CoreError::InvalidAmount(format!(
            "XRP amount {drops} drops exceeds the 61-bit wire field"
        )));
    }
    let positive = if negative && drops != 0 {
        0
    } else {
        0x4000_0000_0000_0000u64
    };
    Ok((positive | drops).to_be_bytes())
}

/// Decode 8-byte wire format to XRP drops, refusing a negative amount
/// (which [`Amount::Xrp`] cannot hold; [`decode_xrp_signed`] reads one).
pub fn decode_amount_xrp(bytes: &[u8; 8]) -> Result<u64, CoreError> {
    match decode_xrp_signed(bytes)? {
        (false, drops) => Ok(drops),
        (true, drops) => Err(CoreError::InvalidAmount(format!(
            "-{drops} drops: a negative XRP amount"
        ))),
    }
}

/// Decode 8-byte XRP wire format as `rippled` reads it
/// (`STAmount(SerialIter&)`): whether it is negative (bit 62 clear), and the
/// drops. Negative zero is refused, as `rippled` refuses it.
pub fn decode_xrp_signed(bytes: &[u8; 8]) -> Result<(bool, u64), CoreError> {
    let value = u64::from_be_bytes(*bytes);
    if value & 0x8000_0000_0000_0000 != 0 {
        return Err(CoreError::InvalidAmount(
            "bit 63 set — this is an IOU, not XRP".to_string(),
        ));
    }
    if value & 0x2000_0000_0000_0000 != 0 {
        return Err(CoreError::InvalidAmount(
            "bit 61 set — this is an MPT amount, not XRP".to_string(),
        ));
    }
    let drops = value & 0x1FFF_FFFF_FFFF_FFFF;
    let negative = value & 0x4000_0000_0000_0000 == 0;
    if negative && drops == 0 {
        return Err(CoreError::InvalidAmount(
            "negative zero XRP is not canonical; rippled refuses it".to_string(),
        ));
    }
    Ok((negative, drops))
}

/// Encode IOU mantissa/exponent to 8-byte wire format.
/// Bit 63 = 1 (IOU), Bit 62 = positive flag, Bits 54-61 = exponent + 97, Bits 0-53 = mantissa.
pub fn encode_iou_value(iou: &IouAmount) -> Result<[u8; 8], CoreError> {
    if iou.is_zero() {
        // Special zero encoding: bit 63 set, everything else 0
        return Ok(0x8000_0000_0000_0000u64.to_be_bytes());
    }
    // IouAmount's fields are public, so a value made by hand may not be in
    // the canonical form every constructor produces: refuse it rather than
    // write bits rippled would read as another amount.
    if !(IOU_MANTISSA_MIN..IOU_MANTISSA_MAX).contains(&iou.mantissa)
        || !(IOU_EXPONENT_MIN..=IOU_EXPONENT_MAX).contains(&iou.exponent)
    {
        return Err(CoreError::InvalidAmount(format!(
            "token amount {}e{} is not normalized (build it with IouAmount::new or from_decimal)",
            iou.mantissa, iou.exponent
        )));
    }

    let mut value: u64 = 0x8000_0000_0000_0000; // bit 63: not XRP
    if !iou.is_negative {
        value |= 0x4000_0000_0000_0000; // bit 62: positive
    }
    // Bits 54-61: exponent + 97, in 1..=177 for the checked range.
    let biased = u64::try_from(i16::from(iou.exponent) + 97)
        .map_err(|_| CoreError::InvalidAmount("exponent out of range".to_string()))?;
    value |= biased << 54;
    value |= iou.mantissa; // below 10^16 < 2^54
    Ok(value.to_be_bytes())
}

/// Decode 8-byte IOU value to IouAmount.
///
/// Only `rippled`'s canonical form is accepted, as `rippled` accepts only it
/// (`STAmount(SerialIter&)`): a mantissa of 10^15 to 10^16 - 1 with an
/// exponent of -96 to 80, or zero written as exactly `0x8000000000000000`.
pub fn decode_iou_value(bytes: &[u8; 8]) -> Result<IouAmount, CoreError> {
    let value = u64::from_be_bytes(*bytes);

    // Bit 63 must be 1 for IOU
    if value & 0x8000_0000_0000_0000 == 0 {
        return Err(CoreError::InvalidAmount(
            "bit 63 not set — this is XRP, not IOU".to_string(),
        ));
    }

    let mantissa = value & 0x003F_FFFF_FFFF_FFFF;
    if mantissa == 0 {
        if value == 0x8000_0000_0000_0000 {
            return Ok(IouAmount::zero());
        }
        return Err(CoreError::InvalidAmount(format!(
            "token amount 0x{value:016X}: zero must be 0x8000000000000000; rippled refuses other zeros"
        )));
    }

    let is_negative = value & 0x4000_0000_0000_0000 == 0;
    let exponent = ((value >> 54) & 0xFF) as i16 - 97;
    if !(IOU_MANTISSA_MIN..IOU_MANTISSA_MAX).contains(&mantissa)
        || !(i16::from(IOU_EXPONENT_MIN)..=i16::from(IOU_EXPONENT_MAX)).contains(&exponent)
    {
        return Err(CoreError::InvalidAmount(format!(
            "token amount {mantissa}e{exponent} is not in canonical form; rippled refuses it"
        )));
    }

    Ok(IouAmount {
        mantissa,
        exponent: exponent as i8,
        is_negative,
    })
}

/// Encode a full IOU Amount to 48 bytes (8B value + 20B currency + 20B
/// issuer). A value not in canonical form, XRP's currency code or the zero
/// account as issuer is refused, as `rippled` refuses to read them.
pub fn encode_amount_iou(
    iou: &IouAmount,
    currency: &Currency,
    issuer: &AccountId,
) -> Result<[u8; 48], CoreError> {
    if currency.to_bytes() == [0u8; 20] || issuer.as_bytes() == &[0u8; 20] {
        return Err(CoreError::InvalidAmount(
            "a token amount cannot have XRP's currency or issuer (all zeros)".to_string(),
        ));
    }
    let mut buf = [0u8; 48];
    buf[..8].copy_from_slice(&encode_iou_value(iou)?);
    buf[8..28].copy_from_slice(&currency.to_bytes());
    buf[28..48].copy_from_slice(issuer.as_bytes());
    Ok(buf)
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

    // rippled: "invalid native currency" / "invalid native account".
    if currency_bytes == [0u8; 20] || issuer_bytes == [0u8; 20] {
        return Err(CoreError::InvalidAmount(
            "a token amount cannot have XRP's currency or issuer (all zeros)".to_string(),
        ));
    }

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
        } => Ok(encode_amount_iou(value, currency, issuer)?.to_vec()),
        Amount::Mpt { value, issuance_id } => {
            // -i64::MAX..=i64::MAX: i64::MIN's magnitude, 2^63, is past
            // maxMPTokenAmount and rippled refuses it.
            if *value == i64::MIN {
                return Err(CoreError::InvalidAmount(
                    "an MPT amount's magnitude is at most i64::MAX".to_string(),
                ));
            }
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
        let encoded = encode_iou_value(&zero).unwrap();
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
        let encoded = encode_iou_value(&iou).unwrap();
        let decoded = decode_iou_value(&encoded).unwrap();
        assert_eq!(iou.mantissa, decoded.mantissa);
        assert_eq!(iou.exponent, decoded.exponent);
        assert_eq!(iou.is_negative, decoded.is_negative);
    }

    #[test]
    fn iou_negative_roundtrip() {
        let iou = IouAmount::from_decimal("-42.5").unwrap();
        let encoded = encode_iou_value(&iou).unwrap();
        let decoded = decode_iou_value(&encoded).unwrap();
        assert!(decoded.is_negative);
        assert_eq!(decoded.to_decimal(), "-42.5");
    }

    #[test]
    fn full_iou_amount_roundtrip() {
        let iou = IouAmount::from_decimal("100").unwrap();
        let currency = Currency::from_ascii("USD").unwrap();
        let issuer = AccountId::from_address("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap();

        let encoded = encode_amount_iou(&iou, &currency, &issuer).unwrap();
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
        assert!(
            encode_amount_xrp(0x2000_0000_0000_0000).is_err(),
            "bit 61 marks an MPT amount"
        );
    }

    // Each byte form was put to rippled 3.4.1 (`simulate` of a Payment with
    // this Amount): the Amount it showed, or None where it refused the blob.
    #[test]
    fn reads_xrp_as_rippled_does() {
        for (hex_value, shown) in [
            ("4000000000000064", Some("100")),
            ("0000000000000064", Some("-100")),
            ("0000000000000001", Some("-1")),
            ("4000000000000000", Some("0")),
            ("0000000000000000", None),
        ] {
            let bytes: [u8; 8] = hex::decode(hex_value).unwrap().try_into().unwrap();
            let got = decode_xrp_signed(&bytes)
                .ok()
                .map(|(neg, d)| format!("{}{d}", if neg { "-" } else { "" }));
            assert_eq!(got.as_deref(), shown, "{hex_value}");
            if let Ok((neg, d)) = decode_xrp_signed(&bytes) {
                assert_eq!(encode_xrp_signed(neg, d).unwrap(), bytes, "{hex_value}");
            }
        }
        assert!(decode_amount_xrp(&0x0000_0000_0000_0064u64.to_be_bytes()).is_err());
    }

    // As above, for 48-byte token amounts (USD, issued by the genesis account).
    #[test]
    fn reads_token_amounts_as_rippled_does() {
        const USD: &str = "0000000000000000000000005553440000000000";
        const ISSUER: &str = "B5F762798A53D543A014CAF8B297CFF8F2F937E8";
        const ZEROS: &str = "0000000000000000000000000000000000000000";
        for (value, currency, issuer, shown) in [
            ("D4838D7EA4C68000", USD, ISSUER, Some("1")),
            ("94838D7EA4C68000", USD, ISSUER, Some("-1")),
            ("D5038D7EA4C68000", USD, ISSUER, Some("100")),
            ("8000000000000000", USD, ISSUER, Some("0")),
            ("EC6386F26FC0FFFF", USD, ISSUER, Some("9999999999999999e80")),
            ("EC6386F26FC10000", USD, ISSUER, None), // mantissa 10^16
            ("8000004000000000", USD, ISSUER, None), // mantissa 2^38: what the fuzzer found
            ("940000000000000A", USD, ISSUER, None), // mantissa 10
            ("C000000000000000", USD, ISSUER, None), // a zero with the sign bit
            ("8040000000000000", USD, ISSUER, None), // a zero with an exponent
            ("D4838D7EA4C68000", ZEROS, ISSUER, None), // XRP's currency
            ("D4838D7EA4C68000", USD, ZEROS, None),  // XRP's issuer
            ("8000000000000000", ZEROS, ISSUER, None),
        ] {
            let bytes: [u8; 48] = hex::decode(format!("{value}{currency}{issuer}"))
                .unwrap()
                .try_into()
                .unwrap();
            let got = decode_amount_iou(&bytes).ok().map(|(v, ..)| v.to_decimal());
            assert_eq!(got.as_deref(), shown, "{value} {currency} {issuer}");
        }
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

    /// A hand-made IouAmount out of canonical form, an MPT of i64::MIN, or
    /// a token in XRP's currency is refused, never written as other bits.
    #[test]
    fn encoders_refuse_what_rippled_cannot_read() {
        let raw = |mantissa: u64, exponent: i8| IouAmount {
            mantissa,
            exponent,
            is_negative: false,
        };
        assert!(encode_iou_value(&raw(1u64 << 60, 0)).is_err());
        assert!(encode_iou_value(&raw(5, 0)).is_err());
        assert!(encode_iou_value(&raw(IOU_MANTISSA_MIN, 81)).is_err());
        assert!(encode_iou_value(&raw(IOU_MANTISSA_MIN, -97)).is_err());
        assert!(encode_iou_value(&raw(IOU_MANTISSA_MIN, 80)).is_ok());
        let issuer = AccountId::from_address("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap();
        let one = IouAmount::from_decimal("1").unwrap();
        assert!(encode_amount_iou(&one, &Currency::xrp(), &issuer).is_err());
        assert!(encode_amount_iou(
            &one,
            &Currency::from_ascii("USD").unwrap(),
            &AccountId::from_bytes([0; 20])
        )
        .is_err());
        let mpt = |value| Amount::Mpt {
            value,
            issuance_id: crate::types::Hash192::from_hex(
                "00000001B5F762798A53D543A014CAF8B297CFF8F2F937E8",
            )
            .unwrap(),
        };
        assert!(encode_amount(&mpt(i64::MIN)).is_err());
        assert!(encode_amount(&mpt(-i64::MAX)).is_ok());
    }
}
