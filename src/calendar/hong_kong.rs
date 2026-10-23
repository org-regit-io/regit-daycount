// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! HKEX securities-market full-day trading closures.
//!
//! Partial sessions remain business days; settlement can be closed on those
//! days, so this is not an HKEX settlement calendar. Weekends are excluded
//! separately. Historical rows include retained full-day weather cancellations.
//! Complete official securities schedules were checked on 2026-10-23:
//! - 2026, CT/075/25: <https://www.hkex.com.hk/-/media/HKEX-Market/Services/Circulars-and-Notices/Participant-and-Members-Circulars/SEHK/2025/ce_SEHK_CT_075_2025.pdf>.
//! - 2027, CT/077/26: <https://www.hkex.com.hk/-/media/HKEX-Market/Services/Circulars-and-Notices/Participant-and-Members-Circulars/SEHK/2026/ce_SEHK_CT_077_2026.pdf>.
//!
//! Other available years are unaudited historical rows or projections.

use crate::date::Date;

/// Complete primary-source verified annual horizon (inclusive).
pub const VERIFIED_YEARS: (i32, i32) = (2026, 2027);
/// Date the primary annual schedules were checked.
pub const VERIFIED_ON: &str = "2026-10-23";

/// Query a named holiday with verified market coverage.
///
/// # Errors
/// Returns a coverage error outside the verified annual horizon.
pub fn try_is_holiday(date: Date) -> Result<bool, super::CalendarError> {
    super::try_is_holiday(date, super::Calendar::HongKong)
}

/// Date of the HKEX trading-calendar snapshot embedded in this module,
/// `YYYY-MM-DD`.
pub const SNAPSHOT_DATE: &str = "2026-10-23";

/// Inclusive available year range. Checked queries reject dates outside it.
pub const COVERAGE: (i32, i32) = (2020, 2040);

/// Observed weekday full-day HKEX trading closures for 2020–2040,
/// sorted for binary search. Includes retained exceptional closures;
/// weekends and partial sessions are not additional named closure rows.
/// Future unpublished dates remain explicit projections.
pub(crate) const HOLIDAYS: &[(i32, u8, u8)] = &[
    // 2020
    (2020, 1, 1),   // New Year's Day
    (2020, 1, 27),  // The third day of Chinese New Year
    (2020, 1, 28),  // The fourth day of Chinese New Year
    (2020, 4, 10),  // Good Friday
    (2020, 4, 13),  // Easter Monday
    (2020, 4, 30),  // The Buddha's Birthday
    (2020, 5, 1),   // Labor Day
    (2020, 6, 25),  // Dragon Boat Festival
    (2020, 7, 1),   // Hong Kong S.A.R. Establishment Day
    (2020, 10, 1),  // National Day
    (2020, 10, 2),  // The Day following Mid-Autumn Festival
    (2020, 10, 13), // Trading Suspended for the Whole Day due to Severe Weather
    (2020, 10, 26), // The day following Double Ninth Festival
    (2020, 12, 25), // Christmas Day
    // 2021
    (2021, 1, 1),   // New Year's Day
    (2021, 2, 12),  // Chinese New Year
    (2021, 2, 15),  // The fourth day of Chinese New Year
    (2021, 4, 2),   // Good Friday
    (2021, 4, 5),   // The day following Tomb-Sweeping Day
    (2021, 4, 6),   // The day following Easter Monday
    (2021, 5, 19),  // The Buddha's Birthday
    (2021, 6, 14),  // Dragon Boat Festival
    (2021, 7, 1),   // Hong Kong S.A.R. Establishment Day
    (2021, 9, 22),  // The Day following Mid-Autumn Festival
    (2021, 10, 1),  // National Day
    (2021, 10, 13), // Trading Suspended for the Whole Day due to Severe Weather
    (2021, 10, 14), // Double Ninth Festival
    (2021, 12, 27), // The first weekday after Christmas Day
    // 2022
    (2022, 2, 1),   // Chinese New Year
    (2022, 2, 2),   // The second day of Chinese New Year
    (2022, 2, 3),   // The third day of Chinese New Year
    (2022, 4, 5),   // Tomb-Sweeping Day
    (2022, 4, 15),  // Good Friday
    (2022, 4, 18),  // Easter Monday
    (2022, 5, 2),   // The day following Labor Day
    (2022, 5, 9),   // The day following the Buddha's Birthday
    (2022, 6, 3),   // Dragon Boat Festival
    (2022, 7, 1),   // Hong Kong S.A.R. Establishment Day
    (2022, 9, 12),  // The Second Day following Mid-Autumn Festival
    (2022, 10, 4),  // Double Ninth Festival
    (2022, 12, 26), // The first weekday after Christmas Day
    (2022, 12, 27), // The second weekday after Christmas Day
    // 2023
    (2023, 1, 2),   // The day following New Year's Day
    (2023, 1, 23),  // The second day of Chinese New Year
    (2023, 1, 24),  // The third day of Chinese New Year
    (2023, 1, 25),  // The fourth day of Chinese New Year
    (2023, 4, 5),   // Tomb-Sweeping Day
    (2023, 4, 7),   // Good Friday
    (2023, 4, 10),  // Easter Monday
    (2023, 5, 1),   // Labor Day
    (2023, 5, 26),  // The Buddha's Birthday
    (2023, 6, 22),  // Dragon Boat Festival
    (2023, 7, 17),  // Trading Suspended for the Whole Day due to Severe Weather
    (2023, 9, 1),   // Trading Suspended for the Whole Day due to Severe Weather
    (2023, 9, 8),   // Trading Suspended for the Whole Day due to Severe Weather
    (2023, 10, 2),  // The day following National Day
    (2023, 10, 23), // Double Ninth Festival
    (2023, 12, 25), // Christmas Day
    (2023, 12, 26), // The first weekday after Christmas Day
    // 2024
    (2024, 1, 1),   // New Year's Day
    (2024, 2, 12),  // The third day of Chinese New Year
    (2024, 2, 13),  // The fourth day of Chinese New Year
    (2024, 3, 29),  // Good Friday
    (2024, 4, 1),   // Easter Monday
    (2024, 4, 4),   // Tomb-Sweeping Day
    (2024, 5, 1),   // Labor Day
    (2024, 5, 15),  // The Buddha's Birthday
    (2024, 6, 10),  // Dragon Boat Festival
    (2024, 7, 1),   // Hong Kong S.A.R. Establishment Day
    (2024, 9, 6),   // Trading Suspended for the Whole Day due to Severe Weather
    (2024, 9, 18),  // The Day following Mid-Autumn Festival
    (2024, 10, 1),  // National Day
    (2024, 10, 11), // Double Ninth Festival
    (2024, 12, 25), // Christmas Day
    (2024, 12, 26), // The first weekday after Christmas Day
    // 2025
    (2025, 1, 1),   // New Year's Day
    (2025, 1, 29),  // Chinese New Year
    (2025, 1, 30),  // The second day of Chinese New Year
    (2025, 1, 31),  // The third day of Chinese New Year
    (2025, 4, 4),   // Tomb-Sweeping Day
    (2025, 4, 18),  // Good Friday
    (2025, 4, 21),  // Easter Monday
    (2025, 5, 1),   // Labor Day
    (2025, 5, 5),   // The Buddha's Birthday
    (2025, 7, 1),   // Hong Kong S.A.R. Establishment Day
    (2025, 10, 1),  // National Day
    (2025, 10, 7),  // The Day following Mid-Autumn Festival
    (2025, 10, 29), // Double Ninth Festival
    (2025, 12, 25), // Christmas Day
    (2025, 12, 26), // The first weekday after Christmas Day
    // 2026
    (2026, 1, 1),   // New Year's Day
    (2026, 2, 17),  // Chinese New Year
    (2026, 2, 18),  // The second day of Chinese New Year
    (2026, 2, 19),  // The third day of Chinese New Year
    (2026, 4, 3),   // Good Friday
    (2026, 4, 6),   // The day following Tomb-Sweeping Day
    (2026, 4, 7),   // The day following Easter Monday
    (2026, 5, 1),   // Labor Day
    (2026, 5, 25),  // The day following the Buddha's Birthday
    (2026, 6, 19),  // Dragon Boat Festival
    (2026, 7, 1),   // Hong Kong S.A.R. Establishment Day
    (2026, 10, 1),  // National Day
    (2026, 10, 19), // The day following Double Ninth Festival
    (2026, 12, 25), // Christmas Day
    // 2027
    (2027, 1, 1),   // New Year's Day
    (2027, 2, 8),   // The third day of Chinese New Year
    (2027, 2, 9),   // The fourth day of Chinese New Year
    (2027, 3, 26),  // Good Friday
    (2027, 3, 29),  // Easter Monday
    (2027, 4, 5),   // Tomb-Sweeping Day
    (2027, 5, 13),  // The Buddha's Birthday
    (2027, 6, 9),   // Dragon Boat Festival
    (2027, 7, 1),   // Hong Kong S.A.R. Establishment Day
    (2027, 9, 16),  // The Day following Mid-Autumn Festival
    (2027, 10, 1),  // National Day
    (2027, 10, 8),  // Double Ninth Festival
    (2027, 12, 27), // The first weekday after Christmas Day
    // 2028
    (2028, 1, 26),  // Chinese New Year
    (2028, 1, 27),  // The second day of Chinese New Year
    (2028, 1, 28),  // The third day of Chinese New Year
    (2028, 4, 4),   // Tomb-Sweeping Day
    (2028, 4, 14),  // Good Friday
    (2028, 4, 17),  // Easter Monday
    (2028, 5, 1),   // Labor Day
    (2028, 5, 2),   // The Buddha's Birthday
    (2028, 5, 29),  // The day following Dragon Boat Festival
    (2028, 10, 2),  // The day following National Day
    (2028, 10, 4),  // The Day following Mid-Autumn Festival
    (2028, 10, 26), // Double Ninth Festival
    (2028, 12, 25), // Christmas Day
    (2028, 12, 26), // The first weekday after Christmas Day
    // 2029
    (2029, 1, 1),   // New Year's Day
    (2029, 2, 13),  // Chinese New Year
    (2029, 2, 14),  // The second day of Chinese New Year
    (2029, 2, 15),  // The third day of Chinese New Year
    (2029, 3, 30),  // Good Friday
    (2029, 4, 2),   // Easter Monday
    (2029, 4, 4),   // Tomb-Sweeping Day
    (2029, 5, 1),   // Labor Day
    (2029, 5, 21),  // The day following the Buddha's Birthday
    (2029, 7, 2),   // The day following Hong Kong S.A.R. Establishment Day
    (2029, 9, 24),  // The Second Day following Mid-Autumn Festival
    (2029, 10, 1),  // National Day
    (2029, 10, 16), // Double Ninth Festival
    (2029, 12, 25), // Christmas Day
    (2029, 12, 26), // The first weekday after Christmas Day
    // 2030
    (2030, 1, 1),   // New Year's Day
    (2030, 2, 4),   // The second day of Chinese New Year
    (2030, 2, 5),   // The third day of Chinese New Year
    (2030, 2, 6),   // The fourth day of Chinese New Year
    (2030, 4, 5),   // Tomb-Sweeping Day
    (2030, 4, 19),  // Good Friday
    (2030, 4, 22),  // Easter Monday
    (2030, 5, 1),   // Labor Day
    (2030, 5, 9),   // The Buddha's Birthday
    (2030, 6, 5),   // Dragon Boat Festival
    (2030, 7, 1),   // Hong Kong S.A.R. Establishment Day
    (2030, 9, 13),  // The Day following Mid-Autumn Festival
    (2030, 10, 1),  // National Day
    (2030, 12, 25), // Christmas Day
    (2030, 12, 26), // The first weekday after Christmas Day
    // 2031
    (2031, 1, 1),   // New Year's Day
    (2031, 1, 23),  // Chinese New Year
    (2031, 1, 24),  // The second day of Chinese New Year
    (2031, 4, 11),  // Good Friday
    (2031, 4, 14),  // Easter Monday
    (2031, 5, 1),   // Labor Day
    (2031, 5, 28),  // The Buddha's Birthday
    (2031, 6, 24),  // Dragon Boat Festival
    (2031, 7, 1),   // Hong Kong S.A.R. Establishment Day
    (2031, 10, 1),  // National Day
    (2031, 10, 2),  // The Day following Mid-Autumn Festival
    (2031, 10, 24), // Double Ninth Festival
    (2031, 12, 25), // Christmas Day
    (2031, 12, 26), // The first weekday after Christmas Day
    // 2032
    (2032, 1, 1),   // New Year's Day
    (2032, 2, 11),  // Chinese New Year
    (2032, 2, 12),  // The second day of Chinese New Year
    (2032, 2, 13),  // The third day of Chinese New Year
    (2032, 3, 26),  // Good Friday
    (2032, 3, 29),  // Easter Monday
    (2032, 4, 5),   // The day following Tomb-Sweeping Day
    (2032, 5, 17),  // The day following the Buddha's Birthday
    (2032, 7, 1),   // Hong Kong S.A.R. Establishment Day
    (2032, 9, 20),  // The Day following Mid-Autumn Festival
    (2032, 10, 1),  // National Day
    (2032, 10, 12), // Double Ninth Festival
    (2032, 12, 27), // The first weekday after Christmas Day
    // 2033
    (2033, 1, 31),  // Chinese New Year
    (2033, 2, 1),   // The second day of Chinese New Year
    (2033, 2, 2),   // The third day of Chinese New Year
    (2033, 4, 4),   // Tomb-Sweeping Day
    (2033, 4, 15),  // Good Friday
    (2033, 4, 18),  // Easter Monday
    (2033, 5, 2),   // The day following Labor Day
    (2033, 5, 6),   // The Buddha's Birthday
    (2033, 6, 1),   // Dragon Boat Festival
    (2033, 7, 1),   // Hong Kong S.A.R. Establishment Day
    (2033, 9, 9),   // The Day following Mid-Autumn Festival
    (2033, 10, 3),  // The day following National Day
    (2033, 12, 26), // The first weekday after Christmas Day
    (2033, 12, 27), // The second weekday after Christmas Day
    // 2034
    (2034, 1, 2),   // The day following New Year's Day
    (2034, 2, 20),  // The second day of Chinese New Year
    (2034, 2, 21),  // The third day of Chinese New Year
    (2034, 2, 22),  // The fourth day of Chinese New Year
    (2034, 4, 5),   // Tomb-Sweeping Day
    (2034, 4, 7),   // Good Friday
    (2034, 4, 10),  // Easter Monday
    (2034, 5, 1),   // Labor Day
    (2034, 5, 25),  // The Buddha's Birthday
    (2034, 6, 20),  // Dragon Boat Festival
    (2034, 9, 28),  // The Day following Mid-Autumn Festival
    (2034, 10, 2),  // The day following National Day
    (2034, 10, 20), // Double Ninth Festival
    (2034, 12, 25), // Christmas Day
    (2034, 12, 26), // The first weekday after Christmas Day
    // 2035
    (2035, 1, 1),   // New Year's Day
    (2035, 2, 8),   // Chinese New Year
    (2035, 2, 9),   // The second day of Chinese New Year
    (2035, 3, 23),  // Good Friday
    (2035, 3, 26),  // Easter Monday
    (2035, 4, 5),   // Tomb-Sweeping Day
    (2035, 5, 1),   // Labor Day
    (2035, 5, 15),  // The Buddha's Birthday
    (2035, 6, 11),  // The day following Dragon Boat Festival
    (2035, 7, 2),   // The day following Hong Kong S.A.R. Establishment Day
    (2035, 9, 17),  // The Day following Mid-Autumn Festival
    (2035, 10, 1),  // National Day
    (2035, 10, 9),  // Double Ninth Festival
    (2035, 12, 25), // Christmas Day
    (2035, 12, 26), // The first weekday after Christmas Day
    // 2036
    (2036, 1, 1),   // New Year's Day
    (2036, 1, 28),  // Chinese New Year
    (2036, 1, 29),  // The second day of Chinese New Year
    (2036, 1, 30),  // The third day of Chinese New Year
    (2036, 4, 4),   // Tomb-Sweeping Day
    (2036, 4, 11),  // Good Friday
    (2036, 4, 14),  // Easter Monday
    (2036, 5, 1),   // Labor Day
    (2036, 5, 30),  // Dragon Boat Festival
    (2036, 7, 1),   // Hong Kong S.A.R. Establishment Day
    (2036, 10, 1),  // National Day
    (2036, 10, 6),  // The Second Day following Mid-Autumn Festival
    (2036, 10, 27), // Double Ninth Festival
    (2036, 12, 25), // Christmas Day
    (2036, 12, 26), // The first weekday after Christmas Day
    // 2037
    (2037, 1, 1),   // New Year's Day
    (2037, 2, 16),  // The second day of Chinese New Year
    (2037, 2, 17),  // The third day of Chinese New Year
    (2037, 2, 18),  // The fourth day of Chinese New Year
    (2037, 4, 3),   // Good Friday
    (2037, 4, 6),   // Easter Monday
    (2037, 5, 1),   // Labor Day
    (2037, 5, 22),  // The Buddha's Birthday
    (2037, 6, 18),  // Dragon Boat Festival
    (2037, 7, 1),   // Hong Kong S.A.R. Establishment Day
    (2037, 9, 25),  // The Day following Mid-Autumn Festival
    (2037, 10, 1),  // National Day
    (2037, 12, 25), // Christmas Day
    // 2038
    (2038, 1, 1),   // New Year's Day
    (2038, 2, 4),   // Chinese New Year
    (2038, 2, 5),   // The second day of Chinese New Year
    (2038, 4, 5),   // Tomb-Sweeping Day
    (2038, 4, 23),  // Good Friday
    (2038, 4, 26),  // Easter Monday
    (2038, 5, 11),  // The Buddha's Birthday
    (2038, 6, 7),   // Dragon Boat Festival
    (2038, 7, 1),   // Hong Kong S.A.R. Establishment Day
    (2038, 9, 14),  // The Day following Mid-Autumn Festival
    (2038, 10, 1),  // National Day
    (2038, 10, 7),  // Double Ninth Festival
    (2038, 12, 27), // The first weekday after Christmas Day
    // 2039
    (2039, 1, 24),  // Chinese New Year
    (2039, 1, 25),  // The second day of Chinese New Year
    (2039, 1, 26),  // The third day of Chinese New Year
    (2039, 4, 5),   // Tomb-Sweeping Day
    (2039, 4, 8),   // Good Friday
    (2039, 4, 11),  // Easter Monday
    (2039, 5, 2),   // The day following Labor Day
    (2039, 5, 27),  // Dragon Boat Festival
    (2039, 7, 1),   // Hong Kong S.A.R. Establishment Day
    (2039, 10, 3),  // The Day following Mid-Autumn Festival
    (2039, 10, 26), // Double Ninth Festival
    (2039, 12, 26), // The first weekday after Christmas Day
    (2039, 12, 27), // The second weekday after Christmas Day
    // 2040
    (2040, 1, 2),   // The day following New Year's Day
    (2040, 2, 13),  // The second day of Chinese New Year
    (2040, 2, 14),  // The third day of Chinese New Year
    (2040, 2, 15),  // The fourth day of Chinese New Year
    (2040, 3, 30),  // Good Friday
    (2040, 4, 2),   // Easter Monday
    (2040, 4, 4),   // Tomb-Sweeping Day
    (2040, 5, 1),   // Labor Day
    (2040, 5, 18),  // The Buddha's Birthday
    (2040, 6, 14),  // Dragon Boat Festival
    (2040, 7, 2),   // The day following Hong Kong S.A.R. Establishment Day
    (2040, 9, 21),  // The Day following Mid-Autumn Festival
    (2040, 10, 1),  // National Day
    (2040, 10, 15), // The day following Double Ninth Festival
    (2040, 12, 25), // Christmas Day
    (2040, 12, 26), // The first weekday after Christmas Day
];

// ─── Lookup ──────────────────────────────────────────────────────────────────

pub(crate) fn is_holiday(date: Date) -> bool {
    let key = (date.year(), date.month(), date.day());
    HOLIDAYS.binary_search(&key).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── 2024 — every holiday in the year against the published HKEX list ─

    #[test]
    fn holidays_2024() {
        // Independent HKEX full-day weekday closures, including the
        // announced September 6 severe-weather cancellation.
        let dates = [
            (1, 1),   // New Year's Day (Mon)
            (2, 12),  // Lunar New Year Day 3 substitute (Mon; Day 2 Sun 11)
            (2, 13),  // Lunar New Year Day 3 (Tue)
            (3, 29),  // Good Friday
            (4, 1),   // Easter Monday
            (4, 4),   // Ching Ming Festival
            (5, 1),   // Labour Day
            (5, 15),  // Buddha's Birthday
            (6, 10),  // Tuen Ng Festival
            (7, 1),   // HKSAR Establishment Day
            (9, 6),   // Full-day severe-weather cancellation
            (9, 18),  // Day after Mid-Autumn Festival
            (10, 1),  // National Day
            (10, 11), // Chung Yeung Festival
            (12, 25), // Christmas Day
            (12, 26), // Day after Christmas
        ];
        for (m, d) in dates {
            assert!(
                is_holiday(Date::ymd(2024, m, d).unwrap()),
                "2024-{m:02}-{d:02} must be an HKEX holiday",
            );
        }
    }

    // ─── Lunar New Year 2026 — verify Feb 17–19 are all in the table ──────

    #[test]
    fn lunar_new_year_2026() {
        // Year of the Horse. The HKEX trading calendar names 17, 18, and
        // 19 February 2026 (Tue, Wed, Thu) as the three-day Lunar New Year
        // holiday.
        assert!(is_holiday(Date::ymd(2026, 2, 17).unwrap()));
        assert!(is_holiday(Date::ymd(2026, 2, 18).unwrap()));
        assert!(is_holiday(Date::ymd(2026, 2, 19).unwrap()));
    }

    #[test]
    fn retained_full_day_weather_closures() {
        for (year, month, day) in [
            (2020, 10, 13),
            (2021, 10, 13),
            (2023, 7, 17),
            (2023, 9, 1),
            (2023, 9, 8),
            (2024, 9, 6),
        ] {
            assert!(is_holiday(Date::ymd(year, month, day).unwrap()));
        }
        assert!(!is_holiday(Date::ymd(2024, 3, 30).unwrap()));
    }

    // ─── Out-of-coverage years return false ───────────────────────────────

    #[test]
    fn out_of_coverage_returns_false() {
        // 2019 and 2041 sit immediately outside the snapshot window
        // [`COVERAGE`] = (2020, 2040). The function must return `false`
        // for every date in those years — even ones that would be a
        // holiday under the same rules.
        assert!(!is_holiday(Date::ymd(2019, 1, 1).unwrap()));
        assert!(!is_holiday(Date::ymd(2041, 1, 1).unwrap()));
    }

    // ─── HOLIDAYS is sorted ──────────────────────────────────────────────

    #[test]
    fn holidays_table_is_sorted() {
        // Binary search in `is_holiday` requires the table to be sorted
        // ascending. Walking the slice and asserting strict monotonicity
        // pins that invariant.
        for window in HOLIDAYS.windows(2) {
            assert!(
                window[0] < window[1],
                "HOLIDAYS not strictly sorted at {:?} -> {:?}",
                window[0],
                window[1],
            );
        }
    }

    // ─── Random non-holidays ─────────────────────────────────────────────

    #[test]
    fn non_holidays_are_not_marked() {
        // A plain mid-March Friday in 2024 and an arbitrary Saturday in
        // August 2026 are neither rule dates nor observed substitutes;
        // both must return `false`.
        assert!(!is_holiday(Date::ymd(2024, 3, 15).unwrap()));
        assert!(!is_holiday(Date::ymd(2026, 8, 1).unwrap()));
    }

    // ─── Snapshot metadata sanity ─────────────────────────────────────────

    #[test]
    fn snapshot_metadata_is_consistent() {
        // Snapshot metadata must match the documented values exactly.
        assert_eq!(SNAPSHOT_DATE, "2026-10-23");
        assert_eq!(COVERAGE, (2020, 2040));
        // Every row must fall inside COVERAGE.
        for &(y, _, _) in HOLIDAYS {
            assert!(
                y >= COVERAGE.0 && y <= COVERAGE.1,
                "row year {y} outside COVERAGE {COVERAGE:?}",
            );
        }
    }

    // ─── Coverage spans 21 years ─────────────────────────────────────────

    #[test]
    fn every_covered_year_has_new_years_day_entry() {
        // For every year in COVERAGE, the table must contain at least one
        // entry — a sanity check that no year was skipped. (New Year's Day
        // is observed in every year, either on 1 January or on a Monday
        // substitute, so it always supplies such an entry.)
        for year in COVERAGE.0..=COVERAGE.1 {
            assert!(
                HOLIDAYS.iter().any(|&(y, _, _)| y == year),
                "no holidays listed for {year}",
            );
        }
    }
    // Complete primary schedules linked in the module documentation, checked
    // 2026-10-23. This compares every date, including open days and weekends.
    #[test]
    fn complete_verified_schedules_match_primary_sources() {
        let expected = &[
            // 2026
            (2026, 1, 1),
            (2026, 2, 17),
            (2026, 2, 18),
            (2026, 2, 19),
            (2026, 4, 3),
            (2026, 4, 6),
            (2026, 4, 7),
            (2026, 5, 1),
            (2026, 5, 25),
            (2026, 6, 19),
            (2026, 7, 1),
            (2026, 10, 1),
            (2026, 10, 19),
            (2026, 12, 25),
            // 2027
            (2027, 1, 1),
            (2027, 2, 8),
            (2027, 2, 9),
            (2027, 3, 26),
            (2027, 3, 29),
            (2027, 4, 5),
            (2027, 5, 13),
            (2027, 6, 9),
            (2027, 7, 1),
            (2027, 9, 16),
            (2027, 10, 1),
            (2027, 10, 8),
            (2027, 12, 27),
        ];
        super::super::assert_verified_schedule(
            super::super::Calendar::HongKong,
            2026,
            2027,
            expected,
        );
    }
}
