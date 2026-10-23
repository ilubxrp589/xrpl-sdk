use crate::CoreError;

/// Type codes from the XRPL serialization spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum TypeCode {
    /// The `Unknown` field type (code 0).
    Unknown = 0,
    /// The `UInt16` field type (code 1).
    UInt16 = 1,
    /// The `UInt32` field type (code 2).
    UInt32 = 2,
    /// The `UInt64` field type (code 3).
    UInt64 = 3,
    /// The `Hash128` field type (code 4).
    Hash128 = 4,
    /// The `Hash256` field type (code 5).
    Hash256 = 5,
    /// The `Amount` field type (code 6).
    Amount = 6,
    /// The `Blob` field type (code 7).
    Blob = 7,
    /// The `AccountId` field type (code 8).
    AccountId = 8,
    /// The `Number` field type (code 9).
    Number = 9,
    /// The `Int32` field type (code 10).
    Int32 = 10,
    /// The `Int64` field type (code 11).
    Int64 = 11,
    /// The `StObject` field type (code 14).
    StObject = 14,
    /// The `StArray` field type (code 15).
    StArray = 15,
    /// The `UInt8` field type (code 16).
    UInt8 = 16,
    /// The `Hash160` field type (code 17).
    Hash160 = 17,
    /// The `PathSet` field type (code 18).
    PathSet = 18,
    /// The `Vector256` field type (code 19).
    Vector256 = 19,
    /// The `UInt96` field type (code 20).
    UInt96 = 20,
    /// The `Hash192` field type (code 21).
    Hash192 = 21,
    /// The `UInt384` field type (code 22).
    UInt384 = 22,
    /// The `UInt512` field type (code 23).
    UInt512 = 23,
    /// The `Issue` field type (code 24).
    Issue = 24,
    /// The `XChainBridge` field type (code 25).
    XChainBridge = 25,
    /// The `Currency` field type (code 26).
    Currency = 26,
}

impl TypeCode {
    /// The type with this code, if known.
    pub fn from_u16(v: u16) -> Option<Self> {
        match v {
            1 => Some(Self::UInt16),
            2 => Some(Self::UInt32),
            3 => Some(Self::UInt64),
            4 => Some(Self::Hash128),
            5 => Some(Self::Hash256),
            6 => Some(Self::Amount),
            7 => Some(Self::Blob),
            8 => Some(Self::AccountId),
            9 => Some(Self::Number),
            10 => Some(Self::Int32),
            11 => Some(Self::Int64),
            14 => Some(Self::StObject),
            15 => Some(Self::StArray),
            16 => Some(Self::UInt8),
            17 => Some(Self::Hash160),
            18 => Some(Self::PathSet),
            19 => Some(Self::Vector256),
            20 => Some(Self::UInt96),
            21 => Some(Self::Hash192),
            22 => Some(Self::UInt384),
            23 => Some(Self::UInt512),
            24 => Some(Self::Issue),
            25 => Some(Self::XChainBridge),
            26 => Some(Self::Currency),
            _ => None,
        }
    }

    /// Returns true if this type uses variable-length prefix encoding.
    pub fn is_vl_encoded(self) -> bool {
        matches!(self, Self::Blob | Self::AccountId | Self::Vector256)
    }
}

/// A field identifier combining type code and field code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldId {
    /// The type's code.
    pub type_code: u16,
    /// The field's code within its type.
    pub field_code: u16,
}

impl FieldId {
    /// A field ID from its type and field codes.
    pub fn new(type_code: u16, field_code: u16) -> Self {
        Self {
            type_code,
            field_code,
        }
    }

    /// Sort key for canonical field ordering: (type_code, field_code).
    pub fn sort_key(&self) -> (u16, u16) {
        (self.type_code, self.field_code)
    }

    /// Encode field header to 1, 2, or 3 bytes per XRPL spec.
    #[allow(
        clippy::cast_possible_truncation,
        reason = "only serialized fields are encoded; their codes are all below 256 (test every_code_fits_a_byte)"
    )]
    pub fn encode(&self) -> Vec<u8> {
        let tc = self.type_code;
        let fc = self.field_code;

        if tc < 16 && fc < 16 {
            // 1-byte header: both fit in 4 bits
            vec![((tc as u8) << 4) | (fc as u8)]
        } else if tc >= 16 && fc < 16 {
            // 2-byte: type > 15, field fits
            vec![fc as u8, tc as u8]
        } else if tc < 16 && fc >= 16 {
            // 2-byte: type fits, field > 15
            vec![(tc as u8) << 4, fc as u8]
        } else {
            // 3-byte: both > 15
            vec![0x00, tc as u8, fc as u8]
        }
    }

    /// Decode field header from bytes. Returns (FieldId, bytes_consumed).
    ///
    /// Only the shortest header for each code is read, as `rippled`'s
    /// `SerialIter::getFieldID` requires: a type or name byte in a longer
    /// header must be 16 or more ("uncommon type/name out of range").
    pub fn decode(buf: &[u8]) -> Result<(Self, usize), CoreError> {
        let uncommon = |b: u8, what: &str| -> Result<u16, CoreError> {
            if b < 16 {
                Err(CoreError::CodecError(format!(
                    "non-canonical field header: {what} {b} fits in the first byte"
                )))
            } else {
                Ok(u16::from(b))
            }
        };
        let &byte0 = buf
            .first()
            .ok_or_else(|| CoreError::CodecError("empty buffer for field header".to_string()))?;
        let type_nibble = (byte0 >> 4) & 0x0F;
        let field_nibble = byte0 & 0x0F;

        match (type_nibble, field_nibble) {
            (0, 0) => {
                // 3-byte header: both > 15
                let &b1 = buf.get(1).ok_or_else(|| {
                    CoreError::CodecError("need 3 bytes for field header".to_string())
                })?;
                let &b2 = buf.get(2).ok_or_else(|| {
                    CoreError::CodecError("need 3 bytes for field header".to_string())
                })?;
                Ok((Self::new(uncommon(b1, "type")?, uncommon(b2, "name")?), 3))
            }
            (0, fc) => {
                // 2-byte: type > 15, field fits in nibble
                let &b1 = buf.get(1).ok_or_else(|| {
                    CoreError::CodecError("need 2 bytes for field header".to_string())
                })?;
                Ok((Self::new(uncommon(b1, "type")?, u16::from(fc)), 2))
            }
            (tc, 0) => {
                // 2-byte: type fits, field > 15
                let &b1 = buf.get(1).ok_or_else(|| {
                    CoreError::CodecError("need 2 bytes for field header".to_string())
                })?;
                Ok((Self::new(u16::from(tc), uncommon(b1, "name")?), 2))
            }
            (tc, fc) => {
                // 1-byte header
                Ok((Self::new(u16::from(tc), u16::from(fc)), 1))
            }
        }
    }
}

/// The longest length a VL prefix can carry (`Serializer::addEncoded`).
pub const MAX_VL_LENGTH: usize = 918_744;

/// The variable-length prefix for `len` bytes: 1, 2 or 3 bytes, as
/// `rippled`'s `Serializer::addEncoded` writes it.
///
/// # Errors
///
/// [`CoreError::CodecError`] when `len` is over [`MAX_VL_LENGTH`]: no prefix
/// can say it, and writing a shorter one would let the bytes after it be
/// read as other fields.
pub fn encode_vl(len: usize) -> Result<Vec<u8>, CoreError> {
    let byte = |n: usize| u8::try_from(n).map_err(|_| too_long(len));
    if len <= 192 {
        Ok(vec![byte(len)?])
    } else if len <= 12_480 {
        let adjusted = len - 193;
        Ok(vec![byte(adjusted / 256 + 193)?, byte(adjusted % 256)?])
    } else if len <= MAX_VL_LENGTH {
        let adjusted = len - 12_481;
        Ok(vec![
            byte(241 + (adjusted >> 16))?,
            byte((adjusted >> 8) & 0xFF)?,
            byte(adjusted & 0xFF)?,
        ])
    } else {
        Err(too_long(len))
    }
}

fn too_long(len: usize) -> CoreError {
    CoreError::CodecError(format!(
        "a variable-length field holds at most {MAX_VL_LENGTH} bytes, got {len}"
    ))
}

/// Decode variable-length prefix. Returns (length, bytes_consumed).
pub fn decode_vl(buf: &[u8]) -> Result<(usize, usize), CoreError> {
    let &b0_byte = buf
        .first()
        .ok_or_else(|| CoreError::CodecError("empty buffer for VL prefix".to_string()))?;
    let b0 = b0_byte as usize;

    if b0 <= 192 {
        Ok((b0, 1))
    } else if b0 <= 240 {
        let &b1_byte = buf
            .get(1)
            .ok_or_else(|| CoreError::CodecError("need 2 bytes for VL prefix".to_string()))?;
        let b1 = b1_byte as usize;
        let len = 193 + ((b0 - 193) * 256) + b1;
        Ok((len, 2))
    } else if b0 <= 254 {
        let &b1_byte = buf
            .get(1)
            .ok_or_else(|| CoreError::CodecError("need 3 bytes for VL prefix".to_string()))?;
        let &b2_byte = buf
            .get(2)
            .ok_or_else(|| CoreError::CodecError("need 3 bytes for VL prefix".to_string()))?;
        let b1 = b1_byte as usize;
        let b2 = b2_byte as usize;
        let len = 12481 + ((b0 - 241) << 16) + (b1 << 8) + b2;
        Ok((len, 3))
    } else {
        Err(CoreError::CodecError(format!(
            "invalid VL prefix byte: {b0}"
        )))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn field_id_1_byte() {
        // TransactionType: type=1 (UInt16), field=2
        let fid = FieldId::new(1, 2);
        let encoded = fid.encode();
        assert_eq!(encoded, vec![0x12]); // (1 << 4) | 2 = 0x12
        let (decoded, consumed) = FieldId::decode(&encoded).unwrap();
        assert_eq!(decoded, fid);
        assert_eq!(consumed, 1);
    }

    #[test]
    fn field_id_2_byte_type_large() {
        // UInt8 type=16, TickSize field=16
        let fid = FieldId::new(16, 16);
        let encoded = fid.encode();
        // type > 15, field > 15 → 3-byte
        assert_eq!(encoded, vec![0x00, 16, 16]);
        let (decoded, consumed) = FieldId::decode(&encoded).unwrap();
        assert_eq!(decoded, fid);
        assert_eq!(consumed, 3);
    }

    #[test]
    fn field_id_2_byte_field_large() {
        // type=2 (UInt32), field=27 (LastLedgerSequence)
        let fid = FieldId::new(2, 27);
        let encoded = fid.encode();
        // type < 16, field > 15 → 2-byte: [(type << 4), field]
        assert_eq!(encoded, vec![0x20, 27]);
        let (decoded, consumed) = FieldId::decode(&encoded).unwrap();
        assert_eq!(decoded, fid);
        assert_eq!(consumed, 2);
    }

    #[test]
    fn vl_encode_short() {
        assert_eq!(encode_vl(0).unwrap(), vec![0]);
        assert_eq!(encode_vl(10).unwrap(), vec![10]);
        assert_eq!(encode_vl(192).unwrap(), vec![192]);
    }

    #[test]
    fn vl_encode_medium() {
        let encoded = encode_vl(193).unwrap();
        let (decoded, consumed) = decode_vl(&encoded).unwrap();
        assert_eq!(decoded, 193);
        assert_eq!(consumed, 2);
    }

    #[test]
    fn vl_roundtrip() {
        for len in [
            0,
            1,
            100,
            192,
            193,
            500,
            1000,
            12480,
            12481,
            50000,
            MAX_VL_LENGTH,
        ] {
            let encoded = encode_vl(len).unwrap();
            let (decoded, _) = decode_vl(&encoded).unwrap();
            assert_eq!(decoded, len, "roundtrip failed for len={len}");
        }
    }

    #[test]
    fn vl_refuses_a_length_no_prefix_can_say() {
        assert!(encode_vl(MAX_VL_LENGTH + 1).is_err());
        assert!(encode_vl(usize::MAX).is_err());
    }

    /// A longer header than the code needs is refused, as rippled refuses it.
    #[test]
    fn only_the_shortest_header_is_read() {
        // Sequence (UInt32 = 2, field 4) is 0x24; the same in longer forms:
        for long in [&[0x00, 0x02, 0x04][..], &[0x04, 0x02], &[0x20, 0x04]] {
            assert!(FieldId::decode(long).is_err(), "{long:02X?}");
        }
        assert_eq!(FieldId::decode(&[0x24]).unwrap(), (FieldId::new(2, 4), 1));
        // Uncommon codes keep their long forms.
        assert_eq!(
            FieldId::decode(&[0x01, 0x10]).unwrap(),
            (FieldId::new(16, 1), 2)
        );
        assert_eq!(
            FieldId::decode(&[0x20, 0x1B]).unwrap(),
            (FieldId::new(2, 27), 2)
        );
        assert_eq!(
            FieldId::decode(&[0x00, 0x10, 0x10]).unwrap(),
            (FieldId::new(16, 16), 3)
        );
    }

    /// FieldId::encode writes each code in a byte.
    #[test]
    fn every_code_fits_a_byte() {
        let defs: serde_json::Value =
            serde_json::from_str(crate::codec::definitions::DEFINITIONS_JSON).unwrap();
        for (name, def) in defs["FIELDS"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| (&f[0], &f[1]))
        {
            // Only serialized fields are written (hash, index, … are not).
            if def["isSerialized"] == true {
                assert!(def["nth"].as_i64().unwrap() < 256, "{name}");
            }
        }
        for code in defs["TYPES"].as_object().unwrap().values() {
            let code = code.as_i64().unwrap();
            assert!(!(256..10_000).contains(&code), "{code}");
        }
    }
}
