// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! NL/365 — "No-Leap / 365", conventional money-market variant.
//!
//! NL/365 is a money-market variant of Act/365F that excludes 29 February
//! from the numerator: the fraction's numerator is the actual count of
//! calendar days between `start` and `end`, *minus* the number of
//! 29-February dates that fall in the half-open interval `[start, end)`;
//! the denominator is the fixed constant 365. The effect is that a full
//! leap-year span and a full non-leap-year span both return exactly
//! `1.0` — the convention is "leap-year-blind" by construction in the
//! same way Act/360 is leap-year-blind by virtue of a constant 360
//! denominator, but here every year — leap or not — contributes exactly
//! `365 / 365` to the fraction.
//!
//! The convention has no entry in the ISDA 2006 Definitions §4.16
//! catalogue; it is a market practice convention found principally in
//! some legacy money-market and fixed-income contracts, and is included
//! here for completeness alongside the ICMA Act/365L and the ISDA
//! fractions.
//!
//! # Algorithm
//!
//! ```text
//! numerator   = days_between(start, end) - leap_days_in_interval(start, end)
//! denominator = 365
//! fraction    = numerator / 365.0
//! ```
//!
//! Where `days_between` is the signed count of days from `start` to
//! `end`, end-exclusive, start-inclusive, and `leap_days_in_interval`
//! is the count of dates `Date::ymd(y, 2, 29)` (for any leap year `y`)
//! satisfying `start <= d < end` — the same half-open convention.
//!
//! For an inverted interval (`start > end`) the function preserves
//! `fraction(end, start) == -fraction(start, end)`: the leap-day count
//! is taken over the forward half-open interval and the sign is carried
//! by the signed `days_between`.
//!
//! # Worked example — Q1 2024 straddles 29 February
//!
//! ```text
//! start = 2024-01-01
//! end   = 2024-04-01
//! days  = 31 + 29 + 31 = 91     (Jan + Feb (2024 leap) + Mar)
//! 29-Feb in [start, end): 2024-02-29 → subtract 1
//! num   = 91 - 1 = 90
//! f     = 90 / 365 = 0.246575342465753
//! ```
//!
//! # References
//!
//! - Market-convention day-count catalogues that list NL/365 alongside
//!   the ISDA / ICMA fractions — the convention is non-statutory; no
//!   single primary reference applies.

use crate::date::Date;

/// Computes the NL/365 year fraction between two dates.
///
/// The result is `(days_between(start, end) - leap_days_in_interval) /
/// 365`, where `days_between` is signed and `leap_days_in_interval` is
/// the count of 29-February dates in the half-open interval
/// `[min(start, end), max(start, end))`. See the module-level docstring
/// for the full algorithm and worked example.
///
/// The function is sign-preserving: `fraction(end, start) ==
/// -fraction(start, end)`. The crate does not reject inverted intervals
/// because some callers compute reverse-period accruals.
///
/// # Examples
///
/// ```
/// use regit_daycount::Date;
/// use regit_daycount::day_count::nl_365;
///
/// // Q1 2024 straddles 2024-02-29: 91 actual days − 1 leap day = 90.
/// let start = Date::ymd(2024, 1, 1).unwrap();
/// let end   = Date::ymd(2024, 4, 1).unwrap();
/// assert!((nl_365::fraction(start, end) - 90.0_f64 / 365.0).abs() < 1e-12);
/// ```
#[must_use]
pub fn fraction(start: Date, end: Date) -> f64 {
    let raw = start.days_between(end);
    // Order the endpoints so the leap-day count is taken over the
    // forward half-open interval `[lo, hi)`; the sign of the result is
    // carried by `raw` itself. This preserves
    // `fraction(end, start) == -fraction(start, end)`.
    let (lo, hi, sign) = if raw >= 0 {
        (start, end, 1_i32)
    } else {
        (end, start, -1_i32)
    };
    let leap_days = leap_days_in_interval(lo, hi);
    let numerator = raw - sign * leap_days;
    f64::from(numerator) / 365.0
}

/// Counts the 29-February dates that fall in the half-open interval
/// `[lo, hi)`, where `lo <= hi`.
///
/// Iterates over each year touched by the interval (inclusive of both
/// endpoints' years) and tests whether `Date::ymd(y, 2, 29)` — when it
/// exists, i.e. when `y` is a Gregorian leap year — lies in `[lo, hi)`.
/// The iteration is bounded by the year span of the interval and is
/// allocation-free.
//
// `lo <= hi` is established by the caller (the only call site orders
// the endpoints before invoking this helper); the loop bound therefore
// terminates and never iterates an empty range backward.
fn leap_days_in_interval(lo: Date, hi: Date) -> i32 {
    let mut count: i32 = 0;
    let y_start = lo.year();
    let y_end = hi.year();
    let mut y = y_start;
    while y <= y_end {
        if Date::is_leap_year(y) {
            let feb29 = Date::ymd_unchecked(y, 2, 29);
            if feb29 >= lo && feb29 < hi {
                count += 1;
            }
        }
        y += 1;
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    // The tolerance 1e-12 is the working slack used throughout this crate's
    // numeric assertions: it is loose enough that a benign last-bit rounding
    // never causes a spurious failure, and tight enough that any drift large
    // enough to mis-state a cashflow at the cent level (≥ 1e-9 on a unit
    // notional) is caught immediately. We deliberately do not assert against
    // `f64::EPSILON`; the goal is auditable, not maximally-tight.
    const TOL: f64 = 1e-12;

    // ─── Worked example — Q1 2024 straddles 29 February ──────────────────

    #[test]
    fn worked_2024_q1_straddles_leap() {
        // doc/ALGORITHMS.md §A.9 worked example:
        //   start = 2024-01-01, end = 2024-04-01
        //   days  = 31 + 29 + 31 = 91 (Jan + Feb leap + Mar)
        //   29-Feb in [start, end): 2024-02-29 → subtract 1
        //   num   = 90
        //   f     = 90 / 365 = 0.246575342465753
        let start = Date::ymd(2024, 1, 1).unwrap();
        let end = Date::ymd(2024, 4, 1).unwrap();
        assert_eq!(start.days_between(end), 91);
        let f = fraction(start, end);
        assert!((f - 90.0_f64 / 365.0).abs() < TOL);
    }

    // ─── Worked example — Q2 2024 (no 29 Feb in interval) ────────────────

    #[test]
    fn worked_2024_q2_no_leap() {
        // start = 2024-04-01, end = 2024-07-01
        // days  = 30 (Apr) + 31 (May) + 30 (Jun) = 91
        // No 29-Feb in [start, end). num = 91.
        // f = 91 / 365 = 0.249315068493151
        let start = Date::ymd(2024, 4, 1).unwrap();
        let end = Date::ymd(2024, 7, 1).unwrap();
        assert_eq!(start.days_between(end), 91);
        let f = fraction(start, end);
        assert!((f - 91.0_f64 / 365.0).abs() < TOL);
    }

    // ─── Worked example — full leap year ─────────────────────────────────

    #[test]
    fn worked_full_leap_year() {
        // start = 2024-01-01, end = 2025-01-01 (full leap year)
        // days  = 366. Subtract 1 for 2024-02-29. num = 365.
        // f = 365 / 365 = 1.0 exactly.
        let start = Date::ymd(2024, 1, 1).unwrap();
        let end = Date::ymd(2025, 1, 1).unwrap();
        assert_eq!(start.days_between(end), 366);
        let f = fraction(start, end);
        // Exact equality: 365 / 365 is exactly 1.0 in IEEE-754.
        assert!((f - 1.0).abs() < TOL);
    }

    // ─── Worked example — full non-leap year ─────────────────────────────

    #[test]
    fn worked_full_non_leap_year() {
        // start = 2025-01-01, end = 2026-01-01 (full non-leap year)
        // days  = 365. No 29-Feb in interval. num = 365.
        // f = 365 / 365 = 1.0.
        let start = Date::ymd(2025, 1, 1).unwrap();
        let end = Date::ymd(2026, 1, 1).unwrap();
        assert_eq!(start.days_between(end), 365);
        let f = fraction(start, end);
        assert!((f - 1.0).abs() < TOL);
    }

    // ─── Worked example — span over two leap years (2024..2028) ──────────

    #[test]
    fn worked_two_leap_years_span_2024_2028() {
        // start = 2024-01-01, end = 2028-01-01
        // days = 366 + 365 + 365 + 365 = 1461 (2024 leap; 2025, 2026, 2027
        // non-leap). 2024-02-29 IS in [start, end); 2028-02-29 is NOT
        // (the interval ends at 2028-01-01 < 2028-02-29). Subtract 1.
        // num = 1460. f = 1460 / 365 = 4.0 exactly.
        let start = Date::ymd(2024, 1, 1).unwrap();
        let end = Date::ymd(2028, 1, 1).unwrap();
        assert_eq!(start.days_between(end), 1461);
        let f = fraction(start, end);
        assert!((f - 4.0).abs() < TOL);
    }

    // ─── Zero-length interval ────────────────────────────────────────────

    #[test]
    fn zero_length_is_zero() {
        // `fraction(d, d)` is identically zero — a degenerate but legal
        // input; callers occasionally hit it on a same-day reset. The
        // half-open interval `[d, d)` is empty so no 29-Feb can land in it.
        let d = Date::ymd(2024, 2, 29).unwrap();
        let f = fraction(d, d);
        assert!(f.abs() < TOL);
    }

    // ─── Inverted interval ───────────────────────────────────────────────

    #[test]
    fn inverted_negates() {
        // The crate does not reject `end < start`; the result is the
        // negation of the forward fraction. Picked an interval that
        // contains 2024-02-29 so the leap-day subtraction also exercises
        // the inverted branch.
        let start = Date::ymd(2024, 1, 1).unwrap();
        let end = Date::ymd(2024, 4, 1).unwrap();
        let forward = fraction(start, end);
        let backward = fraction(end, start);
        assert!((backward + forward).abs() < TOL);
        assert!(backward < 0.0);
    }
}
