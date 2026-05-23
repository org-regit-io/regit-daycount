// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! 30E/360 ISDA — ISDA 2006 §4.16(h).
//!
//! 30E/360 ISDA is the maturity-day-aware sibling of 30E/360. The
//! 30-day-month / 360-day-year arithmetic is identical to that of the
//! plain 30E/360 fraction (§4.16(g)), but the day-of-month adjustments
//! are governed by a different rule: a day is collapsed to 30 when it is
//! the **last calendar day of its month** — not merely when it is the
//! 31st. So a 28 February in a non-leap year, a 29 February in a leap
//! year, and a 30 April all qualify, where the plain 30E/360 fraction
//! adjusts only the four 31-day-month endings.
//!
//! On top of that, the convention carries one deliberate carve-out: when
//! the end date is the **maturity date** of the instrument *and* the end
//! month is February, the end-day adjustment is **suppressed** — `D2`
//! is left at the actual last day of February (28 or 29). This is the
//! load-bearing case of the convention; it is the entire reason §4.16(h)
//! exists alongside §4.16(g). Mishandling it silently over- or
//! under-accrues the final coupon of every February-maturing instrument
//! that quotes on 30E/360 ISDA — a 1/360 to 2/360 absolute drift in the
//! year fraction, well above any oracle-matching tolerance.
//!
//! # Signature
//!
//! Unlike every other fraction in this catalogue, the entry point takes
//! an additional flag, `end_is_maturity`, because the convention is not
//! a pure function of `(start, end)`. The flag is the caller's
//! responsibility — it is `true` if and only if `end` is the maturity
//! date of the instrument the fraction is being computed for.
//!
//! # Algorithm
//!
//! With `(Y1, M1, D1)` from `start` and `(Y2, M2, D2)` from `end`:
//!
//! ```text
//! 1. If D1 is the last day of month M1 in year Y1, set D1 = 30.
//!    (i.e. D1 == days_in_month(Y1, M1))
//! 2. If D2 is the last day of month M2 in year Y2,
//!    AND NOT (M2 == 2 AND end_is_maturity),
//!    set D2 = 30.
//! 3. Numerator = 360 * (Y2 - Y1) + 30 * (M2 - M1) + (D2 - D1)
//! 4. f = Numerator / 360.0
//! ```
//!
//! All arithmetic is performed in `i32` and converted to `f64` for the
//! final division; the intermediate values are bounded by the supported
//! year range and cannot overflow.
//!
//! # Worked example — maturity-day suppression
//!
//! ```text
//! start = 2024-01-31, end = 2025-02-28, end_is_maturity = true
//!
//! D1 = 31 = days_in_month(2024, 1)   → D1 = 30
//! D2 = 28 = days_in_month(2025, 2)
//!       AND M2 == 2 AND end_is_maturity → SUPPRESS, D2 stays 28
//!
//! Numerator = 360 * (2025 - 2024) + 30 * (2 - 1) + (28 - 30)
//!           = 360 + 30 - 2
//!           = 388
//! f         = 388 / 360 = 1.077777777777778
//! ```
//!
//! Re-computing the same dates with `end_is_maturity = false` instead
//! takes `D2` to 30 and yields `390 / 360 = 1.083333333333333` — the
//! 2/360 swing the maturity-day suppression is designed to introduce.
//!
//! # Inverted intervals
//!
//! `fraction(end, start, _)` for `start < end` produces a negative
//! result by the natural arithmetic of the formula above — the
//! `(Y2 - Y1)`, `(M2 - M1)`, and `(D2 - D1)` differences all flip sign
//! together. No special case is needed in the body; the crate as a whole
//! does not reject inverted intervals (some callers compute
//! reverse-period accruals) and this module mirrors that behaviour.
//!
//! # References
//!
//! - ISDA 2006 Definitions §4.16(h), *30E/360 (ISDA)*.

use crate::date::Date;

/// Computes the 30E/360 ISDA year fraction between two dates.
///
/// `end_is_maturity` is `true` if and only if `end` is the maturity
/// date of the instrument the fraction is being computed for; it
/// governs the February-suppression carve-out described in the
/// module-level docstring.
///
/// See the module-level docstring for the full algorithm and worked
/// examples.
///
/// # Examples
///
/// The same `(start, end)` pair yields two different fractions
/// depending on the maturity flag — the entire reason this convention
/// is distinct from 30E/360:
///
/// ```
/// use regit_daycount::Date;
/// use regit_daycount::day_count::thirty_e_360_isda;
///
/// let start = Date::ymd(2024, 1, 31).unwrap();
/// let end   = Date::ymd(2025, 2, 28).unwrap();
///
/// // At maturity: D2 == 28 is the last day of February AND end is the
/// // maturity date AND M2 == 2, so the end-day adjustment is
/// // suppressed; D2 stays 28. Numerator = 360 + 30 - 2 = 388.
/// let at_maturity = thirty_e_360_isda::fraction(start, end, true);
/// assert!((at_maturity - 388.0_f64 / 360.0).abs() < 1e-12);
///
/// // Not at maturity: D2 == 28 is still the last day of February but
/// // the suppression carve-out does not apply, so D2 is taken to 30.
/// // Numerator = 360 + 30 + 0 = 390.
/// let mid_period = thirty_e_360_isda::fraction(start, end, false);
/// assert!((mid_period - 390.0_f64 / 360.0).abs() < 1e-12);
///
/// // The two differ by exactly 2/360 — the load-bearing swing.
/// assert!((mid_period - at_maturity - 2.0_f64 / 360.0).abs() < 1e-12);
/// ```
#[must_use]
pub fn fraction(start: Date, end: Date, end_is_maturity: bool) -> f64 {
    let y1 = start.year();
    let m1 = start.month();
    let mut d1 = start.day();
    let y2 = end.year();
    let m2 = end.month();
    let mut d2 = end.day();

    // Step 1: collapse D1 to 30 if it is the last day of its month.
    // `days_in_month` returns 28/29/30/31 for any valid (year, month),
    // so equality is exact.
    if d1 == Date::days_in_month(y1, m1) {
        d1 = 30;
    }

    // Step 2: collapse D2 to 30 if it is the last day of its month —
    // UNLESS the end month is February AND `end` is the maturity date,
    // in which case the adjustment is suppressed and D2 stays at the
    // actual last day of February (28 or 29).
    if d2 == Date::days_in_month(y2, m2) && !(m2 == 2 && end_is_maturity) {
        d2 = 30;
    }

    // Step 3: numerator in days. All terms fit comfortably in `i32`
    // for years in the supported range [1583, 9999]: the year term is
    // bounded by 360 * 8416 ≈ 3 × 10⁶, the month term by 30 * 11 = 330,
    // and the day term by ±30.
    let numerator =
        360 * (y2 - y1) + 30 * (i32::from(m2) - i32::from(m1)) + (i32::from(d2) - i32::from(d1));

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

    // ─── D1-and-D2 both last-day, neither at maturity ────────────────────

    #[test]
    fn isda_aug_31_no_maturity_adjusts() {
        // start = 2024-02-29 (leap), end = 2024-08-31, end_is_maturity=false.
        //   D1 = 29 = days_in_month(2024, 2) = 29 → D1 = 30.
        //   D2 = 31 = days_in_month(2024, 8) = 31 (M2 = 8 ≠ 2) → D2 = 30.
        //   Numerator = 360*0 + 30*(8-2) + (30-30) = 180.
        //   f = 180 / 360 = 0.5.
        let start = Date::ymd(2024, 2, 29).unwrap();
        let end = Date::ymd(2024, 8, 31).unwrap();
        let f = fraction(start, end, false);
        assert!((f - 180.0_f64 / 360.0).abs() < 1e-15);
        assert!((f - 0.5).abs() < TOL);
    }

    // ─── February-at-maturity suppression — the load-bearing case ────────

    #[test]
    fn isda_feb_28_at_maturity_does_not_adjust_d2() {
        // start = 2024-01-31, end = 2025-02-28, MATURITY.
        //   D1 = 31 = days_in_month(2024, 1) = 31 → D1 = 30.
        //   D2 = 28 = days_in_month(2025, 2) = 28, AND M2 = 2 AND
        //   end_is_maturity → SUPPRESS, D2 stays 28.
        //   Numerator = 360 + 30*(2-1) + (28-30) = 360 + 30 - 2 = 388.
        //   f = 388 / 360 ≈ 1.077777777777778.
        let start = Date::ymd(2024, 1, 31).unwrap();
        let end = Date::ymd(2025, 2, 28).unwrap();
        let f = fraction(start, end, true);
        assert!((f - 388.0_f64 / 360.0).abs() < TOL);
    }

    #[test]
    fn isda_feb_28_not_at_maturity_does_adjust_d2() {
        // Same dates as above but end_is_maturity = false. Now the
        // February-suppression carve-out does NOT apply.
        //   D1 = 31 → 30 (as above).
        //   D2 = 28 = days_in_month(2025, 2) → D2 = 30.
        //   Numerator = 360 + 30*(2-1) + (30-30) = 390.
        //   f = 390 / 360 ≈ 1.083333333333333.
        let start = Date::ymd(2024, 1, 31).unwrap();
        let end = Date::ymd(2025, 2, 28).unwrap();
        let f = fraction(start, end, false);
        assert!((f - 390.0_f64 / 360.0).abs() < TOL);
    }

    #[test]
    fn isda_feb_29_at_maturity_in_leap_does_not_adjust() {
        // start = 2023-02-28, end = 2024-02-29 (leap), MATURITY.
        //   D1 = 28 = days_in_month(2023, 2) → D1 = 30.
        //   D2 = 29 = days_in_month(2024, 2) (leap), AND M2 = 2 AND
        //   end_is_maturity → SUPPRESS, D2 stays 29.
        //   Numerator = 360 + 30*0 + (29-30) = 359.
        //   f = 359 / 360 ≈ 0.997222222222222.
        let start = Date::ymd(2023, 2, 28).unwrap();
        let end = Date::ymd(2024, 2, 29).unwrap();
        let f = fraction(start, end, true);
        assert!((f - 359.0_f64 / 360.0).abs() < TOL);
    }

    // ─── No-adjustment path ──────────────────────────────────────────────

    #[test]
    fn isda_d1_not_last_day_no_adjust() {
        // start = 2026-01-15, end = 2026-07-15, not maturity. Neither
        // endpoint is the last day of its month, so no adjustment fires.
        //   Numerator = 360*0 + 30*(7-1) + (15-15) = 180.
        //   f = 180 / 360 = 0.5.
        let start = Date::ymd(2026, 1, 15).unwrap();
        let end = Date::ymd(2026, 7, 15).unwrap();
        let f = fraction(start, end, false);
        assert!((f - 180.0_f64 / 360.0).abs() < 1e-15);
        assert!((f - 0.5).abs() < TOL);
    }

    // ─── Zero-length interval ────────────────────────────────────────────

    #[test]
    fn isda_zero_length() {
        // `fraction(d, d, _)` is identically zero — degenerate but legal.
        // Holds for both flag values, since the numerator is zero regardless.
        let d = Date::ymd(2026, 5, 23).unwrap();
        assert!(fraction(d, d, false).abs() < TOL);
        assert!(fraction(d, d, true).abs() < TOL);
    }

    // ─── Inverted interval ───────────────────────────────────────────────

    #[test]
    fn isda_inverted_negative() {
        // The crate does not reject `end < start`; the numerator flips
        // sign by the natural arithmetic of the formula. Picks a pair
        // where neither endpoint is a last day of its month, so the
        // adjustments do not introduce asymmetry between the forward and
        // reverse evaluations.
        let start = Date::ymd(2026, 1, 15).unwrap();
        let end = Date::ymd(2026, 7, 15).unwrap();
        let forward = fraction(start, end, false);
        let backward = fraction(end, start, false);
        assert!((backward + forward).abs() < TOL);
        assert!(backward < 0.0);
    }

    // ─── D1 = last day of February in a leap year ────────────────────────

    #[test]
    fn isda_d1_feb_29_in_leap_year_adjusts() {
        // start = 2024-02-29, end = 2024-04-15, not maturity.
        //   D1 = 29 = days_in_month(2024, 2) → D1 = 30.
        //   D2 = 15, not a last day → stays.
        //   Numerator = 360*0 + 30*(4-2) + (15-30) = 60 - 15 = 45.
        //   f = 45 / 360 = 0.125.
        let start = Date::ymd(2024, 2, 29).unwrap();
        let end = Date::ymd(2024, 4, 15).unwrap();
        let f = fraction(start, end, false);
        assert!((f - 45.0_f64 / 360.0).abs() < TOL);
        assert!((f - 0.125).abs() < TOL);
    }
}
