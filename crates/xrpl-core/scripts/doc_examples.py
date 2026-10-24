"""Doc examples for the generated types (gen_transactions.py,
gen_ledger_objects.py): the sample values they use, and the code blocks built
from them. `cargo test --doc` compiles and runs every one, so a sample a type
refuses, or an assertion that does not hold, fails the build.

Addresses are the docs' public test values only: the genesis account, the two
test-key addresses (crypto/signing.rs) and one more public test address. IDs
are real-looking 256-bit values from the test fixtures; they name nothing an
example depends on."""

import json

A = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"  # the genesis account
B = "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe"
C = "rU6K7V3Po4snVhBBaU29sesqs2qTQJWDw1"  # sp5fghtJtpUorTwvof1NpDXAzNwf5's address
D = "rLUEXYuLiQptky37CqLcm9USQpPiz5rkpD"  # sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r's address
D_PUBLIC_KEY = "ED01FA53FA5A7E77798F882ECE20B1ABC00BB358A9E55A202D0D0676BD0CE37A63"  # D's key
ACCOUNT_ZERO = "rrrrrrrrrrrrrrrrrrrrrhoLvTp"  # the Account of pseudo-transactions
ACCOUNT_ONE = "rrrrrrrrrrrrrrrrrrrrBZbvji"  # the issuer of a trust line's Balance
MPT_ID = "000004C463C52827307480341125DA0577DEFC38405B0E3E"  # models/validate.rs's tests
LP_CURRENCY = "03930D02208264E2E40EC1B0C09E4DB96EE197B1"  # an AMM's LP token code (fixtures)

# 256-bit IDs by field, from the fixtures (tests/fixtures, crates/xrpl-client/tests/fixtures).
HASH = {
    "CheckID": "5500034F12C4319A1D93AC2684DED262F372D24F0BFF649C34BCAB8F73C3BF6B",
    "Channel": "5DB01B7FFED6B67E6B0414DED11E051D2EE2B7619CE0EAA6286D67A3A4D5BDB3",
    "NFTokenID": "00081388DC1AB4937C899037B2FDFC3CB20F6F64E73120BB5F8AA66A00000228",
    "NFTokenSellOffer": "D50092E5B3F02AC1145272FFB88F22C8D8CF82EE330E10EBFDC29E7A0635660C",
    "NFTokenBuyOffer": "D50092E5B3F02AC1145272FFB88F22C8D8CF82EE330E10EBFDC29E7A0635660C",
    "NFTokenOffers": "D50092E5B3F02AC1145272FFB88F22C8D8CF82EE330E10EBFDC29E7A0635660C",
    "Amendment": "42426C4D4F1009EE67080A9B7965B44656D7714D104A72F9B4369F97ABF044EE",
    "Amendments": "42426C4D4F1009EE67080A9B7965B44656D7714D104A72F9B4369F97ABF044EE",
    "BookDirectory": "5C8970D155D65DB8FF49B291D7EFFA4A09F9E8A68D9974B25A0E35FA931A0000",
    "PreviousTxnID": "ED3269BC0709DF99EBEE6B0320532389F655498356575A094ABB995A70431D99",
    "RootIndex": "00000299BD60630F6C37B91FB1FDA6979E59D19E9D42C2DBF39ABBEF66F6FBB8",
    "Indexes": "65A11EF47BFBDF73B6F8CDF7229DD6A80C45CC9DCC949E9FDADA5A59C6C8391A",
    "Hashes": "5A76D6976A3598E3DAD0B1B177F10C8032EAEEDA0A084F8C8B9419C1416D73D1",
    "VaultID": "1C09AD34A51B4B01A2C6675E748087AC85DBDD9247AC2D42D8CF3CA9F7AE8B3B",
    "LoanBrokerID": "30BF70FA03BF434831FF6BB9574F7A455A0513AC8DC6F01A10C9A3E77B58029F",
    "LoanID": "47B9C3D85AACF4761581C146FF4C2A60AAB9F3EFB6A70ED6DAAC99CD786810BA",
    "DomainID": "4B1F78FFAD8C0CB39B26F911E9A3C4AE9015A13FF133DB4D23CEB0596614760A",
    "CredentialIDs": "36EF950959ADC499F6F924717CB1344C5D65DE4A4BD4DBAD54B4B7D694A41DE8",
    "BlindingFactor": "218937A23DF0713D6BF560037ABF5CDBCF78F794921FE6747BED813C94ABD390",
    "ObjectID": "1EA05B0FC11101F7C500BD0DAC794A8BC746A7FBA6250B75489603EB820E0FF5",
    "InvoiceID": "4AFAF6A3BD112E8455FC9CEFDD3B5B4A06ABB6290AFC08308CC600AC406EB2B0",
}
DEFAULT_HASH = "0F3C11EC536374AFADC500109E4A2A89F57D8FC34B407C838DCD132189687421"
EMAIL_HASH = "98B4375E1D753E5B91627516F6D70977"

# Small numbers by field (ledger time = seconds since 2000-01-01).
NUMBER = {
    "DestinationTag": 12345, "OfferSequence": 7, "SettleDelay": 86400, "TicketCount": 5,
    "SignerQuorum": 2, "LedgerSequence": 106911489, "Expiration": 843322252,
    "CancelAfter": 843322252, "FinishAfter": 843321657, "LastUpdateTime": 1789671597,
    "OracleDocumentID": 1, "NFTokenTaxon": 0, "TradingFee": 500, "TransferFee": 5000,
    "LedgerFixType": 1, "UNLModifyDisabling": 1, "WasLockingChainSend": 1, "AssetScale": 2,
    "WithdrawalPolicy": 1, "TickSize": 5, "TransferRate": 1005000000, "Sequence": 5,
    "OwnerCount": 0, "PreviousTxnLgrSeq": 92436010, "TicketSequence": 7, "SignerListID": 0,
    "LoanSequence": 1, "StartDate": 843321657, "PaymentInterval": 2592000,
    "RemainingOwnerCountDelta": 1,
}

# Strings rippled writes for UInt64 and Number fields (UInt64 in hex; base 10 for MPT amounts).
STRING = {
    "MPTAmount": "1000", "MaximumAmount": "1000000", "OutstandingAmount": "1000",
    "XChainClaimID": "1", "XChainAccountCreateCount": "1", "XChainAccountClaimCount": "0",
    "PrincipalRequested": "10000", "AssetsMaximum": "1000000", "DebtMaximum": "1000000",
    "PeriodicPayment": "1000",
}


def text_hex(s):
    """`s` as the hex the ledger stores for a text field (upper case)."""
    return s.encode().hex().upper()


# Blob fields with a meaningful value; the others get BLOB_PLACEHOLDER.
BLOB = {
    "PublicKey": D_PUBLIC_KEY, "MessageKey": D_PUBLIC_KEY, "UNLModifyValidator": D_PUBLIC_KEY,
    "ValidatorToDisable": D_PUBLIC_KEY,
    "CredentialType": text_hex("KYC"),
    "URI": text_hex("https://example.com/item.json"),
    "Domain": text_hex("example.com"),
    "Condition": "A02580202029C15F227CB9472684A3A1F97FB887893E5B8F2D47319274A0DC0C4C90550C810120",
    "Fulfillment": "A0228020DABC3482FAE7764C0FACE68EDE5E45B8451176FC813AE745750F183051A58E76",
    "MPTokenMetadata": text_hex('{"ticker":"TST","name":"Test token"}'),
    "Provider": text_hex("provider"),
    "AssetClass": text_hex("currency"),
    "Data": text_hex("example"),
    "DIDDocument": text_hex("{}"),
    "MemoData": text_hex("closing"),
}
# Ciphertexts, commitments, proofs and signatures come from cryptography an
# example cannot show; it passes this short stand-in, and says so.
BLOB_PLACEHOLDER = "0A0B0C0D"


def is_crypto_blob(name):
    return name not in BLOB


def hex_value(v):
    """`0x0003_0000`: a flags value as the docs write it."""
    return f"0x{v >> 16:04X}_{v & 0xFFFF:04X}"


# Code lines are kept to this width: with `    /// ` in front (an item in an impl), within
# rustfmt's 100.
WIDTH = 92


def call_lines(head, args, setters, end=";"):
    """`head(args)` and its `.with_…(…)` setters, laid out as rustfmt lays out code."""
    flat = f"{head}({', '.join(args)})"
    if len(flat + "".join(setters) + end) <= WIDTH:
        return [flat + "".join(setters) + end]

    def setter(s, indent):
        if len(indent + s + end) <= WIDTH:
            return [indent + s]
        name, arg = s.split("(", 1)
        return [f"{indent}{name}(", f"{indent}    {arg[:-1]},", f"{indent})"]

    if len(flat) <= WIDTH:
        lines = [flat] + [line for s in setters for line in setter(s, "    ")]
    else:
        lines = [f"{head}("] + [f"    {a}," for a in args] + [")"]
        lines += [line for s in setters for line in setter(s, "")]
    lines[-1] += end
    return lines


def let_lines(name, expr):
    """`let name = expr;`, broken after `=` when it does not fit."""
    line = f"let {name} = {expr};"
    return [line] if len(line) <= WIDTH else [f"let {name} =", f"    {expr};"]


def assert_eq_lines(left, right, comment=None):
    """`assert_eq!(left, right);` (and `// comment` when it fits), as rustfmt breaks it."""
    line = f"assert_eq!({left}, {right});"
    if comment and len(f"{line} // {comment}") <= WIDTH:
        return [f"{line} // {comment}"]
    if len(line) <= WIDTH:
        return [line]
    return ["assert_eq!(", f"    {left},", f"    {right}", ");"]


def use_line(module, names, hidden=False):
    """`use xrpl_core::{module}::{names};` (a hidden line when `hidden`)."""
    names = sorted(names)
    path = f"use xrpl_core::{module}::"
    line = path + (names[0] if len(names) == 1 else "{" + ", ".join(names) + "}") + ";"
    if hidden:
        return ["# " + line]
    if len(line) <= WIDTH:
        return [line]
    return [path + "{", "    " + ", ".join(names) + ",", "};"]


def code_block(lines):
    """Doc comment lines (`/// …`) for a doctest of `lines` (`# ` lines hidden)."""
    out = ["///", "/// ```"]
    out += [f"/// {line}".rstrip() for line in lines]
    out.append("/// ```")
    return out


def flags_example(type_name, module, consts):
    """The example of a flags type: `consts` is [(CONST, value)], one or two
    flags that go together."""
    lines = ["use xrpl_core::flags::Flags;"] + use_line(module, [type_name]) + [""]
    if len(consts) == 2:
        (c1, v1), (c2, v2) = consts
        first, second = f"{type_name}::{c1}", f"{type_name}::{c2}"
        if len(f"let flags = {first} | {second};") <= WIDTH:
            lines.append(f"let flags = {first} | {second};")
        elif len(f"    {first} | {second};") <= WIDTH:
            lines += ["let flags =", f"    {first} | {second};"]
        else:
            lines += [f"let flags = {first}", f"    | {second};"]
        lines += [f"assert!(flags.contains({second}));",
                  f"assert_eq!(u32::from(flags), {hex_value(v1 | v2)});"]
    else:
        (c1, v1), = consts
        lines += [f"let flags = {type_name}::{c1};",
                  f"assert!(flags.contains({type_name}::{c1}));",
                  f"assert!(!{type_name}::empty().contains(flags));",
                  f"assert_eq!(u32::from(flags), {hex_value(v1)});"]
    return code_block(lines)


def names_example(module, count, sample):
    return code_block([f"use xrpl_core::{module}::NAMES;", "",
                       f"assert_eq!(NAMES.len(), {count});",
                       f"assert!(NAMES.contains(&\"{sample}\"));"])


def rust_str(s):
    return json.dumps(s)
