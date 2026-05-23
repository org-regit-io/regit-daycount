// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! Catalogue of holiday calendars and their dispatcher.
//!
//! A calendar is the rule that decides, for a given date, whether the
//! market it names is open. Five operations depend on it: deciding whether
//! a date is itself a business day, adjusting a non-business date forward
//! or backward, counting business days between two dates, and (under
//! Bus/252) computing year fractions over them.
//!
//! Seven named calendars are catalogued here; each gets its own submodule
//! with either a fixed-rule generator (TARGET2 — Easter-derived, six
//! dates a year) or a dated snapshot of the exchange's or central bank's
//! published list (everything else, covering 2020–2040 inclusive):
//!
//! ```text
//! Target2         ECB fixed rule — Easter-derived; six dates a year
//! UnitedStates    NYSE / Federal Reserve published list
//! UnitedKingdom   Bank of England bank holiday schedule
//! Japan           Japan Exchange Group / National Holidays Act
//! Switzerland     SIX Swiss Exchange published schedule
//! HongKong        HKEX published schedule
//! Singapore       SGX published schedule
//! ```
//!
//! Two compound kinds — `Composite` (union of holidays) and
//! `JointBusiness` (intersection of business days) — let multiple named
//! calendars combine; both live in the `composite` submodule.
//!
//! # Dispatchers
//!
//! - `is_holiday` — true if `date` is a holiday under `cal`. Weekends are
//!   **not** counted as holidays here — they are handled separately by
//!   `is_weekend` / `is_business_day`.
//! - `is_weekend` — true if `date` is Saturday or Sunday.
//! - `is_business_day` — true if `date` is neither a weekend nor a
//!   holiday under `cal`.
//! - `adjust` — applies a [`Roll`](crate::Roll) to `date` under `cal` by
//!   forwarding to [`roll::apply`](crate::roll::apply) with `cal`'s
//!   business-day predicate.
//! - `business_days_between` — counts business days in `[start, end)`
//!   under `cal`. Walks one day at a time with a defensive cap.
//! - `bus_252_for_calendar` — convenience wrapper around
//!   [`crate::day_count::bus_252::fraction`] that closes over `cal`'s
//!   business-day predicate.
//!
//! All five dispatchers are `Copy`-by-value, allocation-free, and
//! `no_std`-clean.
//!
//! # References
//!
//! - ECB Decision on the TARGET2 closing days (the six fixed dates).
//! - Loi du 21 juillet 1928 sur le droit du travail (Luxembourg labour law)
//!   and Loi du 28 février 2019 fixant les jours fériés légaux (the law
//!   that added Europe Day to the LU schedule).
//! - NYSE / Federal Reserve Bank of New York published holiday schedules.
//! - Bank of England bank holiday schedule.
//! - Japan Exchange Group calendar and the Japanese National Holidays Act.
//! - SIX Swiss Exchange trading calendar.
//! - HKEX trading calendar.
//! - SGX trading calendar.

use crate::date::{Date, Weekday};
use crate::roll::{self, Roll};

// ─── Calendar ────────────────────────────────────────────────────────────────

/// A named holiday calendar.
///
/// The variants name the seven primary calendars this crate ships. The
/// compound kinds (a union of holidays and an intersection of business
/// days) live in [`composite`], so that a primary calendar always has the
/// `Copy`-into-an-enum-tag shape that the dispatcher matches on.
///
/// # Examples
///
/// ```
/// use regit_daycount::Calendar;
///
/// let cal = Calendar::Target2;
/// assert_eq!(cal, Calendar::Target2);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Calendar {
    /// TARGET2 — the euro-area RTGS calendar, defined by ECB fixed rule.
    Target2,
    /// Luxembourg — national bank holidays under the Loi du 21 juillet
    /// 1928 (and the Loi du 28 février 2019 that added Europe Day).
    /// Rule-based — every date is fixed or Easter-derived. The natural
    /// pairing with TARGET2 for Luxembourg-domiciled UCITS / SIF NAV
    /// schedules (compose them via [`JointBusiness`]).
    Luxembourg,
    /// United States — NYSE / Federal Reserve published holiday list.
    UnitedStates,
    /// United Kingdom — Bank of England bank holiday schedule.
    UnitedKingdom,
    /// Japan — Japan Exchange Group / National Holidays Act schedule.
    Japan,
    /// Switzerland — SIX Swiss Exchange trading calendar.
    Switzerland,
    /// Hong Kong — HKEX trading calendar.
    HongKong,
    /// Singapore — SGX trading calendar.
    Singapore,
}

// ─── Per-calendar submodules ─────────────────────────────────────────────────

pub mod composite;
pub mod hong_kong;
pub mod japan;
pub mod luxembourg;
pub mod singapore;
pub mod switzerland;
pub mod target2;
pub mod united_kingdom;
pub mod united_states;

pub use composite::{Composite, JointBusiness};

// ─── Dispatchers ─────────────────────────────────────────────────────────────

/// True if `date` falls on a Saturday or Sunday.
///
/// All seven calendars in this crate use the Western Sat/Sun weekend; the
/// dispatcher applies this uniformly. (Markets that follow a Fri/Sat
/// weekend — Saudi Arabia, the UAE pre-2022 — are not currently modelled.)
///
/// # Examples
///
/// ```
/// use regit_daycount::Date;
/// use regit_daycount::calendar::is_weekend;
///
/// assert!( is_weekend(Date::ymd(2026, 5, 23).unwrap())); // Saturday
/// assert!(!is_weekend(Date::ymd(2026, 5, 22).unwrap())); // Friday
/// ```
#[must_use]
#[inline]
pub fn is_weekend(date: Date) -> bool {
    matches!(date.day_of_week(), Weekday::Sat | Weekday::Sun)
}

/// True if `date` is a holiday under `cal`.
///
/// Returns true *only* for calendar holidays — weekends are NOT counted
/// here. Use [`is_business_day`] for the combined "neither a holiday nor a
/// weekend" check.
///
/// For the six snapshot-based calendars (everything except TARGET2), dates
/// outside the embedded `COVERAGE` range return `false`. The bound is
/// documented in each submodule's docstring.
///
/// # Examples
///
/// ```
/// use regit_daycount::{Calendar, Date};
/// use regit_daycount::calendar::is_holiday;
///
/// // 2026-12-25 is a TARGET2 holiday.
/// assert!( is_holiday(Date::ymd(2026, 12, 25).unwrap(), Calendar::Target2));
///
/// // 2026-05-23 (Saturday) is NOT a TARGET2 holiday — weekends are
/// // handled separately by `is_business_day`.
/// assert!(!is_holiday(Date::ymd(2026, 5, 23).unwrap(), Calendar::Target2));
/// ```
#[must_use]
pub fn is_holiday(date: Date, cal: Calendar) -> bool {
    match cal {
        Calendar::Target2 => target2::is_holiday(date),
        Calendar::Luxembourg => luxembourg::is_holiday(date),
        Calendar::UnitedStates => united_states::is_holiday(date),
        Calendar::UnitedKingdom => united_kingdom::is_holiday(date),
        Calendar::Japan => japan::is_holiday(date),
        Calendar::Switzerland => switzerland::is_holiday(date),
        Calendar::HongKong => hong_kong::is_holiday(date),
        Calendar::Singapore => singapore::is_holiday(date),
    }
}

/// True if `date` is a business day under `cal` — neither a weekend nor a
/// holiday.
///
/// # Examples
///
/// ```
/// use regit_daycount::{Calendar, Date};
/// use regit_daycount::calendar::is_business_day;
///
/// // 2026-05-22 is a Friday and not a TARGET2 holiday.
/// assert!( is_business_day(Date::ymd(2026, 5, 22).unwrap(), Calendar::Target2));
///
/// // 2026-05-23 is a Saturday → not a business day.
/// assert!(!is_business_day(Date::ymd(2026, 5, 23).unwrap(), Calendar::Target2));
///
/// // 2026-12-25 is Christmas — a TARGET2 holiday.
/// assert!(!is_business_day(Date::ymd(2026, 12, 25).unwrap(), Calendar::Target2));
/// ```
#[must_use]
pub fn is_business_day(date: Date, cal: Calendar) -> bool {
    !is_weekend(date) && !is_holiday(date, cal)
}

/// Adjusts `date` under `conv` using `cal`'s business-day predicate.
///
/// Thin wrapper around [`crate::roll::apply`] that supplies the closure
/// `|d| is_business_day(d, cal)`. See [`Roll`] for the seven roll
/// conventions.
///
/// # Examples
///
/// ```
/// use regit_daycount::{Calendar, Date, Roll};
/// use regit_daycount::calendar::adjust;
///
/// // Saturday 2026-05-23, rolled Following under TARGET2 → Mon 2026-05-25.
/// let saturday = Date::ymd(2026, 5, 23).unwrap();
/// let monday   = adjust(saturday, Roll::Following, Calendar::Target2);
/// assert_eq!(monday, Date::ymd(2026, 5, 25).unwrap());
/// ```
#[must_use]
pub fn adjust(date: Date, conv: Roll, cal: Calendar) -> Date {
    roll::apply(date, conv, |d| is_business_day(d, cal))
}

/// Maximum number of one-day steps [`business_days_between`] will take
/// before giving up. Bounds the walk at roughly 200 years (365 × 200 =
/// 73 000), which is far beyond any plausible date-count input; the cap
/// is purely defensive against a pathological caller.
const BD_MAX_STEPS: u32 = 365 * 200;

/// Counts business days in `[start, end)` under `cal`.
///
/// `start` is inclusive, `end` is exclusive — matching the crate's
/// universal half-open day-count convention. If `start >= end` the count
/// is `0` (inverted intervals are not negated here; callers that want a
/// signed count should swap and negate themselves).
///
/// The walk is capped at `BD_MAX_STEPS` one-day steps (~200 years). If
/// the cap is reached the count so far is returned, with no error — the
/// cap is purely defensive and the typical caller is nowhere near it.
///
/// # Examples
///
/// ```
/// use regit_daycount::{Calendar, Date};
/// use regit_daycount::calendar::business_days_between;
///
/// // The two weeks 2026-05-01 (Fri) → 2026-05-15 (Fri), half-open,
/// // under TARGET2. May 1 is a TARGET2 holiday (Labour Day, Fri); the
/// // business days in [May 1, May 15) are May 4, 5, 6, 7, 8, 11, 12,
/// // 13, 14 = 9 days.
/// let start = Date::ymd(2026, 5, 1).unwrap();
/// let end   = Date::ymd(2026, 5, 15).unwrap();
/// assert_eq!(business_days_between(start, end, Calendar::Target2), 9);
/// ```
#[must_use]
pub fn business_days_between(start: Date, end: Date, cal: Calendar) -> u32 {
    if start >= end {
        return 0;
    }
    let mut count = 0u32;
    let mut d = start;
    let mut steps = 0u32;
    while d < end {
        if steps >= BD_MAX_STEPS {
            return count;
        }
        if is_business_day(d, cal) {
            count += 1;
        }
        d = d.add_days(1);
        steps += 1;
    }
    count
}

/// Returns the next business day on or after `date` under `cal`.
///
/// Equivalent to `adjust(date, Roll::Following, cal)` — if `date` is
/// itself a business day, it is returned unchanged; otherwise the walk
/// advances one day at a time until a business day is found.
///
/// # Examples
///
/// ```
/// use regit_daycount::{Calendar, Date};
/// use regit_daycount::calendar::next_business_day;
///
/// // 2026-12-25 (Fri) is a TARGET2 holiday; the next business day is
/// // Mon 2026-12-28.
/// let xmas = Date::ymd(2026, 12, 25).unwrap();
/// assert_eq!(
///     next_business_day(xmas, Calendar::Target2),
///     Date::ymd(2026, 12, 28).unwrap(),
/// );
/// ```
#[must_use]
#[inline]
pub fn next_business_day(date: Date, cal: Calendar) -> Date {
    adjust(date, Roll::Following, cal)
}

/// Returns the previous business day on or before `date` under `cal`.
///
/// Equivalent to `adjust(date, Roll::Preceding, cal)`.
///
/// # Examples
///
/// ```
/// use regit_daycount::{Calendar, Date};
/// use regit_daycount::calendar::previous_business_day;
///
/// // 2026-12-26 (Sat) is the day after Christmas — both Sat and the
/// // Dec 26 TARGET2 holiday are non-business; previous business day is
/// // Wed 2026-12-24.
/// let dec26 = Date::ymd(2026, 12, 26).unwrap();
/// assert_eq!(
///     previous_business_day(dec26, Calendar::Target2),
///     Date::ymd(2026, 12, 24).unwrap(),
/// );
/// ```
#[must_use]
#[inline]
pub fn previous_business_day(date: Date, cal: Calendar) -> Date {
    adjust(date, Roll::Preceding, cal)
}

/// Maximum number of one-day steps [`add_business_days`] will take in
/// either direction before giving up. The cap is purely defensive
/// against a pathological `n` (no real settlement convention exceeds
/// T+10 in any market).
const ABD_MAX_STEPS: u32 = 365 * 200;

/// Returns the business day `n` business days from `date` under `cal`.
///
/// `n` is signed: positive walks forward, negative walks backward,
/// zero rolls `date` to the next/previous business day if it is itself
/// not a business day (specifically: zero returns `date` unchanged if
/// it is a business day, otherwise it walks forward — matching the
/// market convention for T+0 settlement on a non-business start date).
///
/// The walk is one calendar day at a time, counting only business days
/// against `n`; weekends and holidays are skipped without consuming a
/// count. The walk is capped at ~200 years of calendar walking as a
/// defensive bound — no real `n` ever approaches this.
///
/// # Examples
///
/// ```
/// use regit_daycount::{Calendar, Date};
/// use regit_daycount::calendar::add_business_days;
///
/// // T+2 from Mon 2026-05-25 under TARGET2 → Wed 2026-05-27.
/// let mon = Date::ymd(2026, 5, 25).unwrap();
/// assert_eq!(
///     add_business_days(mon, 2, Calendar::Target2),
///     Date::ymd(2026, 5, 27).unwrap(),
/// );
///
/// // T+2 across a weekend: Thu 2026-05-28 + 2 = Mon 2026-06-01.
/// let thu = Date::ymd(2026, 5, 28).unwrap();
/// assert_eq!(
///     add_business_days(thu, 2, Calendar::Target2),
///     Date::ymd(2026, 6, 1).unwrap(),
/// );
///
/// // Negative n walks backward.
/// assert_eq!(
///     add_business_days(thu, -2, Calendar::Target2),
///     Date::ymd(2026, 5, 26).unwrap(),
/// );
/// ```
#[must_use]
pub fn add_business_days(date: Date, n: i32, cal: Calendar) -> Date {
    if n == 0 {
        // T+0 on a non-business day rolls forward to the next business
        // day; on a business day it is the identity.
        return next_business_day(date, cal);
    }
    let step: i32 = if n > 0 { 1 } else { -1 };
    let mut remaining = n.unsigned_abs();
    let mut d = date;
    let mut steps = 0u32;
    while remaining > 0 {
        if steps >= ABD_MAX_STEPS {
            return d;
        }
        d = d.add_days(step);
        steps += 1;
        if is_business_day(d, cal) {
            remaining -= 1;
        }
    }
    d
}

/// Computes the Bus/252 year fraction over `[start, end)` under `cal`.
///
/// Convenience wrapper around [`crate::day_count::bus_252::fraction`] that
/// supplies `cal`'s business-day predicate, so callers do not have to
/// build the closure themselves. The signed-interval semantics match
/// `bus_252::fraction` (inverted intervals negate).
///
/// # Examples
///
/// ```
/// use regit_daycount::{Calendar, Date};
/// use regit_daycount::calendar::bus_252_for_calendar;
///
/// let start = Date::ymd(2026, 5, 1).unwrap();
/// let end   = Date::ymd(2026, 5, 15).unwrap();
/// // 9 business days in [May 1, May 15) under TARGET2 (May 1 is Labour
/// // Day); fraction = 9 / 252.
/// let yf = bus_252_for_calendar(start, end, Calendar::Target2);
/// assert!((yf - 9.0 / 252.0).abs() < 1e-12);
/// ```
#[must_use]
pub fn bus_252_for_calendar(start: Date, end: Date, cal: Calendar) -> f64 {
    crate::day_count::bus_252::fraction(start, end, |d| is_business_day(d, cal))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── is_weekend ──────────────────────────────────────────────────────

    #[test]
    fn weekend_classification() {
        assert!(is_weekend(Date::ymd(2026, 5, 23).unwrap())); // Sat
        assert!(is_weekend(Date::ymd(2026, 5, 24).unwrap())); // Sun
        assert!(!is_weekend(Date::ymd(2026, 5, 22).unwrap())); // Fri
        assert!(!is_weekend(Date::ymd(2026, 5, 25).unwrap())); // Mon
    }

    // ─── is_holiday dispatch ────────────────────────────────────────────

    #[test]
    fn dispatches_to_every_named_calendar() {
        // One known holiday per calendar — a smoke test that the dispatch
        // arms wire to the right submodule.
        assert!(is_holiday(
            Date::ymd(2026, 12, 25).unwrap(),
            Calendar::Target2
        ));
        assert!(is_holiday(
            Date::ymd(2024, 7, 4).unwrap(),
            Calendar::UnitedStates,
        ));
        assert!(is_holiday(
            Date::ymd(2024, 12, 26).unwrap(),
            Calendar::UnitedKingdom,
        ));
        assert!(is_holiday(Date::ymd(2024, 1, 1).unwrap(), Calendar::Japan));
        assert!(is_holiday(
            Date::ymd(2024, 8, 1).unwrap(),
            Calendar::Switzerland,
        ));
        assert!(is_holiday(
            Date::ymd(2024, 12, 25).unwrap(),
            Calendar::HongKong,
        ));
        assert!(is_holiday(
            Date::ymd(2024, 8, 9).unwrap(),
            Calendar::Singapore,
        ));
    }

    // ─── is_business_day ─────────────────────────────────────────────────

    #[test]
    fn business_day_excludes_weekends_and_holidays() {
        // Friday, not a holiday → business day.
        assert!(is_business_day(
            Date::ymd(2026, 5, 22).unwrap(),
            Calendar::Target2,
        ));
        // Saturday → not a business day.
        assert!(!is_business_day(
            Date::ymd(2026, 5, 23).unwrap(),
            Calendar::Target2,
        ));
        // Christmas Day, weekday → not a business day.
        assert!(!is_business_day(
            Date::ymd(2026, 12, 25).unwrap(),
            Calendar::Target2,
        ));
    }

    // ─── adjust ──────────────────────────────────────────────────────────

    #[test]
    fn adjust_forwards_a_holiday_friday_under_target2() {
        // Fri 2026-05-01 is Labour Day → Following lands on Mon 2026-05-04.
        let labour = Date::ymd(2026, 5, 1).unwrap();
        let mon = Date::ymd(2026, 5, 4).unwrap();
        assert_eq!(adjust(labour, Roll::Following, Calendar::Target2), mon);
    }

    #[test]
    fn adjust_modified_following_falls_back_at_month_end() {
        // Find a holiday that is the last business day of its month under
        // a calendar — 2026-05-29 (Fri) is the last business day of May;
        // 2026-05-31 (Sun) ModifiedFollowing under TARGET2 must go back to
        // Fri 2026-05-29 (forward would land on Mon 2026-06-01 = new month).
        let sun = Date::ymd(2026, 5, 31).unwrap();
        let fri = Date::ymd(2026, 5, 29).unwrap();
        assert_eq!(adjust(sun, Roll::ModifiedFollowing, Calendar::Target2), fri,);
    }

    // ─── business_days_between ───────────────────────────────────────────

    #[test]
    fn business_days_between_two_weeks_with_target2_labour_day() {
        // [2026-05-01, 2026-05-15) under TARGET2 — May 1 is Labour Day, so
        // 9 business days (May 4, 5, 6, 7, 8, 11, 12, 13, 14).
        let s = Date::ymd(2026, 5, 1).unwrap();
        let e = Date::ymd(2026, 5, 15).unwrap();
        assert_eq!(business_days_between(s, e, Calendar::Target2), 9);
    }

    #[test]
    fn business_days_between_zero_for_empty_or_inverted() {
        let d = Date::ymd(2026, 5, 1).unwrap();
        assert_eq!(business_days_between(d, d, Calendar::Target2), 0);
        // Inverted — returns 0, not a negation.
        let later = Date::ymd(2026, 6, 1).unwrap();
        assert_eq!(business_days_between(later, d, Calendar::Target2), 0);
    }

    // ─── bus_252_for_calendar ────────────────────────────────────────────

    #[test]
    fn bus_252_for_calendar_matches_manual_count() {
        // [2026-05-01, 2026-05-15) under TARGET2 — 9 business days /
        // 252.
        let s = Date::ymd(2026, 5, 1).unwrap();
        let e = Date::ymd(2026, 5, 15).unwrap();
        let yf = bus_252_for_calendar(s, e, Calendar::Target2);
        assert!((yf - 9.0 / 252.0).abs() < 1e-12);
    }
}
