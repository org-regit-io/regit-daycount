// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! United States — NYSE / Federal Reserve holiday calendar.
//!
//! Embedded, dated snapshot of the holidays observed by the New York Stock
//! Exchange and the US Federal Reserve. The NYSE schedule is a superset of
//! the Federal Reserve operating-day list (the NYSE adds Good Friday); for
//! the purposes of US bond and equity settlement dates this module models
//! the NYSE list, which is the conservative — and standard — choice. The
//! table covers the inclusive window `2020..=2040`.
//!
//! # Holiday rules
//!
//! | Holiday                                  | Rule                                                                                     |
//! |------------------------------------------|------------------------------------------------------------------------------------------|
//! | New Year's Day                           | Jan 1 (observed Fri preceding if Sat; observed Mon following if Sun)                     |
//! | Martin Luther King Jr. Day               | Third Monday of January                                                                  |
//! | Washington's Birthday (Presidents' Day)  | Third Monday of February                                                                 |
//! | Good Friday                              | Easter Sunday − 2 days                                                                   |
//! | Memorial Day                             | Last Monday of May                                                                       |
//! | Juneteenth (from 2022)                   | Jun 19 (observed Mon following if Sun; observed Fri preceding if Sat); NYSE since 2022   |
//! | Independence Day                         | Jul 4 (observed Fri preceding if Sat; observed Mon following if Sun)                     |
//! | Labor Day                                | First Monday of September                                                                |
//! | Thanksgiving Day                         | Fourth Thursday of November                                                              |
//! | Christmas Day                            | Dec 25 (observed Fri preceding if Sat; observed Mon following if Sun)                    |
//!
//! Discretionary closures are out of scope. The NYSE has from time to time
//! closed the trading floor for events that no rule predicts — deaths of US
//! presidents (e.g. George H. W. Bush, 2018-12-05), national days of mourning,
//! and the four-session weather closure for Hurricane Sandy (2012-10-29 and
//! 2012-10-30). These dates are published year by year in NYSE notices and
//! cannot be derived from a calendar rule; they are not included here. A
//! caller that needs them must overlay a per-year exceptions list of its own.
//!
//! # Snapshot
//!
//! - Snapshot date: see [`SNAPSHOT_DATE`].
//! - Coverage (inclusive year range): see [`COVERAGE`].
//! - Entries: 208 observed holiday dates (10 holidays × 21 years − 2
//!   Juneteenth dates for 2020 and 2021 when it was not yet a NYSE holiday).
//!
//! # References
//!
//! - NYSE Holiday Calendar, published at
//!   <https://www.nyse.com/markets/hours-calendars>.
//! - Federal Reserve Bank Services holiday schedule, published at
//!   <https://www.frbservices.org/about/holiday-schedules>.
//! - SEC / NYSE notices for discretionary closures (e.g. Hurricane Sandy,
//!   2012-10-29 and 2012-10-30; George H. W. Bush state funeral, 2018-12-05).
//!
//! # Verification
//!
//! On 2026-05-23 every row of [`HOLIDAYS`] was cross-checked against the
//! calendar rules in the table above. The check is also encoded as an
//! in-crate regression test (`every_holiday_row_matches_its_calendar_rule`)
//! that re-derives the entire snapshot from the rules each run, so future
//! edits to the table cannot drift from the documented rule set:
//!
//! - Fixed-date holidays (New Year's Day, Juneteenth, Independence Day,
//!   Christmas Day) are weekend-shifted Sat → Fri preceding and Sun → Mon
//!   following, matching NYSE and Federal Reserve practice.
//! - Floating Monday holidays (MLK, Washington's Birthday, Memorial Day,
//!   Labor Day) and Thanksgiving (4th Thursday) are derived with
//!   [`Date::nth_weekday_of_month`](crate::Date::nth_weekday_of_month).
//! - Good Friday is derived as
//!   [`Date::easter_sunday`](crate::Date::easter_sunday) − 2 days using the
//!   Meeus / Anonymous-Gregorian algorithm.
//! - Juneteenth rows start in 2022 — the year the NYSE first observed it
//!   (the federal holiday took effect 2021-06-17).
//!
//! For the years 2020 through ~2027 the rule-derived dates are
//! independently cross-checkable against the NYSE Holiday Calendar and the
//! Federal Reserve schedule linked above. For later years (publication
//! windows on both sites only extend ~3 years forward) the rows are
//! rule-derived; they will match the official lists once published because
//! the rules themselves are statutory and unchanged.

use crate::date::Date;

// ─── Snapshot metadata ───────────────────────────────────────────────────────

/// Date of the NYSE / Federal Reserve holiday-list snapshot embedded in this
/// module, `YYYY-MM-DD`. Holiday observances are published years in advance
/// but the static table here is the authoritative copy as of this date.
pub const SNAPSHOT_DATE: &str = "2026-05-23";

/// Inclusive `(first_year, last_year)` year range covered by [`HOLIDAYS`].
/// Outside this range, [`is_holiday`] returns `false` — the snapshot has
/// nothing to say about dates the caller has not opted into.
pub const COVERAGE: (i32, i32) = (2020, 2040);

// ─── Holiday table ───────────────────────────────────────────────────────────

/// Every observed NYSE / Federal Reserve holiday in
/// `COVERAGE.0..=COVERAGE.1`, sorted ascending so [`is_holiday`] can binary
/// search the table.
///
/// Each row is a `(year, month, day)` triple of the **observed** holiday
/// date — the weekend-shift rules in the module docstring have already been
/// applied, so a row of `(2021, 12, 31)` is the observed date for the
/// 2022 New Year's Day holiday (2022-01-01 fell on a Saturday).
///
/// The list is hand-listed for auditor friendliness — every date can be
/// checked against the NYSE published calendar without running code.
pub const HOLIDAYS: &[(i32, u8, u8)] = &[
    // ─── 2020 ────────────────────────────────────────────────────────────
    (2020, 1, 1),   // 2020-01-01 — New Year's Day
    (2020, 1, 20),  // 2020-01-20 — Martin Luther King Jr. Day
    (2020, 2, 17),  // 2020-02-17 — Washington's Birthday
    (2020, 4, 10),  // 2020-04-10 — Good Friday
    (2020, 5, 25),  // 2020-05-25 — Memorial Day
    (2020, 7, 3),   // 2020-07-03 — Independence Day (observed; Jul 4 was Sat)
    (2020, 9, 7),   // 2020-09-07 — Labor Day
    (2020, 11, 26), // 2020-11-26 — Thanksgiving Day
    (2020, 12, 25), // 2020-12-25 — Christmas Day
    // ─── 2021 ────────────────────────────────────────────────────────────
    (2021, 1, 1),   // 2021-01-01 — New Year's Day
    (2021, 1, 18),  // 2021-01-18 — Martin Luther King Jr. Day
    (2021, 2, 15),  // 2021-02-15 — Washington's Birthday
    (2021, 4, 2),   // 2021-04-02 — Good Friday
    (2021, 5, 31),  // 2021-05-31 — Memorial Day
    (2021, 7, 5),   // 2021-07-05 — Independence Day (observed; Jul 4 was Sun)
    (2021, 9, 6),   // 2021-09-06 — Labor Day
    (2021, 11, 25), // 2021-11-25 — Thanksgiving Day
    (2021, 12, 24), // 2021-12-24 — Christmas Day (observed; Dec 25 was Sat)
    (2021, 12, 31), // 2021-12-31 — New Year's Day 2022 (observed; Jan 1 was Sat)
    // ─── 2022 ────────────────────────────────────────────────────────────
    (2022, 1, 17),  // 2022-01-17 — Martin Luther King Jr. Day
    (2022, 2, 21),  // 2022-02-21 — Washington's Birthday
    (2022, 4, 15),  // 2022-04-15 — Good Friday
    (2022, 5, 30),  // 2022-05-30 — Memorial Day
    (2022, 6, 20),  // 2022-06-20 — Juneteenth (observed; Jun 19 was Sun)
    (2022, 7, 4),   // 2022-07-04 — Independence Day
    (2022, 9, 5),   // 2022-09-05 — Labor Day
    (2022, 11, 24), // 2022-11-24 — Thanksgiving Day
    (2022, 12, 26), // 2022-12-26 — Christmas Day (observed; Dec 25 was Sun)
    // ─── 2023 ────────────────────────────────────────────────────────────
    (2023, 1, 2),   // 2023-01-02 — New Year's Day (observed; Jan 1 was Sun)
    (2023, 1, 16),  // 2023-01-16 — Martin Luther King Jr. Day
    (2023, 2, 20),  // 2023-02-20 — Washington's Birthday
    (2023, 4, 7),   // 2023-04-07 — Good Friday
    (2023, 5, 29),  // 2023-05-29 — Memorial Day
    (2023, 6, 19),  // 2023-06-19 — Juneteenth
    (2023, 7, 4),   // 2023-07-04 — Independence Day
    (2023, 9, 4),   // 2023-09-04 — Labor Day
    (2023, 11, 23), // 2023-11-23 — Thanksgiving Day
    (2023, 12, 25), // 2023-12-25 — Christmas Day
    // ─── 2024 ────────────────────────────────────────────────────────────
    (2024, 1, 1),   // 2024-01-01 — New Year's Day
    (2024, 1, 15),  // 2024-01-15 — Martin Luther King Jr. Day
    (2024, 2, 19),  // 2024-02-19 — Washington's Birthday
    (2024, 3, 29),  // 2024-03-29 — Good Friday
    (2024, 5, 27),  // 2024-05-27 — Memorial Day
    (2024, 6, 19),  // 2024-06-19 — Juneteenth
    (2024, 7, 4),   // 2024-07-04 — Independence Day
    (2024, 9, 2),   // 2024-09-02 — Labor Day
    (2024, 11, 28), // 2024-11-28 — Thanksgiving Day
    (2024, 12, 25), // 2024-12-25 — Christmas Day
    // ─── 2025 ────────────────────────────────────────────────────────────
    (2025, 1, 1),   // 2025-01-01 — New Year's Day
    (2025, 1, 20),  // 2025-01-20 — Martin Luther King Jr. Day
    (2025, 2, 17),  // 2025-02-17 — Washington's Birthday
    (2025, 4, 18),  // 2025-04-18 — Good Friday
    (2025, 5, 26),  // 2025-05-26 — Memorial Day
    (2025, 6, 19),  // 2025-06-19 — Juneteenth
    (2025, 7, 4),   // 2025-07-04 — Independence Day
    (2025, 9, 1),   // 2025-09-01 — Labor Day
    (2025, 11, 27), // 2025-11-27 — Thanksgiving Day
    (2025, 12, 25), // 2025-12-25 — Christmas Day
    // ─── 2026 ────────────────────────────────────────────────────────────
    (2026, 1, 1),   // 2026-01-01 — New Year's Day
    (2026, 1, 19),  // 2026-01-19 — Martin Luther King Jr. Day
    (2026, 2, 16),  // 2026-02-16 — Washington's Birthday
    (2026, 4, 3),   // 2026-04-03 — Good Friday
    (2026, 5, 25),  // 2026-05-25 — Memorial Day
    (2026, 6, 19),  // 2026-06-19 — Juneteenth
    (2026, 7, 3),   // 2026-07-03 — Independence Day (observed; Jul 4 was Sat)
    (2026, 9, 7),   // 2026-09-07 — Labor Day
    (2026, 11, 26), // 2026-11-26 — Thanksgiving Day
    (2026, 12, 25), // 2026-12-25 — Christmas Day
    // ─── 2027 ────────────────────────────────────────────────────────────
    (2027, 1, 1),   // 2027-01-01 — New Year's Day
    (2027, 1, 18),  // 2027-01-18 — Martin Luther King Jr. Day
    (2027, 2, 15),  // 2027-02-15 — Washington's Birthday
    (2027, 3, 26),  // 2027-03-26 — Good Friday
    (2027, 5, 31),  // 2027-05-31 — Memorial Day
    (2027, 6, 18),  // 2027-06-18 — Juneteenth (observed; Jun 19 was Sat)
    (2027, 7, 5),   // 2027-07-05 — Independence Day (observed; Jul 4 was Sun)
    (2027, 9, 6),   // 2027-09-06 — Labor Day
    (2027, 11, 25), // 2027-11-25 — Thanksgiving Day
    (2027, 12, 24), // 2027-12-24 — Christmas Day (observed; Dec 25 was Sat)
    (2027, 12, 31), // 2027-12-31 — New Year's Day 2028 (observed; Jan 1 was Sat)
    // ─── 2028 ────────────────────────────────────────────────────────────
    (2028, 1, 17),  // 2028-01-17 — Martin Luther King Jr. Day
    (2028, 2, 21),  // 2028-02-21 — Washington's Birthday
    (2028, 4, 14),  // 2028-04-14 — Good Friday
    (2028, 5, 29),  // 2028-05-29 — Memorial Day
    (2028, 6, 19),  // 2028-06-19 — Juneteenth
    (2028, 7, 4),   // 2028-07-04 — Independence Day
    (2028, 9, 4),   // 2028-09-04 — Labor Day
    (2028, 11, 23), // 2028-11-23 — Thanksgiving Day
    (2028, 12, 25), // 2028-12-25 — Christmas Day
    // ─── 2029 ────────────────────────────────────────────────────────────
    (2029, 1, 1),   // 2029-01-01 — New Year's Day
    (2029, 1, 15),  // 2029-01-15 — Martin Luther King Jr. Day
    (2029, 2, 19),  // 2029-02-19 — Washington's Birthday
    (2029, 3, 30),  // 2029-03-30 — Good Friday
    (2029, 5, 28),  // 2029-05-28 — Memorial Day
    (2029, 6, 19),  // 2029-06-19 — Juneteenth
    (2029, 7, 4),   // 2029-07-04 — Independence Day
    (2029, 9, 3),   // 2029-09-03 — Labor Day
    (2029, 11, 22), // 2029-11-22 — Thanksgiving Day
    (2029, 12, 25), // 2029-12-25 — Christmas Day
    // ─── 2030 ────────────────────────────────────────────────────────────
    (2030, 1, 1),   // 2030-01-01 — New Year's Day
    (2030, 1, 21),  // 2030-01-21 — Martin Luther King Jr. Day
    (2030, 2, 18),  // 2030-02-18 — Washington's Birthday
    (2030, 4, 19),  // 2030-04-19 — Good Friday
    (2030, 5, 27),  // 2030-05-27 — Memorial Day
    (2030, 6, 19),  // 2030-06-19 — Juneteenth
    (2030, 7, 4),   // 2030-07-04 — Independence Day
    (2030, 9, 2),   // 2030-09-02 — Labor Day
    (2030, 11, 28), // 2030-11-28 — Thanksgiving Day
    (2030, 12, 25), // 2030-12-25 — Christmas Day
    // ─── 2031 ────────────────────────────────────────────────────────────
    (2031, 1, 1),   // 2031-01-01 — New Year's Day
    (2031, 1, 20),  // 2031-01-20 — Martin Luther King Jr. Day
    (2031, 2, 17),  // 2031-02-17 — Washington's Birthday
    (2031, 4, 11),  // 2031-04-11 — Good Friday
    (2031, 5, 26),  // 2031-05-26 — Memorial Day
    (2031, 6, 19),  // 2031-06-19 — Juneteenth
    (2031, 7, 4),   // 2031-07-04 — Independence Day
    (2031, 9, 1),   // 2031-09-01 — Labor Day
    (2031, 11, 27), // 2031-11-27 — Thanksgiving Day
    (2031, 12, 25), // 2031-12-25 — Christmas Day
    // ─── 2032 ────────────────────────────────────────────────────────────
    (2032, 1, 1),   // 2032-01-01 — New Year's Day
    (2032, 1, 19),  // 2032-01-19 — Martin Luther King Jr. Day
    (2032, 2, 16),  // 2032-02-16 — Washington's Birthday
    (2032, 3, 26),  // 2032-03-26 — Good Friday
    (2032, 5, 31),  // 2032-05-31 — Memorial Day
    (2032, 6, 18),  // 2032-06-18 — Juneteenth (observed; Jun 19 was Sat)
    (2032, 7, 5),   // 2032-07-05 — Independence Day (observed; Jul 4 was Sun)
    (2032, 9, 6),   // 2032-09-06 — Labor Day
    (2032, 11, 25), // 2032-11-25 — Thanksgiving Day
    (2032, 12, 24), // 2032-12-24 — Christmas Day (observed; Dec 25 was Sat)
    (2032, 12, 31), // 2032-12-31 — New Year's Day 2033 (observed; Jan 1 was Sat)
    // ─── 2033 ────────────────────────────────────────────────────────────
    (2033, 1, 17),  // 2033-01-17 — Martin Luther King Jr. Day
    (2033, 2, 21),  // 2033-02-21 — Washington's Birthday
    (2033, 4, 15),  // 2033-04-15 — Good Friday
    (2033, 5, 30),  // 2033-05-30 — Memorial Day
    (2033, 6, 20),  // 2033-06-20 — Juneteenth (observed; Jun 19 was Sun)
    (2033, 7, 4),   // 2033-07-04 — Independence Day
    (2033, 9, 5),   // 2033-09-05 — Labor Day
    (2033, 11, 24), // 2033-11-24 — Thanksgiving Day
    (2033, 12, 26), // 2033-12-26 — Christmas Day (observed; Dec 25 was Sun)
    // ─── 2034 ────────────────────────────────────────────────────────────
    (2034, 1, 2),   // 2034-01-02 — New Year's Day (observed; Jan 1 was Sun)
    (2034, 1, 16),  // 2034-01-16 — Martin Luther King Jr. Day
    (2034, 2, 20),  // 2034-02-20 — Washington's Birthday
    (2034, 4, 7),   // 2034-04-07 — Good Friday
    (2034, 5, 29),  // 2034-05-29 — Memorial Day
    (2034, 6, 19),  // 2034-06-19 — Juneteenth
    (2034, 7, 4),   // 2034-07-04 — Independence Day
    (2034, 9, 4),   // 2034-09-04 — Labor Day
    (2034, 11, 23), // 2034-11-23 — Thanksgiving Day
    (2034, 12, 25), // 2034-12-25 — Christmas Day
    // ─── 2035 ────────────────────────────────────────────────────────────
    (2035, 1, 1),   // 2035-01-01 — New Year's Day
    (2035, 1, 15),  // 2035-01-15 — Martin Luther King Jr. Day
    (2035, 2, 19),  // 2035-02-19 — Washington's Birthday
    (2035, 3, 23),  // 2035-03-23 — Good Friday
    (2035, 5, 28),  // 2035-05-28 — Memorial Day
    (2035, 6, 19),  // 2035-06-19 — Juneteenth
    (2035, 7, 4),   // 2035-07-04 — Independence Day
    (2035, 9, 3),   // 2035-09-03 — Labor Day
    (2035, 11, 22), // 2035-11-22 — Thanksgiving Day
    (2035, 12, 25), // 2035-12-25 — Christmas Day
    // ─── 2036 ────────────────────────────────────────────────────────────
    (2036, 1, 1),   // 2036-01-01 — New Year's Day
    (2036, 1, 21),  // 2036-01-21 — Martin Luther King Jr. Day
    (2036, 2, 18),  // 2036-02-18 — Washington's Birthday
    (2036, 4, 11),  // 2036-04-11 — Good Friday
    (2036, 5, 26),  // 2036-05-26 — Memorial Day
    (2036, 6, 19),  // 2036-06-19 — Juneteenth
    (2036, 7, 4),   // 2036-07-04 — Independence Day
    (2036, 9, 1),   // 2036-09-01 — Labor Day
    (2036, 11, 27), // 2036-11-27 — Thanksgiving Day
    (2036, 12, 25), // 2036-12-25 — Christmas Day
    // ─── 2037 ────────────────────────────────────────────────────────────
    (2037, 1, 1),   // 2037-01-01 — New Year's Day
    (2037, 1, 19),  // 2037-01-19 — Martin Luther King Jr. Day
    (2037, 2, 16),  // 2037-02-16 — Washington's Birthday
    (2037, 4, 3),   // 2037-04-03 — Good Friday
    (2037, 5, 25),  // 2037-05-25 — Memorial Day
    (2037, 6, 19),  // 2037-06-19 — Juneteenth
    (2037, 7, 3),   // 2037-07-03 — Independence Day (observed; Jul 4 was Sat)
    (2037, 9, 7),   // 2037-09-07 — Labor Day
    (2037, 11, 26), // 2037-11-26 — Thanksgiving Day
    (2037, 12, 25), // 2037-12-25 — Christmas Day
    // ─── 2038 ────────────────────────────────────────────────────────────
    (2038, 1, 1),   // 2038-01-01 — New Year's Day
    (2038, 1, 18),  // 2038-01-18 — Martin Luther King Jr. Day
    (2038, 2, 15),  // 2038-02-15 — Washington's Birthday
    (2038, 4, 23),  // 2038-04-23 — Good Friday
    (2038, 5, 31),  // 2038-05-31 — Memorial Day
    (2038, 6, 18),  // 2038-06-18 — Juneteenth (observed; Jun 19 was Sat)
    (2038, 7, 5),   // 2038-07-05 — Independence Day (observed; Jul 4 was Sun)
    (2038, 9, 6),   // 2038-09-06 — Labor Day
    (2038, 11, 25), // 2038-11-25 — Thanksgiving Day
    (2038, 12, 24), // 2038-12-24 — Christmas Day (observed; Dec 25 was Sat)
    (2038, 12, 31), // 2038-12-31 — New Year's Day 2039 (observed; Jan 1 was Sat)
    // ─── 2039 ────────────────────────────────────────────────────────────
    (2039, 1, 17),  // 2039-01-17 — Martin Luther King Jr. Day
    (2039, 2, 21),  // 2039-02-21 — Washington's Birthday
    (2039, 4, 8),   // 2039-04-08 — Good Friday
    (2039, 5, 30),  // 2039-05-30 — Memorial Day
    (2039, 6, 20),  // 2039-06-20 — Juneteenth (observed; Jun 19 was Sun)
    (2039, 7, 4),   // 2039-07-04 — Independence Day
    (2039, 9, 5),   // 2039-09-05 — Labor Day
    (2039, 11, 24), // 2039-11-24 — Thanksgiving Day
    (2039, 12, 26), // 2039-12-26 — Christmas Day (observed; Dec 25 was Sun)
    // ─── 2040 ────────────────────────────────────────────────────────────
    (2040, 1, 2),   // 2040-01-02 — New Year's Day (observed; Jan 1 was Sun)
    (2040, 1, 16),  // 2040-01-16 — Martin Luther King Jr. Day
    (2040, 2, 20),  // 2040-02-20 — Washington's Birthday
    (2040, 3, 30),  // 2040-03-30 — Good Friday
    (2040, 5, 28),  // 2040-05-28 — Memorial Day
    (2040, 6, 19),  // 2040-06-19 — Juneteenth
    (2040, 7, 4),   // 2040-07-04 — Independence Day
    (2040, 9, 3),   // 2040-09-03 — Labor Day
    (2040, 11, 22), // 2040-11-22 — Thanksgiving Day
    (2040, 12, 25), // 2040-12-25 — Christmas Day
];

// ─── Lookup ──────────────────────────────────────────────────────────────────

/// Returns `true` if `date` is a NYSE / Federal Reserve holiday under the
/// embedded snapshot.
///
/// The decision is a binary search over [`HOLIDAYS`]. Dates outside the
/// snapshot's [`COVERAGE`] window return `false` — the snapshot has nothing
/// to say about them, and the caller is expected to treat such a date as a
/// regular business day (or to extend the snapshot itself). Discretionary
/// closures published in NYSE notices are not modelled; see the module
/// docstring.
///
/// # Examples
///
/// ```
/// use regit_daycount::Date;
/// use regit_daycount::calendar::united_states::is_holiday;
///
/// // Independence Day 2024 is a NYSE holiday.
/// assert!( is_holiday(Date::ymd(2024, 7, 4).unwrap()));
///
/// // The Friday immediately after is a regular trading day.
/// assert!(!is_holiday(Date::ymd(2024, 7, 5).unwrap()));
/// ```
#[must_use]
pub fn is_holiday(date: Date) -> bool {
    let (lo, hi) = COVERAGE;
    if date.year() < lo || date.year() > hi {
        return false;
    }
    let key = (date.year(), date.month(), date.day());
    HOLIDAYS.binary_search(&key).is_ok()
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Convenience constructor for tests; panics on invalid input, which is
    /// fine inside `#[cfg(test)]`.
    fn d(year: i32, month: u8, day: u8) -> Date {
        Date::ymd(year, month, day).expect("valid test date")
    }

    // ─── 2024 — every holiday in a recent fully-known year ───────────────

    #[test]
    fn known_2024_holidays_are_recognised() {
        let cases = [
            (2024, 1, 1, "New Year's Day"),
            (2024, 1, 15, "Martin Luther King Jr. Day"),
            (2024, 2, 19, "Washington's Birthday"),
            (2024, 3, 29, "Good Friday"),
            (2024, 5, 27, "Memorial Day"),
            (2024, 6, 19, "Juneteenth"),
            (2024, 7, 4, "Independence Day"),
            (2024, 9, 2, "Labor Day"),
            (2024, 11, 28, "Thanksgiving Day"),
            (2024, 12, 25, "Christmas Day"),
        ];
        for (y, m, dd, name) in cases {
            assert!(is_holiday(d(y, m, dd)), "{y}-{m:02}-{dd:02} {name}");
        }
    }

    // ─── New Year's Day — weekend-shift observance ───────────────────────

    #[test]
    fn new_year_2026_thursday_is_unshifted() {
        // 2026-01-01 is a Thursday — observed on the day itself.
        assert!(is_holiday(d(2026, 1, 1)));
    }

    #[test]
    fn new_year_2027_friday_is_unshifted() {
        // 2027-01-01 is a Friday — observed on the day itself.
        assert!(is_holiday(d(2027, 1, 1)));
    }

    #[test]
    fn new_year_2028_saturday_shifts_to_prior_friday() {
        // 2028-01-01 is a Saturday — observed Fri 2027-12-31.
        assert!(is_holiday(d(2027, 12, 31)));
        assert!(!is_holiday(d(2028, 1, 1)));
    }

    #[test]
    fn new_year_2022_saturday_shifts_to_prior_friday() {
        // 2022-01-01 was a Saturday — observed Fri 2021-12-31.
        assert!(is_holiday(d(2021, 12, 31)));
        assert!(!is_holiday(d(2022, 1, 1)));
    }

    #[test]
    fn new_year_2023_sunday_shifts_to_following_monday() {
        // 2023-01-01 was a Sunday — observed Mon 2023-01-02.
        assert!(is_holiday(d(2023, 1, 2)));
        assert!(!is_holiday(d(2023, 1, 1)));
    }

    // ─── Independence Day — weekend-shift observance ─────────────────────

    #[test]
    fn independence_day_2020_saturday_shifts_to_prior_friday() {
        // 2020-07-04 was a Saturday — observed Fri 2020-07-03.
        assert!(is_holiday(d(2020, 7, 3)));
        assert!(!is_holiday(d(2020, 7, 4)));
    }

    #[test]
    fn independence_day_2026_saturday_shifts_to_prior_friday() {
        // 2026-07-04 is a Saturday — observed Fri 2026-07-03.
        assert!(is_holiday(d(2026, 7, 3)));
        assert!(!is_holiday(d(2026, 7, 4)));
    }

    #[test]
    fn independence_day_2021_sunday_shifts_to_following_monday() {
        // 2021-07-04 was a Sunday — observed Mon 2021-07-05.
        assert!(is_holiday(d(2021, 7, 5)));
        assert!(!is_holiday(d(2021, 7, 4)));
    }

    // ─── Juneteenth — adopted as a NYSE holiday in 2022 ──────────────────

    #[test]
    fn juneteenth_not_observed_before_2022() {
        // Federal holiday took effect 2021-06-17; NYSE began observing in
        // 2022. The snapshot therefore omits 2020-06-19 and 2021-06-19.
        assert!(!is_holiday(d(2020, 6, 19)));
        assert!(!is_holiday(d(2021, 6, 19)));
    }

    #[test]
    fn juneteenth_observed_from_2022() {
        // 2022-06-19 fell on Sunday — observed Mon 2022-06-20.
        assert!(is_holiday(d(2022, 6, 20)));
        // 2023-06-19 (Mon) and 2024-06-19 (Wed) are observed on the day.
        assert!(is_holiday(d(2023, 6, 19)));
        assert!(is_holiday(d(2024, 6, 19)));
    }

    // ─── Out-of-coverage returns false ───────────────────────────────────

    #[test]
    fn out_of_coverage_returns_false_below() {
        // 2019-12-25 — a Christmas Day before the snapshot starts.
        assert!(!is_holiday(d(2019, 12, 25)));
    }

    #[test]
    fn out_of_coverage_returns_false_above() {
        // 2041-01-01 — past the snapshot's last covered year.
        assert!(!is_holiday(d(2041, 1, 1)));
    }

    // ─── Non-holiday returns false ───────────────────────────────────────

    #[test]
    fn ordinary_business_day_is_not_a_holiday() {
        // 2026-03-15 (Sun) and 2026-03-16 (Mon) are not holidays.
        assert!(!is_holiday(d(2026, 3, 15)));
        assert!(!is_holiday(d(2026, 3, 16)));
    }

    // ─── Static-table invariants ─────────────────────────────────────────

    #[test]
    fn holidays_table_is_sorted_ascending() {
        // Binary search in `is_holiday` requires ascending order.
        for window in HOLIDAYS.windows(2) {
            assert!(window[0] < window[1], "out of order at {window:?}");
        }
    }

    #[test]
    fn holidays_table_has_no_duplicates() {
        for window in HOLIDAYS.windows(2) {
            assert_ne!(window[0], window[1], "duplicate row at {window:?}");
        }
    }

    #[test]
    fn holidays_table_entry_count() {
        // 10 holidays × 21 years − 2 (Juneteenth not observed in 2020, 2021)
        // = 208 observed dates.
        assert_eq!(HOLIDAYS.len(), 208);
    }

    #[test]
    fn snapshot_date_and_coverage_constants() {
        assert_eq!(SNAPSHOT_DATE, "2026-05-23");
        assert_eq!(COVERAGE, (2020, 2040));
    }

    // ─── Rule-derivation cross-check ─────────────────────────────────────
    //
    // The embedded table is hand-listed for auditor friendliness; this test
    // is the in-crate cross-check that every row matches its calendar rule.
    // It derives the expected holiday set from the rules cited in the
    // module docstring (Meeus Easter for Good Friday, `nth_weekday_of_month`
    // for the Monday/Thursday rules, the standard NYSE weekend-shift
    // convention for fixed-date holidays) and compares it row-for-row to
    // [`HOLIDAYS`]. If a row drifts — typo, mis-edited year, missing
    // Juneteenth, an out-of-order Good Friday — this test fails and prints
    // the offending pair, so the snapshot cannot silently diverge.

    #[test]
    fn every_holiday_row_matches_its_calendar_rule() {
        use crate::date::Weekday;

        // Standard NYSE weekend-shift: Sat → Fri preceding, Sun → Mon
        // following, weekday → unchanged.
        fn shift_weekend(d: Date) -> Date {
            match d.day_of_week() {
                Weekday::Sat => d.add_days(-1),
                Weekday::Sun => d.add_days(1),
                _ => d,
            }
        }

        // Last Monday of May. The 5th Monday exists in some years; when it
        // does not, the rule degrades to the 4th. `nth_weekday_of_month`
        // signals non-existence via `OutOfRange`.
        fn last_monday_of_may(year: i32) -> Date {
            Date::nth_weekday_of_month(year, 5, 5, Weekday::Mon).unwrap_or_else(|_| {
                Date::nth_weekday_of_month(year, 5, 4, Weekday::Mon)
                    .expect("4th Monday of May exists in every Gregorian year")
            })
        }

        // Fixed-capacity scratch buffer; the snapshot has 208 rows, so 256
        // is a safe upper bound and keeps the test allocation-free.
        let mut expected: [(i32, u8, u8); 256] = [(0, 0, 0); 256];
        let mut n: usize = 0;

        let (lo, hi) = COVERAGE;
        for y in lo..=hi {
            // New Year's Day. Listed under year `y` only when the shifted
            // date is still in `y`; when Jan 1 falls on a Saturday the
            // observed date is 31 Dec of `y - 1` and is added by the
            // `nyd_next` branch in that earlier iteration.
            let nyd = shift_weekend(Date::ymd(y, 1, 1).expect("Jan 1 within coverage"));
            if nyd.year() == y {
                expected[n] = (nyd.year(), nyd.month(), nyd.day());
                n += 1;
            }

            let mlk = Date::nth_weekday_of_month(y, 1, 3, Weekday::Mon)
                .expect("3rd Mon Jan exists in every year");
            expected[n] = (mlk.year(), mlk.month(), mlk.day());
            n += 1;

            let wbd = Date::nth_weekday_of_month(y, 2, 3, Weekday::Mon)
                .expect("3rd Mon Feb exists in every year");
            expected[n] = (wbd.year(), wbd.month(), wbd.day());
            n += 1;

            let gf = Date::easter_sunday(y).add_days(-2);
            expected[n] = (gf.year(), gf.month(), gf.day());
            n += 1;

            let mem = last_monday_of_may(y);
            expected[n] = (mem.year(), mem.month(), mem.day());
            n += 1;

            if y >= 2022 {
                let jt = shift_weekend(Date::ymd(y, 6, 19).expect("Jun 19 within coverage"));
                expected[n] = (jt.year(), jt.month(), jt.day());
                n += 1;
            }

            let ind = shift_weekend(Date::ymd(y, 7, 4).expect("Jul 4 within coverage"));
            expected[n] = (ind.year(), ind.month(), ind.day());
            n += 1;

            let lbr = Date::nth_weekday_of_month(y, 9, 1, Weekday::Mon)
                .expect("1st Mon Sep exists in every year");
            expected[n] = (lbr.year(), lbr.month(), lbr.day());
            n += 1;

            let tg = Date::nth_weekday_of_month(y, 11, 4, Weekday::Thu)
                .expect("4th Thu Nov exists in every year");
            expected[n] = (tg.year(), tg.month(), tg.day());
            n += 1;

            let xm = shift_weekend(Date::ymd(y, 12, 25).expect("Dec 25 within coverage"));
            expected[n] = (xm.year(), xm.month(), xm.day());
            n += 1;

            // The observed New Year's Day of `y + 1` that shifts back into
            // year `y` (only when Jan 1 of `y + 1` falls on a Saturday).
            let nyd_next =
                shift_weekend(Date::ymd(y + 1, 1, 1).expect("Jan 1 of y+1 within coverage+1"));
            if nyd_next.year() == y {
                expected[n] = (nyd_next.year(), nyd_next.month(), nyd_next.day());
                n += 1;
            }
        }

        assert_eq!(
            n,
            HOLIDAYS.len(),
            "row count drift: rule derives {n} rows, table has {}",
            HOLIDAYS.len()
        );
        for (i, (got, want)) in HOLIDAYS.iter().zip(expected.iter().take(n)).enumerate() {
            assert_eq!(
                got, want,
                "row {i}: table has {got:?}, rule derives {want:?}",
            );
        }
    }
}
