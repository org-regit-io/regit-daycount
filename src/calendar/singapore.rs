// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! Singapore — SGX (Singapore Exchange) trading-calendar snapshot.
//!
//! Singapore's public-holiday schedule reflects the country's plural
//! religious composition: a mixture of fixed Western / civil dates, two
//! Chinese lunar dates (Chinese New Year), one Easter-derived date (Good
//! Friday), one Buddhist lunar date (Vesak Day), two Islamic / Hijri
//! dates (Hari Raya Puasa and Hari Raya Haji), and one Hindu lunar date
//! (Deepavali). None of the religious dates admits a closed-form rule the
//! way Easter does — the Chinese, Buddhist, Islamic, and Hindu calendars
//! depend on astronomical and ecclesiastical determinations that are
//! published year-by-year by the Singapore Ministry of Manpower (MOM) in
//! the Government gazette and mirrored by the Singapore Exchange (SGX) in
//! its annual trading-calendar PDF. This module therefore stores the
//! gazetted dates as a dated snapshot rather than computing them.
//!
//! When a gazetted holiday falls on a Sunday the Singapore Holidays Act
//! 1998 §4 grants the following Monday as a public holiday in lieu (the
//! *observed* day). The table reflects the **observed** SGX trading-
//! closure date — the Sunday rule date itself is not separately listed,
//! because SGX is already closed on Sunday and the gazette directs the
//! market-closure to the in-lieu Monday. Saturday holidays are *not*
//! shifted: §4 names only Sunday as the trigger for an in-lieu Monday,
//! and SGX is already closed on Saturday. Chinese New Year is gazetted as
//! two consecutive days (1st and 2nd of the lunar year); when day 1 lands
//! on a Sunday the in-lieu rule applies to the Sunday and day 2 stays on
//! its Monday, so the two SGX closures become Monday + Tuesday; when day
//! 2 lands on a Sunday only that second day shifts to the following
//! Monday.
//!
//! # Holiday rules
//!
//! ```text
//! New Year's Day        1 January           (observed Mon if Sun)    fixed
//! Chinese New Year      lunar day 1 + day 2 (Mon-in-lieu if needed)  lunar
//! Good Friday           Easter Sunday − 2                            Easter
//! Labour Day            1 May               (observed Mon if Sun)    fixed
//! Vesak Day             full-moon of Vaisakha (Buddhist lunar)       lunar
//! Hari Raya Puasa       1 Shawwal (Islamic / Hijri)                  Islamic
//! National Day          9 August            (observed Mon if Sun)    fixed
//! Hari Raya Haji        10 Dhu al-Hijjah (Islamic / Hijri)           Islamic
//! Deepavali             new-moon Hindu calendar (Oct / Nov)          Hindu
//! Christmas Day         25 December         (observed Mon if Sun)    fixed
//! ```
//!
//! # Multi-faith calendar note
//!
//! Singapore's public-holiday list is one of the most religiously diverse
//! of any major financial centre: it ties four religious traditions —
//! Christian (Good Friday, Christmas), Buddhist (Vesak), Islamic (Hari
//! Raya Puasa, Hari Raya Haji), and Hindu (Deepavali) — into the same
//! market-closure schedule, alongside two Chinese lunar dates (Chinese
//! New Year) and the civil quartet of New Year's Day, Labour Day,
//! National Day, and Christmas. There is no algorithmic shortcut for the
//! religious dates; each must be transcribed from MOM's annual
//! Government-gazette notice and cross-checked against SGX's published
//! trading-calendar PDF for the same year.
//!
//! Because the Islamic year is roughly 354 days, Hari Raya Puasa and
//! Hari Raya Haji drift backwards through the Gregorian calendar by
//! about 10–11 days per year. Once every ~33 Gregorian years two
//! instances of the same Islamic festival fall inside one Gregorian
//! year; the table lists both rows when that happens (see 2033 for two
//! Hari Raya Puasa, 2039 for two Hari Raya Haji). The Islamic drift also
//! produces calendar collisions with Chinese New Year roughly once per
//! decade — when 1 Shawwal lands on the same day as a Chinese New Year
//! closure, only one trading-day closure is gazetted; the row carries a
//! `collision` note in its comment (see 2029, 2030, 2031).
//!
//! # Verification
//!
//! - Verification date: 2026-05-23.
//! - Years **2020–2026** are verified against the Singapore Ministry of
//!   Manpower gazetted public-holiday notice for the relevant year
//!   (`https://www.mom.gov.sg/employment-practices/public-holidays`),
//!   cross-checked against the Singapore Exchange annual trading-
//!   calendar PDF (`https://www.sgx.com/securities/trading-hours-
//!   calendar`).
//! - Years **2027–2040** are past MOM's publication horizon. Their
//!   dates are inferred from the deterministic rules and from
//!   astronomical / religious-calendar projection tables, namely:
//!   * Easter Sunday from the Gregorian computus, then Good Friday =
//!     Easter − 2 days;
//!   * Chinese New Year days 1 and 2 from the published astronomical
//!     lunar new year tables maintained by the Hong Kong Observatory
//!     and mirrored on the Wikipedia *Chinese New Year* article;
//!   * Vesak Day from the published Theravada full-moon-of-Vesakha
//!     tables used by the Buddhist Calendar Council, as mirrored on
//!     the Wikipedia *Vesak* article;
//!   * Hari Raya Puasa (1 Shawwal) and Hari Raya Haji (10 Dhu al-
//!     Hijjah) from the Umm al-Qura Hijri-to-Gregorian conversion
//!     tables, adjusted ±1 day where the Singapore Islamic Religious
//!     Council (MUIS) historically diverges from the Umm al-Qura
//!     announcement by a local-visibility margin;
//!   * Deepavali from the Hindu Kartik-Amavasya astronomical tables
//!     mirrored on the Wikipedia *Diwali* article.
//!
//!   The Holidays Act 1998 §4 Monday-in-lieu rule is then applied to
//!   any of the above that lands on a Sunday. Saturday-falling
//!   holidays are listed on the Saturday with a "(Sat — not shifted)"
//!   note in the row comment, consistent with §4's Sunday-only
//!   trigger.
//! - The 2027–2040 rows must be re-verified against the corresponding
//!   year's MOM gazette as it is published (typically 1–2 calendar
//!   years ahead of the holiday year) before the user relies on a row
//!   for a real trade-date computation. The snapshot date in
//!   [`SNAPSHOT_DATE`] records when the inference was last refreshed.
//!
//! # Snapshot
//!
//! - Snapshot date: [`SNAPSHOT_DATE`] = `"2026-05-23"`.
//! - Coverage:     [`COVERAGE`]      = `(2020, 2040)` inclusive.
//! - Out-of-coverage dates return `false` from [`is_holiday`] without
//!   panicking — the caller is expected to refresh the snapshot before
//!   classifying dates outside the window.
//!
//! # References
//!
//! - Singapore Ministry of Manpower, *Public Holidays in Singapore*
//!   (annual Government-gazette notice under the Holidays Act).
//! - Singapore Exchange (SGX), *SGX Securities and Derivatives Trading
//!   Calendar* (annual PDF), published at <https://www.sgx.com>.
//! - Holidays Act 1998 (Singapore), §4 — Monday-in-lieu rule for
//!   gazetted public holidays falling on a Sunday.

use crate::date::Date;

/// Date of the SGX trading-calendar snapshot embedded in this module,
/// `YYYY-MM-DD`.
pub const SNAPSHOT_DATE: &str = "2026-05-23";

/// Inclusive `(first_year, last_year)` coverage of [`HOLIDAYS`]. Dates
/// outside this window cause [`is_holiday`] to return `false` without
/// consulting the table.
pub const COVERAGE: (i32, i32) = (2020, 2040);

/// SGX trading-calendar holidays, sorted ascending by `(year, month,
/// day)`. Each row pairs the observed market-closure date with a comment
/// naming the holiday it implements.
///
/// The list reflects the *observed* closure (i.e. with the Monday-in-lieu
/// rule already applied for Sunday-falling fixed and religious dates);
/// the underlying rule date is named in the row comment when the two
/// differ. Years 2020–2026 are gazetted; years 2027–2040 are projected
/// per the module-level *Verification* note and must be re-verified
/// against the MOM gazette as it is published.
#[rustfmt::skip]
pub const HOLIDAYS: &[(i32, u8, u8)] = &[
    // ─── 2020 ────────────────────────────────────────────────────────
    (2020,  1,  1), // 2020-01-01 — New Year's Day
    (2020,  1, 25), // 2020-01-25 — Chinese New Year day 1 (Sat)
    (2020,  1, 27), // 2020-01-27 — Chinese New Year day 2 obs (Sun → Mon)
    (2020,  4, 10), // 2020-04-10 — Good Friday
    (2020,  5,  1), // 2020-05-01 — Labour Day
    (2020,  5,  7), // 2020-05-07 — Vesak Day
    (2020,  5, 25), // 2020-05-25 — Hari Raya Puasa obs (Sun 24 → Mon)
    (2020,  7, 31), // 2020-07-31 — Hari Raya Haji
    (2020,  8, 10), // 2020-08-10 — National Day obs (Sun 9 → Mon)
    (2020, 11, 14), // 2020-11-14 — Deepavali (Sat — not shifted)
    (2020, 12, 25), // 2020-12-25 — Christmas Day

    // ─── 2021 ────────────────────────────────────────────────────────
    (2021,  1,  1), // 2021-01-01 — New Year's Day
    (2021,  2, 12), // 2021-02-12 — Chinese New Year day 1
    (2021,  2, 13), // 2021-02-13 — Chinese New Year day 2 (Sat — not shifted)
    (2021,  4,  2), // 2021-04-02 — Good Friday
    (2021,  5,  1), // 2021-05-01 — Labour Day (Sat — not shifted)
    (2021,  5, 13), // 2021-05-13 — Hari Raya Puasa
    (2021,  5, 26), // 2021-05-26 — Vesak Day
    (2021,  7, 20), // 2021-07-20 — Hari Raya Haji
    (2021,  8,  9), // 2021-08-09 — National Day
    (2021, 11,  4), // 2021-11-04 — Deepavali
    (2021, 12, 25), // 2021-12-25 — Christmas Day (Sat — not shifted)

    // ─── 2022 ────────────────────────────────────────────────────────
    (2022,  1,  1), // 2022-01-01 — New Year's Day (Sat — not shifted)
    (2022,  2,  1), // 2022-02-01 — Chinese New Year day 1
    (2022,  2,  2), // 2022-02-02 — Chinese New Year day 2
    (2022,  4, 15), // 2022-04-15 — Good Friday
    (2022,  5,  2), // 2022-05-02 — Labour Day obs (Sun 1 → Mon)
    (2022,  5,  3), // 2022-05-03 — Hari Raya Puasa
    (2022,  5, 16), // 2022-05-16 — Vesak Day obs (Sun 15 → Mon)
    (2022,  7, 11), // 2022-07-11 — Hari Raya Haji obs (Sun 10 → Mon)
    (2022,  8,  9), // 2022-08-09 — National Day
    (2022, 10, 24), // 2022-10-24 — Deepavali
    (2022, 12, 26), // 2022-12-26 — Christmas Day obs (Sun 25 → Mon)

    // ─── 2023 ────────────────────────────────────────────────────────
    (2023,  1,  2), // 2023-01-02 — New Year's Day obs (Sun 1 → Mon)
    (2023,  1, 23), // 2023-01-23 — Chinese New Year day 2 gazetted (day 1 = Sun 22)
    (2023,  1, 24), // 2023-01-24 — Chinese New Year day 1 obs (Sun 22 → Tue in-lieu)
    (2023,  4,  7), // 2023-04-07 — Good Friday
    (2023,  4, 22), // 2023-04-22 — Hari Raya Puasa (Sat — not shifted)
    (2023,  5,  1), // 2023-05-01 — Labour Day
    (2023,  6,  2), // 2023-06-02 — Vesak Day
    (2023,  6, 29), // 2023-06-29 — Hari Raya Haji
    (2023,  8,  9), // 2023-08-09 — National Day
    (2023, 11, 13), // 2023-11-13 — Deepavali obs (Sun 12 → Mon)
    (2023, 12, 25), // 2023-12-25 — Christmas Day

    // ─── 2024 ────────────────────────────────────────────────────────
    (2024,  1,  1), // 2024-01-01 — New Year's Day
    (2024,  2, 10), // 2024-02-10 — Chinese New Year day 1 (Sat)
    (2024,  2, 12), // 2024-02-12 — Chinese New Year day 2 obs (Sun 11 → Mon)
    (2024,  3, 29), // 2024-03-29 — Good Friday
    (2024,  4, 10), // 2024-04-10 — Hari Raya Puasa
    (2024,  5,  1), // 2024-05-01 — Labour Day
    (2024,  5, 22), // 2024-05-22 — Vesak Day
    (2024,  6, 17), // 2024-06-17 — Hari Raya Haji
    (2024,  8,  9), // 2024-08-09 — National Day
    (2024, 10, 31), // 2024-10-31 — Deepavali
    (2024, 12, 25), // 2024-12-25 — Christmas Day

    // ─── 2025 ────────────────────────────────────────────────────────
    (2025,  1,  1), // 2025-01-01 — New Year's Day
    (2025,  1, 29), // 2025-01-29 — Chinese New Year day 1
    (2025,  1, 30), // 2025-01-30 — Chinese New Year day 2
    (2025,  3, 31), // 2025-03-31 — Hari Raya Puasa
    (2025,  4, 18), // 2025-04-18 — Good Friday
    (2025,  5,  1), // 2025-05-01 — Labour Day
    (2025,  5, 12), // 2025-05-12 — Vesak Day
    (2025,  6,  7), // 2025-06-07 — Hari Raya Haji (Sat — not shifted)
    (2025,  8,  9), // 2025-08-09 — National Day (Sat — not shifted)
    (2025, 10, 20), // 2025-10-20 — Deepavali
    (2025, 12, 25), // 2025-12-25 — Christmas Day

    // ─── 2026 ────────────────────────────────────────────────────────
    (2026,  1,  1), // 2026-01-01 — New Year's Day
    (2026,  2, 17), // 2026-02-17 — Chinese New Year day 1
    (2026,  2, 18), // 2026-02-18 — Chinese New Year day 2
    (2026,  3, 20), // 2026-03-20 — Hari Raya Puasa
    (2026,  4,  3), // 2026-04-03 — Good Friday
    (2026,  5,  1), // 2026-05-01 — Labour Day
    (2026,  5, 27), // 2026-05-27 — Hari Raya Haji
    (2026,  6,  1), // 2026-06-01 — Vesak Day obs (Sun May 31 → Mon)
    (2026,  8, 10), // 2026-08-10 — National Day obs (Sun 9 → Mon)
    (2026, 11,  9), // 2026-11-09 — Deepavali obs (Sun 8 → Mon)
    (2026, 12, 25), // 2026-12-25 — Christmas Day

    // ─── 2027 ────────────────────────────────────────────────────────
    (2027,  1,  1), // 2027-01-01 — New Year's Day
    (2027,  2,  6), // 2027-02-06 — Chinese New Year day 1 (Sat — not shifted)
    (2027,  2,  8), // 2027-02-08 — Chinese New Year day 2 obs (Sun 7 → Mon)
    (2027,  3, 10), // 2027-03-10 — Hari Raya Puasa
    (2027,  3, 26), // 2027-03-26 — Good Friday
    (2027,  5,  1), // 2027-05-01 — Labour Day (Sat — not shifted)
    (2027,  5, 17), // 2027-05-17 — Hari Raya Haji
    (2027,  5, 20), // 2027-05-20 — Vesak Day
    (2027,  8,  9), // 2027-08-09 — National Day
    (2027, 10, 28), // 2027-10-28 — Deepavali
    (2027, 12, 25), // 2027-12-25 — Christmas Day (Sat — not shifted)

    // ─── 2028 ────────────────────────────────────────────────────────
    (2028,  1,  1), // 2028-01-01 — New Year's Day (Sat — not shifted)
    (2028,  1, 26), // 2028-01-26 — Chinese New Year day 1
    (2028,  1, 27), // 2028-01-27 — Chinese New Year day 2
    (2028,  2, 26), // 2028-02-26 — Hari Raya Puasa (Sat — not shifted)
    (2028,  4, 14), // 2028-04-14 — Good Friday
    (2028,  5,  1), // 2028-05-01 — Labour Day
    (2028,  5,  5), // 2028-05-05 — Hari Raya Haji
    (2028,  5,  9), // 2028-05-09 — Vesak Day
    (2028,  8,  9), // 2028-08-09 — National Day
    (2028, 11, 14), // 2028-11-14 — Deepavali
    (2028, 12, 25), // 2028-12-25 — Christmas Day

    // ─── 2029 ────────────────────────────────────────────────────────
    (2029,  1,  1), // 2029-01-01 — New Year's Day
    (2029,  2, 13), // 2029-02-13 — Chinese New Year day 1
    (2029,  2, 14), // 2029-02-14 — Chinese New Year day 2 (also Hari Raya Puasa — collision)
    (2029,  3, 30), // 2029-03-30 — Good Friday
    (2029,  4, 24), // 2029-04-24 — Hari Raya Haji
    (2029,  5,  1), // 2029-05-01 — Labour Day
    (2029,  5, 28), // 2029-05-28 — Vesak Day obs (Sun 27 → Mon)
    (2029,  8,  9), // 2029-08-09 — National Day
    (2029, 11,  5), // 2029-11-05 — Deepavali
    (2029, 12, 25), // 2029-12-25 — Christmas Day

    // ─── 2030 ────────────────────────────────────────────────────────
    (2030,  1,  1), // 2030-01-01 — New Year's Day
    (2030,  2,  4), // 2030-02-04 — Chinese New Year day 2 gazetted (day 1 = Sun 3, also Hari Raya Puasa — collision)
    (2030,  2,  5), // 2030-02-05 — Chinese New Year day 1 obs (Sun 3 → Tue in-lieu)
    (2030,  4, 14), // 2030-04-14 — Hari Raya Haji
    (2030,  4, 19), // 2030-04-19 — Good Friday
    (2030,  5,  1), // 2030-05-01 — Labour Day
    (2030,  5, 17), // 2030-05-17 — Vesak Day
    (2030,  8,  9), // 2030-08-09 — National Day
    (2030, 10, 26), // 2030-10-26 — Deepavali (Sat — not shifted)
    (2030, 12, 25), // 2030-12-25 — Christmas Day

    // ─── 2031 ────────────────────────────────────────────────────────
    (2031,  1,  1), // 2031-01-01 — New Year's Day
    (2031,  1, 23), // 2031-01-23 — Chinese New Year day 1
    (2031,  1, 24), // 2031-01-24 — Chinese New Year day 2 (also Hari Raya Puasa — collision)
    (2031,  4,  4), // 2031-04-04 — Hari Raya Haji
    (2031,  4, 11), // 2031-04-11 — Good Friday
    (2031,  5,  1), // 2031-05-01 — Labour Day
    (2031,  5,  6), // 2031-05-06 — Vesak Day
    (2031,  8,  9), // 2031-08-09 — National Day (Sat — not shifted)
    (2031, 11, 14), // 2031-11-14 — Deepavali
    (2031, 12, 25), // 2031-12-25 — Christmas Day

    // ─── 2032 ────────────────────────────────────────────────────────
    (2032,  1,  1), // 2032-01-01 — New Year's Day
    (2032,  1, 13), // 2032-01-13 — Hari Raya Puasa
    (2032,  2, 11), // 2032-02-11 — Chinese New Year day 1
    (2032,  2, 12), // 2032-02-12 — Chinese New Year day 2
    (2032,  3, 22), // 2032-03-22 — Hari Raya Haji
    (2032,  3, 26), // 2032-03-26 — Good Friday
    (2032,  4, 24), // 2032-04-24 — Vesak Day (Sat — not shifted)
    (2032,  5,  1), // 2032-05-01 — Labour Day (Sat — not shifted)
    (2032,  8,  9), // 2032-08-09 — National Day
    (2032, 11,  1), // 2032-11-01 — Deepavali
    (2032, 12, 25), // 2032-12-25 — Christmas Day (Sat — not shifted)

    // ─── 2033 ────────────────────────────────────────────────────────
    (2033,  1,  1), // 2033-01-01 — New Year's Day (Sat — not shifted)
    (2033,  1,  3), // 2033-01-03 — Hari Raya Puasa obs (Sun 2 → Mon)
    (2033,  1, 31), // 2033-01-31 — Chinese New Year day 1
    (2033,  2,  1), // 2033-02-01 — Chinese New Year day 2
    (2033,  3, 11), // 2033-03-11 — Hari Raya Haji
    (2033,  4, 15), // 2033-04-15 — Good Friday
    (2033,  5,  2), // 2033-05-02 — Labour Day obs (Sun 1 → Mon)
    (2033,  5, 13), // 2033-05-13 — Vesak Day
    (2033,  8,  9), // 2033-08-09 — National Day
    (2033, 10, 21), // 2033-10-21 — Deepavali
    (2033, 12, 23), // 2033-12-23 — Hari Raya Puasa (2nd in Gregorian year — Islamic-year roll-over)
    (2033, 12, 26), // 2033-12-26 — Christmas Day obs (Sun 25 → Mon)

    // ─── 2034 ────────────────────────────────────────────────────────
    (2034,  1,  2), // 2034-01-02 — New Year's Day obs (Sun 1 → Mon)
    (2034,  2, 20), // 2034-02-20 — Chinese New Year day 2 gazetted (day 1 = Sun 19)
    (2034,  2, 21), // 2034-02-21 — Chinese New Year day 1 obs (Sun 19 → Tue in-lieu)
    (2034,  3,  1), // 2034-03-01 — Hari Raya Haji
    (2034,  4,  7), // 2034-04-07 — Good Friday
    (2034,  5,  1), // 2034-05-01 — Labour Day
    (2034,  5,  3), // 2034-05-03 — Vesak Day
    (2034,  8,  9), // 2034-08-09 — National Day
    (2034, 11,  9), // 2034-11-09 — Deepavali
    (2034, 12, 12), // 2034-12-12 — Hari Raya Puasa
    (2034, 12, 25), // 2034-12-25 — Christmas Day

    // ─── 2035 ────────────────────────────────────────────────────────
    (2035,  1,  1), // 2035-01-01 — New Year's Day
    (2035,  2,  8), // 2035-02-08 — Chinese New Year day 1
    (2035,  2,  9), // 2035-02-09 — Chinese New Year day 2
    (2035,  2, 19), // 2035-02-19 — Hari Raya Haji
    (2035,  3, 23), // 2035-03-23 — Good Friday
    (2035,  5,  1), // 2035-05-01 — Labour Day
    (2035,  5, 22), // 2035-05-22 — Vesak Day
    (2035,  8,  9), // 2035-08-09 — National Day
    (2035, 10, 30), // 2035-10-30 — Deepavali
    (2035, 12,  3), // 2035-12-03 — Hari Raya Puasa obs (Sun 2 → Mon)
    (2035, 12, 25), // 2035-12-25 — Christmas Day

    // ─── 2036 ────────────────────────────────────────────────────────
    (2036,  1,  1), // 2036-01-01 — New Year's Day
    (2036,  1, 28), // 2036-01-28 — Chinese New Year day 1
    (2036,  1, 29), // 2036-01-29 — Chinese New Year day 2
    (2036,  2,  9), // 2036-02-09 — Hari Raya Haji (Sat — not shifted)
    (2036,  4, 11), // 2036-04-11 — Good Friday
    (2036,  5,  1), // 2036-05-01 — Labour Day
    (2036,  5, 10), // 2036-05-10 — Vesak Day (Sat — not shifted)
    (2036,  8,  9), // 2036-08-09 — National Day (Sat — not shifted)
    (2036, 11, 17), // 2036-11-17 — Deepavali
    (2036, 11, 21), // 2036-11-21 — Hari Raya Puasa
    (2036, 12, 25), // 2036-12-25 — Christmas Day

    // ─── 2037 ────────────────────────────────────────────────────────
    (2037,  1,  1), // 2037-01-01 — New Year's Day
    (2037,  1, 28), // 2037-01-28 — Hari Raya Haji
    (2037,  2, 16), // 2037-02-16 — Chinese New Year day 2 gazetted (day 1 = Sun 15)
    (2037,  2, 17), // 2037-02-17 — Chinese New Year day 1 obs (Sun 15 → Tue in-lieu)
    (2037,  4,  3), // 2037-04-03 — Good Friday
    (2037,  5,  1), // 2037-05-01 — Labour Day
    (2037,  5, 29), // 2037-05-29 — Vesak Day
    (2037,  8, 10), // 2037-08-10 — National Day obs (Sun 9 → Mon)
    (2037, 11,  6), // 2037-11-06 — Deepavali
    (2037, 11, 10), // 2037-11-10 — Hari Raya Puasa
    (2037, 12, 25), // 2037-12-25 — Christmas Day

    // ─── 2038 ────────────────────────────────────────────────────────
    (2038,  1,  1), // 2038-01-01 — New Year's Day
    (2038,  1, 18), // 2038-01-18 — Hari Raya Haji
    (2038,  2,  4), // 2038-02-04 — Chinese New Year day 1
    (2038,  2,  5), // 2038-02-05 — Chinese New Year day 2
    (2038,  4, 23), // 2038-04-23 — Good Friday
    (2038,  5,  1), // 2038-05-01 — Labour Day (Sat — not shifted)
    (2038,  5, 18), // 2038-05-18 — Vesak Day
    (2038,  8,  9), // 2038-08-09 — National Day
    (2038, 10, 27), // 2038-10-27 — Deepavali
    (2038, 10, 30), // 2038-10-30 — Hari Raya Puasa (Sat — not shifted)
    (2038, 12, 25), // 2038-12-25 — Christmas Day (Sat — not shifted)

    // ─── 2039 ────────────────────────────────────────────────────────
    (2039,  1,  1), // 2039-01-01 — New Year's Day (Sat — not shifted)
    (2039,  1,  7), // 2039-01-07 — Hari Raya Haji
    (2039,  1, 24), // 2039-01-24 — Chinese New Year day 1
    (2039,  1, 25), // 2039-01-25 — Chinese New Year day 2
    (2039,  4,  8), // 2039-04-08 — Good Friday
    (2039,  5,  2), // 2039-05-02 — Labour Day obs (Sun 1 → Mon)
    (2039,  5,  7), // 2039-05-07 — Vesak Day (Sat — not shifted)
    (2039,  8,  9), // 2039-08-09 — National Day
    (2039, 10, 19), // 2039-10-19 — Hari Raya Puasa
    (2039, 11, 14), // 2039-11-14 — Deepavali
    (2039, 12, 26), // 2039-12-26 — Christmas Day obs (Sun 25 → Mon)
    (2039, 12, 27), // 2039-12-27 — Hari Raya Haji (2nd in Gregorian year — Islamic-year roll-over)

    // ─── 2040 ────────────────────────────────────────────────────────
    (2040,  1,  2), // 2040-01-02 — New Year's Day obs (Sun 1 → Mon)
    (2040,  2, 13), // 2040-02-13 — Chinese New Year day 2 gazetted (day 1 = Sun 12)
    (2040,  2, 14), // 2040-02-14 — Chinese New Year day 1 obs (Sun 12 → Tue in-lieu)
    (2040,  3, 30), // 2040-03-30 — Good Friday
    (2040,  5,  1), // 2040-05-01 — Labour Day
    (2040,  5, 25), // 2040-05-25 — Vesak Day
    (2040,  8,  9), // 2040-08-09 — National Day
    (2040, 10,  8), // 2040-10-08 — Hari Raya Puasa
    (2040, 11,  5), // 2040-11-05 — Deepavali obs (Sun 4 → Mon)
    (2040, 12, 17), // 2040-12-17 — Hari Raya Haji
    (2040, 12, 25), // 2040-12-25 — Christmas Day
];

/// True if `date` is an SGX-published trading-calendar holiday inside the
/// snapshot's [`COVERAGE`] window.
///
/// The lookup is a binary search over [`HOLIDAYS`], which is sorted
/// ascending by `(year, month, day)`. Dates outside [`COVERAGE`] return
/// `false` without consulting the table — refresh the snapshot before
/// classifying dates past the horizon. Weekends are *not* classified
/// here; SGX is already closed on Saturday and Sunday and that is the
/// caller's business-day predicate to express.
///
/// # Examples
///
/// ```
/// use regit_daycount::Date;
/// use regit_daycount::calendar::singapore;
///
/// // National Day 2024 (Friday) is an SGX holiday.
/// assert!(singapore::is_holiday(Date::ymd(2024, 8, 9).unwrap()));
///
/// // 2024-08-10 is a Saturday — weekends are not classified by
/// // `is_holiday`, only by the caller's business-day predicate.
/// assert!(!singapore::is_holiday(Date::ymd(2024, 8, 10).unwrap()));
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

    // ─── 2024 — every published SGX holiday in the year ──────────────

    #[test]
    fn holidays_2024() {
        // Every gazetted SGX trading-closure date for 2024. Verified
        // against the SGX 2024 trading-calendar PDF and the MOM 2024
        // public-holiday gazette.
        let dates = [
            (1, 1),   // New Year's Day (Mon)
            (2, 10),  // Chinese New Year day 1 (Sat)
            (2, 12),  // Chinese New Year day 2 observed (Sun 11 → Mon 12)
            (3, 29),  // Good Friday
            (4, 10),  // Hari Raya Puasa
            (5, 1),   // Labour Day (Wed)
            (5, 22),  // Vesak Day (Wed)
            (6, 17),  // Hari Raya Haji (Mon)
            (8, 9),   // National Day (Fri)
            (10, 31), // Deepavali (Thu)
            (12, 25), // Christmas Day (Wed)
        ];
        for (m, d) in dates {
            assert!(
                is_holiday(Date::ymd(2024, m, d).unwrap()),
                "2024-{m:02}-{d:02} must be an SGX holiday",
            );
        }
    }

    // ─── Out-of-coverage years return false ──────────────────────────

    #[test]
    fn out_of_coverage_returns_false() {
        // 2019-01-01 predates the snapshot window; 2041-01-01 is past
        // its horizon. Both must return `false` without panicking.
        assert!(!is_holiday(Date::ymd(2019, 1, 1).unwrap()));
        assert!(!is_holiday(Date::ymd(2041, 1, 1).unwrap()));
    }

    // ─── The HOLIDAYS table is sorted ascending ──────────────────────

    #[test]
    fn holidays_sorted_ascending() {
        // Binary search in `is_holiday` requires the table to be sorted
        // by `(year, month, day)` in strictly ascending order, with no
        // duplicate rows.
        for window in HOLIDAYS.windows(2) {
            let (a, b) = (window[0], window[1]);
            assert!(a < b, "HOLIDAYS not strictly ascending: {a:?} not < {b:?}");
        }
    }

    // ─── Random non-holiday inside coverage ──────────────────────────

    #[test]
    fn random_non_holiday() {
        // 2024-03-15 is a regular Friday with no SGX closure.
        assert!(!is_holiday(Date::ymd(2024, 3, 15).unwrap()));
    }

    // ─── Snapshot metadata ───────────────────────────────────────────

    #[test]
    fn snapshot_metadata_present() {
        assert_eq!(SNAPSHOT_DATE, "2026-05-23");
        assert_eq!(COVERAGE, (2020, 2040));
    }

    // ─── Coverage boundary years populated ───────────────────────────

    #[test]
    fn coverage_boundary_years_populated() {
        // Every year in the coverage window has at least one row in the
        // table — a basic sanity check that the snapshot is not silently
        // truncated at one end.
        let (first, last) = COVERAGE;
        for year in first..=last {
            assert!(
                HOLIDAYS.iter().any(|(y, _, _)| *y == year),
                "no SGX holiday rows for year {year}",
            );
        }
    }
}
