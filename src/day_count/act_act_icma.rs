// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! Act/Act (ICMA) — ICMA Rule 251.
//!
//! Act/Act (ICMA) is the bond-market day-count convention prescribed by the
//! International Capital Market Association in Rule 251 and used to compute
//! accrued interest on fixed-rate bonds the world over. Unlike the
//! money-market fractions (Act/360, Act/365F) the denominator is not a
//! constant: it is the actual length, in calendar days, of the *reference*
//! coupon period multiplied by the coupon frequency. A semi-annual bond
//! whose calculation period coincides with one full coupon period therefore
//! always returns exactly `0.5`, regardless of whether that period happens
//! to be 181 or 184 days long — the property the bond market wants from a
//! "fair fraction of a coupon".
//!
//! This module implements the **regular-period** case only: the calculation
//! period sits inside (or coincides with) one reference coupon period. The
//! standard further prescribes a split-into-regular-sub-periods procedure
//! for irregular (long / short) first or last coupons; that variant is
//! deliberately out of scope here and will be added in a future revision as
//! a separate `act_act_icma_irregular` function. Callers handling an
//! irregular calculation period must either pre-decompose or wait for that
//! addition.
//!
//! # Algorithm
//!
//! ```text
//! fraction(start, end, ref_start, ref_end, freq)
//!     = days_between(start, end) / (freq * days_between(ref_start, ref_end))
//! ```
//!
//! Where `days_between` is the signed count of days from the first argument
//! to the second, end-exclusive, start-inclusive (the convention shared by
//! every fraction in this crate). `freq` is the integer coupon frequency:
//! 1 (annual), 2 (semi-annual), 4 (quarterly), 12 (monthly).
//!
//! # Defensive behaviour
//!
//! If the caller passes `freq == 0` or a degenerate reference period for
//! which `days_between(ref_start, ref_end) == 0`, the function returns
//! `0.0`. Both are precondition violations by the caller — a coupon
//! frequency of zero is not a thing, and a zero-length reference period
//! cannot be a coupon period — but returning `0.0` is preferable to
//! emitting a silent `NaN` that would propagate through downstream cashflow
//! arithmetic undetected.
//!
//! # Worked example — full semi-annual period
//!
//! ```text
//! start     = 2026-03-01
//! end       = 2026-09-01
//! ref_start = 2026-03-01    (calculation period coincides with the
//! ref_end   = 2026-09-01     reference coupon period)
//! freq      = 2             (semi-annual)
//! days(calc) = 31 + 30 + 31 + 30 + 31 + 31 = 184
//! days(ref)  = 184          (same interval)
//! f          = 184 / (2 * 184) = 0.500000000000000
//! ```
//!
//! # References
//!
//! - International Capital Market Association, *ICMA Rule 251 — Accrued
//!   Interest Calculation*.

use crate::date::Date;

/// Computes the Act/Act (ICMA) year fraction for a regular calculation
/// period.
///
/// The formula is `days_between(start, end) / (freq *
/// days_between(ref_start, ref_end))`, where the reference period is the
/// one coupon period that contains (or coincides with) the calculation
/// period. See the module-level docstring for the full algorithm, the
/// regular-period precondition, and the defensive behaviour for
/// degenerate inputs.
///
/// # Arguments
///
/// - `start`, `end`: the calculation period.
/// - `ref_start`, `ref_end`: the reference coupon period that contains
///   `[start, end)`.
/// - `freq`: coupon frequency — `1` (annual), `2` (semi-annual), `4`
///   (quarterly), `12` (monthly).
///
/// Returns `0.0` defensively if `freq == 0` or if `ref_start == ref_end`
/// (a degenerate reference period). Both are caller-side precondition
/// violations; the alternative is a silent `NaN` propagating through
/// downstream arithmetic.
///
/// # Examples
///
/// ```
/// use regit_daycount::Date;
/// use regit_daycount::day_count::act_act_icma;
///
/// // Semi-annual bond, calculation period coincides with the reference
/// // coupon period: f = 184 / (2 * 184) = 0.5 exactly.
/// let start     = Date::ymd(2026, 3, 1).unwrap();
/// let end       = Date::ymd(2026, 9, 1).unwrap();
/// let ref_start = start;
/// let ref_end   = end;
/// let f = act_act_icma::fraction(start, end, ref_start, ref_end, 2);
/// assert!((f - 0.5).abs() < 1e-12);
/// ```
#[must_use]
pub fn fraction(start: Date, end: Date, ref_start: Date, ref_end: Date, freq: u8) -> f64 {
    if freq == 0 {
        return 0.0;
    }
    let ref_days = ref_start.days_between(ref_end);
    if ref_days == 0 {
        return 0.0;
    }
    let calc_days = start.days_between(end);
    f64::from(calc_days) / (f64::from(freq) * f64::from(ref_days))
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

    // ─── ICMA worked example — full semi-annual period ───────────────────

    #[test]
    fn icma_semiannual_full_period_is_half() {
        // doc/ALGORITHMS.md §A.4 worked example:
        //   start = ref_start = 2026-03-01
        //   end   = ref_end   = 2026-09-01
        //   freq  = 2 (semi-annual)
        //   days(calc) = 31 + 30 + 31 + 30 + 31 + 31 = 184
        //   days(ref)  = 184
        //   f = 184 / (2 * 184) = 0.500000000000000
        let start = Date::ymd(2026, 3, 1).unwrap();
        let end = Date::ymd(2026, 9, 1).unwrap();
        assert_eq!(start.days_between(end), 184);
        let f = fraction(start, end, start, end, 2);
        assert!((f - 0.5).abs() < TOL);
    }

    // ─── Half of a semi-annual period ────────────────────────────────────

    #[test]
    fn icma_semiannual_half_of_period_is_quarter() {
        // Calculation period covers the first three months of a six-month
        // reference coupon period.
        //   start     = 2026-03-01, end     = 2026-06-01  → 31+30+31 = 92
        //   ref_start = 2026-03-01, ref_end = 2026-09-01  → 184 (above)
        //   freq      = 2
        //   f = 92 / (2 * 184) = 92 / 368 = 0.250000000000000
        let start = Date::ymd(2026, 3, 1).unwrap();
        let end = Date::ymd(2026, 6, 1).unwrap();
        let ref_start = Date::ymd(2026, 3, 1).unwrap();
        let ref_end = Date::ymd(2026, 9, 1).unwrap();
        assert_eq!(start.days_between(end), 92);
        assert_eq!(ref_start.days_between(ref_end), 184);
        let f = fraction(start, end, ref_start, ref_end, 2);
        assert!((f - 0.25).abs() < TOL);
    }

    // ─── Annual full period ──────────────────────────────────────────────

    #[test]
    fn icma_annual_full_period_is_one() {
        // Annual coupon, calculation period coincides with the reference
        // coupon period (one full non-leap year).
        //   start = ref_start = 2026-01-01
        //   end   = ref_end   = 2027-01-01
        //   freq  = 1
        //   days(calc) = days(ref) = 365
        //   f = 365 / (1 * 365) = 1.000000000000000
        let start = Date::ymd(2026, 1, 1).unwrap();
        let end = Date::ymd(2027, 1, 1).unwrap();
        assert_eq!(start.days_between(end), 365);
        let f = fraction(start, end, start, end, 1);
        assert!((f - 1.0).abs() < TOL);
    }

    // ─── Quarterly full period ───────────────────────────────────────────

    #[test]
    fn icma_quarterly_full_period_is_quarter() {
        // Quarterly coupon, calculation period coincides with the
        // reference coupon period.
        //   start = ref_start = 2026-03-01
        //   end   = ref_end   = 2026-06-01
        //   freq  = 4
        //   days(calc) = days(ref) = 92
        //   f = 92 / (4 * 92) = 0.250000000000000
        let start = Date::ymd(2026, 3, 1).unwrap();
        let end = Date::ymd(2026, 6, 1).unwrap();
        assert_eq!(start.days_between(end), 92);
        let f = fraction(start, end, start, end, 4);
        assert!((f - 0.25).abs() < TOL);
    }

    // ─── Defensive: zero frequency ───────────────────────────────────────

    #[test]
    fn icma_zero_freq_returns_zero() {
        // `freq == 0` is a caller-side precondition violation; the
        // function returns 0.0 rather than emitting a silent NaN that
        // would propagate through downstream cashflow arithmetic.
        let start = Date::ymd(2026, 3, 1).unwrap();
        let end = Date::ymd(2026, 9, 1).unwrap();
        let f = fraction(start, end, start, end, 0);
        assert!(f.abs() < TOL);
    }

    // ─── Defensive: degenerate reference period ──────────────────────────

    #[test]
    fn icma_zero_ref_period_returns_zero() {
        // `ref_start == ref_end` makes the denominator zero. Same
        // precondition-violation policy as `freq == 0`: return 0.0 rather
        // than NaN.
        let start = Date::ymd(2026, 3, 1).unwrap();
        let end = Date::ymd(2026, 9, 1).unwrap();
        let ref_pt = Date::ymd(2026, 3, 1).unwrap();
        let f = fraction(start, end, ref_pt, ref_pt, 2);
        assert!(f.abs() < TOL);
    }
}
