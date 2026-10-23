#!/usr/bin/env python3
"""Ask a rippled node where its transaction cost starts, for the rules in
src/transaction/cost.rs (and docs/PARITY.md).

It uses `simulate` only, which applies nothing: unsigned transactions from the
genesis account at fees just below and at the cost cost.rs computes. Below
the cost rippled refuses the fee (telCAN_NOT_QUEUE when the open ledger is
not escalated); at the cost it moves on to the transaction's own checks
(tecHAS_OBLIGATIONS, tecNO_TARGET, ...), so the fee was enough.

Recorded against rippled 3.4.1 on mainnet, 2026-10-23 (base fee 10 drops,
owner reserve 200,000 drops):

    AccountDelete                     199999 -> telCAN_NOT_QUEUE   200000 -> tecHAS_OBLIGATIONS
    EscrowFinish, 100-byte Fulfillment   389 -> telCAN_NOT_QUEUE      390 -> tecNO_TARGET

RPC defaults to http://127.0.0.1:5005; set RIPPLED_RPC for another node (it
must allow `simulate`). Standard library only.
"""

import json
import os
import urllib.request

RPC = os.environ.get("RIPPLED_RPC", "http://127.0.0.1:5005")
GENESIS = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"
OTHER = "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe"


def rpc(method, params):
    body = json.dumps({"method": method, "params": [params]}).encode()
    req = urllib.request.Request(RPC, body, {"content-type": "application/json"})
    return json.load(urllib.request.urlopen(req, timeout=30))["result"]


def main():
    seq = rpc("account_info", {"account": GENESIS, "ledger_index": "current"})["account_data"]["Sequence"]
    ledger = rpc("server_state", {})["state"]["validated_ledger"]
    base, reserve = ledger["base_fee"], ledger["reserve_inc"]
    common = {"Account": GENESIS, "Sequence": seq, "SigningPubKey": ""}
    cases = [
        ("AccountDelete", {"TransactionType": "AccountDelete", "Destination": OTHER}, reserve),
        (
            "EscrowFinish, 100-byte Fulfillment",
            {
                "TransactionType": "EscrowFinish",
                "Owner": GENESIS,
                "OfferSequence": 1,
                "Condition": "A0258020" + "00" * 32 + "810120",
                "Fulfillment": "A0" * 100,
            },
            base * (1 + 32 + 100 // 16),
        ),
    ]
    for name, fields, cost in cases:
        answers = []
        for fee in (cost - 1, cost):
            tx = dict(common, **fields, Fee=str(fee))
            answers.append(f"{fee:>6} -> {rpc('simulate', {'tx_json': tx}).get('engine_result')}")
        print(f"{name:<36}", "   ".join(answers))


main()
