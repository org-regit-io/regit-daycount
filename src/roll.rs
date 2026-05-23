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
//! EndOfMonth          anchor to the last business day of the month when
//!                     the input is the last business day of its month
//! ```
//!
//! The free function [`apply`] applies a roll. To avoid the cycle between
//! `roll` and `calendar` (`roll::apply` needs to know what a business day
//! is, and `calendar::adjust` is the thin wrapper that closes over a
//! [`crate::Calendar`]), `apply` takes a caller-supplied `is_business_day`
//! predicate rather than a `Calendar`. The `calendar` module passes
//! `|d| !is_holiday(d, cal) && !is_weekend(d)` in.
//!
//! # References
//!
//! - ISDA 2006 Definitions §4.12 (Business Day Convention).

use crate::date::Date;

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
    /// Anchor every rolled date to the last business day of the month when
    /// the input is itself the last business day of its month.
    EndOfMonth,
}

// ─── apply ───────────────────────────────────────────────────────────────────

/// Maximum number of one-day steps `apply` will take in any direction
/// before giving up. A real calendar has at most a handful of consecutive
/// non-business days, so 366 is a generous defensive bound. If the cap is
/// reached, `apply` returns the original `date` unchanged — silently, since
/// the function has no `Result` return — which is the safest fallback for a
/// pathological predicate.
const MAX_STEPS: u32 = 366;

/// Applies a date-roll convention to `date` under a caller-supplied
/// business-day predicate.
///
/// `is_business_day` is the bridge to the caller's holiday calendar: it
/// returns `true` exactly for dates the market is open. The convention is
/// then a deterministic walk over the predicate.
///
/// The walk is capped at `MAX_STEPS` one-day steps in either direction —
/// a defensive bound to keep a pathological predicate from looping
/// indefinitely. The cap is never reached for any real holiday calendar.
/// If the cap is reached, `apply` returns the original `date` unchanged.
///
/// # Convention details
///
/// - **`Unadjusted`** — return `date`.
/// - **`Following`** — walk forward day-by-day until the predicate holds.
/// - **`ModifiedFollowing`** — `Following`; if the result is in a different
///   month from `date`, fall back to `Preceding` from `date`.
/// - **`Preceding`** — walk backward day-by-day until the predicate holds.
/// - **`ModifiedPreceding`** — `Preceding`; if the result is in a different
///   month from `date`, fall back to `Following` from `date`.
/// - **`Nearest`** — if `date` is a business day, return it. Otherwise
///   compute the day distances to the next and previous business days and
///   return the closer one; ties go to `Following`.
/// - **`EndOfMonth`** — ISDA 2006 §4.12. If `date` is the **last business
///   day of its month** under the predicate, then any rolled coupon date
///   anchors to *the last business day of the month it lands in*. Concretely
///   `apply` returns the last business day of `date`'s own month; the
///   caller is expected to combine this with month-arithmetic (see
///   [`Date::add_months_eom_aware`]) to roll forward by a coupon period.
///   When `date` is not the last business day of its month, `EndOfMonth`
///   falls back to `ModifiedFollowing`.
///
/// # Examples
///
/// ```
/// use regit_daycount::{Date, Roll, Weekday};
/// use regit_daycount::roll::apply;
///
/// // A weekend-only calendar: Sat and Sun are not business days.
/// let is_biz = |d: Date| {
///     let wd = d.day_of_week();
///     wd != Weekday::Sat && wd != Weekday::Sun
/// };
///
/// // Saturday 2026-05-23 rolled Following lands on Monday 2026-05-25.
/// let sat = Date::ymd(2026, 5, 23).unwrap();
/// assert_eq!(apply(sat, Roll::Following, is_biz), Date::ymd(2026, 5, 25).unwrap());
/// ```
#[must_use]
pub fn apply(date: Date, conv: Roll, is_business_day: impl Fn(Date) -> bool) -> Date {
    match conv {
        Roll::Unadjusted => date,
        Roll::Following => walk_forward(date, &is_business_day),
        Roll::Preceding => walk_backward(date, &is_business_day),
        Roll::ModifiedFollowing => modified_following(date, &is_business_day),
        Roll::ModifiedPreceding => {
            let p = walk_backward(date, &is_business_day);
            if p.month() == date.month() {
                p
            } else {
                walk_forward(date, &is_business_day)
            }
        }
        Roll::Nearest => {
            if is_business_day(date) {
                return date;
            }
            // Find both neighbours and compare distances. Tie → forward.
            let forward = walk_forward(date, &is_business_day);
            let backward = walk_backward(date, &is_business_day);
            let fwd_dist = date.days_between(forward).unsigned_abs();
            let bwd_dist = date.days_between(backward).unsigned_abs();
            if fwd_dist <= bwd_dist {
                forward
            } else {
                backward
            }
        }
        Roll::EndOfMonth => {
            if is_last_business_day_of_month(date, &is_business_day) {
                last_business_day_of_month(date, &is_business_day)
            } else {
                // Fall through to ModifiedFollowing.
                modified_following(date, &is_business_day)
            }
        }
    }
}

/// `ModifiedFollowing` body — extracted so `EndOfMonth` can reuse it.
fn modified_following(date: Date, is_biz: &impl Fn(Date) -> bool) -> Date {
    let f = walk_forward(date, is_biz);
    if f.month() == date.month() {
        f
    } else {
        walk_backward(date, is_biz)
    }
}

// ─── Walkers ─────────────────────────────────────────────────────────────────

/// Advances `date` by one day at a time until `is_biz` holds, up to
/// [`MAX_STEPS`] steps. Returns `date` unchanged if the cap is reached.
fn walk_forward(date: Date, is_biz: &impl Fn(Date) -> bool) -> Date {
    let mut d = date;
    let mut steps = 0u32;
    while !is_biz(d) {
        if steps >= MAX_STEPS {
            return date;
        }
        d = d.add_days(1);
        steps += 1;
    }
    d
}

/// Decrements `date` by one day at a time until `is_biz` holds, up to
/// [`MAX_STEPS`] steps. Returns `date` unchanged if the cap is reached.
fn walk_backward(date: Date, is_biz: &impl Fn(Date) -> bool) -> Date {
    let mut d = date;
    let mut steps = 0u32;
    while !is_biz(d) {
        if steps >= MAX_STEPS {
            return date;
        }
        d = d.add_days(-1);
        steps += 1;
    }
    d
}

/// Returns the last business day of `date`'s month under `is_biz`. If no
/// business day exists in the month (degenerate predicate), returns the
/// last calendar day of the month.
fn last_business_day_of_month(date: Date, is_biz: &impl Fn(Date) -> bool) -> Date {
    let last = Date::ymd_unchecked(
        date.year(),
        date.month(),
        Date::days_in_month(date.year(), date.month()),
    );
    let mut d = last;
    let mut steps = 0u32;
    while !is_biz(d) {
        if steps >= MAX_STEPS {
            return last;
        }
        // Walk backward, but stay inside the month.
        let prev = d.add_days(-1);
        if prev.month() != date.month() || prev.year() != date.year() {
            return last;
        }
        d = prev;
        steps += 1;
    }
    d
}

/// Returns `true` if `date` is itself a business day and there is no
/// later business day inside its month.
fn is_last_business_day_of_month(date: Date, is_biz: &impl Fn(Date) -> bool) -> bool {
    if !is_biz(date) {
        return false;
    }
    let mut d = date.add_days(1);
    while d.month() == date.month() && d.year() == date.year() {
        if is_biz(d) {
            return false;
        }
        d = d.add_days(1);
    }
    true
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
        assert_eq!(apply(sat, Roll::Unadjusted, weekends_only), sat);
    }

    // ─── Following / Preceding ───────────────────────────────────────────

    #[test]
    fn following_from_saturday() {
        // 2026-05-23 is a Saturday → next business day is Mon 2026-05-25.
        let sat = Date::ymd(2026, 5, 23).unwrap();
        let mon = Date::ymd(2026, 5, 25).unwrap();
        assert_eq!(apply(sat, Roll::Following, weekends_only), mon);
    }

    #[test]
    fn preceding_from_saturday() {
        // 2026-05-23 is a Saturday → previous business day is Fri 2026-05-22.
        let sat = Date::ymd(2026, 5, 23).unwrap();
        let fri = Date::ymd(2026, 5, 22).unwrap();
        assert_eq!(apply(sat, Roll::Preceding, weekends_only), fri);
    }

    #[test]
    fn following_on_business_day_is_identity() {
        let mon = Date::ymd(2026, 5, 25).unwrap();
        assert_eq!(apply(mon, Roll::Following, weekends_only), mon);
        assert_eq!(apply(mon, Roll::Preceding, weekends_only), mon);
    }

    // ─── ModifiedFollowing / ModifiedPreceding ───────────────────────────

    #[test]
    fn modified_following_falls_back_when_month_changes() {
        // Sunday 2026-05-31 → Following would land on Mon 2026-06-01 (new
        // month). Fall back to Preceding from 2026-05-31 → Fri 2026-05-29.
        let sun = Date::ymd(2026, 5, 31).unwrap();
        let fri = Date::ymd(2026, 5, 29).unwrap();
        assert_eq!(apply(sun, Roll::ModifiedFollowing, weekends_only), fri);
    }

    #[test]
    fn modified_following_within_month_just_follows() {
        // Saturday 2026-05-23 → Following = Mon 2026-05-25, same month. No
        // fall-back.
        let sat = Date::ymd(2026, 5, 23).unwrap();
        let mon = Date::ymd(2026, 5, 25).unwrap();
        assert_eq!(apply(sat, Roll::ModifiedFollowing, weekends_only), mon);
    }

    #[test]
    fn modified_preceding_within_month_just_precedes() {
        // Saturday 2026-01-31 → Preceding = Fri 2026-01-30, same month. No
        // fall-back.
        let sat = Date::ymd(2026, 1, 31).unwrap();
        let fri = Date::ymd(2026, 1, 30).unwrap();
        assert_eq!(apply(sat, Roll::ModifiedPreceding, weekends_only), fri);
    }

    #[test]
    fn modified_preceding_falls_back_when_month_changes() {
        // Sunday 2026-02-01 → Preceding would land on Fri 2026-01-30 (prior
        // month). Fall back to Following from 2026-02-01 → Mon 2026-02-02.
        let sun = Date::ymd(2026, 2, 1).unwrap();
        let mon = Date::ymd(2026, 2, 2).unwrap();
        assert_eq!(apply(sun, Roll::ModifiedPreceding, weekends_only), mon);
    }

    // ─── Nearest ─────────────────────────────────────────────────────────

    #[test]
    fn nearest_business_day_is_identity() {
        let mon = Date::ymd(2026, 5, 25).unwrap();
        assert_eq!(apply(mon, Roll::Nearest, weekends_only), mon);
    }

    #[test]
    fn nearest_on_saturday_goes_friday() {
        // Saturday: Friday is 1 day back, Monday is 2 days forward → Friday.
        let sat = Date::ymd(2026, 5, 23).unwrap();
        let fri = Date::ymd(2026, 5, 22).unwrap();
        assert_eq!(apply(sat, Roll::Nearest, weekends_only), fri);
    }

    #[test]
    fn nearest_on_sunday_goes_monday() {
        // Sunday: Friday is 2 days back, Monday is 1 day forward → Monday.
        let sun = Date::ymd(2026, 5, 24).unwrap();
        let mon = Date::ymd(2026, 5, 25).unwrap();
        assert_eq!(apply(sun, Roll::Nearest, weekends_only), mon);
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
        assert_eq!(apply(wed, Roll::Nearest, is_biz), thu);
    }

    // ─── EndOfMonth ──────────────────────────────────────────────────────

    #[test]
    fn end_of_month_anchors_when_last_business_day() {
        // 2026-02-28 is a Saturday; under weekends-only, the last business
        // day of February 2026 is Fri 2026-02-27. Feeding 2026-02-27 to
        // EndOfMonth keeps it at the last business day of its month (itself).
        let fri = Date::ymd(2026, 2, 27).unwrap();
        assert_eq!(apply(fri, Roll::EndOfMonth, weekends_only), fri);
    }

    #[test]
    fn end_of_month_falls_back_to_modified_following() {
        // 2026-05-15 (Fri) is *not* the last business day of May; under
        // EndOfMonth, fall through to ModifiedFollowing — and since it's a
        // business day, the result is the date itself.
        let mid = Date::ymd(2026, 5, 15).unwrap();
        assert_eq!(apply(mid, Roll::EndOfMonth, weekends_only), mid);
    }

    #[test]
    fn end_of_month_holiday_aware() {
        // With Fri 2026-12-25 as a holiday, the last business day of
        // December 2026 is Thu 2026-12-31 (Thursday). Feeding that in to
        // EndOfMonth must keep it.
        let thu = Date::ymd(2026, 12, 31).unwrap();
        assert_eq!(apply(thu, Roll::EndOfMonth, weekends_plus_christmas), thu);
    }

    // ─── holiday-aware adjustments ───────────────────────────────────────

    #[test]
    fn following_skips_a_holiday() {
        // 2026-12-25 (Fri) is a holiday; Following from that date lands on
        // Mon 2026-12-28 (skipping Sat/Sun and the holiday).
        let xmas = Date::ymd(2026, 12, 25).unwrap();
        let mon = Date::ymd(2026, 12, 28).unwrap();
        assert_eq!(apply(xmas, Roll::Following, weekends_plus_christmas), mon);
    }

    #[test]
    fn preceding_skips_a_holiday() {
        // Preceding from Fri 2026-12-25 (holiday) → Thu 2026-12-24.
        let xmas = Date::ymd(2026, 12, 25).unwrap();
        let thu = Date::ymd(2026, 12, 24).unwrap();
        assert_eq!(apply(xmas, Roll::Preceding, weekends_plus_christmas), thu);
    }

    // ─── defensive cap ───────────────────────────────────────────────────

    #[test]
    fn pathological_predicate_returns_input() {
        // A predicate that never says "business day" must not loop forever.
        let always_false = |_d: Date| false;
        let any = Date::ymd(2026, 5, 23).unwrap();
        assert_eq!(apply(any, Roll::Following, always_false), any);
        assert_eq!(apply(any, Roll::Preceding, always_false), any);
    }
}
