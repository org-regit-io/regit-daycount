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
//! ```text
//! if start.year == end.year:
//!     d     = days_between(start, end)
//!     denom = 366 if is_leap_year(start.year) else 365
//!     return d / denom
//!
//! else (multi-year span):
//!     sum = 0.0
//!     for y in start.year ..= end.year:
//!         sub_start = max(start, Date(y,     1, 1))
//!         sub_end   = min(end,   Date(y + 1, 1, 1))
//!         d_y       = days_between(sub_start, sub_end)
//!         if d_y > 0:
//!             denom = 366 if is_leap_year(y) else 365
//!             sum  += d_y / denom
//!     return sum
//! ```
//!
//! All day counts use `Date::days_between` — signed, end-exclusive,
//! start-inclusive — so the per-year contributions cover the half-open
//! interval `[Date(y, 1, 1), Date(y + 1, 1, 1))` (i.e. the whole calendar
//! year `y`) and the partial-year boundaries fall out of the
//! `max` / `min` clamp without an additional case. The same-year case is
//! fast-pathed: it is the common one and skips the loop and the two
//! `ymd_unchecked` constructions.
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
/// multi-year case splits at every January 1 the interval crosses, divides
/// each sub-period by its own per-year denominator, and sums. Inverted
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

    // Same-year fast path: one division, no `ymd_unchecked` constructions.
    if start_year == end_year {
        let d = f64::from(start.days_between(end));
        let denom = if Date::is_leap_year(start_year) {
            366.0_f64
        } else {
            365.0_f64
        };
        return d / denom;
    }

    // Multi-year span. Iterate `y` over every calendar year the interval
    // touches; for each year clamp the sub-interval to `[Date(y, 1, 1),
    // Date(y + 1, 1, 1))` and accumulate `d_y / denom_y`. `y + 1` is safe
    // because year range is `[1583, 9999]` and `end_year <= 9999`, so the
    // worst-case `Date::ymd_unchecked(10_000, 1, 1)` is constructed but
    // never observed by a caller (it only feeds `days_between` as a
    // clamp endpoint).
    let mut sum = 0.0_f64;
    for y in start_year..=end_year {
        let year_start = Date::ymd_unchecked(y, 1, 1);
        let year_end = Date::ymd_unchecked(y + 1, 1, 1);
        let sub_start = if start > year_start {
            start
        } else {
            year_start
        };
        let sub_end = if end < year_end { end } else { year_end };
        let d_y = sub_start.days_between(sub_end);
        if d_y > 0 {
            let denom = if Date::is_leap_year(y) {
                366.0_f64
            } else {
                365.0_f64
            };
            sum += f64::from(d_y) / denom;
        }
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::*;

    // The tolerance 1e-12 is the working slack used throughout this crate's
    // numeric assertions: it is loose enough that a benign last-bit rounding
    // never causes a spurious failure, and tight enough that any drift large
    // enough to mis-state a cashflow at the cent level (≥ 1e-9 on a unit
    // notional) is caught immediately. We deliberately do not assert against
    // `f64::EPSILON`; the goal is auditable, not maximally-tight. The ISDA
    // worked example below matches its printed 9-dp number to well inside
    // 1e-12, so this tolerance is generous for the present module.
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
}
