//! Sign and submit with a key that never enters this program: a signing
//! service holds it (here a task standing in for a cloud KMS or an HSM
//! behind a network call), and the program only ever sees its public key
//! and the signatures it returns.
//!
//! The program implements `AsyncSigningKey` for its handle on the service
//! and passes the handle wherever a `Wallet` goes. The SDK checks each
//! signature against the public key before anything is submitted, and turns
//! a high-S secp256k1 signature (AWS KMS returns one half the time) into the
//! low-S form `rippled` requires.
//!
//! Usage: cargo run --example remote_signer

use tokio::sync::{mpsc, oneshot};
use xrpl_sdk::xrpl_core::crypto::{secp256k1, sha512_half};
use xrpl_sdk::*;

/// A request to the signing service: the 32-byte digest to sign, and where
/// to send the DER signature.
type SignRequest = (Vec<u8>, oneshot::Sender<Result<Vec<u8>, String>>);

/// The program's handle on a key held by the signing service.
struct RemoteKey {
    /// Fetched once, when the key is set up (a KMS `GetPublicKey`).
    public_key: Vec<u8>,
    service: mpsc::Sender<SignRequest>,
}

impl AsyncSigningKey for RemoteKey {
    fn public_key(&self) -> &[u8] {
        &self.public_key
    }

    async fn sign_signing_data(&self, data: &[u8]) -> Result<Vec<u8>, CoreError> {
        // A secp256k1 key signs SHA-512Half of the data: send the digest, as
        // a KMS `Sign` call with message type DIGEST does.
        let (reply, answer) = oneshot::channel();
        self.service
            .send((sha512_half(data).to_vec(), reply))
            .await
            .map_err(|_| CoreError::SigningKeyFailed("the signing service is gone".into()))?;
        answer
            .await
            .map_err(|_| CoreError::SigningKeyFailed("the signing service did not answer".into()))?
            .map_err(|e| CoreError::SigningKeyFailed(e.into()))
    }
}

/// The stand-in signing service: it makes a secp256k1 key, gives out the
/// public key, and signs digests on request. The secret stays in this task.
fn start_signing_service() -> Result<(Vec<u8>, mpsc::Sender<SignRequest>), CoreError> {
    let keys = Keypair::from_seed(&Seed::generate_with_type(KeyType::Secp256k1))?;
    let public_key = keys.public_key().to_vec();
    let (requests, mut incoming) = mpsc::channel::<SignRequest>(8);
    tokio::spawn(async move {
        while let Some((digest, reply)) = incoming.recv().await {
            let signature = secp256k1::sign(keys.private_key(), &digest).map_err(|e| e.to_string());
            let _ = reply.send(signature);
        }
    });
    Ok((public_key, requests))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (public_key, service) = start_signing_service()?;
    let key = RemoteKey {
        public_key,
        service,
    };
    // The address is the public key's: this program never sees the secret.
    let address = AccountId::from_public_key(AsyncSigningKey::public_key(&key)).to_address();
    println!("Remote key's address: {address}");

    let network = FaucetNetwork::Testnet;
    println!("Funding it from the {network:?} faucet…");
    faucet::fund_and_wait(network, &address).await?;

    let client = XrplHttpClient::new(network.rpc_url())?;
    let payment = models::Payment::new(
        address.as_str(),
        "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
        Amount::xrp(utils::xrp_to_drops("1")?)?,
    );
    let result = sign_and_submit(&client, &payment, &key).await?;
    println!(
        "Paid 1 XRP, signed by the service: {} in ledger {}",
        result.transaction_result().unwrap_or("?"),
        result.ledger_index.map_or("?".into(), |l| l.to_string())
    );

    // Offline, the same key signs without a network: `sign_async` returns
    // the blob and the ID.
    let offline = models::Payment::new(
        address.as_str(),
        "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
        Amount::xrp(1)?,
    )
    .with_fee(12)
    .with_sequence(1);
    let signed = sign_async(&offline, &key).await?;
    assert!(verify_transaction(&signed.tx_json)?);
    println!("Signed offline too: {}", signed.hash);
    Ok(())
}
