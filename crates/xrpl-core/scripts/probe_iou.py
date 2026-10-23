#!/usr/bin/env python3
"""Ask a rippled node how it reads token (IOU) values from JSON, for the
table in src/types/amount.rs (`reads_token_values_as_rippled_does`).

It uses `simulate` only (applies nothing): a Payment from the genesis
account for {USD, value}; prints the mantissa and exponent rippled wrote,
or its refusal.

    probe_iou.py 1.5 12345678901234565 1e-110 .5

RPC defaults to http://127.0.0.1:5005; set RIPPLED_RPC for another node (it
must allow `simulate`). Standard library only.
"""
import json, os, sys, urllib.request, urllib.error
RPC = os.environ.get("RIPPLED_RPC", "http://127.0.0.1:5005")
G = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"
def rpc(m, p):
    r = urllib.request.Request(RPC, json.dumps({"method": m, "params": [p]}).encode(), {"content-type": "application/json"})
    return json.load(urllib.request.urlopen(r, timeout=15))["result"]
for v in sys.argv[1:]:
    tx = {"TransactionType": "Payment", "Account": G, "Destination": "rU6K7V3Po4snVhBBaU29sesqs2qTQJWDw1",
          "Amount": {"currency": "USD", "issuer": "rU6K7V3Po4snVhBBaU29sesqs2qTQJWDw1", "value": v},
          "Fee": "12", "Sequence": 1, "Flags": 0, "SigningPubKey": ""}
    try:
        res = rpc("simulate", {"tx_json": tx, "binary": True})
    except urllib.error.HTTPError as e:
        print(f"{v!r:32} HTTP {e.code}"); continue
    blob = res.get("tx_blob")
    if not blob:
        print(f"{v!r:32} refused: {res.get('error')} {res.get('error_exception') or res.get('error_message')}"); continue
    b = bytes.fromhex(blob); i = b.find(bytes([0x61]))
    raw = int.from_bytes(b[i+1:i+9], "big")
    if raw == 0x8000000000000000:
        print(f"{v!r:32} zero"); continue
    neg = not (raw >> 62) & 1; exp = ((raw >> 54) & 0xFF) - 97; man = raw & ((1 << 54) - 1)
    print(f"{v!r:32} {'-' if neg else ''}{man}e{exp}")
