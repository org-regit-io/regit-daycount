// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! TARGET2 — the Eurosystem closing-days calendar (ECB fixed rule).
//!
//! TARGET2 (Trans-European Automated Real-time Gross settlement Express
//! Transfer system) is the euro-area large-value payment system, and its
//! closing-days schedule defines the "TARGET2 business day" used by every
//! EUR-denominated cashflow this crate processes. Unlike the exchange and
//! bank-holiday calendars catalogued in the other submodules of
//! [`crate::calendar`], TARGET2 is **fully rule-based**: the European
//! Central Bank fixes exactly six closing dates per year, derivable from
//! the year alone, and that schedule has not changed since TARGET2 went
//! live in 2008. No dated snapshot is stored — the rule is exact for every
//! year in the supported [`Date`] range.
//!
//! The six closing dates are:
//!
//! ```text
//! 1.  1 January       New Year's Day                  (fixed)
//! 2.  Good Friday     Easter Sunday − 2 days          (Easter-derived)
//! 3.  Easter Monday   Easter Sunday + 1 day           (Easter-derived)
//! 4.  1 May           Labour Day                      (fixed)
//! 5.  25 December     Christmas Day                   (fixed)
//! 6.  26 December     Christmas Holiday / Boxing Day  (fixed)
//! ```
//!
//! Whit Monday (Easter + 49 days) was a TARGET closing day in the
//! original 1999–2001 schedule and was removed when TARGET2 succeeded
//! TARGET in 2002; it is **not** a TARGET2 holiday and this module does
//! not return `true` for it.
//!
//! # Algorithm
//!
//! ```text
//! easter        = easter_sunday(date.year())          // Computus / Meeus
//! good_friday   = easter − 2 days
//! easter_monday = easter + 1 day
//!
//! is_holiday(date) =
//!     date == (year,  1,  1)   ||   // New Year's Day
//!     date == good_friday      ||
//!     date == easter_monday    ||
//!     date == (year,  5,  1)   ||   // Labour Day
//!     date == (year, 12, 25)   ||   // Christmas Day
//!     date == (year, 12, 26)        // Christmas Holiday
//! ```
//!
//! Weekends (Saturday and Sunday) are **not** classified as holidays by
//! this function. They are the responsibility of the caller's business-day
//! predicate (see the `is_business_day` dispatcher in [`crate::calendar`]),
//! which composes a weekend test with this holiday test. The rule dates
//! themselves are reported unconditionally — a holiday is not "shifted"
//! when it falls on a Saturday or Sunday; the TARGET2 schedule simply
//! coincides with the weekend in that case.
//!
//! # Worked example — 2026
//!
//! ```text
//! easter_sunday(2026) = 2026-04-05  (Computus)
//! good_friday         = 2026-04-03
//! easter_monday       = 2026-04-06
//!
//! holidays(2026) = { 2026-01-01, 2026-04-03, 2026-04-06,
//!                    2026-05-01, 2026-12-25, 2026-12-26 }
//! ```
//!
//! # References
//!
//! - European Central Bank, *Decision of the European Central Bank on
//!   the TARGET2 closing days* — the six fixed closing dates,
//!   unchanged since TARGET2 went live on 19 November 2007 and replaced
//!   TARGET on 19 May 2008.
//! - European Central Bank, *TARGET Annual Report* (any year) —
//!   reproduces the closing-day list.
//! - Jean Meeus, *Astronomical Algorithms*, 2nd ed., Willmann-Bell, 1998,
//!   §8 "The date of Easter" — the Computus used by [`Date::easter_sunday`].

use crate::date::Date;

/// True if `date` is a TARGET2 holiday (a Eurozone-wide bank holiday).
///
/// TARGET2 is the only calendar in this crate that is fully rule-based —
/// the six holidays are derived from the year alone (Easter via the
/// Computus algorithm in [`Date::easter_sunday`]), so no dated table is
/// stored and the calendar is exact for every year in the supported
/// `Date` range.
///
/// Weekends are NOT classified as holidays here — they are handled by the
/// caller's business-day predicate (see the `is_business_day` dispatcher
/// in [`crate::calendar`]).
///
/// # Examples
///
/// ```
/// use regit_daycount::Date;
/// use regit_daycount::calendar::target2;
///
/// // Christmas Day 2026 is a TARGET2 holiday.
/// assert!(target2::is_holiday(Date::ymd(2026, 12, 25).unwrap()));
///
/// // A regular Wednesday is not.
/// assert!(!target2::is_holiday(Date::ymd(2026, 5, 20).unwrap()));
/// ```
#[must_use]
pub fn is_holiday(date: Date) -> bool {
    let year = date.year();
    let easter = Date::easter_sunday(year);
    let good_friday = easter.add_days(-2);
    let easter_monday = easter.add_days(1);

    date == Date::ymd_unchecked(year, 1, 1)         // New Year's Day
        || date == good_friday                       // Easter − 2
        || date == easter_monday                     // Easter + 1
        || date == Date::ymd_unchecked(year, 5, 1)  // Labour Day
        || date == Date::ymd_unchecked(year, 12, 25) // Christmas Day
        || date == Date::ymd_unchecked(year, 12, 26) // Christmas Holiday
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── 2024 — every holiday in the year (Easter = 31 Mar) ──────────────

    #[test]
    fn holidays_2024() {
        // Easter Sunday 2024 = 2024-03-31, so Good Friday = 2024-03-29 and
        // Easter Monday = 2024-04-01.
        let dates = [
            (1, 1),   // New Year's Day (Mon)
            (3, 29),  // Good Friday
            (4, 1),   // Easter Monday
            (5, 1),   // Labour Day (Wed)
            (12, 25), // Christmas Day (Wed)
            (12, 26), // Christmas Holiday (Thu)
        ];
        for (m, d) in dates {
            assert!(
                is_holiday(Date::ymd(2024, m, d).unwrap()),
                "2024-{m:02}-{d:02} must be a TARGET2 holiday",
            );
        }
    }

    // ─── 2026 — every holiday in the year (Easter = 5 Apr) ───────────────

    #[test]
    fn holidays_2026() {
        // Easter Sunday 2026 = 2026-04-05, so Good Friday = 2026-04-03 and
        // Easter Monday = 2026-04-06.
        let dates = [
            (1, 1),   // New Year's Day
            (4, 3),   // Good Friday
            (4, 6),   // Easter Monday
            (5, 1),   // Labour Day
            (12, 25), // Christmas Day
            (12, 26), // Christmas Holiday
        ];
        for (m, d) in dates {
            assert!(
                is_holiday(Date::ymd(2026, m, d).unwrap()),
                "2026-{m:02}-{d:02} must be a TARGET2 holiday",
            );
        }
    }

    // ─── 2026 — explicit non-holidays adjacent to the rule dates ─────────

    #[test]
    fn non_holidays_2026() {
        // Each of these sits next to a real TARGET2 holiday and is a
        // common source of off-by-one regressions. Easter Sunday itself
        // is checked here: it is a Sunday, but the function reports the
        // rule date, not "the date is or is not closed". Easter Sunday
        // is not in the six-date TARGET2 rule, so `is_holiday` returns
        // `false` for it — the weekend closure is handled separately by
        // the caller's `is_business_day` wrapper.
        let dates = [
            (1, 2),   // day after New Year's Day
            (4, 4),   // Easter Saturday
            (4, 5),   // Easter Sunday (not a TARGET2 rule date)
            (5, 2),   // day after Labour Day
            (12, 24), // Christmas Eve
            (12, 27), // day after Christmas Holiday
        ];
        for (m, d) in dates {
            assert!(
                !is_holiday(Date::ymd(2026, m, d).unwrap()),
                "2026-{m:02}-{d:02} must NOT be a TARGET2 holiday",
            );
        }
    }

    // ─── Whit Monday regression ──────────────────────────────────────────

    #[test]
    fn whit_monday_is_not_a_holiday() {
        // Whit Monday is the day after Pentecost; Pentecost is the
        // "fiftieth day" — Easter Sunday + 49 days = Pentecost Sunday,
        // so Whit Monday = Easter Sunday + 50 days. In 2026 that is
        // 2026-04-05 + 50 = 2026-05-25 (Mon). Whit Monday was a TARGET
        // closing day pre-2002 but was removed when TARGET2 superseded
        // TARGET; this regression test pins that decision.
        let easter = Date::easter_sunday(2026);
        let whit_monday = easter.add_days(50);
        assert_eq!(whit_monday, Date::ymd(2026, 5, 25).unwrap());
        assert!(!is_holiday(whit_monday));

        // Sanity check across a few more years.
        for year in [2024, 2025, 2027, 2028] {
            let wm = Date::easter_sunday(year).add_days(50);
            assert!(
                !is_holiday(wm),
                "Whit Monday {year} ({wm:?}) must not be a TARGET2 holiday",
            );
        }
    }

    // ─── Holidays do not shift to the next business day ──────────────────

    #[test]
    fn independence_from_weekday() {
        // The function reports the rule date, not the observed date. A
        // TARGET2 holiday that falls on a weekend is still reported as a
        // holiday on its rule date — there is no observed-day shift in
        // the ECB schedule. 2027-01-01 falls on a Friday (a working day
        // in the surrounding week), and is reported as a holiday for its
        // rule reason; what matters is that the function is independent
        // of the weekday — it never returns `false` because the date
        // happens to be a Saturday or Sunday.
        assert!(is_holiday(Date::ymd(2027, 1, 1).unwrap()));

        // Christmas Day 2027 = Sat, Boxing Day 2027 = Sun. Both must
        // still be reported as TARGET2 holidays — the rule does not
        // suppress them on weekends.
        assert!(is_holiday(Date::ymd(2027, 12, 25).unwrap()));
        assert!(is_holiday(Date::ymd(2027, 12, 26).unwrap()));

        // And Labour Day 2027 falls on a Saturday; same expectation.
        assert!(is_holiday(Date::ymd(2027, 5, 1).unwrap()));
    }

    // ─── Easter-anchored dates against the published Computus table ──────

    #[test]
    fn easter_anchored_dates_match_computus() {
        // Published Easter Sundays for the Western (Gregorian) Computus.
        // For each year, Good Friday = Easter − 2 and Easter Monday =
        // Easter + 1 must both be reported as TARGET2 holidays.
        let cases = [
            (2024, 3, 31), // 2024-03-31
            (2025, 4, 20), // 2025-04-20
            (2026, 4, 5),  // 2026-04-05
        ];
        for (y, m, d) in cases {
            let easter = Date::ymd(y, m, d).unwrap();
            assert_eq!(
                Date::easter_sunday(y),
                easter,
                "Computus disagrees with the published Easter date for {y}",
            );
            assert!(
                is_holiday(easter.add_days(-2)),
                "Good Friday {y} must be a TARGET2 holiday",
            );
            assert!(
                is_holiday(easter.add_days(1)),
                "Easter Monday {y} must be a TARGET2 holiday",
            );
        }
    }
}
