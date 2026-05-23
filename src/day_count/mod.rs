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
//! Two free functions dispatch on the [`DayCount`] enum:
//!
//! - [`fraction`] — the common entry point: takes `(start, end, basis)` and
//!   returns the year fraction. Three variants need information the
//!   dispatcher cannot supply on its own:
//!   - [`DayCount::ActActIcma`] needs a reference period and frequency —
//!     use [`year_fraction_with_freq`] instead.
//!   - [`DayCount::Bus252`] needs a business-day predicate — call
//!     [`bus_252::fraction`] directly with the predicate from your
//!     calendar.
//!   - [`DayCount::ThirtyE360Isda`] takes an `end_is_maturity` flag — the
//!     dispatcher defaults it to `false`; for the maturity case call
//!     [`thirty_e_360_isda::fraction`] directly.
//!
//!   When the dispatcher cannot compute (the first two cases above), it
//!   returns `f64::NAN` so the error propagates loudly through arithmetic
//!   rather than silently being absorbed as `0.0`.
//!
//! - [`year_fraction_with_freq`] — dispatches `ActActIcma` to its full
//!   five-argument signature; for every other variant the `freq`,
//!   `ref_start`, and `ref_end` are ignored and the call delegates to
//!   [`fraction`].
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

/// Computes the year fraction under `basis` between two dates.
///
/// This is the common entry point for the eleven conventions in this crate.
/// Three variants need information `fraction` cannot supply on its own:
///
/// - [`DayCount::ActActIcma`] requires a reference period and frequency.
///   `fraction` returns `f64::NAN` for this variant; use
///   [`year_fraction_with_freq`] instead.
/// - [`DayCount::Bus252`] requires a business-day predicate. `fraction`
///   returns `f64::NAN` for this variant; call [`bus_252::fraction`]
///   directly with the predicate from your chosen calendar.
/// - [`DayCount::ThirtyE360Isda`] takes an `end_is_maturity` flag.
///   `fraction` defaults it to `false`; for the maturity case call
///   [`thirty_e_360_isda::fraction`] directly with the flag.
///
/// `f64::NAN` is the "cannot compute" sentinel because it propagates loudly
/// through arithmetic — silently absorbing the call as `0.0` would be the
/// most dangerous possible failure (a wrong cashflow with no warning).
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
    match basis {
        DayCount::Act360 => act_360::fraction(start, end),
        DayCount::Act365F => act_365f::fraction(start, end),
        DayCount::ActActIsda => act_act_isda::fraction(start, end),
        DayCount::Thirty360BondBasis => thirty_360_bond_basis::fraction(start, end),
        DayCount::ThirtyE360 => thirty_e_360::fraction(start, end),
        DayCount::ThirtyE360Isda => thirty_e_360_isda::fraction(start, end, false),
        DayCount::Act365L => act_365l::fraction(start, end),
        DayCount::Nl365 => nl_365::fraction(start, end),
        DayCount::OneOne => one_one::fraction(start, end),
        DayCount::ActActIcma | DayCount::Bus252 => f64::NAN,
    }
}

/// Computes the year fraction under `basis`, supplying a reference period
/// and frequency that [`DayCount::ActActIcma`] requires.
///
/// For [`DayCount::ActActIcma`] this delegates to [`act_act_icma::fraction`]
/// with the five-argument signature `(start, end, ref_start, ref_end,
/// freq)`. For every other variant the `freq` / `ref_start` / `ref_end` are
/// ignored and the call delegates to [`fraction`] — so [`DayCount::Bus252`]
/// still returns `f64::NAN` here too.
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
    match basis {
        DayCount::ActActIcma => act_act_icma::fraction(start, end, ref_start, ref_end, freq),
        _ => fraction(start, end, basis),
    }
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
    fn dispatcher_routes_thirty_e_360_isda_with_no_maturity_default() {
        // The dispatcher defaults `end_is_maturity = false` for
        // ThirtyE360Isda. 2024-02-29 → 2024-08-31 (!maturity): D1 29→30,
        // D2 31→30, f = 0.5.
        let s = Date::ymd(2024, 2, 29).unwrap();
        let e = Date::ymd(2024, 8, 31).unwrap();
        assert!((fraction(s, e, DayCount::ThirtyE360Isda) - 0.5).abs() < TOL);
    }

    #[test]
    fn dispatcher_routes_act_365l() {
        // Q1 2024 straddles 2024-02-29; the denominator is 366.
        let s = Date::ymd(2024, 1, 1).unwrap();
        let e = Date::ymd(2024, 4, 1).unwrap();
        assert!((fraction(s, e, DayCount::Act365L) - 91.0 / 366.0).abs() < TOL);
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
        // The ref/freq arguments are silently dropped for the ten other
        // variants; the call must equal `fraction(s, e, basis)`.
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
