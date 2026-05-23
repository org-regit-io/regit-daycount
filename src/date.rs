// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! Gregorian-calendar date primitive.
//!
//! A [`Date`] is a `(year, month, day)` triple under the proleptic Gregorian
//! calendar — the calendar in use everywhere day-count fractions and holiday
//! tables apply. It is the only date type this crate uses; every day-count
//! fraction, date-roll convention, and calendar lookup operates on it.
//!
//! The representation is a `Copy`, three-field struct: a signed 32-bit year
//! and two `u8` fields. This is intentionally cheap to pass by value, cheap
//! to compare, and cheap to hash; the crate moves dates around freely.
//!
//! All arithmetic is grounded in **Howard Hinnant's `days_from_civil` /
//! `civil_from_days`** algorithm — a public-domain, branchless, integer-only
//! conversion between a `(year, month, day)` triple and a signed count of
//! days from the civil epoch 1970-01-01 — transcribed here directly. The
//! original derivation is at
//! <http://howardhinnant.github.io/date_algorithms.html>. The algorithm is
//! `no_std`-clean, allocation-free, and exact for every Gregorian date the
//! crate accepts.
//!
//! # References
//!
//! - ISO 8601, *Date and time — Representations for information
//!   interchange*, §3.4.1 (calendar date) and §3.4.2 (proleptic Gregorian
//!   calendar).
//! - Howard E. Hinnant, *chrono-Compatible Low-Level Date Algorithms*,
//!   <http://howardhinnant.github.io/date_algorithms.html> (public domain).

use crate::errors::ValidationError;

// ─── Weekday ─────────────────────────────────────────────────────────────────

/// A day of the week.
///
/// The variants are listed in the conventional Monday-first order used by
/// ISO 8601 (`Monday = 1`, `Sunday = 7`). The discriminants are not part of
/// the public API; callers should match on the variant name.
///
/// # Examples
///
/// ```
/// use regit_daycount::{Date, Weekday};
///
/// // 2026-05-23 is a Saturday.
/// assert_eq!(Date::ymd(2026, 5, 23).unwrap().day_of_week(), Weekday::Sat);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Weekday {
    /// Monday.
    Mon,
    /// Tuesday.
    Tue,
    /// Wednesday.
    Wed,
    /// Thursday.
    Thu,
    /// Friday.
    Fri,
    /// Saturday.
    Sat,
    /// Sunday.
    Sun,
}

// ─── Date ────────────────────────────────────────────────────────────────────

/// A Gregorian-calendar date — `(year, month, day)`.
///
/// A `Date` is created by [`Date::ymd`] (validating) or
/// [`Date::ymd_unchecked`] (caller asserts validity); the field layout is
/// private so the invariant that the triple names a real date cannot be
/// violated by direct construction. The type is `Copy` and allocates
/// nothing.
///
/// `PartialOrd` / `Ord` follow the natural chronological order — equivalent
/// to lexicographic order on `(year, month, day)`, which is the same thing
/// for Gregorian dates.
///
/// The accepted year range is `1583..=9999`: the Gregorian calendar took
/// effect in October 1582, and from 1583 onward every day-count and
/// holiday-calendar rule this crate implements is defined uniformly.
///
/// # Examples
///
/// ```
/// use regit_daycount::Date;
///
/// let d = Date::ymd(2026, 5, 23).unwrap();
/// assert_eq!(d.year(),  2026);
/// assert_eq!(d.month(), 5);
/// assert_eq!(d.day(),   23);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Date {
    /// The year, as a signed integer (proleptic Gregorian; year 0 exists).
    year: i32,
    /// The month, `1..=12`.
    month: u8,
    /// The day of the month, `1..=31` (further constrained by `month` and
    /// `year` — see [`Date::ymd`]).
    day: u8,
}

impl Date {
    /// Smallest accepted year (1583 — the first full year after the
    /// Gregorian reform of October 1582).
    pub const MIN_YEAR: i32 = 1583;
    /// Largest accepted year (9999 — keeps the year an `i16`-fittable
    /// value and matches every published holiday-calendar horizon).
    pub const MAX_YEAR: i32 = 9999;

    // ─── Construction ────────────────────────────────────────────────────

    /// Constructs a validated `Date` from a `(year, month, day)` triple.
    ///
    /// Validation, in order: the year is in `MIN_YEAR..=MAX_YEAR`, the
    /// month is in `1..=12`, and the day is in `1..=days_in_month(year,
    /// month)` — which accounts for leap years.
    ///
    /// # Errors
    ///
    /// - [`ValidationError::OutOfRange`] with `what = "year < 1583"` or
    ///   `what = "year > 9999"` if the year is outside the supported
    ///   range.
    /// - [`ValidationError::InvalidDate`] with `rule = "month-out-of-range"`
    ///   if the month is not in `1..=12`.
    /// - [`ValidationError::InvalidDate`] with `rule = "day-out-of-range"`
    ///   if the day is not in `1..=days_in_month(year, month)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use regit_daycount::{Date, ValidationError};
    ///
    /// // A valid date.
    /// assert!(Date::ymd(2026, 5, 23).is_ok());
    ///
    /// // 2025 is not a leap year — 29 February is rejected.
    /// assert_eq!(
    ///     Date::ymd(2025, 2, 29),
    ///     Err(ValidationError::InvalidDate { rule: "day-out-of-range" }),
    /// );
    /// ```
    pub fn ymd(year: i32, month: u8, day: u8) -> Result<Self, ValidationError> {
        if year < Self::MIN_YEAR {
            return Err(ValidationError::OutOfRange {
                what: "year < 1583",
            });
        }
        if year > Self::MAX_YEAR {
            return Err(ValidationError::OutOfRange {
                what: "year > 9999",
            });
        }
        if !(1..=12).contains(&month) {
            return Err(ValidationError::InvalidDate {
                rule: "month-out-of-range",
            });
        }
        let dim = Self::days_in_month(year, month);
        if !(1..=dim).contains(&day) {
            return Err(ValidationError::InvalidDate {
                rule: "day-out-of-range",
            });
        }
        Ok(Self { year, month, day })
    }

    /// Constructs a `Date` without validating the triple.
    ///
    /// The caller asserts that `(year, month, day)` names a real Gregorian
    /// date inside the supported year range. This exists for `const`-context
    /// construction and for reconstructing a `Date` from fields validated
    /// earlier; prefer [`Date::ymd`] for any untrusted input.
    ///
    /// # Examples
    ///
    /// ```
    /// use regit_daycount::Date;
    ///
    /// let d = Date::ymd_unchecked(2026, 5, 23);
    /// assert_eq!(d.year(), 2026);
    /// ```
    #[must_use]
    pub const fn ymd_unchecked(year: i32, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }

    // ─── Trivial accessors ───────────────────────────────────────────────

    /// Returns the year.
    ///
    /// # Examples
    ///
    /// ```
    /// use regit_daycount::Date;
    ///
    /// assert_eq!(Date::ymd_unchecked(2026, 5, 23).year(), 2026);
    /// ```
    #[must_use]
    #[inline]
    pub const fn year(&self) -> i32 {
        self.year
    }

    /// Returns the month, `1..=12`.
    ///
    /// # Examples
    ///
    /// ```
    /// use regit_daycount::Date;
    ///
    /// assert_eq!(Date::ymd_unchecked(2026, 5, 23).month(), 5);
    /// ```
    #[must_use]
    #[inline]
    pub const fn month(&self) -> u8 {
        self.month
    }

    /// Returns the day of the month, `1..=31`.
    ///
    /// # Examples
    ///
    /// ```
    /// use regit_daycount::Date;
    ///
    /// assert_eq!(Date::ymd_unchecked(2026, 5, 23).day(), 23);
    /// ```
    #[must_use]
    #[inline]
    pub const fn day(&self) -> u8 {
        self.day
    }

    // ─── Calendar rules ──────────────────────────────────────────────────

    /// Returns `true` if `year` is a Gregorian leap year.
    ///
    /// A year is a leap year if it is divisible by 4 **and** either not
    /// divisible by 100 **or** divisible by 400. Hence 1900 is not a leap
    /// year, 2000 is, and 2100 is not.
    ///
    /// # Examples
    ///
    /// ```
    /// use regit_daycount::Date;
    ///
    /// assert!( Date::is_leap_year(2000));
    /// assert!( Date::is_leap_year(2024));
    /// assert!(!Date::is_leap_year(1900));
    /// assert!(!Date::is_leap_year(2025));
    /// ```
    #[must_use]
    #[inline]
    pub const fn is_leap_year(year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
    }

    /// Returns the number of days in `(year, month)`.
    ///
    /// February takes 29 days in a leap year and 28 otherwise; every other
    /// month is the conventional 30 or 31. If `month` is not in `1..=12`,
    /// returns `0` (the caller is expected to validate the month first;
    /// [`Date::ymd`] does).
    ///
    /// # Examples
    ///
    /// ```
    /// use regit_daycount::Date;
    ///
    /// assert_eq!(Date::days_in_month(2024, 2), 29);
    /// assert_eq!(Date::days_in_month(2025, 2), 28);
    /// assert_eq!(Date::days_in_month(2026, 4), 30);
    /// assert_eq!(Date::days_in_month(2026, 7), 31);
    /// ```
    #[must_use]
    #[inline]
    pub const fn days_in_month(year: i32, month: u8) -> u8 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if Self::is_leap_year(year) {
                    29
                } else {
                    28
                }
            }
            _ => 0,
        }
    }

    // ─── Hinnant epoch conversions ───────────────────────────────────────

    /// Days from the civil epoch 1970-01-01 to `(year, month, day)`.
    ///
    /// Transcribed verbatim (in `i64`) from Howard Hinnant's
    /// `days_from_civil` — public-domain, branchless, integer-only — at
    /// <http://howardhinnant.github.io/date_algorithms.html>. The output is
    /// negative for dates before 1970-01-01 and zero on that date itself.
    ///
    /// The arithmetic is performed in `i64` so that adding a 32-bit day
    /// offset cannot overflow even at the extremes of the supported year
    /// range.
    //
    // The cast lints are silenced for the algorithm body: `yoe ∈ [0, 399]`
    // and `doe ∈ [0, 146_096]` are bounds established by the algorithm
    // itself, so the `as u32` / `as i64` casts are exact, not narrowing.
    #[inline]
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_lossless
    )]
    const fn to_civil_days(year: i32, month: u8, day: u8) -> i64 {
        // Hinnant: y' = y - (m <= 2); era = floor(y' / 400)
        let y = year as i64 - if month <= 2 { 1 } else { 0 };
        let era = if y >= 0 { y } else { y - 399 } / 400;
        let yoe = (y - era * 400) as u32; // [0, 399]
        let m = month as u32;
        // doy = (153 * (m > 2 ? m - 3 : m + 9) + 2) / 5 + d - 1
        let m_shift = if m > 2 { m - 3 } else { m + 9 };
        let doy = (153 * m_shift + 2) / 5 + (day as u32) - 1; // [0, 365]
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
        era * 146_097 + doe as i64 - 719_468
    }

    /// Inverse of [`Self::to_civil_days`].
    ///
    /// Transcribed verbatim from Howard Hinnant's `civil_from_days` —
    /// public-domain, branchless, integer-only — at
    /// <http://howardhinnant.github.io/date_algorithms.html>.
    ///
    /// The output triple always satisfies `is_leap_year(year)`'s rule for
    /// its `(month, day)`; if it falls outside this crate's supported year
    /// range, [`Self::ymd_unchecked`] wraps it without re-validating —
    /// arithmetic that escapes the window is the caller's responsibility
    /// to bound.
    //
    // The cast lints are silenced for the algorithm body: `doe ∈
    // [0, 146_096]`, `mp ∈ [0, 11]`, `d ∈ [1, 31]`, and `year` fits in
    // `i32` for any input within roughly ±5 million civil days of the
    // 1970 epoch — bounds established by the algorithm itself, so the
    // narrowing casts are exact in the inputs the crate accepts.
    #[inline]
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_lossless
    )]
    const fn from_civil_days(z: i64) -> Self {
        let z = z + 719_468;
        let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
        let doe = (z - era * 146_097) as u32; // [0, 146096]
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
        let y = yoe as i64 + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
        let mp = (5 * doy + 2) / 153; // [0, 11]
        let d = (doy - (153 * mp + 2) / 5 + 1) as u8; // [1, 31]
        let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u8; // [1, 12]
        let year = (y + if m <= 2 { 1 } else { 0 }) as i32;
        Self {
            year,
            month: m,
            day: d,
        }
    }

    // ─── Day-of-week / arithmetic ────────────────────────────────────────

    /// Returns the day of the week for `self`.
    ///
    /// Computed by reducing the Hinnant civil-day count modulo 7 (the civil
    /// epoch 1970-01-01 was a Thursday).
    ///
    /// # Examples
    ///
    /// ```
    /// use regit_daycount::{Date, Weekday};
    ///
    /// assert_eq!(Date::ymd(2026, 5, 23).unwrap().day_of_week(), Weekday::Sat);
    /// assert_eq!(Date::ymd(2000, 1,  1).unwrap().day_of_week(), Weekday::Sat);
    /// assert_eq!(Date::ymd(2024, 12, 25).unwrap().day_of_week(), Weekday::Wed);
    /// ```
    #[must_use]
    pub fn day_of_week(self) -> Weekday {
        // 1970-01-01 is a Thursday. Reduce the signed civil-day count to a
        // non-negative weekday index using Euclidean modulo, which is
        // defined directly on `i64` without needing a manual shift.
        let z = Self::to_civil_days(self.year, self.month, self.day);
        // Thursday is index 3 in (Mon, Tue, Wed, Thu, Fri, Sat, Sun).
        let idx = (z + 3).rem_euclid(7);
        match idx {
            0 => Weekday::Mon,
            1 => Weekday::Tue,
            2 => Weekday::Wed,
            3 => Weekday::Thu,
            4 => Weekday::Fri,
            5 => Weekday::Sat,
            _ => Weekday::Sun,
        }
    }

    /// Returns `self` advanced by `days` calendar days (negative goes back).
    ///
    /// Implemented by converting `self` to its Hinnant civil-day count,
    /// adding `days`, and converting back. The intermediate arithmetic is
    /// `i64`, so a 32-bit `days` cannot overflow. The result is constructed
    /// via [`Self::ymd_unchecked`] and is *not* re-validated against the
    /// `[1583, 9999]` window — arithmetic that escapes that window is the
    /// caller's responsibility.
    ///
    /// # Examples
    ///
    /// ```
    /// use regit_daycount::Date;
    ///
    /// // Plain forward step into the next month.
    /// assert_eq!(
    ///     Date::ymd(2026, 1, 1).unwrap().add_days(31),
    ///     Date::ymd(2026, 2, 1).unwrap(),
    /// );
    ///
    /// // Leap-year boundary.
    /// assert_eq!(
    ///     Date::ymd(2024, 2, 28).unwrap().add_days(1),
    ///     Date::ymd(2024, 2, 29).unwrap(),
    /// );
    /// ```
    #[must_use]
    pub fn add_days(self, days: i32) -> Self {
        let z = Self::to_civil_days(self.year, self.month, self.day) + i64::from(days);
        Self::from_civil_days(z)
    }

    /// Returns `self` advanced by `months` months, clamping the day of the
    /// month to the last day of the resulting month when the original day
    /// does not exist there.
    ///
    /// The rule is: compute the new year and month from `(self.year * 12 +
    /// self.month - 1) + months`, then set the day to `min(self.day,
    /// days_in_month(new_year, new_month))`. So `2026-01-31 + 1 month =
    /// 2026-02-28` (not a leap year) and `2024-01-31 + 1 month =
    /// 2024-02-29` (leap year).
    ///
    /// The result is constructed via [`Self::ymd_unchecked`] and is *not*
    /// re-validated against the `[1583, 9999]` window.
    ///
    /// # Examples
    ///
    /// ```
    /// use regit_daycount::Date;
    ///
    /// assert_eq!(
    ///     Date::ymd(2026, 1, 31).unwrap().add_months_eom_aware(1),
    ///     Date::ymd(2026, 2, 28).unwrap(),
    /// );
    /// assert_eq!(
    ///     Date::ymd(2024, 1, 31).unwrap().add_months_eom_aware(1),
    ///     Date::ymd(2024, 2, 29).unwrap(),
    /// );
    /// ```
    #[must_use]
    pub fn add_months_eom_aware(self, months: i32) -> Self {
        // Convert (year, month) to a zero-based month index, add `months`,
        // and decompose. Performed in `i64` to avoid overflow on the
        // extremes of the supported year range. `new_year_i64` is at most
        // ~`MAX_YEAR + |months|/12`, which fits in `i32` for any reasonable
        // offset; `new_month_i64` is in `1..=12`.
        let total = i64::from(self.year) * 12 + (i64::from(self.month) - 1) + i64::from(months);
        let new_year_i64 = total.div_euclid(12);
        let new_month_i64 = total.rem_euclid(12) + 1;
        let new_year = i32::try_from(new_year_i64).unwrap_or(self.year);
        let new_month = u8::try_from(new_month_i64).unwrap_or(self.month);
        let dim = Self::days_in_month(new_year, new_month);
        let new_day = if self.day < dim { self.day } else { dim };
        Self::ymd_unchecked(new_year, new_month, new_day)
    }

    /// Returns the date of the `n`-th occurrence of `weekday` in
    /// `(year, month)`.
    ///
    /// `n = 1` is the first occurrence, `n = 5` is the fifth (if it
    /// exists). Computed by finding the first occurrence of `weekday` in
    /// the month and adding `(n - 1) * 7` days.
    ///
    /// # Errors
    ///
    /// - [`ValidationError::OutOfRange`] with `what = "n must be 1..=5"`
    ///   if `n` is not in `1..=5`.
    /// - [`ValidationError::InvalidDate`] with `rule = "month-out-of-range"`
    ///   if `month` is not in `1..=12`.
    /// - [`ValidationError::OutOfRange`] with `what = "year < 1583"` /
    ///   `"year > 9999"` if the year is outside the supported range.
    /// - [`ValidationError::OutOfRange`] with `what = "nth weekday does not
    ///   exist in this month"` if the computed date would fall after the
    ///   last day of the month (e.g. the 5th Monday of a 28-day February).
    ///
    /// # Examples
    ///
    /// ```
    /// use regit_daycount::{Date, Weekday};
    ///
    /// // The 3rd Friday of June 2026 is 2026-06-19.
    /// assert_eq!(
    ///     Date::nth_weekday_of_month(2026, 6, 3, Weekday::Fri).unwrap(),
    ///     Date::ymd(2026, 6, 19).unwrap(),
    /// );
    /// ```
    pub fn nth_weekday_of_month(
        year: i32,
        month: u8,
        n: u8,
        weekday: Weekday,
    ) -> Result<Self, ValidationError> {
        if !(1..=5).contains(&n) {
            return Err(ValidationError::OutOfRange {
                what: "n must be 1..=5",
            });
        }
        // Validate (year, month, 1) — that pins the month and year.
        let first = Self::ymd(year, month, 1)?;
        let first_wd = first.day_of_week();
        // Offset from `first` to the first occurrence of `weekday`.
        let offset = (weekday_index(weekday) + 7 - weekday_index(first_wd)) % 7;
        let day_in_month = 1 + offset + (i32::from(n) - 1) * 7;
        let dim = i32::from(Self::days_in_month(year, month));
        if day_in_month > dim {
            return Err(ValidationError::OutOfRange {
                what: "nth weekday does not exist in this month",
            });
        }
        // `day_in_month ∈ [1, 31]` here — bounded by the `> dim` check
        // above and a per-month maximum of 31 days.
        let day = u8::try_from(day_in_month).unwrap_or(1);
        Ok(Self { year, month, day })
    }

    /// Returns the date of Easter Sunday in the (Western, Gregorian) year
    /// `year`.
    ///
    /// Implemented by the **Anonymous Gregorian** / **Computus** algorithm
    /// (Meeus / Butcher form): a closed-form integer-only computation that
    /// is exact for every Gregorian year. See Jean Meeus, *Astronomical
    /// Algorithms* (2nd ed., 1998), §8 — "The date of Easter".
    ///
    /// The TARGET2 holiday rule derives Good Friday and Easter Monday from
    /// this date.
    ///
    /// # Examples
    ///
    /// ```
    /// use regit_daycount::Date;
    ///
    /// // Easter Sunday 2026 falls on 5 April.
    /// assert_eq!(Date::easter_sunday(2026), Date::ymd(2026, 4, 5).unwrap());
    /// ```
    //
    // The Meeus / Butcher / Anonymous-Gregorian variables `a..m` are the
    // standard one-letter names used in Meeus §8; preserving them keeps the
    // implementation auditable line-by-line against the published algorithm.
    // The casts at the bottom narrow values bounded by the algorithm to
    // `month ∈ {3, 4}` and `day ∈ [1, 31]`.
    #[must_use]
    #[allow(
        clippy::many_single_char_names,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    pub fn easter_sunday(year: i32) -> Self {
        // Meeus / Butcher / Anonymous Gregorian. Variable names follow the
        // standard presentation in Meeus §8.
        let y = year;
        let a = y % 19;
        let b = y / 100;
        let c = y % 100;
        let d = b / 4;
        let e = b % 4;
        let f = (b + 8) / 25;
        let g = (b - f + 1) / 3;
        let h = (19 * a + b - d - g + 15) % 30;
        let i = c / 4;
        let k = c % 4;
        let l = (32 + 2 * e + 2 * i - h - k) % 7;
        let m = (a + 11 * h + 22 * l) / 451;
        let month_num = (h + l - 7 * m + 114) / 31; // 3 = March, 4 = April
        let day_num = ((h + l - 7 * m + 114) % 31) + 1;
        Self {
            year: y,
            month: month_num as u8,
            day: day_num as u8,
        }
    }

    /// Returns the signed number of days from `self` to `other`.
    ///
    /// The convention is end-exclusive, start-inclusive: a one-day interval
    /// `[2026-01-01, 2026-01-02)` returns `1`. Computed by subtracting the
    /// two Hinnant civil-day counts; the result is `i32` and fits comfortably
    /// for any pair of dates in the supported year range (the full window
    /// 1583–9999 spans roughly `3.1 * 10⁶` days, well inside `i32`).
    ///
    /// # Examples
    ///
    /// ```
    /// use regit_daycount::Date;
    ///
    /// let a = Date::ymd(2026, 1, 1).unwrap();
    /// let b = Date::ymd(2026, 1, 2).unwrap();
    /// assert_eq!(a.days_between(b),  1);
    /// assert_eq!(b.days_between(a), -1);
    /// assert_eq!(a.days_between(a),  0);
    /// ```
    //
    // The difference of two civil-day counts for dates in `[1583, 9999]`
    // is bounded by ~3.1 × 10⁶, which fits in `i32`. `try_from` is used
    // defensively; on the (impossible-in-range) overflow path it falls back
    // to `i32::MAX`.
    #[must_use]
    pub fn days_between(self, other: Self) -> i32 {
        let a = Self::to_civil_days(self.year, self.month, self.day);
        let b = Self::to_civil_days(other.year, other.month, other.day);
        i32::try_from(b - a).unwrap_or(i32::MAX)
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Numeric index for a `Weekday`, Monday = 0.
#[inline]
const fn weekday_index(w: Weekday) -> i32 {
    match w {
        Weekday::Mon => 0,
        Weekday::Tue => 1,
        Weekday::Wed => 2,
        Weekday::Thu => 3,
        Weekday::Fri => 4,
        Weekday::Sat => 5,
        Weekday::Sun => 6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── is_leap_year ────────────────────────────────────────────────────

    #[test]
    fn leap_year_known_anchors() {
        assert!(!Date::is_leap_year(1900));
        assert!(Date::is_leap_year(2000));
        assert!(Date::is_leap_year(2004));
        assert!(Date::is_leap_year(2020));
        assert!(Date::is_leap_year(2024));
        assert!(!Date::is_leap_year(2025));
        assert!(!Date::is_leap_year(2026));
        assert!(!Date::is_leap_year(2100));
        assert!(Date::is_leap_year(2400));
    }

    // ─── days_in_month ───────────────────────────────────────────────────

    #[test]
    fn days_in_month_non_leap() {
        let expected = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        for (m, &d) in (1..=12).zip(expected.iter()) {
            assert_eq!(Date::days_in_month(2025, m), d, "month {m} non-leap");
        }
    }

    #[test]
    fn days_in_month_leap() {
        let expected = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        for (m, &d) in (1..=12).zip(expected.iter()) {
            assert_eq!(Date::days_in_month(2024, m), d, "month {m} leap");
        }
    }

    #[test]
    fn days_in_month_invalid_month_is_zero() {
        assert_eq!(Date::days_in_month(2026, 0), 0);
        assert_eq!(Date::days_in_month(2026, 13), 0);
    }

    // ─── ymd validation ──────────────────────────────────────────────────

    #[test]
    fn ymd_accepts_boundary_years() {
        assert!(Date::ymd(1583, 1, 1).is_ok());
        assert!(Date::ymd(9999, 12, 31).is_ok());
    }

    #[test]
    fn ymd_rejects_year_below_min() {
        assert_eq!(
            Date::ymd(1582, 1, 1),
            Err(ValidationError::OutOfRange {
                what: "year < 1583",
            })
        );
    }

    #[test]
    fn ymd_rejects_year_above_max() {
        assert_eq!(
            Date::ymd(10_000, 1, 1),
            Err(ValidationError::OutOfRange {
                what: "year > 9999",
            })
        );
    }

    #[test]
    fn ymd_rejects_month_zero_and_thirteen() {
        assert_eq!(
            Date::ymd(2026, 0, 1),
            Err(ValidationError::InvalidDate {
                rule: "month-out-of-range",
            })
        );
        assert_eq!(
            Date::ymd(2026, 13, 1),
            Err(ValidationError::InvalidDate {
                rule: "month-out-of-range",
            })
        );
    }

    #[test]
    fn ymd_rejects_day_zero_and_overflow() {
        assert_eq!(
            Date::ymd(2026, 1, 0),
            Err(ValidationError::InvalidDate {
                rule: "day-out-of-range",
            })
        );
        assert_eq!(
            Date::ymd(2026, 1, 32),
            Err(ValidationError::InvalidDate {
                rule: "day-out-of-range",
            })
        );
        // 30 April does not exist.
        assert_eq!(
            Date::ymd(2026, 4, 31),
            Err(ValidationError::InvalidDate {
                rule: "day-out-of-range",
            })
        );
    }

    #[test]
    fn ymd_rejects_feb_29_in_non_leap() {
        assert_eq!(
            Date::ymd(2025, 2, 29),
            Err(ValidationError::InvalidDate {
                rule: "day-out-of-range",
            })
        );
        // ... and accepts it in a leap year.
        assert!(Date::ymd(2024, 2, 29).is_ok());
    }

    // ─── day_of_week ─────────────────────────────────────────────────────

    #[test]
    fn day_of_week_named_dates() {
        // Hand-verified against published calendars.
        assert_eq!(
            Date::ymd(1900, 1, 1).unwrap().day_of_week(),
            Weekday::Mon,
            "1900-01-01"
        );
        assert_eq!(
            Date::ymd(1999, 12, 31).unwrap().day_of_week(),
            Weekday::Fri,
            "1999-12-31"
        );
        assert_eq!(
            Date::ymd(2000, 1, 1).unwrap().day_of_week(),
            Weekday::Sat,
            "2000-01-01"
        );
        assert_eq!(
            Date::ymd(2020, 2, 29).unwrap().day_of_week(),
            Weekday::Sat,
            "2020-02-29"
        );
        assert_eq!(
            Date::ymd(2024, 12, 25).unwrap().day_of_week(),
            Weekday::Wed,
            "2024-12-25"
        );
        assert_eq!(
            Date::ymd(2026, 1, 1).unwrap().day_of_week(),
            Weekday::Thu,
            "2026-01-01"
        );
        assert_eq!(
            Date::ymd(2026, 5, 23).unwrap().day_of_week(),
            Weekday::Sat,
            "2026-05-23"
        );
        assert_eq!(
            Date::ymd(2026, 12, 31).unwrap().day_of_week(),
            Weekday::Thu,
            "2026-12-31"
        );
        assert_eq!(
            Date::ymd(2038, 1, 19).unwrap().day_of_week(),
            Weekday::Tue,
            "2038-01-19"
        );
        assert_eq!(
            Date::ymd(1969, 7, 20).unwrap().day_of_week(),
            Weekday::Sun,
            "1969-07-20 (Apollo 11 Moon landing)"
        );
    }

    #[test]
    fn day_of_week_cycles_correctly() {
        // Seven consecutive days must produce all seven weekdays once.
        let mut d = Date::ymd(2026, 1, 5).unwrap(); // Monday
        let expected = [
            Weekday::Mon,
            Weekday::Tue,
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
            Weekday::Sat,
            Weekday::Sun,
        ];
        for &e in &expected {
            assert_eq!(d.day_of_week(), e);
            d = d.add_days(1);
        }
    }

    // ─── add_days ────────────────────────────────────────────────────────

    #[test]
    fn add_days_into_next_month() {
        assert_eq!(
            Date::ymd(2026, 1, 1).unwrap().add_days(31),
            Date::ymd(2026, 2, 1).unwrap()
        );
    }

    #[test]
    fn add_days_leap_boundary() {
        assert_eq!(
            Date::ymd(2024, 2, 28).unwrap().add_days(1),
            Date::ymd(2024, 2, 29).unwrap()
        );
        assert_eq!(
            Date::ymd(2025, 2, 28).unwrap().add_days(1),
            Date::ymd(2025, 3, 1).unwrap()
        );
    }

    #[test]
    fn add_days_year_boundary() {
        assert_eq!(
            Date::ymd(2026, 12, 31).unwrap().add_days(1),
            Date::ymd(2027, 1, 1).unwrap()
        );
    }

    #[test]
    fn add_days_round_trip_small_grid() {
        // Manual deterministic round-trip grid over a wide span of offsets.
        let anchor = Date::ymd(2026, 5, 23).unwrap();
        for &n in &[
            -10_000, -3_653, -366, -365, -100, -7, -1, 0, 1, 7, 100, 365, 366, 3_653, 10_000,
        ] {
            assert_eq!(anchor.add_days(n).add_days(-n), anchor, "offset {n}");
        }
    }

    #[test]
    fn add_days_round_trip_proptest() {
        use proptest::prelude::*;
        let anchor = Date::ymd(2026, 5, 23).unwrap();
        proptest!(|(n in -10_000i32..10_000)| {
            prop_assert_eq!(anchor.add_days(n).add_days(-n), anchor);
        });
    }

    // ─── add_months_eom_aware ────────────────────────────────────────────

    #[test]
    fn add_months_eom_aware_non_leap_clamp() {
        assert_eq!(
            Date::ymd(2026, 1, 31).unwrap().add_months_eom_aware(1),
            Date::ymd(2026, 2, 28).unwrap()
        );
    }

    #[test]
    fn add_months_eom_aware_leap_keeps_29() {
        assert_eq!(
            Date::ymd(2024, 1, 31).unwrap().add_months_eom_aware(1),
            Date::ymd(2024, 2, 29).unwrap()
        );
    }

    #[test]
    fn add_months_eom_aware_year_rollover() {
        assert_eq!(
            Date::ymd(2026, 12, 31).unwrap().add_months_eom_aware(1),
            Date::ymd(2027, 1, 31).unwrap()
        );
    }

    #[test]
    fn add_months_eom_aware_negative_rollover() {
        // -1 month from 2026-03-31 lands on 2026-02-28 (clamped).
        assert_eq!(
            Date::ymd(2026, 3, 31).unwrap().add_months_eom_aware(-1),
            Date::ymd(2026, 2, 28).unwrap()
        );
        // -1 month from 2024-03-31 lands on 2024-02-29 (clamped, leap).
        assert_eq!(
            Date::ymd(2024, 3, 31).unwrap().add_months_eom_aware(-1),
            Date::ymd(2024, 2, 29).unwrap()
        );
        // -12 months goes back exactly one year.
        assert_eq!(
            Date::ymd(2026, 5, 15).unwrap().add_months_eom_aware(-12),
            Date::ymd(2025, 5, 15).unwrap()
        );
    }

    // ─── nth_weekday_of_month ────────────────────────────────────────────

    #[test]
    fn nth_weekday_third_friday_june_2026() {
        // 3rd Friday of June 2026 = 2026-06-19.
        assert_eq!(
            Date::nth_weekday_of_month(2026, 6, 3, Weekday::Fri).unwrap(),
            Date::ymd(2026, 6, 19).unwrap()
        );
    }

    #[test]
    fn nth_weekday_fifth_monday_feb_2026_does_not_exist() {
        // February 2026 has 28 days starting Sunday, so it has only four
        // Mondays.
        assert_eq!(
            Date::nth_weekday_of_month(2026, 2, 5, Weekday::Mon),
            Err(ValidationError::OutOfRange {
                what: "nth weekday does not exist in this month",
            })
        );
    }

    #[test]
    fn nth_weekday_first_of_each_weekday_jan_2026() {
        // 2026-01-01 is a Thursday. So in January 2026:
        //   first Thu = 2026-01-01, first Fri = 2026-01-02,
        //   first Sat = 2026-01-03, first Sun = 2026-01-04,
        //   first Mon = 2026-01-05, first Tue = 2026-01-06,
        //   first Wed = 2026-01-07.
        let cases = [
            (Weekday::Thu, 1),
            (Weekday::Fri, 2),
            (Weekday::Sat, 3),
            (Weekday::Sun, 4),
            (Weekday::Mon, 5),
            (Weekday::Tue, 6),
            (Weekday::Wed, 7),
        ];
        for (wd, day) in cases {
            assert_eq!(
                Date::nth_weekday_of_month(2026, 1, 1, wd).unwrap(),
                Date::ymd(2026, 1, day).unwrap(),
                "first {wd:?} of Jan 2026",
            );
        }
    }

    #[test]
    fn nth_weekday_rejects_invalid_n_and_month() {
        assert_eq!(
            Date::nth_weekday_of_month(2026, 1, 0, Weekday::Mon),
            Err(ValidationError::OutOfRange {
                what: "n must be 1..=5",
            })
        );
        assert_eq!(
            Date::nth_weekday_of_month(2026, 13, 1, Weekday::Mon),
            Err(ValidationError::InvalidDate {
                rule: "month-out-of-range",
            })
        );
    }

    // ─── easter_sunday ───────────────────────────────────────────────────

    #[test]
    fn easter_sunday_known_dates() {
        // Published Easter table — Western (Gregorian) computus.
        let cases = [
            (2024, 3, 31),
            (2025, 4, 20),
            (2026, 4, 5),
            (2027, 3, 28),
            (2028, 4, 16),
            (2030, 4, 21),
            (2038, 4, 25),
        ];
        for (y, m, d) in cases {
            assert_eq!(
                Date::easter_sunday(y),
                Date::ymd(y, m, d).unwrap(),
                "Easter {y}",
            );
        }
    }

    // ─── days_between ────────────────────────────────────────────────────

    #[test]
    fn days_between_one_day() {
        let a = Date::ymd(2026, 1, 1).unwrap();
        let b = Date::ymd(2026, 1, 2).unwrap();
        assert_eq!(a.days_between(b), 1);
        assert_eq!(b.days_between(a), -1);
    }

    #[test]
    fn days_between_same_date_is_zero() {
        let a = Date::ymd(2026, 5, 23).unwrap();
        assert_eq!(a.days_between(a), 0);
    }

    #[test]
    fn days_between_year_is_365_or_366() {
        // 2025 is not a leap year.
        let s = Date::ymd(2025, 1, 1).unwrap();
        let e = Date::ymd(2026, 1, 1).unwrap();
        assert_eq!(s.days_between(e), 365);
        // 2024 is a leap year.
        let s = Date::ymd(2024, 1, 1).unwrap();
        let e = Date::ymd(2025, 1, 1).unwrap();
        assert_eq!(s.days_between(e), 366);
    }

    #[test]
    fn days_between_additive() {
        let a = Date::ymd(2024, 1, 1).unwrap();
        let b = Date::ymd(2024, 7, 15).unwrap();
        let c = Date::ymd(2025, 3, 31).unwrap();
        assert_eq!(
            a.days_between(b) + b.days_between(c),
            a.days_between(c),
            "additivity",
        );
    }

    #[test]
    fn days_between_additive_proptest() {
        use proptest::prelude::*;
        let anchor = Date::ymd(2026, 1, 1).unwrap();
        proptest!(|(n1 in -3_000i32..3_000, n2 in -3_000i32..3_000)| {
            let b = anchor.add_days(n1);
            let c = b.add_days(n2);
            prop_assert_eq!(
                anchor.days_between(b) + b.days_between(c),
                anchor.days_between(c),
            );
        });
    }

    // ─── Ord / Copy ──────────────────────────────────────────────────────

    #[test]
    fn date_ordering_is_chronological() {
        let a = Date::ymd(2026, 1, 1).unwrap();
        let b = Date::ymd(2026, 1, 2).unwrap();
        let c = Date::ymd(2026, 2, 1).unwrap();
        let d = Date::ymd(2027, 1, 1).unwrap();
        assert!(a < b);
        assert!(b < c);
        assert!(c < d);
    }

    #[test]
    fn date_is_copy() {
        let a = Date::ymd(2026, 5, 23).unwrap();
        let b = a; // Copy
        assert_eq!(a, b);
    }
}
