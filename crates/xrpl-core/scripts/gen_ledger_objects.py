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
def rust_ty(name):
    return RUST.get(ftype.get(name, ""), "Value")
names = sorted(k for k in fmts if k != "common")
used = sorted({rust_ty(f["name"]).replace("Vec<", "").rstrip(">") for n in names for f in fmts[n]} | {"AccountId", "Hash256"})
crate_types = [t for t in used if t[0].isupper() and t != "Value" and t != "String"]
out = ["// @generated from rippled 3.4.1's server_definitions (LEDGER_ENTRY_FORMATS) by",
       "// crates/xrpl-core/scripts/gen_ledger_objects.py: one struct per ledger entry type, its fields as",
       "// rippled lists them. Regenerate: python3 gen_ledger_objects.py <server_definitions.json> <this file>",
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
out.append("/// Every ledger entry type rippled 3.4.1 defines, by `LedgerEntryType`.")
out.append("pub const NAMES: &[&str] = &[" + ", ".join(f'"{n}"' for n in names) + "];")
out.append("")
out.append("macro_rules! each_entry_type { ($m:ident) => { $m!(" + ", ".join(names) + "); }; }")
out.append("pub(super) use each_entry_type;")
open(sys.argv[2], "w").write("\n".join(out) + "\n")
print(len(names), "entry types;", sum(len(fmts[n]) for n in names), "fields")
