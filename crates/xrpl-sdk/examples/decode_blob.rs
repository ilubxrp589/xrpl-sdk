//! Decode a hex-encoded transaction blob to JSON.
//!
//! Takes a hex blob string as an argument, decodes it using
//! the binary codec, and prints the result as pretty JSON.
//!
//! Usage: cargo run --example decode_blob -- <HEX_BLOB>

use xrpl_sdk::decode_transaction_binary;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let blob_hex = std::env::args()
        .nth(1)
        .ok_or("Usage: decode_blob <HEX_BLOB>")?;

    let bytes = hex::decode(&blob_hex)?;
    let decoded = decode_transaction_binary(&bytes)?;

    println!("{}", serde_json::to_string_pretty(&decoded)?);
    Ok(())
}
