#!/usr/bin/env python3
"""Ask a rippled node how it writes and reads STNumber values, for the tables
in src/codec/number.rs (and docs/PARITY.md).

It uses `simulate` only, which applies nothing: a VaultCreate from the genesis
account whose AssetsMaximum holds the value under test. A node where the
SingleAssetVault amendment is not enabled answers temDISABLED, which is fine:
the answer still shows how rippled parsed the transaction.

    # JSON -> the bytes rippled writes and the JSON it shows back
    probe_number.py encode 1.5 138307567657966440 9999999999999999999
    # raw bytes (mantissa,exponent) -> the JSON rippled shows for them
    probe_number.py decode 13830756765796644,1 1,-40000

RPC defaults to http://127.0.0.1:5005; set RIPPLED_RPC for another node (it
must allow `simulate`). Standard library only.
"""

import json
import os
import struct
import sys
import urllib.error
import urllib.request

RPC = os.environ.get("RIPPLED_RPC", "http://127.0.0.1:5005")
GENESIS = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"
GENESIS_ID = bytes.fromhex("B5F762798A53D543A014CAF8B297CFF8F2F937E8")


def rpc(method, params):
    body = json.dumps({"method": method, "params": [params]}).encode()
    request = urllib.request.Request(RPC, body, {"content-type": "application/json"})
    return json.load(urllib.request.urlopen(request, timeout=15))["result"]


def vault_create(assets_maximum):
    return {
        "TransactionType": "VaultCreate", "Account": GENESIS, "Asset": {"currency": "XRP"},
        "AssetsMaximum": assets_maximum, "Fee": "12", "Sequence": 1, "Flags": 0, "SigningPubKey": "",
    }


def vault_create_blob(number):
    """The same VaultCreate, unsigned, with 12 raw STNumber bytes."""
    b = bytes([0x12]) + struct.pack(">H", 65)  # TransactionType VaultCreate
    b += bytes([0x22]) + struct.pack(">I", 0)  # Flags
    b += bytes([0x24]) + struct.pack(">I", 1)  # Sequence
    b += bytes([0x68]) + struct.pack(">Q", 0x4000000000000000 | 12)  # Fee 12 drops
    b += bytes([0x73, 0x00])  # SigningPubKey ""
    b += bytes([0x81, 0x14]) + GENESIS_ID  # Account
    b += bytes([0x93]) + number  # AssetsMaximum
    b += bytes([0x03, 0x18]) + bytes(20)  # Asset: XRP
    return b.hex().upper()


def encode(value):
    try:
        written = rpc("simulate", {"tx_json": vault_create(value), "binary": True})
    except urllib.error.HTTPError as e:
        return f"refused (HTTP {e.code})"
    if "tx_blob" not in written:
        return f"refused: {written.get('error_message') or written.get('error')}"
    blob = written["tx_blob"]
    at = blob.index(GENESIS_ID.hex().upper()) + 40
    assert blob[at:at + 2] == "93", blob
    mantissa, exponent = struct.unpack(">qi", bytes.fromhex(blob[at + 2:at + 26]))
    shown = rpc("simulate", {"tx_json": vault_create(value)})["tx_json"]["AssetsMaximum"]
    return f"mantissa {mantissa} exponent {exponent}; shown as {shown!r}"


def decode(mantissa, exponent):
    answer = rpc("simulate", {"tx_blob": vault_create_blob(struct.pack(">qi", mantissa, exponent))})
    if "tx_json" not in answer:
        return f"refused: {answer.get('error_message') or answer.get('error')}"
    return f"shown as {answer['tx_json']['AssetsMaximum']!r}"


def main():
    if len(sys.argv) < 3 or sys.argv[1] not in ("encode", "decode"):
        sys.exit(__doc__)
    for arg in sys.argv[2:]:
        if sys.argv[1] == "encode":
            value = json.loads(arg) if arg.lstrip("-").isdigit() and os.environ.get("AS_JSON_NUMBER") else arg
            print(f"{arg!r:30} -> {encode(value)}")
        else:
            mantissa, exponent = (int(x) for x in arg.split(","))
            print(f"{mantissa},{exponent:<12} -> {decode(mantissa, exponent)}")


if __name__ == "__main__":
    main()
