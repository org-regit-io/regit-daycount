// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! 30E/360 — ISDA 2006 §4.16(g).
//!
//! 30E/360 — the "Eurobond Basis" — is the day-count fraction the European
//! corporate-bond market uses on its fixed legs. It is a thirty-day-month
//! convention: every month is treated as having exactly 30 days and every
//! year as having exactly 360, with a small set of end-of-month adjustments
//! applied to the raw `(year, month, day)` triples before the arithmetic.
//! The result is a clean, semi-annual coupon period that always lands on
//! 0.5 regardless of the actual day count.
//!
//! # Difference from 30/360 Bond Basis (ISDA §4.16(f))
//!
//! 30E/360 and 30/360 Bond Basis share the `D1 = 31 → 30` rewrite, but they
//! differ on the `D2` adjustment. Bond Basis only rewrites `D2 = 31 → 30`
//! when `D1` is already `30` or `31` after step 1 — i.e. it makes the `D2`
//! collapse *conditional* on the start of the period. 30E/360 makes the
//! same `D2 = 31 → 30` collapse *unconditional* — it always fires, no
//! matter what `D1` is. The two conventions therefore disagree on intervals
//! whose `start.day` is not in `{30, 31}` and whose `end.day` is `31`:
//! `2026-01-15` → `2026-07-31` yields 195/360 here and 196/360 under Bond
//! Basis. Every other input the two conventions agree on.
//!
//! # Algorithm
//!
//! With `(Y1, M1, D1) = (start.year, start.month, start.day)` and
//! `(Y2, M2, D2) = (end.year, end.month, end.day)`:
//!
//! ```text
//! 1. if D1 == 31:           D1 = 30
//! 2. if D2 == 31:           D2 = 30          (unconditional on D1)
//! 3. numerator = 360 * (Y2 - Y1) + 30 * (M2 - M1) + (D2 - D1)
//! 4. f         = numerator / 360.0
//! ```
//!
//! The arithmetic is performed in `i32`; for any input pair in the crate's
//! supported year range the numerator fits comfortably (a 9999-year span
//! is `~3.6 × 10⁶`, well inside `i32`).
//!
//! # Worked example — semi-annual coupon ending on month-end
//!
//! ```text
//! start = 2026-01-31
//! end   = 2026-07-31
//! D1: 31 → 30
//! D2: 31 → 30                              (unconditional)
//! numerator = 360*0 + 30*(7-1) + (30-30) = 180
//! f         = 180 / 360 = 0.500000000000000
//! ```
//!
//! # References
//!
//! - ISDA 2006 Definitions §4.16(g), *30E/360* (also marketed as the
//!   "Eurobond Basis").

use crate::date::Date;

/// Computes the 30E/360 (Eurobond Basis) year fraction between two dates.
///
/// See the module-level docstring for the algorithm, the `BondBasis`
/// difference, and the worked example. The result is signed — inverted
/// intervals (`end < start`) yield a negative fraction by construction —
/// because the crate does not reject reverse-period accruals.
///
/// # Examples
///
/// ```
/// use regit_daycount::Date;
/// use regit_daycount::day_count::thirty_e_360;
///
/// // ISDA §4.16(g) style worked example — 31 Jan to 31 Jul collapses to 0.5.
/// let start = Date::ymd(2026, 1, 31).unwrap();
/// let end   = Date::ymd(2026, 7, 31).unwrap();
/// assert!((thirty_e_360::fraction(start, end) - 0.5).abs() < 1e-12);
/// ```
#[must_use]
pub fn fraction(start: Date, end: Date) -> f64 {
    let y1 = start.year();
    let m1 = i32::from(start.month());
    let mut d1 = i32::from(start.day());
    let y2 = end.year();
    let m2 = i32::from(end.month());
    let mut d2 = i32::from(end.day());

    // Step 1: collapse a `D1 == 31` start to a 30-day-month start.
    if d1 == 31 {
        d1 = 30;
    }
    // Step 2: collapse a `D2 == 31` end to a 30-day-month end. The
    // adjustment is UNCONDITIONAL on `D1` — this is the single rule that
    // separates 30E/360 from 30/360 Bond Basis (§4.16(f)).
    if d2 == 31 {
        d2 = 30;
    }

    let numerator = 360 * (y2 - y1) + 30 * (m2 - m1) + (d2 - d1);
    f64::from(numerator) / 360.0
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

    // ─── No-adjustment baseline ──────────────────────────────────────────

    #[test]
    fn eurobond_6m_mid_month() {
        // start = 2026-01-15, end = 2026-07-15. Neither D1 nor D2 hits the
        // 31 → 30 rewrite, so the numerator is the raw 30 * (M2 - M1).
        //   numerator = 360*0 + 30*(7-1) + (15-15) = 180
        //   f         = 180 / 360 = 0.5
        let start = Date::ymd(2026, 1, 15).unwrap();
        let end = Date::ymd(2026, 7, 15).unwrap();
        let f = fraction(start, end);
        assert!((f - 180.0_f64 / 360.0).abs() < 1e-15);
        assert!((f - 0.5).abs() < TOL);
    }

    // ─── D1 adjustment ───────────────────────────────────────────────────

    #[test]
    fn eurobond_d1_31_adjusts() {
        // start = 2026-01-31, end = 2026-02-28. D1: 31 → 30. D2 stays 28.
        //   numerator = 360*0 + 30*(2-1) + (28-30) = 30 + (-2) = 28
        //   f         = 28 / 360
        let start = Date::ymd(2026, 1, 31).unwrap();
        let end = Date::ymd(2026, 2, 28).unwrap();
        let f = fraction(start, end);
        assert!((f - 28.0_f64 / 360.0).abs() < TOL);
    }

    // ─── D2 unconditional adjustment ─────────────────────────────────────

    #[test]
    fn eurobond_d2_31_unconditional_adjust() {
        // start = 2026-01-15, end = 2026-07-31. D1 = 15 (no change),
        // D2: 31 → 30 unconditionally. Under 30E/360 the D2 collapse
        // fires even though D1 is not in {30, 31}. (Under Bond Basis the
        // same input would leave D2 = 31 and produce 196/360.)
        //   numerator = 360*0 + 30*(7-1) + (30-15) = 180 + 15 = 195
        //   f         = 195 / 360
        let start = Date::ymd(2026, 1, 15).unwrap();
        let end = Date::ymd(2026, 7, 31).unwrap();
        let f = fraction(start, end);
        assert!((f - 195.0_f64 / 360.0).abs() < TOL);
    }

    // ─── Both D1 and D2 adjust ───────────────────────────────────────────

    #[test]
    fn eurobond_d1_31_d2_31() {
        // start = 2026-01-31, end = 2026-07-31. Both endpoints collapse
        // to 30; the numerator is a clean 30 * (M2 - M1).
        //   numerator = 360*0 + 30*(7-1) + (30-30) = 180
        //   f         = 180 / 360 = 0.5
        let start = Date::ymd(2026, 1, 31).unwrap();
        let end = Date::ymd(2026, 7, 31).unwrap();
        let f = fraction(start, end);
        assert!((f - 0.5).abs() < TOL);
    }

    // ─── ALGORITHMS.md §A.6 worked example ───────────────────────────────

    #[test]
    fn eurobond_worked_2026_01_31_to_07_31() {
        // doc/ALGORITHMS.md §A.6 worked example:
        //   start = 2026-01-31, end = 2026-07-31
        //   D1 = 31 → D1 = 30, D2 = 31 → D2 = 30
        //   numerator = 360*0 + 30*(7-1) + (30-30) = 180
        //   f         = 180 / 360 = 0.500000000000000
        let start = Date::ymd(2026, 1, 31).unwrap();
        let end = Date::ymd(2026, 7, 31).unwrap();
        let f = fraction(start, end);
        assert!((f - 180.0_f64 / 360.0).abs() < 1e-15);
        assert!((f - 0.5).abs() < TOL);
    }

    // ─── Zero-length interval ────────────────────────────────────────────

    #[test]
    fn eurobond_zero_length() {
        // `fraction(d, d)` is identically zero — a degenerate but legal
        // input; callers occasionally hit it on a same-day reset.
        let d = Date::ymd(2026, 5, 23).unwrap();
        let f = fraction(d, d);
        assert!(f.abs() < TOL);
    }

    // ─── Inverted interval ───────────────────────────────────────────────

    #[test]
    fn eurobond_inverted_negative() {
        // The crate does not reject `end < start`; with the unconditional
        // D2 rule the forward / backward fractions are not strictly
        // negatives of each other when one endpoint has day 31 (because
        // the D1 / D2 rewrites apply asymmetrically). Use mid-month
        // endpoints where no rewrite fires, so the relation is exact.
        //   start = 2026-01-15, end = 2026-07-15
        //   forward  =  180 / 360 =  0.5
        //   backward = -180 / 360 = -0.5
        let start = Date::ymd(2026, 1, 15).unwrap();
        let end = Date::ymd(2026, 7, 15).unwrap();
        let forward = fraction(start, end);
        let backward = fraction(end, start);
        assert!((backward + forward).abs() < TOL);
        assert!(backward < 0.0);
    }

    // ─── Cross-convention demonstration ──────────────────────────────────

    #[test]
    fn eurobond_vs_bond_basis_d2_31_only_diff() {
        // Demonstrates the exact case where 30E/360 and 30/360 Bond Basis
        // disagree by 1/360.
        //
        //   start = 2026-01-15, end = 2026-07-31
        //   D1 = 15 (not in {30, 31}), D2 = 31
        //
        // 30E/360 here:  D2 → 30 unconditionally
        //   numerator = 30*(7-1) + (30-15) = 195
        //   f         = 195 / 360
        //
        // 30/360 Bond Basis on the same inputs:  D2 stays 31 because
        // D1 ∉ {30, 31}
        //   numerator = 30*(7-1) + (31-15) = 196
        //   f         = 196 / 360
        //
        // The test asserts the 30E/360 value and computes the Bond-Basis
        // value by hand on the same line so the gap is auditable.
        let start = Date::ymd(2026, 1, 15).unwrap();
        let end = Date::ymd(2026, 7, 31).unwrap();
        let eurobond = fraction(start, end);
        let bond_basis_hand = 196.0_f64 / 360.0;
        assert!((eurobond - 195.0_f64 / 360.0).abs() < TOL);
        // The two conventions disagree by exactly 1/360 on this input.
        assert!((bond_basis_hand - eurobond - 1.0_f64 / 360.0).abs() < TOL);
    }
}
