// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! Switzerland — SIX Swiss Exchange trading calendar.
//!
//! SIX Swiss Exchange is the operator of the principal Swiss regulated
//! securities market, headquartered in Zurich; its published trading-day
//! list closes on ten dates each year, listed below. The calendar shipped
//! here is a dated, sorted snapshot of those dates, derived per year from
//! the rules in the table — six fixed Gregorian dates plus four
//! Easter-anchored dates (Good Friday, Easter Monday, Ascension Day, Whit
//! Monday) computed against that year's Western Easter Sunday via the
//! Computus algorithm in [`Date::easter_sunday`].
//!
//! ```text
//! 1.   1 January     New Year's Day            (fixed)
//! 2.   2 January     Berchtoldstag             (fixed)
//! 3.   Good Friday   Easter Sunday − 2 days    (Easter-derived)
//! 4.   Easter Mon.   Easter Sunday + 1 day     (Easter-derived)
//! 5.   1 May         Labour Day                (fixed)
//! 6.   Ascension     Easter Sunday + 39 days   (Easter-derived)
//! 7.   Whit Monday   Easter Sunday + 50 days   (Easter-derived)
//! 8.   1 August      Swiss National Day        (fixed)
//! 9.  25 December    Christmas Day             (fixed)
//! 10. 26 December    St. Stephen's Day         (fixed)
//! ```
//!
//! # No weekend-observance shifts
//!
//! SIX does **not** shift a holiday that falls on a weekend to a
//! neighbouring weekday. When 1 January falls on a Sunday, the market is
//! closed Sunday (which is closed in any case as a weekend) and open as
//! usual on Monday; there is no observed-Monday substitute in the
//! schedule. The table accordingly contains every holiday on its rule
//! date, including the weekend coincidences. Combined with the
//! dispatcher's `is_business_day = !weekend && !is_holiday(d, cal)`
//! predicate, the weekend coincidences are correctly absorbed: a holiday
//! on a Saturday or Sunday adds no further closed day, but the table
//! remains honest as a "list of holidays the market observes" rather than
//! a "list of extra non-business days".
//!
//! # Snapshot
//!
//! - Source: SIX Swiss Exchange trading calendar.
//! - Snapshot date: 2026-05-23 (see [`SNAPSHOT_DATE`]).
//! - Coverage: 2020 through 2040 inclusive (see [`COVERAGE`]).
//! - Entries: 210 (10 holidays × 21 years; no extraordinary closures
//!   in the covered window).
//!
//! Out-of-coverage queries return `false` from [`is_holiday`]; the caller
//! is expected to re-snapshot before extending the horizon.
//!
//! # References
//!
//! - SIX Swiss Exchange, *Trading calendar* — the published list of
//!   exchange closing days for the Zurich-traded Swiss securities
//!   market.
//! - Jean Meeus, *Astronomical Algorithms*, 2nd ed., Willmann-Bell, 1998,
//!   §8 "The date of Easter" — the Computus used by
//!   [`Date::easter_sunday`] to anchor Good Friday, Easter Monday,
//!   Ascension Day, and Whit Monday.
//!
//! # Verification
//!
//! - Verification date: 2026-05-23.
//! - Primary source consulted: SIX Swiss Exchange trading calendar,
//!   <https://www.six-group.com/en/products-services/the-swiss-stock-exchange/market-data/trading-calendar.html>
//!   (alternate URL
//!   <https://www.six-group.com/en/services/trading-calendar.html>);
//!   cross-reference for canton-of-Zurich holiday rules,
//!   <https://en.wikipedia.org/wiki/Public_holidays_in_Switzerland>.
//! - Sample years cross-checked against the published per-year SIX
//!   trading calendar: 2024 (Easter 31 Mar — Good Friday 29 Mar, Easter
//!   Monday 1 Apr, Ascension 9 May, Whit Monday 20 May), 2025 (Easter
//!   20 Apr — Good Friday 18 Apr, Easter Monday 21 Apr, Ascension 29 May,
//!   Whit Monday 9 Jun), and 2026 (Easter 5 Apr — Good Friday 3 Apr,
//!   Easter Monday 6 Apr, Ascension 14 May, Whit Monday 25 May). Every
//!   one of the 10 rule-derived holidays for each sample year appears in
//!   [`HOLIDAYS`] on its rule date.
//! - Rule-derivation invariant: for every year in [`COVERAGE`] the table
//!   carries exactly 10 entries (210 total), with the six fixed dates on
//!   their Gregorian rule date and the four Easter-anchored dates at
//!   `easter − 2`, `easter + 1`, `easter + 39`, `easter + 50`. No
//!   weekend-observance shifts are applied (see "No weekend-observance
//!   shifts" above), in agreement with the SIX published schedule.

use crate::date::Date;

/// Date of the SIX Swiss Exchange trading-calendar snapshot embedded in
/// this module, `YYYY-MM-DD`.
pub const SNAPSHOT_DATE: &str = "2026-05-23";

/// Inclusive year range covered by [`HOLIDAYS`]: `(first_year, last_year)`.
/// Queries outside this window return `false` from [`is_holiday`].
pub const COVERAGE: (i32, i32) = (2020, 2040);

/// The SIX Swiss Exchange holiday table, sorted ascending by
/// `(year, month, day)`.
///
/// Each entry is a `(year, month, day)` triple of a date the exchange
/// closes for trading. The table is generated per the ten-rule schedule
/// described in the module-level documentation, applied to every year in
/// [`COVERAGE`]; Easter-anchored entries are computed against that year's
/// Western Easter Sunday. Holidays that coincide with a Saturday or
/// Sunday are included on their rule date — SIX does not shift weekend
/// holidays to an observed weekday.
pub const HOLIDAYS: &[(i32, u8, u8)] = &[
    // ─── 2020 (Easter Sunday 2020-04-12) ────────────────────────────────
    (2020, 1, 1),   // 2020-01-01 — New Year's Day
    (2020, 1, 2),   // 2020-01-02 — Berchtoldstag
    (2020, 4, 10),  // 2020-04-10 — Good Friday
    (2020, 4, 13),  // 2020-04-13 — Easter Monday
    (2020, 5, 1),   // 2020-05-01 — Labour Day
    (2020, 5, 21),  // 2020-05-21 — Ascension Day
    (2020, 6, 1),   // 2020-06-01 — Whit Monday
    (2020, 8, 1),   // 2020-08-01 — Swiss National Day
    (2020, 12, 25), // 2020-12-25 — Christmas Day
    (2020, 12, 26), // 2020-12-26 — St. Stephen's Day
    // ─── 2021 (Easter Sunday 2021-04-04) ────────────────────────────────
    (2021, 1, 1),   // 2021-01-01 — New Year's Day
    (2021, 1, 2),   // 2021-01-02 — Berchtoldstag
    (2021, 4, 2),   // 2021-04-02 — Good Friday
    (2021, 4, 5),   // 2021-04-05 — Easter Monday
    (2021, 5, 1),   // 2021-05-01 — Labour Day
    (2021, 5, 13),  // 2021-05-13 — Ascension Day
    (2021, 5, 24),  // 2021-05-24 — Whit Monday
    (2021, 8, 1),   // 2021-08-01 — Swiss National Day
    (2021, 12, 25), // 2021-12-25 — Christmas Day
    (2021, 12, 26), // 2021-12-26 — St. Stephen's Day
    // ─── 2022 (Easter Sunday 2022-04-17) ────────────────────────────────
    (2022, 1, 1),   // 2022-01-01 — New Year's Day
    (2022, 1, 2),   // 2022-01-02 — Berchtoldstag
    (2022, 4, 15),  // 2022-04-15 — Good Friday
    (2022, 4, 18),  // 2022-04-18 — Easter Monday
    (2022, 5, 1),   // 2022-05-01 — Labour Day
    (2022, 5, 26),  // 2022-05-26 — Ascension Day
    (2022, 6, 6),   // 2022-06-06 — Whit Monday
    (2022, 8, 1),   // 2022-08-01 — Swiss National Day
    (2022, 12, 25), // 2022-12-25 — Christmas Day
    (2022, 12, 26), // 2022-12-26 — St. Stephen's Day
    // ─── 2023 (Easter Sunday 2023-04-09) ────────────────────────────────
    (2023, 1, 1),   // 2023-01-01 — New Year's Day
    (2023, 1, 2),   // 2023-01-02 — Berchtoldstag
    (2023, 4, 7),   // 2023-04-07 — Good Friday
    (2023, 4, 10),  // 2023-04-10 — Easter Monday
    (2023, 5, 1),   // 2023-05-01 — Labour Day
    (2023, 5, 18),  // 2023-05-18 — Ascension Day
    (2023, 5, 29),  // 2023-05-29 — Whit Monday
    (2023, 8, 1),   // 2023-08-01 — Swiss National Day
    (2023, 12, 25), // 2023-12-25 — Christmas Day
    (2023, 12, 26), // 2023-12-26 — St. Stephen's Day
    // ─── 2024 (Easter Sunday 2024-03-31) ────────────────────────────────
    (2024, 1, 1),   // 2024-01-01 — New Year's Day
    (2024, 1, 2),   // 2024-01-02 — Berchtoldstag
    (2024, 3, 29),  // 2024-03-29 — Good Friday
    (2024, 4, 1),   // 2024-04-01 — Easter Monday
    (2024, 5, 1),   // 2024-05-01 — Labour Day
    (2024, 5, 9),   // 2024-05-09 — Ascension Day
    (2024, 5, 20),  // 2024-05-20 — Whit Monday
    (2024, 8, 1),   // 2024-08-01 — Swiss National Day
    (2024, 12, 25), // 2024-12-25 — Christmas Day
    (2024, 12, 26), // 2024-12-26 — St. Stephen's Day
    // ─── 2025 (Easter Sunday 2025-04-20) ────────────────────────────────
    (2025, 1, 1),   // 2025-01-01 — New Year's Day
    (2025, 1, 2),   // 2025-01-02 — Berchtoldstag
    (2025, 4, 18),  // 2025-04-18 — Good Friday
    (2025, 4, 21),  // 2025-04-21 — Easter Monday
    (2025, 5, 1),   // 2025-05-01 — Labour Day
    (2025, 5, 29),  // 2025-05-29 — Ascension Day
    (2025, 6, 9),   // 2025-06-09 — Whit Monday
    (2025, 8, 1),   // 2025-08-01 — Swiss National Day
    (2025, 12, 25), // 2025-12-25 — Christmas Day
    (2025, 12, 26), // 2025-12-26 — St. Stephen's Day
    // ─── 2026 (Easter Sunday 2026-04-05) ────────────────────────────────
    (2026, 1, 1),   // 2026-01-01 — New Year's Day
    (2026, 1, 2),   // 2026-01-02 — Berchtoldstag
    (2026, 4, 3),   // 2026-04-03 — Good Friday
    (2026, 4, 6),   // 2026-04-06 — Easter Monday
    (2026, 5, 1),   // 2026-05-01 — Labour Day
    (2026, 5, 14),  // 2026-05-14 — Ascension Day
    (2026, 5, 25),  // 2026-05-25 — Whit Monday
    (2026, 8, 1),   // 2026-08-01 — Swiss National Day
    (2026, 12, 25), // 2026-12-25 — Christmas Day
    (2026, 12, 26), // 2026-12-26 — St. Stephen's Day
    // ─── 2027 (Easter Sunday 2027-03-28) ────────────────────────────────
    (2027, 1, 1),   // 2027-01-01 — New Year's Day
    (2027, 1, 2),   // 2027-01-02 — Berchtoldstag
    (2027, 3, 26),  // 2027-03-26 — Good Friday
    (2027, 3, 29),  // 2027-03-29 — Easter Monday
    (2027, 5, 1),   // 2027-05-01 — Labour Day
    (2027, 5, 6),   // 2027-05-06 — Ascension Day
    (2027, 5, 17),  // 2027-05-17 — Whit Monday
    (2027, 8, 1),   // 2027-08-01 — Swiss National Day
    (2027, 12, 25), // 2027-12-25 — Christmas Day
    (2027, 12, 26), // 2027-12-26 — St. Stephen's Day
    // ─── 2028 (Easter Sunday 2028-04-16) ────────────────────────────────
    (2028, 1, 1),   // 2028-01-01 — New Year's Day
    (2028, 1, 2),   // 2028-01-02 — Berchtoldstag
    (2028, 4, 14),  // 2028-04-14 — Good Friday
    (2028, 4, 17),  // 2028-04-17 — Easter Monday
    (2028, 5, 1),   // 2028-05-01 — Labour Day
    (2028, 5, 25),  // 2028-05-25 — Ascension Day
    (2028, 6, 5),   // 2028-06-05 — Whit Monday
    (2028, 8, 1),   // 2028-08-01 — Swiss National Day
    (2028, 12, 25), // 2028-12-25 — Christmas Day
    (2028, 12, 26), // 2028-12-26 — St. Stephen's Day
    // ─── 2029 (Easter Sunday 2029-04-01) ────────────────────────────────
    (2029, 1, 1),   // 2029-01-01 — New Year's Day
    (2029, 1, 2),   // 2029-01-02 — Berchtoldstag
    (2029, 3, 30),  // 2029-03-30 — Good Friday
    (2029, 4, 2),   // 2029-04-02 — Easter Monday
    (2029, 5, 1),   // 2029-05-01 — Labour Day
    (2029, 5, 10),  // 2029-05-10 — Ascension Day
    (2029, 5, 21),  // 2029-05-21 — Whit Monday
    (2029, 8, 1),   // 2029-08-01 — Swiss National Day
    (2029, 12, 25), // 2029-12-25 — Christmas Day
    (2029, 12, 26), // 2029-12-26 — St. Stephen's Day
    // ─── 2030 (Easter Sunday 2030-04-21) ────────────────────────────────
    (2030, 1, 1),   // 2030-01-01 — New Year's Day
    (2030, 1, 2),   // 2030-01-02 — Berchtoldstag
    (2030, 4, 19),  // 2030-04-19 — Good Friday
    (2030, 4, 22),  // 2030-04-22 — Easter Monday
    (2030, 5, 1),   // 2030-05-01 — Labour Day
    (2030, 5, 30),  // 2030-05-30 — Ascension Day
    (2030, 6, 10),  // 2030-06-10 — Whit Monday
    (2030, 8, 1),   // 2030-08-01 — Swiss National Day
    (2030, 12, 25), // 2030-12-25 — Christmas Day
    (2030, 12, 26), // 2030-12-26 — St. Stephen's Day
    // ─── 2031 (Easter Sunday 2031-04-13) ────────────────────────────────
    (2031, 1, 1),   // 2031-01-01 — New Year's Day
    (2031, 1, 2),   // 2031-01-02 — Berchtoldstag
    (2031, 4, 11),  // 2031-04-11 — Good Friday
    (2031, 4, 14),  // 2031-04-14 — Easter Monday
    (2031, 5, 1),   // 2031-05-01 — Labour Day
    (2031, 5, 22),  // 2031-05-22 — Ascension Day
    (2031, 6, 2),   // 2031-06-02 — Whit Monday
    (2031, 8, 1),   // 2031-08-01 — Swiss National Day
    (2031, 12, 25), // 2031-12-25 — Christmas Day
    (2031, 12, 26), // 2031-12-26 — St. Stephen's Day
    // ─── 2032 (Easter Sunday 2032-03-28) ────────────────────────────────
    (2032, 1, 1),   // 2032-01-01 — New Year's Day
    (2032, 1, 2),   // 2032-01-02 — Berchtoldstag
    (2032, 3, 26),  // 2032-03-26 — Good Friday
    (2032, 3, 29),  // 2032-03-29 — Easter Monday
    (2032, 5, 1),   // 2032-05-01 — Labour Day
    (2032, 5, 6),   // 2032-05-06 — Ascension Day
    (2032, 5, 17),  // 2032-05-17 — Whit Monday
    (2032, 8, 1),   // 2032-08-01 — Swiss National Day
    (2032, 12, 25), // 2032-12-25 — Christmas Day
    (2032, 12, 26), // 2032-12-26 — St. Stephen's Day
    // ─── 2033 (Easter Sunday 2033-04-17) ────────────────────────────────
    (2033, 1, 1),   // 2033-01-01 — New Year's Day
    (2033, 1, 2),   // 2033-01-02 — Berchtoldstag
    (2033, 4, 15),  // 2033-04-15 — Good Friday
    (2033, 4, 18),  // 2033-04-18 — Easter Monday
    (2033, 5, 1),   // 2033-05-01 — Labour Day
    (2033, 5, 26),  // 2033-05-26 — Ascension Day
    (2033, 6, 6),   // 2033-06-06 — Whit Monday
    (2033, 8, 1),   // 2033-08-01 — Swiss National Day
    (2033, 12, 25), // 2033-12-25 — Christmas Day
    (2033, 12, 26), // 2033-12-26 — St. Stephen's Day
    // ─── 2034 (Easter Sunday 2034-04-09) ────────────────────────────────
    (2034, 1, 1),   // 2034-01-01 — New Year's Day
    (2034, 1, 2),   // 2034-01-02 — Berchtoldstag
    (2034, 4, 7),   // 2034-04-07 — Good Friday
    (2034, 4, 10),  // 2034-04-10 — Easter Monday
    (2034, 5, 1),   // 2034-05-01 — Labour Day
    (2034, 5, 18),  // 2034-05-18 — Ascension Day
    (2034, 5, 29),  // 2034-05-29 — Whit Monday
    (2034, 8, 1),   // 2034-08-01 — Swiss National Day
    (2034, 12, 25), // 2034-12-25 — Christmas Day
    (2034, 12, 26), // 2034-12-26 — St. Stephen's Day
    // ─── 2035 (Easter Sunday 2035-03-25) ────────────────────────────────
    (2035, 1, 1),   // 2035-01-01 — New Year's Day
    (2035, 1, 2),   // 2035-01-02 — Berchtoldstag
    (2035, 3, 23),  // 2035-03-23 — Good Friday
    (2035, 3, 26),  // 2035-03-26 — Easter Monday
    (2035, 5, 1),   // 2035-05-01 — Labour Day
    (2035, 5, 3),   // 2035-05-03 — Ascension Day
    (2035, 5, 14),  // 2035-05-14 — Whit Monday
    (2035, 8, 1),   // 2035-08-01 — Swiss National Day
    (2035, 12, 25), // 2035-12-25 — Christmas Day
    (2035, 12, 26), // 2035-12-26 — St. Stephen's Day
    // ─── 2036 (Easter Sunday 2036-04-13) ────────────────────────────────
    (2036, 1, 1),   // 2036-01-01 — New Year's Day
    (2036, 1, 2),   // 2036-01-02 — Berchtoldstag
    (2036, 4, 11),  // 2036-04-11 — Good Friday
    (2036, 4, 14),  // 2036-04-14 — Easter Monday
    (2036, 5, 1),   // 2036-05-01 — Labour Day
    (2036, 5, 22),  // 2036-05-22 — Ascension Day
    (2036, 6, 2),   // 2036-06-02 — Whit Monday
    (2036, 8, 1),   // 2036-08-01 — Swiss National Day
    (2036, 12, 25), // 2036-12-25 — Christmas Day
    (2036, 12, 26), // 2036-12-26 — St. Stephen's Day
    // ─── 2037 (Easter Sunday 2037-04-05) ────────────────────────────────
    (2037, 1, 1),   // 2037-01-01 — New Year's Day
    (2037, 1, 2),   // 2037-01-02 — Berchtoldstag
    (2037, 4, 3),   // 2037-04-03 — Good Friday
    (2037, 4, 6),   // 2037-04-06 — Easter Monday
    (2037, 5, 1),   // 2037-05-01 — Labour Day
    (2037, 5, 14),  // 2037-05-14 — Ascension Day
    (2037, 5, 25),  // 2037-05-25 — Whit Monday
    (2037, 8, 1),   // 2037-08-01 — Swiss National Day
    (2037, 12, 25), // 2037-12-25 — Christmas Day
    (2037, 12, 26), // 2037-12-26 — St. Stephen's Day
    // ─── 2038 (Easter Sunday 2038-04-25) ────────────────────────────────
    (2038, 1, 1),   // 2038-01-01 — New Year's Day
    (2038, 1, 2),   // 2038-01-02 — Berchtoldstag
    (2038, 4, 23),  // 2038-04-23 — Good Friday
    (2038, 4, 26),  // 2038-04-26 — Easter Monday
    (2038, 5, 1),   // 2038-05-01 — Labour Day
    (2038, 6, 3),   // 2038-06-03 — Ascension Day
    (2038, 6, 14),  // 2038-06-14 — Whit Monday
    (2038, 8, 1),   // 2038-08-01 — Swiss National Day
    (2038, 12, 25), // 2038-12-25 — Christmas Day
    (2038, 12, 26), // 2038-12-26 — St. Stephen's Day
    // ─── 2039 (Easter Sunday 2039-04-10) ────────────────────────────────
    (2039, 1, 1),   // 2039-01-01 — New Year's Day
    (2039, 1, 2),   // 2039-01-02 — Berchtoldstag
    (2039, 4, 8),   // 2039-04-08 — Good Friday
    (2039, 4, 11),  // 2039-04-11 — Easter Monday
    (2039, 5, 1),   // 2039-05-01 — Labour Day
    (2039, 5, 19),  // 2039-05-19 — Ascension Day
    (2039, 5, 30),  // 2039-05-30 — Whit Monday
    (2039, 8, 1),   // 2039-08-01 — Swiss National Day
    (2039, 12, 25), // 2039-12-25 — Christmas Day
    (2039, 12, 26), // 2039-12-26 — St. Stephen's Day
    // ─── 2040 (Easter Sunday 2040-04-01) ────────────────────────────────
    (2040, 1, 1),   // 2040-01-01 — New Year's Day
    (2040, 1, 2),   // 2040-01-02 — Berchtoldstag
    (2040, 3, 30),  // 2040-03-30 — Good Friday
    (2040, 4, 2),   // 2040-04-02 — Easter Monday
    (2040, 5, 1),   // 2040-05-01 — Labour Day
    (2040, 5, 10),  // 2040-05-10 — Ascension Day
    (2040, 5, 21),  // 2040-05-21 — Whit Monday
    (2040, 8, 1),   // 2040-08-01 — Swiss National Day
    (2040, 12, 25), // 2040-12-25 — Christmas Day
    (2040, 12, 26), // 2040-12-26 — St. Stephen's Day
];

/// Returns `true` if `date` is a SIX Swiss Exchange holiday under the
/// embedded snapshot.
///
/// Implemented as a binary search over [`HOLIDAYS`], which is sorted
/// ascending by `(year, month, day)`. Dates with `date.year()` outside
/// [`COVERAGE`] return `false`: the snapshot does not extrapolate, and
/// extending the horizon requires re-snapshotting the published SIX
/// trading calendar.
///
/// Weekends are NOT classified as holidays here — they are handled by
/// the caller's business-day predicate (see the `is_business_day`
/// dispatcher in [`crate::calendar`]). A SIX holiday that coincides with
/// a Saturday or Sunday is reported on its rule date all the same; SIX
/// does not shift weekend holidays to an observed weekday, so no extra
/// non-business day is introduced by the coincidence.
///
/// # Examples
///
/// ```
/// use regit_daycount::Date;
/// use regit_daycount::calendar::switzerland;
///
/// // Swiss National Day 2026 (Saturday) is a SIX holiday on its rule date.
/// assert!(switzerland::is_holiday(Date::ymd(2026, 8, 1).unwrap()));
///
/// // The Sunday immediately after is not a SIX holiday — it is closed as
/// // a weekend, but the weekend rule is the caller's job, not this
/// // function's.
/// assert!(!switzerland::is_holiday(Date::ymd(2026, 8, 2).unwrap()));
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
            (12, 25), // Christmas Day
            (12, 26), // St. Stephen's Day
        ];
        for (m, d) in dates {
            assert!(
                is_holiday(Date::ymd(2024, m, d).unwrap()),
                "2024-{m:02}-{d:02} must be a SIX holiday",
            );
        }
        // Count the 2024 entries in HOLIDAYS — must be exactly ten.
        let count_2024 = HOLIDAYS.iter().filter(|(y, _, _)| *y == 2024).count();
        assert_eq!(count_2024, 10, "2024 must have exactly 10 SIX holidays");
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
            (8, 1),   // Swiss National Day
            (12, 25), // Christmas Day
            (12, 26), // St. Stephen's Day
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
    fn no_weekend_observance_shift() {
        // 2022-01-01 was a Saturday. SIX does not move New Year's Day to
        // Monday — the rule date itself is in HOLIDAYS, and the
        // following Monday (2022-01-03) is NOT.
        assert!(
            is_holiday(Date::ymd(2022, 1, 1).unwrap()),
            "2022-01-01 (Sat) must remain a SIX holiday on its rule date",
        );
        assert!(
            !is_holiday(Date::ymd(2022, 1, 3).unwrap()),
            "2022-01-03 (Mon) must NOT be a SIX holiday — no observed shift",
        );
        // (2022-01-02 IS a holiday — Berchtoldstag — but that is its own
        // rule date, not an observance of New Year's Day.)
        assert!(
            is_holiday(Date::ymd(2022, 1, 2).unwrap()),
            "2022-01-02 must be a SIX holiday on its own rule (Berchtoldstag)",
        );
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
    fn holidays_total_count_matches_rule() {
        // 10 holidays per year, 21 years (2020..=2040 inclusive). No
        // extraordinary closures in this calendar's covered window.
        assert_eq!(HOLIDAYS.len(), 10 * 21, "expected 210 SIX holiday entries");
    }
}
