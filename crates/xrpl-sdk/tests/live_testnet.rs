//! End to end on the XRPL Testnet (`--features live-tests`): two faucet
//! wallets submit one transaction of each kind that creates a ledger entry,
//! and every entry's ID must be the one `keylet` computes. Along the way the
//! SDK's own transaction hash, NFTokenID, balance changes and payment
//! channel claim signature are checked against what the network did, a
//! multi-signed transaction goes through `submit_multisigned`, and the typed
//! requests for what was created (channels, NFT offers, oracle prices, MPT
//! holders) must parse what the servers answer.
//!
//! With `XRPL_RECORD_DIR=<dir>` the responses of those requests are written
//! there in the format of `halcyon-xrpl-client`'s `tests/fixtures/rippled`.
#![cfg(feature = "live-tests")]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use serde_json::{json, Value};
use xrpl_sdk::requests::*;
use xrpl_sdk::utils::{self, nftoken::parse_nftoken_id, str_to_hex};
use xrpl_sdk::wallet::collect_signers;
use xrpl_sdk::{
    batch, generate_faucet_wallet, hash_signed_tx_blob, keylet, models, Amount, Client,
    ClientError, FaucetNetwork, LedgerIndex, TxModel, Wallet, XrplHttpClient,
};

/// Testnet's public Clio server (for the Clio-only methods).
const TESTNET_CLIO: &str = "https://clio.altnet.rippletest.net:51234";

/// Send a typed request, keep the raw response if `XRPL_RECORD_DIR` is set,
/// and parse it into the request's response type.
async fn query<R: Request>(client: &XrplHttpClient, req: &R, record_as: &str) -> R::Response {
    let params = serde_json::to_value(req).unwrap();
    let result = client
        .raw_request(R::METHOD, params.clone())
        .await
        .unwrap_or_else(|e| panic!("{}: {e}", R::METHOD));
    if let Ok(dir) = std::env::var("XRPL_RECORD_DIR") {
        let file = std::path::Path::new(&dir).join(format!("{record_as}.json"));
        // As sent: the client adds the API version.
        let mut params = params;
        params["api_version"] = json!(xrpl_sdk::xrpl_client::API_VERSION);
        let recorded =
            json!({"method": R::METHOD, "params": params, "result": result, "network": "testnet"});
        std::fs::write(file, serde_json::to_string_pretty(&recorded).unwrap()).unwrap();
    }
    serde_json::from_value(result).unwrap_or_else(|e| panic!("{} does not parse: {e}", R::METHOD))
}

/// The typed metadata of a transaction `submit` returned.
fn typed_meta(tx: &Value) -> xrpl_sdk::TransactionMetadata {
    serde_json::from_value(tx["meta"].clone()).unwrap()
}

/// Poll `tx` until the transaction is in a validated ledger.
async fn wait_validated(client: &XrplHttpClient, hash: &str) -> Value {
    for _ in 0..30 {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        if let Ok(r) = client.tx(hash).await {
            if r.validated == Some(true) {
                return serde_json::to_value(r.meta.unwrap()).unwrap();
            }
        }
    }
    panic!("{hash} was not validated");
}

/// Autofill, sign, submit and wait. `None` when the network has not enabled
/// the transaction's amendment (temDISABLED); panics on any other failure.
async fn submit(client: &XrplHttpClient, wallet: &Wallet, mut tx: Value) -> Option<Value> {
    client.autofill(&mut tx).await.unwrap();
    let blob = wallet.sign(&tx).unwrap().tx_blob;
    let our_hash = hash_signed_tx_blob(&blob).unwrap().to_string();
    let result = match client.submit_and_wait(&blob).await {
        Ok(r) => r,
        Err(ClientError::TransactionFailed { engine_result, .. })
            if engine_result == "temDISABLED" =>
        {
            println!(
                "  {} skipped: not enabled on testnet",
                tx["TransactionType"]
            );
            return None;
        }
        Err(e) => panic!("{}: {e}", tx["TransactionType"]),
    };
    let meta = result.meta.clone().unwrap();
    assert_eq!(
        meta.transaction_result, "tesSUCCESS",
        "{}",
        tx["TransactionType"]
    );
    assert_eq!(
        result.hash(),
        Some(our_hash.as_str()),
        "hash_signed_tx_blob"
    );
    let mut out = tx;
    out["meta"] = serde_json::to_value(meta).unwrap();
    Some(out)
}

/// Sign and submit a Batch (autofilling it first if it has no Sequence), wait
/// for it, and return each inner transaction's validated result. `None` if
/// Batch is not enabled on the network.
async fn submit_batch(
    client: &XrplHttpClient,
    wallet: &Wallet,
    batch_tx: &mut Value,
) -> Option<Vec<String>> {
    if batch_tx.get("Sequence").is_none() {
        client.autofill(batch_tx).await.unwrap();
    }
    let ids = batch::inner_transaction_ids(batch_tx).unwrap();
    let blob = wallet.sign(&*batch_tx).unwrap().tx_blob;
    match client.submit_and_wait(&blob).await {
        Ok(r) => assert_eq!(r.meta.unwrap().transaction_result, "tesSUCCESS", "Batch"),
        Err(ClientError::TransactionFailed { engine_result, .. })
            if engine_result == "temDISABLED" =>
        {
            println!("  Batch skipped: not enabled on testnet");
            return None;
        }
        Err(e) => panic!("Batch: {e}"),
    }
    let mut results = Vec::new();
    for id in ids {
        let meta = wait_validated(client, &id.to_string()).await;
        results.push(meta["TransactionResult"].as_str().unwrap().to_string());
    }
    Some(results)
}

/// The ID of the one entry of `entry_type` the transaction created.
fn created(tx: &Value, entry_type: &str) -> String {
    let ids: Vec<&str> = tx["meta"]["AffectedNodes"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|n| n.get("CreatedNode"))
        .filter(|n| n["LedgerEntryType"] == entry_type)
        .map(|n| n["LedgerIndex"].as_str().unwrap())
        .collect();
    assert_eq!(ids.len(), 1, "{entry_type} entries created: {ids:?}");
    ids[0].to_string()
}

fn check(checked: &mut Vec<String>, name: &str, got: String, want: String) {
    assert_eq!(got, want, "{name}");
    checked.push(name.to_string());
}

fn seq(tx: &Value) -> u32 {
    u32::try_from(tx["Sequence"].as_u64().unwrap()).unwrap()
}

#[tokio::test]
async fn keylets_and_helpers_match_testnet() {
    let net = FaucetNetwork::Testnet;
    let (alice, bob) = (
        generate_faucet_wallet(net).await.unwrap(),
        generate_faucet_wallet(net).await.unwrap(),
    );
    let (a, b) = (alice.classic_address(), bob.classic_address());
    let client = XrplHttpClient::new(net.rpc_url()).unwrap();
    let now = utils::time::system_time_to_ripple_time(std::time::SystemTime::now()).unwrap();
    let mut checked = Vec::new();

    // A Payment through the typed `submit` request: what get_balance_changes reports.
    let mut pay =
        json!({"TransactionType": "Payment", "Account": a, "Destination": b, "Amount": "1000000"});
    client.autofill(&mut pay).await.unwrap();
    let blob = alice.sign(&pay).unwrap().tx_blob;
    let submitted = query(&client, &SubmitRequest::new(blob.clone()), "submit").await;
    assert_eq!(submitted.engine_result, "tesSUCCESS");
    pay["meta"] = wait_validated(&client, &hash_signed_tx_blob(&blob).unwrap().to_string()).await;
    let changes = utils::get_balance_changes(&typed_meta(&pay)).unwrap();
    let fee = utils::drops_to_xrp(pay["Fee"].as_str().unwrap().parse().unwrap());
    let of = |acct: &str| {
        changes.iter().find(|c| c.account == acct).unwrap().balances[0]
            .value
            .clone()
    };
    assert_eq!(of(b), "1");
    assert_eq!(
        of(a),
        format!(
            "-{}",
            utils::xrp::drops_to_xrp(1_000_000 + utils::xrp_to_drops(&fee).unwrap())
        )
    );
    checked.push("get_balance_changes".into());

    // ledger_entry by the computed ID returns Alice's AccountRoot.
    let entry = client
        .ledger_entry(
            &keylet::account(a).unwrap().to_string(),
            LedgerIndex::Validated,
        )
        .await
        .unwrap();
    assert_eq!(entry.node.unwrap().to_value().unwrap()["Account"], a);
    checked.push("account".into());

    if let Some(tx) = submit(
        &client,
        &alice,
        json!({"TransactionType": "TicketCreate", "Account": a, "TicketCount": 1}),
    )
    .await
    {
        check(
            &mut checked,
            "ticket",
            keylet::ticket(a, seq(&tx) + 1).unwrap().to_string(),
            created(&tx, "Ticket"),
        );
        check(
            &mut checked,
            "owner_dir",
            keylet::owner_dir(a).unwrap().to_string(),
            created(&tx, "DirectoryNode"),
        );
    }
    if let Some(tx) = submit(&client, &alice, json!({"TransactionType": "CheckCreate", "Account": a, "Destination": b, "SendMax": "1000000"})).await {
        check(&mut checked, "check", keylet::check(a, seq(&tx)).unwrap().to_string(), created(&tx, "Check"));
    }
    if let Some(tx) = submit(&client, &alice, json!({"TransactionType": "EscrowCreate", "Account": a, "Destination": b, "Amount": "1000000", "FinishAfter": now + 3600})).await {
        check(&mut checked, "escrow", keylet::escrow(a, seq(&tx)).unwrap().to_string(), created(&tx, "Escrow"));
    }
    if let Some(tx) = submit(&client, &alice, json!({"TransactionType": "PaymentChannelCreate", "Account": a, "Destination": b, "Amount": "2000000", "SettleDelay": 60, "PublicKey": alice.public_key_hex()})).await {
        let channel = created(&tx, "PayChannel");
        check(&mut checked, "payment_channel", keylet::payment_channel(a, b, seq(&tx)).unwrap().to_string(), channel.clone());
        // Bob redeems 0.5 XRP with Alice's claim: the network accepts our signature.
        let signature = alice.sign_payment_channel_claim(&channel, 500_000).unwrap();
        submit(&client, &bob, json!({"TransactionType": "PaymentChannelClaim", "Account": b, "Channel": channel, "Balance": "500000", "Amount": "500000", "Signature": signature, "PublicKey": alice.public_key_hex()})).await.unwrap();
        checked.push("payment channel claim accepted".into());
        let channels = query(&client, &AccountChannelsRequest::new(a), "account_channels").await;
        assert_eq!(channels.channels[0].channel_id, channel);
        assert_eq!(channels.channels[0].balance, "500000");
        checked.push("account_channels".into());
    }
    if let Some(tx) = submit(&client, &alice, json!({"TransactionType": "NFTokenMint", "Account": a, "NFTokenTaxon": 7, "Flags": 8, "TransferFee": 500})).await {
        let mut meta = typed_meta(&tx);
        let id = utils::get_nftoken_id(&meta).unwrap();
        meta.nftoken_id = None;
        check(&mut checked, "get_nftoken_id (page diff)", utils::get_nftoken_id(&meta).unwrap(), id.clone());
        let p = parse_nftoken_id(&id).unwrap();
        assert_eq!((p.issuer.as_str(), p.taxon, p.flags, p.transfer_fee), (a, 7, 8, 500));
        if let Some(offer) = submit(&client, &alice, json!({"TransactionType": "NFTokenCreateOffer", "Account": a, "NFTokenID": id, "Amount": "1", "Flags": 1})).await {
            let offer_id = created(&offer, "NFTokenOffer");
            check(&mut checked, "nftoken_offer", keylet::nftoken_offer(a, seq(&offer)).unwrap().to_string(), offer_id.clone());
            let offers = query(&client, &NftSellOffersRequest::new(id.clone()), "nft_sell_offers").await;
            assert_eq!(offers.offers[0].nft_offer_index, offer_id);
            checked.push("nft_sell_offers".into());
        }
    }
    if let Some(tx) = submit(&client, &alice, json!({"TransactionType": "DIDSet", "Account": a, "URI": str_to_hex("https://example.com/did")})).await {
        check(&mut checked, "did", keylet::did(a).unwrap().to_string(), created(&tx, "DID"));
    }
    let unix_now = utils::ripple_time_to_unix(now);
    if let Some(tx) = submit(&client, &alice, json!({"TransactionType": "OracleSet", "Account": a, "OracleDocumentID": 1, "Provider": str_to_hex("halcyon"), "AssetClass": str_to_hex("currency"), "LastUpdateTime": unix_now, "PriceDataSeries": [{"PriceData": {"BaseAsset": "XRP", "QuoteAsset": "USD", "AssetPrice": 74, "Scale": 2}}]})).await {
        check(&mut checked, "oracle", keylet::oracle(a, 1).unwrap().to_string(), created(&tx, "Oracle"));
        let oracles = vec![OracleId { account: a.into(), oracle_document_id: 1 }];
        let price = query(&client, &GetAggregatePriceRequest::new("XRP", "USD", oracles), "get_aggregate_price").await;
        assert_eq!(price.median, "0.74");
        checked.push("get_aggregate_price".into());
    }
    if let Some(tx) = submit(&client, &alice, json!({"TransactionType": "MPTokenIssuanceCreate", "Account": a, "AssetScale": 2, "MaximumAmount": "100000", "Flags": 0x20})).await {
        let id = keylet::mpt_issuance_id(seq(&tx), a).unwrap().to_string();
        if let Some(rippled_id) = tx["meta"].get("mpt_issuance_id").and_then(Value::as_str) {
            check(&mut checked, "mpt_issuance_id", id.clone(), rippled_id.to_string());
        }
        check(&mut checked, "mpt_issuance", keylet::mpt_issuance(&id).unwrap().to_string(), created(&tx, "MPTokenIssuance"));
        if let Some(auth) = submit(&client, &bob, json!({"TransactionType": "MPTokenAuthorize", "Account": b, "MPTokenIssuanceID": id})).await {
            check(&mut checked, "mptoken", keylet::mptoken(&id, b).unwrap().to_string(), created(&auth, "MPToken"));
            // Clio answers mpt_holders once it has ingested the ledger.
            let clio = XrplHttpClient::new(TESTNET_CLIO).unwrap();
            for attempt in 0..10 {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                let holders: Result<Value, _> = clio.raw_request("mpt_holders", json!({"mpt_issuance_id": id})).await;
                if holders.as_ref().is_ok_and(|h| h["mptokens"].as_array().is_some_and(|m| !m.is_empty())) || attempt == 9 {
                    let holders = query(&clio, &MptHoldersRequest::new(id.clone()), "mpt_holders").await;
                    assert_eq!(holders.mptokens[0].account, b);
                    checked.push("mpt_holders (Clio)".into());
                    break;
                }
            }
        }
    }
    let credential_type = str_to_hex("KYC");
    if let Some(tx) = submit(&client, &alice, json!({"TransactionType": "CredentialCreate", "Account": a, "Subject": b, "CredentialType": credential_type})).await {
        check(&mut checked, "credential", keylet::credential(b, a, &credential_type).unwrap().to_string(), created(&tx, "Credential"));
    }
    if let Some(tx) = submit(&client, &alice, json!({"TransactionType": "PermissionedDomainSet", "Account": a, "AcceptedCredentials": [{"Credential": {"Issuer": a, "CredentialType": credential_type}}]})).await {
        check(&mut checked, "permissioned_domain", keylet::permissioned_domain(a, seq(&tx)).unwrap().to_string(), created(&tx, "PermissionedDomain"));
    }
    if let Some(tx) = submit(
        &client,
        &alice,
        json!({"TransactionType": "DepositPreauth", "Account": a, "Authorize": b}),
    )
    .await
    {
        check(
            &mut checked,
            "deposit_preauth",
            keylet::deposit_preauth(a, b).unwrap().to_string(),
            created(&tx, "DepositPreauth"),
        );
    }
    if let Some(tx) = submit(&client, &alice, json!({"TransactionType": "DelegateSet", "Account": a, "Authorize": b, "Permissions": [{"Permission": {"PermissionValue": "Payment"}}]})).await {
        check(&mut checked, "delegate", keylet::delegate(a, b).unwrap().to_string(), created(&tx, "Delegate"));
    }
    if let Some(tx) = submit(&client, &alice, json!({"TransactionType": "TrustSet", "Account": a, "LimitAmount": {"currency": "USD", "issuer": b, "value": "100"}})).await {
        check(&mut checked, "trust_line", keylet::trust_line(a, b, "USD").unwrap().to_string(), created(&tx, "RippleState"));
    }
    // Batch, one account: Alice pays Bob twice, all or nothing. autofill numbers
    // the inner transactions and prices the batch; each inner transaction lands
    // on the ledger under the ID batch::inner_transaction_ids computed.
    let mut one = json!({"TransactionType": "Batch", "Account": a, "Flags": u32::from(models::BatchFlags::ALL_OR_NOTHING), "RawTransactions": [
        {"RawTransaction": {"TransactionType": "Payment", "Account": a, "Destination": b, "Amount": "1000000"}},
        {"RawTransaction": {"TransactionType": "Payment", "Account": a, "Destination": b, "Amount": "2000000"}},
    ]});
    if let Some(done) = submit_batch(&client, &alice, &mut one).await {
        for id in done {
            check(
                &mut checked,
                "batch inner (one account)",
                "tesSUCCESS".into(),
                id,
            );
        }
    }
    // Batch, two accounts, built from the typed models: Alice pays Bob, Bob pays
    // Alice, and Bob approves it as a BatchSigner.
    let inner_a: models::Tx = models::Payment::new(a, b, Amount::Xrp(1_000_000)).into();
    let inner_b: models::Tx = models::Payment::new(b, a, Amount::Xrp(500_000)).into();
    let mut typed = models::Batch::new(a, vec![inner_a, inner_b]);
    typed.common.flags = Some(u32::from(models::BatchFlags::ALL_OR_NOTHING));
    let mut two = typed.to_json().unwrap();
    client.autofill(&mut two).await.unwrap();
    let bob_approves = bob.sign_batch(&two).unwrap();
    batch::add_batch_signers(&mut two, vec![bob_approves]).unwrap();
    assert!(batch::verify_batch_signers(&two).unwrap());
    if let Some(done) = submit_batch(&client, &alice, &mut two).await {
        for id in done {
            check(
                &mut checked,
                "batch inner (two accounts, BatchSigner)",
                "tesSUCCESS".into(),
                id,
            );
        }
    }

    // Multi-signing end to end: Bob alone signs for Alice through her signer list.
    if submit(&client, &alice, json!({"TransactionType": "SignerListSet", "Account": a, "SignerQuorum": 1, "SignerEntries": [{"SignerEntry": {"Account": b, "SignerWeight": 1}}]})).await.is_some() {
        let mut tx = json!({"TransactionType": "AccountSet", "Account": a, "Domain": str_to_hex("example.com")});
        client.autofill(&mut tx).await.unwrap();
        let fee: u64 = tx["Fee"].as_str().unwrap().parse().unwrap();
        tx["Fee"] = json!((fee * 2).to_string()); // (1 + signers) × the single-signature fee
        tx["SigningPubKey"] = json!("");
        let signature = bob.sign_for_multisigning(&tx).unwrap();
        collect_signers(&mut tx, vec![signature]).unwrap();
        let submitted = query(&client, &SubmitMultisignedRequest::new(tx), "submit_multisigned").await;
        assert_eq!(submitted.engine_result, "tesSUCCESS", "{}", submitted.engine_result_message);
        let hash = submitted.hash().unwrap().to_string();
        let meta = wait_validated(&client, &hash).await;
        assert_eq!(meta["TransactionResult"], "tesSUCCESS");
        checked.push("submit_multisigned".into());
    }
    println!("checked on testnet: {}", checked.join(", "));
    assert!(checked.len() >= 12, "only {checked:?}");
}
