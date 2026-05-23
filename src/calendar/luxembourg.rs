// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! Luxembourg — the national bank-holiday calendar (legal rule).
//!
//! The Grand Duchy of Luxembourg fixes its national bank holidays by
//! statute — the *Loi du 21 juillet 1928 sur le droit du travail* (as
//! subsequently amended), with Europe Day added by the *Loi du 28 février
//! 2019 fixant les jours fériés légaux*. The eleven dates the statute
//! enumerates are the days on which Luxembourg banks (including the
//! Banque centrale du Luxembourg and Spuerkeess / BCEE) close, and are
//! the calendar a Luxembourg-domiciled UCITS or SIF fund typically
//! composites with TARGET2 when computing NAV.
//!
//! Like TARGET2, Luxembourg is **fully rule-based**: every holiday is
//! either a fixed civil date or an Easter-derived date that
//! [`Date::easter_sunday`] already gives us, so no dated snapshot is
//! stored and the rule is exact for every year in the supported [`Date`]
//! range.
//!
//! The eleven holiday dates are:
//!
//! ```text
//!  1.  1 January       New Year's Day                                (fixed)
//!  2.  Easter Monday   Easter Sunday + 1 day                         (Easter-derived)
//!  3.  1 May           Labour Day                                    (fixed)
//!  4.  9 May           Europe Day                                    (fixed, from 2019)
//!  5.  Ascension Day   Easter Sunday + 39 days                       (Easter-derived)
//!  6.  Whit Monday     Easter Sunday + 50 days                       (Easter-derived)
//!  7. 23 June          National Day (Grand Duke's official birthday) (fixed)
//!  8. 15 August        Assumption of Mary                            (fixed)
//!  9.  1 November      All Saints' Day                               (fixed)
//! 10. 25 December      Christmas Day                                 (fixed)
//! 11. 26 December      Boxing Day / St. Stephen's Day                (fixed)
//! ```
//!
//! Europe Day (9 May) was gazetted only by the 2019 amendment; this
//! module returns `true` for 9 May only in years ≥ 2019, and `false`
//! for 9 May in 2018 and earlier. Good Friday is **not** a Luxembourg
//! bank holiday — banks open on Good Friday, despite it being a *jour
//! férié religieux* — and this module returns `false` for it. Whit
//! Monday **is** a Luxembourg holiday (unlike TARGET2, which dropped
//! Whit Monday when it replaced TARGET in 2002).
//!
//! Like TARGET2, Luxembourg does **not** shift holidays falling on a
//! weekend to a neighbouring weekday: the rule date is reported in
//! every year, and weekend handling is the responsibility of the
//! caller's business-day predicate (see the `is_business_day`
//! dispatcher in [`crate::calendar`]).
//!
//! # Algorithm
//!
//! ```text
//! easter        = easter_sunday(date.year())          // Computus / Meeus
//! easter_monday = easter + 1 day
//! ascension     = easter + 39 days
//! whit_monday   = easter + 50 days
//!
//! is_holiday(date) =
//!     date == (year,  1,  1)                       ||  // New Year's Day
//!     date == easter_monday                        ||
//!     date == (year,  5,  1)                       ||  // Labour Day
//!     (date == (year, 5, 9) && year >= 2019)       ||  // Europe Day
//!     date == ascension                            ||
//!     date == whit_monday                          ||
//!     date == (year,  6, 23)                       ||  // National Day
//!     date == (year,  8, 15)                       ||  // Assumption
//!     date == (year, 11,  1)                       ||  // All Saints'
//!     date == (year, 12, 25)                       ||  // Christmas Day
//!     date == (year, 12, 26)                            // Boxing Day
//! ```
//!
//! # Worked example — 2026
//!
//! ```text
//! easter_sunday(2026) = 2026-04-05  (Computus)
//! easter_monday       = 2026-04-06
//! ascension           = 2026-05-14
//! whit_monday         = 2026-05-25
//!
//! holidays(2026) = { 2026-01-01, 2026-04-06, 2026-05-01, 2026-05-09,
//!                    2026-05-14, 2026-05-25, 2026-06-23, 2026-08-15,
//!                    2026-11-01, 2026-12-25, 2026-12-26 }
//! ```
//!
//! # References
//!
//! - *Loi du 21 juillet 1928 sur le droit du travail*, as amended —
//!   the statutory list of Luxembourg jours fériés légaux.
//! - *Loi du 28 février 2019 fixant les jours fériés légaux dans le
//!   secteur privé* — adds Europe Day (9 May) to the schedule from
//!   2019 onward.
//! - Banque centrale du Luxembourg (BCL), *Jours de fermeture* —
//!   reproduces the bank closing-day list.
//! - Jean Meeus, *Astronomical Algorithms*, 2nd ed., Willmann-Bell, 1998,
//!   §8 "The date of Easter" — the Computus used by [`Date::easter_sunday`].

use crate::date::Date;

/// True if `date` is a Luxembourg national bank holiday.
///
/// Luxembourg, like TARGET2, is fully rule-based — the eleven holidays
/// are derived from the year alone (Easter via the Computus algorithm
/// in [`Date::easter_sunday`]), so no dated table is stored and the
/// calendar is exact for every year in the supported `Date` range.
///
/// Weekends are NOT classified as holidays here — they are handled by
/// the caller's business-day predicate (see the `is_business_day`
/// dispatcher in [`crate::calendar`]). Holidays falling on a Saturday
/// or Sunday are reported on their rule date with no shift.
///
/// Europe Day (9 May) was gazetted only by the *Loi du 28 février 2019*;
/// this function returns `true` for 9 May only in years ≥ 2019.
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

    fixed || europe_day || date == easter_monday || date == ascension || date == whit_monday
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

    // ─── Good Friday is NOT a Luxembourg holiday ─────────────────────────

    #[test]
    fn good_friday_is_not_a_holiday() {
        // Many sources misclassify Good Friday as a Luxembourg holiday
        // because it is widely religious; the labour-law schedule under
        // the Loi du 21 juillet 1928 does NOT include it, and Luxembourg
        // banks open on Good Friday. Explicit regression: 2024-03-29
        // (Good Friday — Easter 2024 = 2024-03-31, so Easter − 2 days =
        // 2024-03-29) must return `false`.
        let easter_2024 = Date::easter_sunday(2024);
        let good_friday_2024 = easter_2024.add_days(-2);
        assert_eq!(good_friday_2024, Date::ymd(2024, 3, 29).unwrap());
        assert!(
            !is_holiday(good_friday_2024),
            "Good Friday 2024 (2024-03-29) must NOT be a Luxembourg holiday",
        );

        // Sanity check across a few more years.
        for year in [2025, 2026, 2027, 2028] {
            let gf = Date::easter_sunday(year).add_days(-2);
            assert!(
                !is_holiday(gf),
                "Good Friday {year} ({gf:?}) must NOT be a Luxembourg holiday",
            );
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
