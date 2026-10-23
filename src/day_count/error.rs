// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! Typed day-count context and validation errors.

use core::fmt;

/// A year fraction cannot be computed from the supplied contractual context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DayCountError {
    /// Act/365L or Act/Act ICMA requires an explicit reference coupon period.
    MissingCouponPeriod,
    /// 30E/360 ISDA requires an explicit end-is-maturity flag.
    MissingMaturity,
    /// Bus/252 requires an explicit business-day predicate.
    MissingBusinessDayPredicate,
    /// Coupon payments per year must be positive.
    ZeroFrequency,
    /// The reference coupon end must be strictly after its start.
    InvalidReferencePeriod,
    /// The accrual must be contained within the single reference period.
    AccrualOutsideReferencePeriod,
}

impl fmt::Display for DayCountError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::MissingCouponPeriod => {
                "day count requires a reference coupon period and frequency"
            }
            Self::MissingMaturity => "day count requires an explicit end-is-maturity flag",
            Self::MissingBusinessDayPredicate => "Bus/252 requires a business-day predicate",
            Self::ZeroFrequency => "coupon frequency must be positive",
            Self::InvalidReferencePeriod => "reference coupon end must be after its start",
            Self::AccrualOutsideReferencePeriod => {
                "accrual lies outside the single reference coupon period"
            }
        })
    }
}

impl core::error::Error for DayCountError {}
