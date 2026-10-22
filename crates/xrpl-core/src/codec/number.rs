//! `STNumber` (type 9), the decimal type of vaults and loans, as `rippled`
//! 3.4.1 reads, writes and prints it (`STNumber.cpp`, `Number.cpp`).
//!
//! On the wire a Number is a signed 64-bit mantissa and a signed 32-bit
//! exponent, both big-endian. `rippled` keeps the mantissa normalized to 19
//! digits (10^18 to 9,999,999,999,999,999,999) and the exponent within
//! ±32,768; a mantissa above `i64::MAX` goes on the wire as a tenth of itself
//! with the exponent one higher. Zero is mantissa 0, exponent `i32::MIN`.
//!
//! Reading bytes normalizes them as `rippled` does, so bytes that are not in
//! normal form read as the value `rippled` would show (and re-encode to the
//! normal form). Reading JSON takes only what `rippled` takes: a decimal
//! string it can hold exactly, never rounded, or a 32-bit JSON integer.

use crate::CoreError;
use serde_json::Value;

const MIN_MANTISSA: u64 = 1_000_000_000_000_000_000;
const MAX_MANTISSA: u64 = 9_999_999_999_999_999_999;
/// The largest mantissa the wire holds as is (`i64::MAX`).
const MAX_REP: u64 = i64::MAX as u64;
/// `i64::MAX` rounded up to a multiple of ten (`kMaxRepUp`): at the top
/// exponent, a mantissa above it overflows.
const MAX_REP_UP: u64 = 9_223_372_036_854_775_810;
const MIN_EXPONENT: i64 = -32_768;
const MAX_EXPONENT: i64 = 32_768;
/// The exponent of zero on the wire.
const ZERO_EXPONENT: i32 = i32::MIN;

/// A nonzero Number in `rippled`'s internal normal form.
#[derive(Debug, PartialEq)]
struct Normal {
    negative: bool,
    mantissa: u64,
    exponent: i64,
}

fn unrepresentable(s: &str, why: &str) -> CoreError {
    CoreError::InvalidAmount(format!("Number {s:?} {why}"))
}

/// The 12 bytes for a Number given as `rippled` JSON.
pub(crate) fn encode(value: &Value) -> Result<[u8; 12], CoreError> {
    let normal = match value {
        Value::String(s) => {
            let (negative, mantissa, exponent) = parse(s)?;
            exact(negative, mantissa, exponent).map_err(|why| unrepresentable(s, why))?
        }
        // rippled's JSON integers are 32-bit; it refuses larger ones.
        Value::Number(n) => match (n.as_i64(), n.as_u64()) {
            (Some(i), _) if i32::try_from(i).is_ok() => exact(i < 0, i.unsigned_abs(), 0)
                .map_err(|why| unrepresentable(&i.to_string(), why))?,
            (_, Some(u)) if u32::try_from(u).is_ok() => {
                exact(false, u, 0).map_err(|why| unrepresentable(&u.to_string(), why))?
            }
            _ => {
                return Err(CoreError::InvalidAmount(format!(
                    "Number {n}: rippled takes only 32-bit integers in JSON; write it as a string"
                )))
            }
        },
        other => {
            return Err(CoreError::CodecError(format!(
                "a Number must be a string or an integer, not {other}"
            )))
        }
    };
    let (mantissa, exponent) = match normal {
        None => (0i64, ZERO_EXPONENT),
        Some(n) => {
            let (m, e) = if n.mantissa > MAX_REP {
                (n.mantissa / 10, n.exponent + 1)
            } else {
                (n.mantissa, n.exponent)
            };
            // m <= MAX_REP and |e| <= 32,769, so both conversions hold.
            let m = i64::try_from(m).map_err(|_| unrepresentable(&m.to_string(), "overflows"))?;
            let e = i32::try_from(e).map_err(|_| unrepresentable(&e.to_string(), "overflows"))?;
            (if n.negative { -m } else { m }, e)
        }
    };
    let [m0, m1, m2, m3, m4, m5, m6, m7] = mantissa.to_be_bytes();
    let [e0, e1, e2, e3] = exponent.to_be_bytes();
    Ok([m0, m1, m2, m3, m4, m5, m6, m7, e0, e1, e2, e3])
}

/// The JSON string `rippled` shows for a Number's 12 bytes.
pub(crate) fn decode(bytes: [u8; 12]) -> Result<String, CoreError> {
    let [m0, m1, m2, m3, m4, m5, m6, m7, e0, e1, e2, e3] = bytes;
    let mantissa = i64::from_be_bytes([m0, m1, m2, m3, m4, m5, m6, m7]);
    let exponent = i32::from_be_bytes([e0, e1, e2, e3]);
    if mantissa == 0 {
        return Ok("0".into());
    }
    // rippled 3.4.1 reads i64::MIN as -i64::MAX (seen with `simulate`).
    let mut m = if mantissa == i64::MIN {
        MAX_REP
    } else {
        mantissa.unsigned_abs()
    };
    let mut e = i64::from(exponent);
    while m < MIN_MANTISSA && e > MIN_EXPONENT {
        m *= 10;
        e -= 1;
    }
    if m < MIN_MANTISSA || e < MIN_EXPONENT {
        return Ok("0".into());
    }
    if e > MAX_EXPONENT || (m > MAX_REP_UP && e >= MAX_EXPONENT) {
        return Err(CoreError::CodecError(format!(
            "Number {mantissa}e{exponent} overflows; rippled refuses it"
        )));
    }
    Ok(render(&Normal {
        negative: mantissa < 0,
        mantissa: m,
        exponent: e,
    }))
}

/// `rippled`'s `partsFromString`: sign, mantissa and exponent of a decimal
/// string matching `^[-+]?(0|[1-9][0-9]*)(\.[0-9]+)?([eE][-+]?[0-9]+)?$`,
/// with the digits fitting a `u64` and the written exponent an `i32`.
pub(crate) fn parse(s: &str) -> Result<(bool, u64, i64), CoreError> {
    let not_read = |why: &str| CoreError::InvalidAmount(format!("{s:?} {why}"));
    let bad = || not_read("is not a number rippled reads");
    let (negative, rest) = match (s.strip_prefix('-'), s.strip_prefix('+')) {
        (Some(rest), _) => (true, rest),
        (_, Some(rest)) => (false, rest),
        _ => (false, s),
    };
    let (body, exp) = match rest.split_once(['e', 'E']) {
        Some((body, exp)) => (body, Some(exp)),
        None => (rest, None),
    };
    let (int, frac) = match body.split_once('.') {
        Some((i, f)) => (i, Some(f)),
        None => (body, None),
    };
    let digits = |d: &str| !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit());
    if !digits(int) || (int.len() > 1 && int.starts_with('0')) || frac.is_some_and(|f| !digits(f)) {
        return Err(bad());
    }
    let mut mantissa: u64 = 0;
    for b in int.bytes().chain(frac.unwrap_or("").bytes()) {
        mantissa = mantissa
            .checked_mul(10)
            .and_then(|m| m.checked_add(u64::from(b - b'0')))
            .ok_or_else(|| not_read("has more digits than rippled reads"))?;
    }
    let mut exponent = -(frac.map_or(0, str::len) as i64);
    if let Some(exp) = exp {
        let (sign, n) = match (exp.strip_prefix('-'), exp.strip_prefix('+')) {
            (Some(n), _) => (-1, n),
            (_, Some(n)) => (1, n),
            _ => (1, exp),
        };
        if !digits(n) {
            return Err(bad());
        }
        let n: i32 = n
            .parse()
            .map_err(|_| not_read("has an exponent rippled does not read"))?;
        exponent += sign * i64::from(n);
    }
    Ok((negative, mantissa, exponent))
}

/// `rippled`'s normal form of `mantissa`e`exponent`, or why it has none that
/// is exactly the same value (`numberFromJson` refuses any it would round).
fn exact(negative: bool, mantissa: u64, exponent: i64) -> Result<Option<Normal>, &'static str> {
    if mantissa == 0 {
        return Ok(None);
    }
    let (mut m, mut e) = (mantissa, exponent);
    while m < MIN_MANTISSA && e > MIN_EXPONENT {
        m *= 10;
        e -= 1;
    }
    while m > MAX_MANTISSA {
        if m % 10 != 0 {
            return Err("has more significant digits than rippled holds");
        }
        if e >= MAX_EXPONENT {
            return Err("overflows");
        }
        m /= 10;
        e += 1;
    }
    if m < MIN_MANTISSA || e < MIN_EXPONENT {
        return Err("is too small: rippled would round it to zero");
    }
    // Above i64::MAX the wire holds a tenth of the mantissa: it must end in 0,
    // and rippled refuses it at the top exponent.
    if m > MAX_REP && m % 10 != 0 {
        return Err("has more significant digits than rippled holds");
    }
    if e > MAX_EXPONENT || (m > MAX_REP_UP && e >= MAX_EXPONENT) {
        return Err("overflows");
    }
    Ok(Some(Normal {
        negative,
        mantissa: m,
        exponent: e,
    }))
}

/// `rippled`'s `to_string(Number)`: plain decimals for exponents from -28 to
/// -8 (and 0), otherwise `<mantissa>e<exponent>` without trailing zeros.
fn render(n: &Normal) -> String {
    let sign = if n.negative { "-" } else { "" };
    let (mut m, mut e) = (n.mantissa, n.exponent);
    if e != 0 && !(-28..=-8).contains(&e) {
        while m % 10 == 0 && e < MAX_EXPONENT {
            m /= 10;
            e += 1;
        }
        return if e == 0 {
            format!("{sign}{m}")
        } else {
            format!("{sign}{m}e{e}")
        };
    }
    let digits = m.to_string();
    // Digits before the decimal point; at most 19, at least -9.
    let point = digits.len() as i64 + e;
    let (int, frac) = match usize::try_from(point)
        .ok()
        .and_then(|p| digits.split_at_checked(p))
    {
        Some((i, f)) if !i.is_empty() => (i.to_string(), f.to_string()),
        _ => (
            "0".to_string(),
            "0".repeat(point.unsigned_abs() as usize) + &digits,
        ),
    };
    match frac.trim_end_matches('0') {
        "" => format!("{sign}{int}"),
        f => format!("{sign}{int}.{f}"),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;
    use serde_json::json;

    fn wire(m: i64, e: i32) -> [u8; 12] {
        let mut b = [0u8; 12];
        b[..8].copy_from_slice(&m.to_be_bytes());
        b[8..].copy_from_slice(&e.to_be_bytes());
        b
    }

    // Each row was put to rippled 3.4.1 (mainnet node, `simulate` of a
    // VaultCreate whose AssetsMaximum held these bytes); the string is the
    // `tx_json` it answered, None where it refused the blob.
    const READ_BY_RIPPLED: &[(i64, i32, Option<&str>)] = &[
        (13830756765796644, 1, Some("13830756765796644e1")),
        (1383075676579664400, -1, Some("13830756765796644e1")),
        (138307567657966440, 0, Some("13830756765796644e1")),
        (15, -1, Some("1.5")),
        (-15, -1, Some("-1.5")),
        (0, 5, Some("0")),
        (0, i32::MIN, Some("0")),
        (1, 0, Some("1")),
        (123, -10, Some("0.0000000123")),
        (15, -20, Some("15e-20")),
        (922337203685477580, 1, Some("9223372036854775800")),
        (922337203685477581, 1, Some("9223372036854775810")),
        (999999999999999999, 0, Some("999999999999999999")),
        (-999999999999999999, 3, Some("-999999999999999999e3")),
        (i64::MAX, 0, Some("9223372036854775807")),
        (i64::MAX, 1, Some("9223372036854775807e1")),
        (-i64::MAX, 0, Some("-9223372036854775807")),
        (i64::MIN, 0, Some("-9223372036854775807")),
        (i64::MIN, 5, Some("-9223372036854775807e5")),
        (i64::MIN, -20, Some("-0.09223372036854775807")),
        (1, -40000, Some("0")),
        (1, -32768, Some("0")),
        (1, -32750, Some("1e-32750")),
        (1, -32749, Some("1e-32749")),
        (5, -32787, Some("0")),
        (1000000000000000000, -32768, Some("1e-32750")),
        (1000000000000000000, -32769, Some("0")),
        (1000000000000000000, -32800, Some("0")),
        (1000000000000000000, 32760, Some("10000000000e32768")),
        (
            1000000000000000000,
            32768,
            Some("1000000000000000000e32768"),
        ),
        (100000000000000000, 32769, Some("1000000000000000000e32768")),
        (1000000000000000000, 32769, None),
        (922337203685477581, 32768, Some("922337203685477581e32768")),
        (922337203685477581, 32769, Some("9223372036854775810e32768")),
        (932337203618368716, 32768, Some("932337203618368716e32768")),
        (932337203618368716, 32769, None), // what the fuzzer found
        (1, 40000, None),
    ];

    #[test]
    fn reads_bytes_as_rippled_does() {
        for &(m, e, want) in READ_BY_RIPPLED {
            let got = decode(wire(m, e)).ok();
            assert_eq!(got.as_deref(), want, "{m}e{e}");
        }
    }

    // Each row was put to rippled 3.4.1 (`simulate` with `binary: true` of a
    // VaultCreate whose AssetsMaximum was this JSON): the bytes it wrote and
    // the `tx_json` it showed.
    const WRITTEN_BY_RIPPLED: &[(&str, i64, i32, &str)] = &[
        ("1.5", 1500000000000000000, -18, "1.5"),
        (
            "138307567657966440",
            1383075676579664400,
            -1,
            "13830756765796644e1",
        ),
        ("0", 0, i32::MIN, "0"),
        ("-0", 0, i32::MIN, "0"),
        ("-0.0", 0, i32::MIN, "0"),
        ("0e5", 0, i32::MIN, "0"),
        ("+5", 5000000000000000000, -18, "5"),
        ("1E5", 1000000000000000000, -13, "100000"),
        ("0.0000000123", 1230000000000000000, -26, "0.0000000123"),
        ("1.23e-30", 1230000000000000000, -48, "123e-32"),
        ("0.00000000000000000001", 1000000000000000000, -38, "1e-20"),
        (
            "0.0000000000000000000000000000000000001",
            1000000000000000000,
            -55,
            "1e-37",
        ),
        ("9223372036854775807", i64::MAX, 0, "9223372036854775807"),
        (
            "9223372036854775810",
            922337203685477581,
            1,
            "9223372036854775810",
        ),
        (
            "9300000000000000000",
            930000000000000000,
            1,
            "9300000000000000000",
        ),
        (
            "922337203685477580.7",
            i64::MAX,
            -1,
            "9223372036854775807e-1",
        ),
        ("10000000000000000000", 1000000000000000000, 1, "1e19"),
        ("1e32768", 1000000000000000000, 32750, "1e32768"),
        (
            "93e32784",
            930000000000000000,
            32768,
            "930000000000000000e32768",
        ),
        ("1e-32750", 1000000000000000000, -32768, "1e-32750"),
        (
            "9223372036854775810e32767",
            922337203685477581,
            32768,
            "922337203685477581e32768",
        ),
        (
            "9223372036854775810e32768",
            922337203685477581,
            32769,
            "9223372036854775810e32768",
        ),
        (
            "932337203618368716e32768",
            932337203618368716,
            32768,
            "932337203618368716e32768",
        ),
    ];

    #[test]
    fn writes_json_as_rippled_does() {
        for &(s, m, e, shown) in WRITTEN_BY_RIPPLED {
            let bytes = encode(&json!(s)).unwrap_or_else(|err| panic!("{s}: {err}"));
            assert_eq!(bytes, wire(m, e), "{s}");
            assert_eq!(decode(bytes).unwrap(), shown, "{s}");
            // What rippled shows encodes to the same bytes.
            assert_eq!(encode(&json!(shown)).unwrap(), bytes, "{s}");
        }
        for (n, shown) in [
            (json!(5), "5"),
            (json!(-5), "-5"),
            (json!(4294967295u32), "4294967295"),
            (json!(-2147483648i64), "-2147483648"),
        ] {
            assert_eq!(decode(encode(&n).unwrap()).unwrap(), shown);
        }
    }

    #[test]
    fn refuses_what_rippled_refuses() {
        // rippled 3.4.1 answered "Field 'tx_json.AssetsMaximum' has invalid
        // data." to each (and HTTP 400 to the integers beyond 32 bits).
        for s in [
            "9999999999999999999",
            "9223372036854775808",
            "9223372036854775815",
            "-9223372036854775808",
            "18446744073709551615",
            "18446744073709551616",
            "12345678901234567890123",
            "12345678901234567850000",
            "1.000000000000000000000000000000",
            "1e-40000",
            "1e-32768",
            "1e-32786",
            "1e40000",
            "1e32787",
            "93e32785",
            "1e2147483647",
            "1e2147483648",
            "0e99999999999",
            ".5",
            "5.",
            "00012",
        ] {
            assert!(encode(&json!(s)).is_err(), "{s} should be refused");
        }
        for v in [
            json!(4294967296u64),
            json!(-2147483649i64),
            json!(1.5),
            json!(true),
        ] {
            assert!(encode(&v).is_err(), "{v} should be refused");
        }
        // Not tried on rippled; its pattern rules them out.
        for s in [
            "", "-", "e5", "1e", "1e+", "1.2.3", "1 ", " 1", "0x10", "--1", "1e5.5", "∞",
        ] {
            assert!(encode(&json!(s)).is_err(), "{s:?} should be refused");
        }
        assert!(encode(&json!(null)).is_err());
    }

    #[test]
    fn every_read_value_writes_back_to_normal_form() {
        // What the fuzzer found: bytes not in normal form (here 13830756765796644e1)
        // must read as a value that encodes to the normal form, which reads the same.
        for &(m, e, want) in READ_BY_RIPPLED {
            if let Some(shown) = want {
                let normal = encode(&json!(shown)).unwrap_or_else(|err| panic!("{shown}: {err}"));
                assert_eq!(decode(normal).unwrap(), shown, "{m}e{e}");
            }
        }
    }
}
