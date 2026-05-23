// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! 30/360 Bond Basis — ISDA 2006 §4.16(f).
//!
//! 30/360 Bond Basis is the US bond-market default: every month is treated
//! as 30 days long and every year as 360 days, with two specific
//! day-of-month adjustments applied to the endpoints before differencing.
//! The convention underpins coupon accrual on US Treasury notes and
//! bonds, US agency debt, and most US corporate fixed-rate bonds; it is
//! also the historic basis for the swap-leg "30/360" tag in pre-ISDA
//! confirmations.
//!
//! # Algorithm
//!
//! Let `(Y1, M1, D1)` and `(Y2, M2, D2)` be the year, month, and day of
//! `start` and `end` respectively.
//!
//! ```text
//! 1. If D1 == 31, set D1 = 30.
//! 2. If D2 == 31 AND D1 in {30, 31}, set D2 = 30.
//!    (Equivalent after step 1: D2 == 31 AND D1 == 30.)
//! 3. Numerator = 360 * (Y2 - Y1) + 30 * (M2 - M1) + (D2 - D1)
//! 4. f = Numerator / 360.0
//! ```
//!
//! All endpoint arithmetic is performed in `i32`; the cast to `f64`
//! happens only at the final division. The numerator is signed, so an
//! inverted interval (`end < start`) yields a negative fraction.
//!
//! # `BondBasis` vs 30E/360
//!
//! 30/360 Bond Basis and 30E/360 (ISDA 2006 §4.16(g), the "Eurobond"
//! variant) share the `D1 == 31 → 30` step but differ in their treatment
//! of `D2`. For 30/360 Bond Basis, the `D2 == 31 → 30` adjustment is
//! **conditional on `D1`**: it fires only when `D1` is already 30 or 31
//! (equivalently, only when `D1` was 30 or 31 *before* step 1 collapsed
//! 31 to 30). For 30E/360, the same adjustment is **unconditional** —
//! every `D2 == 31` collapses to 30 regardless of `D1`. The two
//! conventions therefore disagree on intervals such as `2026-01-15` to
//! `2026-07-31`: `BondBasis` keeps `D2 = 31` (numerator 196), 30E/360
//! collapses it to 30 (numerator 195).
//!
//! # Worked example — 6-month mid-month interval
//!
//! ```text
//! start = 2026-01-15
//! end   = 2026-07-15
//! D1 = 15 (no change), D2 = 15 (no change)
//! Numerator = 360*0 + 30*(7-1) + (15-15) = 180
//! f         = 180 / 360 = 0.500000000000000
//! ```
//!
//! # References
//!
//! - ISDA 2006 Definitions §4.16(f), *30/360* (also known as "Bond Basis").

use crate::date::Date;

/// Computes the 30/360 Bond Basis year fraction between two dates.
///
/// Applies the two endpoint adjustments from ISDA 2006 §4.16(f) — the
/// unconditional `D1 == 31 → 30` and the `D1`-conditional
/// `D2 == 31 → 30` — and returns the signed numerator divided by 360.
/// See the module-level docstring for the full algorithm, the
/// `BondBasis` vs 30E/360 distinction, and a worked example.
///
/// # Examples
///
/// ```
/// use regit_daycount::Date;
/// use regit_daycount::day_count::thirty_360_bond_basis;
///
/// // 6-month mid-month interval — the §A.5 worked example.
/// let start = Date::ymd(2026, 1, 15).unwrap();
/// let end   = Date::ymd(2026, 7, 15).unwrap();
/// assert!((thirty_360_bond_basis::fraction(start, end) - 0.5).abs() < 1e-12);
/// ```
#[must_use]
pub fn fraction(start: Date, end: Date) -> f64 {
    // Endpoint arithmetic in `i32`; widening from the storage `u8` / `i32`
    // fields is exact. The two day-of-month adjustments below mutate
    // local copies only — the input `Date` values are untouched.
    let y1 = start.year();
    let m1 = i32::from(start.month());
    let mut d1 = i32::from(start.day());
    let y2 = end.year();
    let m2 = i32::from(end.month());
    let mut d2 = i32::from(end.day());

    // Step 1: collapse D1 = 31 to 30 unconditionally.
    if d1 == 31 {
        d1 = 30;
    }
    // Step 2: collapse D2 = 31 to 30 only if D1 was 30 or 31. After
    // step 1 has run, `d1 == 30` covers both the "D1 was 30" and
    // "D1 was 31, now 30" branches — the two are indistinguishable here
    // and the ISDA rule treats them identically. This is the
    // conditionality that distinguishes BondBasis from 30E/360.
    if d2 == 31 && d1 == 30 {
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

    // ─── 6-month mid-month — no adjustments ──────────────────────────────

    #[test]
    fn bond_basis_6m_mid_month() {
        // doc/ALGORITHMS.md §A.5 worked example:
        //   start = 2026-01-15, end = 2026-07-15
        //   D1 = 15 (no change), D2 = 15 (no change)
        //   Numerator = 30*6 + 0 = 180
        //   f         = 180 / 360 = 0.5
        let start = Date::ymd(2026, 1, 15).unwrap();
        let end = Date::ymd(2026, 7, 15).unwrap();
        let f = fraction(start, end);
        assert!((f - 0.5).abs() < TOL);
    }

    // ─── D1 = 31 collapses to 30 ─────────────────────────────────────────

    #[test]
    fn bond_basis_d1_31_adjusts_to_30() {
        // start = 2026-01-31, end = 2026-02-28
        // D1: 31 → 30. D2 = 28 (not 31), stays.
        // Numerator = 360*0 + 30*(2-1) + (28-30) = 30 - 2 = 28
        // f         = 28 / 360
        let start = Date::ymd(2026, 1, 31).unwrap();
        let end = Date::ymd(2026, 2, 28).unwrap();
        let f = fraction(start, end);
        assert!((f - 28.0_f64 / 360.0).abs() < TOL);
    }

    // ─── D2 = 31 with D1 = 30 ⇒ D2 adjusts ───────────────────────────────

    #[test]
    fn bond_basis_d2_31_with_d1_30_adjusts() {
        // start = 2026-01-30, end = 2026-07-31
        // D1 = 30 (no change). D2 = 31, D1 = 30 ⇒ D2 → 30.
        // Numerator = 360*0 + 30*(7-1) + (30-30) = 180
        // f         = 180 / 360 = 0.5
        let start = Date::ymd(2026, 1, 30).unwrap();
        let end = Date::ymd(2026, 7, 31).unwrap();
        let f = fraction(start, end);
        assert!((f - 0.5).abs() < TOL);
    }

    // ─── D2 = 31 with D1 = 15 ⇒ no adjustment (the BondBasis case) ───────

    #[test]
    fn bond_basis_d2_31_with_d1_15_no_adjust() {
        // This is the load-bearing test that distinguishes BondBasis from
        // 30E/360. Under 30E/360 the D2 = 31 → 30 step is unconditional
        // and the fraction would be 195/360; under BondBasis the step
        // requires D1 ∈ {30, 31}, so D2 stays at 31.
        //   start = 2026-01-15, end = 2026-07-31
        //   D1 = 15 (no change). D2 = 31, but D1 = 15 ⇒ NO adjust.
        //   Numerator = 360*0 + 30*(7-1) + (31-15) = 180 + 16 = 196
        //   f         = 196 / 360
        let start = Date::ymd(2026, 1, 15).unwrap();
        let end = Date::ymd(2026, 7, 31).unwrap();
        let f = fraction(start, end);
        assert!((f - 196.0_f64 / 360.0).abs() < TOL);
    }

    // ─── Both endpoints on day 31 ────────────────────────────────────────

    #[test]
    fn bond_basis_d1_31_d2_31() {
        // start = 2026-01-31, end = 2026-07-31
        // D1: 31 → 30. D2 = 31, D1 (now 30) satisfies the rule ⇒ D2 → 30.
        // Numerator = 360*0 + 30*(7-1) + (30-30) = 180
        // f         = 180 / 360 = 0.5
        let start = Date::ymd(2026, 1, 31).unwrap();
        let end = Date::ymd(2026, 7, 31).unwrap();
        let f = fraction(start, end);
        assert!((f - 0.5).abs() < TOL);
    }

    // ─── Zero-length interval ────────────────────────────────────────────

    #[test]
    fn bond_basis_zero_length() {
        // `fraction(d, d)` is identically zero — a degenerate but legal
        // input; callers occasionally hit it on a same-day reset.
        let d = Date::ymd(2026, 5, 23).unwrap();
        let f = fraction(d, d);
        assert!(f.abs() < TOL);
    }

    // ─── Inverted interval ───────────────────────────────────────────────

    #[test]
    fn bond_basis_inverted_is_negative() {
        // The crate does not reject `end < start`; the BondBasis numerator
        // is signed, so the fraction is the negation of the forward case.
        let start = Date::ymd(2026, 1, 15).unwrap();
        let end = Date::ymd(2026, 7, 15).unwrap();
        let forward = fraction(start, end);
        let backward = fraction(end, start);
        assert!((backward + forward).abs() < TOL);
        assert!(backward < 0.0);
    }

    // ─── February 28 is not 31 ⇒ no adjustment ───────────────────────────

    #[test]
    fn bond_basis_does_not_adjust_feb_28() {
        // The BondBasis adjustments only fire on D == 31, never on a
        // short-month end-of-month like 28 Feb. So a clean 6-month span
        // ending on 28 Feb returns the exact 0.5 you would expect.
        //   start = 2026-02-28, end = 2026-08-28
        //   D1 = 28 (not 31), D2 = 28 (not 31). No changes.
        //   Numerator = 360*0 + 30*(8-2) + (28-28) = 180
        //   f         = 180 / 360 = 0.5
        let start = Date::ymd(2026, 2, 28).unwrap();
        let end = Date::ymd(2026, 8, 28).unwrap();
        let f = fraction(start, end);
        assert!((f - 0.5).abs() < TOL);
    }
}
