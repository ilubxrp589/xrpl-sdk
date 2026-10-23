#!/usr/bin/env python3
"""Seed each fuzz target's corpus (fuzz/corpus/<target>/) from the real
transactions, ledger entries and metadata the tests use: rippled 3.4.1's
signed transactions, mainnet and Testnet transactions, mainnet ledger entries,
XRPLF's codec fixtures and xrpl.js's metadata fixtures. A fuzzer that starts
from real XRPL data reaches deep paths in seconds instead of hours.

    python3 fuzz/seed_corpus.py      # from the repository root; idempotent

Seeds are named by their SHA-1, as libFuzzer names its own, so re-running
adds nothing twice. Standard library only.
"""

import hashlib
import json
import pathlib

ROOT = pathlib.Path(__file__).resolve().parent.parent
FIXTURES = ROOT / "crates/xrpl-core/tests/fixtures"
CORPUS = ROOT / "fuzz/corpus"


def lines(name):
    return [json.loads(l) for l in (FIXTURES / name).read_text().splitlines() if l.strip()]


def compact(value):
    return json.dumps(value, separators=(",", ":")).encode()


signed = lines("rippled_signed_transactions.jsonl")
transactions = lines("transactions.jsonl")
ledger_objects = lines("mainnet_ledger_objects.jsonl")
codec = json.loads((ROOT / "tests/vectors/codec-fixtures.json").read_text())
codec_entries = codec["transactions"] + codec["accountState"]
metadata = []
for path in sorted((FIXTURES / "xrpljs").glob("*.json")):
    doc = json.loads(path.read_text())
    if isinstance(doc, dict) and isinstance(doc.get("meta"), dict):
        metadata.append(doc["meta"])

blobs = [bytes.fromhex(r["tx_blob"]) for r in signed] + [bytes.fromhex(e["binary"]) for e in codec_entries]
tx_json = [compact(r["tx_json"]) for r in signed] + [compact(t) for t in transactions]

# A real binary `ledger` answer cut to its first two transactions: small
# enough to mutate quickly, and still the shape verify_ledger_response reads.
small_ledger = json.loads((FIXTURES / "mainnet_ledger_107852951.json").read_text())["binary"]
small_ledger["ledger"]["transactions"] = small_ledger["ledger"]["transactions"][:2]

seeds = {
    "fuzz_decode": blobs,
    "fuzz_roundtrip": blobs,
    "fuzz_encode": tx_json + [compact(e["json"]) for e in codec_entries],
    "fuzz_models": tx_json + [compact(o) for o in ledger_objects],
    "fuzz_hashes": [bytes.fromhex(r["tx_blob"]) for r in signed] + [compact(t) for t in transactions],
    "fuzz_utils": [compact(m) for m in metadata]
    + [
        s.encode()
        for s in [
            "1.5", "0.000001", "1.01", "1%", "100000000000",
            "2024-06-01T12:00:00Z", "2000-01-01T00:00:00Z",
            "000B0539C35B55AA096BA6D87A6E6C965A6534150DC56E5E12C5D09E0000",
            "X7AcgcsBL6XDcUb289X4mJ8djcdyKaB5hJDWMArnXr61cqZ",
            "10.5|RLUSD|rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De",
            "-1e-81|USD|rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "48616C63796F6E",
        ]
    ],
    "fuzz_address": [
        s.encode()
        for s in [
            "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", "rrrrrrrrrrrrrrrrrrrrrhoLvTp",
            "sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r", "sp5fghtJtpUorTwvof1NpDXAzNwf5",
        ]
    ],
    "fuzz_keys": [
        s.encode()
        for s in [v[1] for v in json.loads((FIXTURES.parent / "vectors/bip39_trezor.json").read_text())["english"]]
        + [
            "399150 474506 009147 088773 432160 282843 253738 605430",
            "YOKE DREW DIME DRUM LAIN NIBS DUAL PEA BAIT CHOW JOY FILE",
            "m/44'/144'/0'/0/0", "m/0h/1/2H", "secp256k1",
        ]
    ]
    + [compact(small_ledger)]
    + [compact(r["tx_json"]) for r in signed[:20]],
    "fuzz_amount": [
        bytes.fromhex(h)
        for h in [
            "4000000000000064",  # 100 drops
            "0000000000000000",  # -0 drops
            "8000000000000000",  # token zero
            "D4838D7EA4C68000",  # 1 (token value)
            "D4838D7EA4C68000" + "0000000000000000000000005553440000000000"
            + "B5F762798A53D543A014CAF8B297CFF8F2F937E8",  # 1 USD
        ]
    ],
}

added = 0
for target, items in seeds.items():
    directory = CORPUS / target
    directory.mkdir(parents=True, exist_ok=True)
    for data in items:
        path = directory / hashlib.sha1(data).hexdigest()
        if not path.exists():
            path.write_bytes(data)
            added += 1
    print(f"{target}: {len(list(directory.iterdir()))} seeds")
print(f"{added} added")
