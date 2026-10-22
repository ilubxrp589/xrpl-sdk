//! NFTokenIDs: what the 32 bytes hold, and which token a transaction minted.

use crate::address::encode_account_id;
use crate::CoreError;
use serde_json::Value;
use std::collections::HashSet;

/// `tfBurnable`: the issuer may burn the token.
pub const NFT_FLAG_BURNABLE: u16 = 0x0001;
/// `tfOnlyXRP`: offers for the token must be in XRP.
pub const NFT_FLAG_ONLY_XRP: u16 = 0x0002;
/// `tfTrustLine` (no longer honoured by rippled).
pub const NFT_FLAG_TRUSTLINE: u16 = 0x0004;
/// `tfTransferable`: holders other than the issuer may transfer it.
pub const NFT_FLAG_TRANSFERABLE: u16 = 0x0008;
/// `tfMutable`: the issuer may change its URI (DynamicNFT).
pub const NFT_FLAG_MUTABLE: u16 = 0x0010;

/// The fields packed into an NFTokenID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NFTokenIdParts {
    /// The ID as given, in upper case.
    pub nftoken_id: String,
    /// `NFT_FLAG_*` bits.
    pub flags: u16,
    /// Fee on secondary sales, in 1/100,000 (50000 = 50%).
    pub transfer_fee: u16,
    /// The issuer's classic address.
    pub issuer: String,
    /// The taxon the issuer chose (unscrambled).
    pub taxon: u32,
    /// The issuer's mint sequence for this token.
    pub sequence: u32,
}

impl NFTokenIdParts {
    pub fn is_burnable(&self) -> bool {
        self.flags & NFT_FLAG_BURNABLE != 0
    }
    pub fn is_only_xrp(&self) -> bool {
        self.flags & NFT_FLAG_ONLY_XRP != 0
    }
    pub fn is_transferable(&self) -> bool {
        self.flags & NFT_FLAG_TRANSFERABLE != 0
    }
    pub fn is_mutable(&self) -> bool {
        self.flags & NFT_FLAG_MUTABLE != 0
    }
}

/// Split an NFTokenID (64 hex digits) into its fields, as rippled packs them
/// (`nft::createNFTokenID`): flags, transfer fee, issuer, scrambled taxon, sequence.
///
/// ```
/// use xrpl_core::utils::nftoken::parse_nftoken_id;
/// let p = parse_nftoken_id("000B0539C35B55AA096BA6D87A6E6C965A6534150DC56E5E12C5D09E0000000C").unwrap();
/// assert_eq!((p.flags, p.transfer_fee, p.taxon, p.sequence), (11, 1337, 1337, 12));
/// assert_eq!(p.issuer, "rJoxBSzpXhPtAuqFmqxQtGKjA13jUJWthE");
/// ```
pub fn parse_nftoken_id(nftoken_id: &str) -> Result<NFTokenIdParts, CoreError> {
    let bytes = hex::decode(nftoken_id).map_err(|e| CoreError::InvalidHex(e.to_string()))?;
    let bytes: [u8; 32] = bytes.try_into().map_err(|b: Vec<u8>| {
        CoreError::ValidationError(format!("an NFTokenID is 32 bytes, got {}", b.len()))
    })?;
    let [f0, f1, t0, t1, rest @ ..] = bytes;
    let (issuer, rest) = rest.split_first_chunk::<20>().ok_or_else(short)?;
    let (taxon, rest) = rest.split_first_chunk::<4>().ok_or_else(short)?;
    let sequence: [u8; 4] = rest.try_into().map_err(|_| short())?;
    let sequence = u32::from_be_bytes(sequence);
    Ok(NFTokenIdParts {
        nftoken_id: nftoken_id.to_ascii_uppercase(),
        flags: u16::from_be_bytes([f0, f1]),
        transfer_fee: u16::from_be_bytes([t0, t1]),
        issuer: encode_account_id(issuer),
        taxon: u32::from_be_bytes(*taxon) ^ taxon_cipher(sequence),
        sequence,
    })
}

fn short() -> CoreError {
    CoreError::ValidationError("an NFTokenID is 32 bytes".into())
}

/// rippled's `nft::cipheredTaxon` mask: (384160001 × sequence + 2459) mod 2³².
fn taxon_cipher(sequence: u32) -> u32 {
    384_160_001u32.wrapping_mul(sequence).wrapping_add(2459)
}

/// The NFTokenID a transaction minted (or, for an accepted offer, moved into a
/// page), from its metadata.
///
/// rippled adds the answer to the metadata as `nftoken_id`; when it is absent
/// (older servers, or metadata decoded from binary) it is worked out the way
/// xrpl.js's `getNFTokenID` does: the token in an `NFTokenPage` after the
/// transaction that was not in it before. `Ok(None)` if no token was added.
pub fn get_nftoken_id(meta: &Value) -> Result<Option<String>, CoreError> {
    let nodes = meta
        .get("AffectedNodes")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            CoreError::ValidationError(
                "expected transaction metadata (an object with AffectedNodes)".into(),
            )
        })?;
    if let Some(id) = meta.get("nftoken_id").and_then(Value::as_str) {
        return Ok(Some(id.to_string()));
    }
    let token_ids = |tokens: Option<&Value>| -> Vec<String> {
        tokens
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|t| t.pointer("/NFToken/NFTokenID").and_then(Value::as_str))
            .map(str::to_string)
            .collect()
    };
    let mut before = HashSet::new();
    let mut after = Vec::new();
    for node in nodes {
        if let Some(created) = node.get("CreatedNode") {
            if created.get("LedgerEntryType").and_then(Value::as_str) == Some("NFTokenPage") {
                after.extend(token_ids(created.pointer("/NewFields/NFTokens")));
            }
        } else if let Some(modified) = node.get("ModifiedNode") {
            let previous = modified.pointer("/PreviousFields/NFTokens");
            if modified.get("LedgerEntryType").and_then(Value::as_str) == Some("NFTokenPage")
                && previous.is_some()
            {
                before.extend(token_ids(previous));
                after.extend(token_ids(modified.pointer("/FinalFields/NFTokens")));
            }
        }
    }
    Ok(after.into_iter().find(|id| !before.contains(id)))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    // xrpl.js parseNFTokenID.test.ts.
    #[test]
    fn parse_matches_xrpl_js() {
        for (id, flags, fee, issuer, taxon, seq) in [
            (
                "000B0539C35B55AA096BA6D87A6E6C965A6534150DC56E5E12C5D09E0000000C",
                11,
                1337,
                "rJoxBSzpXhPtAuqFmqxQtGKjA13jUJWthE",
                1337,
                12,
            ),
            (
                "000000005EC8BC31F0415E5DD4A8AAAC3718249F8F27323C2EEE87B80000001E",
                0,
                0,
                "r9ewzMXVRAD9CjZQ6LTQ4P21vUUucDuqd4",
                2_147_483_649,
                30,
            ),
        ] {
            let p = parse_nftoken_id(id).unwrap();
            assert_eq!(
                (
                    p.flags,
                    p.transfer_fee,
                    p.issuer.as_str(),
                    p.taxon,
                    p.sequence
                ),
                (flags, fee, issuer, taxon, seq),
                "{id}"
            );
        }
        let p =
            parse_nftoken_id("00081388BE9E48FA0E6C95A3E970EB9503E3D3967E8DF95041FED82604D933AB")
                .unwrap();
        assert_eq!((p.flags, p.transfer_fee), (8, 5000));
        assert!(p.is_transferable() && !p.is_burnable());
        assert!(parse_nftoken_id("00").is_err());
        assert!(parse_nftoken_id("zz").is_err());
    }

    fn fixture(name: &str) -> Value {
        let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    // xrpl.js getNFTokenID.test.ts (fixtures/rippled/tx/NFTokenMint*.json).
    #[test]
    fn get_matches_xrpl_js() {
        let mint = fixture("xrpljs/NFTokenMint.json");
        assert_eq!(
            get_nftoken_id(&mint["meta"]).unwrap().as_deref(),
            Some("00081388DC1AB4937C899037B2FDFC3CB20F6F64E73120BB5F8AA66A00000228")
        );
        let mint2 = fixture("xrpljs/NFTokenMint2.json");
        assert_eq!(
            get_nftoken_id(&mint2["meta"]).unwrap().as_deref(),
            Some("0008125CBE4B401B2F62ED35CC67362165AA813CCA06316FFA766254000003EE")
        );
        assert!(get_nftoken_id(&mint).is_err(), "a whole tx is not metadata");
    }

    // A mainnet NFTokenMint (ledger 107830973): rippled's own nftoken_id must equal
    // the page diff, and parse back to the minting account and its fields.
    #[test]
    fn mainnet_mint_agrees_with_rippled() {
        let tx = fixture("mainnet_nftoken_mint.json");
        let mut meta = tx["meta"].clone();
        let rippled_id = meta["nftoken_id"].as_str().unwrap().to_string();
        assert_eq!(
            get_nftoken_id(&meta).unwrap().as_deref(),
            Some(rippled_id.as_str())
        );
        meta.as_object_mut().unwrap().remove("nftoken_id");
        assert_eq!(get_nftoken_id(&meta).unwrap(), Some(rippled_id.clone()));
        let p = parse_nftoken_id(&rippled_id).unwrap();
        let issuer = tx.get("Issuer").unwrap_or(&tx["Account"]).as_str().unwrap();
        assert_eq!(p.issuer, issuer);
        assert_eq!(p.taxon, tx["NFTokenTaxon"].as_u64().unwrap() as u32);
        assert_eq!(
            u32::from(p.flags),
            tx["Flags"].as_u64().unwrap_or(0) as u32 & 0xFFFF
        );
        assert_eq!(
            u64::from(p.transfer_fee),
            tx["TransferFee"].as_u64().unwrap_or(0)
        );
    }
}
