// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! SIX Swiss Exchange full-day trading closures.
//!
//! This includes Christmas Eve and New Year's Eve; Swiss banking/settlement
//! calendars are different. Weekend-only nominal holidays are omitted, and
//! no weekday substitute is invented. Weekends are always non-business days.
//! Complete official annual schedules were checked on 2026-10-23:
//! - 2026: <https://www.six-group.com/en/market-data/news-tools/trading-currency-holiday-calendar.html>.
//! - 2027: <https://www.six-group.com/dam/download/the-swiss-stock-exchange/trading/trading-provisions/regulation/trading-guides/trading-calendar-2027.pdf>.
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
    super::try_is_holiday(date, super::Calendar::Switzerland)
}

/// Date of the SIX Swiss Exchange trading-calendar snapshot embedded in
/// this module, `YYYY-MM-DD`.
pub const SNAPSHOT_DATE: &str = "2026-10-23";

/// Inclusive available year range. Checked queries reject dates outside it.
pub const COVERAGE: (i32, i32) = (2020, 2040);

/// The SIX Swiss Exchange holiday table, sorted ascending by
/// `(year, month, day)`.
///
/// Each entry is an observed weekday full-day trading closure. The list
/// includes Christmas Eve and New Year's Eve, distinguishes SIX trading
/// from Swiss banking settlement, and omits weekend-only nominal dates.
/// Independent market mappings and the complete 2020–2040 comparison are
/// recorded in `tests/modules/calendar.py` and generated validation evidence
/// under `target/test-results/`.
pub(crate) const HOLIDAYS: &[(i32, u8, u8)] = &[
    // 2020
    (2020, 1, 1),   // New Year's Day
    (2020, 1, 2),   // Saint Berchtold's Day
    (2020, 4, 10),  // Good Friday
    (2020, 4, 13),  // Easter Monday
    (2020, 5, 1),   // Labor Day
    (2020, 5, 21),  // Ascension Day
    (2020, 6, 1),   // Pentecost Monday
    (2020, 12, 24), // Christmas Eve
    (2020, 12, 25), // Christmas Day
    (2020, 12, 31), // New Year's Eve
    // 2021
    (2021, 1, 1),   // New Year's Day
    (2021, 4, 2),   // Good Friday
    (2021, 4, 5),   // Easter Monday
    (2021, 5, 13),  // Ascension Day
    (2021, 5, 24),  // Pentecost Monday
    (2021, 12, 24), // Christmas Eve
    (2021, 12, 31), // New Year's Eve
    // 2022
    (2022, 4, 15),  // Good Friday
    (2022, 4, 18),  // Easter Monday
    (2022, 5, 26),  // Ascension Day
    (2022, 6, 6),   // Pentecost Monday
    (2022, 8, 1),   // National Day
    (2022, 12, 26), // Saint Stephen's Day
    // 2023
    (2023, 1, 2),   // Saint Berchtold's Day
    (2023, 4, 7),   // Good Friday
    (2023, 4, 10),  // Easter Monday
    (2023, 5, 1),   // Labor Day
    (2023, 5, 18),  // Ascension Day
    (2023, 5, 29),  // Pentecost Monday
    (2023, 8, 1),   // National Day
    (2023, 12, 25), // Christmas Day
    (2023, 12, 26), // Saint Stephen's Day
    // 2024
    (2024, 1, 1),   // New Year's Day
    (2024, 1, 2),   // Saint Berchtold's Day
    (2024, 3, 29),  // Good Friday
    (2024, 4, 1),   // Easter Monday
    (2024, 5, 1),   // Labor Day
    (2024, 5, 9),   // Ascension Day
    (2024, 5, 20),  // Pentecost Monday
    (2024, 8, 1),   // National Day
    (2024, 12, 24), // Christmas Eve
    (2024, 12, 25), // Christmas Day
    (2024, 12, 26), // Saint Stephen's Day
    (2024, 12, 31), // New Year's Eve
    // 2025
    (2025, 1, 1),   // New Year's Day
    (2025, 1, 2),   // Saint Berchtold's Day
    (2025, 4, 18),  // Good Friday
    (2025, 4, 21),  // Easter Monday
    (2025, 5, 1),   // Labor Day
    (2025, 5, 29),  // Ascension Day
    (2025, 6, 9),   // Pentecost Monday
    (2025, 8, 1),   // National Day
    (2025, 12, 24), // Christmas Eve
    (2025, 12, 25), // Christmas Day
    (2025, 12, 26), // Saint Stephen's Day
    (2025, 12, 31), // New Year's Eve
    // 2026
    (2026, 1, 1),   // New Year's Day
    (2026, 1, 2),   // Saint Berchtold's Day
    (2026, 4, 3),   // Good Friday
    (2026, 4, 6),   // Easter Monday
    (2026, 5, 1),   // Labor Day
    (2026, 5, 14),  // Ascension Day
    (2026, 5, 25),  // Pentecost Monday
    (2026, 12, 24), // Christmas Eve
    (2026, 12, 25), // Christmas Day
    (2026, 12, 31), // New Year's Eve
    // 2027
    (2027, 1, 1),   // New Year's Day
    (2027, 3, 26),  // Good Friday
    (2027, 3, 29),  // Easter Monday
    (2027, 5, 6),   // Ascension Day
    (2027, 5, 17),  // Pentecost Monday
    (2027, 12, 24), // Christmas Eve
    (2027, 12, 31), // New Year's Eve
    // 2028
    (2028, 4, 14),  // Good Friday
    (2028, 4, 17),  // Easter Monday
    (2028, 5, 1),   // Labor Day
    (2028, 5, 25),  // Ascension Day
    (2028, 6, 5),   // Pentecost Monday
    (2028, 8, 1),   // National Day
    (2028, 12, 25), // Christmas Day
    (2028, 12, 26), // Saint Stephen's Day
    // 2029
    (2029, 1, 1),   // New Year's Day
    (2029, 1, 2),   // Saint Berchtold's Day
    (2029, 3, 30),  // Good Friday
    (2029, 4, 2),   // Easter Monday
    (2029, 5, 1),   // Labor Day
    (2029, 5, 10),  // Ascension Day
    (2029, 5, 21),  // Pentecost Monday
    (2029, 8, 1),   // National Day
    (2029, 12, 24), // Christmas Eve
    (2029, 12, 25), // Christmas Day
    (2029, 12, 26), // Saint Stephen's Day
    (2029, 12, 31), // New Year's Eve
    // 2030
    (2030, 1, 1),   // New Year's Day
    (2030, 1, 2),   // Saint Berchtold's Day
    (2030, 4, 19),  // Good Friday
    (2030, 4, 22),  // Easter Monday
    (2030, 5, 1),   // Labor Day
    (2030, 5, 30),  // Ascension Day
    (2030, 6, 10),  // Pentecost Monday
    (2030, 8, 1),   // National Day
    (2030, 12, 24), // Christmas Eve
    (2030, 12, 25), // Christmas Day
    (2030, 12, 26), // Saint Stephen's Day
    (2030, 12, 31), // New Year's Eve
    // 2031
    (2031, 1, 1),   // New Year's Day
    (2031, 1, 2),   // Saint Berchtold's Day
    (2031, 4, 11),  // Good Friday
    (2031, 4, 14),  // Easter Monday
    (2031, 5, 1),   // Labor Day
    (2031, 5, 22),  // Ascension Day
    (2031, 6, 2),   // Pentecost Monday
    (2031, 8, 1),   // National Day
    (2031, 12, 24), // Christmas Eve
    (2031, 12, 25), // Christmas Day
    (2031, 12, 26), // Saint Stephen's Day
    (2031, 12, 31), // New Year's Eve
    // 2032
    (2032, 1, 1),   // New Year's Day
    (2032, 1, 2),   // Saint Berchtold's Day
    (2032, 3, 26),  // Good Friday
    (2032, 3, 29),  // Easter Monday
    (2032, 5, 6),   // Ascension Day
    (2032, 5, 17),  // Pentecost Monday
    (2032, 12, 24), // Christmas Eve
    (2032, 12, 31), // New Year's Eve
    // 2033
    (2033, 4, 15),  // Good Friday
    (2033, 4, 18),  // Easter Monday
    (2033, 5, 26),  // Ascension Day
    (2033, 6, 6),   // Pentecost Monday
    (2033, 8, 1),   // National Day
    (2033, 12, 26), // Saint Stephen's Day
    // 2034
    (2034, 1, 2),   // Saint Berchtold's Day
    (2034, 4, 7),   // Good Friday
    (2034, 4, 10),  // Easter Monday
    (2034, 5, 1),   // Labor Day
    (2034, 5, 18),  // Ascension Day
    (2034, 5, 29),  // Pentecost Monday
    (2034, 8, 1),   // National Day
    (2034, 12, 25), // Christmas Day
    (2034, 12, 26), // Saint Stephen's Day
    // 2035
    (2035, 1, 1),   // New Year's Day
    (2035, 1, 2),   // Saint Berchtold's Day
    (2035, 3, 23),  // Good Friday
    (2035, 3, 26),  // Easter Monday
    (2035, 5, 1),   // Labor Day
    (2035, 5, 3),   // Ascension Day
    (2035, 5, 14),  // Pentecost Monday
    (2035, 8, 1),   // National Day
    (2035, 12, 24), // Christmas Eve
    (2035, 12, 25), // Christmas Day
    (2035, 12, 26), // Saint Stephen's Day
    (2035, 12, 31), // New Year's Eve
    // 2036
    (2036, 1, 1),   // New Year's Day
    (2036, 1, 2),   // Saint Berchtold's Day
    (2036, 4, 11),  // Good Friday
    (2036, 4, 14),  // Easter Monday
    (2036, 5, 1),   // Labor Day
    (2036, 5, 22),  // Ascension Day
    (2036, 6, 2),   // Pentecost Monday
    (2036, 8, 1),   // National Day
    (2036, 12, 24), // Christmas Eve
    (2036, 12, 25), // Christmas Day
    (2036, 12, 26), // Saint Stephen's Day
    (2036, 12, 31), // New Year's Eve
    // 2037
    (2037, 1, 1),   // New Year's Day
    (2037, 1, 2),   // Saint Berchtold's Day
    (2037, 4, 3),   // Good Friday
    (2037, 4, 6),   // Easter Monday
    (2037, 5, 1),   // Labor Day
    (2037, 5, 14),  // Ascension Day
    (2037, 5, 25),  // Pentecost Monday
    (2037, 12, 24), // Christmas Eve
    (2037, 12, 25), // Christmas Day
    (2037, 12, 31), // New Year's Eve
    // 2038
    (2038, 1, 1),   // New Year's Day
    (2038, 4, 23),  // Good Friday
    (2038, 4, 26),  // Easter Monday
    (2038, 6, 3),   // Ascension Day
    (2038, 6, 14),  // Pentecost Monday
    (2038, 12, 24), // Christmas Eve
    (2038, 12, 31), // New Year's Eve
    // 2039
    (2039, 4, 8),   // Good Friday
    (2039, 4, 11),  // Easter Monday
    (2039, 5, 19),  // Ascension Day
    (2039, 5, 30),  // Pentecost Monday
    (2039, 8, 1),   // National Day
    (2039, 12, 26), // Saint Stephen's Day
    // 2040
    (2040, 1, 2),   // Saint Berchtold's Day
    (2040, 3, 30),  // Good Friday
    (2040, 4, 2),   // Easter Monday
    (2040, 5, 1),   // Labor Day
    (2040, 5, 10),  // Ascension Day
    (2040, 5, 21),  // Pentecost Monday
    (2040, 8, 1),   // National Day
    (2040, 12, 24), // Christmas Eve
    (2040, 12, 25), // Christmas Day
    (2040, 12, 26), // Saint Stephen's Day
    (2040, 12, 31), // New Year's Eve
];

pub(crate) fn is_holiday(date: Date) -> bool {
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

    // ─── 2024 — every holiday in the year (Easter = 31 Mar) ──────────────

    #[test]
    fn holidays_2024() {
        // Easter Sunday 2024 = 2024-03-31, so Good Friday = 2024-03-29,
        // Easter Monday = 2024-04-01, Ascension = Mar 31 + 39 = May 9,
        // Whit Monday = Mar 31 + 50 = May 20.
        let dates = [
            (1, 1),   // New Year's Day
            (1, 2),   // Berchtoldstag
            (3, 29),  // Good Friday
            (4, 1),   // Easter Monday
            (5, 1),   // Labour Day
            (5, 9),   // Ascension Day
            (5, 20),  // Whit Monday
            (8, 1),   // Swiss National Day
            (12, 24), // Christmas Eve: trading closed, settlement open
            (12, 25), // Christmas Day
            (12, 26), // St. Stephen's Day
            (12, 31), // New Year's Eve: trading closed, settlement open
        ];
        for (m, d) in dates {
            assert!(
                is_holiday(Date::ymd(2024, m, d).unwrap()),
                "2024-{m:02}-{d:02} must be a SIX holiday",
            );
        }
        // Count the 2024 entries in HOLIDAYS — must be exactly twelve.
        let count_2024 = HOLIDAYS.iter().filter(|(y, _, _)| *y == 2024).count();
        assert_eq!(
            count_2024, 12,
            "2024 must have exactly 12 SIX trading closures"
        );
    }

    // ─── 2026 — every holiday in the year (Easter = 5 Apr) ───────────────

    #[test]
    fn holidays_2026() {
        // Easter Sunday 2026 = 2026-04-05, so Good Friday = 2026-04-03,
        // Easter Monday = 2026-04-06, Ascension = May 14, Whit Monday =
        // May 25.
        let dates = [
            (1, 1),   // New Year's Day
            (1, 2),   // Berchtoldstag
            (4, 3),   // Good Friday
            (4, 6),   // Easter Monday
            (5, 1),   // Labour Day
            (5, 14),  // Ascension Day
            (5, 25),  // Whit Monday
            (12, 24), // Christmas Eve full-day trading closure
            (12, 25), // Christmas Day
            (12, 31), // New Year's Eve full-day trading closure
        ];
        for (m, d) in dates {
            assert!(
                is_holiday(Date::ymd(2026, m, d).unwrap()),
                "2026-{m:02}-{d:02} must be a SIX holiday",
            );
        }
        let count_2026 = HOLIDAYS.iter().filter(|(y, _, _)| *y == 2026).count();
        assert_eq!(count_2026, 10, "2026 must have exactly 10 SIX holidays");
    }

    // ─── Easter-derived correctness across years ─────────────────────────

    #[test]
    fn easter_derived_ascension_and_whit_monday() {
        // For each spot-checked year, Easter + 39 must be Ascension and
        // Easter + 50 must be Whit Monday — both reported as SIX holidays
        // by the embedded snapshot. This pins the Easter-anchored rows
        // to the Computus regardless of any transcription error in the
        // table.
        for year in [2024, 2026, 2030] {
            let easter = Date::easter_sunday(year);
            let ascension = easter.add_days(39);
            let whit_monday = easter.add_days(50);
            assert!(
                is_holiday(ascension),
                "Ascension {year} ({ascension:?}) must be a SIX holiday",
            );
            assert!(
                is_holiday(whit_monday),
                "Whit Monday {year} ({whit_monday:?}) must be a SIX holiday",
            );
        }
    }

    // ─── No weekend-observance shifts ────────────────────────────────────

    #[test]
    fn weekend_nominal_dates_are_not_observed_weekday_closures() {
        assert!(!is_holiday(Date::ymd(2022, 1, 1).unwrap()));
        assert!(!is_holiday(Date::ymd(2022, 1, 2).unwrap()));
        assert!(!is_holiday(Date::ymd(2022, 1, 3).unwrap()));
        assert!(!super::super::is_business_day(
            Date::ymd(2022, 1, 1).unwrap(),
            super::super::Calendar::Switzerland
        ));
    }

    // ─── Out-of-coverage queries ─────────────────────────────────────────

    #[test]
    fn out_of_coverage_returns_false() {
        assert!(
            !is_holiday(Date::ymd(2019, 1, 1).unwrap()),
            "below coverage must return false",
        );
        assert!(
            !is_holiday(Date::ymd(2041, 1, 1).unwrap()),
            "above coverage must return false",
        );
    }

    // ─── Table invariants ────────────────────────────────────────────────

    #[test]
    fn holidays_table_is_sorted_strictly_ascending() {
        for pair in HOLIDAYS.windows(2) {
            assert!(
                pair[0] < pair[1],
                "HOLIDAYS must be sorted strictly ascending; {:?} >= {:?}",
                pair[0],
                pair[1],
            );
        }
    }

    #[test]
    fn trading_snapshot_has_all_reference_closures() {
        // Full-window independent XSWX named-date set, not 10 banking days/year.
        assert_eq!(HOLIDAYS.len(), 198);
    }
    // Complete primary schedules linked in the module documentation, checked
    // 2026-10-23. This compares every date, including open days and weekends.
    #[test]
    fn complete_verified_schedules_match_primary_sources() {
        let expected = &[
            // 2026
            (2026, 1, 1),
            (2026, 1, 2),
            (2026, 4, 3),
            (2026, 4, 6),
            (2026, 5, 1),
            (2026, 5, 14),
            (2026, 5, 25),
            (2026, 12, 24),
            (2026, 12, 25),
            (2026, 12, 31),
            // 2027
            (2027, 1, 1),
            (2027, 3, 26),
            (2027, 3, 29),
            (2027, 5, 6),
            (2027, 5, 17),
            (2027, 12, 24),
            (2027, 12, 31),
        ];
        super::super::assert_verified_schedule(
            super::super::Calendar::Switzerland,
            2026,
            2027,
            expected,
        );
    }
}
