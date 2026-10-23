#!/usr/bin/env python3
"""Re-record the response and stream fixtures (tests/fixtures/rippled and
tests/fixtures/streams) from real servers, in rippled's API version 2.

    record_fixtures.py responses --rippled http://127.0.0.1:5005 [--clio https://s2.ripple.com:51234]
    record_fixtures.py streams --ws ws://127.0.0.1:6006

`responses` sends each recorded request again, with the same parameters and
`api_version: 2`, to a mainnet rippled (the Clio-only methods to a mainnet
Clio) and writes the answer in its place. The Testnet recordings (marked
`"network": "testnet"`) are made by the SDK's live test instead
(`XRPL_RECORD_DIR`). Only methods that read are sent: one that submits or
signs is refused.

`streams` subscribes to a rippled's streams (an admin connection, for the
validation and peer streams) and keeps a few messages of each kind, and an
order book snapshot.
"""

import argparse
import asyncio
import json
import pathlib
import sys
import urllib.request

HERE = pathlib.Path(__file__).resolve().parent.parent / "tests" / "fixtures"
API_VERSION = 2
CLIO_ONLY = {"nft_info", "nft_history", "nfts_by_issuer", "mpt_holders"}
NEVER = {"submit", "submit_multisigned", "sign", "sign_for", "wallet_propose"}


def post(url, method, params):
    body = json.dumps({"method": method, "params": [params]}).encode()
    req = urllib.request.Request(url, data=body, headers={"content-type": "application/json"})
    with urllib.request.urlopen(req, timeout=60) as resp:
        return json.load(resp)["result"]


def write(path, value):
    path.write_text(json.dumps(value, indent=1) + "\n")


def responses(args):
    for path in sorted((HERE / "rippled").glob("*.json")):
        recorded = json.loads(path.read_text())
        method = recorded["method"]
        if recorded.get("network") == "testnet":
            print(f"{path.name}: Testnet, recorded by the live test")
            continue
        if method in NEVER:
            sys.exit(f"{path.name}: {method} is not a read; refusing to send it")
        url = args.clio if method in CLIO_ONLY else args.rippled
        params = {k: v for k, v in recorded["params"].items() if k != "api_version"}
        params["api_version"] = API_VERSION
        result = post(url, method, params)
        if result.get("status") != "success":
            sys.exit(f"{path.name}: {result.get('error')}: {result.get('error_message')}")
        write(path, {"method": method, "params": params, "result": result})
        print(f"{path.name}: recorded")


async def snapshot(websockets, url):
    """An order book snapshot (XRP/RLUSD), two offers each side."""
    rlusd = {"currency": "524C555344000000000000000000000000000000",
             "issuer": "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De"}
    async with websockets.connect(url, max_size=2**25) as ws:
        await ws.send(json.dumps({
            "id": 1, "command": "subscribe", "api_version": API_VERSION,
            "books": [{"taker_gets": {"currency": "XRP"}, "taker_pays": rlusd,
                       "snapshot": True, "both": True}],
        }))
        while True:
            reply = json.loads(await ws.recv())
            if reply.get("id") == 1:
                break
    result = reply["result"]
    write(HERE / "streams" / "books_snapshot.json",
          {"asks": result["asks"][:2], "bids": result["bids"][:2]})
    print("books_snapshot.json: recorded")


async def collect(websockets, url, streams, wanted, seconds):
    """Up to `wanted[kind]` messages of each kind from one connection to
    `streams`, one transaction of each type. Messages are read as fast as
    they come (rippled drops a slow client) and parsed only when needed."""
    kept = {k: [] for k in wanted}
    seen_types = set()
    async with websockets.connect(url, max_size=2**25, max_queue=None) as ws:
        await ws.send(json.dumps({"id": 1, "command": "subscribe",
                                  "api_version": API_VERSION, "streams": streams}))
        deadline = asyncio.get_running_loop().time() + seconds
        while asyncio.get_running_loop().time() < deadline:
            if all(len(kept[k]) >= n for k, n in wanted.items()):
                break
            try:
                text = await asyncio.wait_for(ws.recv(), timeout=5)
            except asyncio.TimeoutError:
                continue
            kind = next((k for k in wanted if f'"type":"{k.split("_")[0]}"' in text), None)
            if kind is None or len(kept[kind]) >= wanted[kind]:
                continue
            message = json.loads(text)
            if message.get("type") == "transaction":
                tx_type = message["tx_json"].get("TransactionType")
                if tx_type in seen_types:
                    continue
                seen_types.add(tx_type)
            kept[kind].append(message)
    return kept


async def streams(args):
    import websockets  # pip install websockets

    await snapshot(websockets, args.ws)
    # One connection per group, so no connection carries more than it can
    # read: the proposed transactions alone are hundreds a second.
    groups = [
        (["transactions"], {"transaction_validated": 4}),
        (["transactions_proposed"], {"transaction_proposed": 4}),
        (["ledger", "validations", "consensus", "book_changes"],
         {"ledgerClosed": 2, "validationReceived": 3, "consensusPhase": 2, "bookChanges": 2}),
        (["server", "peer_status"], {"serverStatus": 1, "peerStatusChange": 2}),
    ]
    results = await asyncio.gather(*(collect(websockets, args.ws, streams, wanted, args.seconds)
                                     for streams, wanted in groups))
    for (_, wanted), kept in zip(groups, results):
        for kind, messages in kept.items():
            if len(messages) < wanted[kind]:
                # The rare kinds (peer and server status) are written the same
                # way in every API version: an older recording stays valid.
                if kind.startswith("transaction"):
                    sys.exit(f"{kind}: only {len(messages)} messages in {args.seconds} s")
                print(f"{kind}.json: only {len(messages)} in {args.seconds} s; left as it was")
                continue
            write(HERE / "streams" / f"{kind}.json", messages)
            print(f"{kind}.json: {len(messages)} messages")


def main():
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest="what", required=True)
    r = sub.add_parser("responses")
    r.add_argument("--rippled", required=True, help="a mainnet rippled's JSON-RPC URL")
    r.add_argument("--clio", default="https://s2.ripple.com:51234", help="a mainnet Clio")
    s = sub.add_parser("streams")
    s.add_argument("--ws", required=True, help="a mainnet rippled's admin WebSocket URL")
    s.add_argument("--seconds", type=int, default=600)
    args = parser.parse_args()
    if args.what == "responses":
        responses(args)
    else:
        asyncio.run(streams(args))


if __name__ == "__main__":
    main()
