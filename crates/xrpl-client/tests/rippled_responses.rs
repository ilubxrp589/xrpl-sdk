//! Every typed response against responses recorded from real servers, in
//! `rippled`'s API version 2: `rippled` 3.4.1 on mainnet
//! (tests/fixtures/rippled/*.json) and Clio (`s2.ripple.com`), recorded with
//! `scripts/record_fixtures.py`, plus the Testnet recordings the live test
//! makes. Each
//! must parse into its request's `Response` type, and serializing that back
//! must keep every field the server sent: a field the types do not model
//! fails the test instead of vanishing.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::Path;
use xrpl_client::requests::*;

/// Every key path in `v` (`a.b[].c`), over all elements of every array.
fn key_paths(v: &Value, prefix: &str, out: &mut BTreeSet<String>) {
    match v {
        Value::Object(m) => {
            for (k, x) in m {
                let p = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{prefix}.{k}")
                };
                out.insert(p.clone());
                key_paths(x, &p, out);
            }
        }
        Value::Array(a) => {
            for x in a {
                key_paths(x, &format!("{prefix}[]"), out);
            }
        }
        _ => {}
    }
}

/// Fields of every response that are not part of its type: the request's
/// `status` (an error never reaches a typed response) and server `warnings`.
const ENVELOPE: &[&str] = &["status", "warnings"];

fn check<R: Request>(file: &str) -> usize
where
    R::Response: Serialize,
{
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/rippled")
        .join(file);
    let recorded: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(
        recorded["method"],
        R::METHOD,
        "{file} was recorded from another method"
    );
    assert_eq!(
        recorded["params"]["api_version"],
        xrpl_client::API_VERSION,
        "{file} was recorded with another API version"
    );
    let mut result = recorded["result"].clone();
    for k in ENVELOPE {
        result.as_object_mut().unwrap().remove(*k);
    }
    let typed: R::Response = serde_json::from_value(result.clone()).unwrap_or_else(|e| {
        panic!(
            "{file}: does not parse as {}: {e}",
            std::any::type_name::<R::Response>()
        )
    });
    let back = serde_json::to_value(&typed).unwrap();
    let (mut sent, mut kept) = (BTreeSet::new(), BTreeSet::new());
    key_paths(&result, "", &mut sent);
    key_paths(&back, "", &mut kept);
    let dropped: Vec<_> = sent.difference(&kept).collect();
    assert!(
        dropped.is_empty(),
        "{file}: the typed response drops {dropped:?}"
    );
    sent.len()
}

#[test]
fn every_recorded_response_parses_and_keeps_every_field() {
    let mut checked: Vec<(&str, usize)> = Vec::new();
    macro_rules! check {
        ($($req:ty => $($file:literal),+;)*) => {$($(
            checked.push(($file, check::<$req>($file)));
        )+)*};
    }
    check! {
        AccountChannelsRequest => "account_channels.json";
        AccountCurrenciesRequest => "account_currencies.json";
        AccountInfoRequest => "account_info.json", "account_info_queue.json";
        AccountLinesRequest => "account_lines.json";
        AccountNftsRequest => "account_nfts.json";
        AccountObjectsRequest => "account_objects.json";
        AccountOffersRequest => "account_offers.json";
        AccountTxRequest => "account_tx.json", "account_tx_binary.json";
        GatewayBalancesRequest => "gateway_balances.json";
        NoRippleCheckRequest => "noripple_check.json";
        LedgerRequest => "ledger.json", "ledger_transactions.json";
        LedgerClosedRequest => "ledger_closed.json";
        LedgerCurrentRequest => "ledger_current.json";
        LedgerDataRequest => "ledger_data.json";
        LedgerEntryRequest => "ledger_entry.json";
        SimulateRequest => "simulate.json";
        SubmitRequest => "submit.json";
        SubmitMultisignedRequest => "submit_multisigned.json";
        TransactionEntryRequest => "transaction_entry.json";
        TxRequest => "tx.json";
        AmmInfoRequest => "amm_info.json";
        BookChangesRequest => "book_changes.json";
        BookOffersRequest => "book_offers.json";
        DepositAuthorizedRequest => "deposit_authorized.json";
        GetAggregatePriceRequest => "get_aggregate_price.json";
        NftSellOffersRequest => "nft_sell_offers.json";
        RipplePathFindRequest => "ripple_path_find.json";
        ChannelVerifyRequest => "channel_verify.json";
        FeeRequest => "fee.json";
        FeatureRequest => "feature.json";
        ManifestRequest => "manifest.json";
        ServerDefinitionsRequest => "server_definitions.json";
        ServerInfoRequest => "server_info.json";
        ServerStateRequest => "server_state.json";
        VersionRequest => "version.json";
        PingRequest => "ping.json";
        RandomRequest => "random.json";
        NftInfoRequest => "nft_info.json";
        NftHistoryRequest => "nft_history.json";
        NftsByIssuerRequest => "nfts_by_issuer.json";
        MptHoldersRequest => "mpt_holders.json";
    }
    // Every recording is checked; none sits in the directory unused.
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/rippled");
    let on_disk: BTreeSet<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .filter(|f| f.ends_with(".json"))
        .collect();
    let used: BTreeSet<String> = checked.iter().map(|(f, _)| f.to_string()).collect();
    assert_eq!(
        on_disk, used,
        "recordings not checked, or checked but missing"
    );
    let fields: usize = checked.iter().map(|(_, n)| n).sum();
    println!(
        "{} recorded responses, {fields} field paths, all kept",
        checked.len()
    );
}

/// The check itself: a response type that leaves fields out must fail it.
#[test]
#[should_panic(expected = "the typed response drops")]
fn a_type_that_drops_fields_fails_the_check() {
    #[derive(serde::Serialize)]
    struct FeeDropsOnly;
    #[derive(serde::Serialize, serde::Deserialize)]
    struct DropsOnly {
        drops: Value,
    }
    impl Request for FeeDropsOnly {
        const METHOD: &'static str = "fee";
        type Response = DropsOnly;
    }
    check::<FeeDropsOnly>("fee.json");
}
