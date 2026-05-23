// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! Act/360 — ISDA 2006 §4.16(e).
//!
//! Act/360 is the money-market default: the year fraction between two
//! dates is the actual count of calendar days separating them, divided by
//! the fixed denominator 360. Every major short-term floating-rate index
//! settled in USD or EUR — SOFR, €STR, the legacy USD-LIBOR fixings, the
//! Fed Funds effective rate — accrues on Act/360, and so do the EUR and
//! USD legs of the overwhelming majority of interest-rate swaps. The
//! procedure is the simplest of the eleven conventions in this crate: a
//! signed day count over the period, then one division.
//!
//! # Algorithm
//!
//! ```text
//! fraction(start, end) = days_between(start, end) / 360.0
//! ```
//!
//! Where `days_between` is the signed count of days from `start` to `end`,
//! end-exclusive, start-inclusive (the standard interval convention).
//! The result is positive for `start <= end` and negative otherwise; the
//! crate does not reject inverted intervals because some callers compute
//! reverse-period accruals.
//!
//! # Worked example — 90-day interval
//!
//! ```text
//! start = 2026-01-01
//! end   = 2026-04-01
//! days  = 31 + 28 + 31 = 90    (January + February (non-leap) + March)
//! f     = 90 / 360 = 0.250000000000000
//! ```
//!
//! # References
//!
//! - ISDA 2006 Definitions §4.16(e), *Actual/360*.

use crate::date::Date;

/// Computes the Act/360 year fraction between two dates.
///
/// The result is `(end - start) / 360` in days, where `(end - start)` is
/// signed. See the module-level docstring for the full algorithm and
/// worked example.
///
/// # Examples
///
/// ```
/// use regit_daycount::Date;
/// use regit_daycount::day_count::act_360;
///
/// // 90-day interval — the ISDA worked example.
/// let start = Date::ymd(2026, 1, 1).unwrap();
/// let end   = Date::ymd(2026, 4, 1).unwrap();
/// assert!((act_360::fraction(start, end) - 0.25).abs() < 1e-12);
/// ```
#[must_use]
pub fn fraction(start: Date, end: Date) -> f64 {
    f64::from(start.days_between(end)) / 360.0
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

    // ─── ISDA worked example ─────────────────────────────────────────────

    #[test]
    fn isda_worked_example_90_days() {
        // doc/ALGORITHMS.md §A.1 worked example 1:
        //   start = 2026-01-01, end = 2026-04-01
        //   days  = 31 + 28 + 31 = 90 (Jan + Feb non-leap + Mar)
        //   f     = 90 / 360 = 0.250000000000000
        let start = Date::ymd(2026, 1, 1).unwrap();
        let end = Date::ymd(2026, 4, 1).unwrap();
        let f = fraction(start, end);
        assert!((f - 90.0_f64 / 360.0).abs() < 1e-15);
        assert!((f - 0.25).abs() < TOL);
    }

    // ─── Second worked example — 1-year non-leap span ────────────────────

    #[test]
    fn one_year_non_leap_is_365_over_360() {
        // doc/ALGORITHMS.md §A.1 worked example 2:
        //   start = 2026-02-01, end = 2027-02-01
        //   days  = 365 (the twelve months Feb 2026 … Jan 2027 sum to 365,
        //                neither 2026 nor the partial 2027 contributes a
        //                29 February)
        //   f     = 365 / 360 = 1.013888888888889
        let start = Date::ymd(2026, 2, 1).unwrap();
        let end = Date::ymd(2027, 2, 1).unwrap();
        let f = fraction(start, end);
        assert_eq!(start.days_between(end), 365);
        assert!((f - 365.0_f64 / 360.0).abs() < TOL);
    }

    // ─── Zero-length interval ────────────────────────────────────────────

    #[test]
    fn zero_length_interval_is_zero() {
        // `fraction(d, d)` is identically zero — a degenerate but legal
        // input; callers occasionally hit it on a same-day reset.
        let d = Date::ymd(2026, 5, 23).unwrap();
        let f = fraction(d, d);
        // Division of zero by 360.0 is exactly 0.0 in IEEE-754, but we
        // assert through the same tolerance the rest of the suite uses
        // rather than compare floats directly.
        assert!(f.abs() < TOL);
    }

    // ─── Inverted interval ───────────────────────────────────────────────

    #[test]
    fn inverted_interval_is_negative() {
        // The crate does not reject `end < start`; the result is the
        // negation of the forward fraction. Some callers compute
        // reverse-period accruals and rely on this.
        let start = Date::ymd(2026, 1, 1).unwrap();
        let end = Date::ymd(2026, 4, 1).unwrap();
        let forward = fraction(start, end);
        let backward = fraction(end, start);
        assert!((backward + forward).abs() < TOL);
        assert!(backward < 0.0);
    }

    // ─── Leap-year span ──────────────────────────────────────────────────

    #[test]
    fn one_year_leap_is_366_over_360() {
        // 2024-02-01 → 2025-02-01 spans 366 days because the interval
        // includes 2024-02-29 (2024 is a Gregorian leap year). Act/360
        // is leap-year-blind in the denominator, so the fraction grows
        // by exactly 1/360 over the non-leap one-year span.
        let start = Date::ymd(2024, 2, 1).unwrap();
        let end = Date::ymd(2025, 2, 1).unwrap();
        assert_eq!(start.days_between(end), 366);
        let f = fraction(start, end);
        assert!((f - 366.0_f64 / 360.0).abs() < TOL);
    }

    // ─── Additivity over a split date ────────────────────────────────────

    #[test]
    fn additivity_over_split() {
        // For any `b` in `[a, c]`, `f(a, c) == f(a, b) + f(b, c)`. This
        // holds exactly for Act/360 because the denominator is constant
        // and `days_between` is itself additive. Three split points are
        // checked — one near the start, one mid-period, one near the end.
        let a = Date::ymd(2024, 1, 1).unwrap();
        let c = Date::ymd(2026, 7, 15).unwrap();
        for b in [
            Date::ymd(2024, 1, 2).unwrap(),
            Date::ymd(2025, 3, 31).unwrap(),
            Date::ymd(2026, 7, 14).unwrap(),
        ] {
            let lhs = fraction(a, c);
            let rhs = fraction(a, b) + fraction(b, c);
            assert!(
                (lhs - rhs).abs() < TOL,
                "additivity at split {b:?}: lhs={lhs}, rhs={rhs}",
            );
        }
    }
}
