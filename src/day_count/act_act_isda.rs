// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! Act/Act ISDA — ISDA 2006 §4.16(b).
//!
//! Actual/Actual (ISDA) is the leap-year-aware day-count fraction used by
//! every standard ISDA-documented interest-rate swap whose floating leg
//! references an "Actual/Actual" basis. Unlike Act/360 and Act/365F — both
//! of which use a single, fixed denominator — Act/Act ISDA splits the
//! calculation period at every January 1 it crosses and divides each
//! sub-period's actual day count by 366 if its containing calendar year is
//! a Gregorian leap year and 365 otherwise. The sub-fractions are summed.
//! The construction is the one specified by the standard verbatim and is
//! the one printed in the ISDA 2006 Definitions appendix.
//!
//! # Algorithm
//!
//! A same-year interval is its actual day count divided by that year's
//! length. For a multi-year interval, add the fraction of the first year,
//! one for each intervening full year, and the fraction of the last year.
//! All subintervals include their start and exclude their end. This requires
//! only two year boundaries, both within the supported date range.
//!
//! # Inverted intervals
//!
//! `fraction(end, start)` for `start < end` returns the negation of the
//! forward fraction. The crate as a whole does not reject inverted
//! intervals — some callers compute reverse-period accruals — and this
//! module mirrors Act/360's behaviour by computing `-fraction(end, start)`
//! when `start > end`. Equality `fraction(end, start) == -fraction(start,
//! end)` is asserted in the test suite.
//!
//! # Worked example — ISDA 2006 Definitions, §4.16(b)
//!
//! ```text
//! start = 2007-12-28
//! end   = 2008-02-29
//!
//! Days in 2007: 2007-12-28 → 2008-01-01 (exclusive) = 4
//!               (28, 29, 30, 31 December). 2007 is not a leap year.
//! Days in 2008: 2008-01-01 → 2008-02-29 (exclusive) = 31 + 28 = 59.
//!               2008 is a leap year.
//!
//! f = 4 / 365 + 59 / 366
//!   = 0.010958904109589 + 0.161202185792350
//!   = 0.172161089901939
//! ```
//!
//! # References
//!
//! - ISDA 2006 Definitions §4.16(b), *Actual/Actual* / *Actual/Actual
//!   (ISDA)* / *Act/Act* / *Act/Act (ISDA)*.

use crate::date::Date;

/// Computes the Act/Act ISDA year fraction between two dates.
///
/// The same-year case is fast-pathed: the whole interval is divided by
/// 366 or 365 depending on whether the year is a Gregorian leap year. The
/// multi-year case adds the first partial year, intervening full years,
/// and the last partial year. Inverted
/// intervals (`start > end`) return the negation of the forward fraction;
/// see the module-level docstring for the rationale.
///
/// # Examples
///
/// ```
/// use regit_daycount::Date;
/// use regit_daycount::day_count::act_act_isda;
///
/// // The ISDA 2006 Definitions printed worked example.
/// let start = Date::ymd(2007, 12, 28).unwrap();
/// let end   = Date::ymd(2008,  2, 29).unwrap();
/// let f     = act_act_isda::fraction(start, end);
/// let want  = 4.0_f64 / 365.0 + 59.0_f64 / 366.0;
/// assert!((f - want).abs() < 1e-12);
/// ```
#[must_use]
pub fn fraction(start: Date, end: Date) -> f64 {
    // Inverted interval: mirror Act/360 by negating the forward result.
    // The crate does not reject `end < start`; some callers compute
    // reverse-period accruals.
    if start > end {
        return -fraction(end, start);
    }
    // Degenerate same-day interval — exact zero, no division needed.
    if start == end {
        return 0.0;
    }

    let start_year = start.year();
    let end_year = end.year();

    // A same-year interval has one denominator.
    if start_year == end_year {
        let d = f64::from(start.days_between(end));
        let denom = if Date::is_leap_year(start_year) {
            366.0_f64
        } else {
            365.0_f64
        };
        return d / denom;
    }

    // start_year < end_year, so both constructed dates lie within the
    // valid Date range, including when the accrual ends in year 9999.
    let first_year_end = Date::ymd_unchecked(start_year + 1, 1, 1);
    let last_year_start = Date::ymd_unchecked(end_year, 1, 1);
    let first_denominator = if Date::is_leap_year(start_year) {
        366.0
    } else {
        365.0
    };
    let last_denominator = if Date::is_leap_year(end_year) {
        366.0
    } else {
        365.0
    };
    let complete_years = end_year - start_year - 1;
    f64::from(complete_years)
        + f64::from(start.days_between(first_year_end)) / first_denominator
        + f64::from(last_year_start.days_between(end)) / last_denominator
}

#[cfg(test)]
mod tests {
    use super::*;

    // Absolute year-fraction tolerance for these worked examples.
    const TOL: f64 = 1e-12;

    // ─── ISDA worked example ─────────────────────────────────────────────

    #[test]
    fn isda_worked_example_2007_to_2008() {
        // doc/ALGORITHMS.md §A.3 worked example — the printed ISDA 2006
        // Definitions example:
        //   start = 2007-12-28, end = 2008-02-29
        //   Days in 2007: 4 (28, 29, 30, 31 Dec). 2007 non-leap → /365.
        //   Days in 2008: 59 (31 Jan + 28 Feb up to but excluding 29 Feb).
        //                 2008 leap → /366.
        //   f = 4/365 + 59/366 = 0.172161089901939
        let start = Date::ymd(2007, 12, 28).unwrap();
        let end = Date::ymd(2008, 2, 29).unwrap();
        let f = fraction(start, end);
        let want = 4.0_f64 / 365.0 + 59.0_f64 / 366.0;
        assert!(
            (f - want).abs() < TOL,
            "ISDA worked example: got {f}, want {want}",
        );
        // And the printed 9-dp number itself, to the same tolerance.
        assert!((f - 0.172_161_089_901_939).abs() < TOL);
    }

    // ─── Same-year cases ─────────────────────────────────────────────────

    #[test]
    fn same_year_non_leap_simplifies() {
        // 2026 is not a leap year, so the whole interval divides by 365.
        // 2026-03-01 → 2026-09-01 (exclusive): 31 (Mar) + 30 (Apr) +
        //   31 (May) + 30 (Jun) + 31 (Jul) + 31 (Aug) = 184.
        let start = Date::ymd(2026, 3, 1).unwrap();
        let end = Date::ymd(2026, 9, 1).unwrap();
        assert_eq!(start.days_between(end), 184);
        let f = fraction(start, end);
        assert!((f - 184.0_f64 / 365.0).abs() < TOL);
    }

    #[test]
    fn same_year_leap_uses_366() {
        // 2024 is a leap year, so the same March–September interval
        // divides by 366. The numerator is still 184 because the
        // interval does not include 29 February.
        let start = Date::ymd(2024, 3, 1).unwrap();
        let end = Date::ymd(2024, 9, 1).unwrap();
        assert_eq!(start.days_between(end), 184);
        let f = fraction(start, end);
        assert!((f - 184.0_f64 / 366.0).abs() < TOL);
    }

    // ─── Zero-length interval ────────────────────────────────────────────

    #[test]
    fn zero_length_interval_is_zero() {
        // `fraction(d, d)` is identically zero — a degenerate but legal
        // input; callers occasionally hit it on a same-day reset. The
        // implementation fast-paths this without dividing, but we still
        // assert through the same tolerance the rest of the suite uses.
        let d = Date::ymd(2026, 5, 23).unwrap();
        let f = fraction(d, d);
        assert!(f.abs() < TOL);
    }

    // ─── Inverted interval ───────────────────────────────────────────────

    #[test]
    fn inverted_interval_is_negative() {
        // The crate does not reject `end < start`; the result is the
        // negation of the forward fraction. We mirror Act/360 here so
        // that callers computing reverse-period accruals get a clean
        // sign-flip and exact additivity around the swap of endpoints.
        let start = Date::ymd(2023, 6, 1).unwrap();
        let end = Date::ymd(2025, 6, 1).unwrap();
        let forward = fraction(start, end);
        let backward = fraction(end, start);
        assert!((backward + forward).abs() < TOL);
        assert!(backward < 0.0);
    }

    // ─── Additivity inside a single calendar year ────────────────────────

    #[test]
    fn additivity_over_split_within_year() {
        // Within a single calendar year the denominator is constant, so
        // additivity holds exactly: for any `b` in `[a, c]` with all
        // three dates in one year, `f(a, c) == f(a, b) + f(b, c)`.
        // (Across a year boundary additivity also holds for Act/Act
        // ISDA because each whole year contributes independently — that
        // is implicit in the multi-year span test below.)
        let a = Date::ymd(2026, 1, 15).unwrap();
        let c = Date::ymd(2026, 11, 30).unwrap();
        for b in [
            Date::ymd(2026, 1, 16).unwrap(),
            Date::ymd(2026, 5, 23).unwrap(),
            Date::ymd(2026, 11, 29).unwrap(),
        ] {
            let lhs = fraction(a, c);
            let rhs = fraction(a, b) + fraction(b, c);
            assert!(
                (lhs - rhs).abs() < TOL,
                "additivity at split {b:?}: lhs={lhs}, rhs={rhs}",
            );
        }
    }

    // ─── Multi-year span ─────────────────────────────────────────────────

    #[test]
    fn multi_year_span_2023_to_2025() {
        // 2023-06-01 to 2025-06-01.
        //   2023 (non-leap): 2023-06-01 → 2024-01-01 (excl) =
        //     30 (Jun) + 31 (Jul) + 31 (Aug) + 30 (Sep) + 31 (Oct)
        //     + 30 (Nov) + 31 (Dec) = 214. Contribution = 214 / 365.
        //   2024 (leap):     2024-01-01 → 2025-01-01 (excl) = 366.
        //     Contribution = 366 / 366 = 1.0.
        //   2025 (non-leap): 2025-01-01 → 2025-06-01 (excl) =
        //     31 (Jan) + 28 (Feb, 2025 non-leap) + 31 (Mar) + 30 (Apr)
        //     + 31 (May) = 151. Contribution = 151 / 365.
        //   f = 214/365 + 1 + 151/365 = (214 + 151)/365 + 1
        //     = 365/365 + 1 = 1 + 1 = 2.0.
        let start = Date::ymd(2023, 6, 1).unwrap();
        let end = Date::ymd(2025, 6, 1).unwrap();
        let f = fraction(start, end);
        assert!(
            (f - 2.0).abs() < TOL,
            "multi-year 2023→2025: got {f}, want 2.0",
        );
    }

    #[test]
    fn supported_date_boundaries_do_not_construct_year_10000() {
        let first = Date::ymd(1583, 1, 1).unwrap();
        let last = Date::ymd(9999, 12, 31).unwrap();
        let expected = 8416.0 + 364.0 / 365.0;
        let actual = fraction(first, last);
        assert!((actual - expected).abs() < 1e-12);
        assert!((fraction(last, first) + expected).abs() < 1e-12);
        let start = Date::ymd(9998, 12, 31).unwrap();
        assert!((fraction(start, last) - 1.0).abs() < 1e-12);
    }
}
