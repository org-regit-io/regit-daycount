// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! Catalogue of day-count fractions.
//!
//! A day-count fraction is the rule that maps a date interval `(start, end)`
//! to a year-fraction `f`. It is the parameter that turns a daily interest
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
//! 1/1                  OIS shortcut
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
//! The source-compatible [`fraction`] wrapper returns `NaN` whenever the
//! checked entry point cannot calculate without context. In particular,
//! Act/365L and 30E/360 ISDA no longer invent schedule or maturity defaults.
//! [`year_fraction_with_freq`] supplies coupon context for both Act/365L
//! and Act/Act ICMA; its checked equivalent is [`try_year_fraction_with_freq`].
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
    /// 1 / 1 — the OIS shortcut; every interval has fraction 1.
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
/// responsible for supplying the contractual schedule and for decomposing
/// irregular coupons that span multiple reference periods.
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

/// Computes the year fraction under `basis` between two dates.
///
/// Legacy context-free wrapper around [`try_fraction`]. Act/365L,
/// Act/Act ICMA, 30E/360 ISDA and Bus/252 require additional context and
/// return `NaN`. All other variants compute directly.
///
/// # Examples
///
/// ```
/// use regit_daycount::{Date, DayCount, day_count};
///
/// let start = Date::ymd(2026, 1, 1).unwrap();
/// let end   = Date::ymd(2026, 4, 1).unwrap();
/// let yf    = day_count::fraction(start, end, DayCount::Act360);
/// assert!((yf - 90.0 / 360.0).abs() < 1e-12);
///
/// // ActActIcma needs a reference period — fraction returns NaN.
/// assert!(day_count::fraction(start, end, DayCount::ActActIcma).is_nan());
/// ```
#[must_use]
pub fn fraction(start: Date, end: Date, basis: DayCount) -> f64 {
    try_fraction(start, end, basis, DayCountContext::default()).unwrap_or(f64::NAN)
}

/// Computes a year fraction using an explicit coupon period and frequency.
///
/// Act/Act ICMA and Act/365L consume the coupon arguments. Other variants
/// delegate to [`fraction`]; 30E/360 ISDA and Bus/252 still return `NaN`
/// because the signature lacks maturity and business-day context.
/// Invalid coupon context also returns `NaN`; use
/// [`try_year_fraction_with_freq`] for a typed error.
///
/// # Examples
///
/// ```
/// use regit_daycount::{Date, DayCount, day_count};
///
/// // ActAct ICMA — semi-annual coupon, regular period.
/// let start = Date::ymd(2026, 3, 1).unwrap();
/// let end   = Date::ymd(2026, 9, 1).unwrap();
/// let yf = day_count::year_fraction_with_freq(
///     start, end, DayCount::ActActIcma, 2, start, end,
/// );
/// assert!((yf - 0.5).abs() < 1e-12);
/// ```
#[must_use]
pub fn year_fraction_with_freq(
    start: Date,
    end: Date,
    basis: DayCount,
    freq: u8,
    ref_start: Date,
    ref_end: Date,
) -> f64 {
    try_year_fraction_with_freq(start, end, basis, freq, ref_start, ref_end).unwrap_or(f64::NAN)
}

/// Computes a year fraction using a single explicit reference coupon period.
///
/// The coupon arguments are used by Act/Act ICMA and Act/365L only.
///
/// # Errors
///
/// Returns the same errors as [`try_fraction`]. 30E/360 ISDA requires an
/// explicit maturity flag, and Bus/252 requires a business-day predicate;
/// this signature cannot supply those, so it returns a missing-context error.
pub fn try_year_fraction_with_freq(
    start: Date,
    end: Date,
    basis: DayCount,
    freq: u8,
    ref_start: Date,
    ref_end: Date,
) -> Result<f64, DayCountError> {
    try_fraction(
        start,
        end,
        basis,
        DayCountContext {
            coupon: Some(CouponPeriod {
                start: ref_start,
                end: ref_end,
                frequency: freq,
            }),
            end_is_maturity: None,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOL: f64 = 1e-12;

    // ─── fraction dispatcher ─────────────────────────────────────────────

    #[test]
    fn dispatcher_routes_act_360() {
        let s = Date::ymd(2026, 1, 1).unwrap();
        let e = Date::ymd(2026, 4, 1).unwrap();
        assert!((fraction(s, e, DayCount::Act360) - 90.0 / 360.0).abs() < TOL);
    }

    #[test]
    fn dispatcher_routes_act_365f() {
        let s = Date::ymd(2026, 1, 1).unwrap();
        let e = Date::ymd(2026, 4, 1).unwrap();
        assert!((fraction(s, e, DayCount::Act365F) - 90.0 / 365.0).abs() < TOL);
    }

    #[test]
    fn dispatcher_routes_act_act_isda() {
        // ISDA 2006 §4.16(b) worked example: 2007-12-28 → 2008-02-29
        // = 4/365 + 59/366 = 0.172161089901939.
        let s = Date::ymd(2007, 12, 28).unwrap();
        let e = Date::ymd(2008, 2, 29).unwrap();
        let expected = 4.0 / 365.0 + 59.0 / 366.0;
        assert!((fraction(s, e, DayCount::ActActIsda) - expected).abs() < TOL);
    }

    #[test]
    fn dispatcher_routes_thirty_360_bond_basis() {
        let s = Date::ymd(2026, 1, 15).unwrap();
        let e = Date::ymd(2026, 7, 15).unwrap();
        assert!((fraction(s, e, DayCount::Thirty360BondBasis) - 0.5).abs() < TOL);
    }

    #[test]
    fn dispatcher_routes_thirty_e_360() {
        // The 30E/360 case where D2 = 31 is unconditionally rewritten to
        // 30 — 2026-01-15 → 2026-07-31 gives 195/360 here vs 196/360 under
        // BondBasis.
        let s = Date::ymd(2026, 1, 15).unwrap();
        let e = Date::ymd(2026, 7, 31).unwrap();
        assert!((fraction(s, e, DayCount::ThirtyE360) - 195.0 / 360.0).abs() < TOL);
    }

    #[test]
    fn dispatcher_requires_thirty_e_360_isda_maturity_context() {
        let s = Date::ymd(2024, 2, 29).unwrap();
        let e = Date::ymd(2024, 8, 31).unwrap();
        assert!(fraction(s, e, DayCount::ThirtyE360Isda).is_nan());
    }

    #[test]
    fn dispatcher_requires_act_365l_coupon_context() {
        let s = Date::ymd(2024, 1, 1).unwrap();
        let e = Date::ymd(2024, 4, 1).unwrap();
        assert!(fraction(s, e, DayCount::Act365L).is_nan());
    }

    #[test]
    fn dispatcher_routes_nl_365() {
        // Q1 2024: 91 actual days minus 1 for 2024-02-29 = 90; /365.
        let s = Date::ymd(2024, 1, 1).unwrap();
        let e = Date::ymd(2024, 4, 1).unwrap();
        assert!((fraction(s, e, DayCount::Nl365) - 90.0 / 365.0).abs() < TOL);
    }

    #[test]
    fn dispatcher_routes_one_one() {
        let s = Date::ymd(2026, 1, 1).unwrap();
        let e = Date::ymd(2026, 4, 1).unwrap();
        assert!((fraction(s, e, DayCount::OneOne) - 1.0).abs() < TOL);
    }

    #[test]
    fn dispatcher_returns_nan_for_act_act_icma() {
        let s = Date::ymd(2026, 3, 1).unwrap();
        let e = Date::ymd(2026, 9, 1).unwrap();
        assert!(fraction(s, e, DayCount::ActActIcma).is_nan());
    }

    #[test]
    fn dispatcher_returns_nan_for_bus_252() {
        let s = Date::ymd(2026, 5, 1).unwrap();
        let e = Date::ymd(2026, 5, 15).unwrap();
        assert!(fraction(s, e, DayCount::Bus252).is_nan());
    }

    // ─── year_fraction_with_freq dispatcher ──────────────────────────────

    #[test]
    fn freq_dispatcher_routes_act_act_icma() {
        let s = Date::ymd(2026, 3, 1).unwrap();
        let e = Date::ymd(2026, 9, 1).unwrap();
        let yf = year_fraction_with_freq(s, e, DayCount::ActActIcma, 2, s, e);
        assert!((yf - 0.5).abs() < TOL);
    }

    #[test]
    fn freq_dispatcher_ignores_ref_args_for_other_variants() {
        // Conventions independent of coupon context ignore these fields.
        let s = Date::ymd(2026, 1, 1).unwrap();
        let e = Date::ymd(2026, 4, 1).unwrap();
        let with_freq = year_fraction_with_freq(s, e, DayCount::Act360, 99, s, e);
        let direct = fraction(s, e, DayCount::Act360);
        assert!((with_freq - direct).abs() < TOL);
    }

    #[test]
    fn freq_dispatcher_still_nan_for_bus_252() {
        let s = Date::ymd(2026, 5, 1).unwrap();
        let e = Date::ymd(2026, 5, 15).unwrap();
        let yf = year_fraction_with_freq(s, e, DayCount::Bus252, 1, s, e);
        assert!(yf.is_nan());
    }
}
