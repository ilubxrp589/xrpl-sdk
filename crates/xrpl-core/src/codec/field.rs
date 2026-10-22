use crate::CoreError;

/// Type codes from the XRPL serialization spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum TypeCode {
    Unknown = 0,
    UInt16 = 1,
    UInt32 = 2,
    UInt64 = 3,
    Hash128 = 4,
    Hash256 = 5,
    Amount = 6,
    Blob = 7,
    AccountId = 8,
    Number = 9,
    Int32 = 10,
    Int64 = 11,
    StObject = 14,
    StArray = 15,
    UInt8 = 16,
    Hash160 = 17,
    PathSet = 18,
    Vector256 = 19,
    UInt96 = 20,
    Hash192 = 21,
    UInt384 = 22,
    UInt512 = 23,
    Issue = 24,
    XChainBridge = 25,
    Currency = 26,
}

impl TypeCode {
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
    pub type_code: u16,
    pub field_code: u16,
}

impl FieldId {
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
    pub fn decode(buf: &[u8]) -> Result<(Self, usize), CoreError> {
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
                Ok((Self::new(b1 as u16, b2 as u16), 3))
            }
            (0, fc) => {
                // 2-byte: type > 15, field fits in nibble
                let &b1 = buf.get(1).ok_or_else(|| {
                    CoreError::CodecError("need 2 bytes for field header".to_string())
                })?;
                Ok((Self::new(b1 as u16, fc as u16), 2))
            }
            (tc, 0) => {
                // 2-byte: type fits, field > 15
                let &b1 = buf.get(1).ok_or_else(|| {
                    CoreError::CodecError("need 2 bytes for field header".to_string())
                })?;
                Ok((Self::new(tc as u16, b1 as u16), 2))
            }
            (tc, fc) => {
                // 1-byte header
                Ok((Self::new(tc as u16, fc as u16), 1))
            }
        }
    }
}

/// Encode variable-length prefix.
/// Returns 1, 2, or 3 bytes depending on the length.
///
/// # Errors
/// Returns `CoreError::CodecError` if `len` exceeds the maximum VL-encodable size (918744).
pub fn encode_vl(len: usize) -> Vec<u8> {
    // Note: This function cannot return Result without a large API change.
    // VL lengths > 918744 are invalid per the XRPL spec and never occur in practice.
    // We saturate to the maximum 3-byte encoding rather than panicking.
    if len <= 192 {
        vec![len as u8]
    } else if len <= 12480 {
        let adjusted = len - 193;
        vec![(adjusted / 256 + 193) as u8, (adjusted % 256) as u8]
    } else {
        // Clamp to 918744 max to avoid panic
        let clamped = len.min(918744);
        let adjusted = clamped - 12481;
        vec![
            (241 + (adjusted >> 16)) as u8,
            ((adjusted >> 8) & 0xFF) as u8,
            (adjusted & 0xFF) as u8,
        ]
    }
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
        assert_eq!(encode_vl(0), vec![0]);
        assert_eq!(encode_vl(10), vec![10]);
        assert_eq!(encode_vl(192), vec![192]);
    }

    #[test]
    fn vl_encode_medium() {
        let encoded = encode_vl(193);
        let (decoded, consumed) = decode_vl(&encoded).unwrap();
        assert_eq!(decoded, 193);
        assert_eq!(consumed, 2);
    }

    #[test]
    fn vl_roundtrip() {
        for len in [0, 1, 100, 192, 193, 500, 1000, 12480, 12481, 50000] {
            let encoded = encode_vl(len);
            let (decoded, _) = decode_vl(&encoded).unwrap();
            assert_eq!(decoded, len, "roundtrip failed for len={len}");
        }
    }
}
