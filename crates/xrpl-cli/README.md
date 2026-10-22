# hxrpl

The XRP Ledger from the command line, built on
[`halcyon-xrpl-sdk`](https://crates.io/crates/halcyon-xrpl-sdk).

```sh
cargo install halcyon-xrpl-cli
hxrpl wallet faucet                                   # a funded Testnet wallet
hxrpl --network mainnet account info rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh
XRPL_SEED=s… hxrpl tx send-xrp rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe 1.5
hxrpl --network mainnet watch ledger --count 5        # live ledger closes
hxrpl --network mainnet book XRP RLUSD.rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De
hxrpl tx hash 120000220000000024…                     # a signed blob's ID, offline
hxrpl id trust-line rA… rB… USD                       # a ledger entry's ID, offline
```

| Command | Does |
|---|---|
| `wallet new` / `show` / `faucet` | make a wallet, show the one in `XRPL_SEED`, or get a funded Testnet / Devnet one |
| `account info` / `lines` / `offers` / `objects` / `nfts` / `tx` / `currencies` / `channels` | query an account (every page) |
| `tx get` / `hash` / `decode` / `encode` / `sign` / `simulate` / `submit` / `send-xrp` | transactions, offline and online |
| `ledger`, `entry`, `id …` | ledger headers, entries, and the ID of any entry |
| `fee`, `server`, `book` | the transaction cost, the server's state, an order book |
| `watch ledger` / `transactions` / `validations` / `book-changes` / `server` / `consensus` | live streams |
| `convert xrp-to-drops` / `drops-to-xrp` / `time` / `nft-id` / `hex` / `text` | conversions |
| `raw <method> '<json>'` | any rippled method |

Safe by default:

- It works on **Testnet** unless told otherwise (`--network mainnet` or `XRPL_NETWORK=mainnet`).
- Sending on mainnet also needs `--yes`.
- Seeds come from `XRPL_SEED` or `--seed-file`, never from arguments, which shell history keeps.

MIT, with an attribution requirement: credit James Turner with a link to
https://github.com/ilubxrp589/xrpl-sdk. See [LICENSE](../../LICENSE).
