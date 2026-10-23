// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! Coverage-aware market calendars and business-day operations.
//!
//! Every market query returns a `Result` and enforces its verified coverage.
//! Explicit [`CoveragePolicy::AllowProjected`](crate::calendar::CoveragePolicy::AllowProjected) opt-in admits available
//! unaudited historical data and future projections, never absent data.
//! [`coverage`](crate::calendar::coverage) reports each market's boundaries and primary-source audit date.
//! Partial trading sessions remain business days in this date-only model.

use crate::date::{Date, Weekday};

/// A named holiday calendar.
///
/// The variants name the market calendars this crate ships. The
/// compound kinds (a union of holidays and an intersection of business
/// days) live in [`composite`], so that a primary calendar always has the
/// `Copy`-into-an-enum-tag shape that the dispatcher matches on.
///
/// # Examples
///
/// ```
/// use regit_daycount::Calendar;
///
/// let cal = Calendar::Target2;
/// assert_eq!(cal, Calendar::Target2);
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Calendar {
    /// TARGET2 — the euro-area RTGS calendar, defined by ECB fixed rule.
    Target2,
    /// Luxembourg full-day bank holidays, including Good Friday.
    /// Christmas Eve's afternoon-only closure is outside this date-only model.
    /// See [`coverage`] for the supported and verified horizons.
    Luxembourg,
    /// New York Stock Exchange full-day trading closures. Early closing
    /// sessions remain business days. Includes the 2025 Carter closure.
    UnitedStatesNyse,
    /// Federal Reserve Bank operating holidays. Good Friday is open;
    /// Columbus Day and Veterans Day are closed. Saturday fixed holidays
    /// do not close the preceding Friday. This is not the 24/7 `FedNow` service.
    UnitedStatesFederalReserve,
    /// England and Wales bank holidays. Scotland and Northern Ireland have
    /// distinct schedules and are not represented by this variant.
    UnitedKingdom,
    /// Japan — Tokyo Stock Exchange cash-market closures. JPX derivatives
    /// holiday trading is a separate calendar and is not represented.
    Japan,
    /// Switzerland — SIX Swiss Exchange trading calendar.
    Switzerland,
    /// Hong Kong — HKEX trading calendar.
    HongKong,
    /// Singapore — SGX trading calendar.
    Singapore,
    /// B3 listed-market full-day trading closures. Partial sessions remain
    /// business days; this is not a universal ANBIMA/OTC settlement calendar.
    BrazilB3,
}

pub mod brazil;
mod checked;
pub mod composite;
pub mod hong_kong;
pub mod japan;
pub mod luxembourg;
pub mod singapore;
pub mod switzerland;
pub mod target2;
pub mod united_kingdom;
pub mod united_states;

pub use checked::*;
pub use composite::{Composite, JointBusiness};

/// Whether `date` is Saturday or Sunday. This is a weekday classification,
/// independent of any market's holiday coverage.
#[must_use]
#[inline]
pub fn is_weekend(date: Date) -> bool {
    matches!(date.day_of_week(), Weekday::Sat | Weekday::Sun)
}

// Raw predicates are implementation details. Every public market query must
// establish coverage before reaching this dispatcher, even for a weekend.
pub(crate) fn is_holiday(date: Date, cal: Calendar) -> bool {
    match cal {
        Calendar::Target2 => target2::is_holiday(date),
        Calendar::Luxembourg => luxembourg::is_holiday(date),
        Calendar::UnitedStatesNyse => united_states::is_holiday(date),
        Calendar::UnitedStatesFederalReserve => united_states::is_federal_reserve_holiday(date),
        Calendar::UnitedKingdom => united_kingdom::is_holiday(date),
        Calendar::Japan => japan::is_holiday(date),
        Calendar::Switzerland => switzerland::is_holiday(date),
        Calendar::HongKong => hong_kong::is_holiday(date),
        Calendar::Singapore => singapore::is_holiday(date),
        Calendar::BrazilB3 => brazil::is_holiday(date),
    }
}

pub(crate) fn is_business_day(date: Date, cal: Calendar) -> bool {
    !is_weekend(date) && !is_holiday(date, cal)
}

#[cfg(test)]
fn assert_verified_schedule(cal: Calendar, first: i32, last: i32, holidays: &[(i32, u8, u8)]) {
    let mut date = Date::ymd(first, 1, 1).unwrap();
    let end = Date::ymd(last, 12, 31).unwrap();
    loop {
        let holiday = holidays.contains(&(date.year(), date.month(), date.day()));
        assert_eq!(try_is_holiday(date, cal), Ok(holiday), "{cal:?} {date:?}");
        assert_eq!(
            try_is_business_day(date, cal),
            Ok(!is_weekend(date) && !holiday),
            "{cal:?} {date:?}"
        );
        if date == end {
            break;
        }
        date = date.add_days(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weekends_are_independent_of_calendar_coverage() {
        assert!(is_weekend(Date::ymd(2026, 5, 23).unwrap()));
        assert!(is_weekend(Date::ymd(2026, 5, 24).unwrap()));
        assert!(!is_weekend(Date::ymd(2026, 5, 22).unwrap()));
    }
}
