"""Generate crates/xrpl-core/src/types/ledger_object/formats.rs from rippled's
server_definitions (LEDGER_ENTRY_FORMATS, LEDGER_ENTRY_FLAGS): one struct per
ledger entry type, its flags type, and the list `LedgerObject` is built from.

    python3 gen_ledger_objects.py <server_definitions.json> <formats.rs>
    cargo fmt                            # the file is kept as rustfmt writes it

Each type and each flags type get a doc example (doc_examples.py has the sample
values); `cargo test --doc` runs them. server_definitions.json is a recorded
`server_definitions` response wrapped in {"result": …} (halcyon-xrpl-client's
tests/fixtures/rippled/server_definitions.json works as is).
"""
import json, os, re, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from field_docs import doc  # noqa: E402
import doc_examples as ex  # noqa: E402
r = json.load(open(sys.argv[1]))["result"]
ftype = {f[0]: f[1]["type"] for f in r["FIELDS"]}
fmts = r["LEDGER_ENTRY_FORMATS"]
common = [f["name"] for f in fmts.get("common", [])]
RUST = {"AccountID": "AccountId", "Amount": "Amount", "UInt8": "u8", "UInt16": "u16", "UInt32": "u32",
        "UInt64": "String", "Int32": "i32", "Hash128": "Hash128", "Hash160": "Hash160", "Hash192": "Hash192",
        "Hash256": "Hash256", "Blob": "String", "Currency": "Currency", "Vector256": "Vec<Hash256>",
        "Number": "String", "STObject": "Value", "STArray": "Vec<Value>"}
KEYWORDS = {"type", "match", "ref", "box", "move", "use", "mod", "self", "loop", "where", "fn", "in"}
def snake(n):
    n = re.sub(r"(?<=[A-Z])ID$", "_ID", n)
    n = re.sub(r"(?<=[a-z0-9])(?=[A-Z])|(?<=[A-Z])(?=[A-Z][a-z])", "_", n)
    s = n.lower()
    return f"r#{s}" if s in KEYWORDS else s

# Each ledger entry flag in LEDGER_ENTRY_FLAGS, described (rippled's names;
# the table key DirNode is the DirectoryNode entry).
FLAG_DOCS = {
    "lsfPasswordSpent": "The account has used its one free regular-key change (a SetRegularKey signed with the master key costs no fee until then).",
    "lsfRequireDestTag": "Payments to the account must carry a DestinationTag (`asfRequireDest`).",
    "lsfRequireAuth": "The account must authorize each trust line that holds its tokens (`asfRequireAuth`).",
    "lsfDisallowXRP": "The account asks not to be sent XRP (`asfDisallowXRP`; advisory, not enforced).",
    "lsfDisableMaster": "The master key cannot sign for the account (`asfDisableMaster`).",
    "lsfNoFreeze": "The account has given up freezing its tokens, for good (`asfNoFreeze`).",
    "lsfGlobalFreeze": "All tokens the account issues are frozen (`asfGlobalFreeze`).",
    "lsfDefaultRipple": "New trust lines to the account allow rippling by default (`asfDefaultRipple`).",
    "lsfDepositAuth": "Only preauthorized accounts may send the account funds (`asfDepositAuth`).",
    "lsfDisallowIncomingNFTokenOffer": "Others may not make NFT offers to the account (`asfDisallowIncomingNFTokenOffer`).",
    "lsfDisallowIncomingCheck": "Others may not create checks to the account (`asfDisallowIncomingCheck`).",
    "lsfDisallowIncomingPayChan": "Others may not open payment channels to the account (`asfDisallowIncomingPayChan`).",
    "lsfDisallowIncomingTrustline": "Others may not open trust lines to the account (`asfDisallowIncomingTrustline`).",
    "lsfAllowTrustLineLocking": "The account's tokens may be locked in escrow (`asfAllowTrustLineLocking`).",
    "lsfAllowTrustLineClawback": "The account may claw back the tokens it issues (`asfAllowTrustLineClawback`).",
    "lsfAccepted": "The subject has accepted the credential.",
    "lsfNFTokenBuyOffers": "The directory lists buy offers for an NFT.",
    "lsfNFTokenSellOffers": "The directory lists sell offers for an NFT.",
    "lsfLoanDefault": "The loan has defaulted.",
    "lsfLoanImpaired": "The loan is impaired.",
    "lsfLoanOverpayment": "The loan accepts overpayments.",
    "lsfMPTLocked": "Locked: the tokens cannot move.",
    "lsfMPTAuthorized": "The issuer has authorized the holder.",
    "lsfMPTAMM": "The holder is an AMM.",
    "lsfMPTCanLock": "The issuer may lock the tokens, for one holder or all.",
    "lsfMPTRequireAuth": "Holders must be authorized by the issuer.",
    "lsfMPTCanEscrow": "The tokens may be put in escrow.",
    "lsfMPTCanTrade": "The tokens may be traded on the DEX.",
    "lsfMPTCanTransfer": "The tokens may be sent between holders, not only to and from the issuer.",
    "lsfMPTCanClawback": "The issuer may claw the tokens back.",
    "lsfMPTCanHoldConfidentialBalance": "Holders may keep confidential balances of the token.",
    "lsfSellNFToken": "The offer sells the NFT (otherwise it is a buy offer).",
    "lsfPassive": "A passive offer: it does not consume offers that match it exactly.",
    "lsfSell": "A sell offer: it exchanges all of TakerGets, even for more than TakerPays.",
    "lsfHybrid": "A hybrid offer: it trades in a permissioned domain's book and the open one.",
    "lsfLowReserve": "The trust line counts toward the low account's owner reserve.",
    "lsfHighReserve": "The trust line counts toward the high account's owner reserve.",
    "lsfLowAuth": "The low account has authorized the high account to hold its tokens.",
    "lsfHighAuth": "The high account has authorized the low account to hold its tokens.",
    "lsfLowNoRipple": "The low account has disabled rippling through this line.",
    "lsfHighNoRipple": "The high account has disabled rippling through this line.",
    "lsfLowFreeze": "The low account has frozen the line.",
    "lsfHighFreeze": "The high account has frozen the line.",
    "lsfAMMNode": "The trust line belongs to an AMM.",
    "lsfLowDeepFreeze": "The low account has deep-frozen the line.",
    "lsfHighDeepFreeze": "The high account has deep-frozen the line.",
    "lsfOneOwnerCount": "The signer list counts as one entry toward the owner reserve.",
    "lsfSponsorshipRequireSignForFee": "The sponsor must sign each transaction whose fee it pays.",
    "lsfSponsorshipRequireSignForReserve": "The sponsor must sign each transaction whose reserve it covers.",
    "lsfVaultPrivate": "A private vault: only authorized accounts may deposit.",
}
FLAG_TABLE = {"DirectoryNode": "DirNode"}


def const_name(flag):
    # lsfDefaultRipple -> LSF_DEFAULT_RIPPLE, lsfMPTCanTransfer -> LSF_MPT_CAN_TRANSFER;
    # "NFToken" stays one word (lsfSellNFToken -> LSF_SELL_NFTOKEN), as for transaction flags.
    flag = flag.replace("NFToken", "Nftoken")
    return re.sub(r"(?<=[a-z0-9])(?=[A-Z])|(?<=[A-Z])(?=[A-Z][a-z])", "_", flag).upper()

def rust_ty(name):
    return RUST.get(ftype.get(name, ""), "Value")


# --- Doc examples ----------------------------------------------------------------------------
# Each entry's example reads it from the JSON rippled writes (its required fields, and for a
# type with none, a field that shows what it holds) and checks a field; a type with flags sets
# one in `Flags` and checks it. Sample values come from doc_examples by field; LEDGER_EXAMPLE
# gives a type other values, extra fields, and the field to check.

def usd(value, issuer):
    return {"currency": "USD", "issuer": issuer, "value": value}


XRP_ISSUE = {"currency": "XRP"}
BRIDGE = {"LockingChainDoor": ex.B, "LockingChainIssue": XRP_ISSUE,
          "IssuingChainDoor": ex.A, "IssuingChainIssue": XRP_ISSUE}
ARRAY_JSON = {
    "Permissions": [{"Permission": {"PermissionValue": "Payment"}}],
    "NFTokens": [{"NFToken": {"NFTokenID": ex.HASH["NFTokenID"], "URI": ex.BLOB["URI"]}}],
    "PriceDataSeries": [{"PriceData": {"BaseAsset": "XRP", "QuoteAsset": "USD",
                                       "AssetPrice": "61A8", "Scale": 4}}],
    "AcceptedCredentials": [{"Credential": {"Issuer": ex.B, "CredentialType": ex.BLOB["CredentialType"]}}],
    "SignerEntries": [{"SignerEntry": {"Account": ex.B, "SignerWeight": 1}},
                      {"SignerEntry": {"Account": ex.C, "SignerWeight": 1}}],
}
OWNERS = {"Account": ex.A, "Owner": ex.A, "Issuer": ex.A}
BOOKKEEPING = {"Flags", "OwnerNode", "PreviousTxnID", "PreviousTxnLgrSeq"}

# values: field values; extra: optional fields to add; flag: the flag `Flags` holds; check:
# (Rust expression, the Rust value it equals), and the types the check needs.
LEDGER_EXAMPLE = {
    "AMM": {"values": {"Account": ex.C, "LPTokenBalance": {"currency": ex.LP_CURRENCY, "issuer": ex.C,
                                                          "value": "1000"}},
            "check": ('entry.asset2["currency"]', '"USD"')},
    "AccountRoot": {"values": {"Balance": "1000000000"}, "flag": "DEFAULT_RIPPLE",
                    "check": ("entry.balance", "Amount::Xrp(1_000_000_000)")},
    "Amendments": {"extra": {"Amendments": [ex.HASH["Amendments"]]},
                   "check": ("entry.amendments.map(|ids| ids.len())", "Some(1)")},
    "Bridge": {"values": {"Account": ex.B, "SignatureReward": "100", "XChainClaimID": "0"},
               "check": ("entry.signature_reward", "Amount::Xrp(100)")},
    "Credential": {"values": {"Subject": ex.B}},
    "DID": {"extra": {"URI": ex.BLOB["URI"]},
            "check": ("entry.uri.as_deref()", f'Some("{ex.BLOB["URI"]}")')},
    "DepositPreauth": {"extra": {"Authorize": ex.B},
                       "check": ("entry.authorize.map(|id| id.to_string()).as_deref()", f'Some("{ex.B}")')},
    "DirectoryNode": {"values": {"Indexes": [ex.HASH["NFTokenBuyOffer"]]},
                      "extra": {"NFTokenID": ex.HASH["NFTokenID"]},
                      "check": ("entry.indexes.len()", "1")},
    "FeeSettings": {"extra": {"BaseFeeDrops": "10", "ReserveBaseDrops": "1000000",
                              "ReserveIncrementDrops": "200000"},
                    "check": ("entry.reserve_base_drops", "Some(Amount::Xrp(1_000_000))")},
    "Loan": {"flag": "LOAN_OVERPAYMENT"},
    "LoanBroker": {"values": {"Account": ex.C}, "check": ("entry.vault_id.to_string()", f'"{ex.HASH["VaultID"]}"')},
    "MPToken": {"values": {"Account": ex.B}, "flag": "MPT_AUTHORIZED", "extra": {"MPTAmount": "100"},
                "check": ("entry.mpt_amount.as_deref()", 'Some("100")')},
    "MPTokenIssuance": {"flag": "MPT_CAN_TRANSFER", "check": ("entry.outstanding_amount", '"1000"')},
    "NFTokenOffer": {"check": ("entry.amount", "Amount::Xrp(1_000_000)")},
    "NegativeUNL": {"extra": {"ValidatorToDisable": ex.D_PUBLIC_KEY},
                    "check": ("entry.validator_to_disable.as_deref()", f'Some("{ex.D_PUBLIC_KEY}")')},
    "Offer": {"values": {"TakerPays": "20000000", "TakerGets": usd("10", ex.B)},
              "check": ("entry.taker_pays", "Amount::Xrp(20_000_000)")},
    "Oracle": {"check": ("entry.last_update_time", "1_789_671_597")},
    "PayChannel": {"values": {"Amount": "10000000"}, "check": ("entry.settle_delay", "86400")},
    "PermissionedDomain": {"check": ("entry.accepted_credentials.len()", "1")},
    "RippleState": {"values": {"Balance": usd("10", ex.ACCOUNT_ONE), "LowLimit": usd("100", ex.A),
                               "HighLimit": usd("0", ex.B)},
                    "check": ("entry.low_limit.to_string()", f'"100 USD.{ex.A}"')},
    "SignerList": {"check": ("entry.signer_quorum", "2")},
    "Vault": {"values": {"Account": ex.C, "ShareMPTID": "000000018049717CC948789F32F267ADC2582484E3DFA698"},
              "flag": "VAULT_PRIVATE", "check": ('entry.asset["currency"]', '"XRP"')},
    "XChainOwnedCreateAccountClaimID": {"values": {"Account": ex.B}},
}
# Flags types whose first two flags do not go together: their example shows one.
SINGLE_FLAG = {"DirectoryNode", "Loan"}


def entry_value(n, name, others):
    """The JSON of field `name` of entry type `n`."""
    spec = LEDGER_EXAMPLE.get(n, {})
    if name in spec.get("values", {}):
        return spec["values"][name]
    t = ftype.get(name)
    if t == "AccountID":
        return OWNERS.get(name) or next(others)
    if t == "Amount":
        return "1000000"
    if name == "Flags":
        return 0
    if t in ("UInt8", "UInt16", "UInt32", "Int32"):
        return ex.NUMBER.get(name, 1)
    if t in ("UInt64", "Number"):
        return ex.STRING.get(name, "0" if t == "UInt64" else "1000")
    if t == "Hash256":
        return ex.HASH.get(name, ex.DEFAULT_HASH)
    if t == "Hash192":
        return ex.MPT_ID
    if t == "Blob":
        return ex.BLOB.get(name, ex.BLOB_PLACEHOLDER)
    if t == "Vector256":
        return [ex.HASH.get(name, ex.DEFAULT_HASH)]
    if t == "Issue":
        return XRP_ISSUE if name == "Asset" else {"currency": "USD", "issuer": ex.B}
    if t == "XChainBridge":
        return BRIDGE
    if t == "STArray":
        return ARRAY_JSON.get(name, [])
    raise SystemExit(f"{n}.{name}: no sample for {t}")


def compact(v):
    return json.dumps(v, ensure_ascii=False, separators=(", ", ": "))


# An entry's example is a top-level item's docs: `/// ` and this, within rustfmt's 100.
WIDTH = 96


def json_lines(key, v, indent, last):
    """`"key": value` in the json! macro, broken over lines where it does not fit."""
    head = f"{indent}{json.dumps(key)}: " if key is not None else indent
    end = "" if last else ","
    if len(head + compact(v) + end) <= WIDTH or not isinstance(v, (dict, list)) or not v:
        return [head + compact(v) + end]
    inner = indent + "    "
    if isinstance(v, dict):
        items = list(v.items())
        lines = [head + "{"]
        for i, (k, x) in enumerate(items):
            lines += json_lines(k, x, inner, i == len(items) - 1)
        return lines + [indent + "}" + end]
    lines = [head + "["]
    for i, x in enumerate(v):
        lines += json_lines(None, x, inner, i == len(v) - 1)
    return lines + [indent + "]" + end]


def check_for(name, value):
    """A check of field `name` holding `value`: (Rust expression, the Rust value it equals)."""
    t, f = ftype.get(name), snake(name)
    if t == "AccountID" or t in ("Hash128", "Hash160", "Hash192", "Hash256"):
        return f"entry.{f}.to_string()", json.dumps(value)
    if t in ("UInt8", "UInt16", "UInt32", "Int32"):
        return f"entry.{f}", f"{value:_}" if value >= 100_000 else str(value)
    if t in ("UInt64", "Number", "Blob"):
        return f"entry.{f}", json.dumps(value)
    if t == "Amount" and isinstance(value, str):
        return f"entry.{f}", f"Amount::Xrp({int(value):_})"
    if t in ("Vector256", "STArray"):
        return f"entry.{f}.len()", str(len(value))
    return None


def entry_example(n, fields, flags_ty, flags):
    """The doc block of ledger entry struct `n` (`fields` as the struct lists them)."""
    spec = LEDGER_EXAMPLE.get(n, {})
    others = iter([ex.B, ex.C] * 4)
    required = [f["name"] for f in fields if f["optionality"] == 0]
    own = [(name, entry_value(n, name, others)) for name in required if name not in BOOKKEEPING
           and not name.endswith("Node")]
    own += list(spec.get("extra", {}).items())
    flag = None
    if flags:
        flag = spec.get("flag", const_name(sorted(flags.items(), key=lambda kv: kv[1])[0][0])[4:])
        flag_value = next(v for k, v in flags.items() if const_name(k)[4:] == flag)
    book = [(name, entry_value(n, name, others)) for name in required if name not in dict(own)
            and name != "PreviousTxnID"]
    book = [(k, flag_value if k == "Flags" and flag else v) for k, v in book]
    items = [("LedgerEntryType", n)] + own
    lines = []
    for i, (k, v) in enumerate(items):
        lines += json_lines(k, v, "    ", False)
    # The bookkeeping fields, packed: they say where the entry is, not what it holds.
    row = ""
    for k, v in book:
        piece = f"{json.dumps(k)}: {compact(v)},"
        if row and len("    " + row + " " + piece) > WIDTH:
            lines.append("    " + row)
            row = piece
        else:
            row = f"{row} {piece}" if row else piece
    if row:
        lines.append("    " + row)
    if "PreviousTxnID" in required:
        lines.append(f'    "PreviousTxnID": "{ex.HASH["PreviousTxnID"]}",')
    lines[-1] = lines[-1].rstrip(",")

    # The check: a field that says what the entry holds (its owner only if nothing else does).
    checks = [(k, check_for(k, v)) for k, v in own]
    checks = [(k, c) for k, c in checks if c]
    check = spec.get("check") or next((c for k, c in checks if k not in OWNERS), checks[0][1])
    names = [n] + ([flags_ty] if flag else [])
    body = ["use serde_json::json;"]
    if flag:
        body.append("use xrpl_core::flags::Flags;")
    if "Amount::" in check[1]:
        body.append("use xrpl_core::types::Amount;")
    body += ex.use_line("types::ledger_object", names) + [""]
    body.append(f"let entry: {n} = serde_json::from_value(json!({{")
    body += lines + ["}))?;"]
    body += ex.assert_eq_lines(*check)
    if flag:
        body.append(f"assert!(entry.flags.contains({flags_ty}::{flag}));")
    body.append("# Ok::<(), Box<dyn std::error::Error>>(())")
    return ex.code_block(body)


def flags_doc(n, flags_ty, flags):
    consts = [(const_name(k)[4:], v) for k, v in sorted(flags.items(), key=lambda kv: kv[1])]
    pick = consts[:1] if n in SINGLE_FLAG else consts[:2]
    return ex.flags_example(flags_ty, "types::ledger_object", pick)
names = sorted(k for k in fmts if k != "common")
used = sorted({rust_ty(f["name"]).replace("Vec<", "").rstrip(">") for n in names for f in fmts[n]} | {"AccountId", "Hash256"})
crate_types = [t for t in used if t[0].isupper() and t != "Value" and t != "String"]
out = ["// @generated from rippled 3.4.1's server_definitions (LEDGER_ENTRY_FORMATS) by",
       "// crates/xrpl-core/scripts/gen_ledger_objects.py: one struct per ledger entry type, its fields as",
       "// rippled lists them, and each type's ledger entry flags (LEDGER_ENTRY_FLAGS) as a flags type.",
       "// Regenerate: python3 gen_ledger_objects.py <server_definitions.json> <this file>",
       "// Required fields are plain; optional and default ones (omitted from JSON at their default) are",
       "// Option. UInt64, Blob and Number stay the strings rippled writes; objects and arrays stay JSON.",
       "", "#![allow(clippy::upper_case_acronyms)]", "",
       "use crate::types::{" + ", ".join(crate_types) + "};",
       "use crate::flags::flags_type;",
       "use serde::{Deserialize, Serialize};", "use serde_json::Value;", ""]
for n in names:
    fields = [f for f in fmts[n] if f["name"] not in ("LedgerEntryType",)]
    seen = set(); fl = []
    for f in [{"name": c, "optionality": o} for c, o in [("Flags", 0), ("Sponsor", 1)]] + fields:
        if f["name"] in seen or f["name"] == "LedgerIndex": continue
        seen.add(f["name"]); fl.append(f)
    flags = r.get("LEDGER_ENTRY_FLAGS", {}).get(FLAG_TABLE.get(n, n), {})
    if flags:
        out.append("flags_type!(")
        out.append(f"    /// The flags of a [`{n}`] entry (`Flags`).")
        out += ["    " + line for line in flags_doc(n, f"{n}Flags", flags)]
        out.append(f"    {n}Flags {{")
        for flag, value in sorted(flags.items(), key=lambda kv: kv[1]):
            out.append(f"        /// `{flag}`: {FLAG_DOCS[flag]}")
            out.append(f"        {const_name(flag)[4:]} = 0x{value:08X},")
        out.append("    }")
        out.append(");")
        out.append("")
    out.append(f"/// `{n}` ledger entry.")
    out += entry_example(n, fl, f"{n}Flags", flags)
    out.append("#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]")
    out.append("#[non_exhaustive]")
    out.append(f"pub struct {n} {{")
    out.append("    /// The object's ID (rippled's `index`).")
    out.append("    #[serde(default, skip_serializing_if = \"Option::is_none\")]")
    out.append("    pub index: Option<Hash256>,")
    for f in fl:
        t = rust_ty(f["name"]); req = f["optionality"] == 0
        if f["name"] == "Flags" and flags:
            t = f"{n}Flags"
        out.append("    " + doc(f["name"], ftype.get(f["name"]), req))
        if req:
            out.append(f"    #[serde(rename = \"{f['name']}\")]")
            out.append(f"    pub {snake(f['name'])}: {t},")
        else:
            out.append(f"    #[serde(rename = \"{f['name']}\", default, skip_serializing_if = \"Option::is_none\")]")
            out.append(f"    pub {snake(f['name'])}: Option<{t}>,")
    out.append("}")
    out.append("")
out.append("/// Every ledger entry type rippled 3.4.1 defines, by `LedgerEntryType`.")
out += ex.names_example("types::ledger_object", len(names), "AccountRoot")
out.append("pub const NAMES: &[&str] = &[" + ", ".join(f'"{n}"' for n in names) + "];")
out.append("")
out.append("macro_rules! each_entry_type { ($m:ident) => { $m!(" + ", ".join(names) + "); }; }")
out.append("pub(super) use each_entry_type;")
open(sys.argv[2], "w").write("\n".join(out) + "\n")
print(len(names), "entry types;", sum(len(fmts[n]) for n in names), "fields")
