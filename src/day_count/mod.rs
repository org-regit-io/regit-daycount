// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! Catalogue of day-count fractions.
//!
//! A day-count fraction is the rule that maps a date interval `(start, end)`
//! to a year-fraction `f`. It is the parameter that turns an annualized interest
//! rate into an interest amount for an actual interval — and getting it
//! wrong silently mis-states a cashflow.
//!
//! Eleven conventions are catalogued here; each gets its own submodule with
//! the algorithm of the governing standard and a worked example computed by
//! hand:
//!
//! ```text
//! Act/360              ISDA 2006 §4.16(e)
//! Act/365F             ISDA 2006 §4.16(d)
//! ActAct ISDA          ISDA 2006 §4.16(b)  — splits at the year boundary
//! ActAct ICMA          ICMA Rule 251       — needs the reference period
//! 30/360 BondBasis     ISDA 2006 §4.16(f)
//! 30E/360              ISDA 2006 §4.16(g)
//! 30E/360 ISDA         ISDA 2006 §4.16(h)  — maturity-day-aware
//! Act/365L             ICMA / sterling money-market
//! NL/365               No-Leap — 29 Feb skipped in the numerator
//! Bus/252              Brazilian — business days only, denominator 252
//! 1/1                  explicitly defined constant-one shortcut
//! ```
//!
//! # Dispatchers
//!
//! [`try_fraction`] is the checked entry point. [`DayCountContext`] supplies
//! the coupon period for Act/365L and Act/Act ICMA, and an explicit maturity
//! flag for 30E/360 ISDA. Missing or invalid context is a [`DayCountError`].
//! [`try_fraction_with_business_days`] additionally accepts the predicate
//! required by Bus/252.
//!
//! Schedule-dependent conventions have no context-free entry point.
//! The per-convention functions that need no additional context return a
//! finite `f64` for every pair of valid dates.
//!
//! Reverse and equal endpoints follow each convention's arithmetic. The
//! schedule-aware conventions use an ordered coupon period even for reversed
//! accruals. In particular, 1/1 always returns one; 30/360 Bond Basis need
//! not be antisymmetric, and 30E/360 ISDA can return a nonzero fraction for
//! equal February maturity endpoints because only the first day is adjusted.
//!
//! # References
//!
//! - ISDA 2006 Definitions §4.16 (*Day Count Fraction*).
//! - ICMA Rule 251 (*Yield calculation for fixed-rate bonds*).

use crate::date::Date;

// ─── DayCount ────────────────────────────────────────────────────────────────

/// A day-count fraction — the rule by which a date interval is mapped to a
/// year-fraction.
///
/// The variants cover every convention this crate implements; each one is
/// defined precisely by the standard cited next to it in the module-level
/// docstring.
///
/// # Examples
///
/// ```
/// use regit_daycount::DayCount;
///
/// let basis = DayCount::Act360;
/// assert_eq!(basis, DayCount::Act360);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DayCount {
    /// Actual / 360 — the money-market default — ISDA 2006 §4.16(e).
    Act360,
    /// Actual / 365 Fixed — ISDA 2006 §4.16(d).
    Act365F,
    /// Actual / Actual ISDA — splits at the year boundary —
    /// ISDA 2006 §4.16(b).
    ActActIsda,
    /// Actual / Actual ICMA — reference-period aware — ICMA Rule 251.
    ActActIcma,
    /// 30 / 360 Bond Basis — ISDA 2006 §4.16(f).
    Thirty360BondBasis,
    /// 30E / 360 — the "Eurobond" variant — ISDA 2006 §4.16(g).
    ThirtyE360,
    /// 30E / 360 ISDA — the maturity-day-aware variant —
    /// ISDA 2006 §4.16(h).
    ThirtyE360Isda,
    /// Actual / 365L — leap-year-sensitive — ICMA.
    Act365L,
    /// No-Leap / 365 — the 29 February is skipped in the numerator.
    Nl365,
    /// Business / 252 — Brazilian convention — denominator is 252.
    Bus252,
    /// 1 / 1 — constant-one shortcut; every interval has fraction 1.
    OneOne,
}

// ─── Per-convention submodules ───────────────────────────────────────────────

pub mod error;
pub use error::DayCountError;

pub mod act_360;
pub mod act_365f;
pub mod act_365l;
pub mod act_act_icma;
pub mod act_act_isda;
pub mod bus_252;
pub mod nl_365;
pub mod one_one;
pub mod thirty_360_bond_basis;
pub mod thirty_e_360;
pub mod thirty_e_360_isda;

// ─── Dispatchers ─────────────────────────────────────────────────────────────

/// A single ordered reference coupon period, supplied by the caller.
///
/// `frequency` is the positive number of coupon payments per year. Both
/// accrual endpoints must lie within `[start, end]`. The caller remains
/// responsible for supplying the contractual regular or notional reference
/// period, its frequency, and for decomposing irregular coupons that span
/// multiple reference periods. Ordering and containment are checked; the
/// correct financial schedule cannot be established from these fields alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CouponPeriod {
    /// Start of the reference coupon period.
    pub start: Date,
    /// End of the reference coupon period (the next coupon date).
    pub end: Date,
    /// Number of coupon payments per year.
    pub frequency: u8,
}

impl CouponPeriod {
    pub(crate) fn validate(self, start: Date, end: Date) -> Result<(), DayCountError> {
        if self.frequency == 0 {
            return Err(DayCountError::ZeroFrequency);
        }
        if self.start >= self.end {
            return Err(DayCountError::InvalidReferencePeriod);
        }
        if core::cmp::min(start, end) < self.start || core::cmp::max(start, end) > self.end {
            return Err(DayCountError::AccrualOutsideReferencePeriod);
        }
        Ok(())
    }
}

/// Contractual information needed by schedule-dependent day counts.
///
/// Fields not used by the selected convention are ignored. A missing field
/// required by the selected convention is an error, including on equal dates.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DayCountContext {
    /// Reference coupon period for Act/365L or Act/Act ICMA.
    pub coupon: Option<CouponPeriod>,
    /// Whether the supplied accrual `end` is the instrument's maturity date.
    pub end_is_maturity: Option<bool>,
}

/// Computes a year fraction, requiring all context used by the convention.
///
/// # Errors
///
/// Returns a missing-context error for Act/365L, Act/Act ICMA, 30E/360 ISDA,
/// or Bus/252 when its required context is absent. Coupon validation errors
/// are propagated from [`act_365l::try_fraction`] or
/// [`act_act_icma::try_fraction`]. Use [`try_fraction_with_business_days`] for
/// Bus/252 or [`bus_252::fraction`] directly with a business-day predicate.
///
/// # Examples
///
/// ```
/// use regit_daycount::{Date, DayCount, day_count::{self, CouponPeriod, DayCountContext}};
/// let start = Date::ymd(2024, 4, 1).unwrap();
/// let end = Date::ymd(2024, 7, 1).unwrap();
/// let context = DayCountContext {
///     coupon: Some(CouponPeriod { start, end, frequency: 4 }),
///     ..DayCountContext::default()
/// };
/// let fraction = day_count::try_fraction(start, end, DayCount::Act365L, context).unwrap();
/// assert!((fraction - 91.0 / 366.0).abs() < 1e-12);
/// ```
pub fn try_fraction(
    start: Date,
    end: Date,
    basis: DayCount,
    context: DayCountContext,
) -> Result<f64, DayCountError> {
    match basis {
        DayCount::ActActIcma | DayCount::Act365L => {
            let coupon = context.coupon.ok_or(DayCountError::MissingCouponPeriod)?;
            if basis == DayCount::ActActIcma {
                act_act_icma::try_fraction(start, end, coupon.start, coupon.end, coupon.frequency)
            } else {
                act_365l::try_fraction(start, end, coupon.start, coupon.end, coupon.frequency)
            }
        }
        DayCount::ThirtyE360Isda => {
            let maturity = context
                .end_is_maturity
                .ok_or(DayCountError::MissingMaturity)?;
            Ok(thirty_e_360_isda::fraction(start, end, maturity))
        }
        DayCount::Bus252 => Err(DayCountError::MissingBusinessDayPredicate),
        DayCount::Act360 => Ok(act_360::fraction(start, end)),
        DayCount::Act365F => Ok(act_365f::fraction(start, end)),
        DayCount::ActActIsda => Ok(act_act_isda::fraction(start, end)),
        DayCount::Thirty360BondBasis => Ok(thirty_360_bond_basis::fraction(start, end)),
        DayCount::ThirtyE360 => Ok(thirty_e_360::fraction(start, end)),
        DayCount::Nl365 => Ok(nl_365::fraction(start, end)),
        DayCount::OneOne => Ok(one_one::fraction(start, end)),
    }
}

/// Computes a year fraction with an explicit business-day predicate.
///
/// The predicate is used only for Bus/252. All other conventions delegate
/// to [`try_fraction`] with the supplied context.
///
/// # Errors
///
/// Returns the same errors as [`try_fraction`] for conventions other than
/// Bus/252. The predicate defines the business-day set and must cover the
/// complete interval; this function does not infer a calendar or coverage.
pub fn try_fraction_with_business_days(
    start: Date,
    end: Date,
    basis: DayCount,
    context: DayCountContext,
    is_business_day: impl Fn(Date) -> bool,
) -> Result<f64, DayCountError> {
    if basis == DayCount::Bus252 {
        Ok(bus_252::fraction(start, end, is_business_day))
    } else {
        try_fraction(start, end, basis, context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(year: i32, month: u8, day: u8) -> Date {
        Date::ymd(year, month, day).unwrap()
    }

    fn checked(start: Date, end: Date, basis: DayCount) -> Result<f64, DayCountError> {
        try_fraction(start, end, basis, DayCountContext::default())
    }

    #[test]
    fn dispatch_preserves_each_context_free_conventions_formula() {
        let start = d(2024, 1, 15);
        let end = d(2024, 7, 31);
        for (basis, expected) in [
            (DayCount::Act360, 198.0 / 360.0),
            (DayCount::Act365F, 198.0 / 365.0),
            (DayCount::ActActIsda, 198.0 / 366.0),
            (DayCount::Thirty360BondBasis, 196.0 / 360.0),
            (DayCount::ThirtyE360, 195.0 / 360.0),
            (DayCount::Nl365, 197.0 / 365.0),
            (DayCount::OneOne, 1.0),
        ] {
            let actual = checked(start, end, basis).unwrap();
            assert!((actual - expected).abs() < 1e-12, "{basis:?}");
        }
    }

    #[test]
    fn absent_context_is_an_error_even_for_equal_dates() {
        let point = d(2024, 2, 29);
        for basis in [DayCount::Act365L, DayCount::ActActIcma] {
            assert_eq!(
                checked(point, point, basis),
                Err(DayCountError::MissingCouponPeriod)
            );
        }
        assert_eq!(
            checked(point, point, DayCount::ThirtyE360Isda),
            Err(DayCountError::MissingMaturity)
        );
        assert_eq!(
            checked(point, point, DayCount::Bus252),
            Err(DayCountError::MissingBusinessDayPredicate)
        );
    }

    #[test]
    fn coupon_context_is_shared_without_inventing_schedule_defaults() {
        let start = d(2024, 4, 1);
        let end = d(2024, 7, 1);
        let context = DayCountContext {
            coupon: Some(CouponPeriod {
                start,
                end,
                frequency: 4,
            }),
            ..DayCountContext::default()
        };
        assert!(
            (try_fraction(start, end, DayCount::Act365L, context).unwrap() - 91.0 / 366.0).abs()
                < 1e-12
        );
        assert!(
            (try_fraction(start, end, DayCount::ActActIcma, context).unwrap() - 0.25).abs() < 1e-12
        );
        let invalid_context = DayCountContext {
            coupon: Some(CouponPeriod {
                frequency: 0,
                ..context.coupon.unwrap()
            }),
            ..context
        };
        for basis in [DayCount::Act365L, DayCount::ActActIcma] {
            assert_eq!(
                try_fraction(start, end, basis, invalid_context),
                Err(DayCountError::ZeroFrequency)
            );
        }
        // Unused context has no effect on a convention defined only by dates.
        assert!(
            (try_fraction(start, end, DayCount::Act360, invalid_context).unwrap() - 91.0 / 360.0)
                .abs()
                < 1e-12
        );
    }

    #[test]
    fn maturity_context_preserves_the_february_exception() {
        let start = d(2024, 1, 31);
        let end = d(2025, 2, 28);
        for (is_maturity, numerator) in [(true, 388.0), (false, 390.0)] {
            let context = DayCountContext {
                end_is_maturity: Some(is_maturity),
                ..DayCountContext::default()
            };
            assert!(
                (try_fraction(start, end, DayCount::ThirtyE360Isda, context).unwrap()
                    - numerator / 360.0)
                    .abs()
                    < 1e-12
            );
        }
    }

    #[test]
    fn predicate_is_used_only_for_business252() {
        let start = d(2026, 9, 1);
        let end = d(2026, 9, 10);
        let is_business = |date: Date| {
            !matches!(
                date.day_of_week(),
                crate::Weekday::Sat | crate::Weekday::Sun
            ) && date != d(2026, 9, 7)
        };
        let result = try_fraction_with_business_days(
            start,
            end,
            DayCount::Bus252,
            DayCountContext::default(),
            is_business,
        )
        .unwrap();
        assert!((result - 6.0 / 252.0).abs() < 1e-12);
        let result = try_fraction_with_business_days(
            start,
            end,
            DayCount::Act360,
            DayCountContext::default(),
            |_| panic!("date-only convention must not consult the predicate"),
        )
        .unwrap();
        assert!((result - 9.0 / 360.0).abs() < 1e-12);
    }

    #[test]
    fn context_free_dispatch_is_finite_at_both_date_boundaries() {
        let first = d(1583, 1, 1);
        let last = d(9999, 12, 31);
        for basis in [
            DayCount::Act360,
            DayCount::Act365F,
            DayCount::ActActIsda,
            DayCount::Thirty360BondBasis,
            DayCount::ThirtyE360,
            DayCount::Nl365,
            DayCount::OneOne,
        ] {
            for (start, end) in [(first, last), (last, first), (first, first), (last, last)] {
                assert!(checked(start, end, basis).unwrap().is_finite(), "{basis:?}");
            }
        }
    }
}
