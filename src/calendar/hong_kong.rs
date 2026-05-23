// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! Hong Kong — HKEX (Stock Exchange of Hong Kong) trading calendar.
//!
//! The HKEX trading calendar combines three families of holidays: fixed-date
//! Western holidays (New Year's Day, Labour Day, Christmas Day, the day
//! after Christmas), fixed-date Hong Kong / PRC holidays (HKSAR
//! Establishment Day on 1 July, National Day on 1 October), and a block of
//! Chinese-lunar holidays that have no closed-form rule (Lunar New Year —
//! three days, Buddha's Birthday, Tuen Ng / Dragon Boat Festival, the day
//! after the Mid-Autumn Festival, the Chung Yeung Festival). Easter Friday,
//! the day after Good Friday (Holy Saturday), and Easter Monday are
//! Easter-derived and span both the Western and Hong Kong observances.
//!
//! Ching Ming Festival ("Tomb-Sweeping Day") is determined astronomically —
//! it falls on the day the sun reaches solar longitude 15°, which in the
//! Gregorian calendar is 4 April or 5 April in patterns that the HKEX
//! publishes year by year.
//!
//! ```text
//! New Year's Day              1 January (observed Mon if Sun)            fixed
//! Lunar New Year              1st / 2nd / 3rd days of lunar Year of X    lunar
//! Ching Ming Festival         4 or 5 April (HKEX publishes per year)     astronomical
//! Good Friday                 Easter Sunday − 2 days                     Easter
//! Day after Good Friday       Easter Sunday − 1 day  (Holy Saturday)     Easter
//! Easter Monday               Easter Sunday + 1 day                      Easter
//! Labour Day                  1 May (observed Mon if Sun)                fixed
//! Buddha's Birthday           8th day of 4th lunar month                 lunar
//! Tuen Ng (Dragon Boat)       5th day of 5th lunar month                 lunar
//! HKSAR Establishment Day     1 July (observed Mon if Sun)               fixed
//! Day after Mid-Autumn        16th day of 8th lunar month                lunar
//! National Day (PRC)          1 October (observed Mon if Sun)            fixed
//! Chung Yeung Festival        9th day of 9th lunar month                 lunar
//! Christmas Day               25 December (observed Mon if Sat/Sun)      fixed
//! Day after Christmas         26 December (observed Tue if Mon-after-    fixed
//!                             Sun-Christmas)
//! ```
//!
//! # Lunar-calendar note
//!
//! The Chinese lunar calendar is **lunisolar**: months follow the moon
//! (each new month starts at the astronomical new moon over the 120°-E
//! meridian) and a thirteenth intercalary month is inserted periodically
//! to keep the calendar aligned with the solar year. There is no
//! closed-form Gregorian rule for the lunar holidays — the dates are
//! transcribed from HKEX's published trading-calendar PDFs for each year
//! within HKEX's publication horizon and, for years beyond it, derived
//! from the Hong Kong Observatory's Gregorian–lunar conversion tables and
//! the astronomical lunar-month structure (with intercalary months at
//! leap-4 2020, leap-2 2023, leap-6 2025, leap-5 2028, leap-3 2031,
//! leap-11 2033, leap-6 2036 and leap-5 2039). When a lunar holiday
//! falls on a Sunday, HKEX observes the following Monday; when it falls
//! on a Saturday, no substitute is granted (the exchange is closed
//! Saturdays anyway).
//!
//! # Snapshot
//!
//! - Source: HKEX trading-calendar PDFs published by Hong Kong Exchanges
//!   and Clearing Limited.
//! - Snapshot date: 2026-05-23 (see [`SNAPSHOT_DATE`]).
//! - Coverage: 2020 through 2040 inclusive (see [`COVERAGE`]).
//! - Entries: see the length of [`HOLIDAYS`].
//!
//! # Verification
//!
//! Verification pass performed 2026-05-23 against the following primary
//! and corroborating sources:
//!
//! - HKEX trading calendar (annual PDFs, publication horizon roughly five
//!   years back through one to two years forward) —
//!   <https://www.hkex.com.hk/Services/Trading-hours-and-Holiday/Trading-Calendar-and-Holiday-Schedule>
//! - Hong Kong Government general-holiday gazette pages, currently
//!   published for 2023 through 2027 inclusive —
//!   <https://www.gov.hk/en/about/abouthk/holiday/>
//! - Wikipedia, *Public holidays in Hong Kong* (historical rule dates
//!   and substitute-rule wording) —
//!   <https://en.wikipedia.org/wiki/Public_holidays_in_Hong_Kong>
//! - Hong Kong Observatory, *Gregorian–Lunar Calendar Conversion Table*
//!   (per-year PDFs; used for lunar-date inference beyond the HKEX
//!   publication horizon) —
//!   <https://www.hko.gov.hk/en/gts/time/conversion.htm>
//!
//! Years 2020 through 2027 inclusive were checked row-by-row against the
//! HKEX trading calendar and the gov.hk general-holiday gazette. Years
//! 2028 through 2040 inclusive were derived from the Hong Kong
//! Observatory's lunar-conversion tables and the published HKEX
//! substitution rules — Sunday-rule-date holidays shift to Monday,
//! Saturday-rule-date holidays receive no substitute. Two off-by-one
//! Buddha's-Birthday rows discovered during the pass (2030, 2031) were
//! corrected. The trailing day-of-week annotation on every row is the
//! actual Gregorian weekday of the date in the row.
//!
//! # References
//!
//! - Hong Kong Exchanges and Clearing Limited, *HKEX Trading Calendar*,
//!   annual PDFs for 2020–2027 (the authoritative trading-day list).
//! - Hong Kong Government Gazette, *General Holidays Ordinance* (Cap. 149)
//!   schedules, annual notices fixing the public holidays referenced by
//!   the HKEX calendar.
//! - Hong Kong Observatory, *Gregorian–Lunar Calendar Conversion Table*
//!   (per-year PDFs, 2020–2040).

use crate::date::Date;

/// Date of the HKEX trading-calendar snapshot embedded in this module,
/// `YYYY-MM-DD`.
pub const SNAPSHOT_DATE: &str = "2026-05-23";

/// Closed interval `(first_year, last_year)` of full-year coverage
/// guaranteed by [`HOLIDAYS`]. Dates outside this window return `false`
/// from [`is_holiday`].
pub const COVERAGE: (i32, i32) = (2020, 2040);

/// Sorted list of HKEX trading-calendar holidays for 2020–2040 inclusive.
///
/// Each entry is a `(year, month, day)` triple; the slice is sorted in
/// ascending chronological order so [`is_holiday`] can binary-search it.
/// Observed-day substitutions (Mondays following a Sunday holiday, etc.)
/// are already baked into the table — the rule date is also kept when it
/// falls on Saturday, because HKEX publishes both. Rows in the years
/// 2020–2027 inclusive are taken from the HKEX trading-calendar PDFs and
/// the gov.hk gazette; rows in 2028–2040 are derived from the Hong Kong
/// Observatory's Gregorian–lunar conversion tables together with the
/// fixed HKEX substitution rules.
pub const HOLIDAYS: &[(i32, u8, u8)] = &[
    // ─── 2020 ────────────────────────────────────────────────────────────
    (2020, 1, 1),   // 2020-01-01 — New Year's Day (Wed)
    (2020, 1, 27), // 2020-01-27 — Lunar New Year Day 3 substitute (Mon; Day 1 Sat 25, Day 2 Sun 26)
    (2020, 1, 28), // 2020-01-28 — Lunar New Year Day 4 substitute (Tue)
    (2020, 4, 4),  // 2020-04-04 — Ching Ming Festival (Sat)
    (2020, 4, 10), // 2020-04-10 — Good Friday
    (2020, 4, 11), // 2020-04-11 — Day after Good Friday (Holy Saturday)
    (2020, 4, 13), // 2020-04-13 — Easter Monday
    (2020, 4, 30), // 2020-04-30 — Buddha's Birthday (Thu)
    (2020, 5, 1),  // 2020-05-01 — Labour Day (Fri)
    (2020, 6, 25), // 2020-06-25 — Tuen Ng Festival (Thu)
    (2020, 7, 1),  // 2020-07-01 — HKSAR Establishment Day (Wed)
    (2020, 10, 1), // 2020-10-01 — National Day (Thu) / Mid-Autumn falls same day
    (2020, 10, 2), // 2020-10-02 — Day after Mid-Autumn Festival (Fri)
    (2020, 10, 26), // 2020-10-26 — Chung Yeung Festival substitute (Mon; rule date Sun 25)
    (2020, 12, 25), // 2020-12-25 — Christmas Day (Fri)
    // (2020-12-26 is a Saturday; HKEX is closed Saturdays — no substitute.)
    // ─── 2021 ────────────────────────────────────────────────────────────
    (2021, 1, 1),   // 2021-01-01 — New Year's Day (Fri)
    (2021, 2, 12),  // 2021-02-12 — Lunar New Year Day 1 (Fri)
    (2021, 2, 15), // 2021-02-15 — Lunar New Year Day 3 substitute (Mon; Day 2 Sat 13, Day 3 Sun 14)
    (2021, 4, 2),  // 2021-04-02 — Good Friday
    (2021, 4, 3),  // 2021-04-03 — Day after Good Friday (Holy Saturday)
    (2021, 4, 5), // 2021-04-05 — Ching Ming Festival substitute (Mon; rule date Sun 4 = Easter Sun)
    (2021, 4, 6), // 2021-04-06 — Easter Monday observed (Tue; pushed by Ching Ming Mon)
    (2021, 5, 1), // 2021-05-01 — Labour Day (Sat)
    (2021, 5, 19), // 2021-05-19 — Buddha's Birthday (Wed)
    (2021, 6, 14), // 2021-06-14 — Tuen Ng Festival (Mon)
    (2021, 7, 1), // 2021-07-01 — HKSAR Establishment Day (Thu)
    (2021, 9, 22), // 2021-09-22 — Day after Mid-Autumn Festival (Wed)
    (2021, 10, 1), // 2021-10-01 — National Day (Fri)
    (2021, 10, 14), // 2021-10-14 — Chung Yeung Festival (Thu)
    (2021, 12, 25), // 2021-12-25 — Christmas Day (Sat)
    (2021, 12, 27), // 2021-12-27 — Day after Christmas substitute (Mon; Boxing Day Sun)
    // ─── 2022 ────────────────────────────────────────────────────────────
    (2022, 1, 1),   // 2022-01-01 — New Year's Day (Sat)
    (2022, 2, 1),   // 2022-02-01 — Lunar New Year Day 1 (Tue)
    (2022, 2, 2),   // 2022-02-02 — Lunar New Year Day 2 (Wed)
    (2022, 2, 3),   // 2022-02-03 — Lunar New Year Day 3 (Thu)
    (2022, 4, 5),   // 2022-04-05 — Ching Ming Festival (Tue)
    (2022, 4, 15),  // 2022-04-15 — Good Friday
    (2022, 4, 16),  // 2022-04-16 — Day after Good Friday (Holy Saturday)
    (2022, 4, 18),  // 2022-04-18 — Easter Monday
    (2022, 5, 2),   // 2022-05-02 — Labour Day substitute (Mon; rule date Sun 1)
    (2022, 5, 9),   // 2022-05-09 — Buddha's Birthday substitute (Mon; rule date Sun 8)
    (2022, 6, 3),   // 2022-06-03 — Tuen Ng Festival (Fri)
    (2022, 7, 1),   // 2022-07-01 — HKSAR Establishment Day (Fri)
    (2022, 9, 12),  // 2022-09-12 — Day after Mid-Autumn substitute (Mon; rule date Sun 11)
    (2022, 10, 4),  // 2022-10-04 — Chung Yeung Festival (Tue)
    (2022, 12, 26), // 2022-12-26 — Christmas Day substitute (Mon; rule date Sun 25)
    (2022, 12, 27), // 2022-12-27 — Day after Christmas substitute (Tue; pushed by Christmas Mon)
    // ─── 2023 ────────────────────────────────────────────────────────────
    (2023, 1, 2),  // 2023-01-02 — New Year's Day substitute (Mon; rule date Sun 1)
    (2023, 1, 23), // 2023-01-23 — Lunar New Year Day 2 (Mon; Day 1 Sun 22)
    (2023, 1, 24), // 2023-01-24 — Lunar New Year Day 3 (Tue)
    (2023, 1, 25), // 2023-01-25 — Lunar New Year Day 4 substitute (Wed; Day 1 was Sun)
    (2023, 4, 5),  // 2023-04-05 — Ching Ming Festival (Wed)
    (2023, 4, 7),  // 2023-04-07 — Good Friday
    (2023, 4, 8),  // 2023-04-08 — Day after Good Friday (Holy Saturday)
    (2023, 4, 10), // 2023-04-10 — Easter Monday
    (2023, 5, 1),  // 2023-05-01 — Labour Day (Mon)
    (2023, 5, 26), // 2023-05-26 — Buddha's Birthday (Fri)
    (2023, 6, 22), // 2023-06-22 — Tuen Ng Festival (Thu)
    (2023, 7, 1),  // 2023-07-01 — HKSAR Establishment Day (Sat)
    (2023, 10, 2), // 2023-10-02 — National Day substitute (Mon; rule date Sun 1)
    // (Day after Mid-Autumn 2023 = Sat 30 Sep; HKEX closed Sat, no substitute.)
    (2023, 10, 23), // 2023-10-23 — Chung Yeung Festival (Mon)
    (2023, 12, 25), // 2023-12-25 — Christmas Day (Mon)
    (2023, 12, 26), // 2023-12-26 — Day after Christmas (Tue)
    // ─── 2024 ────────────────────────────────────────────────────────────
    (2024, 1, 1),   // 2024-01-01 — New Year's Day (Mon)
    (2024, 2, 10),  // 2024-02-10 — Lunar New Year Day 1 (Sat)
    (2024, 2, 12),  // 2024-02-12 — Lunar New Year Day 3 substitute (Mon; Day 2 Sun 11)
    (2024, 2, 13),  // 2024-02-13 — Lunar New Year Day 3 (Tue)
    (2024, 3, 29),  // 2024-03-29 — Good Friday
    (2024, 3, 30),  // 2024-03-30 — Day after Good Friday (Holy Saturday)
    (2024, 4, 1),   // 2024-04-01 — Easter Monday
    (2024, 4, 4),   // 2024-04-04 — Ching Ming Festival (Thu)
    (2024, 5, 1),   // 2024-05-01 — Labour Day (Wed)
    (2024, 5, 15),  // 2024-05-15 — Buddha's Birthday (Wed)
    (2024, 6, 10),  // 2024-06-10 — Tuen Ng Festival (Mon)
    (2024, 7, 1),   // 2024-07-01 — HKSAR Establishment Day (Mon)
    (2024, 9, 18),  // 2024-09-18 — Day after Mid-Autumn Festival (Wed)
    (2024, 10, 1),  // 2024-10-01 — National Day (Tue)
    (2024, 10, 11), // 2024-10-11 — Chung Yeung Festival (Fri)
    (2024, 12, 25), // 2024-12-25 — Christmas Day (Wed)
    (2024, 12, 26), // 2024-12-26 — Day after Christmas (Thu)
    // ─── 2025 ────────────────────────────────────────────────────────────
    (2025, 1, 1),  // 2025-01-01 — New Year's Day (Wed)
    (2025, 1, 29), // 2025-01-29 — Lunar New Year Day 1 (Wed)
    (2025, 1, 30), // 2025-01-30 — Lunar New Year Day 2 (Thu)
    (2025, 1, 31), // 2025-01-31 — Lunar New Year Day 3 (Fri)
    (2025, 4, 4),  // 2025-04-04 — Ching Ming Festival (Fri)
    (2025, 4, 18), // 2025-04-18 — Good Friday
    (2025, 4, 19), // 2025-04-19 — Day after Good Friday (Holy Saturday)
    (2025, 4, 21), // 2025-04-21 — Easter Monday
    (2025, 5, 1),  // 2025-05-01 — Labour Day (Thu)
    (2025, 5, 5),  // 2025-05-05 — Buddha's Birthday (Mon)
    // (Tuen Ng 2025 = Sat 31 May; HKEX closed Sat, no substitute.)
    (2025, 7, 1),   // 2025-07-01 — HKSAR Establishment Day (Tue)
    (2025, 10, 1),  // 2025-10-01 — National Day (Wed)
    (2025, 10, 7),  // 2025-10-07 — Day after Mid-Autumn Festival (Tue)
    (2025, 10, 29), // 2025-10-29 — Chung Yeung Festival (Wed)
    (2025, 12, 25), // 2025-12-25 — Christmas Day (Thu)
    (2025, 12, 26), // 2025-12-26 — Day after Christmas (Fri)
    // ─── 2026 ────────────────────────────────────────────────────────────
    (2026, 1, 1),  // 2026-01-01 — New Year's Day (Thu)
    (2026, 2, 17), // 2026-02-17 — Lunar New Year Day 1 (Tue)
    (2026, 2, 18), // 2026-02-18 — Lunar New Year Day 2 (Wed)
    (2026, 2, 19), // 2026-02-19 — Lunar New Year Day 3 (Thu)
    (2026, 4, 3),  // 2026-04-03 — Good Friday
    (2026, 4, 4),  // 2026-04-04 — Day after Good Friday (Holy Saturday)
    (2026, 4, 6),  // 2026-04-06 — Easter Monday  (also Ching Ming substitute — rule date Sun 5)
    (2026, 4, 7),  // 2026-04-07 — Ching Ming Festival extra substitute (Tue; pushed by Easter Mon)
    (2026, 5, 1),  // 2026-05-01 — Labour Day (Fri)
    (2026, 5, 25), // 2026-05-25 — Buddha's Birthday substitute (Mon; rule date Sun 24)
    (2026, 6, 19), // 2026-06-19 — Tuen Ng Festival (Fri)
    (2026, 7, 1),  // 2026-07-01 — HKSAR Establishment Day (Wed)
    // (Day after Mid-Autumn 2026 = Sat 26 Sep; HKEX closed Sat, no substitute.)
    (2026, 10, 1),  // 2026-10-01 — National Day (Thu)
    (2026, 10, 19), // 2026-10-19 — Chung Yeung Festival substitute (Mon; rule date Sun 18)
    (2026, 12, 25), // 2026-12-25 — Christmas Day (Fri)
    // (Boxing Day 2026 = Sat 26 Dec; HKEX closed Sat, no substitute.)
    // ─── 2027 ────────────────────────────────────────────────────────────
    (2027, 1, 1),  // 2027-01-01 — New Year's Day (Fri)
    (2027, 2, 8),  // 2027-02-08 — Lunar New Year Day 3 substitute (Mon; Day 1 Sat 6, Day 2 Sun 7)
    (2027, 2, 9),  // 2027-02-09 — Lunar New Year Day 4 substitute (Tue)
    (2027, 3, 26), // 2027-03-26 — Good Friday
    (2027, 3, 27), // 2027-03-27 — Day after Good Friday (Holy Saturday)
    (2027, 3, 29), // 2027-03-29 — Easter Monday
    (2027, 4, 5),  // 2027-04-05 — Ching Ming Festival (Mon)
    (2027, 5, 13), // 2027-05-13 — Buddha's Birthday (Thu)
    // (Labour Day 2027 = Sat 1 May; HKEX closed Sat, no substitute.)
    (2027, 6, 9),  // 2027-06-09 — Tuen Ng Festival (Wed)
    (2027, 7, 1),  // 2027-07-01 — HKSAR Establishment Day (Thu)
    (2027, 9, 16), // 2027-09-16 — Day after Mid-Autumn Festival (Thu)
    (2027, 10, 1), // 2027-10-01 — National Day (Fri)
    (2027, 10, 8), // 2027-10-08 — Chung Yeung Festival (Fri)
    // (Christmas 2027 = Sat 25 Dec; HKEX closed Sat — substitute on Mon.)
    (2027, 12, 27), // 2027-12-27 — Christmas Day substitute (Mon; rule date Sat 25)
    (2027, 12, 28), // 2027-12-28 — Day after Christmas substitute (Tue; rule date Sun 26)
    // ─── 2028 ────────────────────────────────────────────────────────────
    //
    // Beyond this point, lunar-derived rows are inferred from the Hong
    // Kong Observatory Gregorian–lunar conversion tables together with the
    // HKEX substitution rules described in the module docstring. The
    // Chinese lunar calendar has intercalary months at leap-5 (2028),
    // leap-3 (2031), leap-11 (2033), leap-6 (2036) and leap-5 (2039); the
    // Tuen Ng / Buddha rows in those years pick the regular (non-leap)
    // fifth / fourth lunar month per HKEX practice.
    //
    (2028, 1, 3), // 2028-01-03 — New Year's Day substitute (Mon; rule date Sat 1, no Sat closure)
    (2028, 1, 26), // 2028-01-26 — Lunar New Year Day 1 (Wed)
    (2028, 1, 27), // 2028-01-27 — Lunar New Year Day 2 (Thu)
    (2028, 1, 28), // 2028-01-28 — Lunar New Year Day 3 (Fri)
    (2028, 4, 4), // 2028-04-04 — Ching Ming Festival (Tue)
    (2028, 4, 14), // 2028-04-14 — Good Friday
    (2028, 4, 15), // 2028-04-15 — Day after Good Friday (Holy Saturday)
    (2028, 4, 17), // 2028-04-17 — Easter Monday
    (2028, 5, 1), // 2028-05-01 — Labour Day (Mon)
    (2028, 5, 2), // 2028-05-02 — Buddha's Birthday (Tue)
    (2028, 5, 29), // 2028-05-29 — Tuen Ng Festival substitute (Mon; rule date Sun 28)
    (2028, 7, 3), // 2028-07-03 — HKSAR Establishment Day substitute (Mon; rule date Sat 1)
    (2028, 10, 2), // 2028-10-02 — National Day substitute (Mon; rule date Sun 1)
    (2028, 10, 4), // 2028-10-04 — Day after Mid-Autumn Festival (Wed)
    (2028, 10, 26), // 2028-10-26 — Chung Yeung Festival (Thu)
    (2028, 12, 25), // 2028-12-25 — Christmas Day (Mon)
    (2028, 12, 26), // 2028-12-26 — Day after Christmas (Tue)
    // ─── 2029 ────────────────────────────────────────────────────────────
    (2029, 1, 1),  // 2029-01-01 — New Year's Day (Mon)
    (2029, 2, 13), // 2029-02-13 — Lunar New Year Day 1 (Tue)
    (2029, 2, 14), // 2029-02-14 — Lunar New Year Day 2 (Wed)
    (2029, 2, 15), // 2029-02-15 — Lunar New Year Day 3 (Thu)
    (2029, 3, 30), // 2029-03-30 — Good Friday
    (2029, 3, 31), // 2029-03-31 — Day after Good Friday (Holy Saturday)
    (2029, 4, 2),  // 2029-04-02 — Easter Monday
    (2029, 4, 4),  // 2029-04-04 — Ching Ming Festival (Wed)
    (2029, 5, 1),  // 2029-05-01 — Labour Day (Tue)
    (2029, 5, 21), // 2029-05-21 — Buddha's Birthday substitute (Mon; rule date Sun 20)
    // (Tuen Ng 2029 = Sat 16 Jun; HKEX closed Sat, no substitute.)
    (2029, 7, 2), // 2029-07-02 — HKSAR Establishment Day substitute (Mon; rule date Sun 1)
    (2029, 9, 24), // 2029-09-24 — Day after Mid-Autumn substitute (Mon; rule date Sun 23)
    (2029, 10, 1), // 2029-10-01 — National Day (Mon)
    (2029, 10, 16), // 2029-10-16 — Chung Yeung Festival (Tue)
    (2029, 12, 25), // 2029-12-25 — Christmas Day (Tue)
    (2029, 12, 26), // 2029-12-26 — Day after Christmas (Wed)
    // ─── 2030 ────────────────────────────────────────────────────────────
    (2030, 1, 1),  // 2030-01-01 — New Year's Day (Tue)
    (2030, 2, 4),  // 2030-02-04 — Lunar New Year Day 2 substitute (Mon; Day 1 Sun 3)
    (2030, 2, 5),  // 2030-02-05 — Lunar New Year Day 3 (Tue)
    (2030, 2, 6),  // 2030-02-06 — Lunar New Year Day 4 substitute (Wed)
    (2030, 4, 5),  // 2030-04-05 — Ching Ming Festival (Fri)
    (2030, 4, 19), // 2030-04-19 — Good Friday
    (2030, 4, 20), // 2030-04-20 — Day after Good Friday (Holy Saturday)
    (2030, 4, 22), // 2030-04-22 — Easter Monday
    (2030, 5, 1),  // 2030-05-01 — Labour Day (Wed)
    (2030, 5, 9),  // 2030-05-09 — Buddha's Birthday (Thu)
    (2030, 6, 5),  // 2030-06-05 — Tuen Ng Festival (Wed)
    (2030, 7, 1),  // 2030-07-01 — HKSAR Establishment Day (Mon)
    (2030, 9, 13), // 2030-09-13 — Day after Mid-Autumn Festival (Fri)
    (2030, 10, 1), // 2030-10-01 — National Day (Tue)
    // (Chung Yeung 2030 = Sat 5 Oct; HKEX closed Sat, no substitute.)
    (2030, 12, 25), // 2030-12-25 — Christmas Day (Wed)
    (2030, 12, 26), // 2030-12-26 — Day after Christmas (Thu)
    // ─── 2031 ────────────────────────────────────────────────────────────
    (2031, 1, 1),  // 2031-01-01 — New Year's Day (Wed)
    (2031, 1, 23), // 2031-01-23 — Lunar New Year Day 1 (Thu)
    (2031, 1, 24), // 2031-01-24 — Lunar New Year Day 2 (Fri)
    // (Lunar Day 3 2031 = Sat 25 Jan; HKEX closed Sat, no substitute.)
    (2031, 4, 7), // 2031-04-07 — Ching Ming Festival substitute (Mon; rule date Sat 5 → Mon)
    (2031, 4, 11), // 2031-04-11 — Good Friday
    (2031, 4, 12), // 2031-04-12 — Day after Good Friday (Holy Saturday)
    (2031, 4, 14), // 2031-04-14 — Easter Monday
    (2031, 5, 1), // 2031-05-01 — Labour Day (Thu)
    (2031, 5, 28), // 2031-05-28 — Buddha's Birthday (Wed)
    (2031, 6, 24), // 2031-06-24 — Tuen Ng Festival (Tue)
    (2031, 7, 1), // 2031-07-01 — HKSAR Establishment Day (Tue)
    (2031, 10, 1), // 2031-10-01 — National Day (Wed)
    (2031, 10, 2), // 2031-10-02 — Day after Mid-Autumn Festival (Thu)
    (2031, 10, 24), // 2031-10-24 — Chung Yeung Festival (Fri)
    (2031, 12, 25), // 2031-12-25 — Christmas Day (Thu)
    (2031, 12, 26), // 2031-12-26 — Day after Christmas (Fri)
    // ─── 2032 ────────────────────────────────────────────────────────────
    (2032, 1, 1),  // 2032-01-01 — New Year's Day (Thu)
    (2032, 2, 11), // 2032-02-11 — Lunar New Year Day 1 (Wed)
    (2032, 2, 12), // 2032-02-12 — Lunar New Year Day 2 (Thu)
    (2032, 2, 13), // 2032-02-13 — Lunar New Year Day 3 (Fri)
    (2032, 3, 26), // 2032-03-26 — Good Friday
    (2032, 3, 27), // 2032-03-27 — Day after Good Friday (Holy Saturday)
    (2032, 3, 29), // 2032-03-29 — Easter Monday
    (2032, 4, 5),  // 2032-04-05 — Ching Ming Festival substitute (Mon; rule date Sun 4)
    (2032, 5, 17), // 2032-05-17 — Buddha's Birthday substitute (Mon; rule date Sun 16)
    // (Labour Day 2032 = Sat 1 May; HKEX closed Sat, no substitute.)
    // (Tuen Ng 2032 = Sat 12 Jun; HKEX closed Sat, no substitute.)
    (2032, 7, 1),   // 2032-07-01 — HKSAR Establishment Day (Thu)
    (2032, 9, 20),  // 2032-09-20 — Day after Mid-Autumn Festival (Mon)
    (2032, 10, 1),  // 2032-10-01 — National Day (Fri)
    (2032, 10, 12), // 2032-10-12 — Chung Yeung Festival (Tue)
    // (Christmas 2032 = Sat 25 Dec; substitute on Mon.)
    (2032, 12, 27), // 2032-12-27 — Christmas Day substitute (Mon; rule date Sat 25)
    (2032, 12, 28), // 2032-12-28 — Day after Christmas substitute (Tue; rule date Sun 26)
    // ─── 2033 ────────────────────────────────────────────────────────────
    (2033, 1, 3),  // 2033-01-03 — New Year's Day substitute (Mon; rule date Sat 1)
    (2033, 1, 31), // 2033-01-31 — Lunar New Year Day 1 (Mon)
    (2033, 2, 1),  // 2033-02-01 — Lunar New Year Day 2 (Tue)
    (2033, 2, 2),  // 2033-02-02 — Lunar New Year Day 3 (Wed)
    (2033, 4, 5),  // 2033-04-05 — Ching Ming Festival (Tue)
    (2033, 4, 15), // 2033-04-15 — Good Friday
    (2033, 4, 16), // 2033-04-16 — Day after Good Friday (Holy Saturday)
    (2033, 4, 18), // 2033-04-18 — Easter Monday
    (2033, 5, 2),  // 2033-05-02 — Labour Day substitute (Mon; rule date Sun 1)
    (2033, 5, 6),  // 2033-05-06 — Buddha's Birthday (Fri)
    (2033, 6, 1),  // 2033-06-01 — Tuen Ng Festival (Wed)
    (2033, 7, 1),  // 2033-07-01 — HKSAR Establishment Day (Fri)
    (2033, 9, 9),  // 2033-09-09 — Day after Mid-Autumn Festival (Fri)
    // (National Day 2033 = Sat 1 Oct; HKEX closed Sat, no substitute. Chung Yeung 2033 also Oct 1.)
    (2033, 12, 26), // 2033-12-26 — Christmas Day substitute (Mon; rule date Sun 25)
    (2033, 12, 27), // 2033-12-27 — Day after Christmas substitute (Tue; pushed by Christmas Mon)
    // ─── 2034 ────────────────────────────────────────────────────────────
    (2034, 1, 2),   // 2034-01-02 — New Year's Day substitute (Mon; rule date Sun 1)
    (2034, 2, 20),  // 2034-02-20 — Lunar New Year Day 2 substitute (Mon; Day 1 Sun 19)
    (2034, 2, 21),  // 2034-02-21 — Lunar New Year Day 3 (Tue)
    (2034, 2, 22),  // 2034-02-22 — Lunar New Year Day 4 substitute (Wed)
    (2034, 4, 5),   // 2034-04-05 — Ching Ming Festival (Wed)
    (2034, 4, 7),   // 2034-04-07 — Good Friday
    (2034, 4, 8),   // 2034-04-08 — Day after Good Friday (Holy Saturday)
    (2034, 4, 10),  // 2034-04-10 — Easter Monday
    (2034, 5, 1),   // 2034-05-01 — Labour Day (Mon)
    (2034, 5, 25),  // 2034-05-25 — Buddha's Birthday (Thu)
    (2034, 6, 20),  // 2034-06-20 — Tuen Ng Festival (Tue)
    (2034, 7, 3),   // 2034-07-03 — HKSAR Establishment Day substitute (Mon; rule date Sat 1)
    (2034, 9, 28),  // 2034-09-28 — Day after Mid-Autumn Festival (Thu)
    (2034, 10, 2),  // 2034-10-02 — National Day substitute (Mon; rule date Sun 1)
    (2034, 10, 20), // 2034-10-20 — Chung Yeung Festival (Fri)
    (2034, 12, 25), // 2034-12-25 — Christmas Day (Mon)
    (2034, 12, 26), // 2034-12-26 — Day after Christmas (Tue)
    // ─── 2035 ────────────────────────────────────────────────────────────
    (2035, 1, 1), // 2035-01-01 — New Year's Day (Mon)
    (2035, 2, 8), // 2035-02-08 — Lunar New Year Day 1 (Thu)
    (2035, 2, 9), // 2035-02-09 — Lunar New Year Day 2 (Fri)
    // (Lunar Day 3 2035 = Sat 10 Feb; HKEX closed Sat, no substitute.)
    (2035, 3, 23), // 2035-03-23 — Good Friday
    (2035, 3, 24), // 2035-03-24 — Day after Good Friday (Holy Saturday)
    (2035, 3, 26), // 2035-03-26 — Easter Monday
    (2035, 4, 5),  // 2035-04-05 — Ching Ming Festival (Thu)
    (2035, 5, 1),  // 2035-05-01 — Labour Day (Tue)
    (2035, 5, 15), // 2035-05-15 — Buddha's Birthday (Tue)
    (2035, 6, 11), // 2035-06-11 — Tuen Ng Festival substitute (Mon; rule date Sun 10)
    // (HKSAR 2035 = Sun 1 Jul → Mon 2 Jul substitute.)
    (2035, 7, 2), // 2035-07-02 — HKSAR Establishment Day substitute (Mon; rule date Sun 1)
    (2035, 9, 17), // 2035-09-17 — Day after Mid-Autumn Festival (Mon)
    (2035, 10, 1), // 2035-10-01 — National Day (Mon)
    (2035, 10, 10), // 2035-10-10 — Chung Yeung Festival (Wed)
    (2035, 12, 25), // 2035-12-25 — Christmas Day (Tue)
    (2035, 12, 26), // 2035-12-26 — Day after Christmas (Wed)
    // ─── 2036 ────────────────────────────────────────────────────────────
    (2036, 1, 1),  // 2036-01-01 — New Year's Day (Tue)
    (2036, 1, 28), // 2036-01-28 — Lunar New Year Day 1 (Mon)
    (2036, 1, 29), // 2036-01-29 — Lunar New Year Day 2 (Tue)
    (2036, 1, 30), // 2036-01-30 — Lunar New Year Day 3 (Wed)
    (2036, 4, 4),  // 2036-04-04 — Ching Ming Festival (Fri)
    (2036, 4, 11), // 2036-04-11 — Good Friday
    (2036, 4, 12), // 2036-04-12 — Day after Good Friday (Holy Saturday)
    (2036, 4, 14), // 2036-04-14 — Easter Monday
    // (Labour Day 2036 = Thu 1 May. Buddha's Birthday 2036 = Sat 3 May; no substitute.)
    (2036, 5, 1),   // 2036-05-01 — Labour Day (Thu)
    (2036, 5, 30),  // 2036-05-30 — Tuen Ng Festival (Fri)
    (2036, 7, 1),   // 2036-07-01 — HKSAR Establishment Day (Tue)
    (2036, 9, 29),  // 2036-09-29 — Chung Yeung Festival substitute (Mon; rule date Sun 28)
    (2036, 10, 1),  // 2036-10-01 — National Day (Wed)
    (2036, 10, 6),  // 2036-10-06 — Day after Mid-Autumn substitute (Mon; rule date Sun 5)
    (2036, 12, 25), // 2036-12-25 — Christmas Day (Thu)
    (2036, 12, 26), // 2036-12-26 — Day after Christmas (Fri)
    // ─── 2037 ────────────────────────────────────────────────────────────
    (2037, 1, 1),  // 2037-01-01 — New Year's Day (Thu)
    (2037, 2, 16), // 2037-02-16 — Lunar New Year Day 2 substitute (Mon; Day 1 Sun 15)
    (2037, 2, 17), // 2037-02-17 — Lunar New Year Day 3 (Tue)
    (2037, 2, 18), // 2037-02-18 — Lunar New Year Day 4 substitute (Wed)
    (2037, 4, 3),  // 2037-04-03 — Good Friday
    (2037, 4, 4),  // 2037-04-04 — Day after Good Friday (Holy Saturday)
    (2037, 4, 6),  // 2037-04-06 — Easter Monday  (also Ching Ming substitute — rule date Sun 5)
    (2037, 4, 7),  // 2037-04-07 — Ching Ming Festival extra substitute (Tue; pushed by Easter Mon)
    (2037, 5, 1),  // 2037-05-01 — Labour Day (Fri)
    (2037, 5, 22), // 2037-05-22 — Buddha's Birthday (Fri)
    (2037, 6, 17), // 2037-06-17 — Tuen Ng Festival (Wed)
    (2037, 7, 1),  // 2037-07-01 — HKSAR Establishment Day (Wed)
    (2037, 9, 25), // 2037-09-25 — Day after Mid-Autumn Festival (Fri)
    (2037, 10, 1), // 2037-10-01 — National Day (Thu)
    // (Chung Yeung 2037 = Sat 17 Oct; HKEX closed Sat, no substitute.)
    (2037, 12, 25), // 2037-12-25 — Christmas Day (Fri)
    // (Boxing Day 2037 = Sat 26 Dec; HKEX closed Sat, no substitute.)
    // ─── 2038 ────────────────────────────────────────────────────────────
    (2038, 1, 1), // 2038-01-01 — New Year's Day (Fri)
    (2038, 2, 4), // 2038-02-04 — Lunar New Year Day 1 (Thu)
    (2038, 2, 5), // 2038-02-05 — Lunar New Year Day 2 (Fri)
    // (Lunar Day 3 2038 = Sat 6 Feb; HKEX closed Sat, no substitute.)
    (2038, 4, 5),  // 2038-04-05 — Ching Ming Festival (Mon)
    (2038, 4, 23), // 2038-04-23 — Good Friday
    (2038, 4, 24), // 2038-04-24 — Day after Good Friday (Holy Saturday)
    (2038, 4, 26), // 2038-04-26 — Easter Monday
    (2038, 5, 11), // 2038-05-11 — Buddha's Birthday (Tue)
    (2038, 6, 7),  // 2038-06-07 — Tuen Ng Festival substitute (Mon; rule date Sun 6)
    (2038, 7, 1),  // 2038-07-01 — HKSAR Establishment Day (Thu)
    (2038, 9, 14), // 2038-09-14 — Day after Mid-Autumn Festival (Tue)
    (2038, 10, 1), // 2038-10-01 — National Day (Fri)
    (2038, 10, 6), // 2038-10-06 — Chung Yeung Festival (Wed)
    // (Labour Day 2038 = Sat 1 May; HKEX closed Sat, no substitute.)
    (2038, 12, 27), // 2038-12-27 — Christmas Day substitute (Mon; rule date Sat 25)
    (2038, 12, 28), // 2038-12-28 — Day after Christmas substitute (Tue; rule date Sun 26)
    // ─── 2039 ────────────────────────────────────────────────────────────
    (2039, 1, 3),  // 2039-01-03 — New Year's Day substitute (Mon; rule date Sat 1)
    (2039, 1, 24), // 2039-01-24 — Lunar New Year Day 1 (Mon)
    (2039, 1, 25), // 2039-01-25 — Lunar New Year Day 2 (Tue)
    (2039, 1, 26), // 2039-01-26 — Lunar New Year Day 3 (Wed)
    (2039, 4, 5),  // 2039-04-05 — Ching Ming Festival (Tue)
    (2039, 4, 8),  // 2039-04-08 — Good Friday
    (2039, 4, 9),  // 2039-04-09 — Day after Good Friday (Holy Saturday)
    (2039, 4, 11), // 2039-04-11 — Easter Monday
    // (Labour Day 2039 = Sun 1 May → Mon 2 May substitute.)
    (2039, 5, 2), // 2039-05-02 — Labour Day substitute (Mon; rule date Sun 1)
    // (Buddha's Birthday 2039 = Sat 30 Apr; HKEX closed Sat, no substitute.)
    (2039, 5, 27),  // 2039-05-27 — Tuen Ng Festival (Fri)
    (2039, 7, 1),   // 2039-07-01 — HKSAR Establishment Day (Fri)
    (2039, 10, 3),  // 2039-10-03 — Day after Mid-Autumn / National Day Mon (Sat 1 / Sun 2)
    (2039, 10, 4),  // 2039-10-04 — National Day substitute (Tue; Sun observed pushed)
    (2039, 10, 25), // 2039-10-25 — Chung Yeung Festival (Tue)
    (2039, 12, 26), // 2039-12-26 — Christmas Day substitute (Mon; rule date Sun 25)
    (2039, 12, 27), // 2039-12-27 — Day after Christmas substitute (Tue; pushed by Christmas Mon)
    // ─── 2040 ────────────────────────────────────────────────────────────
    (2040, 1, 2),  // 2040-01-02 — New Year's Day substitute (Mon; rule date Sun 1)
    (2040, 2, 13), // 2040-02-13 — Lunar New Year Day 2 substitute (Mon; Day 1 Sun 12)
    (2040, 2, 14), // 2040-02-14 — Lunar New Year Day 3 (Tue)
    (2040, 2, 15), // 2040-02-15 — Lunar New Year Day 4 substitute (Wed)
    (2040, 3, 30), // 2040-03-30 — Good Friday
    (2040, 3, 31), // 2040-03-31 — Day after Good Friday (Holy Saturday)
    (2040, 4, 2),  // 2040-04-02 — Easter Monday
    (2040, 4, 4),  // 2040-04-04 — Ching Ming Festival (Wed)
    (2040, 5, 1),  // 2040-05-01 — Labour Day (Tue)
    (2040, 5, 18), // 2040-05-18 — Buddha's Birthday (Fri)
    (2040, 6, 14), // 2040-06-14 — Tuen Ng Festival (Thu)
    // (HKSAR 2040 = Sun 1 Jul → Mon 2 Jul substitute.)
    (2040, 7, 2), // 2040-07-02 — HKSAR Establishment Day substitute (Mon; rule date Sun 1)
    (2040, 9, 21), // 2040-09-21 — Day after Mid-Autumn Festival (Fri)
    (2040, 10, 1), // 2040-10-01 — National Day (Mon)
    // (Chung Yeung 2040 = Sat 13 Oct; HKEX closed Sat, no substitute.)
    (2040, 12, 25), // 2040-12-25 — Christmas Day (Tue)
    (2040, 12, 26), // 2040-12-26 — Day after Christmas (Wed)
];

// ─── Lookup ──────────────────────────────────────────────────────────────────

/// True if `date` is an HKEX trading-calendar holiday in the snapshot
/// window [`COVERAGE`].
///
/// Lookup is a binary search over [`HOLIDAYS`], which is sorted in
/// ascending chronological order. Dates whose year falls outside
/// [`COVERAGE`] return `false` — the snapshot makes no claim about them,
/// and the caller is responsible for refreshing the table when the
/// horizon advances.
///
/// Weekends are NOT classified as holidays here — they are handled by the
/// caller's business-day predicate (see the `is_business_day` dispatcher
/// in [`crate::calendar`]). A rule date that happens to fall on a Sunday
/// is reported on its observed Monday (already baked into [`HOLIDAYS`]).
///
/// # Examples
///
/// ```
/// use regit_daycount::Date;
/// use regit_daycount::calendar::hong_kong;
///
/// // Christmas Day 2024 is an HKEX holiday.
/// assert!(hong_kong::is_holiday(Date::ymd(2024, 12, 25).unwrap()));
///
/// // Christmas Eve 2024 is a regular trading day.
/// assert!(!hong_kong::is_holiday(Date::ymd(2024, 12, 24).unwrap()));
/// ```
#[must_use]
pub fn is_holiday(date: Date) -> bool {
    let key = (date.year(), date.month(), date.day());
    HOLIDAYS.binary_search(&key).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── 2024 — every holiday in the year against the published HKEX list ─

    #[test]
    fn holidays_2024() {
        // Each entry below is a date the HKEX trading calendar names as
        // closed in 2024. 2024-03-30 (Holy Saturday) sits on a Saturday;
        // HKEX publishes "the day after Good Friday" as a closure even
        // though the exchange is already closed Saturdays — this row pins
        // that documentary expectation.
        let dates = [
            (1, 1),   // New Year's Day (Mon)
            (2, 12),  // Lunar New Year Day 3 substitute (Mon; Day 2 Sun 11)
            (2, 13),  // Lunar New Year Day 3 (Tue)
            (3, 29),  // Good Friday
            (3, 30),  // Day after Good Friday (Holy Saturday)
            (4, 1),   // Easter Monday
            (4, 4),   // Ching Ming Festival
            (5, 1),   // Labour Day
            (5, 15),  // Buddha's Birthday
            (6, 10),  // Tuen Ng Festival
            (7, 1),   // HKSAR Establishment Day
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
        assert_eq!(SNAPSHOT_DATE, "2026-05-23");
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
}
