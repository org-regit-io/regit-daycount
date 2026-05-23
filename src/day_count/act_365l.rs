// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! Act/365L (Actual/365 Leap-year-sensitive) — ICMA / sterling
//! money-market convention.
//!
//! Act/365L is the "leap-aware" cousin of Act/365F: the numerator is the
//! same signed count of actual calendar days separating the two dates,
//! but the denominator switches to 366 whenever a 29 February falls
//! inside the period and stays at 365 otherwise. The convention is used
//! for sterling floating-rate notes whose coupon basis must reflect the
//! presence of a leap day inside the accrual window.
//!
//! # Algorithm
//!
//! ```text
//! numerator   = days_between(start, end)                    // signed
//! denominator = 366 if any 29 February d satisfies start <= d < end
//!               (or end <= d < start for inverted intervals)
//!               else 365
//! fraction    = numerator as f64 / denominator as f64
//! ```
//!
//! `days_between` is the signed count of days from `start` to `end`,
//! end-exclusive, start-inclusive (the standard interval convention).
//!
//! # Leap-day inclusion convention
//!
//! The test for whether a given 29 February belongs to the period is
//! **start-inclusive, end-exclusive** — `start <= ymd(y, 2, 29) < end` —
//! exactly matching the half-open convention `days_between` itself uses
//! for the numerator. So a period anchored on a leap-day boundary picks
//! up that 29 February only when it sits on the `start` edge, never when
//! it sits on the `end` edge. For an inverted interval (`start > end`),
//! the symmetric test is `end <= ymd(y, 2, 29) < start`; this keeps
//! `fraction(end, start) == -fraction(start, end)` exact.
//!
//! # Worked example — 91-day interval straddling a leap day
//!
//! ```text
//! start = 2024-01-01
//! end   = 2024-04-01
//! days  = 31 + 29 + 31 = 91    (January + February 2024 leap + March)
//! 2024-02-29 lies in [2024-01-01, 2024-04-01) → denominator = 366
//! f     = 91 / 366 = 0.248633879781421
//! ```
//!
//! # References
//!
//! - ICMA Rule 251 and the sterling money-market convention literature
//!   on the *Actual/365L* leap-aware day count.

use crate::date::Date;

/// Computes the Act/365L year fraction between two dates.
///
/// The numerator is `days_between(start, end)`; the denominator is 366 if
/// any 29 February falls inside the period under the half-open inclusion
/// convention `start <= ymd(y, 2, 29) < end` (mirrored for inverted
/// intervals), and 365 otherwise. See the module-level docstring for the
/// full algorithm and worked example.
///
/// # Examples
///
/// ```
/// use regit_daycount::Date;
/// use regit_daycount::day_count::act_365l;
///
/// // 91-day interval straddling 2024-02-29 — denominator 366.
/// let start = Date::ymd(2024, 1, 1).unwrap();
/// let end   = Date::ymd(2024, 4, 1).unwrap();
/// assert!((act_365l::fraction(start, end) - 91.0_f64 / 366.0).abs() < 1e-12);
/// ```
#[must_use]
pub fn fraction(start: Date, end: Date) -> f64 {
    let numerator = f64::from(start.days_between(end));
    let denominator = if period_contains_leap_day(start, end) {
        366.0_f64
    } else {
        365.0_f64
    };
    numerator / denominator
}

/// Returns `true` if any 29 February falls inside the half-open period
/// bounded by `start` and `end`.
///
/// The convention is start-inclusive, end-exclusive on the forward
/// orientation and end-inclusive, start-exclusive on the inverted
/// orientation, so `period_contains_leap_day(a, b) ==
/// period_contains_leap_day(b, a)` — the denominator does not depend on
/// the direction the period is traversed.
fn period_contains_leap_day(start: Date, end: Date) -> bool {
    // Normalise the orientation: scan the years of the smaller endpoint
    // through the years of the larger one. The half-open convention
    // `lower <= ymd(y, 2, 29) < upper` (the `lower` edge is included,
    // the `upper` edge is not) matches the half-open convention
    // `days_between` already uses, so the denominator stays consistent
    // for inverted intervals: `fraction(end, start) == -fraction(start,
    // end)` exactly.
    let (lower, upper) = if start <= end {
        (start, end)
    } else {
        (end, start)
    };
    let y_lo = lower.year();
    let y_hi = upper.year();
    let mut y = y_lo;
    while y <= y_hi {
        if Date::is_leap_year(y) {
            let feb29 = Date::ymd_unchecked(y, 2, 29);
            if lower <= feb29 && feb29 < upper {
                return true;
            }
        }
        y += 1;
    }
    false
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

    // ─── Worked example — Q1 2024 straddles 2024-02-29 ───────────────────

    #[test]
    fn worked_2024_q1_straddles_leap() {
        // doc/ALGORITHMS.md §A.8 worked example:
        //   start = 2024-01-01, end = 2024-04-01
        //   days  = 31 + 29 + 31 = 91 (Jan + Feb leap + Mar)
        //   2024-02-29 ∈ [2024-01-01, 2024-04-01) → denominator = 366
        //   f     = 91 / 366 = 0.248633879781421
        let start = Date::ymd(2024, 1, 1).unwrap();
        let end = Date::ymd(2024, 4, 1).unwrap();
        assert_eq!(start.days_between(end), 91);
        let f = fraction(start, end);
        assert!((f - 91.0_f64 / 366.0).abs() < 1e-15);
        assert!((f - 0.248_633_879_781_421).abs() < TOL);
    }

    // ─── Worked example — Q2 2024 lies after the leap day ────────────────

    #[test]
    fn worked_2024_q2_no_leap() {
        // start = 2024-04-01, end = 2024-07-01. 2024 is a leap year but
        // 2024-02-29 sits *before* the period, so the denominator
        // collapses to 365.
        //   days  = 30 + 31 + 30 = 91 (Apr + May + Jun)
        //   f     = 91 / 365 = 0.249315068493151
        let start = Date::ymd(2024, 4, 1).unwrap();
        let end = Date::ymd(2024, 7, 1).unwrap();
        assert_eq!(start.days_between(end), 91);
        let f = fraction(start, end);
        assert!((f - 91.0_f64 / 365.0).abs() < 1e-15);
        assert!((f - 0.249_315_068_493_151).abs() < TOL);
    }

    // ─── Worked example — Q1 2026 — non-leap year ────────────────────────

    #[test]
    fn worked_2026_no_leap_in_2026() {
        // start = 2026-01-01, end = 2026-04-01. 2026 is not a leap year,
        // so no 29 February exists anywhere in the span.
        //   days  = 31 + 28 + 31 = 90 (Jan + Feb non-leap + Mar)
        //   f     = 90 / 365 = 0.246575342465753
        let start = Date::ymd(2026, 1, 1).unwrap();
        let end = Date::ymd(2026, 4, 1).unwrap();
        assert_eq!(start.days_between(end), 90);
        let f = fraction(start, end);
        assert!((f - 90.0_f64 / 365.0).abs() < 1e-15);
        assert!((f - 0.246_575_342_465_753).abs() < TOL);
    }

    // ─── Cross-year straddle picking up 2024-02-29 ───────────────────────

    #[test]
    fn straddle_leap_2024_to_2025() {
        // start = 2023-06-01, end = 2024-06-01. The span crosses
        // 2024-02-29 from before, so the denominator is 366. The actual
        // day count is also 366, so the fraction is exactly 1.
        let start = Date::ymd(2023, 6, 1).unwrap();
        let end = Date::ymd(2024, 6, 1).unwrap();
        assert_eq!(start.days_between(end), 366);
        let f = fraction(start, end);
        assert!((f - 1.0).abs() < TOL);
    }

    // ─── One-year non-leap span ──────────────────────────────────────────

    #[test]
    fn period_in_non_leap_year() {
        // start = 2025-01-01, end = 2026-01-01. 2025 is not a leap year
        // and no 29 February falls inside, so denominator = 365 and the
        // 365-day numerator gives exactly 1.
        let start = Date::ymd(2025, 1, 1).unwrap();
        let end = Date::ymd(2026, 1, 1).unwrap();
        assert_eq!(start.days_between(end), 365);
        let f = fraction(start, end);
        assert!((f - 1.0).abs() < TOL);
    }

    // ─── Zero-length interval ────────────────────────────────────────────

    #[test]
    fn zero_length_interval_is_zero() {
        // `fraction(d, d)` is identically zero — a degenerate but legal
        // input; callers occasionally hit it on a same-day reset. The
        // half-open inclusion test cannot match because the interval is
        // empty, so the denominator falls through to 365 and 0 / 365 = 0.
        let d = Date::ymd(2026, 5, 23).unwrap();
        let f = fraction(d, d);
        assert!(f.abs() < TOL);
    }

    // ─── Inverted interval ───────────────────────────────────────────────

    #[test]
    fn inverted_interval_negates_correctly() {
        // The crate does not reject `end < start`; the result is the
        // negation of the forward fraction. The denominator-scan must
        // therefore be orientation-symmetric: if the forward period
        // picked up 2024-02-29 with denominator 366, the reverse period
        // must pick it up with denominator 366 as well, or the negation
        // identity breaks. Span chosen to straddle 2024-02-29.
        let start = Date::ymd(2024, 1, 1).unwrap();
        let end = Date::ymd(2024, 4, 1).unwrap();
        let forward = fraction(start, end);
        let backward = fraction(end, start);
        assert!((backward + forward).abs() < TOL);
        assert!(backward < 0.0);
        // Sanity: the magnitudes are the 91 / 366 worked-example value.
        assert!((forward - 91.0_f64 / 366.0).abs() < TOL);
        assert!((backward + 91.0_f64 / 366.0).abs() < TOL);
    }
}
