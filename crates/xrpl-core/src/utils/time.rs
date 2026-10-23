//! Ledger time ("Ripple time": seconds since 2000-01-01T00:00:00Z) ↔ Unix time
//! and ISO 8601, as rippled's `close_time`, `Expiration`, `CancelAfter`,
//! `FinishAfter` and the like use it.

use crate::CoreError;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// 2000-01-01T00:00:00Z in Unix seconds: ledger time zero.
pub const RIPPLE_EPOCH_UNIX: i64 = 946_684_800;

/// Ledger time → Unix seconds.
pub fn ripple_time_to_unix(ripple_time: u32) -> i64 {
    i64::from(ripple_time) + RIPPLE_EPOCH_UNIX
}

/// Unix seconds → ledger time. Errors before 2000 or after the last second a
/// `u32` ledger time can hold (2136-02-07T06:28:15Z).
///
/// # Errors
///
/// [`CoreError::ValidationError`] for a time outside that range.
pub fn unix_to_ripple_time(unix_seconds: i64) -> Result<u32, CoreError> {
    unix_seconds
        .checked_sub(RIPPLE_EPOCH_UNIX)
        .and_then(|t| u32::try_from(t).ok())
        .ok_or_else(|| {
            CoreError::ValidationError(format!(
                "Unix time {unix_seconds} is outside ledger time (2000-01-01 to 2136-02-07)"
            ))
        })
}

/// Ledger time → `SystemTime`.
pub fn ripple_time_to_system_time(ripple_time: u32) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(ripple_time_to_unix(ripple_time) as u64)
}

/// `SystemTime` → ledger time, truncated to the second.
///
/// # Errors
///
/// [`CoreError::ValidationError`] for a time before 2000 or after
/// 2136-02-07T06:28:15Z.
pub fn system_time_to_ripple_time(time: SystemTime) -> Result<u32, CoreError> {
    let secs = time
        .duration_since(UNIX_EPOCH)
        .map_err(|_| CoreError::ValidationError("time is before 1970".into()))?
        .as_secs();
    unix_to_ripple_time(i64::try_from(secs).unwrap_or(i64::MAX))
}

/// Ledger time → ISO 8601 in UTC (`"2000-01-01T00:00:00Z"`).
///
/// ```
/// use xrpl_core::utils::time::ripple_time_to_iso8601;
/// assert_eq!(ripple_time_to_iso8601(0), "2000-01-01T00:00:00Z");
/// ```
pub fn ripple_time_to_iso8601(ripple_time: u32) -> String {
    let unix = ripple_time_to_unix(ripple_time);
    let (days, secs) = (unix.div_euclid(86_400), unix.rem_euclid(86_400));
    let (y, m, d) = civil_from_days(days);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        secs / 3600,
        secs % 3600 / 60,
        secs % 60
    )
}

/// ISO 8601 in UTC (`"2024-06-01T12:00:00Z"`, `"…:00.250Z"`, `"…:00+00:00"`)
/// → ledger time. A fraction of a second is dropped.
///
/// # Errors
///
/// [`CoreError::ValidationError`] for text not in that form (or not a real date
/// and time), or a time outside ledger time (2000-01-01 to
/// 2136-02-07T06:28:15Z).
pub fn iso8601_to_ripple_time(iso: &str) -> Result<u32, CoreError> {
    let bad = || CoreError::ValidationError(format!("not a UTC ISO 8601 time: {iso:?}"));
    let body = iso
        .strip_suffix('Z')
        .or_else(|| iso.strip_suffix("+00:00"))
        .ok_or_else(bad)?;
    let body = match body.split_once('.') {
        Some((b, frac)) if !frac.is_empty() && frac.bytes().all(|c| c.is_ascii_digit()) => b,
        Some(_) => return Err(bad()),
        None => body,
    };
    let field = |range: std::ops::Range<usize>| -> Result<i64, CoreError> {
        let s = body.get(range).ok_or_else(bad)?;
        if !s.bytes().all(|c| c.is_ascii_digit()) {
            return Err(bad());
        }
        s.parse().map_err(|_| bad())
    };
    let shape_ok = body.len() == 19
        && body.get(4..5) == Some("-")
        && body.get(7..8) == Some("-")
        && body.get(10..11) == Some("T")
        && body.get(13..14) == Some(":")
        && body.get(16..17) == Some(":");
    if !shape_ok {
        return Err(bad());
    }
    let (y, m, d) = (field(0..4)?, field(5..7)?, field(8..10)?);
    let (hh, mm, ss) = (field(11..13)?, field(14..16)?, field(17..19)?);
    if !(1..=12).contains(&m) || d < 1 || d > days_in_month(y, m) || hh > 23 || mm > 59 || ss > 59 {
        return Err(bad());
    }
    unix_to_ripple_time(days_from_civil(y, m, d) * 86_400 + hh * 3600 + mm * 60 + ss)
}

fn days_in_month(y: i64, m: i64) -> i64 {
    match m {
        2 if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

// Howard Hinnant's days ↔ civil date algorithms (proleptic Gregorian, UTC).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (yoe + era * 400 + i64::from(m <= 2), m, d)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // xrpl.js timeConversion tests (it counts Unix time in milliseconds).
    #[test]
    fn matches_xrpl_js() {
        assert_eq!(ripple_time_to_iso8601(0), "2000-01-01T00:00:00Z");
        assert_eq!(
            iso8601_to_ripple_time("2000-01-01T00:00:00.000Z").unwrap(),
            0
        );
        assert_eq!(unix_to_ripple_time(946_684_801).unwrap(), 1);
        assert_eq!(ripple_time_to_unix(1), 946_684_801);
    }

    #[test]
    fn round_trips_and_edges() {
        for t in [0, 1, 59, 86_399, 86_400, 814_406_400, 846_000_000, u32::MAX] {
            assert_eq!(
                iso8601_to_ripple_time(&ripple_time_to_iso8601(t)).unwrap(),
                t
            );
            assert_eq!(unix_to_ripple_time(ripple_time_to_unix(t)).unwrap(), t);
        }
        assert_eq!(ripple_time_to_iso8601(u32::MAX), "2136-02-07T06:28:15Z");
        assert_eq!(ripple_time_to_iso8601(131_328_000), "2004-02-29T00:00:00Z");
        assert_eq!(
            iso8601_to_ripple_time("2024-06-01T12:00:00+00:00").unwrap(),
            770_558_400
        );
        assert!(unix_to_ripple_time(RIPPLE_EPOCH_UNIX - 1).is_err());
        for bad in [
            "1999-12-31T23:59:59Z",
            "2023-02-29T00:00:00Z",
            "2024-13-01T00:00:00Z",
            "2024-06-01T24:00:00Z",
            "2024-06-01T12:00:00",
            "2024-06-01 12:00:00Z",
            "2024-06-01T12:00:00.Z",
            "2024-06-01T12:00:00+02:00",
            "",
        ] {
            assert!(iso8601_to_ripple_time(bad).is_err(), "{bad:?} parsed");
        }
        let t = ripple_time_to_system_time(5);
        assert_eq!(system_time_to_ripple_time(t).unwrap(), 5);
    }

    #[test]
    fn extreme_unix_times_are_errors_not_panics() {
        assert!(unix_to_ripple_time(i64::MIN).is_err());
        assert!(unix_to_ripple_time(i64::MAX).is_err());
    }
}
