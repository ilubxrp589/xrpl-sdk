import json, os, re, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from field_docs import doc  # noqa: E402
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
names = sorted(k for k in fmts if k != "common")
used = sorted({rust_ty(f["name"]).replace("Vec<", "").rstrip(">") for n in names for f in fmts[n]} | {"AccountId", "Hash256"})
crate_types = [t for t in used if t[0].isupper() and t != "Value" and t != "String"]
out = ["// @generated from rippled 3.4.1's server_definitions (LEDGER_ENTRY_FORMATS) by",
       "// crates/xrpl-core/scripts/gen_ledger_objects.py: one struct per ledger entry type, its fields as",
       "// rippled lists them, and its ledger entry flags (LEDGER_ENTRY_FLAGS) as LSF_ constants.",
       "// Regenerate: python3 gen_ledger_objects.py <server_definitions.json> <this file>",
       "// Required fields are plain; optional and default ones (omitted from JSON at their default) are",
       "// Option. UInt64, Blob and Number stay the strings rippled writes; objects and arrays stay JSON.",
       "", "#![allow(clippy::upper_case_acronyms)]", "",
       "use crate::types::{" + ", ".join(crate_types) + "};",
       "use serde::{Deserialize, Serialize};", "use serde_json::Value;", ""]
for n in names:
    fields = [f for f in fmts[n] if f["name"] not in ("LedgerEntryType",)]
    seen = set(); fl = []
    for f in [{"name": c, "optionality": o} for c, o in [("Flags", 0), ("Sponsor", 1)]] + fields:
        if f["name"] in seen or f["name"] == "LedgerIndex": continue
        seen.add(f["name"]); fl.append(f)
    out.append(f"/// `{n}` ledger entry.")
    out.append("#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]")
    out.append("#[non_exhaustive]")
    out.append(f"pub struct {n} {{")
    out.append("    /// The object's ID (rippled's `index`).")
    out.append("    #[serde(default, skip_serializing_if = \"Option::is_none\")]")
    out.append("    pub index: Option<Hash256>,")
    for f in fl:
        t = rust_ty(f["name"]); req = f["optionality"] == 0
        out.append("    " + doc(f["name"], ftype.get(f["name"]), req))
        if req:
            out.append(f"    #[serde(rename = \"{f['name']}\")]")
            out.append(f"    pub {snake(f['name'])}: {t},")
        else:
            out.append(f"    #[serde(rename = \"{f['name']}\", default, skip_serializing_if = \"Option::is_none\")]")
            out.append(f"    pub {snake(f['name'])}: Option<{t}>,")
    out.append("}")
    out.append("")
    flags = r.get("LEDGER_ENTRY_FLAGS", {}).get(FLAG_TABLE.get(n, n), {})
    if flags:
        out.append(f"impl {n} {{")
        for flag, value in sorted(flags.items(), key=lambda kv: kv[1]):
            out.append(f"    /// `{flag}`: {FLAG_DOCS[flag]}")
            out.append(f"    pub const {const_name(flag)}: u32 = 0x{value:08X};")
        out.append("    /// Every flag `rippled` 3.4.1 defines for this entry: a bit outside it")
        out.append("    /// is one this version does not know.")
        out.append(f"    pub const ALL_FLAGS: u32 = 0x{sum(flags.values()):08X};")
        out.append("")
        out.append("    /// Whether `flag` (one of the `LSF_` constants above) is set.")
        out.append("    #[must_use]")
        out.append("    pub fn has_flag(&self, flag: u32) -> bool {")
        out.append("        self.flags & flag == flag")
        out.append("    }")
        out.append("}")
        out.append("")
out.append("/// Every ledger entry type rippled 3.4.1 defines, by `LedgerEntryType`.")
out.append("pub const NAMES: &[&str] = &[" + ", ".join(f'"{n}"' for n in names) + "];")
out.append("")
out.append("macro_rules! each_entry_type { ($m:ident) => { $m!(" + ", ".join(names) + "); }; }")
out.append("pub(super) use each_entry_type;")
open(sys.argv[2], "w").write("\n".join(out) + "\n")
print(len(names), "entry types;", sum(len(fmts[n]) for n in names), "fields")
