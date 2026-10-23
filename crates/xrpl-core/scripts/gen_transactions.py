"""Generate crates/xrpl-core/src/transaction/models/formats.rs from rippled's
server_definitions: one struct per transaction type (TRANSACTION_FORMATS),
its flags (TRANSACTION_FLAGS, ACCOUNT_SET_FLAGS) as constants, and the list
the `Tx` enum is built from.

    python3 gen_transactions.py <server_definitions.json> <formats.rs>

server_definitions.json is a recorded `server_definitions` response: either
the bare result or {"result": …} (halcyon-xrpl-client's
tests/fixtures/rippled/server_definitions.json works as is).
"""
import json, os, re, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from field_docs import doc  # noqa: E402

raw = json.load(open(sys.argv[1]))
r = raw.get("result", raw)
ftype = {f[0]: f[1]["type"] for f in r["FIELDS"]}
formats = r["TRANSACTION_FORMATS"]
tx_flags = r["TRANSACTION_FLAGS"]
asf = r["ACCOUNT_SET_FLAGS"]
common = {f["name"] for f in formats["common"]}

# Field type → Rust type. Arrays and objects with a known shape get their own
# types (models/mod.rs); `serde(with)` modules turn rippled's wrapper objects
# ({"Memo": {…}}) into plain Vecs.
RUST = {"AccountID": "String", "Amount": "Amount", "UInt8": "u8", "UInt16": "u16",
        "UInt32": "u32", "UInt64": "String", "Int32": "i32", "Hash128": "Hash128",
        "Hash160": "Hash160", "Hash192": "Hash192", "Hash256": "Hash256", "Blob": "String",
        "Currency": "Currency", "Vector256": "Vec<Hash256>", "Number": "String",
        "Issue": "Asset", "XChainBridge": "XChainBridge", "PathSet": "Vec<Vec<PathStep>>",
        "STObject": "Value", "STArray": "Vec<Value>"}
# STArray fields with a known element type: field → (element type, wrapper key, with-module).
ARRAYS = {
    "SignerEntries": ("SignerEntry", "SignerEntry", "signer_entries"),
    "PriceDataSeries": ("PriceData", "PriceData", "price_data_series"),
    "AuthAccounts": ("AuthAccount", "AuthAccount", "auth_accounts"),
    "Permissions": ("Permission", "Permission", "permissions"),
    "AcceptedCredentials": ("CredentialSpec", "Credential", "credentials"),
    "AuthorizeCredentials": ("CredentialSpec", "Credential", "credentials"),
    "UnauthorizeCredentials": ("CredentialSpec", "Credential", "credentials"),
    "RawTransactions": ("Tx", "RawTransaction", "raw_transactions"),
    "BatchSigners": ("BatchSigner", "BatchSigner", "batch_signers"),
}
KEYWORDS = {"type", "match", "ref", "box", "move", "use", "mod", "self", "loop", "where", "fn", "in"}


def snake(n):
    n = re.sub(r"IDs$", "Ids", n)
    n = re.sub(r"(?<=[A-Z])ID$", "_ID", n)
    n = re.sub(r"(?<=[a-z0-9])(?=[A-Z])|(?<=[A-Z])(?=[A-Z][a-z])", "_", n)
    s = n.lower()
    return f"r#{s}" if s in KEYWORDS else s


def const_name(flag):
    # tfNoRippleDirect -> TF_NO_RIPPLE_DIRECT, tfMPTCanTransfer -> TF_MPT_CAN_TRANSFER,
    # tfLPToken -> TF_LP_TOKEN; "NFToken" stays one word (tfSellNFToken -> TF_SELL_NFTOKEN).
    flag = flag.replace("NFToken", "Nftoken")
    return re.sub(r"(?<=[a-z0-9])(?=[A-Z])|(?<=[A-Z])(?=[A-Z][a-z])", "_", flag).upper()


def doc_slug(name):  # xrpl.org's page name for a transaction type
    return name.lower()


def field(f, optional):
    return ["    " + doc(f["name"], ftype.get(f["name"]), not optional)] + _field(f, optional)


def _field(f, optional):
    name = f["name"]
    if name in ARRAYS:
        elem, _, module = ARRAYS[name]
        ty = f"Vec<{elem}>"
        with_ = module + ("_opt" if optional else "")
        if optional:
            return [f'    #[serde(rename = "{name}", default, skip_serializing_if = "Option::is_none", with = "{with_}")]',
                    f"    pub {snake(name)}: Option<{ty}>,"]
        return [f'    #[serde(rename = "{name}", with = "{with_}")]', f"    pub {snake(name)}: {ty},"]
    ty = RUST.get(ftype.get(name, ""), "Value")
    if optional:
        return [f'    #[serde(rename = "{name}", default, skip_serializing_if = "Option::is_none")]',
                f"    pub {snake(name)}: Option<{ty}>,"]
    return [f'    #[serde(rename = "{name}")]', f"    pub {snake(name)}: {ty},"]


def rust_ty(name):
    if name in ARRAYS:
        return f"Vec<{ARRAYS[name][0]}>"
    return RUST.get(ftype.get(name, ""), "Value")


# The types with preflight checks in models/validate.rs, and their function.
VALIDATED = {
    "AccountSet": "account_set", "AMMCreate": "amm_create", "AMMDeposit": "amm_deposit",
    "AMMWithdraw": "amm_withdraw", "AMMVote": "amm_vote", "AMMBid": "amm_bid",
    "Clawback": "clawback", "DelegateSet": "delegate_set", "SignerListSet": "signer_list_set",
    "TrustSet": "trust_set", "NFTokenAcceptOffer": "nf_token_accept_offer",
}

names = sorted(k for k in formats if k != "common")
out = [
    "// @generated from rippled 3.4.1's server_definitions (TRANSACTION_FORMATS, TRANSACTION_FLAGS,",
    "// ACCOUNT_SET_FLAGS) by crates/xrpl-core/scripts/gen_transactions.py. Do not edit; regenerate:",
    "//     python3 gen_transactions.py <server_definitions.json> <this file>",
    "// One struct per transaction type: the common fields in `common`, then the type's own fields as",
    "// rippled lists them. Required fields are plain, optional and default ones `Option`.",
    "",
    "#![allow(clippy::upper_case_acronyms, clippy::too_many_arguments)]",
    "",
    "use super::*;",
    "",
    "use crate::flags::flags_type;",
    "",
]
UNIVERSAL = [
    ("FULLY_CANONICAL_SIG", tx_flags["universal"]["tfFullyCanonicalSig"],
     "`tfFullyCanonicalSig` (any type): set on every transaction since the RequireFullyCanonicalSig amendment."),
    ("INNER_BATCH_TXN", tx_flags["universal"]["tfInnerBatchTxn"],
     "`tfInnerBatchTxn` (any type): marks a transaction inside a `Batch` (unsigned, zero fee)."),
]


def flags_block(type_name, doc, flags):
    """flags_type! for `flags` [(CONST, value, doc)] plus the universal ones."""
    lines = ["flags_type!(", f"    /// {doc}", f"    {type_name} {{"]
    for const, value, d in list(flags) + UNIVERSAL:
        lines.append(f"        /// {d}")
        lines.append(f"        {const} = 0x{value:08X},")
    lines += ["    }", ");", ""]
    return lines


out += flags_block("UniversalFlags", "The flags any transaction may carry (for the types with none of their own).", [])


def variant(flag):  # asfRequireDest -> RequireDest
    return flag[3:]


out += ["/// A value of [`AccountSet`]'s `SetFlag` / `ClearFlag` (`asf…`): one account setting to turn on",
        "/// or off. `u32::from(flag)` is its number.",
        "#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]",
        "#[non_exhaustive]",
        "pub enum AccountSetFlag {"]
for flag, value in sorted(asf.items(), key=lambda kv: kv[1]):
    out += [f"    /// `{flag}` ({value}).", f"    {variant(flag)} = {value},"]
out += ["}", "",
        "impl From<AccountSetFlag> for u32 {",
        "    fn from(flag: AccountSetFlag) -> u32 {",
        "        flag as u32",
        "    }",
        "}", "",
        "impl AccountSetFlag {",
        "    /// The flag with this number, if this version knows it.",
        "    pub fn from_value(value: u32) -> Option<Self> {",
        "        match value {"]
for flag, value in sorted(asf.items(), key=lambda kv: kv[1]):
    out.append(f"            {value} => Some(Self::{variant(flag)}),")
out += ["            _ => None,", "        }", "    }", "}", ""]
nfields = 0
for n in names:
    own = [f for f in formats[n] if f["name"] not in common]
    required = [f for f in own if f["optionality"] == 0]
    own_flags = sorted(tx_flags.get(n, {}).items(), key=lambda kv: kv[1])
    flags_ty = f"{n}Flags" if own_flags else "UniversalFlags"
    if own_flags:
        out += flags_block(f"{n}Flags", f"The flags of a [`{n}`] (`Flags`).",
                           [(const_name(f)[3:], v, f"`{f}`.") for f, v in own_flags])
    out.append(f"/// [`{n}`](https://xrpl.org/docs/references/protocol/transactions/types/{doc_slug(n)}) transaction.")
    out.append("///")
    req_names = ", ".join(f"`{f['name']}`" for f in required) or "none beyond the common fields"
    out.append(f"/// Required: {req_names}.")
    out.append("#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]")
    out.append("#[non_exhaustive]")
    out.append(f"pub struct {n} {{")
    out.append("    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).")
    out.append("    #[serde(flatten)]")
    out.append("    pub common: Common,")
    for f in own:
        lines = field(f, f["optionality"] != 0)
        if n == "Payment" and f["name"] == "Amount":
            lines[0] = ("    /// How much to send at most (`Amount`: XRP in drops, a token, or an MPT). Required.\n"
                        "    /// With `tfPartialPayment` the payment may deliver less: what arrived is the\n"
                        "    /// metadata's `delivered_amount` (`utils::delivered_amount`), never this field.")
        out.extend(lines)
        nfields += 1
    if n == "OfferCreate":
        out += ['    /// What the owner holds of `TakerGets`\' asset: `rippled` adds it in transaction',
                '    /// streams and in `ledger` with `owner_funds: true`. Not a transaction field: the',
                '    /// codec ignores it.',
                '    #[serde(rename = "owner_funds", default, skip_serializing_if = "Option::is_none")]',
                '    pub owner_funds: Option<String>,']
    if n == "Payment":
        out += ['    /// `rippled`\'s API name for `Amount` in what it returns (API v1 writes both; v2 only',
                '    /// this). Not a field of its own: the codec takes it as `Amount` (it must equal',
                '    /// `Amount` when both are present), and a transaction read without `Amount` takes',
                '    /// its `Amount` from here.',
                '    #[serde(rename = "DeliverMax", default, skip_serializing_if = "Option::is_none")]',
                '    pub deliver_max: Option<Amount>,']
    out.append("}")
    out.append("")
    out.append(f"impl {n} {{")
    out.append(f'    /// `"{n}"`.')
    out.append(f'    pub const TRANSACTION_TYPE: &\'static str = "{n}";')
    def param(f):
        t = rust_ty(f["name"])
        return f"{snake(f['name'])}: {'impl Into<String>' if t == 'String' else t}"
    params = ", ".join(param(f) for f in required)
    sep = ", " if params else ""
    out.append("")
    out.append(f"    /// A `{n}` from `account` with its required fields; everything else unset.")
    out.append(f"    pub fn new(account: impl Into<String>{sep}{params}) -> Self {{")
    out.append("        Self {")
    out.append("            common: Common::new(account),")
    for f in own:
        s = snake(f["name"])
        if f["optionality"] != 0:
            out.append(f"            {s}: None,")
        elif rust_ty(f["name"]) == "String":
            out.append(f"            {s}: {s}.into(),")
        else:
            out.append(f"            {s},")
    if n == "Payment":
        out.append("            deliver_max: None,")
    if n == "OfferCreate":
        out.append("            owner_funds: None,")
    out.append("        }")
    out.append("    }")
    for f in own:
        if f["optionality"] == 0:
            continue
        s_ = snake(f["name"])
        bare = s_.replace("r#", "")
        out.append("")
        out.append(f"    /// With `{f['name']}` set (see [`Self::{bare}`]).")
        out.append("    #[must_use]")
        out.append(f"    pub fn with_{bare}(mut self, value: impl Into<{rust_ty(f['name'])}>) -> Self {{")
        out.append(f"        self.{s_} = Some(value.into());")
        out.append("        self")
        out.append("    }")
    out.append("}")
    out.append("")
    out.append(f"impl TxModel for {n} {{")
    out.append(f'    const TRANSACTION_TYPE: &\'static str = "{n}";')
    out.append(f"    type Flags = {flags_ty};")
    if n in VALIDATED:
        out.append("    fn validate(&self) -> Result<(), CoreError> {")
        out.append(f"        super::validate::{VALIDATED[n]}(self)")
        out.append("    }")
    out.append("    fn common(&self) -> &Common {")
    out.append("        &self.common")
    out.append("    }")
    out.append("    fn common_mut(&mut self) -> &mut Common {")
    out.append("        &mut self.common")
    out.append("    }")
    out.append("}")
    out.append("")
    out.append(f"impl From<{n}> for Tx {{")
    out.append(f"    fn from(tx: {n}) -> Self {{")
    out.append(f"        Tx::{n}(tx)")
    out.append("    }")
    out.append("}")
    out.append("")
out.append("/// Every transaction type rippled 3.4.1 defines, by `TransactionType`.")
out.append("pub const NAMES: &[&str] = &[" + ", ".join(f'"{n}"' for n in names) + "];")
out.append("")
out.append("macro_rules! each_transaction_type { ($m:ident) => { $m!(" + ", ".join(names) + "); }; }")
out.append("pub(super) use each_transaction_type;")
open(sys.argv[2], "w").write("\n".join(out) + "\n")
print(len(names), "transaction types;", nfields, "type-specific fields;",
      sum(len(v) for k, v in tx_flags.items() if k != "universal") + len(asf), "flag constants")
