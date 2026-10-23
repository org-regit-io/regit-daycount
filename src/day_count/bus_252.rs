// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! Bus/252: the signed business-day count divided by 252.
//!
//! The caller supplies a complete business-day predicate for the interval.
//! This module has no market-calendar default: the calendar must match the
//! instrument's contractual terms. With the `calendars` feature, the calendar
//! module offers coverage-checked Bus/252 calculation for named markets.
//!
//! Forward accruals count dates in `[start, end)`. Reversed accruals negate
//! the count over `[end, start)`, including when an endpoint is a holiday.
//! This normalized reverse policy can differ by one business day from a
//! reference library that traverses backwards with different inclusivity.
//! Equal dates return zero without querying the predicate.
//!
//! The walk is bounded by the valid date endpoints. Every increment remains
//! within their range. Predicate errors cannot be represented by `bool`;
//! callers with fallible calendar queries should use the checked calendar
//! calculation rather than converting unknown dates to business days.
//!
//! References: ANBIMA, *Caderno de Fórmulas — Títulos Públicos Federais*;
//! B3, *Manual de Apreçamento — Derivativos de Renda Fixa*.

use crate::date::Date;

/// Computes the Bus/252 year fraction between two dates.
///
/// The numerator is the count of `d` with `start <= d < end` (or
/// `end <= d < start` for an inverted interval, with the sign flipped)
/// satisfying `is_business_day(d)`; the denominator is the constant 252.
/// The predicate must be defined for every queried date and use the
/// instrument's contractual calendar. See the module documentation for
/// endpoint inclusivity and the reversed-interval policy.
///
/// # Examples
///
/// ```
/// use regit_daycount::{Date, Weekday};
/// use regit_daycount::day_count::bus_252;
///
/// // Weekends-only calendar: Sat and Sun are not business days.
/// let is_biz = |d: Date| {
///     let wd = d.day_of_week();
///     wd != Weekday::Sat && wd != Weekday::Sun
/// };
///
/// // The worked example: 2026-05-01 (Fri) → 2026-05-15 (Fri).
/// // Business days in [2026-05-01, 2026-05-15) are May 1, 4, 5, 6, 7, 8,
/// // 11, 12, 13, 14 — ten in total.
/// let start = Date::ymd(2026, 5, 1).unwrap();
/// let end = Date::ymd(2026, 5, 15).unwrap();
/// let f = bus_252::fraction(start, end, is_biz);
/// assert!((f - 10.0 / 252.0).abs() < 1e-12);
/// ```
#[must_use]
pub fn fraction(start: Date, end: Date, is_business_day: impl Fn(Date) -> bool) -> f64 {
    if start == end {
        return 0.0;
    }
    if start > end {
        return -fraction(end, start, is_business_day);
    }

    // start < end: walk from `start` to `end - 1` inclusive, counting
    // business days. Every increment remains at or before the valid `end`.
    let mut count: u32 = 0;
    let mut d = start;
    while d < end {
        if is_business_day(d) {
            count += 1;
        }
        d = d.add_days(1);
    }
    f64::from(count) / 252.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::date::Weekday;

    // Absolute year-fraction tolerance for these worked examples.
    const TOL: f64 = 1e-12;

    /// Weekend-only predicate: Saturday and Sunday are not business days.
    /// Mirrors the canonical test predicate in `roll.rs`.
    fn weekends_only(d: Date) -> bool {
        let wd = d.day_of_week();
        wd != Weekday::Sat && wd != Weekday::Sun
    }

    // ─── Module-level worked example ─────────────────────────────────────

    #[test]
    fn worked_two_weeks_weekends_only() {
        // start = 2026-05-01 (Fri), end = 2026-05-15 (Fri),
        // predicate = weekends-only.
        // Business days in [2026-05-01, 2026-05-15):
        //   May  1 (Fri), May  4 (Mon), May  5 (Tue), May  6 (Wed),
        //   May  7 (Thu), May  8 (Fri), May 11 (Mon), May 12 (Tue),
        //   May 13 (Wed), May 14 (Thu)
        // = 10 business days.
        // f = 10 / 252 = 0.039682539682540.
        let start = Date::ymd(2026, 5, 1).unwrap();
        let end = Date::ymd(2026, 5, 15).unwrap();
        let f = fraction(start, end, weekends_only);
        assert!((f - 10.0_f64 / 252.0).abs() < TOL);
    }

    // ─── Zero-length interval ────────────────────────────────────────────

    #[test]
    fn zero_length_is_zero() {
        // `fraction(d, d, _)` is identically zero regardless of the
        // predicate — the half-open interval `[d, d)` is empty.
        let d = Date::ymd(2026, 5, 23).unwrap();
        let always_true = |_d: Date| true;
        let always_false = |_d: Date| false;
        assert!(fraction(d, d, always_true).abs() < TOL);
        assert!(fraction(d, d, always_false).abs() < TOL);
    }

    // ─── Inverted interval ───────────────────────────────────────────────

    #[test]
    fn inverted_negates() {
        // The crate's policy on inverted intervals is to return the
        // negation of the forward fraction; Bus/252 follows that policy.
        let start = Date::ymd(2026, 5, 1).unwrap();
        let end = Date::ymd(2026, 5, 15).unwrap();
        let forward = fraction(start, end, weekends_only);
        let backward = fraction(end, start, weekends_only);
        assert!((backward + forward).abs() < TOL);
        assert!(backward < 0.0);
    }

    // ─── Entirely-holidays predicate ─────────────────────────────────────

    #[test]
    fn entirely_holidays_is_zero() {
        // A predicate that never says "business day" — every date in the
        // interval is skipped, so the numerator is 0 and the fraction is
        // exactly 0.0.
        let start = Date::ymd(2026, 5, 1).unwrap();
        let end = Date::ymd(2026, 5, 15).unwrap();
        let always_false = |_d: Date| false;
        let f = fraction(start, end, always_false);
        assert!(f.abs() < TOL);
    }

    // ─── Full calendar year ──────────────────────────────────────────────

    #[test]
    fn full_calendar_year_weekends_only_approx_one() {
        // 2026 is a non-leap year: 365 calendar days.
        // 2026-01-01 is a Thursday → over 365 days starting on a Thursday,
        // weekdays Mon–Fri appear 52 times each plus one extra Thursday
        // (the 365th day = day 1 + 364 = same weekday as start, i.e. Thu;
        // but the interval is half-open, so the last counted day is
        // 2026-12-31 = Thursday). Mon–Fri count = 52 * 5 + 1 = 261.
        // Sat / Sun = 52 each → 104. Total 365. ✓
        // f = 261 / 252 ≈ 1.035714285714286, within ~3.6% of 1.0.
        let start = Date::ymd(2026, 1, 1).unwrap();
        let end = Date::ymd(2027, 1, 1).unwrap();
        let f = fraction(start, end, weekends_only);
        assert!((f - 261.0_f64 / 252.0).abs() < TOL);
        // Sanity: the year approximates 1.0 to within ~3.6%.
        assert!((f - 1.0).abs() < 0.04);
    }

    // ─── Predicate with a holiday on top of weekends ─────────────────────

    #[test]
    fn with_a_christmas_holiday() {
        // Predicate: weekends-only + Fri 2026-12-25 marked as a holiday.
        // Interval [2026-12-21, 2026-12-26):
        //   Dec 21 Mon — business
        //   Dec 22 Tue — business
        //   Dec 23 Wed — business
        //   Dec 24 Thu — business
        //   Dec 25 Fri — holiday (predicate returns false)
        // = 4 business days.
        // f = 4 / 252.
        let weekends_plus_christmas = |d: Date| {
            if d == Date::ymd_unchecked(2026, 12, 25) {
                return false;
            }
            weekends_only(d)
        };
        let start = Date::ymd(2026, 12, 21).unwrap();
        let end = Date::ymd(2026, 12, 26).unwrap();
        let f = fraction(start, end, weekends_plus_christmas);
        assert!((f - 4.0_f64 / 252.0).abs() < TOL);
    }

    #[test]
    fn endpoint_membership_and_reverse_orientation_are_explicit() {
        let friday = Date::ymd(2026, 9, 4).unwrap();
        let saturday = Date::ymd(2026, 9, 5).unwrap();
        // Friday is included, Saturday excluded, in either orientation.
        assert!((fraction(friday, saturday, weekends_only) - 1.0 / 252.0).abs() < TOL);
        assert!((fraction(saturday, friday, weekends_only) + 1.0 / 252.0).abs() < TOL);
    }

    #[test]
    fn valid_boundary_dates_never_require_an_out_of_domain_step() {
        let first = Date::ymd(1583, 1, 1).unwrap();
        let first_next = Date::ymd(1583, 1, 2).unwrap();
        let last_prior = Date::ymd(9999, 12, 30).unwrap();
        let last = Date::ymd(9999, 12, 31).unwrap();
        for (start, end) in [(first, first_next), (last_prior, last)] {
            assert!((fraction(start, end, |_| true) - 1.0 / 252.0).abs() < TOL);
        }
        assert!(
            fraction(last, last, |_| panic!(
                "empty interval must not query predicate"
            ))
            .abs()
                < TOL
        );
    }
}
