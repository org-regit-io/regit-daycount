// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! Allocation-free calendar combinations with checked coverage on every leg.
//!
//! [`Composite`] represents the union of named holidays; [`JointBusiness`]
//! represents days when all markets are open. Their business-day semantics are
//! equivalent. Empty combinations are rejected. Every query validates every
//! market, so an early holiday cannot hide a later leg's missing coverage.
//!
//! ```
//! use regit_daycount::{Calendar, Date};
//! use regit_daycount::calendar::Composite;
//! static MARKETS: &[Calendar] = &[Calendar::Target2, Calendar::UnitedKingdom];
//! let calendar = Composite::new(MARKETS)?;
//! assert!(calendar.try_is_holiday(Date::ymd(2026, 12, 25)?)?);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use super::{Calendar, CalendarError, CoveragePolicy, checked, try_is_holiday_with_policy};
use crate::{Date, Roll};

/// Union of holidays across a nonempty static slice of named markets.
#[derive(Debug, Clone, Copy)]
pub struct Composite {
    kinds: &'static [Calendar],
}

/// Business days shared by every market in a nonempty static slice.
#[derive(Debug, Clone, Copy)]
pub struct JointBusiness {
    kinds: &'static [Calendar],
}

macro_rules! constructor {
    ($kind:ty) => {
        impl $kind {
            /// Construct a calendar from at least one named market.
            ///
            /// # Errors
            /// Returns [`CalendarError::EmptyComposite`] for an empty slice.
            pub const fn new(kinds: &'static [Calendar]) -> Result<Self, CalendarError> {
                if kinds.is_empty() {
                    Err(CalendarError::EmptyComposite)
                } else {
                    Ok(Self { kinds })
                }
            }

            /// Markets participating in this combination, in supplied order.
            #[must_use]
            pub const fn kinds(&self) -> &'static [Calendar] {
                self.kinds
            }
        }
    };
}

constructor!(Composite);
constructor!(JointBusiness);

// Checked combinators intentionally evaluate every leg: a holiday on the
// first leg must not hide unavailable data on a later leg.
fn checked_holiday(
    kinds: &[Calendar],
    date: Date,
    policy: CoveragePolicy,
) -> Result<bool, CalendarError> {
    if kinds.is_empty() {
        return Err(CalendarError::EmptyComposite);
    }
    let mut holiday = false;
    for &cal in kinds {
        holiday |= try_is_holiday_with_policy(date, cal, policy)?;
    }
    Ok(holiday)
}

// Both public combinators have identical business-day semantics. Keep their
// checked implementations together so validation and bounded walking agree.
macro_rules! checked_combinator {
    ($kind:ty) => {
        impl $kind {
            /// Checked union-of-holidays query using verified data only.
            ///
            /// # Errors
            /// Returns an error when any leg lacks verified coverage.
            pub fn try_is_holiday(&self, date: Date) -> Result<bool, CalendarError> {
                self.try_is_holiday_with_policy(date, CoveragePolicy::VerifiedOnly)
            }

            /// Checked holiday query; every leg must satisfy the policy.
            ///
            /// # Errors
            /// Returns an error for unavailable data on any leg.
            pub fn try_is_holiday_with_policy(
                &self,
                date: Date,
                policy: CoveragePolicy,
            ) -> Result<bool, CalendarError> {
                checked_holiday(self.kinds, date, policy)
            }

            /// Checked intersection of business days using verified data only.
            ///
            /// # Errors
            /// Returns an error when any leg lacks verified coverage.
            pub fn try_is_business_day(&self, date: Date) -> Result<bool, CalendarError> {
                self.try_is_business_day_with_policy(date, CoveragePolicy::VerifiedOnly)
            }

            /// Checked business query; every leg must satisfy the policy even
            /// when an earlier leg already classifies the date as closed.
            ///
            /// # Errors
            /// Returns an error for unavailable data on any leg.
            pub fn try_is_business_day_with_policy(
                &self,
                date: Date,
                policy: CoveragePolicy,
            ) -> Result<bool, CalendarError> {
                let holiday = checked_holiday(self.kinds, date, policy)?;
                Ok(!super::is_weekend(date) && !holiday)
            }

            /// Checked adjustment with verified data on every leg.
            ///
            /// # Errors
            /// Returns coverage, date-boundary, or walk-limit errors.
            pub fn try_adjust(&self, date: Date, conv: Roll) -> Result<Date, CalendarError> {
                self.try_adjust_with_policy(date, conv, CoveragePolicy::VerifiedOnly)
            }

            /// Checked adjustment with explicit policy on every traversed date.
            ///
            /// # Errors
            /// Returns coverage, date-boundary, or walk-limit errors.
            pub fn try_adjust_with_policy(
                &self,
                date: Date,
                conv: Roll,
                policy: CoveragePolicy,
            ) -> Result<Date, CalendarError> {
                checked::adjust_with_predicate(date, conv, |d| {
                    self.try_is_business_day_with_policy(d, policy)
                })
            }

            /// Checked business-day count in `[start, end)` with verified data.
            /// Every counted date is checked on every market. A nonempty
            /// interval's exclusive end need not have market data.
            ///
            /// # Errors
            /// Returns coverage, date-boundary, or walk-limit errors.
            pub fn try_business_days_between(
                &self,
                start: Date,
                end: Date,
            ) -> Result<u32, CalendarError> {
                self.try_business_days_between_with_policy(start, end, CoveragePolicy::VerifiedOnly)
            }

            /// Checked business-day count with explicit policy on every leg.
            ///
            /// # Errors
            /// Returns coverage, date-boundary, or walk-limit errors.
            pub fn try_business_days_between_with_policy(
                &self,
                start: Date,
                end: Date,
                policy: CoveragePolicy,
            ) -> Result<u32, CalendarError> {
                checked::count_with_predicate(start, end, |d| {
                    self.try_is_business_day_with_policy(d, policy)
                })
            }

            /// Checked signed business-day addition using verified data.
            ///
            /// # Errors
            /// Returns coverage, date-boundary, or walk-limit errors.
            pub fn try_add_business_days(&self, date: Date, n: i32) -> Result<Date, CalendarError> {
                self.try_add_business_days_with_policy(date, n, CoveragePolicy::VerifiedOnly)
            }

            /// Checked signed business-day addition with explicit data policy.
            ///
            /// # Errors
            /// Returns coverage, date-boundary, or walk-limit errors.
            pub fn try_add_business_days_with_policy(
                &self,
                date: Date,
                n: i32,
                policy: CoveragePolicy,
            ) -> Result<Date, CalendarError> {
                checked::add_with_predicate(date, n, |d| {
                    self.try_is_business_day_with_policy(d, policy)
                })
            }

            /// Checked signed Bus/252 fraction using verified data.
            ///
            /// # Errors
            /// Returns coverage, date-boundary, or walk-limit errors.
            pub fn try_bus_252(&self, start: Date, end: Date) -> Result<f64, CalendarError> {
                self.try_bus_252_with_policy(start, end, CoveragePolicy::VerifiedOnly)
            }

            /// Checked signed Bus/252 fraction with explicit data policy.
            ///
            /// # Errors
            /// Returns coverage, date-boundary, or walk-limit errors.
            pub fn try_bus_252_with_policy(
                &self,
                start: Date,
                end: Date,
                policy: CoveragePolicy,
            ) -> Result<f64, CalendarError> {
                let (lo, hi, sign) = if start > end {
                    (end, start, -1.0)
                } else {
                    (start, end, 1.0)
                };
                self.try_business_days_between_with_policy(lo, hi, policy)
                    .map(|n| sign * f64::from(n) / 252.0)
            }
        }
    };
}

checked_combinator!(Composite);
checked_combinator!(JointBusiness);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_calendars_are_rejected_at_construction() {
        assert!(matches!(
            Composite::new(&[]),
            Err(CalendarError::EmptyComposite)
        ));
        assert!(matches!(
            JointBusiness::new(&[]),
            Err(CalendarError::EmptyComposite)
        ));
    }

    #[test]
    fn holiday_on_first_leg_does_not_hide_missing_second_leg() {
        static MARKETS: &[Calendar] = &[Calendar::Target2, Calendar::BrazilB3];
        let date = Date::ymd(2027, 12, 25).unwrap();
        let expected = Err(CalendarError::OutOfCoverage {
            calendar: Calendar::BrazilB3,
            date,
        });
        let composite = Composite::new(MARKETS).unwrap();
        let joint = JointBusiness::new(MARKETS).unwrap();
        assert_eq!(composite.try_is_holiday(date), expected);
        assert_eq!(composite.try_is_business_day(date), expected);
        assert_eq!(joint.try_is_holiday(date), expected);
        assert_eq!(joint.try_is_business_day(date), expected);
    }

    #[test]
    fn queries_combine_holidays_and_weekends() {
        static MARKETS: &[Calendar] = &[Calendar::Target2, Calendar::UnitedStatesNyse];
        let calendar = Composite::new(MARKETS).unwrap();
        assert_eq!(calendar.kinds(), MARKETS);
        assert_eq!(
            calendar.try_is_holiday(Date::ymd(2026, 7, 3).unwrap()),
            Ok(true)
        );
        assert_eq!(
            calendar.try_is_business_day(Date::ymd(2026, 7, 3).unwrap()),
            Ok(false)
        );
        assert_eq!(
            calendar.try_is_business_day(Date::ymd(2026, 7, 4).unwrap()),
            Ok(false)
        );
        assert_eq!(
            calendar.try_is_business_day(Date::ymd(2026, 7, 6).unwrap()),
            Ok(true)
        );
    }
}
