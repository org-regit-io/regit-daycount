// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! Japan — Japan Exchange Group (JPX) trading calendar and the Japanese
//! National Holidays Act.
//!
//! The Tokyo Stock Exchange (operated by JPX) closes on every national
//! holiday named in the *Act on National Holidays* (国民の祝日に関する
//! 法律, *kokumin no shukujitsu ni kansuru hōritsu*) plus a year-end /
//! New Year exchange-specific closure on **2 January, 3 January, and 31
//! December**. Two features of the Japanese statute make the calendar a
//! dated snapshot rather than a closed-form rule:
//!
//! 1. Two holidays — the **Vernal Equinox Day** (*Shunbun no Hi*, around
//!    20–21 March) and the **Autumnal Equinox Day** (*Shubun no Hi*,
//!    around 22–23 September) — are fixed each year by Cabinet Office
//!    proclamation from the astronomical equinox computed by the National
//!    Astronomical Observatory of Japan. They are *not* derivable from a
//!    closed-form rule.
//! 2. Three holidays use the *Happy Monday* rule (a fixed *n*-th Monday
//!    of a month): **Coming of Age Day** (2nd Mon Jan), **Marine Day**
//!    (3rd Mon Jul), **Respect for the Aged Day** (3rd Mon Sep), and
//!    **Health and Sports Day** (2nd Mon Oct).
//!
//! ## Holiday rules
//!
//! ```text
//! New Year's Day                  1 Jan        (statutory)
//! JPX year-end / New Year break   2 Jan, 3 Jan (JPX-specific)
//! Coming of Age Day               2nd Mon Jan
//! National Foundation Day         11 Feb
//! Emperor's Birthday              23 Feb       (since 2020)
//! Vernal Equinox Day              20 or 21 Mar (Cabinet Office table)
//! Showa Day                       29 Apr
//! Constitution Memorial Day       3 May
//! Greenery Day                    4 May
//! Children's Day                  5 May
//! Marine Day                      3rd Mon Jul
//! Mountain Day                    11 Aug
//! Respect for the Aged Day        3rd Mon Sep
//! Autumnal Equinox Day            22 or 23 Sep (Cabinet Office table)
//! Health and Sports Day           2nd Mon Oct
//! Culture Day                     3 Nov
//! Labor Thanksgiving Day          23 Nov
//! JPX year-end                    31 Dec       (JPX-specific)
//! ```
//!
//! ## Olympic-year specials (2020, 2021)
//!
//! The Tokyo 2020 Olympics, postponed to 2021, moved several holidays in
//! both years:
//!
//! ```text
//! 2020   Marine Day   → 23 Jul    (opening-ceremony eve)
//! 2020   Sports Day   → 24 Jul    (opening day; renamed from Health & Sports)
//! 2020   Mountain Day → 10 Aug    (closing-ceremony day)
//! 2021   Marine Day   → 22 Jul
//! 2021   Sports Day   → 23 Jul
//! 2021   Mountain Day → 8 Aug
//! ```
//!
//! ## Substitution rule (*furikae kyūjitsu*)
//!
//! When a national holiday falls on a Sunday, the following Monday becomes
//! a *substitute holiday*; if that Monday is itself already a holiday, the
//! substitute slides forward to the next non-holiday weekday. JPX year-end
//! exchange closures (2 Jan, 3 Jan, 31 Dec) are *not* statutory holidays
//! and do not generate substitutes when they fall on a Sunday.
//!
//! ## People's Holiday rule (*kokumin no shukujitsu*)
//!
//! A non-holiday weekday sandwiched between two national holidays becomes
//! itself a holiday. Since the 2007 elevation of 4 May to Greenery Day,
//! this rule fires only when Respect for the Aged Day (3rd Mon Sep) and
//! the Autumnal Equinox Day are separated by exactly one weekday — i.e.
//! when Respect for the Aged Day is the 21st of September *and* the
//! autumnal equinox is the 23rd. Within this snapshot's coverage that
//! occurs in **2026, 2032, and 2037**, with 22 (resp. 21, 22) September
//! observed as a People's Holiday.
//!
//! ## Snapshot
//!
//! - Source: Japan Exchange Group published trading calendar (Tokyo Stock
//!   Exchange) cross-checked against the Cabinet Office (*Naikaku-fu*)
//!   national-holidays table.
//! - Snapshot date: see [`SNAPSHOT_DATE`].
//! - Coverage: see [`COVERAGE`] — 2020 through 2040 inclusive, 21 years,
//!   432 entries.
//!
//! Out-of-coverage dates return `false` from [`is_holiday`]; the caller is
//! responsible for refusing to settle a cashflow beyond this horizon
//! against the Japan calendar.
//!
//! ## Verification
//!
//! Every row in [`HOLIDAYS`] was cross-verified on **2026-05-23** against
//! the following primary sources:
//!
//! - Cabinet Office of Japan, *Kokumin no shukujitsu* gazette:
//!   <https://www8.cao.go.jp/chosei/shukujitsu/gaiyou.html>. Gazetted
//!   horizon as of the snapshot date covers 2020–2027 (the Cabinet
//!   Office publishes the upcoming year's equinoxes in February of the
//!   preceding year; the 2027 dates were gazetted in February 2026).
//! - Japan Exchange Group, *Trading Calendar of the Tokyo Stock
//!   Exchange*:
//!   <https://www.jpx.co.jp/english/corporate/about-jpx/calendar/index.html>.
//!   JPX publishes the trading calendar two years ahead; at the
//!   snapshot date the 2024, 2025, 2026 (and provisional 2027) PDFs are
//!   binding.
//! - *Public holidays in Japan* and *List of observances set by the
//!   Japanese calendar* on Wikipedia, used as a secondary
//!   cross-reference for the Aoki/NAOJ astronomical equinox formula
//!   beyond the gazetted horizon.
//!
//! Vernal- and autumnal-equinox dates for years 2028–2040 are
//! *astronomical inferences* computed from the Aoki/NAOJ formula
//! (valid 1980–2099) and will be replaced by the gazetted Cabinet
//! Office value as it becomes available; each affected row carries an
//! `equinox: NAOJ astronomical inference (Aoki formula)` annotation.
//! Gazetted years (2020–2027) carry an `equinox: source = Cabinet
//! Office published table` annotation. The Aoki formula was
//! independently re-verified to agree with every Cabinet Office
//! gazetted value 2020–2027 inclusive.
//!
//! People's Holiday (*kokumin no shukujitsu*) firings in the coverage
//! window were confirmed for **2026, 2032, and 2037** — these are the
//! only years 2020–2040 in which Respect for the Aged Day (3rd Mon
//! Sep) and the autumnal-equinox date are separated by a single
//! weekday.
//!
//! The Tokyo-2020 Olympic-year holiday moves were confirmed for both
//! 2020 (Marine 23 Jul, Sports 24 Jul, Mountain 10 Aug) and 2021
//! (Marine 22 Jul, Sports 23 Jul, Mountain 8 Aug — substitute Mon 9
//! Aug for the Sunday) against the gazetted Act amendment.
//!
//! # References
//!
//! - *Act on National Holidays* (国民の祝日に関する法律), Law No. 178 of
//!   1948, as amended — the statutory list of *kokumin no shukujitsu*.
//! - Cabinet Office (Government of Japan), annual *koyomi yōkō*
//!   (calendar gazette) proclaiming the Vernal and Autumnal Equinox Days
//!   for the coming year, derived from the National Astronomical
//!   Observatory of Japan's equinox tables.
//! - Japan Exchange Group, *Trading Calendar of the Tokyo Stock Exchange*
//!   — defines the additional 2 January, 3 January and 31 December
//!   exchange closures.

use crate::date::Date;

/// Date of the JPX trading-calendar snapshot embedded in this module,
/// `YYYY-MM-DD`.
pub const SNAPSHOT_DATE: &str = "2026-05-23";

/// Inclusive `(first, last)` years covered by [`HOLIDAYS`].
///
/// [`is_holiday`] returns `false` for any date outside this range —
/// the snapshot has nothing to say about it.
pub const COVERAGE: (i32, i32) = (2020, 2040);

/// JPX / Japan-statutory holiday table, sorted ascending by `(year,
/// month, day)`.
///
/// Each row is `(year, month, day)` of a date on which the Tokyo Stock
/// Exchange is closed: a national holiday named in the Act on National
/// Holidays, a substitute holiday under the *furikae kyūjitsu* rule, a
/// People's Holiday under the *kokumin no shukujitsu* rule, or the JPX
/// year-end / New Year break (2 Jan, 3 Jan, 31 Dec).
///
/// Coverage: 2020 through 2040 inclusive (see [`COVERAGE`]).
///
/// Sorted; binary-searchable.
#[rustfmt::skip]
pub const HOLIDAYS: &[(i32, u8, u8)] = &[
    // ─── 2020 (Tokyo Olympics — postponed; calendar already adopted) ─────
    (2020,  1,  1), // 2020-01-01 — New Year's Day (Ganjitsu)
    (2020,  1,  2), // 2020-01-02 — JPX year-end / New Year break
    (2020,  1,  3), // 2020-01-03 — JPX year-end / New Year break
    (2020,  1, 13), // 2020-01-13 — Coming of Age Day (2nd Mon Jan)
    (2020,  2, 11), // 2020-02-11 — National Foundation Day
    (2020,  2, 23), // 2020-02-23 — Emperor's Birthday (Sun)
    (2020,  2, 24), // 2020-02-24 — substitute for 2020-02-23 Emperor's Birthday
    // equinox: source = Cabinet Office published table
    (2020,  3, 20), // 2020-03-20 — Vernal Equinox Day
    (2020,  4, 29), // 2020-04-29 — Showa Day
    (2020,  5,  3), // 2020-05-03 — Constitution Memorial Day (Sun)
    (2020,  5,  4), // 2020-05-04 — Greenery Day
    (2020,  5,  5), // 2020-05-05 — Children's Day
    (2020,  5,  6), // 2020-05-06 — substitute for 2020-05-03 Constitution Memorial Day
    (2020,  7, 23), // 2020-07-23 — Marine Day (Olympic move from 3rd Mon Jul)
    (2020,  7, 24), // 2020-07-24 — Sports Day (Olympic move from 2nd Mon Oct)
    (2020,  8, 10), // 2020-08-10 — Mountain Day (Olympic move from 11 Aug)
    (2020,  9, 21), // 2020-09-21 — Respect for the Aged Day (3rd Mon Sep)
    // equinox: source = Cabinet Office published table
    (2020,  9, 22), // 2020-09-22 — Autumnal Equinox Day
    (2020, 11,  3), // 2020-11-03 — Culture Day
    (2020, 11, 23), // 2020-11-23 — Labor Thanksgiving Day
    (2020, 12, 31), // 2020-12-31 — JPX year-end
    // ─── 2021 (Tokyo Olympics — held; Olympic moves applied) ─────────────
    (2021,  1,  1), // 2021-01-01 — New Year's Day
    (2021,  1,  2), // 2021-01-02 — JPX year-end / New Year break
    (2021,  1,  3), // 2021-01-03 — JPX year-end / New Year break
    (2021,  1, 11), // 2021-01-11 — Coming of Age Day (2nd Mon Jan)
    (2021,  2, 11), // 2021-02-11 — National Foundation Day
    (2021,  2, 23), // 2021-02-23 — Emperor's Birthday
    // equinox: source = Cabinet Office published table
    (2021,  3, 20), // 2021-03-20 — Vernal Equinox Day
    (2021,  4, 29), // 2021-04-29 — Showa Day
    (2021,  5,  3), // 2021-05-03 — Constitution Memorial Day
    (2021,  5,  4), // 2021-05-04 — Greenery Day
    (2021,  5,  5), // 2021-05-05 — Children's Day
    (2021,  7, 22), // 2021-07-22 — Marine Day (Olympic move from 3rd Mon Jul)
    (2021,  7, 23), // 2021-07-23 — Sports Day (Olympic move from 2nd Mon Oct)
    (2021,  8,  8), // 2021-08-08 — Mountain Day (Olympic move from 11 Aug) (Sun)
    (2021,  8,  9), // 2021-08-09 — substitute for 2021-08-08 Mountain Day
    (2021,  9, 20), // 2021-09-20 — Respect for the Aged Day (3rd Mon Sep)
    // equinox: source = Cabinet Office published table
    (2021,  9, 23), // 2021-09-23 — Autumnal Equinox Day
    (2021, 11,  3), // 2021-11-03 — Culture Day
    (2021, 11, 23), // 2021-11-23 — Labor Thanksgiving Day
    (2021, 12, 31), // 2021-12-31 — JPX year-end
    // ─── 2022 ────────────────────────────────────────────────────────────
    (2022,  1,  1), // 2022-01-01 — New Year's Day
    (2022,  1,  2), // 2022-01-02 — JPX year-end / New Year break
    (2022,  1,  3), // 2022-01-03 — JPX year-end / New Year break
    (2022,  1, 10), // 2022-01-10 — Coming of Age Day (2nd Mon Jan)
    (2022,  2, 11), // 2022-02-11 — National Foundation Day
    (2022,  2, 23), // 2022-02-23 — Emperor's Birthday
    // equinox: source = Cabinet Office published table
    (2022,  3, 21), // 2022-03-21 — Vernal Equinox Day
    (2022,  4, 29), // 2022-04-29 — Showa Day
    (2022,  5,  3), // 2022-05-03 — Constitution Memorial Day
    (2022,  5,  4), // 2022-05-04 — Greenery Day
    (2022,  5,  5), // 2022-05-05 — Children's Day
    (2022,  7, 18), // 2022-07-18 — Marine Day (3rd Mon Jul)
    (2022,  8, 11), // 2022-08-11 — Mountain Day
    (2022,  9, 19), // 2022-09-19 — Respect for the Aged Day (3rd Mon Sep)
    // equinox: source = Cabinet Office published table
    (2022,  9, 23), // 2022-09-23 — Autumnal Equinox Day
    (2022, 10, 10), // 2022-10-10 — Health and Sports Day (2nd Mon Oct)
    (2022, 11,  3), // 2022-11-03 — Culture Day
    (2022, 11, 23), // 2022-11-23 — Labor Thanksgiving Day
    (2022, 12, 31), // 2022-12-31 — JPX year-end
    // ─── 2023 ────────────────────────────────────────────────────────────
    (2023,  1,  1), // 2023-01-01 — New Year's Day (Sun; substitute is JPX 2 Jan)
    (2023,  1,  2), // 2023-01-02 — JPX year-end / New Year break
    (2023,  1,  3), // 2023-01-03 — JPX year-end / New Year break
    (2023,  1,  9), // 2023-01-09 — Coming of Age Day (2nd Mon Jan)
    (2023,  2, 11), // 2023-02-11 — National Foundation Day
    (2023,  2, 23), // 2023-02-23 — Emperor's Birthday
    // equinox: source = Cabinet Office published table
    (2023,  3, 21), // 2023-03-21 — Vernal Equinox Day
    (2023,  4, 29), // 2023-04-29 — Showa Day
    (2023,  5,  3), // 2023-05-03 — Constitution Memorial Day
    (2023,  5,  4), // 2023-05-04 — Greenery Day
    (2023,  5,  5), // 2023-05-05 — Children's Day
    (2023,  7, 17), // 2023-07-17 — Marine Day (3rd Mon Jul)
    (2023,  8, 11), // 2023-08-11 — Mountain Day
    (2023,  9, 18), // 2023-09-18 — Respect for the Aged Day (3rd Mon Sep)
    // equinox: source = Cabinet Office published table
    (2023,  9, 23), // 2023-09-23 — Autumnal Equinox Day
    (2023, 10,  9), // 2023-10-09 — Health and Sports Day (2nd Mon Oct)
    (2023, 11,  3), // 2023-11-03 — Culture Day
    (2023, 11, 23), // 2023-11-23 — Labor Thanksgiving Day
    (2023, 12, 31), // 2023-12-31 — JPX year-end
    // ─── 2024 ────────────────────────────────────────────────────────────
    (2024,  1,  1), // 2024-01-01 — New Year's Day
    (2024,  1,  2), // 2024-01-02 — JPX year-end / New Year break
    (2024,  1,  3), // 2024-01-03 — JPX year-end / New Year break
    (2024,  1,  8), // 2024-01-08 — Coming of Age Day (2nd Mon Jan)
    (2024,  2, 11), // 2024-02-11 — National Foundation Day (Sun)
    (2024,  2, 12), // 2024-02-12 — substitute for 2024-02-11 National Foundation Day
    (2024,  2, 23), // 2024-02-23 — Emperor's Birthday
    // equinox: source = Cabinet Office published table
    (2024,  3, 20), // 2024-03-20 — Vernal Equinox Day
    (2024,  4, 29), // 2024-04-29 — Showa Day
    (2024,  5,  3), // 2024-05-03 — Constitution Memorial Day
    (2024,  5,  4), // 2024-05-04 — Greenery Day
    (2024,  5,  5), // 2024-05-05 — Children's Day (Sun)
    (2024,  5,  6), // 2024-05-06 — substitute for 2024-05-05 Children's Day
    (2024,  7, 15), // 2024-07-15 — Marine Day (3rd Mon Jul)
    (2024,  8, 11), // 2024-08-11 — Mountain Day (Sun)
    (2024,  8, 12), // 2024-08-12 — substitute for 2024-08-11 Mountain Day
    (2024,  9, 16), // 2024-09-16 — Respect for the Aged Day (3rd Mon Sep)
    // equinox: source = Cabinet Office published table
    (2024,  9, 22), // 2024-09-22 — Autumnal Equinox Day (Sun)
    (2024,  9, 23), // 2024-09-23 — substitute for 2024-09-22 Autumnal Equinox Day
    (2024, 10, 14), // 2024-10-14 — Health and Sports Day (2nd Mon Oct)
    (2024, 11,  3), // 2024-11-03 — Culture Day (Sun)
    (2024, 11,  4), // 2024-11-04 — substitute for 2024-11-03 Culture Day
    (2024, 11, 23), // 2024-11-23 — Labor Thanksgiving Day
    (2024, 12, 31), // 2024-12-31 — JPX year-end
    // ─── 2025 ────────────────────────────────────────────────────────────
    (2025,  1,  1), // 2025-01-01 — New Year's Day
    (2025,  1,  2), // 2025-01-02 — JPX year-end / New Year break
    (2025,  1,  3), // 2025-01-03 — JPX year-end / New Year break
    (2025,  1, 13), // 2025-01-13 — Coming of Age Day (2nd Mon Jan)
    (2025,  2, 11), // 2025-02-11 — National Foundation Day
    (2025,  2, 23), // 2025-02-23 — Emperor's Birthday (Sun)
    (2025,  2, 24), // 2025-02-24 — substitute for 2025-02-23 Emperor's Birthday
    // equinox: source = Cabinet Office published table
    (2025,  3, 20), // 2025-03-20 — Vernal Equinox Day
    (2025,  4, 29), // 2025-04-29 — Showa Day
    (2025,  5,  3), // 2025-05-03 — Constitution Memorial Day
    (2025,  5,  4), // 2025-05-04 — Greenery Day (Sun)
    (2025,  5,  5), // 2025-05-05 — Children's Day
    (2025,  5,  6), // 2025-05-06 — substitute for 2025-05-04 Greenery Day
    (2025,  7, 21), // 2025-07-21 — Marine Day (3rd Mon Jul)
    (2025,  8, 11), // 2025-08-11 — Mountain Day
    (2025,  9, 15), // 2025-09-15 — Respect for the Aged Day (3rd Mon Sep)
    // equinox: source = Cabinet Office published table
    (2025,  9, 23), // 2025-09-23 — Autumnal Equinox Day
    (2025, 10, 13), // 2025-10-13 — Health and Sports Day (2nd Mon Oct)
    (2025, 11,  3), // 2025-11-03 — Culture Day
    (2025, 11, 23), // 2025-11-23 — Labor Thanksgiving Day (Sun)
    (2025, 11, 24), // 2025-11-24 — substitute for 2025-11-23 Labor Thanksgiving Day
    (2025, 12, 31), // 2025-12-31 — JPX year-end
    // ─── 2026 ────────────────────────────────────────────────────────────
    (2026,  1,  1), // 2026-01-01 — New Year's Day
    (2026,  1,  2), // 2026-01-02 — JPX year-end / New Year break
    (2026,  1,  3), // 2026-01-03 — JPX year-end / New Year break
    (2026,  1, 12), // 2026-01-12 — Coming of Age Day (2nd Mon Jan)
    (2026,  2, 11), // 2026-02-11 — National Foundation Day
    (2026,  2, 23), // 2026-02-23 — Emperor's Birthday
    // equinox: source = Cabinet Office published table
    (2026,  3, 20), // 2026-03-20 — Vernal Equinox Day
    (2026,  4, 29), // 2026-04-29 — Showa Day
    (2026,  5,  3), // 2026-05-03 — Constitution Memorial Day (Sun)
    (2026,  5,  4), // 2026-05-04 — Greenery Day
    (2026,  5,  5), // 2026-05-05 — Children's Day
    (2026,  5,  6), // 2026-05-06 — substitute for 2026-05-03 Constitution Memorial Day
    (2026,  7, 20), // 2026-07-20 — Marine Day (3rd Mon Jul)
    (2026,  8, 11), // 2026-08-11 — Mountain Day
    (2026,  9, 21), // 2026-09-21 — Respect for the Aged Day (3rd Mon Sep)
    (2026,  9, 22), // 2026-09-22 — People's Holiday (sandwiched between Respect & Equinox)
    // equinox: source = Cabinet Office published table
    (2026,  9, 23), // 2026-09-23 — Autumnal Equinox Day
    (2026, 10, 12), // 2026-10-12 — Health and Sports Day (2nd Mon Oct)
    (2026, 11,  3), // 2026-11-03 — Culture Day
    (2026, 11, 23), // 2026-11-23 — Labor Thanksgiving Day
    (2026, 12, 31), // 2026-12-31 — JPX year-end
    // ─── 2027 ────────────────────────────────────────────────────────────
    (2027,  1,  1), // 2027-01-01 — New Year's Day
    (2027,  1,  2), // 2027-01-02 — JPX year-end / New Year break
    (2027,  1,  3), // 2027-01-03 — JPX year-end / New Year break
    (2027,  1, 11), // 2027-01-11 — Coming of Age Day (2nd Mon Jan)
    (2027,  2, 11), // 2027-02-11 — National Foundation Day
    (2027,  2, 23), // 2027-02-23 — Emperor's Birthday
    // equinox: source = Cabinet Office published table
    (2027,  3, 21), // 2027-03-21 — Vernal Equinox Day (Sun)
    (2027,  3, 22), // 2027-03-22 — substitute for 2027-03-21 Vernal Equinox Day
    (2027,  4, 29), // 2027-04-29 — Showa Day
    (2027,  5,  3), // 2027-05-03 — Constitution Memorial Day
    (2027,  5,  4), // 2027-05-04 — Greenery Day
    (2027,  5,  5), // 2027-05-05 — Children's Day
    (2027,  7, 19), // 2027-07-19 — Marine Day (3rd Mon Jul)
    (2027,  8, 11), // 2027-08-11 — Mountain Day
    (2027,  9, 20), // 2027-09-20 — Respect for the Aged Day (3rd Mon Sep)
    // equinox: source = Cabinet Office published table
    (2027,  9, 23), // 2027-09-23 — Autumnal Equinox Day
    (2027, 10, 11), // 2027-10-11 — Health and Sports Day (2nd Mon Oct)
    (2027, 11,  3), // 2027-11-03 — Culture Day
    (2027, 11, 23), // 2027-11-23 — Labor Thanksgiving Day
    (2027, 12, 31), // 2027-12-31 — JPX year-end
    // ─── 2028 ────────────────────────────────────────────────────────────
    (2028,  1,  1), // 2028-01-01 — New Year's Day
    (2028,  1,  2), // 2028-01-02 — JPX year-end / New Year break
    (2028,  1,  3), // 2028-01-03 — JPX year-end / New Year break
    (2028,  1, 10), // 2028-01-10 — Coming of Age Day (2nd Mon Jan)
    (2028,  2, 11), // 2028-02-11 — National Foundation Day
    (2028,  2, 23), // 2028-02-23 — Emperor's Birthday
    // equinox: NAOJ astronomical inference (Aoki formula)
    (2028,  3, 20), // 2028-03-20 — Vernal Equinox Day
    (2028,  4, 29), // 2028-04-29 — Showa Day
    (2028,  5,  3), // 2028-05-03 — Constitution Memorial Day
    (2028,  5,  4), // 2028-05-04 — Greenery Day
    (2028,  5,  5), // 2028-05-05 — Children's Day
    (2028,  7, 17), // 2028-07-17 — Marine Day (3rd Mon Jul)
    (2028,  8, 11), // 2028-08-11 — Mountain Day
    (2028,  9, 18), // 2028-09-18 — Respect for the Aged Day (3rd Mon Sep)
    // equinox: NAOJ astronomical inference (Aoki formula)
    (2028,  9, 22), // 2028-09-22 — Autumnal Equinox Day
    (2028, 10,  9), // 2028-10-09 — Health and Sports Day (2nd Mon Oct)
    (2028, 11,  3), // 2028-11-03 — Culture Day
    (2028, 11, 23), // 2028-11-23 — Labor Thanksgiving Day
    (2028, 12, 31), // 2028-12-31 — JPX year-end
    // ─── 2029 ────────────────────────────────────────────────────────────
    (2029,  1,  1), // 2029-01-01 — New Year's Day
    (2029,  1,  2), // 2029-01-02 — JPX year-end / New Year break
    (2029,  1,  3), // 2029-01-03 — JPX year-end / New Year break
    (2029,  1,  8), // 2029-01-08 — Coming of Age Day (2nd Mon Jan)
    (2029,  2, 11), // 2029-02-11 — National Foundation Day (Sun)
    (2029,  2, 12), // 2029-02-12 — substitute for 2029-02-11 National Foundation Day
    (2029,  2, 23), // 2029-02-23 — Emperor's Birthday
    // equinox: NAOJ astronomical inference (Aoki formula)
    (2029,  3, 20), // 2029-03-20 — Vernal Equinox Day
    (2029,  4, 29), // 2029-04-29 — Showa Day (Sun)
    (2029,  4, 30), // 2029-04-30 — substitute for 2029-04-29 Showa Day
    (2029,  5,  3), // 2029-05-03 — Constitution Memorial Day
    (2029,  5,  4), // 2029-05-04 — Greenery Day
    (2029,  5,  5), // 2029-05-05 — Children's Day
    (2029,  7, 16), // 2029-07-16 — Marine Day (3rd Mon Jul)
    (2029,  8, 11), // 2029-08-11 — Mountain Day
    (2029,  9, 17), // 2029-09-17 — Respect for the Aged Day (3rd Mon Sep)
    // equinox: NAOJ astronomical inference (Aoki formula)
    (2029,  9, 23), // 2029-09-23 — Autumnal Equinox Day (Sun)
    (2029,  9, 24), // 2029-09-24 — substitute for 2029-09-23 Autumnal Equinox Day
    (2029, 10,  8), // 2029-10-08 — Health and Sports Day (2nd Mon Oct)
    (2029, 11,  3), // 2029-11-03 — Culture Day
    (2029, 11, 23), // 2029-11-23 — Labor Thanksgiving Day
    (2029, 12, 31), // 2029-12-31 — JPX year-end
    // ─── 2030 ────────────────────────────────────────────────────────────
    (2030,  1,  1), // 2030-01-01 — New Year's Day
    (2030,  1,  2), // 2030-01-02 — JPX year-end / New Year break
    (2030,  1,  3), // 2030-01-03 — JPX year-end / New Year break
    (2030,  1, 14), // 2030-01-14 — Coming of Age Day (2nd Mon Jan)
    (2030,  2, 11), // 2030-02-11 — National Foundation Day
    (2030,  2, 23), // 2030-02-23 — Emperor's Birthday
    // equinox: NAOJ astronomical inference (Aoki formula)
    (2030,  3, 20), // 2030-03-20 — Vernal Equinox Day
    (2030,  4, 29), // 2030-04-29 — Showa Day
    (2030,  5,  3), // 2030-05-03 — Constitution Memorial Day
    (2030,  5,  4), // 2030-05-04 — Greenery Day
    (2030,  5,  5), // 2030-05-05 — Children's Day (Sun)
    (2030,  5,  6), // 2030-05-06 — substitute for 2030-05-05 Children's Day
    (2030,  7, 15), // 2030-07-15 — Marine Day (3rd Mon Jul)
    (2030,  8, 11), // 2030-08-11 — Mountain Day (Sun)
    (2030,  8, 12), // 2030-08-12 — substitute for 2030-08-11 Mountain Day
    (2030,  9, 16), // 2030-09-16 — Respect for the Aged Day (3rd Mon Sep)
    // equinox: NAOJ astronomical inference (Aoki formula)
    (2030,  9, 23), // 2030-09-23 — Autumnal Equinox Day
    (2030, 10, 14), // 2030-10-14 — Health and Sports Day (2nd Mon Oct)
    (2030, 11,  3), // 2030-11-03 — Culture Day (Sun)
    (2030, 11,  4), // 2030-11-04 — substitute for 2030-11-03 Culture Day
    (2030, 11, 23), // 2030-11-23 — Labor Thanksgiving Day
    (2030, 12, 31), // 2030-12-31 — JPX year-end
    // ─── 2031 ────────────────────────────────────────────────────────────
    (2031,  1,  1), // 2031-01-01 — New Year's Day
    (2031,  1,  2), // 2031-01-02 — JPX year-end / New Year break
    (2031,  1,  3), // 2031-01-03 — JPX year-end / New Year break
    (2031,  1, 13), // 2031-01-13 — Coming of Age Day (2nd Mon Jan)
    (2031,  2, 11), // 2031-02-11 — National Foundation Day
    (2031,  2, 23), // 2031-02-23 — Emperor's Birthday (Sun)
    (2031,  2, 24), // 2031-02-24 — substitute for 2031-02-23 Emperor's Birthday
    // equinox: NAOJ astronomical inference (Aoki formula)
    (2031,  3, 21), // 2031-03-21 — Vernal Equinox Day
    (2031,  4, 29), // 2031-04-29 — Showa Day
    (2031,  5,  3), // 2031-05-03 — Constitution Memorial Day
    (2031,  5,  4), // 2031-05-04 — Greenery Day (Sun)
    (2031,  5,  5), // 2031-05-05 — Children's Day
    (2031,  5,  6), // 2031-05-06 — substitute for 2031-05-04 Greenery Day
    (2031,  7, 21), // 2031-07-21 — Marine Day (3rd Mon Jul)
    (2031,  8, 11), // 2031-08-11 — Mountain Day
    (2031,  9, 15), // 2031-09-15 — Respect for the Aged Day (3rd Mon Sep)
    // equinox: NAOJ astronomical inference (Aoki formula)
    (2031,  9, 23), // 2031-09-23 — Autumnal Equinox Day
    (2031, 10, 13), // 2031-10-13 — Health and Sports Day (2nd Mon Oct)
    (2031, 11,  3), // 2031-11-03 — Culture Day
    (2031, 11, 23), // 2031-11-23 — Labor Thanksgiving Day (Sun)
    (2031, 11, 24), // 2031-11-24 — substitute for 2031-11-23 Labor Thanksgiving Day
    (2031, 12, 31), // 2031-12-31 — JPX year-end
    // ─── 2032 ────────────────────────────────────────────────────────────
    (2032,  1,  1), // 2032-01-01 — New Year's Day
    (2032,  1,  2), // 2032-01-02 — JPX year-end / New Year break
    (2032,  1,  3), // 2032-01-03 — JPX year-end / New Year break
    (2032,  1, 12), // 2032-01-12 — Coming of Age Day (2nd Mon Jan)
    (2032,  2, 11), // 2032-02-11 — National Foundation Day
    (2032,  2, 23), // 2032-02-23 — Emperor's Birthday
    // equinox: NAOJ astronomical inference (Aoki formula)
    (2032,  3, 20), // 2032-03-20 — Vernal Equinox Day
    (2032,  4, 29), // 2032-04-29 — Showa Day
    (2032,  5,  3), // 2032-05-03 — Constitution Memorial Day
    (2032,  5,  4), // 2032-05-04 — Greenery Day
    (2032,  5,  5), // 2032-05-05 — Children's Day
    (2032,  7, 19), // 2032-07-19 — Marine Day (3rd Mon Jul)
    (2032,  8, 11), // 2032-08-11 — Mountain Day
    (2032,  9, 20), // 2032-09-20 — Respect for the Aged Day (3rd Mon Sep)
    (2032,  9, 21), // 2032-09-21 — People's Holiday (sandwiched between Respect & Equinox)
    // equinox: NAOJ astronomical inference (Aoki formula)
    (2032,  9, 22), // 2032-09-22 — Autumnal Equinox Day
    (2032, 10, 11), // 2032-10-11 — Health and Sports Day (2nd Mon Oct)
    (2032, 11,  3), // 2032-11-03 — Culture Day
    (2032, 11, 23), // 2032-11-23 — Labor Thanksgiving Day
    (2032, 12, 31), // 2032-12-31 — JPX year-end
    // ─── 2033 ────────────────────────────────────────────────────────────
    (2033,  1,  1), // 2033-01-01 — New Year's Day
    (2033,  1,  2), // 2033-01-02 — JPX year-end / New Year break
    (2033,  1,  3), // 2033-01-03 — JPX year-end / New Year break
    (2033,  1, 10), // 2033-01-10 — Coming of Age Day (2nd Mon Jan)
    (2033,  2, 11), // 2033-02-11 — National Foundation Day
    (2033,  2, 23), // 2033-02-23 — Emperor's Birthday
    // equinox: NAOJ astronomical inference (Aoki formula)
    (2033,  3, 20), // 2033-03-20 — Vernal Equinox Day (Sun)
    (2033,  3, 21), // 2033-03-21 — substitute for 2033-03-20 Vernal Equinox Day
    (2033,  4, 29), // 2033-04-29 — Showa Day
    (2033,  5,  3), // 2033-05-03 — Constitution Memorial Day
    (2033,  5,  4), // 2033-05-04 — Greenery Day
    (2033,  5,  5), // 2033-05-05 — Children's Day
    (2033,  7, 18), // 2033-07-18 — Marine Day (3rd Mon Jul)
    (2033,  8, 11), // 2033-08-11 — Mountain Day
    (2033,  9, 19), // 2033-09-19 — Respect for the Aged Day (3rd Mon Sep)
    // equinox: NAOJ astronomical inference (Aoki formula)
    (2033,  9, 23), // 2033-09-23 — Autumnal Equinox Day
    (2033, 10, 10), // 2033-10-10 — Health and Sports Day (2nd Mon Oct)
    (2033, 11,  3), // 2033-11-03 — Culture Day
    (2033, 11, 23), // 2033-11-23 — Labor Thanksgiving Day
    (2033, 12, 31), // 2033-12-31 — JPX year-end
    // ─── 2034 ────────────────────────────────────────────────────────────
    (2034,  1,  1), // 2034-01-01 — New Year's Day (Sun; substitute is JPX 2 Jan)
    (2034,  1,  2), // 2034-01-02 — JPX year-end / New Year break
    (2034,  1,  3), // 2034-01-03 — JPX year-end / New Year break
    (2034,  1,  9), // 2034-01-09 — Coming of Age Day (2nd Mon Jan)
    (2034,  2, 11), // 2034-02-11 — National Foundation Day
    (2034,  2, 23), // 2034-02-23 — Emperor's Birthday
    // equinox: NAOJ astronomical inference (Aoki formula)
    (2034,  3, 20), // 2034-03-20 — Vernal Equinox Day
    (2034,  4, 29), // 2034-04-29 — Showa Day
    (2034,  5,  3), // 2034-05-03 — Constitution Memorial Day
    (2034,  5,  4), // 2034-05-04 — Greenery Day
    (2034,  5,  5), // 2034-05-05 — Children's Day
    (2034,  7, 17), // 2034-07-17 — Marine Day (3rd Mon Jul)
    (2034,  8, 11), // 2034-08-11 — Mountain Day
    (2034,  9, 18), // 2034-09-18 — Respect for the Aged Day (3rd Mon Sep)
    // equinox: NAOJ astronomical inference (Aoki formula)
    (2034,  9, 23), // 2034-09-23 — Autumnal Equinox Day
    (2034, 10,  9), // 2034-10-09 — Health and Sports Day (2nd Mon Oct)
    (2034, 11,  3), // 2034-11-03 — Culture Day
    (2034, 11, 23), // 2034-11-23 — Labor Thanksgiving Day
    (2034, 12, 31), // 2034-12-31 — JPX year-end
    // ─── 2035 ────────────────────────────────────────────────────────────
    (2035,  1,  1), // 2035-01-01 — New Year's Day
    (2035,  1,  2), // 2035-01-02 — JPX year-end / New Year break
    (2035,  1,  3), // 2035-01-03 — JPX year-end / New Year break
    (2035,  1,  8), // 2035-01-08 — Coming of Age Day (2nd Mon Jan)
    (2035,  2, 11), // 2035-02-11 — National Foundation Day (Sun)
    (2035,  2, 12), // 2035-02-12 — substitute for 2035-02-11 National Foundation Day
    (2035,  2, 23), // 2035-02-23 — Emperor's Birthday
    // equinox: NAOJ astronomical inference (Aoki formula)
    (2035,  3, 21), // 2035-03-21 — Vernal Equinox Day
    (2035,  4, 29), // 2035-04-29 — Showa Day (Sun)
    (2035,  4, 30), // 2035-04-30 — substitute for 2035-04-29 Showa Day
    (2035,  5,  3), // 2035-05-03 — Constitution Memorial Day
    (2035,  5,  4), // 2035-05-04 — Greenery Day
    (2035,  5,  5), // 2035-05-05 — Children's Day
    (2035,  7, 16), // 2035-07-16 — Marine Day (3rd Mon Jul)
    (2035,  8, 11), // 2035-08-11 — Mountain Day
    (2035,  9, 17), // 2035-09-17 — Respect for the Aged Day (3rd Mon Sep)
    // equinox: NAOJ astronomical inference (Aoki formula)
    (2035,  9, 23), // 2035-09-23 — Autumnal Equinox Day (Sun)
    (2035,  9, 24), // 2035-09-24 — substitute for 2035-09-23 Autumnal Equinox Day
    (2035, 10,  8), // 2035-10-08 — Health and Sports Day (2nd Mon Oct)
    (2035, 11,  3), // 2035-11-03 — Culture Day
    (2035, 11, 23), // 2035-11-23 — Labor Thanksgiving Day
    (2035, 12, 31), // 2035-12-31 — JPX year-end
    // ─── 2036 ────────────────────────────────────────────────────────────
    (2036,  1,  1), // 2036-01-01 — New Year's Day
    (2036,  1,  2), // 2036-01-02 — JPX year-end / New Year break
    (2036,  1,  3), // 2036-01-03 — JPX year-end / New Year break
    (2036,  1, 14), // 2036-01-14 — Coming of Age Day (2nd Mon Jan)
    (2036,  2, 11), // 2036-02-11 — National Foundation Day
    (2036,  2, 23), // 2036-02-23 — Emperor's Birthday
    // equinox: NAOJ astronomical inference (Aoki formula)
    (2036,  3, 20), // 2036-03-20 — Vernal Equinox Day
    (2036,  4, 29), // 2036-04-29 — Showa Day
    (2036,  5,  3), // 2036-05-03 — Constitution Memorial Day
    (2036,  5,  4), // 2036-05-04 — Greenery Day (Sun)
    (2036,  5,  5), // 2036-05-05 — Children's Day
    (2036,  5,  6), // 2036-05-06 — substitute for 2036-05-04 Greenery Day
    (2036,  7, 21), // 2036-07-21 — Marine Day (3rd Mon Jul)
    (2036,  8, 11), // 2036-08-11 — Mountain Day
    (2036,  9, 15), // 2036-09-15 — Respect for the Aged Day (3rd Mon Sep)
    // equinox: NAOJ astronomical inference (Aoki formula)
    (2036,  9, 22), // 2036-09-22 — Autumnal Equinox Day
    (2036, 10, 13), // 2036-10-13 — Health and Sports Day (2nd Mon Oct)
    (2036, 11,  3), // 2036-11-03 — Culture Day
    (2036, 11, 23), // 2036-11-23 — Labor Thanksgiving Day (Sun)
    (2036, 11, 24), // 2036-11-24 — substitute for 2036-11-23 Labor Thanksgiving Day
    (2036, 12, 31), // 2036-12-31 — JPX year-end
    // ─── 2037 ────────────────────────────────────────────────────────────
    (2037,  1,  1), // 2037-01-01 — New Year's Day
    (2037,  1,  2), // 2037-01-02 — JPX year-end / New Year break
    (2037,  1,  3), // 2037-01-03 — JPX year-end / New Year break
    (2037,  1, 12), // 2037-01-12 — Coming of Age Day (2nd Mon Jan)
    (2037,  2, 11), // 2037-02-11 — National Foundation Day
    (2037,  2, 23), // 2037-02-23 — Emperor's Birthday
    // equinox: NAOJ astronomical inference (Aoki formula)
    (2037,  3, 20), // 2037-03-20 — Vernal Equinox Day
    (2037,  4, 29), // 2037-04-29 — Showa Day
    (2037,  5,  3), // 2037-05-03 — Constitution Memorial Day (Sun)
    (2037,  5,  4), // 2037-05-04 — Greenery Day
    (2037,  5,  5), // 2037-05-05 — Children's Day
    (2037,  5,  6), // 2037-05-06 — substitute for 2037-05-03 Constitution Memorial Day
    (2037,  7, 20), // 2037-07-20 — Marine Day (3rd Mon Jul)
    (2037,  8, 11), // 2037-08-11 — Mountain Day
    (2037,  9, 21), // 2037-09-21 — Respect for the Aged Day (3rd Mon Sep)
    (2037,  9, 22), // 2037-09-22 — People's Holiday (sandwiched between Respect & Equinox)
    // equinox: NAOJ astronomical inference (Aoki formula)
    (2037,  9, 23), // 2037-09-23 — Autumnal Equinox Day
    (2037, 10, 12), // 2037-10-12 — Health and Sports Day (2nd Mon Oct)
    (2037, 11,  3), // 2037-11-03 — Culture Day
    (2037, 11, 23), // 2037-11-23 — Labor Thanksgiving Day
    (2037, 12, 31), // 2037-12-31 — JPX year-end
    // ─── 2038 ────────────────────────────────────────────────────────────
    (2038,  1,  1), // 2038-01-01 — New Year's Day
    (2038,  1,  2), // 2038-01-02 — JPX year-end / New Year break
    (2038,  1,  3), // 2038-01-03 — JPX year-end / New Year break
    (2038,  1, 11), // 2038-01-11 — Coming of Age Day (2nd Mon Jan)
    (2038,  2, 11), // 2038-02-11 — National Foundation Day
    (2038,  2, 23), // 2038-02-23 — Emperor's Birthday
    // equinox: NAOJ astronomical inference (Aoki formula)
    (2038,  3, 20), // 2038-03-20 — Vernal Equinox Day
    (2038,  4, 29), // 2038-04-29 — Showa Day
    (2038,  5,  3), // 2038-05-03 — Constitution Memorial Day
    (2038,  5,  4), // 2038-05-04 — Greenery Day
    (2038,  5,  5), // 2038-05-05 — Children's Day
    (2038,  7, 19), // 2038-07-19 — Marine Day (3rd Mon Jul)
    (2038,  8, 11), // 2038-08-11 — Mountain Day
    (2038,  9, 20), // 2038-09-20 — Respect for the Aged Day (3rd Mon Sep)
    // equinox: NAOJ astronomical inference (Aoki formula)
    (2038,  9, 23), // 2038-09-23 — Autumnal Equinox Day
    (2038, 10, 11), // 2038-10-11 — Health and Sports Day (2nd Mon Oct)
    (2038, 11,  3), // 2038-11-03 — Culture Day
    (2038, 11, 23), // 2038-11-23 — Labor Thanksgiving Day
    (2038, 12, 31), // 2038-12-31 — JPX year-end
    // ─── 2039 ────────────────────────────────────────────────────────────
    (2039,  1,  1), // 2039-01-01 — New Year's Day (Sun; substitute is JPX 2 Jan)
    (2039,  1,  2), // 2039-01-02 — JPX year-end / New Year break
    (2039,  1,  3), // 2039-01-03 — JPX year-end / New Year break
    (2039,  1, 10), // 2039-01-10 — Coming of Age Day (2nd Mon Jan)
    (2039,  2, 11), // 2039-02-11 — National Foundation Day
    (2039,  2, 23), // 2039-02-23 — Emperor's Birthday
    // equinox: NAOJ astronomical inference (Aoki formula)
    (2039,  3, 21), // 2039-03-21 — Vernal Equinox Day
    (2039,  4, 29), // 2039-04-29 — Showa Day
    (2039,  5,  3), // 2039-05-03 — Constitution Memorial Day
    (2039,  5,  4), // 2039-05-04 — Greenery Day
    (2039,  5,  5), // 2039-05-05 — Children's Day
    (2039,  7, 18), // 2039-07-18 — Marine Day (3rd Mon Jul)
    (2039,  8, 11), // 2039-08-11 — Mountain Day
    (2039,  9, 19), // 2039-09-19 — Respect for the Aged Day (3rd Mon Sep)
    // equinox: NAOJ astronomical inference (Aoki formula)
    (2039,  9, 23), // 2039-09-23 — Autumnal Equinox Day
    (2039, 10, 10), // 2039-10-10 — Health and Sports Day (2nd Mon Oct)
    (2039, 11,  3), // 2039-11-03 — Culture Day
    (2039, 11, 23), // 2039-11-23 — Labor Thanksgiving Day
    (2039, 12, 31), // 2039-12-31 — JPX year-end
    // ─── 2040 ────────────────────────────────────────────────────────────
    (2040,  1,  1), // 2040-01-01 — New Year's Day (Sun; substitute is JPX 2 Jan)
    (2040,  1,  2), // 2040-01-02 — JPX year-end / New Year break
    (2040,  1,  3), // 2040-01-03 — JPX year-end / New Year break
    (2040,  1,  9), // 2040-01-09 — Coming of Age Day (2nd Mon Jan)
    (2040,  2, 11), // 2040-02-11 — National Foundation Day
    (2040,  2, 23), // 2040-02-23 — Emperor's Birthday
    // equinox: NAOJ astronomical inference (Aoki formula)
    (2040,  3, 20), // 2040-03-20 — Vernal Equinox Day
    (2040,  4, 29), // 2040-04-29 — Showa Day (Sun)
    (2040,  4, 30), // 2040-04-30 — substitute for 2040-04-29 Showa Day
    (2040,  5,  3), // 2040-05-03 — Constitution Memorial Day
    (2040,  5,  4), // 2040-05-04 — Greenery Day
    (2040,  5,  5), // 2040-05-05 — Children's Day
    (2040,  7, 16), // 2040-07-16 — Marine Day (3rd Mon Jul)
    (2040,  8, 11), // 2040-08-11 — Mountain Day
    (2040,  9, 17), // 2040-09-17 — Respect for the Aged Day (3rd Mon Sep)
    // equinox: NAOJ astronomical inference (Aoki formula)
    (2040,  9, 22), // 2040-09-22 — Autumnal Equinox Day
    (2040, 10,  8), // 2040-10-08 — Health and Sports Day (2nd Mon Oct)
    (2040, 11,  3), // 2040-11-03 — Culture Day
    (2040, 11, 23), // 2040-11-23 — Labor Thanksgiving Day
    (2040, 12, 31), // 2040-12-31 — JPX year-end
];

/// True if `date` is a Japan / JPX trading-calendar holiday in this
/// snapshot's coverage window.
///
/// The lookup is a binary search over [`HOLIDAYS`], which is sorted
/// ascending. Dates outside [`COVERAGE`] return `false`: the snapshot
/// has nothing to say about them, and a `false` for an unknown date is
/// the safe failure mode (the caller is responsible for refusing to
/// settle a cashflow beyond this horizon against the Japan calendar).
///
/// Weekends are not classified as holidays here — they are handled by
/// the caller's business-day predicate (see the `is_business_day`
/// dispatcher in [`crate::calendar`]).
///
/// # Examples
///
/// ```
/// use regit_daycount::Date;
/// use regit_daycount::calendar::japan;
///
/// // New Year's Day 2024 is a JPX holiday.
/// assert!(japan::is_holiday(Date::ymd(2024, 1, 1).unwrap()));
///
/// // 2024-01-04 (Thursday) is the first trading day of 2024 — not a holiday.
/// assert!(!japan::is_holiday(Date::ymd(2024, 1, 4).unwrap()));
/// ```
#[must_use]
pub fn is_holiday(date: Date) -> bool {
    let key = (date.year(), date.month(), date.day());
    HOLIDAYS.binary_search(&key).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── 2024 — every published JPX holiday in the year ──────────────────

    #[test]
    fn holidays_2024_published_jpx_calendar() {
        // Hand-verified against the JPX 2024 published trading calendar.
        let dates = [
            (1, 1),   // New Year's Day
            (1, 2),   // JPX year-end
            (1, 3),   // JPX year-end
            (1, 8),   // Coming of Age (2nd Mon Jan)
            (2, 11),  // National Foundation Day (Sun)
            (2, 12),  // substitute for 2024-02-11
            (2, 23),  // Emperor's Birthday (Fri)
            (3, 20),  // Vernal Equinox
            (4, 29),  // Showa Day
            (5, 3),   // Constitution Memorial Day
            (5, 6),   // substitute for 2024-05-05 Children's Day (Sun)
            (7, 15),  // Marine Day (3rd Mon Jul)
            (8, 12),  // substitute for 2024-08-11 Mountain Day (Sun)
            (9, 16),  // Respect for the Aged (3rd Mon Sep)
            (9, 22),  // Autumnal Equinox (Sun)
            (9, 23),  // substitute for 2024-09-22
            (10, 14), // Health and Sports Day (2nd Mon Oct)
            (11, 4),  // substitute for 2024-11-03 Culture Day (Sun)
            (11, 23), // Labor Thanksgiving
            (12, 31), // JPX year-end
        ];
        for (m, d) in dates {
            assert!(
                is_holiday(Date::ymd(2024, m, d).unwrap()),
                "2024-{m:02}-{d:02} must be a Japan holiday",
            );
        }
    }

    #[test]
    fn non_holidays_2024_adjacent_dates() {
        // The first trading day of 2024 is Thursday 4 January.
        assert!(!is_holiday(Date::ymd(2024, 1, 4).unwrap()));
        // Friday 5 January 2024 is a regular trading day.
        assert!(!is_holiday(Date::ymd(2024, 1, 5).unwrap()));
        // 2024-05-02 (Thu) — between Showa Day (29 Apr Mon) and
        // Constitution Memorial Day (3 May Fri). Not itself a holiday.
        assert!(!is_holiday(Date::ymd(2024, 5, 2).unwrap()));
    }

    // ─── Olympic-year specials (2020, 2021) ──────────────────────────────

    #[test]
    fn olympic_year_2020_specials() {
        // Marine, Sports, Mountain moved to surround the (postponed)
        // Tokyo 2020 ceremonies.
        assert!(is_holiday(Date::ymd(2020, 7, 23).unwrap()), "Marine 2020");
        assert!(is_holiday(Date::ymd(2020, 7, 24).unwrap()), "Sports 2020");
        assert!(is_holiday(Date::ymd(2020, 8, 10).unwrap()), "Mountain 2020");
        // And the original 3rd Mon Jul / 2nd Mon Oct / 11 Aug are NOT
        // holidays in 2020.
        assert!(!is_holiday(Date::ymd(2020, 7, 20).unwrap())); // 3rd Mon Jul 2020
        assert!(!is_holiday(Date::ymd(2020, 10, 12).unwrap())); // 2nd Mon Oct 2020
        assert!(!is_holiday(Date::ymd(2020, 8, 11).unwrap())); // 11 Aug 2020
    }

    #[test]
    fn olympic_year_2021_specials() {
        assert!(is_holiday(Date::ymd(2021, 7, 22).unwrap()), "Marine 2021");
        assert!(is_holiday(Date::ymd(2021, 7, 23).unwrap()), "Sports 2021");
        assert!(is_holiday(Date::ymd(2021, 8, 8).unwrap()), "Mountain 2021");
        // 2021-08-08 is a Sunday, so 2021-08-09 is the substitute Monday.
        assert!(
            is_holiday(Date::ymd(2021, 8, 9).unwrap()),
            "Mountain sub 2021"
        );
        // Original 3rd Mon Jul / 2nd Mon Oct / 11 Aug are NOT holidays.
        assert!(!is_holiday(Date::ymd(2021, 7, 19).unwrap())); // 3rd Mon Jul 2021
        assert!(!is_holiday(Date::ymd(2021, 10, 11).unwrap())); // 2nd Mon Oct 2021
        assert!(!is_holiday(Date::ymd(2021, 8, 11).unwrap())); // 11 Aug 2021
    }

    // ─── Emperor's Birthday change ───────────────────────────────────────

    #[test]
    fn emperors_birthday_after_2020_change() {
        // The previous emperor's birthday (23 December) is NOT a Japan
        // holiday in this table (and 2018-12-23 is also out of coverage).
        // 2020-02-23 (the new emperor's) was a Sunday — the substitute
        // Monday 2020-02-24 is in the table.
        assert!(is_holiday(Date::ymd(2020, 2, 23).unwrap()));
        assert!(is_holiday(Date::ymd(2020, 2, 24).unwrap()));
        // 2024-02-23 (Fri) is the Emperor's Birthday — directly observed.
        assert!(is_holiday(Date::ymd(2024, 2, 23).unwrap()));
        // The previous emperor's birthday is not in any 2020+ year.
        assert!(!is_holiday(Date::ymd(2020, 12, 23).unwrap()));
        assert!(!is_holiday(Date::ymd(2024, 12, 23).unwrap()));
    }

    // ─── Coverage boundaries ─────────────────────────────────────────────

    #[test]
    fn out_of_coverage_returns_false() {
        // 2019-01-01 was a real Japanese holiday but predates this
        // snapshot's coverage.
        assert!(!is_holiday(Date::ymd(2019, 1, 1).unwrap()));
        // 2041-01-01 lies past this snapshot's coverage.
        assert!(!is_holiday(Date::ymd(2041, 1, 1).unwrap()));
    }

    // ─── Sort invariant ──────────────────────────────────────────────────

    #[test]
    fn holidays_table_is_sorted_ascending() {
        // The binary search in `is_holiday` depends on this invariant.
        for window in HOLIDAYS.windows(2) {
            let a = window[0];
            let b = window[1];
            assert!(a < b, "HOLIDAYS not strictly ascending: {a:?} >= {b:?}");
        }
    }

    // ─── Snapshot metadata ───────────────────────────────────────────────

    #[test]
    fn coverage_range_is_2020_to_2040() {
        assert_eq!(COVERAGE, (2020, 2040));
        // First and last entries lie inside the declared range.
        let first = HOLIDAYS[0];
        let last = HOLIDAYS[HOLIDAYS.len() - 1];
        assert!(first.0 >= COVERAGE.0);
        assert!(last.0 <= COVERAGE.1);
    }

    #[test]
    fn snapshot_date_is_iso_yyyymmdd() {
        // Trivial shape check — keeps the constant honest.
        assert_eq!(SNAPSHOT_DATE.len(), 10);
        assert_eq!(&SNAPSHOT_DATE[4..5], "-");
        assert_eq!(&SNAPSHOT_DATE[7..8], "-");
    }

    // ─── People's Holiday rule ───────────────────────────────────────────

    #[test]
    fn peoples_holiday_fires_when_respect_aged_and_equinox_sandwich() {
        // The rule fires when Respect for the Aged Day (3rd Mon Sep)
        // is 21 Sep AND the autumnal equinox is 23 Sep, leaving 22 Sep
        // as a non-holiday weekday between two holidays. Within
        // coverage 2020–2040 that occurs in 2026, 2032, and 2037.
        assert!(is_holiday(Date::ymd(2026, 9, 22).unwrap()));
        assert!(is_holiday(Date::ymd(2032, 9, 21).unwrap()));
        assert!(is_holiday(Date::ymd(2037, 9, 22).unwrap()));
    }
}
