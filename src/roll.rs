// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! Date-roll conventions — the rules that adjust a date that falls on a
//! non-business day to a neighbouring business day.
//!
//! Every coupon date, fixing date, and reset date in a financial contract is
//! stated unadjusted on the term sheet. When that date falls on a weekend
//! or a holiday under the relevant calendar, the contract specifies a *roll*
//! that moves it to a nearby business day. This module is the catalogue of
//! those rolls.
//!
//! ```text
//! Unadjusted          return the date as given (no adjustment)
//! Following           next business day
//! ModifiedFollowing   Following, unless it crosses a month boundary —
//!                     then Preceding
//! Preceding           previous business day
//! ModifiedPreceding   Preceding, unless it crosses a month boundary —
//!                     then Following
//! Nearest             the nearer business day; ties broken forward
//! EndOfMonth          standalone-date operation: ModifiedFollowing;
//!                     schedule-origin EOM propagation needs separate context
//! ```
//!
//! The free function [`try_apply`] applies a roll under a caller-supplied
//! business-day predicate. It returns an explicit error if determining the
//! answer would leave the supported date domain or exceed its bounded walk.
//! For named calendars with coverage checks, use `calendar::try_adjust` when
//! the `calendars` feature is enabled.
//!
//! # References
//!
//! - ISDA 2006 Definitions §4.12 (Business Day Convention).

use crate::date::Date;
use core::fmt;

// ─── Roll ────────────────────────────────────────────────────────────────────

/// A date-roll convention — how a non-business date is adjusted to a
/// business day.
///
/// The variants follow ISDA 2006 §4.12 plus the two operational extensions
/// the market uses everywhere (`Nearest`, `EndOfMonth`).
///
/// # Examples
///
/// ```
/// use regit_daycount::Roll;
///
/// let r = Roll::ModifiedFollowing;
/// assert_eq!(r, Roll::ModifiedFollowing);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Roll {
    /// Return the date as given, even if it is a weekend or holiday.
    Unadjusted,
    /// Roll forward to the next business day.
    Following,
    /// Roll forward to the next business day, unless that crosses a month
    /// boundary — in which case roll back to the previous business day.
    ModifiedFollowing,
    /// Roll back to the previous business day.
    Preceding,
    /// Roll back to the previous business day, unless that crosses a month
    /// boundary — in which case roll forward to the next business day.
    ModifiedPreceding,
    /// Roll to the nearer business day; ties are broken forward.
    Nearest,
    /// For a standalone date, preserve a business date and otherwise apply
    /// Modified Following. This API has no schedule origin and does not
    /// propagate an end-of-month anchor across a generated coupon schedule.
    EndOfMonth,
}

/// A date roll could not determine a business date within its supported domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RollError {
    /// Determining the result requires a date outside `1583..=9999`.
    DateBoundary {
        /// The last supported date reached in the required direction.
        date: Date,
    },
    /// No business date was found within the bounded search.
    WalkLimitExceeded {
        /// Maximum one-day distance searched in either direction.
        limit: u32,
    },
}

impl fmt::Display for RollError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DateBoundary { date } => write!(f, "date roll reached date boundary at {date:?}"),
            Self::WalkLimitExceeded { limit } => write!(f, "date roll exceeded {limit} steps"),
        }
    }
}

impl core::error::Error for RollError {}

/// Maximum one-day distance searched in either direction. A failed search
/// returns an error, never an unchanged non-business date or partial answer.
const MAX_STEPS: u32 = 366;

/// Applies a date-roll convention under a caller-supplied business-day predicate.
///
/// The predicate must consistently report whether each date is a business day.
/// The input date is returned unchanged if it is already a business day, or
/// if `conv` is [`Roll::Unadjusted`].
///
/// Modified Following searches forward inside the input month, then searches
/// backward from the input if no business date remains in that month. Modified
/// Preceding mirrors this rule. This avoids requiring an out-of-domain date
/// merely to establish that the month would change at the date boundaries.
/// [`Roll::EndOfMonth`] uses Modified Following for this standalone-date API;
/// a schedule-origin anchor requires separate schedule context.
///
/// [`Roll::Nearest`] searches in increasing day distance; ties go forward.
///
/// # Errors
///
/// Returns [`RollError::DateBoundary`] if a required search leaves the supported
/// date range, or [`RollError::WalkLimitExceeded`] if no business date is found
/// within 366 calendar days in the required direction. No partial result is
/// returned. `Unadjusted` does not call the predicate.
///
/// # Examples
///
/// ```
/// use regit_daycount::{Date, Roll, Weekday};
/// use regit_daycount::roll::try_apply;
///
/// let is_biz = |d: Date| !matches!(d.day_of_week(), Weekday::Sat | Weekday::Sun);
/// let sat = Date::ymd(2026, 5, 23).unwrap();
/// assert_eq!(
///     try_apply(sat, Roll::Following, is_biz).unwrap(),
///     Date::ymd(2026, 5, 25).unwrap(),
/// );
/// ```
pub fn try_apply(
    date: Date,
    conv: Roll,
    is_business_day: impl Fn(Date) -> bool,
) -> Result<Date, RollError> {
    match conv {
        Roll::Unadjusted => Ok(date),
        Roll::Following => walk(date, 1, &is_business_day),
        Roll::Preceding => walk(date, -1, &is_business_day),
        Roll::ModifiedFollowing | Roll::EndOfMonth => {
            match walk_inside_month(date, 1, &is_business_day)? {
                Some(result) => Ok(result),
                None => walk(date, -1, &is_business_day),
            }
        }
        Roll::ModifiedPreceding => match walk_inside_month(date, -1, &is_business_day)? {
            Some(result) => Ok(result),
            None => walk(date, 1, &is_business_day),
        },
        Roll::Nearest => nearest(date, &is_business_day),
    }
}

fn step(date: Date, direction: i32) -> Result<Date, RollError> {
    date.try_add_days(direction)
        .map_err(|_| RollError::DateBoundary { date })
}

fn walk(date: Date, direction: i32, is_biz: &impl Fn(Date) -> bool) -> Result<Date, RollError> {
    let mut current = date;
    for distance in 0..=MAX_STEPS {
        if is_biz(current) {
            return Ok(current);
        }
        if distance < MAX_STEPS {
            current = step(current, direction)?;
        }
    }
    Err(RollError::WalkLimitExceeded { limit: MAX_STEPS })
}

/// `None` means the required direction would leave the input month.
fn walk_inside_month(
    date: Date,
    direction: i32,
    is_biz: &impl Fn(Date) -> bool,
) -> Result<Option<Date>, RollError> {
    let mut current = date;
    loop {
        if is_biz(current) {
            return Ok(Some(current));
        }
        if (direction > 0 && current.day() == Date::days_in_month(date.year(), date.month()))
            || (direction < 0 && current.day() == 1)
        {
            return Ok(None);
        }
        current = step(current, direction)?;
    }
}

fn nearest(date: Date, is_biz: &impl Fn(Date) -> bool) -> Result<Date, RollError> {
    if is_biz(date) {
        return Ok(date);
    }
    let mut forward = date;
    let mut backward = date;
    for _ in 1..=MAX_STEPS {
        forward = step(forward, 1)?;
        if is_biz(forward) {
            return Ok(forward);
        }
        backward = step(backward, -1)?;
        if is_biz(backward) {
            return Ok(backward);
        }
    }
    Err(RollError::WalkLimitExceeded { limit: MAX_STEPS })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::date::Weekday;

    /// Weekend-only predicate: Saturday and Sunday are not business days.
    fn weekends_only(d: Date) -> bool {
        let wd = d.day_of_week();
        wd != Weekday::Sat && wd != Weekday::Sun
    }

    /// Weekend predicate plus a single holiday on 2026-12-25 (Friday).
    fn weekends_plus_christmas(d: Date) -> bool {
        if d == Date::ymd_unchecked(2026, 12, 25) {
            return false;
        }
        weekends_only(d)
    }

    // ─── Unadjusted ──────────────────────────────────────────────────────

    #[test]
    fn unadjusted_returns_input() {
        let sat = Date::ymd(2026, 5, 23).unwrap();
        assert_eq!(try_apply(sat, Roll::Unadjusted, weekends_only), Ok(sat));
    }

    // ─── Following / Preceding ───────────────────────────────────────────

    #[test]
    fn following_from_saturday() {
        // 2026-05-23 is a Saturday → next business day is Mon 2026-05-25.
        let sat = Date::ymd(2026, 5, 23).unwrap();
        let mon = Date::ymd(2026, 5, 25).unwrap();
        assert_eq!(try_apply(sat, Roll::Following, weekends_only), Ok(mon));
    }

    #[test]
    fn preceding_from_saturday() {
        // 2026-05-23 is a Saturday → previous business day is Fri 2026-05-22.
        let sat = Date::ymd(2026, 5, 23).unwrap();
        let fri = Date::ymd(2026, 5, 22).unwrap();
        assert_eq!(try_apply(sat, Roll::Preceding, weekends_only), Ok(fri));
    }

    #[test]
    fn following_on_business_day_is_identity() {
        let mon = Date::ymd(2026, 5, 25).unwrap();
        assert_eq!(try_apply(mon, Roll::Following, weekends_only), Ok(mon));
        assert_eq!(try_apply(mon, Roll::Preceding, weekends_only), Ok(mon));
    }

    // ─── ModifiedFollowing / ModifiedPreceding ───────────────────────────

    #[test]
    fn modified_following_falls_back_when_month_changes() {
        // Sunday 2026-05-31 → Following would land on Mon 2026-06-01 (new
        // month). Fall back to Preceding from 2026-05-31 → Fri 2026-05-29.
        let sun = Date::ymd(2026, 5, 31).unwrap();
        let fri = Date::ymd(2026, 5, 29).unwrap();
        assert_eq!(
            try_apply(sun, Roll::ModifiedFollowing, weekends_only),
            Ok(fri)
        );
    }

    #[test]
    fn modified_following_within_month_just_follows() {
        // Saturday 2026-05-23 → Following = Mon 2026-05-25, same month. No
        // fall-back.
        let sat = Date::ymd(2026, 5, 23).unwrap();
        let mon = Date::ymd(2026, 5, 25).unwrap();
        assert_eq!(
            try_apply(sat, Roll::ModifiedFollowing, weekends_only),
            Ok(mon)
        );
    }

    #[test]
    fn modified_preceding_within_month_just_precedes() {
        // Saturday 2026-01-31 → Preceding = Fri 2026-01-30, same month. No
        // fall-back.
        let sat = Date::ymd(2026, 1, 31).unwrap();
        let fri = Date::ymd(2026, 1, 30).unwrap();
        assert_eq!(
            try_apply(sat, Roll::ModifiedPreceding, weekends_only),
            Ok(fri)
        );
    }

    #[test]
    fn modified_preceding_falls_back_when_month_changes() {
        // Sunday 2026-02-01 → Preceding would land on Fri 2026-01-30 (prior
        // month). Fall back to Following from 2026-02-01 → Mon 2026-02-02.
        let sun = Date::ymd(2026, 2, 1).unwrap();
        let mon = Date::ymd(2026, 2, 2).unwrap();
        assert_eq!(
            try_apply(sun, Roll::ModifiedPreceding, weekends_only),
            Ok(mon)
        );
    }

    // ─── Nearest ─────────────────────────────────────────────────────────

    #[test]
    fn nearest_business_day_is_identity() {
        let mon = Date::ymd(2026, 5, 25).unwrap();
        assert_eq!(try_apply(mon, Roll::Nearest, weekends_only), Ok(mon));
    }

    #[test]
    fn nearest_on_saturday_goes_friday() {
        // Saturday: Friday is 1 day back, Monday is 2 days forward → Friday.
        let sat = Date::ymd(2026, 5, 23).unwrap();
        let fri = Date::ymd(2026, 5, 22).unwrap();
        assert_eq!(try_apply(sat, Roll::Nearest, weekends_only), Ok(fri));
    }

    #[test]
    fn nearest_on_sunday_goes_monday() {
        // Sunday: Friday is 2 days back, Monday is 1 day forward → Monday.
        let sun = Date::ymd(2026, 5, 24).unwrap();
        let mon = Date::ymd(2026, 5, 25).unwrap();
        assert_eq!(try_apply(sun, Roll::Nearest, weekends_only), Ok(mon));
    }

    #[test]
    fn nearest_ties_break_forward() {
        // Christmas 2026-12-25 is a Friday; treat it as a holiday and put
        // the entire weekend off too. Then Thu 2026-12-24 is 1 day back and
        // Mon 2026-12-28 is 3 days forward → backward (Thu).
        // To exercise the tie path explicitly, use a 1-back / 1-forward
        // case: a single Wednesday holiday surrounded by business days.
        let is_biz = |d: Date| {
            if d == Date::ymd_unchecked(2026, 5, 13) {
                return false;
            }
            weekends_only(d)
        };
        // 2026-05-13 is a Wednesday; previous biz Tue 2026-05-12 (1 back),
        // next biz Thu 2026-05-14 (1 forward). Tie → forward (Thursday).
        let wed = Date::ymd(2026, 5, 13).unwrap();
        let thu = Date::ymd(2026, 5, 14).unwrap();
        assert_eq!(try_apply(wed, Roll::Nearest, is_biz), Ok(thu));
    }

    // ─── EndOfMonth ──────────────────────────────────────────────────────

    #[test]
    fn end_of_month_anchors_when_last_business_day() {
        // 2026-02-28 is a Saturday; under weekends-only, the last business
        // day of February 2026 is Fri 2026-02-27. Feeding 2026-02-27 to
        // EndOfMonth keeps it at the last business day of its month (itself).
        let fri = Date::ymd(2026, 2, 27).unwrap();
        assert_eq!(try_apply(fri, Roll::EndOfMonth, weekends_only), Ok(fri));
    }

    #[test]
    fn end_of_month_falls_back_to_modified_following() {
        // 2026-05-15 (Fri) is *not* the last business day of May; under
        // EndOfMonth, fall through to ModifiedFollowing — and since it's a
        // business day, the result is the date itself.
        let mid = Date::ymd(2026, 5, 15).unwrap();
        assert_eq!(try_apply(mid, Roll::EndOfMonth, weekends_only), Ok(mid));
    }

    #[test]
    fn end_of_month_holiday_aware() {
        // With Fri 2026-12-25 as a holiday, the last business day of
        // December 2026 is Thu 2026-12-31 (Thursday). Feeding that in to
        // EndOfMonth must keep it.
        let thu = Date::ymd(2026, 12, 31).unwrap();
        assert_eq!(
            try_apply(thu, Roll::EndOfMonth, weekends_plus_christmas),
            Ok(thu)
        );
    }

    // ─── holiday-aware adjustments ───────────────────────────────────────

    #[test]
    fn following_skips_a_holiday() {
        // 2026-12-25 (Fri) is a holiday; Following from that date lands on
        // Mon 2026-12-28 (skipping Sat/Sun and the holiday).
        let xmas = Date::ymd(2026, 12, 25).unwrap();
        let mon = Date::ymd(2026, 12, 28).unwrap();
        assert_eq!(
            try_apply(xmas, Roll::Following, weekends_plus_christmas),
            Ok(mon)
        );
    }

    #[test]
    fn preceding_skips_a_holiday() {
        // Preceding from Fri 2026-12-25 (holiday) → Thu 2026-12-24.
        let xmas = Date::ymd(2026, 12, 25).unwrap();
        let thu = Date::ymd(2026, 12, 24).unwrap();
        assert_eq!(
            try_apply(xmas, Roll::Preceding, weekends_plus_christmas),
            Ok(thu)
        );
    }

    // ─── defensive cap ───────────────────────────────────────────────────

    #[test]
    fn pathological_predicate_returns_an_error() {
        // A predicate that never says "business day" must not loop forever.
        let always_false = |_d: Date| false;
        let any = Date::ymd(2026, 5, 23).unwrap();
        for convention in [
            Roll::Following,
            Roll::Preceding,
            Roll::ModifiedFollowing,
            Roll::ModifiedPreceding,
            Roll::Nearest,
            Roll::EndOfMonth,
        ] {
            assert_eq!(
                try_apply(any, convention, always_false),
                Err(RollError::WalkLimitExceeded { limit: MAX_STEPS })
            );
        }
    }

    #[test]
    fn directed_rolls_report_date_boundaries() {
        let first = Date::ymd(Date::MIN_YEAR, 1, 1).unwrap();
        let last = Date::ymd(Date::MAX_YEAR, 12, 31).unwrap();
        assert_eq!(
            try_apply(first, Roll::Preceding, |_| false),
            Err(RollError::DateBoundary { date: first })
        );
        assert_eq!(
            try_apply(last, Roll::Following, |_| false),
            Err(RollError::DateBoundary { date: last })
        );
        assert_eq!(
            try_apply(last, Roll::Nearest, |date| date < last),
            Err(RollError::DateBoundary { date: last })
        );
    }

    #[test]
    fn modified_rolls_can_fall_back_at_the_domain_boundary() {
        let first = Date::ymd(Date::MIN_YEAR, 1, 1).unwrap();
        let last = Date::ymd(Date::MAX_YEAR, 12, 31).unwrap();
        assert_eq!(
            try_apply(first, Roll::ModifiedPreceding, |date| date > first),
            Ok(first.try_add_days(1).unwrap())
        );
        for convention in [Roll::ModifiedFollowing, Roll::EndOfMonth] {
            assert_eq!(
                try_apply(last, convention, |date| date < last),
                Ok(last.try_add_days(-1).unwrap())
            );
        }
        assert_eq!(try_apply(last, Roll::EndOfMonth, |_| true), Ok(last));
    }

    #[test]
    fn search_limit_is_inclusive_and_never_returns_a_partial_result() {
        let date = Date::ymd(2026, 1, 1).unwrap();
        let at_limit = date.try_add_days(366).unwrap();
        let after_limit = date.try_add_days(367).unwrap();
        assert_eq!(
            try_apply(date, Roll::Following, |candidate| candidate == at_limit),
            Ok(at_limit)
        );
        assert_eq!(
            try_apply(date, Roll::Following, |candidate| candidate == after_limit),
            Err(RollError::WalkLimitExceeded { limit: MAX_STEPS })
        );
    }

    #[test]
    fn unadjusted_does_not_consult_the_calendar() {
        let last = Date::ymd(Date::MAX_YEAR, 12, 31).unwrap();
        assert_eq!(
            try_apply(last, Roll::Unadjusted, |_| {
                panic!("Unadjusted must not consult a calendar")
            }),
            Ok(last)
        );
    }
}
