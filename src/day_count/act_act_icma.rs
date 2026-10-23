// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! Act/Act ICMA for an accrual contained in one reference coupon period.
//!
//! The fraction is the signed accrual day count divided by the reference
//! period's day count and the number of coupon payments per year. A full
//! regular semiannual reference period therefore has fraction `0.5`.
//!
//! The caller supplies the contractual regular or notional reference period
//! and its frequency. The implementation checks ordering, nonzero frequency,
//! and containment; dates alone cannot establish whether the reference period
//! is the correct one for a particular security. Long coupons or accruals
//! spanning multiple reference periods must be decomposed by the caller and
//! calculated separately. No schedule or frequency is inferred.
//!
//! Reference: ICMA Rule 251, accrued interest calculation.

use crate::date::Date;

use super::{CouponPeriod, DayCountError};

/// Computes Act/Act ICMA within a single reference coupon period.
///
/// `freq` is the positive number of coupon payments per year, such as `1`
/// (annual), `2` (semiannual), `4` (quarterly), or `12` (monthly).
/// Reversed accruals use the same ordered reference period and negate the
/// result. Equal accrual endpoints return zero after context validation.
///
/// # Errors
///
/// Returns [`DayCountError::ZeroFrequency`] for zero frequency,
/// [`DayCountError::InvalidReferencePeriod`] for an empty or inverted
/// reference period, or [`DayCountError::AccrualOutsideReferencePeriod`]
/// if either accrual endpoint falls outside it. A multi-period accrual must
/// be split using the instrument's actual schedule.
///
/// # Examples
///
/// ```
/// use regit_daycount::{Date, day_count::act_act_icma};
/// let ref_start = Date::ymd(2026, 3, 1).unwrap();
/// let ref_end = Date::ymd(2026, 9, 1).unwrap();
/// let end = Date::ymd(2026, 6, 1).unwrap();
/// // 92 accrual days / (2 payments per year * 184 reference days).
/// let fraction = act_act_icma::try_fraction(ref_start, end, ref_start, ref_end, 2).unwrap();
/// assert!((fraction - 0.25).abs() < 1e-12);
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
    let ref_days = ref_start.days_between(ref_end);
    let calc_days = start.days_between(end);
    Ok(f64::from(calc_days) / (f64::from(freq) * f64::from(ref_days)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(year: i32, month: u8, day: u8) -> Date {
        Date::ymd(year, month, day).unwrap()
    }

    #[test]
    fn coupon_frequency_controls_full_regular_periods() {
        for (start, end, frequency, expected_days) in [
            (d(2026, 1, 1), d(2027, 1, 1), 1, 365),
            (d(2026, 3, 1), d(2026, 9, 1), 2, 184),
            (d(2026, 3, 1), d(2026, 6, 1), 4, 92),
            (d(2024, 2, 1), d(2024, 3, 1), 12, 29),
        ] {
            assert_eq!(start.days_between(end), expected_days);
            let actual = try_fraction(start, end, start, end, frequency).unwrap();
            assert!((actual - 1.0 / f64::from(frequency)).abs() < 1e-12);
        }
    }

    #[test]
    fn short_accrual_uses_the_full_notional_reference_period() {
        let ref_start = d(2026, 3, 1);
        let ref_end = d(2026, 9, 1);
        let start = d(2026, 6, 1);
        assert_eq!(start.days_between(ref_end), 92);
        let forward = try_fraction(start, ref_end, ref_start, ref_end, 2).unwrap();
        let reverse = try_fraction(ref_end, start, ref_start, ref_end, 2).unwrap();
        assert!((forward - 0.25).abs() < 1e-12);
        assert!((reverse + 0.25).abs() < 1e-12);
    }

    #[test]
    fn invalid_coupon_context_returns_a_typed_error() {
        let start = d(2026, 3, 1);
        let end = d(2026, 9, 1);
        assert_eq!(
            try_fraction(start, end, start, end, 0),
            Err(DayCountError::ZeroFrequency)
        );
        for (ref_start, ref_end) in [(start, start), (end, start)] {
            assert_eq!(
                try_fraction(start, end, ref_start, ref_end, 2),
                Err(DayCountError::InvalidReferencePeriod)
            );
        }
        for (accrual_start, accrual_end) in [
            (d(2026, 2, 28), end),
            (start, d(2026, 9, 2)),
            (end, d(2026, 2, 28)),
            (d(2026, 9, 2), start),
        ] {
            assert_eq!(
                try_fraction(accrual_start, accrual_end, start, end, 2),
                Err(DayCountError::AccrualOutsideReferencePeriod)
            );
        }
    }

    #[test]
    fn equal_dates_still_require_valid_context() {
        let point = d(2026, 3, 1);
        assert_eq!(try_fraction(point, point, point, d(2026, 9, 1), 2), Ok(0.0));
        assert_eq!(
            try_fraction(point, point, point, point, 2),
            Err(DayCountError::InvalidReferencePeriod)
        );
    }

    #[test]
    fn long_coupon_is_rejected_instead_of_inventing_reference_periods() {
        let start = d(2026, 1, 1);
        let end = d(2026, 9, 1);
        let ref_start = d(2026, 3, 1);
        assert_eq!(
            try_fraction(start, end, ref_start, end, 2),
            Err(DayCountError::AccrualOutsideReferencePeriod)
        );
        // The caller's schedule supplies the preceding notional period.
        let first = try_fraction(start, ref_start, d(2025, 9, 1), ref_start, 2).unwrap();
        let second = try_fraction(ref_start, end, ref_start, end, 2).unwrap();
        assert!((first + second - (59.0 / 362.0 + 0.5)).abs() < 1e-12);
    }

    #[test]
    fn valid_boundary_reference_periods_are_finite() {
        for (start, end) in [
            (d(1583, 1, 1), d(1583, 7, 1)),
            (d(9999, 6, 30), d(9999, 12, 31)),
        ] {
            let fraction = try_fraction(start, end, start, end, 2).unwrap();
            assert!((fraction - 0.5).abs() < 1e-12);
        }
    }
}
