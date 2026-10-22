//! `hxrpl`: the XRP Ledger from the command line, on `halcyon-xrpl-sdk`.
//!
//! It works on Testnet unless told otherwise (`--network mainnet`, or
//! `XRPL_NETWORK=mainnet`), and sending on mainnet also needs `--yes`.
//! Seeds are read from `XRPL_SEED` or `--seed-file`, never from the command
//! line, where shell history and process lists would keep them.

use clap::{Args, Parser, Subcommand, ValueEnum};
use serde_json::{json, Value};
use std::error::Error;
use std::path::PathBuf;
use xrpl_sdk::*;

type Res<T = ()> = Result<T, Box<dyn Error>>;

#[derive(Parser)]
#[command(
    name = "hxrpl",
    version,
    about = "The XRP Ledger from the command line: wallets, faucet, queries, signing, sending, hashes, ledger entry IDs and live streams.",
    after_help = "Examples:\n  hxrpl wallet faucet                       a funded Testnet wallet\n  hxrpl --network mainnet account info rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh\n  XRPL_SEED=s… hxrpl tx send-xrp rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe 1.5\n  hxrpl --network mainnet watch ledger --count 5\n  hxrpl id trust-line rA… rB… USD           a trust line's ledger entry ID\n  hxrpl tx hash 1200002200000000…          a signed blob's transaction ID\n\nSeeds come from XRPL_SEED or --seed-file, never from arguments."
)]
struct Cli {
    /// The network to use.
    #[arg(long, global = true, env = "XRPL_NETWORK", default_value = "testnet")]
    network: Network,
    /// A server to use instead of the network's public one (https://… or wss://…).
    #[arg(long, global = true, env = "XRPL_URL")]
    url: Option<String>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Network {
    Mainnet,
    Testnet,
    Devnet,
}

impl Network {
    fn rpc(self) -> &'static str {
        match self {
            Network::Mainnet => "https://xrplcluster.com",
            Network::Testnet => FaucetNetwork::Testnet.rpc_url(),
            Network::Devnet => FaucetNetwork::Devnet.rpc_url(),
        }
    }
    fn ws(self) -> &'static str {
        match self {
            Network::Mainnet => "wss://xrplcluster.com",
            Network::Testnet => FaucetNetwork::Testnet.ws_url(),
            Network::Devnet => FaucetNetwork::Devnet.ws_url(),
        }
    }
    fn faucet(self) -> Option<FaucetNetwork> {
        match self {
            Network::Mainnet => None,
            Network::Testnet => Some(FaucetNetwork::Testnet),
            Network::Devnet => Some(FaucetNetwork::Devnet),
        }
    }
}

#[derive(Subcommand)]
enum Command {
    /// Make, show and fund wallets.
    #[command(subcommand)]
    Wallet(WalletCmd),
    /// Query an account.
    #[command(subcommand)]
    Account(AccountCmd),
    /// Look up, hash, decode, encode, sign, simulate and submit transactions.
    #[command(subcommand)]
    Tx(TxCmd),
    /// A ledger's header ("validated", "current", "closed" or an index).
    Ledger {
        #[arg(default_value = "validated")]
        ledger: String,
    },
    /// A ledger entry by its ID (`hxrpl id …` computes IDs).
    Entry { id: String },
    /// Compute the ID of a ledger entry.
    #[command(subcommand)]
    Id(IdCmd),
    /// The transaction cost now.
    Fee,
    /// The server's state.
    Server,
    /// An order book: what TAKER_GETS costs in TAKER_PAYS, e.g. `XRP` and `USD.rIssuer…`.
    Book {
        taker_gets: String,
        taker_pays: String,
        #[arg(long, default_value_t = 10)]
        limit: u32,
    },
    /// Watch a stream live.
    Watch {
        stream: Stream,
        /// Stop after this many events.
        #[arg(long, default_value_t = 10)]
        count: usize,
    },
    /// Conversions: XRP and drops, ledger time, NFTokenIDs, text and hex.
    #[command(subcommand)]
    Convert(ConvertCmd),
    /// Call any rippled method with JSON parameters, e.g. `hxrpl raw server_state '{}'`.
    Raw {
        method: String,
        #[arg(default_value = "{}")]
        params: String,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum Stream {
    Ledger,
    Transactions,
    Validations,
    BookChanges,
    Server,
    Consensus,
}

#[derive(Args)]
struct SeedSource {
    /// Read the seed from this file instead of `XRPL_SEED`.
    #[arg(long)]
    seed_file: Option<PathBuf>,
}

#[derive(Subcommand)]
enum WalletCmd {
    /// Generate a new wallet (Ed25519, or secp256k1). Prints its seed: keep it safe.
    New {
        #[arg(long)]
        secp256k1: bool,
    },
    /// Show the address and public key of the wallet in XRPL_SEED / --seed-file.
    Show(SeedSource),
    /// Generate a wallet and fund it from the Testnet or Devnet faucet.
    Faucet,
}

#[derive(Subcommand)]
enum AccountCmd {
    /// Settings and XRP balance.
    Info { address: String },
    /// Trust lines (every page).
    Lines { address: String },
    /// Open DEX offers.
    Offers { address: String },
    /// Owned ledger entries, typed (optionally of one type: offer, state, check, escrow, nft_offer, …).
    Objects {
        address: String,
        #[arg(long = "type")]
        kind: Option<String>,
    },
    /// NFTs held.
    Nfts { address: String },
    /// Recent transactions, newest first.
    Tx {
        address: String,
        #[arg(long, default_value_t = 10)]
        limit: u32,
    },
    /// Currencies it can send and receive.
    Currencies { address: String },
    /// Payment channels it is the source of.
    Channels { address: String },
}

#[derive(Subcommand)]
enum TxCmd {
    /// A transaction by hash.
    Get { hash: String },
    /// The ID of a signed transaction (hex blob, or JSON; `-` reads stdin). Offline.
    Hash { tx: String },
    /// Hex blob → JSON. Offline.
    Decode { blob: String },
    /// JSON (or `-` for stdin) → hex blob. Offline.
    Encode { json: String },
    /// Sign a JSON transaction with the XRPL_SEED wallet; `--autofill` fills
    /// Sequence, Fee and LastLedgerSequence from the network first.
    Sign {
        json: String,
        #[arg(long)]
        autofill: bool,
        #[command(flatten)]
        seed: SeedSource,
    },
    /// What a JSON transaction would do, without signing or submitting it.
    Simulate { json: String },
    /// Submit a signed blob (`--wait` until validated). On mainnet, add --yes.
    Submit {
        blob: String,
        #[arg(long)]
        wait: bool,
        #[arg(long)]
        yes: bool,
    },
    /// Send XRP from the XRPL_SEED wallet: autofill, sign, submit, wait. On mainnet, add --yes.
    SendXrp {
        destination: String,
        /// In XRP, e.g. 1.5.
        amount: String,
        #[arg(long)]
        tag: Option<u32>,
        #[arg(long)]
        yes: bool,
        #[command(flatten)]
        seed: SeedSource,
    },
}

#[derive(Subcommand)]
enum IdCmd {
    /// AccountRoot.
    Account { address: String },
    /// RippleState: the trust line between two accounts.
    TrustLine {
        a: String,
        b: String,
        currency: String,
    },
    /// Offer, by the OfferCreate's sequence.
    Offer { account: String, sequence: u32 },
    /// Check, by the CheckCreate's sequence.
    Check { account: String, sequence: u32 },
    /// Escrow, by the EscrowCreate's sequence.
    Escrow { account: String, sequence: u32 },
    /// PayChannel, by source, destination and the PaymentChannelCreate's sequence.
    Channel {
        source: String,
        destination: String,
        sequence: u32,
    },
    /// Ticket.
    Ticket {
        account: String,
        ticket_sequence: u32,
    },
    /// The owner directory.
    OwnerDir { account: String },
}

#[derive(Subcommand)]
enum ConvertCmd {
    /// XRP → drops.
    XrpToDrops { xrp: String },
    /// Drops → XRP.
    DropsToXrp { drops: u64 },
    /// Ledger time ↔ ISO 8601: give either.
    Time { value: String },
    /// Split an NFTokenID into issuer, taxon, fee, flags and serial.
    NftId { id: String },
    /// Text → hex (for Domain, memos, URIs).
    Hex { text: String },
    /// Hex → text.
    Text { hex: String },
}

fn print(v: &impl serde::Serialize) -> Res {
    println!("{}", serde_json::to_string_pretty(v)?);
    Ok(())
}

/// JSON from an argument, or from stdin when it is `-`.
fn read_json(arg: &str) -> Res<Value> {
    let text = if arg == "-" {
        std::io::read_to_string(std::io::stdin())?
    } else {
        arg.to_string()
    };
    Ok(serde_json::from_str(&text)?)
}

fn wallet_from(seed: &SeedSource) -> Res<Wallet> {
    let secret = match &seed.seed_file {
        Some(path) => std::fs::read_to_string(path)?.trim().to_string(),
        None => std::env::var("XRPL_SEED").map_err(|_| {
            "set XRPL_SEED (or --seed-file); seeds are not taken as arguments, which shell history keeps"
        })?,
    };
    Ok(Wallet::from_seed(&secret)?)
}

/// A currency code as rippled takes it: three characters as they are, 40 hex
/// digits as they are, and anything longer (`RLUSD`) as its ASCII in hex,
/// padded to 40 digits.
fn currency_code(code: &str) -> Res<String> {
    if code.len() == 3 || (code.len() == 40 && code.bytes().all(|b| b.is_ascii_hexdigit())) {
        return Ok(code.to_string());
    }
    if code.is_empty() || code.len() > 20 || !code.is_ascii() {
        return Err(format!(
            "{code}: a currency code is 3 characters, up to 20 ASCII characters, or 40 hex digits"
        )
        .into());
    }
    Ok(format!("{:0<40}", utils::str_to_hex(code)))
}

/// `XRP` → `{"currency":"XRP"}`; `USD.rIssuer` → `{"currency":"USD","issuer":"rIssuer"}`.
fn asset(spec: &str) -> Res<Value> {
    match spec.split_once('.') {
        None if spec.eq_ignore_ascii_case("XRP") => Ok(json!({"currency": "XRP"})),
        Some((currency, issuer)) => {
            Ok(json!({"currency": currency_code(currency)?, "issuer": issuer}))
        }
        None => Err(format!("{spec}: expected XRP or CURRENCY.rIssuer").into()),
    }
}

fn ledger_index(s: &str) -> Res<LedgerIndex> {
    Ok(match s {
        "validated" => LedgerIndex::Validated,
        "current" => LedgerIndex::Current,
        "closed" => LedgerIndex::Closed,
        n => LedgerIndex::Index(n.parse()?),
    })
}

fn mainnet_needs_yes(network: Network, yes: bool) -> Res {
    if network == Network::Mainnet && !yes {
        return Err("this sends a transaction on mainnet; add --yes to go ahead".into());
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli).await {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Res {
    let rpc = cli
        .url
        .clone()
        .unwrap_or_else(|| cli.network.rpc().to_string());
    let client = || XrplHttpClient::new(&rpc);
    let v = LedgerIndex::Validated;
    match cli.command {
        Command::Wallet(cmd) => match cmd {
            WalletCmd::New { secp256k1 } => {
                let w = if secp256k1 {
                    Wallet::generate_with_type(KeyType::Secp256k1)
                } else {
                    Wallet::generate()
                };
                print(
                    &json!({"address": w.classic_address(), "public_key": w.public_key_hex(), "seed": w.seed()}),
                )?;
                eprintln!("Keep the seed private: it controls the account.");
            }
            WalletCmd::Show(seed) => {
                let w = wallet_from(&seed)?;
                print(
                    &json!({"address": w.classic_address(), "public_key": w.public_key_hex(), "key_type": format!("{:?}", w.keypair.key_type)}),
                )?;
            }
            WalletCmd::Faucet => {
                let net = cli
                    .network
                    .faucet()
                    .ok_or("there is no faucet on mainnet")?;
                eprintln!("Funding a new wallet from the {net:?} faucet…");
                let w = generate_faucet_wallet(net).await?;
                print(
                    &json!({"address": w.classic_address(), "seed": w.seed(), "network": format!("{net:?}")}),
                )?;
            }
        },
        Command::Account(cmd) => {
            let c = client()?;
            match cmd {
                AccountCmd::Info { address } => print(&c.account_info(&address, &v).await?)?,
                AccountCmd::Lines { address } => print(&c.account_lines_all(&address, &v).await?)?,
                AccountCmd::Offers { address } => {
                    print(&c.account_offers_all(&address, &v).await?)?
                }
                AccountCmd::Objects { address, kind } => {
                    print(&c.account_objects_all(&address, kind.as_deref(), &v).await?)?
                }
                AccountCmd::Nfts { address } => print(&c.account_nfts_all(&address, &v).await?)?,
                AccountCmd::Tx { address, limit } => print(
                    &c.account_tx(&address, None, None, Some(limit), None, None)
                        .await?,
                )?,
                AccountCmd::Currencies { address } => {
                    print(&c.account_currencies(&address, &v).await?)?
                }
                AccountCmd::Channels { address } => {
                    print(&c.account_channels(&address, None, &v).await?)?
                }
            }
        }
        Command::Tx(cmd) => match cmd {
            TxCmd::Get { hash } => print(&client()?.tx(&hash).await?)?,
            TxCmd::Hash { tx } => {
                let id = if tx.trim_start().starts_with('{') || tx == "-" {
                    hash_signed_tx(&read_json(&tx)?)?
                } else {
                    hash_signed_tx_blob(tx.trim())?
                };
                println!("{id}");
            }
            TxCmd::Decode { blob } => {
                print(&decode_transaction_binary(&hex::decode(blob.trim())?)?)?
            }
            TxCmd::Encode { json } => println!(
                "{}",
                hex::encode_upper(encode_transaction_json(&read_json(&json)?, false)?)
            ),
            TxCmd::Sign {
                json,
                autofill,
                seed,
            } => {
                let wallet = wallet_from(&seed)?;
                let mut tx = read_json(&json)?;
                if autofill {
                    client()?.autofill(&mut tx).await?;
                }
                let blob = wallet.sign_and_encode(&tx)?;
                print(&json!({"tx_blob": blob, "hash": hash_signed_tx_blob(&blob)?.to_string()}))?;
            }
            TxCmd::Simulate { json } => print(&client()?.simulate(&read_json(&json)?).await?)?,
            TxCmd::Submit { blob, wait, yes } => {
                mainnet_needs_yes(cli.network, yes)?;
                let c = client()?;
                if wait {
                    let tx = decode_transaction_binary(&hex::decode(blob.trim())?)?;
                    let last = tx
                        .get("LastLedgerSequence")
                        .and_then(Value::as_u64)
                        .ok_or("--wait needs a LastLedgerSequence")?;
                    print(&c.submit_and_wait(blob.trim(), last as u32).await?)?;
                } else {
                    print(&c.submit(blob.trim()).await?)?;
                }
            }
            TxCmd::SendXrp {
                destination,
                amount,
                tag,
                yes,
                seed,
            } => {
                mainnet_needs_yes(cli.network, yes)?;
                let wallet = wallet_from(&seed)?;
                let mut tx = json!({
                    "TransactionType": "Payment",
                    "Account": wallet.classic_address(),
                    "Destination": destination,
                    "Amount": utils::xrp_to_drops(&amount)?.to_string(),
                });
                if let (Some(tag), Some(obj)) = (tag, tx.as_object_mut()) {
                    obj.insert("DestinationTag".into(), json!(tag));
                }
                let c = client()?;
                let result = submit_transaction(&c, &mut tx, &wallet).await?;
                let outcome = result.transaction_result().unwrap_or("?").to_string();
                print(
                    &json!({"hash": result.hash(), "result": outcome, "ledger_index": result.ledger_index}),
                )?;
            }
        },
        Command::Ledger { ledger } => print(&client()?.ledger(&ledger_index(&ledger)?).await?)?,
        Command::Entry { id } => print(&client()?.ledger_entry(&id, &v).await?)?,
        Command::Id(cmd) => {
            let id = match cmd {
                IdCmd::Account { address } => keylet::account(&address)?,
                IdCmd::TrustLine { a, b, currency } => {
                    keylet::trust_line(&a, &b, &currency_code(&currency)?)?
                }
                IdCmd::Offer { account, sequence } => keylet::offer(&account, sequence)?,
                IdCmd::Check { account, sequence } => keylet::check(&account, sequence)?,
                IdCmd::Escrow { account, sequence } => keylet::escrow(&account, sequence)?,
                IdCmd::Channel {
                    source,
                    destination,
                    sequence,
                } => keylet::payment_channel(&source, &destination, sequence)?,
                IdCmd::Ticket {
                    account,
                    ticket_sequence,
                } => keylet::ticket(&account, ticket_sequence)?,
                IdCmd::OwnerDir { account } => keylet::owner_dir(&account)?,
            };
            println!("{id}");
        }
        Command::Fee => print(&client()?.fee().await?)?,
        Command::Server => print(&client()?.server_info().await?)?,
        Command::Book {
            taker_gets,
            taker_pays,
            limit,
        } => {
            let req = requests::BookOffersRequest {
                limit: Some(limit),
                ..requests::BookOffersRequest::new(asset(&taker_gets)?, asset(&taker_pays)?)
            };
            print(&client()?.request(&req).await?)?;
        }
        Command::Watch { stream, count } => {
            let ws_url = cli
                .url
                .clone()
                .filter(|u| u.starts_with("ws"))
                .unwrap_or_else(|| cli.network.ws().to_string());
            let ws = XrplWsClient::connect(&ws_url).await?;
            let mut events = match stream {
                Stream::Ledger => ws.subscribe_ledger().await?,
                Stream::Transactions => ws.subscribe_transactions().await?,
                Stream::Validations => ws.subscribe_validations().await?,
                Stream::BookChanges => ws.subscribe_book_changes().await?,
                Stream::Server => ws.subscribe_server().await?,
                Stream::Consensus => ws.subscribe_consensus().await?,
            };
            let mut seen = 0;
            while seen < count {
                let event = events.recv().await?;
                let line = match event {
                    WsEvent::Ledger(e) => serde_json::to_string(&e)?,
                    WsEvent::Transaction(e) => serde_json::to_string(&e)?,
                    WsEvent::Validation(e) => serde_json::to_string(&e)?,
                    WsEvent::BookChanges(e) => serde_json::to_string(&e)?,
                    WsEvent::Server(e) => serde_json::to_string(&e)?,
                    WsEvent::Consensus(e) => serde_json::to_string(&e)?,
                    _ => continue,
                };
                println!("{line}");
                seen += 1;
            }
        }
        Command::Convert(cmd) => match cmd {
            ConvertCmd::XrpToDrops { xrp } => println!("{}", utils::xrp_to_drops(&xrp)?),
            ConvertCmd::DropsToXrp { drops } => println!("{}", utils::drops_to_xrp(drops)),
            ConvertCmd::Time { value } => match value.parse::<u32>() {
                Ok(ripple) => println!("{}", utils::ripple_time_to_iso8601(ripple)),
                Err(_) => println!("{}", utils::iso8601_to_ripple_time(&value)?),
            },
            ConvertCmd::NftId { id } => {
                let p = utils::parse_nftoken_id(&id)?;
                print(
                    &json!({"issuer": p.issuer, "taxon": p.taxon, "sequence": p.sequence, "transfer_fee": p.transfer_fee, "flags": p.flags, "burnable": p.is_burnable(), "only_xrp": p.is_only_xrp(), "transferable": p.is_transferable(), "mutable": p.is_mutable()}),
                )?;
            }
            ConvertCmd::Hex { text } => println!("{}", utils::str_to_hex(&text)),
            ConvertCmd::Text { hex } => println!("{}", utils::hex_to_str(&hex)?),
        },
        Command::Raw { method, params } => print(
            &client()?
                .raw_request(&method, serde_json::from_str(&params)?)
                .await?,
        )?,
    }
    Ok(())
}
