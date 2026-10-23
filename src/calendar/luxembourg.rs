// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! Luxembourg full-day bank holidays, including Good Friday.
//!
//! The eleven statutory public holidays are supplemented by the banking
//! Good Friday closure. Christmas Eve closes only in the afternoon and is
//! therefore a business day in this full-day, date-only model. Intraday
//! settlement cutoffs and employee compensatory leave are not represented.
//! Holidays falling on weekends are not shifted to neighbouring weekdays.
//!
//! Easter Monday, Ascension and Whit Monday are Easter +1, +39 and +50 days;
//! Good Friday is Easter −2. Fixed holidays are 1 January, 1 May, 9 May,
//! 23 June, 15 August, 1 November, 25 December and 26 December. Europe Day
//! was introduced in 2019. The unchecked legacy rule accepts earlier dates,
//! but no historical banking-calendar validity is asserted for those dates.
//! Checked coverage starts in 2019, with 2026–2027 independently verified.
//! Outside that verified range a projection-policy opt-in is required.
//!
//! Primary sources read on 2026-10-23:
//! - Luxembourg government, legal holidays 2026–2027 and banking distinction:
//!   <https://luxembourg.public.lu/fr/vivre/qualite-de-vie/jours-feries-legaux-luxembourg.html>.
//! - Banque centrale du Luxembourg, opening days and Good Friday closure:
//!   <https://www.bcl.lu/fr/apropos/jours_douverture/index.html>.
//! - ING Luxembourg full bank-closing schedule 2026, confirming Good Friday
//!   and the Christmas Eve afternoon cutoff:
//!   <https://www.ing.lu/webing/content/siteing/fr/private-banking/contacts-and-information/information/more-information/bank-closing-days.html>.
//!
//! The BCL 2026 web table prints Ascension as 15 May; the government and ING
//! schedules give 14 May, consistent with Easter +39. The generator uses
//! 14 May and does not reproduce that transcription discrepancy.

use crate::date::Date;

/// True if `date` is a Luxembourg full-day bank holiday.
///
/// Includes Good Friday and the statutory public holidays. Christmas Eve's
/// afternoon-only closure is not a full-day holiday. Dates are not shifted
/// when they fall on weekends. Europe Day starts in 2019; no historical
/// verification is implied by this permissive legacy predicate.
/// Prefer [`super::try_is_holiday`] to enforce coverage and verification.
///
/// # Examples
///
/// ```
/// use regit_daycount::Date;
/// use regit_daycount::calendar::luxembourg;
///
/// // Luxembourg National Day (Grand Duke's official birthday) 2026.
/// assert!(luxembourg::is_holiday(Date::ymd(2026, 6, 23).unwrap()));
///
/// // The Monday before is a regular business day.
/// assert!(!luxembourg::is_holiday(Date::ymd(2026, 6, 22).unwrap()));
/// ```
#[must_use]
pub fn is_holiday(date: Date) -> bool {
    let year = date.year();
    let easter = Date::easter_sunday(year);
    let good_friday = easter.add_days(-2);
    let easter_monday = easter.add_days(1);
    let ascension = easter.add_days(39);
    let whit_monday = easter.add_days(50);

    // Fixed-date holidays — collapsed into a single `matches!` so the
    // hot path is a small jump table. The pattern groups dates that
    // share a day-of-month: (1, 5, 11) all have a "1st of the month"
    // holiday (New Year's Day, Labour Day, All Saints' Day), 23 June is
    // National Day, 15 August is Assumption, 25 and 26 December are
    // Christmas Day and Boxing Day.
    let fixed = matches!(
        (date.month(), date.day()),
        (1 | 5 | 11, 1) | (6, 23) | (8, 15) | (12, 25 | 26)
    );
    // Europe Day was gazetted only from 2019 onward.
    let europe_day = date == Date::ymd_unchecked(year, 5, 9) && year >= 2019;

    fixed
        || europe_day
        || date == good_friday
        || date == easter_monday
        || date == ascension
        || date == whit_monday
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── 2024 — every holiday in the year (Easter = 31 Mar) ──────────────

    #[test]
    fn holidays_2024() {
        // Easter Sunday 2024 = 2024-03-31, so Easter Monday = 2024-04-01,
        // Ascension = 2024-05-09 (which coincides with Europe Day this
        // year — the two dates collide in 2024), and Whit Monday =
        // 2024-05-20.
        let dates = [
            (1, 1),   // New Year's Day (Mon)
            (3, 29),  // Good Friday bank holiday
            (4, 1),   // Easter Monday
            (5, 1),   // Labour Day (Wed)
            (5, 9),   // Europe Day = Ascension Day in 2024 (Thu)
            (5, 20),  // Whit Monday
            (6, 23),  // National Day (Sun in 2024)
            (8, 15),  // Assumption of Mary
            (11, 1),  // All Saints' Day (Fri)
            (12, 25), // Christmas Day (Wed)
            (12, 26), // Boxing Day (Thu)
        ];
        for (m, d) in dates {
            assert!(
                is_holiday(Date::ymd(2024, m, d).unwrap()),
                "2024-{m:02}-{d:02} must be a Luxembourg holiday",
            );
        }
        // Ascension and Europe Day both fall on 2024-05-09 — assert the
        // collision explicitly: the date is a holiday under either rule.
        let collision = Date::ymd(2024, 5, 9).unwrap();
        assert_eq!(collision, Date::easter_sunday(2024).add_days(39));
        assert!(is_holiday(collision));
    }

    // ─── 2026 — every holiday in the year (Easter = 5 Apr) ───────────────

    #[test]
    fn holidays_2026() {
        // Easter Sunday 2026 = 2026-04-05, so Easter Monday = 2026-04-06,
        // Ascension = 2026-05-14, and Whit Monday = 2026-05-25.
        let dates = [
            (1, 1),   // New Year's Day
            (4, 3),   // Good Friday bank holiday
            (4, 6),   // Easter Monday
            (5, 1),   // Labour Day
            (5, 9),   // Europe Day
            (5, 14),  // Ascension Day
            (5, 25),  // Whit Monday
            (6, 23),  // National Day
            (8, 15),  // Assumption of Mary
            (11, 1),  // All Saints' Day
            (12, 25), // Christmas Day
            (12, 26), // Boxing Day
        ];
        for (m, d) in dates {
            assert!(
                is_holiday(Date::ymd(2026, m, d).unwrap()),
                "2026-{m:02}-{d:02} must be a Luxembourg holiday",
            );
        }
    }

    // ─── 2026 — explicit non-holidays adjacent to the rule dates ─────────

    #[test]
    fn non_holidays_2026() {
        // Each of these sits next to a real Luxembourg holiday and is a
        // common source of off-by-one regressions. 2026-01-02 is
        // Berchtoldstag in Switzerland but is NOT a Luxembourg holiday.
        // 2026-05-08 is the day before Europe Day. 2026-12-24 is
        // Christmas Eve and 2026-12-27 is the day after Boxing Day —
        // neither is gazetted.
        let dates = [
            (1, 2),   // Berchtoldstag (Swiss, not Luxembourg)
            (5, 8),   // day before Europe Day
            (12, 24), // Christmas Eve
            (12, 27), // day after Boxing Day
        ];
        for (m, d) in dates {
            assert!(
                !is_holiday(Date::ymd(2026, m, d).unwrap()),
                "2026-{m:02}-{d:02} must NOT be a Luxembourg holiday",
            );
        }
    }

    // ─── Europe Day cutover (2019 gazetting) ─────────────────────────────

    #[test]
    fn europe_day_cutover_in_2019() {
        // Europe Day was added to the labour-law schedule by the Loi du
        // 28 février 2019. Before that statute, 9 May was a regular
        // business day in Luxembourg; from 2019 onward it is a holiday.
        // This is a subtle, load-bearing distinction — the kind of
        // exact-year transition that slips past unit tests if the
        // predicate is forgotten.
        assert!(
            !is_holiday(Date::ymd(2018, 5, 9).unwrap()),
            "2018-05-09 must NOT be a Luxembourg holiday (Europe Day not yet gazetted)",
        );
        assert!(
            is_holiday(Date::ymd(2019, 5, 9).unwrap()),
            "2019-05-09 must be a Luxembourg holiday (first year of Europe Day)",
        );
        assert!(
            is_holiday(Date::ymd(2024, 5, 9).unwrap()),
            "2024-05-09 must be a Luxembourg holiday (Europe Day; also Ascension in 2024)",
        );
    }

    // ─── Whit Monday IS a Luxembourg holiday (vs. TARGET2) ───────────────

    #[test]
    fn whit_monday_is_a_holiday() {
        // Whit Monday = Easter Sunday + 50 days. In 2026 that is
        // 2026-04-05 + 50 = 2026-05-25 (Mon). Whit Monday is a
        // Luxembourg labour-law holiday — explicit regression vs.
        // TARGET2, which dropped Whit Monday when TARGET2 succeeded
        // TARGET in 2002 and reports `false` for the same date.
        let easter = Date::easter_sunday(2026);
        let whit_monday = easter.add_days(50);
        assert_eq!(whit_monday, Date::ymd(2026, 5, 25).unwrap());
        assert!(is_holiday(whit_monday));

        // Sanity check across a few more years.
        for year in [2024, 2025, 2027, 2028] {
            let wm = Date::easter_sunday(year).add_days(50);
            assert!(
                is_holiday(wm),
                "Whit Monday {year} ({wm:?}) must be a Luxembourg holiday",
            );
        }
    }

    #[test]
    fn good_friday_is_a_bank_holiday() {
        // Primary government and banking schedules distinguish this from
        // statutory public holidays. These dates are independent fixtures.
        for (year, month, day) in [(2024, 3, 29), (2025, 4, 18), (2026, 4, 3), (2027, 3, 26)] {
            assert!(is_holiday(Date::ymd(year, month, day).unwrap()));
        }
    }

    // ─── Holidays do not shift to the next business day ──────────────────

    #[test]
    fn independence_from_weekday() {
        // The function reports the rule date, not the observed date. A
        // Luxembourg holiday that falls on a weekend is still reported
        // as a holiday on its rule date — there is no observed-day shift
        // in the labour-law schedule.
        //
        // 2027-01-01 falls on a Friday and is reported as a holiday
        // (working day in the surrounding week).
        assert!(is_holiday(Date::ymd(2027, 1, 1).unwrap()));

        // 2028-01-01 falls on a Saturday. `is_holiday` still returns
        // `true` — the rule date is reported regardless of weekday; the
        // dispatcher's weekend filter (`is_business_day`) is what would
        // separately classify the date as non-business.
        assert!(is_holiday(Date::ymd(2028, 1, 1).unwrap()));

        // Christmas Day 2027 = Sat, Boxing Day 2027 = Sun. Both must
        // still be reported as Luxembourg holidays — the rule does not
        // suppress them on weekends.
        assert!(is_holiday(Date::ymd(2027, 12, 25).unwrap()));
        assert!(is_holiday(Date::ymd(2027, 12, 26).unwrap()));

        // National Day 2024 (23 June) falls on a Sunday; same expectation.
        assert!(is_holiday(Date::ymd(2024, 6, 23).unwrap()));
    }

    // ─── Easter-anchored dates against the published Computus table ──────

    #[test]
    fn easter_anchored_dates_match_computus() {
        // Published Easter Sundays for the Western (Gregorian) Computus.
        // For each year, Easter Monday = Easter + 1, Ascension = Easter
        // + 39, and Whit Monday = Easter + 50 must all be reported as
        // Luxembourg holidays.
        let cases = [
            (2024, 3, 31), // 2024-03-31
            (2025, 4, 20), // 2025-04-20
            (2026, 4, 5),  // 2026-04-05
            (2030, 4, 21), // 2030-04-21
            (2038, 4, 25), // 2038-04-25
        ];
        for (y, m, d) in cases {
            let easter = Date::ymd(y, m, d).unwrap();
            assert_eq!(
                Date::easter_sunday(y),
                easter,
                "Computus disagrees with the published Easter date for {y}",
            );
            assert!(
                is_holiday(easter.add_days(1)),
                "Easter Monday {y} must be a Luxembourg holiday",
            );
            assert!(
                is_holiday(easter.add_days(39)),
                "Ascension {y} must be a Luxembourg holiday",
            );
            assert!(
                is_holiday(easter.add_days(50)),
                "Whit Monday {y} must be a Luxembourg holiday",
            );
        }
    }
}
