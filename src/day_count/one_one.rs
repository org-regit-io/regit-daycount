// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! 1/1 — the OIS-shortcut / legacy day-count convention.
//!
//! 1/1 is the degenerate convention: every interval, regardless of length,
//! direction, or endpoint placement, maps to the year-fraction `1.0`. The
//! function ignores its two arguments entirely — they are accepted only so
//! the signature lines up with every other fraction in this crate's
//! dispatcher, where each arm takes `(start, end)` of type [`Date`].
//!
//! Despite the historical "Act/Act" tagline some terminology lists attach
//! to 1/1, this convention is **not** Act/Act in any meaningful sense:
//! it does not count actual days, it does not divide by an actual year
//! length, and it is not additive over splits. Every interval has fraction
//! `1` flat. The name "1/1" makes the procedure transparent — numerator 1,
//! denominator 1 — and the section heading in the OIS-shortcut literature
//! is preserved here so the convention is searchable by its conventional
//! label rather than by what it numerically does.
//!
//! # Use case — the OIS shortcut
//!
//! The convention exists as a placeholder inside the schedule of an
//! overnight-indexed-swap (OIS) contract. In an OIS, the floating coupon
//! is the geometric compound of the overnight rate over the accrual
//! period; the daily compounding does *all* the time-weighting work
//! internally. Once that compounded rate has been computed, multiplying
//! it by the period's year-fraction would double-count the time
//! dimension. Setting the day-count to 1/1 — so that the multiplier is
//! the identity — is the standard shortcut that keeps the cashflow
//! formula uniform across conventions: `payment = notional * rate *
//! fraction(start, end)` with `fraction ≡ 1` on the OIS leg.
//!
//! # Algorithm
//!
//! ```text
//! fraction(_, _) = 1.0
//! ```
//!
//! # References
//!
//! - ISDA OIS terminology notes (the "1/1" / "Act/Act" placeholder on
//!   compounded-rate legs).

use crate::date::Date;

/// Computes the 1/1 year fraction.
///
/// Returns `1.0` for every input. Both `start` and `end` are accepted for
/// API uniformity with the other day-count fractions in this crate but
/// are not used in the computation — the procedure is constant. See the
/// module-level docstring for the OIS-shortcut motivation and a fuller
/// note on why the result is `1.0` even for the degenerate
/// `fraction(d, d)` and inverted `fraction(end, start)` cases.
///
/// # Examples
///
/// ```
/// use regit_daycount::Date;
/// use regit_daycount::day_count::one_one;
///
/// // The function is identically `1.0` — every interval, every direction.
/// let start = Date::ymd(2026, 1, 1).unwrap();
/// let end   = Date::ymd(2026, 4, 1).unwrap();
/// assert_eq!(one_one::fraction(start, end), 1.0);
/// ```
#[must_use]
pub fn fraction(_start: Date, _end: Date) -> f64 {
    1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    // No `const TOL` here. Every other day-count submodule defines `TOL =
    // 1e-12` because its tests do non-trivial arithmetic; in this module
    // `fraction` returns the literal constant `1.0` with no arithmetic in
    // between, so direct bit-equality is the right assertion and no
    // tolerance constant is needed.

    // ─── Typical interval ────────────────────────────────────────────────

    // `clippy::float_cmp` fires on direct `==` between `f64`s; suppressed
    // for this test because the function returns the literal constant
    // `1.0` with no arithmetic, so exact equality is the right assertion.
    #[allow(clippy::float_cmp)]
    #[test]
    fn always_one_for_typical_dates() {
        // The Act/360 worked-example interval — chosen so the contrast
        // with every other convention (which would return a value near
        // 0.25 here) is maximally visible.
        let start = Date::ymd(2026, 1, 1).unwrap();
        let end = Date::ymd(2026, 4, 1).unwrap();
        assert_eq!(fraction(start, end), 1.0);
    }

    // ─── Zero-length interval ────────────────────────────────────────────

    // `clippy::float_cmp` is suppressed for the same reason: the function
    // is a constant, exact equality is correct.
    #[allow(clippy::float_cmp)]
    #[test]
    fn always_one_for_zero_length() {
        // This is the load-bearing point of the 1/1 convention: a
        // same-date interval still returns `1.0`, not `0.0`. Every other
        // fraction in the crate is `0.0` on `(d, d)`; 1/1 is degenerate
        // by design.
        let d = Date::ymd(2026, 5, 23).unwrap();
        assert_eq!(fraction(d, d), 1.0);
    }

    // ─── Inverted interval ───────────────────────────────────────────────

    // `clippy::float_cmp` is suppressed for the same reason: the function
    // is a constant, exact equality is correct.
    #[allow(clippy::float_cmp)]
    #[test]
    fn always_one_for_inverted() {
        // Other fractions negate on swap (`fraction(end, start) ==
        // -fraction(start, end)`); 1/1 does not. The inverted call still
        // returns `1.0` — degenerate by design.
        let start = Date::ymd(2026, 1, 1).unwrap();
        let end = Date::ymd(2026, 4, 1).unwrap();
        assert_eq!(fraction(end, start), 1.0);
    }

    // ─── Extreme dates ───────────────────────────────────────────────────

    // `clippy::float_cmp` is suppressed for the same reason: the function
    // is a constant, exact equality is correct.
    #[allow(clippy::float_cmp)]
    #[test]
    fn always_one_for_extreme_dates() {
        // The widest interval the supported year range [1583, 9999]
        // admits — confirms the constant survives an 8,000-year span
        // identically to a 90-day one.
        let start = Date::ymd(1583, 1, 1).unwrap();
        let end = Date::ymd(9999, 12, 31).unwrap();
        assert_eq!(fraction(start, end), 1.0);
    }

    // ─── Exactness ───────────────────────────────────────────────────────

    // `clippy::float_cmp` is suppressed: the assertion below is the
    // explicit point of the test — bit-exact equality with `1.0_f64`.
    #[allow(clippy::float_cmp)]
    #[test]
    fn result_is_exact_one_not_just_approximately() {
        // The function returns the literal constant `1.0`; no arithmetic
        // happens between the constant and the assertion, so `==` is the
        // correct comparison here. This is the one place in the crate
        // where bit-exact float equality is the right assertion.
        let start = Date::ymd(2026, 1, 1).unwrap();
        let end = Date::ymd(2026, 7, 15).unwrap();
        let f = fraction(start, end);
        assert_eq!(f, 1.0_f64);
        // And the bit pattern is the canonical one for `1.0`.
        assert_eq!(f.to_bits(), 1.0_f64.to_bits());
    }
}
