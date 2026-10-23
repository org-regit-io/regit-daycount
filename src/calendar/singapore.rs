// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! Singapore — SGX full-day public-holiday trading calendar.
//!
//! Gazetted public holidays, including polling days, close the exchange.
//! This snapshot retains Saturday nominal holiday dates, including
//! Hari Raya Puasa on 2026-03-21, and replaces Sunday nominal dates by
//! the observed weekday. Half-day sessions remain business days.
//! The table includes polling days 2020-07-10, 2023-09-01 and 2025-05-03.
//!
//! Complete 2026–2027 lists were independently read from MOM on 2026-10-23:
//! <https://www.mom.gov.sg/employment-practices/public-holidays>.
//! The full 2020–2040 named-date set and every business-day classification
//! were compared against `holidays==0.107` Singapore PUBLIC and XSES/SGX
//! financial calendars. The latter independently confirms market closures
//! and excludes partial sessions; neither comparison turns projections into
//! official published schedules.
//!
//! Religious and lunar dates outside the published horizon are estimates
//! and can change following Singapore determinations. They require explicit
//! [`super::CoveragePolicy::AllowProjected`] opt-in. Unlike a reference with
//! missing future table rows, the independent oracle executes actual future
//! calendar calculations and rejects dates outside its stated bounds.
//! Source hashes, mappings, and estimated-date metadata are retained in
//! `validation/calendar_references.py` and `validation/oracle.json`.
//!
//! Independent financial reference:
//! <https://github.com/vacanza/holidays/blob/v0.107/holidays/financial/singapore_exchange.py>.
//! Official trading information: <https://www.sgx.com/stock-exchange/trading>.

use crate::date::Date;

/// Date of the SGX trading-calendar snapshot embedded in this module,
/// `YYYY-MM-DD`.
pub const SNAPSHOT_DATE: &str = "2026-10-23";

/// Inclusive `(first_year, last_year)` coverage of [`HOLIDAYS`]. Dates
/// outside this window cause [`is_holiday`] to return `false` without
/// consulting the table.
pub const COVERAGE: (i32, i32) = (2020, 2040);

/// SGX trading-calendar holidays, sorted ascending by `(year, month,
/// day)`. Each row pairs the observed market-closure date with a comment
/// naming the holiday it implements.
///
/// The list reflects the *observed* closure (i.e. with the Monday-in-lieu
/// rule already applied for Sunday-falling fixed and religious dates);
/// the underlying rule date is named in the row comment when the two
/// differ. Every 2020–2040 date has an independent reference-library
/// comparison. Complete official annual lists have primary-source readback
/// for 2026–2027; the checked policy keeps that publication evidence
/// separate from library agreement. See [`super::coverage`].
#[rustfmt::skip]
pub const HOLIDAYS: &[(i32, u8, u8)] = &[
    // 2020
    (2020, 1, 1), // New Year's Day
    (2020, 1, 25), // Chinese New Year
    (2020, 1, 27), // Chinese New Year (observed)
    (2020, 4, 10), // Good Friday
    (2020, 5, 1), // Labor Day
    (2020, 5, 7), // Vesak Day
    (2020, 5, 25), // Eid al-Fitr (observed)
    (2020, 7, 10), // Polling Day
    (2020, 7, 31), // Eid al-Adha
    (2020, 8, 10), // National Day (observed)
    (2020, 11, 14), // Diwali
    (2020, 12, 25), // Christmas Day
    // 2021
    (2021, 1, 1), // New Year's Day
    (2021, 2, 12), // Chinese New Year
    (2021, 2, 13), // Chinese New Year
    (2021, 4, 2), // Good Friday
    (2021, 5, 1), // Labor Day
    (2021, 5, 13), // Eid al-Fitr
    (2021, 5, 26), // Vesak Day
    (2021, 7, 20), // Eid al-Adha
    (2021, 8, 9), // National Day
    (2021, 11, 4), // Diwali
    (2021, 12, 25), // Christmas Day
    // 2022
    (2022, 1, 1), // New Year's Day
    (2022, 2, 1), // Chinese New Year
    (2022, 2, 2), // Chinese New Year
    (2022, 4, 15), // Good Friday
    (2022, 5, 2), // Labor Day (observed)
    (2022, 5, 3), // Eid al-Fitr
    (2022, 5, 16), // Vesak Day (observed)
    (2022, 7, 11), // Eid al-Adha (observed)
    (2022, 8, 9), // National Day
    (2022, 10, 24), // Diwali
    (2022, 12, 26), // Christmas Day (observed)
    // 2023
    (2023, 1, 2), // New Year's Day (observed)
    (2023, 1, 23), // Chinese New Year
    (2023, 1, 24), // Chinese New Year (observed)
    (2023, 4, 7), // Good Friday
    (2023, 4, 22), // Eid al-Fitr
    (2023, 5, 1), // Labor Day
    (2023, 6, 2), // Vesak Day
    (2023, 6, 29), // Eid al-Adha
    (2023, 8, 9), // National Day
    (2023, 9, 1), // Polling Day
    (2023, 11, 13), // Diwali (observed)
    (2023, 12, 25), // Christmas Day
    // 2024
    (2024, 1, 1), // New Year's Day
    (2024, 2, 10), // Chinese New Year
    (2024, 2, 12), // Chinese New Year (observed)
    (2024, 3, 29), // Good Friday
    (2024, 4, 10), // Eid al-Fitr
    (2024, 5, 1), // Labor Day
    (2024, 5, 22), // Vesak Day
    (2024, 6, 17), // Eid al-Adha
    (2024, 8, 9), // National Day
    (2024, 10, 31), // Diwali
    (2024, 12, 25), // Christmas Day
    // 2025
    (2025, 1, 1), // New Year's Day
    (2025, 1, 29), // Chinese New Year
    (2025, 1, 30), // Chinese New Year
    (2025, 3, 31), // Eid al-Fitr
    (2025, 4, 18), // Good Friday
    (2025, 5, 1), // Labor Day
    (2025, 5, 3), // Polling Day
    (2025, 5, 12), // Vesak Day
    (2025, 6, 7), // Eid al-Adha
    (2025, 8, 9), // National Day
    (2025, 10, 20), // Diwali
    (2025, 12, 25), // Christmas Day
    // 2026
    (2026, 1, 1), // New Year's Day
    (2026, 2, 17), // Chinese New Year
    (2026, 2, 18), // Chinese New Year
    (2026, 3, 21), // Eid al-Fitr
    (2026, 4, 3), // Good Friday
    (2026, 5, 1), // Labor Day
    (2026, 5, 27), // Eid al-Adha
    (2026, 6, 1), // Vesak Day (observed)
    (2026, 8, 10), // National Day (observed)
    (2026, 11, 9), // Diwali (observed)
    (2026, 12, 25), // Christmas Day
    // 2027
    (2027, 1, 1), // New Year's Day
    (2027, 2, 6), // Chinese New Year
    (2027, 2, 8), // Chinese New Year (observed)
    (2027, 3, 10), // Eid al-Fitr
    (2027, 3, 26), // Good Friday
    (2027, 5, 1), // Labor Day
    (2027, 5, 17), // Eid al-Adha
    (2027, 5, 20), // Vesak Day
    (2027, 8, 9), // National Day
    (2027, 10, 28), // Diwali
    (2027, 12, 25), // Christmas Day
    // 2028
    (2028, 1, 1), // New Year's Day
    (2028, 1, 26), // Chinese New Year (estimated)
    (2028, 1, 27), // Chinese New Year (estimated)
    (2028, 2, 28), // Eid al-Fitr (observed, estimated)
    (2028, 4, 14), // Good Friday
    (2028, 5, 1), // Labor Day
    (2028, 5, 5), // Eid al-Adha (estimated)
    (2028, 5, 9), // Vesak Day (estimated)
    (2028, 8, 9), // National Day
    (2028, 11, 15), // Diwali
    (2028, 12, 25), // Christmas Day
    // 2029
    (2029, 1, 1), // New Year's Day
    (2029, 2, 13), // Chinese New Year (estimated)
    (2029, 2, 14), // Chinese New Year (estimated)
    (2029, 2, 15), // Eid al-Fitr (estimated)
    (2029, 3, 30), // Good Friday
    (2029, 4, 25), // Eid al-Adha (estimated)
    (2029, 5, 1), // Labor Day
    (2029, 5, 28), // Vesak Day (observed, estimated)
    (2029, 8, 9), // National Day
    (2029, 11, 5), // Diwali
    (2029, 12, 25), // Christmas Day
    // 2030
    (2030, 1, 1), // New Year's Day
    (2030, 2, 4), // Chinese New Year (estimated); Eid al-Fitr (estimated)
    (2030, 2, 5), // Chinese New Year (observed, estimated)
    (2030, 4, 15), // Eid al-Adha (observed, estimated)
    (2030, 4, 19), // Good Friday
    (2030, 5, 1), // Labor Day
    (2030, 5, 16), // Vesak Day (estimated)
    (2030, 8, 9), // National Day
    (2030, 10, 26), // Diwali
    (2030, 12, 25), // Christmas Day
    // 2031
    (2031, 1, 1), // New Year's Day
    (2031, 1, 23), // Chinese New Year (estimated)
    (2031, 1, 24), // Chinese New Year (estimated)
    (2031, 1, 25), // Eid al-Fitr (estimated)
    (2031, 4, 3), // Eid al-Adha (estimated)
    (2031, 4, 11), // Good Friday
    (2031, 5, 1), // Labor Day
    (2031, 6, 4), // Vesak Day (estimated)
    (2031, 8, 9), // National Day
    (2031, 11, 14), // Diwali
    (2031, 12, 25), // Christmas Day
    // 2032
    (2032, 1, 1), // New Year's Day
    (2032, 1, 14), // Eid al-Fitr (estimated)
    (2032, 2, 11), // Chinese New Year (estimated)
    (2032, 2, 12), // Chinese New Year (estimated)
    (2032, 3, 22), // Eid al-Adha (estimated)
    (2032, 3, 26), // Good Friday
    (2032, 5, 1), // Labor Day
    (2032, 5, 24), // Vesak Day (observed, estimated)
    (2032, 8, 9), // National Day
    (2032, 11, 2), // Diwali
    (2032, 12, 25), // Christmas Day
    // 2033
    (2033, 1, 1), // New Year's Day
    (2033, 1, 3), // Eid al-Fitr (estimated)
    (2033, 1, 31), // Chinese New Year (estimated)
    (2033, 2, 1), // Chinese New Year (estimated)
    (2033, 3, 12), // Eid al-Adha (estimated)
    (2033, 4, 15), // Good Friday
    (2033, 5, 2), // Labor Day (observed)
    (2033, 5, 13), // Vesak Day (estimated)
    (2033, 8, 9), // National Day
    (2033, 10, 22), // Diwali
    (2033, 12, 23), // Eid al-Fitr (estimated)
    (2033, 12, 26), // Christmas Day (observed)
    // 2034
    (2034, 1, 2), // New Year's Day (observed)
    (2034, 2, 20), // Chinese New Year (estimated)
    (2034, 2, 21), // Chinese New Year (observed, estimated)
    (2034, 3, 1), // Eid al-Adha (estimated)
    (2034, 4, 7), // Good Friday
    (2034, 5, 1), // Labor Day
    (2034, 6, 1), // Vesak Day (estimated)
    (2034, 8, 9), // National Day
    (2034, 11, 10), // Diwali
    (2034, 12, 12), // Eid al-Fitr (estimated)
    (2034, 12, 25), // Christmas Day
    // 2035
    (2035, 1, 1), // New Year's Day
    (2035, 2, 8), // Chinese New Year (estimated)
    (2035, 2, 9), // Chinese New Year (estimated)
    (2035, 2, 19), // Eid al-Adha (estimated)
    (2035, 3, 23), // Good Friday
    (2035, 5, 1), // Labor Day
    (2035, 5, 22), // Vesak Day (estimated)
    (2035, 8, 9), // National Day
    (2035, 10, 30), // Diwali
    (2035, 12, 1), // Eid al-Fitr (estimated)
    (2035, 12, 25), // Christmas Day
    // 2036
    (2036, 1, 1), // New Year's Day
    (2036, 1, 28), // Chinese New Year (estimated)
    (2036, 1, 29), // Chinese New Year (estimated)
    (2036, 2, 8), // Eid al-Adha (estimated)
    (2036, 4, 11), // Good Friday
    (2036, 5, 1), // Labor Day
    (2036, 5, 10), // Vesak Day (estimated)
    (2036, 8, 9), // National Day
    (2036, 11, 17), // Diwali
    (2036, 11, 19), // Eid al-Fitr (estimated)
    (2036, 12, 25), // Christmas Day
    // 2037
    (2037, 1, 1), // New Year's Day
    (2037, 1, 27), // Eid al-Adha (estimated)
    (2037, 2, 16), // Chinese New Year (estimated)
    (2037, 2, 17), // Chinese New Year (observed, estimated)
    (2037, 4, 3), // Good Friday
    (2037, 5, 1), // Labor Day
    (2037, 5, 29), // Vesak Day (estimated)
    (2037, 8, 10), // National Day (observed)
    (2037, 11, 6), // Diwali
    (2037, 11, 9), // Eid al-Fitr (estimated)
    (2037, 12, 25), // Christmas Day
    // 2038
    (2038, 1, 1), // New Year's Day
    (2038, 1, 16), // Eid al-Adha (estimated)
    (2038, 2, 4), // Chinese New Year (estimated)
    (2038, 2, 5), // Chinese New Year (estimated)
    (2038, 4, 23), // Good Friday
    (2038, 5, 1), // Labor Day
    (2038, 5, 18), // Vesak Day (estimated)
    (2038, 8, 9), // National Day
    (2038, 10, 27), // Diwali
    (2038, 10, 30), // Eid al-Fitr (estimated)
    (2038, 12, 25), // Christmas Day
    // 2039
    (2039, 1, 1), // New Year's Day
    (2039, 1, 6), // Eid al-Adha (estimated)
    (2039, 1, 24), // Chinese New Year (estimated)
    (2039, 1, 25), // Chinese New Year (estimated)
    (2039, 4, 8), // Good Friday
    (2039, 5, 2), // Labor Day (observed)
    (2039, 5, 7), // Vesak Day (estimated)
    (2039, 8, 9), // National Day
    (2039, 10, 19), // Eid al-Fitr (estimated)
    (2039, 11, 15), // Diwali
    (2039, 12, 26), // Eid al-Adha (estimated)
    (2039, 12, 27), // Christmas Day (observed)
    // 2040
    (2040, 1, 2), // New Year's Day (observed)
    (2040, 2, 13), // Chinese New Year (estimated)
    (2040, 2, 14), // Chinese New Year (observed, estimated)
    (2040, 3, 30), // Good Friday
    (2040, 5, 1), // Labor Day
    (2040, 5, 25), // Vesak Day (estimated)
    (2040, 8, 9), // National Day
    (2040, 10, 8), // Eid al-Fitr (estimated)
    (2040, 11, 5), // Diwali (observed)
    (2040, 12, 15), // Eid al-Adha (estimated)
    (2040, 12, 25), // Christmas Day
];

/// True if `date` is an SGX-published trading-calendar holiday inside the
/// snapshot's [`COVERAGE`] window.
///
/// The lookup is a binary search over [`HOLIDAYS`], which is sorted
/// ascending by `(year, month, day)`. Dates outside [`COVERAGE`] return
/// `false` without consulting the table — refresh the snapshot before
/// classifying dates past the horizon. Weekends are *not* classified
/// here; SGX is already closed on Saturday and Sunday and that is the
/// caller's business-day predicate to express.
///
/// # Examples
///
/// ```
/// use regit_daycount::Date;
/// use regit_daycount::calendar::singapore;
///
/// // National Day 2024 (Friday) is an SGX holiday.
/// assert!(singapore::is_holiday(Date::ymd(2024, 8, 9).unwrap()));
///
/// // 2024-08-10 is a Saturday — weekends are not classified by
/// // `is_holiday`, only by the caller's business-day predicate.
/// assert!(!singapore::is_holiday(Date::ymd(2024, 8, 10).unwrap()));
/// ```
#[must_use]
pub fn is_holiday(date: Date) -> bool {
    let (first, last) = COVERAGE;
    let year = date.year();
    if year < first || year > last {
        return false;
    }
    let key = (year, date.month(), date.day());
    HOLIDAYS.binary_search(&key).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── 2024 — every published SGX holiday in the year ──────────────

    #[test]
    fn gazetted_polling_days_close_the_market() {
        for (year, month, day) in [(2020, 7, 10), (2023, 9, 1), (2025, 5, 3)] {
            assert!(is_holiday(Date::ymd(year, month, day).unwrap()));
        }
    }

    #[test]
    fn holidays_2024() {
        // Every gazetted SGX trading-closure date for 2024. Verified
        // against the SGX 2024 trading-calendar PDF and the MOM 2024
        // public-holiday gazette.
        let dates = [
            (1, 1),   // New Year's Day (Mon)
            (2, 10),  // Chinese New Year day 1 (Sat)
            (2, 12),  // Chinese New Year day 2 observed (Sun 11 → Mon 12)
            (3, 29),  // Good Friday
            (4, 10),  // Hari Raya Puasa
            (5, 1),   // Labour Day (Wed)
            (5, 22),  // Vesak Day (Wed)
            (6, 17),  // Hari Raya Haji (Mon)
            (8, 9),   // National Day (Fri)
            (10, 31), // Deepavali (Thu)
            (12, 25), // Christmas Day (Wed)
        ];
        for (m, d) in dates {
            assert!(
                is_holiday(Date::ymd(2024, m, d).unwrap()),
                "2024-{m:02}-{d:02} must be an SGX holiday",
            );
        }
    }

    // ─── Out-of-coverage years return false ──────────────────────────

    #[test]
    fn out_of_coverage_returns_false() {
        // 2019-01-01 predates the snapshot window; 2041-01-01 is past
        // its horizon. Both must return `false` without panicking.
        assert!(!is_holiday(Date::ymd(2019, 1, 1).unwrap()));
        assert!(!is_holiday(Date::ymd(2041, 1, 1).unwrap()));
    }

    // ─── The HOLIDAYS table is sorted ascending ──────────────────────

    #[test]
    fn holidays_sorted_ascending() {
        // Binary search in `is_holiday` requires the table to be sorted
        // by `(year, month, day)` in strictly ascending order, with no
        // duplicate rows.
        for window in HOLIDAYS.windows(2) {
            let (a, b) = (window[0], window[1]);
            assert!(a < b, "HOLIDAYS not strictly ascending: {a:?} not < {b:?}");
        }
    }

    // ─── Random non-holiday inside coverage ──────────────────────────

    #[test]
    fn random_non_holiday() {
        // 2024-03-15 is a regular Friday with no SGX closure.
        assert!(!is_holiday(Date::ymd(2024, 3, 15).unwrap()));
    }

    // ─── Snapshot metadata ───────────────────────────────────────────

    #[test]
    fn snapshot_metadata_present() {
        assert_eq!(SNAPSHOT_DATE, "2026-10-23");
        assert_eq!(COVERAGE, (2020, 2040));
    }

    // ─── Coverage boundary years populated ───────────────────────────

    #[test]
    fn coverage_boundary_years_populated() {
        // Every year in the coverage window has at least one row in the
        // table — a basic sanity check that the snapshot is not silently
        // truncated at one end.
        let (first, last) = COVERAGE;
        for year in first..=last {
            assert!(
                HOLIDAYS.iter().any(|(y, _, _)| *y == year),
                "no SGX holiday rows for year {year}",
            );
        }
    }
}
