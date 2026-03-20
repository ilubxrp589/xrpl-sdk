pub mod wallet;

// Re-exports from xrpl-core
pub use xrpl_core::address::KeyType;
pub use xrpl_core::codec::{decode_transaction_binary, encode_transaction_json};
pub use xrpl_core::crypto::{sign_transaction, Keypair, Seed};
pub use xrpl_core::transaction::builders::{self as transactions, Transaction, TxCommon};
pub use xrpl_core::types::*;
pub use xrpl_core::CoreError;

// Re-exports from xrpl-client
pub use xrpl_client::error::ClientError;
pub use xrpl_client::http::XrplHttpClient;
pub use xrpl_client::types::*;
pub use xrpl_client::XrplWsClient;

#[cfg(not(target_arch = "wasm32"))]
pub use xrpl_client::ws::{LedgerEvent, TransactionEvent, WsEvent};

pub use wallet::Wallet;

/// Autofill a transaction, sign it, and return the hex-encoded blob.
pub async fn autofill_and_sign(
    client: &XrplHttpClient,
    tx: &mut serde_json::Value,
    wallet: &Wallet,
) -> Result<String, ClientError> {
    client.autofill(tx).await?;
    let blob = wallet.sign_and_encode(tx).map_err(ClientError::Core)?;
    Ok(blob)
}

/// Autofill, sign, submit, and wait for validation. All-in-one transaction submission.
pub async fn submit_transaction(
    client: &XrplHttpClient,
    tx: &mut serde_json::Value,
    wallet: &Wallet,
) -> Result<TxResult, ClientError> {
    client.autofill(tx).await?;
    let last_ledger_sequence = tx["LastLedgerSequence"].as_u64().ok_or_else(|| {
        ClientError::UnexpectedResponse("autofill did not set LastLedgerSequence".into())
    })? as u32;
    let blob = wallet.sign_and_encode(tx).map_err(ClientError::Core)?;
    client.submit_and_wait(&blob, last_ledger_sequence).await
}
