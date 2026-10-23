#!/usr/bin/env python3
"""Compare rippled's definitions on the public networks with the ones the
SDK is generated from, and say exactly what differs.

    check_definitions.py [--report drift.md] [--committed FILE] [--network NAME=URL ...]
    check_definitions.py --offline

The SDK's codec, transaction models, ledger entry types and flags are all
generated from one recorded `server_definitions` answer
(crates/xrpl-client/tests/fixtures/rippled/server_definitions.json). When a
network's servers run a `rippled` that knows a new transaction type, field,
ledger entry, result code or flag, its definitions change (and so does their
`hash`). Devnet gets a new `rippled` first, then Testnet, then mainnet.

For each network the script asks a public server for `server_definitions`
and `server_info`, and compares section by section: types, fields,
transaction and ledger entry types and their formats, result codes, flags.
With `--report`, a Markdown report is written when anything differs (and
nothing is written when nothing does); CI opens an issue from it
(.github/workflows/definitions.yml). The exit code is 0 unless no network
answered at all.

`--offline` uses no network: it checks that the codec's copy
(crates/xrpl-core/src/codec/definitions.json) holds exactly the recorded
answer's types, fields, transaction and ledger entry types and result
codes, so the two cannot drift apart. CI runs it on every change.
"""

import argparse
import json
import pathlib
import sys
import urllib.request

ROOT = pathlib.Path(__file__).resolve().parents[3]
COMMITTED = ROOT / "crates/xrpl-client/tests/fixtures/rippled/server_definitions.json"
CODEC = ROOT / "crates/xrpl-core/src/codec/definitions.json"

# Public servers, in the order tried: `rippled` first. (A Clio server
# answers with its own build of `rippled`'s library, which can differ from
# the network's `rippled`.)
NETWORKS = {
    "Devnet": ["https://s.devnet.rippletest.net:51234"],
    "Testnet": ["https://s.altnet.rippletest.net:51234", "https://testnet.xrpl-labs.com"],
    "Mainnet": ["https://xrplcluster.com", "https://xrpl.ws", "https://s1.ripple.com:51234"],
}

FIELD_PROPS = ("type", "nth", "isSerialized", "isSigningField", "isVLEncoded")


def post(url, method):
    body = json.dumps({"method": method, "params": [{"api_version": 2}]}).encode()
    request = urllib.request.Request(
        url, data=body,
        headers={"content-type": "application/json",
                 "user-agent": "halcyon-xrpl-sdk definitions check"})
    with urllib.request.urlopen(request, timeout=30) as response:
        result = json.load(response)["result"]
    if result.get("status") != "success":
        raise RuntimeError(f"{method}: {result.get('error')}")
    return result


def fetch(urls):
    """(definitions, rippled version, url) from the first server that answers."""
    errors = []
    for url in urls:
        try:
            definitions = post(url, "server_definitions")
            info = post(url, "server_info").get("info", {})
            if "clio_version" in info:
                version = f"{info.get('rippled_version', '?')} (via Clio {info['clio_version']})"
            else:
                version = info.get("build_version", "?")
            return definitions, version, url
        except Exception as error:  # noqa: BLE001 - any failure: try the next server
            errors.append(f"{url}: {error}")
    raise RuntimeError("; ".join(errors))


def sections(d):
    """Each section of a definitions answer as a flat {name: value} map."""
    fields = {name: {k: props.get(k) for k in FIELD_PROPS} for name, props in d.get("FIELDS", [])}

    def formats(section):
        return {f"{owner}.{entry['name']}": ("required", "optional", "default")[entry["optionality"]]
                if entry["optionality"] in (0, 1, 2) else entry["optionality"]
                for owner, entries in d.get(section, {}).items() for entry in entries}

    def flags(section):
        return {f"{owner}.{flag}": value
                for owner, table in d.get(section, {}).items() for flag, value in table.items()}

    return {
        "Types": d.get("TYPES", {}),
        "Fields": fields,
        "Transaction types": d.get("TRANSACTION_TYPES", {}),
        "Transaction formats (field: required/optional)": formats("TRANSACTION_FORMATS"),
        "Transaction flags": flags("TRANSACTION_FLAGS"),
        "AccountSet flags (asf)": d.get("ACCOUNT_SET_FLAGS", {}),
        "Ledger entry types": d.get("LEDGER_ENTRY_TYPES", {}),
        "Ledger entry formats (field: required/optional)": formats("LEDGER_ENTRY_FORMATS"),
        "Ledger entry flags": flags("LEDGER_ENTRY_FLAGS"),
        "Result codes": d.get("TRANSACTION_RESULTS", {}),
    }


def show(value):
    if isinstance(value, dict):
        return ", ".join(f"{k} {v}" for k, v in value.items() if k != "isSerialized" or not v)
    return f"{value}"


def change(before, after):
    """What changed: only the properties that differ, for a field."""
    if isinstance(before, dict) and isinstance(after, dict):
        keys = [k for k in {**before, **after} if before.get(k) != after.get(k)]
        return ", ".join(f"{k} {before.get(k)} → {after.get(k)}" for k in keys)
    return f"{before} → {after}"


def compare(old, new):
    """Markdown lines for each section that differs."""
    lines = []
    for name, before in sections(old).items():
        after = sections(new)[name]
        added = sorted(set(after) - set(before))
        removed = sorted(set(before) - set(after))
        changed = sorted(k for k in set(before) & set(after) if before[k] != after[k])
        if not (added or removed or changed):
            continue
        lines.append(f"**{name}**")
        lines += [f"- added `{k}`: {show(after[k])}" for k in added]
        lines += [f"- removed `{k}` (was {show(before[k])})" for k in removed]
        lines += [f"- changed `{k}`: {change(before[k], after[k])}" for k in changed]
        lines.append("")
    return lines


def main():
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--report", type=pathlib.Path, help="write a Markdown report here on drift")
    parser.add_argument("--committed", type=pathlib.Path, default=COMMITTED,
                        help="the recorded server_definitions answer the SDK is generated from")
    parser.add_argument("--network", action="append", metavar="NAME=URL",
                        help="check only these (repeatable); default: Devnet, Testnet, Mainnet")
    parser.add_argument("--offline", action="store_true",
                        help="only check that the codec's copy matches the recorded answer")
    args = parser.parse_args()

    committed = json.loads(args.committed.read_text())
    committed = committed.get("result", committed)
    if args.offline:
        codec = json.loads(CODEC.read_text())
        differ = [k for k, v in codec.items() if committed.get(k) != v]
        if differ:
            sys.exit(f"{CODEC.relative_to(ROOT)} differs from the recorded definitions in: "
                     f"{', '.join(differ)}")
        print(f"the codec's definitions ({', '.join(codec)}) are the recorded ones "
              f"({committed.get('hash', '?')[:16]}…)")
        return
    ours = committed.get("hash", "?")
    networks = NETWORKS
    if args.network:
        networks = {}
        for item in args.network:
            name, _, url = item.partition("=")
            networks.setdefault(name, []).append(url)

    differing, body, answered = [], [], 0
    for name, urls in networks.items():
        try:
            definitions, version, url = fetch(urls)
        except RuntimeError as error:
            print(f"{name}: no answer ({error})", file=sys.stderr)
            continue
        answered += 1
        theirs = definitions.get("hash", "?")
        if theirs == ours:
            print(f"{name}: rippled {version}, definitions {theirs[:16]}…: the SDK's")
            continue
        print(f"{name}: rippled {version}, definitions {theirs[:16]}…: DIFFERENT")
        differing.append((name, theirs))
        body += [f"## {name}", "",
                 f"`{url}` runs `rippled` {version}; its definitions hash is `{theirs}`.", ""]
        diff = compare(committed, definitions)
        body += diff or ["The hash differs, but no section does (a change this script does not "
                         "compare, such as ordering).", ""]

    if not answered:
        sys.exit("no network answered")
    if differing and args.report:
        names = ", ".join(f"{n} ({h[:8]})" for n, h in differing)
        report = [
            f"# rippled's definitions differ from the SDK's: {names}",
            "",
            f"The SDK is generated from definitions with hash `{ours}` "
            "(`crates/xrpl-client/tests/fixtures/rippled/server_definitions.json`). "
            "These networks' servers know something it does not, or no longer know something "
            "it does.",
            "",
            *body,
            "## Catching up",
            "",
            "1. Record the new `server_definitions` (from a server that has them) over the "
            "fixture, and the codec's subset into `crates/xrpl-core/src/codec/definitions.json`.",
            "2. Regenerate: `crates/xrpl-core/scripts/gen_transactions.py` and "
            "`gen_ledger_objects.py` (see their headers).",
            "3. Probe what changed against that `rippled` (codec edge cases, fees in "
            "`transaction::cost`, keylets for new entry types), and re-record the response "
            "fixtures (`crates/xrpl-client/scripts/record_fixtures.py`).",
            "4. Release it as a minor version: new types and fields are additions (the "
            "protocol types are `#[non_exhaustive]`).",
            "",
            "*Opened by the weekly definitions check (`.github/workflows/definitions.yml`).*",
        ]
        args.report.write_text("\n".join(report) + "\n")
        print(f"report: {args.report}")


if __name__ == "__main__":
    main()
