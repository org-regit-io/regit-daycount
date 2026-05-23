// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! Bus/252 — Brazilian business-day convention (ANBIMA / B3).
//!
//! Bus/252 is the day-count fraction used for almost every BRL-denominated
//! fixed-income instrument: federal CDI / Selic-indexed notes, ANBIMA
//! reference yields, B3-listed DI futures, and the corporate debentures
//! priced off them. The numerator is the count of **business days** in the
//! half-open interval `[start, end)` under the caller-supplied calendar; the
//! denominator is the fixed constant 252 — the conventional Brazilian
//! business-year length (roughly 365 calendar days minus 104 weekend days
//! minus the federal / state holidays B3 observes).
//!
//! # API shape — closure predicate
//!
//! Every other fraction in this crate has the signature `fn fraction(start:
//! Date, end: Date) -> f64`. Bus/252 cannot — by definition it needs to know
//! which dates are business days, and that knowledge lives in
//! `crate::calendar`. Importing `calendar` here would close the cycle
//! `day_count::bus_252 → calendar → day_count` (calendars compose with
//! year-fraction queries through the dispatcher), so the signature instead
//! takes a caller-supplied `is_business_day: impl Fn(Date) -> bool`
//! predicate. In practice the caller passes `|d| calendar::is_business_day
//! (d, cal)` for their chosen `cal`; `bus_252` itself just counts.
//!
//! This is the same pattern [`crate::roll::apply`] uses for the same
//! reason — see `roll.rs` for the canonical statement.
//!
//! # Algorithm
//!
//! ```text
//! For start <= end:
//!     count = number of dates d with start <= d < end and is_business_day(d)
//!     return count as f64 / 252.0
//! For start > end:
//!     return -fraction(end, start, is_business_day)
//! ```
//!
//! The walk is one day at a time, bounded defensively at `MAX_DAYS` steps
//! — roughly two centuries — to keep a pathological interval from looping
//! indefinitely; no real fixed-income instrument crosses that horizon.
//!
//! # Worked example
//!
//! ```text
//! start = 2026-05-01 (Fri), end = 2026-05-15 (Fri), weekends-only calendar
//! business days in [2026-05-01, 2026-05-15):
//!     May 1 Fri, 4 Mon, 5 Tue, 6 Wed, 7 Thu, 8 Fri,
//!     11 Mon, 12 Tue, 13 Wed, 14 Thu = 10 days
//! f = 10 / 252 = 0.039682539682540
//! ```
//!
//! # References
//!
//! - ANBIMA, *Caderno de Fórmulas — Títulos Públicos Federais*
//!   (Bus/252 numerator definition for LTN, NTN-B, NTN-F).
//! - B3, *Manual de Apreçamento — Derivativos de Renda Fixa*
//!   (Bus/252 denominator = 252 for DI futures).

use crate::date::Date;

/// Maximum number of one-day steps the counter walks before bailing out.
///
/// Sized for ~200 years (`365 * 200 = 73_000`), which covers every term
/// structure of any real BRL instrument by orders of magnitude. If the cap
/// is ever reached — a pathological interval far outside any practical use
/// — the running count at that point is returned, divided by 252 as usual;
/// the function never panics and never loops indefinitely.
const MAX_DAYS: u32 = 73_000;

/// Computes the Bus/252 year fraction between two dates.
///
/// The numerator is the count of `d` with `start <= d < end` (or
/// `end <= d < start` for an inverted interval, with the sign flipped)
/// satisfying `is_business_day(d)`; the denominator is the constant 252.
/// See the module-level docstring for the full algorithm, the rationale for
/// the closure-predicate signature, and the worked example.
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
    // business days. The loop is bounded defensively at `MAX_DAYS` steps.
    let mut count: u32 = 0;
    let mut d = start;
    let mut steps: u32 = 0;
    while d < end {
        if steps >= MAX_DAYS {
            break;
        }
        if is_business_day(d) {
            count += 1;
        }
        d = d.add_days(1);
        steps += 1;
    }
    f64::from(count) / 252.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::date::Weekday;

    // The tolerance 1e-12 is the working slack used throughout this crate's
    // numeric assertions: it is loose enough that a benign last-bit rounding
    // never causes a spurious failure, and tight enough that any drift large
    // enough to mis-state a cashflow at the cent level (≥ 1e-9 on a unit
    // notional) is caught immediately. We deliberately do not assert against
    // `f64::EPSILON`; the goal is auditable, not maximally-tight.
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
}
