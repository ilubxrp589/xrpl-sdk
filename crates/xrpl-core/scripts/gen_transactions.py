"""Generate crates/xrpl-core/src/transaction/models/formats.rs from rippled's
server_definitions: one struct per transaction type (TRANSACTION_FORMATS),
its flags (TRANSACTION_FLAGS, ACCOUNT_SET_FLAGS) as constants, and the list
the `Tx` enum is built from.

    python3 gen_transactions.py <server_definitions.json> <formats.rs>
    cargo fmt                            # the file is kept as rustfmt writes it

Each type, its `new` and each flags type get a doc example (doc_examples.py
has the sample values); `cargo test --doc` runs them.

server_definitions.json is a recorded `server_definitions` response: either
the bare result or {"result": …} (halcyon-xrpl-client's
tests/fixtures/rippled/server_definitions.json works as is).
"""
import json, os, re, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from field_docs import doc  # noqa: E402
import doc_examples as ex  # noqa: E402

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
    "TrustSet": "trust_set", "NFTokenAcceptOffer": "nf_token_accept_offer", "Batch": "batch",
}


# --- Doc examples ----------------------------------------------------------------------------
# A struct's example builds it with `new` from its required fields, sets an optional field with
# its `with_…` setter, and checks the JSON `to_json` writes (and `validate()` for a type with
# checks); `new`'s example checks the fields it set. Sample values come from doc_examples, by
# field; TX_EXAMPLE picks the setter, and a value where a type's rules want another.

class Sample:
    """One field's sample: `expr` to pass (or the variable `bind` sets), what `to_json` writes
    for it (`json`: a path below the field and a Rust literal), and what the struct field then
    equals (`eq`, when it can be compared after the move)."""

    def __init__(self, expr, json=None, eq=None, bind=(), types=(), models=()):
        self.expr, self.json, self.eq, self.bind = expr, json, eq, list(bind)
        self.types, self.models = set(types), set(models)
        self.placeholder = False


def xrp(drops):
    e = f"Amount::xrp({drops:_})?"
    return Sample(e, ("", f'"{drops}"'), e, types=["Amount"])


def usd(value, issuer):
    e = f'Amount::token("{value}", "USD", "{issuer}")?'
    return Sample(e, ('["value"]', f'"{value}"'), e, types=["Amount"])


def mpt(value):
    e = f'Amount::mpt({value}, "{ex.MPT_ID}")?'
    return Sample(e, ('["value"]', f'"{value}"'), e, types=["Amount"])


def asset(currency_issuer=None):
    if currency_issuer is None:
        return Sample("Asset::xrp()", ('["currency"]', '"XRP"'), "Asset::xrp()", types=["Asset"])
    e = f'Asset::token("USD", "{currency_issuer}")?'
    return Sample(e, ('["currency"]', '"USD"'), e, types=["Asset"])


AMOUNTS = {
    "SignatureReward": lambda: xrp(100), "MinAccountCreateAmount": lambda: xrp(10_000_000),
    "BaseFeeDrops": lambda: xrp(10), "LimitAmount": lambda: usd("100", ex.B),
    "Amount2": lambda: usd("10", ex.B), "TakerGets": lambda: usd("10", ex.B),
    "TakerPays": lambda: xrp(20_000_000),
}
# Accounts with a role that fits one address: D, whose key is ex.D_PUBLIC_KEY.
ADDRESSES = {"AttestationSignerAccount": ex.D, "AttestationRewardAccount": ex.D, "RegularKey": ex.D}
# Variable names for bound values, where the field's own name reads badly.
BIND_NAMES = {"MPTokenIssuanceID": "issuance_id", "NFTokenID": "nft_id", "XChainBridge": "bridge",
              "NFTokenSellOffer": "sell_offer", "PriceDataSeries": "prices"}

TX_EXAMPLE = {
    "AMMBid": {"set": ["AuthAccounts"]},
    "AMMClawback": {"values": {"Asset": lambda: asset(ex.A), "Asset2": lambda: asset(),
                               "Amount": lambda: usd("10", ex.A)},
                    "set": ["Amount"]},
    "AMMCreate": {"values": {"Amount": lambda: xrp(10_000_000)}},
    "AMMDeposit": {"set": ["Amount"], "flags": "AMMDepositFlags::SINGLE_ASSET"},
    "AMMWithdraw": {"set": ["Amount"], "flags": "AMMWithdrawFlags::SINGLE_ASSET"},
    "AccountDelete": {"set": ["DestinationTag"]},
    "AccountSet": {"set": ["SetFlag"]},
    "CheckCash": {"set": ["Amount"]},
    "CheckCreate": {"set": ["Expiration"]},
    "Clawback": {"values": {"Amount": lambda: mpt(10)}, "set": ["Holder"]},
    "ConfidentialMPTConvert": {"set": ["HolderEncryptionKey"]},
    "ConfidentialMPTConvertBack": {"set": ["AuditorEncryptedAmount"]},
    "ConfidentialMPTSend": {"set": ["DestinationTag"]},
    "CredentialCreate": {"set": ["URI"]},
    "CredentialDelete": {"set": ["Subject"]},
    "DIDSet": {"set": ["URI"]},
    "DepositPreauth": {"set": ["Authorize"]},
    "EnableAmendment": {"account": ex.ACCOUNT_ZERO},
    "EscrowCreate": {"set": ["FinishAfter"]},
    "EscrowFinish": {"set": ["Condition", "Fulfillment"]},
    "LedgerStateFix": {"set": ["Owner"]},
    "LoanBrokerCoverClawback": {"set": ["LoanBrokerID"]},
    "LoanBrokerCoverWithdraw": {"set": ["Destination"]},
    "LoanBrokerSet": {"set": ["DebtMaximum"]},
    "LoanSet": {"set": ["Counterparty"]},
    "MPTokenAuthorize": {"set": ["Holder"]},
    "MPTokenIssuanceCreate": {"set": ["MaximumAmount"]},
    "MPTokenIssuanceSet": {"set": ["Holder"]},
    "NFTokenAcceptOffer": {"set": ["NFTokenSellOffer"]},
    "NFTokenBurn": {"set": ["Owner"]},
    "NFTokenCreateOffer": {"set": ["Destination"]},
    "NFTokenMint": {"set": ["URI"]},
    "NFTokenModify": {"set": ["URI"]},
    "OfferCreate": {"set": ["Expiration"]},
    "OracleSet": {"set": ["Provider"]},
    "Payment": {"set": ["DestinationTag"]},
    "PaymentChannelClaim": {"set": ["Balance"]},
    "PaymentChannelCreate": {"set": ["DestinationTag"]},
    "PaymentChannelFund": {"set": ["Expiration"]},
    "PermissionedDomainSet": {"set": ["DomainID"]},
    "SetFee": {"account": ex.ACCOUNT_ZERO, "set": ["BaseFeeDrops"]},
    "SetRegularKey": {"set": ["RegularKey"]},
    "SignerListSet": {"set": ["SignerEntries"]},
    "SponsorshipSet": {"set": ["Sponsee"]},
    "SponsorshipTransfer": {"set": ["Sponsee"]},
    "TrustSet": {"set": ["LimitAmount"]},
    "UNLModify": {"account": ex.ACCOUNT_ZERO},
    "VaultClawback": {"values": {"Amount": lambda: usd("10", ex.A)},
                      "set": ["Amount"]},
    "VaultCreate": {"set": ["AssetsMaximum"]},
    "VaultDelete": {"set": ["MemoData"]},
    "VaultSet": {"set": ["AssetsMaximum"]},
    "VaultWithdraw": {"set": ["Destination"]},
    "XChainAddClaimAttestation": {"set": ["Destination"]},
    "XChainClaim": {"set": ["DestinationTag"]},
    "XChainCommit": {"set": ["OtherChainDestination"]},
    "XChainCreateBridge": {"set": ["MinAccountCreateAmount"]},
    "XChainModifyBridge": {"set": ["SignatureReward"]},
}


INT_SUFFIX = {"UInt8": "u8", "UInt16": "u16", "UInt32": "u32", "Int32": "i32"}


def sample(n, name, addresses):
    """The sample for field `name` of type `n`, or None for a field no example sets."""
    t = ftype.get(name)
    special = TX_EXAMPLE.get(n, {}).get("values", {}).get(name)
    if special:
        return special()
    var = BIND_NAMES.get(name, snake(name).replace("r#", ""))
    if name == "SetFlag":
        value = asf["asfDefaultRipple"]
        return Sample("AccountSetFlag::DefaultRipple", ("", str(value)), None,
                      models=["AccountSetFlag"])
    if name in ARRAYS:
        elem, key, _ = ARRAYS[name]
        if name == "SignerEntries":
            bind = [f"let {var} = vec![", f'    SignerEntry::new("{ex.B}", 1),',
                    f'    SignerEntry::new("{ex.C}", 1),', "];"]
            return Sample(var, (f'[0]["{key}"]["Account"]', f'"{ex.B}"'), None, bind,
                          models=["SignerEntry"])
        if name == "PriceDataSeries":
            bind = ["// 2.5 USD per XRP: 25000 (hex 61A8) at scale 4.",
                    f'let {var} = vec![PriceData::new("XRP", "USD").with_price("61A8", 4)];']
            return Sample(var, (f'[0]["{key}"]["BaseAsset"]', '"XRP"'), None, bind,
                          models=["PriceData"])
        if name == "AuthAccounts":
            bind = [f'let {var} = vec![AuthAccount::new("{ex.C}")];']
            return Sample(var, (f'[0]["{key}"]["Account"]', f'"{ex.C}"'), None, bind,
                          models=["AuthAccount"])
        if name == "Permissions":
            bind = [f'let {var} = vec![Permission::new("Payment")];']
            return Sample(var, (f'[0]["{key}"]["PermissionValue"]', '"Payment"'), None, bind,
                          models=["Permission"])
        if elem == "CredentialSpec":
            bind = ex.let_lines(var, f'vec![CredentialSpec::new("{ex.B}", "{ex.BLOB["CredentialType"]}")]')
            return Sample(var, (f'[0]["{key}"]["Issuer"]', f'"{ex.B}"'), None, bind,
                          models=["CredentialSpec"])
        return None
    if t == "AccountID":
        a = ADDRESSES.get(name) or next(addresses)
        return Sample(f'"{a}"', ("", f'"{a}"'), f'"{a}"')
    if t == "Amount":
        return AMOUNTS.get(name, lambda: xrp(1_000_000))()
    if t in INT_SUFFIX:
        v = ex.NUMBER.get(name, 1)
        lit = f"{v:_}" if v >= 100_000 else str(v)
        return Sample(lit, ("", lit), lit)
    if t in ("UInt64", "Number"):
        lit = ex.rust_str(ex.STRING.get(name, "1"))
        return Sample(lit, ("", lit), lit)
    if t == "Blob":
        lit = ex.rust_str(ex.BLOB.get(name, ex.BLOB_PLACEHOLDER))
        s = Sample(lit, ("", lit), lit)
        s.placeholder = ex.is_crypto_blob(name)
        return s
    if t in ("Hash128", "Hash192", "Hash256"):
        value = {"Hash128": ex.EMAIL_HASH, "Hash192": ex.MPT_ID}.get(t, ex.HASH.get(name, ex.DEFAULT_HASH))
        return Sample(var, ("", f"{var}.to_string()"), var, ex.let_lines(var, f'{t}::from_hex("{value}")?'),
                      types=[t])
    if t == "Vector256":
        value = ex.HASH.get(name, ex.DEFAULT_HASH)
        one = {"NFTokenOffers": "offer", "CredentialIDs": "credential_id"}.get(name, var.rstrip("s"))
        return Sample(f"vec![{one}]", ("[0]", f"{one}.to_string()"), None,
                      ex.let_lines(one, f'Hash256::from_hex("{value}")?'), types=["Hash256"])
    if t == "Issue":
        return asset() if name == "Asset" else asset(ex.B)
    if t == "XChainBridge":
        bind = ex.call_lines(f"let {var} = XChainBridge::new",
                          [f'"{ex.B}"', "Asset::xrp()", f'"{ex.A}"', "Asset::xrp()"], [])
        return Sample(var, ('["LockingChainDoor"]', f'"{ex.B}"'), None, bind, types=["Asset"],
                      models=["XChainBridge"])
    return None


def setter_arg(name, s):
    """A sample as a `with_…` argument: an integer literal needs its type."""
    t = ftype.get(name)
    if t in INT_SUFFIX and name != "SetFlag":
        return s.expr + INT_SUFFIX[t]
    return s.expr


def use_lines(models, types, hidden):
    out = ex.use_line("transaction::models", models, hidden)
    if types:
        out += ex.use_line("types", types, hidden)
    return out


def tx_examples(n, own, flags_ty):
    """The doc blocks of struct `n` and of its `new`."""
    if n == "Batch":
        return batch_examples()
    spec = TX_EXAMPLE.get(n, {})
    account = spec.get("account", ex.A)
    addresses = iter([ex.B, ex.C, ex.D] * 4)
    required = [f for f in own if f["optionality"] == 0]
    optional = [f for f in own if f["optionality"] != 0]
    req = [(f["name"], sample(n, f["name"], addresses)) for f in required]
    if any(s is None for _, s in req):
        raise SystemExit(f"{n}: no sample for a required field")
    wanted = spec.get("set")
    if wanted is None:  # the first optional field with a sample
        wanted = [f["name"] for f in optional if sample(n, f["name"], iter([ex.B] * 4))][:1]
    opt = [(name, sample(n, name, addresses)) for name in wanted]
    used = [s for _, s in req + opt]
    models = {n, "TxModel"} | set().union(*(s.models for s in used))
    types = set().union(*(s.types for s in used))
    if "flags" in spec:
        models.add(spec["flags"].split("::")[0])

    args = [ex.rust_str(account)] + [s.expr for _, s in req]
    setters = [f".with_{snake(name).replace('r#', '')}({setter_arg(name, s)})" for name, s in opt]
    if "flags" in spec:
        setters.append(f".with_flags({spec['flags']})")
    binds = [line for _, s in req + opt for line in s.bind]
    note = (['// "0A0B0C0D" stands in for each proof, ciphertext or signature (hex).']
            if any(s.placeholder for s in used) else [])
    # The field checked in the JSON: the one the setter set, else a plain required one.
    check_name, check = next(((name, s) for name, s in opt + [r for r in req if r[1].eq] + req
                              if s.json), (None, None))

    body = use_lines(models, types, False) + [""] + binds + note
    body += ex.call_lines(f"let tx = {n}::new", args, setters)
    if n in VALIDATED:
        body.append("tx.validate()?; // the checks rippled makes before applying it")
    body.append("let json = tx.to_json()?;")
    body += ex.assert_eq_lines('json["TransactionType"]', f"{n}::TRANSACTION_TYPE", f'"{n}"')
    if check:
        body += ex.assert_eq_lines(f'json["{check_name}"]{check.json[0]}', check.json[1])
    else:
        body += ex.assert_eq_lines('json["Account"]', ex.rust_str(account))
    body.append("# Ok::<(), xrpl_core::CoreError>(())")
    struct_doc = ex.code_block(body)

    # `new`: the required fields as given, the optional ones unset.
    req_used = [s for _, s in req]
    new_models = {n} | set().union(*(s.models for s in req_used)) if req_used else {n}
    new_types = set().union(*(s.types for s in req_used)) if req_used else set()
    new_body = use_lines(new_models, new_types, True)
    new_body += [line for s in req_used for line in s.bind] + [
        line for line in note if any(s.placeholder for s in req_used)]
    new_body += ex.call_lines(f"let tx = {n}::new", args, [])
    field = next(((name, s) for name, s in req if s.eq), None)
    if field:
        new_body += ex.assert_eq_lines(f"tx.{snake(field[0])}", field[1].eq)
    else:
        new_body += ex.assert_eq_lines("tx.common.account", ex.rust_str(account))
    if optional:
        new_body.append(f"assert!(tx.{snake(optional[0]['name'])}.is_none()); // optional fields start unset")
    else:
        new_body.append("assert!(tx.common.fee.is_none()); // as are Fee, Sequence, …: autofill sets them")
    if any("?" in line for line in new_body):
        new_body.append("# Ok::<(), xrpl_core::CoreError>(())")
    return struct_doc, ex.code_block(new_body)


def batch_examples():
    a, b = ex.rust_str(ex.A), ex.rust_str(ex.B)
    struct_doc = ex.code_block([
        "use xrpl_core::transaction::models::{Batch, BatchFlags, Payment, PaymentFlags, TxModel};",
        "use xrpl_core::types::Amount;",
        "",
        "// Inner transactions carry tfInnerBatchTxn and no fee (see `transaction::batch`).",
        *ex.call_lines("let pay = Payment::new", [a, b, "Amount::xrp(1_000_000)?"],
                    [".with_flags(PaymentFlags::INNER_BATCH_TXN)", ".with_fee(0)"]),
        "let tx = Batch::new(",
        f"    {a},",
        "    vec![pay.clone().with_sequence(6).into(), pay.with_sequence(7).into()],",
        ")",
        ".with_flags(BatchFlags::ALL_OR_NOTHING);",
        "let json = tx.to_json()?;",
        'assert_eq!(json["TransactionType"], Batch::TRANSACTION_TYPE); // "Batch"',
        'assert_eq!(json["RawTransactions"][1]["RawTransaction"]["Sequence"], 7);',
        "# Ok::<(), xrpl_core::CoreError>(())",
    ])
    new_doc = ex.code_block([
        "# use xrpl_core::transaction::models::{Batch, OfferCancel};",
        f"let cancel = |sequence| OfferCancel::new({a}, sequence);",
        *ex.call_lines("let tx = Batch::new", [a, "vec![cancel(7).into(), cancel(8).into()]"], []),
        "assert_eq!(tx.raw_transactions.len(), 2);",
        "assert!(tx.batch_signers.is_none()); // optional fields start unset",
    ])
    return struct_doc, new_doc


# The flags each flags type's example combines: two that go together, else the first alone.
FLAG_PAIRS = {
    "AccountSet": ("REQUIRE_DEST_TAG", "REQUIRE_AUTH"),
    "MPTokenIssuanceCreate": ("MPT_CAN_TRADE", "MPT_CAN_TRANSFER"),
    "NFTokenMint": ("BURNABLE", "TRANSFERABLE"),
    "OfferCreate": ("PASSIVE", "SELL"),
    "Payment": ("NO_RIPPLE_DIRECT", "PARTIAL_PAYMENT"),
    "SponsorshipSet": ("SPONSORSHIP_SET_REQUIRE_SIGN_FOR_FEE", "SPONSORSHIP_SET_REQUIRE_SIGN_FOR_RESERVE"),
    "TrustSet": ("SET_NO_RIPPLE", "SET_FREEZE"),
    "VaultCreate": ("VAULT_PRIVATE", "VAULT_SHARE_NON_TRANSFERABLE"),
}


def flags_doc(type_name, n, consts):
    """The example of flags type `type_name` (of transaction type `n`), from its
    [(CONST, value)]."""
    values = dict(consts)
    pick = FLAG_PAIRS.get(n, (consts[0][0],))
    return ex.flags_example(type_name, "transaction::models", [(c, values[c]) for c in pick])


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


def flags_block(type_name, doc, flags, n=None):
    """flags_type! for `flags` [(CONST, value, doc)] plus the universal ones, with an example
    (of transaction type `n`'s flags)."""
    consts = [(c, v) for c, v, _ in list(flags) + UNIVERSAL]
    example = flags_doc(type_name, n, consts if flags else [("INNER_BATCH_TXN", consts[1][1])])
    lines = ["flags_type!(", f"    /// {doc}"] + [f"    {line}" for line in example] + [f"    {type_name} {{"]
    for const, value, d in list(flags) + UNIVERSAL:
        lines.append(f"        /// {d}")
        lines.append(f"        {const} = 0x{value:08X},")
    lines += ["    }", ");", ""]
    return lines


out += flags_block("UniversalFlags", "The flags any transaction may carry (for the types with none of their own).", [])


def variant(flag):  # asfRequireDest -> RequireDest
    return flag[3:]


ripple = asf["asfDefaultRipple"]
unknown = next(v for v in range(1000) if v not in asf.values())
out += ["/// A value of [`AccountSet`]'s `SetFlag` / `ClearFlag` (`asf…`): one account setting to turn on",
        "/// or off. `u32::from(flag)` is its number."]
out += ex.code_block(["use xrpl_core::transaction::models::{AccountSet, AccountSetFlag, TxModel};", "",
                      "let flag = AccountSetFlag::DefaultRipple;",
                      f"assert_eq!(u32::from(flag), {ripple});",
                      f'let tx = AccountSet::new("{ex.A}").with_clear_flag(flag);',
                      f'assert_eq!(tx.to_json()?["ClearFlag"], {ripple});',
                      "# Ok::<(), xrpl_core::CoreError>(())"])
out += ["#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]",
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
        "    /// The flag with this number, if this version knows it."]
out += ["    " + line for line in ex.code_block([
    "use xrpl_core::transaction::models::AccountSetFlag;", "",
    f"assert_eq!(AccountSetFlag::from_value({ripple}), Some(AccountSetFlag::DefaultRipple));",
    f"assert_eq!(AccountSetFlag::from_value({unknown}), None);"])]
out += ["    pub fn from_value(value: u32) -> Option<Self> {",
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
                           [(const_name(f)[3:], v, f"`{f}`.") for f, v in own_flags], n)
    struct_doc, new_doc = tx_examples(n, own, flags_ty)
    out.append(f"/// [`{n}`](https://xrpl.org/docs/references/protocol/transactions/types/{doc_slug(n)}) transaction.")
    out.append("///")
    req_names = ", ".join(f"`{f['name']}`" for f in required) or "none beyond the common fields"
    out.append(f"/// Required: {req_names}.")
    out += struct_doc
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
    out += ["    " + line for line in new_doc]
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
out += ex.names_example("transaction::models", len(names), "Payment")
out.append("pub const NAMES: &[&str] = &[" + ", ".join(f'"{n}"' for n in names) + "];")
out.append("")
out.append("macro_rules! each_transaction_type { ($m:ident) => { $m!(" + ", ".join(names) + "); }; }")
out.append("pub(super) use each_transaction_type;")
open(sys.argv[2], "w").write("\n".join(out) + "\n")
print(len(names), "transaction types;", nfields, "type-specific fields;",
      sum(len(v) for k, v in tx_flags.items() if k != "universal") + len(asf), "flag constants")
