use crate::CoreError;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Parsed field definition from definitions.json.
#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: String,
    pub type_code: u16,
    pub field_code: u16,
    pub is_vl_encoded: bool,
    pub is_serialized: bool,
    pub is_signing: bool,
}

/// Parse the embedded definitions.json once and return the Value.
/// Returns an empty object if parsing fails (should never happen with embedded JSON).
fn parse_definitions() -> Value {
    serde_json::from_str(DEFINITIONS_JSON).unwrap_or_default()
}

/// Transaction type name → numeric code mapping.
static TRANSACTION_TYPES: LazyLock<HashMap<String, u16>> = LazyLock::new(|| {
    let defs = parse_definitions();
    let Some(tt) = defs.get("TRANSACTION_TYPES").and_then(|v| v.as_object()) else {
        return HashMap::new();
    };
    tt.iter()
        .filter_map(|(name, val)| {
            let code = val.as_i64()?;
            if code < 0 {
                return None;
            }
            Some((name.clone(), code as u16))
        })
        .collect()
});

/// Ledger entry type name → numeric code mapping.
static LEDGER_ENTRY_TYPES: LazyLock<HashMap<String, u16>> = LazyLock::new(|| {
    let defs = parse_definitions();
    let Some(tt) = defs.get("LEDGER_ENTRY_TYPES").and_then(|v| v.as_object()) else {
        return HashMap::new();
    };
    tt.iter()
        .filter_map(|(name, val)| {
            let code = val.as_i64()?;
            if code < 0 {
                return None;
            }
            Some((name.clone(), code as u16))
        })
        .collect()
});

/// Type name → type code mapping.
static TYPE_CODES: LazyLock<HashMap<String, u16>> = LazyLock::new(|| {
    let defs = parse_definitions();
    let Some(types) = defs.get("TYPES").and_then(|v| v.as_object()) else {
        return HashMap::new();
    };
    types
        .iter()
        .filter_map(|(name, val)| {
            let code = val.as_i64()?;
            if code < 0 {
                return None;
            }
            Some((name.clone(), code as u16))
        })
        .collect()
});

/// All field definitions, keyed by field name.
static FIELD_MAP: LazyLock<HashMap<String, FieldDef>> = LazyLock::new(|| {
    let defs = parse_definitions();
    let Some(fields) = defs.get("FIELDS").and_then(|v| v.as_array()) else {
        return HashMap::new();
    };
    let type_codes = &*TYPE_CODES;

    let mut map = HashMap::new();
    for entry in fields {
        let Some(arr) = entry.as_array() else {
            continue;
        };
        if arr.len() != 2 {
            continue;
        }
        let name = match arr.first().and_then(|v| v.as_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        let Some(props) = arr.get(1) else {
            continue;
        };
        let type_name = match props["type"].as_str() {
            Some(t) => t,
            None => continue,
        };
        let tc = match type_codes.get(type_name) {
            Some(&c) => c,
            None => continue,
        };
        let fc = match props["nth"].as_i64() {
            Some(n) if n >= 0 => n as u16,
            _ => continue,
        };
        let is_vl = props["isVLEncoded"].as_bool().unwrap_or(false);
        let is_serialized = props["isSerialized"].as_bool().unwrap_or(false);
        let is_signing = props["isSigningField"].as_bool().unwrap_or(false);

        map.insert(
            name.clone(),
            FieldDef {
                name,
                type_code: tc,
                field_code: fc,
                is_vl_encoded: is_vl,
                is_serialized,
                is_signing,
            },
        );
    }
    map
});

/// All field definitions keyed by (type_code, field_code).
static FIELD_BY_ID: LazyLock<HashMap<(u16, u16), FieldDef>> = LazyLock::new(|| {
    FIELD_MAP
        .values()
        .map(|fd| ((fd.type_code, fd.field_code), fd.clone()))
        .collect()
});

const DEFINITIONS_JSON: &str = include_str!("../../../../tests/vectors/definitions.json");

/// Look up a field definition by name.
pub fn lookup_field_def(name: &str) -> Option<&'static FieldDef> {
    FIELD_MAP.get(name)
}

/// Look up a field definition by (type_code, field_code).
pub fn lookup_field_def_by_id(type_code: u16, field_code: u16) -> Option<&'static FieldDef> {
    FIELD_BY_ID.get(&(type_code, field_code))
}

/// Convert transaction type name to numeric code.
pub fn transaction_type_code(name: &str) -> Result<u16, CoreError> {
    TRANSACTION_TYPES
        .get(name)
        .copied()
        .ok_or_else(|| CoreError::CodecError(format!("unknown transaction type: {name}")))
}

/// Convert ledger entry type name to numeric code.
pub fn ledger_entry_type_code(name: &str) -> Result<u16, CoreError> {
    LEDGER_ENTRY_TYPES
        .get(name)
        .copied()
        .ok_or_else(|| CoreError::CodecError(format!("unknown ledger entry type: {name}")))
}

/// Granular permission names → PermissionValue codes.
/// These are specific sub-transaction permissions defined in XLS-74d.
static GRANULAR_PERMISSIONS: LazyLock<HashMap<&'static str, u32>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("TrustlineAuthorize", 65537);
    m.insert("TrustlineFreeze", 65538);
    m.insert("TrustlineUnfreeze", 65539);
    m.insert("AccountDomainSet", 65540);
    m.insert("AccountEmailHashSet", 65541);
    m.insert("AccountMessageKeySet", 65542);
    m.insert("AccountTransferRateSet", 65543);
    m.insert("AccountTickSizeSet", 65544);
    m.insert("PaymentMint", 65545);
    m.insert("PaymentBurn", 65546);
    m.insert("MPTokenIssuanceLock", 65547);
    m.insert("MPTokenIssuanceUnlock", 65548);
    m
});

/// Resolve a PermissionValue string to its u32 code.
/// Checks granular permissions first, then falls back to transaction type + 1.
pub fn permission_value_code(name: &str) -> Result<u32, CoreError> {
    if let Some(&code) = GRANULAR_PERMISSIONS.get(name) {
        return Ok(code);
    }
    // Transaction type names use code + 1
    if let Ok(tt_code) = transaction_type_code(name) {
        return Ok(tt_code as u32 + 1);
    }
    Err(CoreError::CodecError(format!(
        "unknown PermissionValue: {name}"
    )))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn loads_transaction_types() {
        assert_eq!(transaction_type_code("Payment").unwrap(), 0);
        assert_eq!(transaction_type_code("OfferCreate").unwrap(), 7);
        assert_eq!(transaction_type_code("TrustSet").unwrap(), 20);
        assert_eq!(transaction_type_code("AMMCreate").unwrap(), 35);
    }

    #[test]
    fn loads_field_definitions() {
        let account = lookup_field_def("Account").unwrap();
        assert_eq!(account.type_code, 8);
        assert_eq!(account.field_code, 1);
        assert!(account.is_vl_encoded);
        assert!(account.is_serialized);

        let fee = lookup_field_def("Fee").unwrap();
        assert_eq!(fee.type_code, 6);
        assert_eq!(fee.field_code, 8);
    }

    #[test]
    fn lookup_by_id_works() {
        let f = lookup_field_def_by_id(1, 2).unwrap();
        assert_eq!(f.name, "TransactionType");
    }

    #[test]
    fn field_count_reasonable() {
        // Should have hundreds of fields
        assert!(FIELD_MAP.len() > 100);
    }
}
