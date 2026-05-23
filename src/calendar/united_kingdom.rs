// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! United Kingdom — Bank of England bank holiday schedule (England & Wales).
//!
//! This module is a dated snapshot of the bank-holiday list observed by the
//! sterling market — the schedule the Bank of England publishes for England
//! and Wales, and the schedule the London Stock Exchange and the GBP money
//! and FX markets close on. Scotland and Northern Ireland diverge on two
//! holidays (St Andrew's Day, the second January day) and are *not* covered
//! by this snapshot; a GBP-market user wants the England & Wales list.
//!
//! Most dates are structural — derivable from the year by a closed-form
//! rule — but two categories make a snapshot the source of truth:
//!
//! 1. Substitute days when New Year's Day, Christmas, or Boxing Day fall on
//!    a weekend (the next available weekday is observed instead).
//! 2. One-off royal-proclamation holidays (coronations, jubilees, state
//!    funerals).
//!
//! # Holiday rules
//!
//! ```text
//! New Year's Day       Jan 1 (observed Mon following if Sat/Sun)
//! Good Friday          Easter Sunday − 2 days
//! Easter Monday        Easter Sunday + 1 day
//! Early May Bank Hol.  First Monday of May
//!                      (2020: moved to Fri 8 May for the 75th anniversary
//!                      of VE Day)
//! Spring Bank Hol.     Last Monday of May
//!                      (2022: moved to Thu 2 Jun for the Platinum Jubilee)
//! Summer Bank Hol.     Last Monday of August
//! Christmas Day        Dec 25 (observed Mon following if Sat/Sun)
//! Boxing Day           Dec 26 (observed Mon following if Sat/Sun; if Dec 25
//!                      is Sun the substitute Mon is taken by Christmas, so
//!                      Boxing Day is observed Tue Dec 27; if Dec 25 is Sat
//!                      the substitute Mon is taken by Christmas, so Boxing
//!                      Day is observed Tue Dec 28)
//! ```
//!
//! # Dated specials (2020–2040 window)
//!
//! ```text
//! 2020-05-08  Early May Bank Holiday moved here for the VE Day 75th
//! 2022-06-02  Spring Bank Holiday moved here for the Platinum Jubilee
//! 2022-06-03  Platinum Jubilee Bank Holiday (extra)
//! 2022-09-19  State Funeral of Queen Elizabeth II
//! 2023-05-08  Coronation of King Charles III
//! ```
//!
//! No further dated specials have been proclaimed in the 2020–2040 window
//! at the snapshot date below; if the Crown proclaims another (a future
//! royal wedding, a jubilee, a state funeral), update both [`HOLIDAYS`]
//! and [`SNAPSHOT_DATE`].
//!
//! # Snapshot
//!
//! - Source: Bank of England bank-holiday schedule for England & Wales.
//! - Snapshot date: [`SNAPSHOT_DATE`] — `2026-05-23`.
//! - Coverage: [`COVERAGE`] — years `(2020, 2040)` inclusive (21 years).
//! - Entries: 171 (8 per year × 21 years − 1 displaced 2020 Early May Bank
//!   Holiday − 1 displaced 2022 Spring Bank Holiday + 5 dated specials).
//!
//! Out-of-coverage queries return `false` from [`is_holiday`]; a caller
//! that needs an earlier or later year should extend the table and re-bump
//! [`SNAPSHOT_DATE`] / [`COVERAGE`].
//!
//! # References
//!
//! - Bank of England, *UK bank holidays*,
//!   <https://www.bankofengland.co.uk/markets/bank-holidays>.
//! - HM Government, *UK bank holidays* (gov.uk),
//!   <https://www.gov.uk/bank-holidays> — England and Wales.
//! - Banking and Financial Dealings Act 1971 (c. 80), Schedule 1 — the
//!   statutory schedule of bank holidays in England and Wales.
//!
//! # Verification
//!
//! Every row in [`HOLIDAYS`] was cross-checked on `2026-05-23` against:
//!
//! 1. HM Government, *UK bank holidays* — <https://www.gov.uk/bank-holidays>
//!    (the canonical England-and-Wales schedule the Bank of England, the
//!    London Stock Exchange and the GBP money / FX markets close on).
//! 2. The Banking and Financial Dealings Act 1971 (Schedule 1) for the
//!    structural rules.
//! 3. Rule-derivation against the Western (Gregorian) Easter computus, the
//!    `nth Monday of month` calendar arithmetic, and the published royal
//!    proclamations for the four dated specials (2020-05-08 VE Day 75th;
//!    2022-06-02 / 06-03 Platinum Jubilee; 2022-09-19 State Funeral of
//!    Queen Elizabeth II; 2023-05-08 Coronation of King Charles III).
//!
//! Years past the gov.uk publication horizon (which typically lists the
//! current and next ~two years only) are rule-derived; the `tests` module
//! below contains a `derivation_matches_table` test that re-derives every
//! structural row from rules and asserts the table matches, so any future
//! hand-edit that drifts from the rule is caught at build time.

use crate::date::Date;

// ─── Public snapshot constants ───────────────────────────────────────────────

/// Date this snapshot of the Bank of England bank-holiday schedule was
/// taken, `YYYY-MM-DD`. Re-export this value when reporting validity to an
/// auditor.
pub const SNAPSHOT_DATE: &str = "2026-05-23";

/// Inclusive year range covered by [`HOLIDAYS`], `(min_year, max_year)`.
/// Queries outside this window return `false` from [`is_holiday`].
pub const COVERAGE: (i32, i32) = (2020, 2040);

/// Every England & Wales bank holiday in the [`COVERAGE`] window, as
/// `(year, month, day)` triples in ascending chronological order.
///
/// The list is the union of the structural holidays (computed by the
/// closed-form rules in the module docstring) and the dated specials
/// (royal proclamations and substitute days that displace a structural
/// date). It is sorted so that [`is_holiday`] can binary-search it.
pub const HOLIDAYS: &[(i32, u8, u8)] = &[
    (2020, 1, 1),   // 2020-01-01 — New Year's Day
    (2020, 4, 10),  // 2020-04-10 — Good Friday
    (2020, 4, 13),  // 2020-04-13 — Easter Monday
    (2020, 5, 8), // 2020-05-08 — Early May Bank Holiday (dated special: moved from Mon 4 May for the 75th anniversary of VE Day)
    (2020, 5, 25), // 2020-05-25 — Spring Bank Holiday
    (2020, 8, 31), // 2020-08-31 — Summer Bank Holiday
    (2020, 12, 25), // 2020-12-25 — Christmas Day
    (2020, 12, 28), // 2020-12-28 — Boxing Day (observed; Dec 26 was Sat)
    (2021, 1, 1), // 2021-01-01 — New Year's Day
    (2021, 4, 2), // 2021-04-02 — Good Friday
    (2021, 4, 5), // 2021-04-05 — Easter Monday
    (2021, 5, 3), // 2021-05-03 — Early May Bank Holiday
    (2021, 5, 31), // 2021-05-31 — Spring Bank Holiday
    (2021, 8, 30), // 2021-08-30 — Summer Bank Holiday
    (2021, 12, 27), // 2021-12-27 — Christmas Day (observed; Dec 25 was Sat)
    (2021, 12, 28), // 2021-12-28 — Boxing Day (observed; Dec 26 was Sun)
    (2022, 1, 3), // 2022-01-03 — New Year's Day (observed; Jan 1 was Sat)
    (2022, 4, 15), // 2022-04-15 — Good Friday
    (2022, 4, 18), // 2022-04-18 — Easter Monday
    (2022, 5, 2), // 2022-05-02 — Early May Bank Holiday
    (2022, 6, 2), // 2022-06-02 — Spring Bank Holiday (dated special: moved for Platinum Jubilee)
    (2022, 6, 3), // 2022-06-03 — Platinum Jubilee Bank Holiday (dated special)
    (2022, 8, 29), // 2022-08-29 — Summer Bank Holiday
    (2022, 9, 19), // 2022-09-19 — State Funeral of Queen Elizabeth II (dated special)
    (2022, 12, 26), // 2022-12-26 — Christmas Day (observed; Dec 25 was Sun)
    (2022, 12, 27), // 2022-12-27 — Boxing Day (observed; substitute Mon taken by Christmas)
    (2023, 1, 2), // 2023-01-02 — New Year's Day (observed; Jan 1 was Sun)
    (2023, 4, 7), // 2023-04-07 — Good Friday
    (2023, 4, 10), // 2023-04-10 — Easter Monday
    (2023, 5, 1), // 2023-05-01 — Early May Bank Holiday
    (2023, 5, 8), // 2023-05-08 — Coronation of King Charles III (dated special)
    (2023, 5, 29), // 2023-05-29 — Spring Bank Holiday
    (2023, 8, 28), // 2023-08-28 — Summer Bank Holiday
    (2023, 12, 25), // 2023-12-25 — Christmas Day
    (2023, 12, 26), // 2023-12-26 — Boxing Day
    (2024, 1, 1), // 2024-01-01 — New Year's Day
    (2024, 3, 29), // 2024-03-29 — Good Friday
    (2024, 4, 1), // 2024-04-01 — Easter Monday
    (2024, 5, 6), // 2024-05-06 — Early May Bank Holiday
    (2024, 5, 27), // 2024-05-27 — Spring Bank Holiday
    (2024, 8, 26), // 2024-08-26 — Summer Bank Holiday
    (2024, 12, 25), // 2024-12-25 — Christmas Day
    (2024, 12, 26), // 2024-12-26 — Boxing Day
    (2025, 1, 1), // 2025-01-01 — New Year's Day
    (2025, 4, 18), // 2025-04-18 — Good Friday
    (2025, 4, 21), // 2025-04-21 — Easter Monday
    (2025, 5, 5), // 2025-05-05 — Early May Bank Holiday
    (2025, 5, 26), // 2025-05-26 — Spring Bank Holiday
    (2025, 8, 25), // 2025-08-25 — Summer Bank Holiday
    (2025, 12, 25), // 2025-12-25 — Christmas Day
    (2025, 12, 26), // 2025-12-26 — Boxing Day
    (2026, 1, 1), // 2026-01-01 — New Year's Day
    (2026, 4, 3), // 2026-04-03 — Good Friday
    (2026, 4, 6), // 2026-04-06 — Easter Monday
    (2026, 5, 4), // 2026-05-04 — Early May Bank Holiday
    (2026, 5, 25), // 2026-05-25 — Spring Bank Holiday
    (2026, 8, 31), // 2026-08-31 — Summer Bank Holiday
    (2026, 12, 25), // 2026-12-25 — Christmas Day
    (2026, 12, 28), // 2026-12-28 — Boxing Day (observed; Dec 26 was Sat)
    (2027, 1, 1), // 2027-01-01 — New Year's Day
    (2027, 3, 26), // 2027-03-26 — Good Friday
    (2027, 3, 29), // 2027-03-29 — Easter Monday
    (2027, 5, 3), // 2027-05-03 — Early May Bank Holiday
    (2027, 5, 31), // 2027-05-31 — Spring Bank Holiday
    (2027, 8, 30), // 2027-08-30 — Summer Bank Holiday
    (2027, 12, 27), // 2027-12-27 — Christmas Day (observed; Dec 25 was Sat)
    (2027, 12, 28), // 2027-12-28 — Boxing Day (observed; Dec 26 was Sun)
    (2028, 1, 3), // 2028-01-03 — New Year's Day (observed; Jan 1 was Sat)
    (2028, 4, 14), // 2028-04-14 — Good Friday
    (2028, 4, 17), // 2028-04-17 — Easter Monday
    (2028, 5, 1), // 2028-05-01 — Early May Bank Holiday
    (2028, 5, 29), // 2028-05-29 — Spring Bank Holiday
    (2028, 8, 28), // 2028-08-28 — Summer Bank Holiday
    (2028, 12, 25), // 2028-12-25 — Christmas Day
    (2028, 12, 26), // 2028-12-26 — Boxing Day
    (2029, 1, 1), // 2029-01-01 — New Year's Day
    (2029, 3, 30), // 2029-03-30 — Good Friday
    (2029, 4, 2), // 2029-04-02 — Easter Monday
    (2029, 5, 7), // 2029-05-07 — Early May Bank Holiday
    (2029, 5, 28), // 2029-05-28 — Spring Bank Holiday
    (2029, 8, 27), // 2029-08-27 — Summer Bank Holiday
    (2029, 12, 25), // 2029-12-25 — Christmas Day
    (2029, 12, 26), // 2029-12-26 — Boxing Day
    (2030, 1, 1), // 2030-01-01 — New Year's Day
    (2030, 4, 19), // 2030-04-19 — Good Friday
    (2030, 4, 22), // 2030-04-22 — Easter Monday
    (2030, 5, 6), // 2030-05-06 — Early May Bank Holiday
    (2030, 5, 27), // 2030-05-27 — Spring Bank Holiday
    (2030, 8, 26), // 2030-08-26 — Summer Bank Holiday
    (2030, 12, 25), // 2030-12-25 — Christmas Day
    (2030, 12, 26), // 2030-12-26 — Boxing Day
    (2031, 1, 1), // 2031-01-01 — New Year's Day
    (2031, 4, 11), // 2031-04-11 — Good Friday
    (2031, 4, 14), // 2031-04-14 — Easter Monday
    (2031, 5, 5), // 2031-05-05 — Early May Bank Holiday
    (2031, 5, 26), // 2031-05-26 — Spring Bank Holiday
    (2031, 8, 25), // 2031-08-25 — Summer Bank Holiday
    (2031, 12, 25), // 2031-12-25 — Christmas Day
    (2031, 12, 26), // 2031-12-26 — Boxing Day
    (2032, 1, 1), // 2032-01-01 — New Year's Day
    (2032, 3, 26), // 2032-03-26 — Good Friday
    (2032, 3, 29), // 2032-03-29 — Easter Monday
    (2032, 5, 3), // 2032-05-03 — Early May Bank Holiday
    (2032, 5, 31), // 2032-05-31 — Spring Bank Holiday
    (2032, 8, 30), // 2032-08-30 — Summer Bank Holiday
    (2032, 12, 27), // 2032-12-27 — Christmas Day (observed; Dec 25 was Sat)
    (2032, 12, 28), // 2032-12-28 — Boxing Day (observed; Dec 26 was Sun)
    (2033, 1, 3), // 2033-01-03 — New Year's Day (observed; Jan 1 was Sat)
    (2033, 4, 15), // 2033-04-15 — Good Friday
    (2033, 4, 18), // 2033-04-18 — Easter Monday
    (2033, 5, 2), // 2033-05-02 — Early May Bank Holiday
    (2033, 5, 30), // 2033-05-30 — Spring Bank Holiday
    (2033, 8, 29), // 2033-08-29 — Summer Bank Holiday
    (2033, 12, 26), // 2033-12-26 — Christmas Day (observed; Dec 25 was Sun)
    (2033, 12, 27), // 2033-12-27 — Boxing Day (observed; substitute Mon taken by Christmas)
    (2034, 1, 2), // 2034-01-02 — New Year's Day (observed; Jan 1 was Sun)
    (2034, 4, 7), // 2034-04-07 — Good Friday
    (2034, 4, 10), // 2034-04-10 — Easter Monday
    (2034, 5, 1), // 2034-05-01 — Early May Bank Holiday
    (2034, 5, 29), // 2034-05-29 — Spring Bank Holiday
    (2034, 8, 28), // 2034-08-28 — Summer Bank Holiday
    (2034, 12, 25), // 2034-12-25 — Christmas Day
    (2034, 12, 26), // 2034-12-26 — Boxing Day
    (2035, 1, 1), // 2035-01-01 — New Year's Day
    (2035, 3, 23), // 2035-03-23 — Good Friday
    (2035, 3, 26), // 2035-03-26 — Easter Monday
    (2035, 5, 7), // 2035-05-07 — Early May Bank Holiday
    (2035, 5, 28), // 2035-05-28 — Spring Bank Holiday
    (2035, 8, 27), // 2035-08-27 — Summer Bank Holiday
    (2035, 12, 25), // 2035-12-25 — Christmas Day
    (2035, 12, 26), // 2035-12-26 — Boxing Day
    (2036, 1, 1), // 2036-01-01 — New Year's Day
    (2036, 4, 11), // 2036-04-11 — Good Friday
    (2036, 4, 14), // 2036-04-14 — Easter Monday
    (2036, 5, 5), // 2036-05-05 — Early May Bank Holiday
    (2036, 5, 26), // 2036-05-26 — Spring Bank Holiday
    (2036, 8, 25), // 2036-08-25 — Summer Bank Holiday
    (2036, 12, 25), // 2036-12-25 — Christmas Day
    (2036, 12, 26), // 2036-12-26 — Boxing Day
    (2037, 1, 1), // 2037-01-01 — New Year's Day
    (2037, 4, 3), // 2037-04-03 — Good Friday
    (2037, 4, 6), // 2037-04-06 — Easter Monday
    (2037, 5, 4), // 2037-05-04 — Early May Bank Holiday
    (2037, 5, 25), // 2037-05-25 — Spring Bank Holiday
    (2037, 8, 31), // 2037-08-31 — Summer Bank Holiday
    (2037, 12, 25), // 2037-12-25 — Christmas Day
    (2037, 12, 28), // 2037-12-28 — Boxing Day (observed; Dec 26 was Sat)
    (2038, 1, 1), // 2038-01-01 — New Year's Day
    (2038, 4, 23), // 2038-04-23 — Good Friday
    (2038, 4, 26), // 2038-04-26 — Easter Monday
    (2038, 5, 3), // 2038-05-03 — Early May Bank Holiday
    (2038, 5, 31), // 2038-05-31 — Spring Bank Holiday
    (2038, 8, 30), // 2038-08-30 — Summer Bank Holiday
    (2038, 12, 27), // 2038-12-27 — Christmas Day (observed; Dec 25 was Sat)
    (2038, 12, 28), // 2038-12-28 — Boxing Day (observed; Dec 26 was Sun)
    (2039, 1, 3), // 2039-01-03 — New Year's Day (observed; Jan 1 was Sat)
    (2039, 4, 8), // 2039-04-08 — Good Friday
    (2039, 4, 11), // 2039-04-11 — Easter Monday
    (2039, 5, 2), // 2039-05-02 — Early May Bank Holiday
    (2039, 5, 30), // 2039-05-30 — Spring Bank Holiday
    (2039, 8, 29), // 2039-08-29 — Summer Bank Holiday
    (2039, 12, 26), // 2039-12-26 — Christmas Day (observed; Dec 25 was Sun)
    (2039, 12, 27), // 2039-12-27 — Boxing Day (observed; substitute Mon taken by Christmas)
    (2040, 1, 2), // 2040-01-02 — New Year's Day (observed; Jan 1 was Sun)
    (2040, 3, 30), // 2040-03-30 — Good Friday
    (2040, 4, 2), // 2040-04-02 — Easter Monday
    (2040, 5, 7), // 2040-05-07 — Early May Bank Holiday
    (2040, 5, 28), // 2040-05-28 — Spring Bank Holiday
    (2040, 8, 27), // 2040-08-27 — Summer Bank Holiday
    (2040, 12, 25), // 2040-12-25 — Christmas Day
    (2040, 12, 26), // 2040-12-26 — Boxing Day
];

// ─── Public API ──────────────────────────────────────────────────────────────

/// Returns `true` if `date` is an England & Wales bank holiday under the
/// Bank of England published schedule.
///
/// Implemented as a binary search over [`HOLIDAYS`]; the table is sorted
/// chronologically (a debug-build self-test in this module's `#[cfg(test)]`
/// block asserts that). Dates outside the [`COVERAGE`] window return
/// `false` — a caller that needs an earlier or later year must extend the
/// table.
///
/// Weekends are *not* implicitly holidays: this function answers only the
/// bank-holiday question. Whether a date is a business day is the union
/// of `!is_holiday(date)` and `date.day_of_week()` not being Saturday or
/// Sunday — applied by the dispatcher in `calendar::mod`.
///
/// # Examples
///
/// ```
/// use regit_daycount::Date;
/// use regit_daycount::calendar::united_kingdom::is_holiday;
///
/// // 2024-12-25 is a UK bank holiday (Christmas Day).
/// assert!(is_holiday(Date::ymd(2024, 12, 25).unwrap()));
///
/// // 2024-12-24 is Christmas Eve — not a bank holiday.
/// assert!(!is_holiday(Date::ymd(2024, 12, 24).unwrap()));
/// ```
#[must_use]
pub fn is_holiday(date: Date) -> bool {
    let (min_year, max_year) = COVERAGE;
    let y = date.year();
    if y < min_year || y > max_year {
        return false;
    }
    let key = (y, date.month(), date.day());
    HOLIDAYS.binary_search(&key).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Structural invariants ───────────────────────────────────────────

    #[test]
    fn holidays_table_is_sorted_strictly_ascending() {
        // Binary search is only correct over a sorted table; this guards
        // any future hand-edit that would break that.
        for w in HOLIDAYS.windows(2) {
            assert!(
                w[0] < w[1],
                "HOLIDAYS not sorted at {:?} -> {:?}",
                w[0],
                w[1]
            );
        }
    }

    #[test]
    fn holidays_within_coverage_window() {
        let (lo, hi) = COVERAGE;
        for &(y, _, _) in HOLIDAYS {
            assert!((lo..=hi).contains(&y), "year {y} outside COVERAGE");
        }
    }

    #[test]
    fn snapshot_constants_have_expected_shape() {
        assert_eq!(SNAPSHOT_DATE, "2026-05-23");
        assert_eq!(COVERAGE, (2020, 2040));
    }

    // ─── 2024 — every structural holiday ─────────────────────────────────

    #[test]
    fn year_2024_full_schedule() {
        let expected = [
            (2024, 1, 1),   // New Year's Day
            (2024, 3, 29),  // Good Friday
            (2024, 4, 1),   // Easter Monday
            (2024, 5, 6),   // Early May Bank Holiday
            (2024, 5, 27),  // Spring Bank Holiday
            (2024, 8, 26),  // Summer Bank Holiday
            (2024, 12, 25), // Christmas Day
            (2024, 12, 26), // Boxing Day
        ];
        for (y, m, d) in expected {
            assert!(
                is_holiday(Date::ymd(y, m, d).unwrap()),
                "{y}-{m:02}-{d:02} should be a UK bank holiday",
            );
        }
        // And a non-holiday weekday in 2024.
        assert!(!is_holiday(Date::ymd(2024, 5, 28).unwrap()));
    }

    // ─── 2026 — Easter-derived plus weekend-shift Boxing Day ─────────────

    #[test]
    fn year_2026_full_schedule() {
        // Easter Sunday 2026 = 2026-04-05.
        let expected = [
            (2026, 1, 1),   // New Year's Day (Thu)
            (2026, 4, 3),   // Good Friday  (Easter − 2)
            (2026, 4, 6),   // Easter Monday (Easter + 1)
            (2026, 5, 4),   // Early May (first Mon of May)
            (2026, 5, 25),  // Spring (last Mon of May)
            (2026, 8, 31),  // Summer (last Mon of August)
            (2026, 12, 25), // Christmas Day (Fri — no shift)
            (2026, 12, 28), // Boxing Day (observed Mon; Dec 26 is Sat)
        ];
        for (y, m, d) in expected {
            assert!(
                is_holiday(Date::ymd(y, m, d).unwrap()),
                "{y}-{m:02}-{d:02} should be a UK bank holiday",
            );
        }
        // Dec 26 itself is not the observed Boxing Day in 2026.
        assert!(!is_holiday(Date::ymd(2026, 12, 26).unwrap()));
    }

    // ─── Weekend-shift cases ─────────────────────────────────────────────

    #[test]
    fn new_year_2022_observed_monday() {
        // 2022-01-01 is a Saturday; the bank holiday is observed Mon 2022-01-03.
        assert!(!is_holiday(Date::ymd(2022, 1, 1).unwrap()));
        assert!(is_holiday(Date::ymd(2022, 1, 3).unwrap()));
    }

    #[test]
    fn christmas_2021_sat_and_boxing_sun_both_shifted() {
        // 2021-12-25 is a Saturday and 2021-12-26 is a Sunday.
        // Christmas observed Mon 2021-12-27; Boxing Day observed Tue 2021-12-28.
        assert!(!is_holiday(Date::ymd(2021, 12, 25).unwrap()));
        assert!(!is_holiday(Date::ymd(2021, 12, 26).unwrap()));
        assert!(is_holiday(Date::ymd(2021, 12, 27).unwrap()));
        assert!(is_holiday(Date::ymd(2021, 12, 28).unwrap()));
    }

    // ─── Dated specials ──────────────────────────────────────────────────

    #[test]
    fn platinum_jubilee_2022_specials() {
        // The Spring Bank Holiday was moved from the last Monday of May
        // (2022-05-30) to Thursday 2022-06-02; 2022-06-03 was added as the
        // Platinum Jubilee Bank Holiday.
        assert!(is_holiday(Date::ymd(2022, 6, 2).unwrap()));
        assert!(is_holiday(Date::ymd(2022, 6, 3).unwrap()));
        assert!(!is_holiday(Date::ymd(2022, 5, 30).unwrap()));
    }

    #[test]
    fn queen_elizabeth_ii_state_funeral_2022_09_19() {
        assert!(is_holiday(Date::ymd(2022, 9, 19).unwrap()));
    }

    #[test]
    fn king_charles_iii_coronation_2023_05_08() {
        assert!(is_holiday(Date::ymd(2023, 5, 8).unwrap()));
    }

    #[test]
    fn ve_day_75th_2020_05_08_replaces_first_monday() {
        // The 2020 Early May Bank Holiday was moved from Mon 4 May to
        // Fri 8 May to coincide with the 75th anniversary of VE Day. The
        // displaced Monday is *not* a holiday.
        assert!(is_holiday(Date::ymd(2020, 5, 8).unwrap()));
        assert!(!is_holiday(Date::ymd(2020, 5, 4).unwrap()));
    }

    // ─── Rule-derivation cross-check ─────────────────────────────────────

    #[test]
    fn derivation_matches_table() {
        // Re-derive every England-and-Wales bank holiday in the coverage
        // window from the published rules (closed-form for the weekday-
        // anchored dates, computus for the Easter-derived ones, hard-coded
        // for the dated specials) and assert the sorted union equals
        // `HOLIDAYS`. Any future hand-edit that drifts from a rule, or
        // that loses / duplicates a dated special, fails this test.
        use crate::Weekday;

        let (lo, hi) = COVERAGE;
        // 8 rows / year * 21 years + a handful of dated specials; 200 is
        // a comfortable upper bound for the no_std fixed-size buffer.
        let mut buf: [(i32, u8, u8); 200] = [(0, 0, 0); 200];
        let mut n: usize = 0;

        // Helper: last Monday of (y, m) — the largest n ∈ 1..=5 for which
        // `nth_weekday_of_month` succeeds.
        let last_monday = |y: i32, m: u8| -> Date {
            let mut last_n: u8 = 1;
            for cand in 1..=5u8 {
                if Date::nth_weekday_of_month(y, m, cand, Weekday::Mon).is_ok() {
                    last_n = cand;
                }
            }
            Date::nth_weekday_of_month(y, m, last_n, Weekday::Mon).unwrap()
        };

        for y in lo..=hi {
            // 1. New Year's Day — Jan 1 shifted to following Mon if Sat/Sun.
            let nyd = Date::ymd(y, 1, 1).unwrap();
            let nyd_obs = match nyd.day_of_week() {
                Weekday::Sat => Date::ymd(y, 1, 3).unwrap(),
                Weekday::Sun => Date::ymd(y, 1, 2).unwrap(),
                _ => nyd,
            };
            buf[n] = (nyd_obs.year(), nyd_obs.month(), nyd_obs.day());
            n += 1;

            // 2. Good Friday = Easter − 2.
            let easter = Date::easter_sunday(y);
            let gf = easter.add_days(-2);
            buf[n] = (gf.year(), gf.month(), gf.day());
            n += 1;

            // 3. Easter Monday = Easter + 1.
            let em = easter.add_days(1);
            buf[n] = (em.year(), em.month(), em.day());
            n += 1;

            // 4. Early May Bank Holiday — 1st Mon of May (2020 special:
            //    moved to Fri 8 May for the 75th anniversary of VE Day).
            if y == 2020 {
                buf[n] = (2020, 5, 8);
            } else {
                let d = Date::nth_weekday_of_month(y, 5, 1, Weekday::Mon).unwrap();
                buf[n] = (d.year(), d.month(), d.day());
            }
            n += 1;

            // 5. Spring Bank Holiday — last Mon of May (2022 special: moved
            //    to Thu 2 Jun for the Platinum Jubilee).
            if y == 2022 {
                buf[n] = (2022, 6, 2);
            } else {
                let d = last_monday(y, 5);
                buf[n] = (d.year(), d.month(), d.day());
            }
            n += 1;

            // 6. Summer Bank Holiday — last Mon of August.
            let d = last_monday(y, 8);
            buf[n] = (d.year(), d.month(), d.day());
            n += 1;

            // 7+8. Christmas + Boxing — cascade on Dec 25 weekday.
            let xmas = Date::ymd(y, 12, 25).unwrap();
            let (x_obs, b_obs) = match xmas.day_of_week() {
                Weekday::Fri => ((y, 12, 25), (y, 12, 28)), // Boxing 26 Sat → Mon 28
                Weekday::Sat => ((y, 12, 27), (y, 12, 28)), // Xmas Mon 27, Boxing Tue 28
                Weekday::Sun => ((y, 12, 26), (y, 12, 27)), // Xmas Mon 26, Boxing Tue 27
                _ => ((y, 12, 25), (y, 12, 26)),
            };
            buf[n] = x_obs;
            n += 1;
            buf[n] = b_obs;
            n += 1;
        }

        // Dated specials layered on top — the displaced 2020 Early May and
        // 2022 Spring rows are already handled in the per-year loop.
        buf[n] = (2022, 6, 3); // Platinum Jubilee extra
        n += 1;
        buf[n] = (2022, 9, 19); // QEII state funeral
        n += 1;
        buf[n] = (2023, 5, 8); // Coronation of King Charles III
        n += 1;

        buf[..n].sort_unstable();

        assert_eq!(
            n,
            HOLIDAYS.len(),
            "rule-derived row count {} != HOLIDAYS.len() {}",
            n,
            HOLIDAYS.len(),
        );
        for (i, (got, want)) in buf[..n].iter().zip(HOLIDAYS.iter()).enumerate() {
            assert_eq!(got, want, "row {i}: derived {got:?} != HOLIDAYS {want:?}");
        }
    }

    // ─── Out-of-coverage ─────────────────────────────────────────────────

    #[test]
    fn out_of_coverage_returns_false() {
        // 2019-12-25 was a Wednesday and historically a UK bank holiday,
        // but it is outside this snapshot's coverage window and must
        // return `false`.
        assert!(!is_holiday(Date::ymd(2019, 12, 25).unwrap()));
        // 2041-01-01 is outside the upper end of the window.
        assert!(!is_holiday(Date::ymd(2041, 1, 1).unwrap()));
    }
}
