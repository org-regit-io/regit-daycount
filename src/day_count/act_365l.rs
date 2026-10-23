// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! Act/365L (Actual/365 Long), with explicit coupon context.
//!
//! The numerator is the signed number of actual days. For annual coupons,
//! the denominator is 366 if a leap day lies after the earlier accrual date
//! and on or before the next coupon date, and 365 otherwise. For nonannual
//! coupons, it is 366 if the next coupon date falls in a leap year, and 365
//! otherwise. The next coupon can differ from the accrual end.
//!
//! Reverse accruals use the same forward coupon context and negate the
//! numerator. Only accruals contained within the supplied coupon period are
//! accepted. Constructing or splitting irregular schedules is out of scope.
//!
//! These ICMA rules are independently checked against the pinned
//! [FinancePy day-count implementation](https://github.com/domokane/FinancePy/blob/80b1e8c5dad16788393f788f6b9aa67019823138/financepy/utils/day_count.py).
//! The annual leap-day interval is `(accrual_start, coupon_end]`.

use crate::date::Date;

use super::{CouponPeriod, DayCountError};

/// Computes Act/365L using the reference coupon period and frequency.
///
/// Annual frequency uses leap-day membership in `(min(start, end), ref_end]`.
/// Other positive frequencies use the leap status of `ref_end.year()`.
/// Frequency denotes coupons per year; the caller supplies the governing
/// schedule's period rather than inferring it from the accrual dates.
///
/// # Errors
///
/// Returns [`DayCountError::ZeroFrequency`] for zero frequency,
/// [`DayCountError::InvalidReferencePeriod`] for an empty or inverted coupon
/// period, and [`DayCountError::AccrualOutsideReferencePeriod`] if either
/// accrual endpoint lies outside the coupon period.
///
/// # Examples
///
/// ```
/// use regit_daycount::{Date, day_count::act_365l};
/// let start = Date::ymd(2024, 4, 1).unwrap();
/// let end = Date::ymd(2024, 7, 1).unwrap();
/// // Quarterly coupon ends in a leap year, even though February is earlier.
/// let result = act_365l::try_fraction(start, end, start, end, 4).unwrap();
/// assert!((result - 91.0 / 366.0).abs() < 1e-12);
/// ```
pub fn try_fraction(
    start: Date,
    end: Date,
    ref_start: Date,
    ref_end: Date,
    freq: u8,
) -> Result<f64, DayCountError> {
    CouponPeriod {
        start: ref_start,
        end: ref_end,
        frequency: freq,
    }
    .validate(start, end)?;
    let lower = core::cmp::min(start, end);
    let leap_denominator = if freq == 1 {
        period_contains_leap_day(lower, ref_end)
    } else {
        Date::is_leap_year(ref_end.year())
    };
    let denominator = if leap_denominator { 366.0 } else { 365.0 };
    Ok(f64::from(start.days_between(end)) / denominator)
}

/// Whether a leap day lies in `(start, end]`, with ordered endpoints.
fn period_contains_leap_day(start: Date, end: Date) -> bool {
    let mut year = start.year();
    while year <= end.year() {
        if Date::is_leap_year(year) {
            let leap = Date::ymd_unchecked(year, 2, 29);
            if start < leap && leap <= end {
                return true;
            }
        }
        year += 1;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(year: i32, month: u8, day: u8) -> Date {
        Date::ymd(year, month, day).unwrap()
    }

    fn assert_fraction(start: Date, end: Date, rs: Date, re: Date, freq: u8, denominator: f64) {
        let expected = f64::from(start.days_between(end)) / denominator;
        let forward = try_fraction(start, end, rs, re, freq).unwrap();
        let reverse = try_fraction(end, start, rs, re, freq).unwrap();
        assert!((forward - expected).abs() < 1e-12);
        assert!((reverse + expected).abs() < 1e-12);
    }

    #[test]
    fn annual_leap_coupon_end_is_included() {
        assert_fraction(
            d(2023, 2, 28),
            d(2024, 2, 29),
            d(2023, 2, 28),
            d(2024, 2, 29),
            1,
            366.0,
        );
    }

    #[test]
    fn annual_leap_accrual_start_is_excluded() {
        assert_fraction(
            d(2024, 2, 29),
            d(2025, 2, 28),
            d(2024, 2, 29),
            d(2025, 2, 28),
            1,
            365.0,
        );
    }

    #[test]
    fn annual_uses_next_coupon_beyond_accrual_end() {
        // February 29 is outside the accrual, but before the next coupon.
        assert_fraction(
            d(2023, 3, 1),
            d(2023, 4, 1),
            d(2023, 3, 1),
            d(2024, 3, 1),
            1,
            366.0,
        );
    }

    #[test]
    fn annual_uses_accrual_start_instead_of_reference_start() {
        // A partial accrual after February 29 has no following leap day.
        assert_fraction(
            d(2024, 3, 1),
            d(2024, 4, 1),
            d(2023, 7, 1),
            d(2024, 7, 1),
            1,
            365.0,
        );
    }

    #[test]
    fn nonannual_uses_coupon_end_year_even_without_leap_day_in_accrual() {
        assert_fraction(
            d(2024, 4, 1),
            d(2024, 7, 1),
            d(2024, 4, 1),
            d(2024, 7, 1),
            4,
            366.0,
        );
        assert_fraction(
            d(2024, 10, 1),
            d(2025, 1, 1),
            d(2024, 10, 1),
            d(2025, 1, 1),
            4,
            365.0,
        );
    }

    #[test]
    fn nonannual_uses_next_coupon_beyond_accrual_end() {
        assert_fraction(
            d(2023, 10, 1),
            d(2023, 11, 1),
            d(2023, 10, 1),
            d(2024, 4, 1),
            2,
            366.0,
        );
    }

    #[test]
    fn valid_equal_accrual_is_zero() {
        let point = d(2024, 2, 29);
        assert_eq!(
            try_fraction(point, point, d(2024, 1, 1), d(2025, 1, 1), 1),
            Ok(0.0)
        );
    }

    #[test]
    fn invalid_coupon_context_is_an_error() {
        let start = d(2024, 1, 1);
        let end = d(2024, 7, 1);
        assert_eq!(
            try_fraction(start, end, start, end, 0),
            Err(DayCountError::ZeroFrequency)
        );
        assert_eq!(
            try_fraction(start, end, end, start, 2),
            Err(DayCountError::InvalidReferencePeriod)
        );
        assert_eq!(
            try_fraction(start, end, start, start, 2),
            Err(DayCountError::InvalidReferencePeriod)
        );
        assert_eq!(
            try_fraction(start, end, d(2024, 2, 1), end, 2),
            Err(DayCountError::AccrualOutsideReferencePeriod)
        );
    }
}
