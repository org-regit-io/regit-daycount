// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! Checked calendar access and bounded, coverage-aware walks.

use super::{Calendar, is_business_day, is_holiday};
use crate::{Date, Roll};
use core::fmt;

/// Whether checked operations may use data without complete primary-source
/// verification. `AllowProjected` includes unverified historical rows as well
/// as future projections; it does not extend the available date range.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoveragePolicy {
    /// Require the verified range returned by [`coverage`].
    #[default]
    VerifiedOnly,
    /// Accept all available rows/rules, including projections and unaudited years.
    AllowProjected,
}

/// Inclusive calendar horizons. Availability describes data or rule generation,
/// not a promise that every available row agrees with the market.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CalendarCoverage {
    /// Inclusive first and last years accepted with explicit projection opt-in.
    pub available_years: (i32, i32),
    /// Complete primary-source verified years, or `None` when no complete
    /// annual audit is retained. TARGET uses its verified standing rule from
    /// 2002 through the 2028 published horizon, not yearly historical snapshots.
    pub verified_years: Option<(i32, i32)>,
    /// Date on which the primary sources supporting `verified_years` were read.
    /// `None` means the snapshot has not been independently reverified.
    pub verified_on: Option<&'static str>,
}

/// Returns each market's available years and complete primary-source verified
/// annual horizon. Unpublished future years and unaudited history require
/// explicit [`CoveragePolicy::AllowProjected`] opt-in. B3 provides only 2026.
#[must_use]
pub const fn coverage(cal: Calendar) -> CalendarCoverage {
    use super::{
        brazil, hong_kong, japan, luxembourg, singapore, switzerland, target2, united_kingdom,
        united_states,
    };
    let (available_years, verified_years, verified_on) = match cal {
        Calendar::Target2 => (
            (2002, Date::MAX_YEAR),
            target2::VERIFIED_YEARS,
            target2::VERIFIED_ON,
        ),
        Calendar::Luxembourg => (
            (2019, Date::MAX_YEAR),
            luxembourg::VERIFIED_YEARS,
            luxembourg::VERIFIED_ON,
        ),
        Calendar::UnitedStatesNyse => (
            united_states::COVERAGE,
            united_states::NYSE_VERIFIED_YEARS,
            united_states::NYSE_VERIFIED_ON,
        ),
        Calendar::UnitedStatesFederalReserve => (
            united_states::COVERAGE,
            united_states::FEDERAL_RESERVE_VERIFIED_YEARS,
            united_states::FEDERAL_RESERVE_VERIFIED_ON,
        ),
        Calendar::UnitedKingdom => (
            united_kingdom::COVERAGE,
            united_kingdom::VERIFIED_YEARS,
            united_kingdom::VERIFIED_ON,
        ),
        Calendar::Japan => (japan::COVERAGE, japan::VERIFIED_YEARS, japan::VERIFIED_ON),
        Calendar::Switzerland => (
            switzerland::COVERAGE,
            switzerland::VERIFIED_YEARS,
            switzerland::VERIFIED_ON,
        ),
        Calendar::HongKong => (
            hong_kong::COVERAGE,
            hong_kong::VERIFIED_YEARS,
            hong_kong::VERIFIED_ON,
        ),
        Calendar::Singapore => (
            singapore::COVERAGE,
            singapore::VERIFIED_YEARS,
            singapore::VERIFIED_ON,
        ),
        Calendar::BrazilB3 => (
            brazil::COVERAGE,
            brazil::VERIFIED_YEARS,
            brazil::VERIFIED_ON,
        ),
    };
    CalendarCoverage {
        available_years,
        verified_years: Some(verified_years),
        verified_on: Some(verified_on),
    }
}

/// A checked calendar query or walk could not determine a reliable result.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalendarError {
    /// At least one named market is required to construct a composite.
    EmptyComposite,
    /// The calendar has no supported data/rule for the requested date.
    OutOfCoverage { calendar: Calendar, date: Date },
    /// Data exists, but its year has not been verified under the requested policy.
    UnverifiedDate { calendar: Calendar, date: Date },
    /// A walk would leave `Date::MIN_YEAR..=Date::MAX_YEAR` or fail to advance.
    DateBoundary { date: Date },
    /// A bounded walk exhausted its limit instead of returning a partial answer.
    WalkLimitExceeded { limit: u32 },
}

impl fmt::Display for CalendarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyComposite => {
                f.write_str("a composite calendar requires at least one market")
            }
            Self::OutOfCoverage { calendar, date } => {
                write!(f, "{calendar:?} has no coverage for {date:?}")
            }
            Self::UnverifiedDate { calendar, date } => {
                write!(f, "{calendar:?} is unverified for {date:?}")
            }
            Self::DateBoundary { date } => {
                write!(f, "calendar walk reached date boundary at {date:?}")
            }
            Self::WalkLimitExceeded { limit } => write!(f, "calendar walk exceeded {limit} steps"),
        }
    }
}

impl core::error::Error for CalendarError {}

/// Maximum one-day steps in checked business-day counts and additions.
/// A cap failure is an error, never a partial financial result.
pub const CALENDAR_MAX_STEPS: u32 = 365 * 200;
const ROLL_MAX_STEPS: u32 = 366;

fn ensure_coverage(date: Date, cal: Calendar, policy: CoveragePolicy) -> Result<(), CalendarError> {
    let horizon = coverage(cal);
    if date.year() < horizon.available_years.0 || date.year() > horizon.available_years.1 {
        return Err(CalendarError::OutOfCoverage {
            calendar: cal,
            date,
        });
    }
    if policy == CoveragePolicy::VerifiedOnly
        && !horizon
            .verified_years
            .is_some_and(|(lo, hi)| (lo..=hi).contains(&date.year()))
    {
        return Err(CalendarError::UnverifiedDate {
            calendar: cal,
            date,
        });
    }
    Ok(())
}

fn step(date: Date, direction: i32) -> Result<Date, CalendarError> {
    date.try_add_days(direction)
        .map_err(|_| CalendarError::DateBoundary { date })
}

/// Checked holiday query using verified data only.
///
/// # Errors
/// Returns an error for unsupported or unverified dates.
pub fn try_is_holiday(date: Date, cal: Calendar) -> Result<bool, CalendarError> {
    try_is_holiday_with_policy(date, cal, CoveragePolicy::VerifiedOnly)
}

/// Checked holiday query with explicit data policy.
///
/// # Errors
/// Returns an error for dates outside the selected coverage.
pub fn try_is_holiday_with_policy(
    date: Date,
    cal: Calendar,
    policy: CoveragePolicy,
) -> Result<bool, CalendarError> {
    ensure_coverage(date, cal, policy)?;
    Ok(is_holiday(date, cal))
}

/// Checked business-day query using verified data only. Coverage is checked
/// even on weekends, so an unknown Saturday does not disguise missing data.
///
/// # Errors
/// Returns an error for unsupported or unverified dates.
pub fn try_is_business_day(date: Date, cal: Calendar) -> Result<bool, CalendarError> {
    try_is_business_day_with_policy(date, cal, CoveragePolicy::VerifiedOnly)
}

/// Checked business-day query with explicit data policy.
///
/// # Errors
/// Returns an error for dates outside the selected coverage.
pub fn try_is_business_day_with_policy(
    date: Date,
    cal: Calendar,
    policy: CoveragePolicy,
) -> Result<bool, CalendarError> {
    ensure_coverage(date, cal, policy)?;
    Ok(is_business_day(date, cal))
}

fn walk(
    date: Date,
    direction: i32,
    predicate: &impl Fn(Date) -> Result<bool, CalendarError>,
) -> Result<Date, CalendarError> {
    let mut d = date;
    for n in 0..=ROLL_MAX_STEPS {
        if predicate(d)? {
            return Ok(d);
        }
        if n == ROLL_MAX_STEPS {
            break;
        }
        d = step(d, direction)?;
    }
    Err(CalendarError::WalkLimitExceeded {
        limit: ROLL_MAX_STEPS,
    })
}

// Modified rolls need only determine whether a business day exists inside
// the original month. Crossing that boundary already selects the fallback;
// no lookup in an irrelevant (possibly unavailable) next month is required.
fn walk_within_month(
    date: Date,
    direction: i32,
    predicate: &impl Fn(Date) -> Result<bool, CalendarError>,
) -> Result<Option<Date>, CalendarError> {
    let mut current = date;
    loop {
        if predicate(current)? {
            return Ok(Some(current));
        }
        if (direction > 0 && current.day() == Date::days_in_month(current.year(), current.month()))
            || (direction < 0 && current.day() == 1)
        {
            return Ok(None);
        }
        current = step(current, direction)?;
    }
}

pub(super) fn adjust_with_predicate(
    date: Date,
    conv: Roll,
    predicate: impl Fn(Date) -> Result<bool, CalendarError>,
) -> Result<Date, CalendarError> {
    // Even Unadjusted validates its input and the requested calendar coverage.
    let is_business = predicate(date)?;
    match conv {
        Roll::Unadjusted => Ok(date),
        Roll::Following => walk(date, 1, &predicate),
        Roll::Preceding => walk(date, -1, &predicate),
        Roll::ModifiedFollowing | Roll::EndOfMonth => {
            // EndOfMonth on a standalone date uses ModifiedFollowing.
            if let Some(following) = walk_within_month(date, 1, &predicate)? {
                Ok(following)
            } else {
                walk(date, -1, &predicate)
            }
        }
        Roll::ModifiedPreceding => {
            if let Some(preceding) = walk_within_month(date, -1, &predicate)? {
                Ok(preceding)
            } else {
                walk(date, 1, &predicate)
            }
        }
        Roll::Nearest => {
            if is_business {
                return Ok(date);
            }
            let mut forward = date;
            let mut backward = date;
            for _ in 0..ROLL_MAX_STEPS {
                forward = step(forward, 1)?;
                if predicate(forward)? {
                    return Ok(forward);
                }
                backward = step(backward, -1)?;
                if predicate(backward)? {
                    return Ok(backward);
                }
            }
            Err(CalendarError::WalkLimitExceeded {
                limit: ROLL_MAX_STEPS,
            })
        }
    }
}

/// Adjust using verified data, validating every date visited.
///
/// # Errors
/// Returns coverage, date-boundary, or walk-limit errors; no partial result.
pub fn try_adjust(date: Date, conv: Roll, cal: Calendar) -> Result<Date, CalendarError> {
    try_adjust_with_policy(date, conv, cal, CoveragePolicy::VerifiedOnly)
}

/// Adjust with explicit data policy. Every examined date must satisfy the
/// policy. Modified rolls stop at the month boundary before trying fallback.
///
/// # Errors
/// Returns coverage, date-boundary, or walk-limit errors; no partial result.
pub fn try_adjust_with_policy(
    date: Date,
    conv: Roll,
    cal: Calendar,
    policy: CoveragePolicy,
) -> Result<Date, CalendarError> {
    adjust_with_predicate(date, conv, |d| {
        try_is_business_day_with_policy(d, cal, policy)
    })
}

/// Next business day on or after `date`, using verified data.
///
/// # Errors
/// Returns coverage, date-boundary, or walk-limit errors.
pub fn try_next_business_day(date: Date, cal: Calendar) -> Result<Date, CalendarError> {
    try_adjust(date, Roll::Following, cal)
}

/// Next business day on or after `date` with an explicit data policy.
///
/// # Errors
/// Returns coverage, date-boundary, or walk-limit errors.
pub fn try_next_business_day_with_policy(
    date: Date,
    cal: Calendar,
    policy: CoveragePolicy,
) -> Result<Date, CalendarError> {
    try_adjust_with_policy(date, Roll::Following, cal, policy)
}

/// Previous business day on or before `date`, using verified data.
///
/// # Errors
/// Returns coverage, date-boundary, or walk-limit errors.
pub fn try_previous_business_day(date: Date, cal: Calendar) -> Result<Date, CalendarError> {
    try_adjust(date, Roll::Preceding, cal)
}

/// Previous business day on or before `date` with an explicit data policy.
///
/// # Errors
/// Returns coverage, date-boundary, or walk-limit errors.
pub fn try_previous_business_day_with_policy(
    date: Date,
    cal: Calendar,
    policy: CoveragePolicy,
) -> Result<Date, CalendarError> {
    try_adjust_with_policy(date, Roll::Preceding, cal, policy)
}

pub(super) fn count_with_predicate(
    start: Date,
    end: Date,
    predicate: impl Fn(Date) -> Result<bool, CalendarError>,
) -> Result<u32, CalendarError> {
    // Only dates counted in a nonempty half-open interval need market data.
    // In particular, the exclusive successor of a final covered day is valid.
    // Empty/inverted unsigned counts still validate both declared endpoints.
    predicate(start)?;
    if start >= end {
        predicate(end)?;
        return Ok(0);
    }
    let mut date = start;
    let mut count = 0;
    let mut steps = 0;
    while date < end {
        if steps == CALENDAR_MAX_STEPS {
            return Err(CalendarError::WalkLimitExceeded {
                limit: CALENDAR_MAX_STEPS,
            });
        }
        if predicate(date)? {
            count += 1;
        }
        date = step(date, 1)?;
        steps += 1;
    }
    Ok(count)
}

/// Count business days in `[start, end)` with verified data. Empty and
/// reversed intervals return zero after validating both endpoints.
///
/// # Errors
/// Returns errors for counted dates outside verified coverage, a date boundary,
/// or the walk limit. Empty/reversed intervals validate both endpoints.
pub fn try_business_days_between(
    start: Date,
    end: Date,
    cal: Calendar,
) -> Result<u32, CalendarError> {
    try_business_days_between_with_policy(start, end, cal, CoveragePolicy::VerifiedOnly)
}

/// Count business days in `[start, end)` with explicit data policy. Every
/// counted date must satisfy the policy; the exclusive end need not have data.
/// Empty/reversed intervals return zero after checking both endpoints.
///
/// # Errors
/// Returns coverage, date-boundary, or walk-limit errors; no partial count.
pub fn try_business_days_between_with_policy(
    start: Date,
    end: Date,
    cal: Calendar,
    policy: CoveragePolicy,
) -> Result<u32, CalendarError> {
    count_with_predicate(start, end, |d| {
        try_is_business_day_with_policy(d, cal, policy)
    })
}

pub(super) fn add_with_predicate(
    date: Date,
    n: i32,
    predicate: impl Fn(Date) -> Result<bool, CalendarError>,
) -> Result<Date, CalendarError> {
    predicate(date)?;
    if n == 0 {
        return walk(date, 1, &predicate);
    }
    let direction = if n > 0 { 1 } else { -1 };
    let mut remaining = n.unsigned_abs();
    let mut date = date;
    for _ in 0..CALENDAR_MAX_STEPS {
        date = step(date, direction)?;
        if predicate(date)? {
            remaining -= 1;
            if remaining == 0 {
                return Ok(date);
            }
        }
    }
    Err(CalendarError::WalkLimitExceeded {
        limit: CALENDAR_MAX_STEPS,
    })
}

/// Add signed business days with verified data. Zero rolls Following;
/// the starting day is excluded for nonzero additions.
///
/// # Errors
/// Returns coverage, date-boundary, or walk-limit errors; no partial date.
pub fn try_add_business_days(date: Date, n: i32, cal: Calendar) -> Result<Date, CalendarError> {
    try_add_business_days_with_policy(date, n, cal, CoveragePolicy::VerifiedOnly)
}

/// Add signed business days with an explicit data policy.
///
/// # Errors
/// Returns coverage, date-boundary, or walk-limit errors; no partial date.
pub fn try_add_business_days_with_policy(
    date: Date,
    n: i32,
    cal: Calendar,
    policy: CoveragePolicy,
) -> Result<Date, CalendarError> {
    add_with_predicate(date, n, |d| try_is_business_day_with_policy(d, cal, policy))
}

/// Bus/252 with verified calendar data. Reversed intervals negate the count.
///
/// # Errors
/// Returns coverage, date-boundary, or walk-limit errors; no partial fraction.
pub fn try_bus_252_for_calendar(
    start: Date,
    end: Date,
    cal: Calendar,
) -> Result<f64, CalendarError> {
    try_bus_252_for_calendar_with_policy(start, end, cal, CoveragePolicy::VerifiedOnly)
}

/// Bus/252 with explicit calendar data policy. Every counted date must satisfy
/// the policy; a nonempty interval's exclusive end need not have market data.
///
/// # Errors
/// Returns coverage, date-boundary, or walk-limit errors; no partial fraction.
pub fn try_bus_252_for_calendar_with_policy(
    start: Date,
    end: Date,
    cal: Calendar,
    policy: CoveragePolicy,
) -> Result<f64, CalendarError> {
    if start > end {
        return try_business_days_between_with_policy(end, start, cal, policy)
            .map(|n| -f64::from(n) / 252.0);
    }
    try_business_days_between_with_policy(start, end, cal, policy).map(|n| f64::from(n) / 252.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permanently_closed_predicate_errors_instead_of_partial_roll() {
        let date = Date::ymd(2026, 5, 1).unwrap();
        assert_eq!(
            adjust_with_predicate(date, Roll::Following, |_| Ok(false)),
            Err(CalendarError::WalkLimitExceeded {
                limit: ROLL_MAX_STEPS
            })
        );
        assert_eq!(
            adjust_with_predicate(date, Roll::Nearest, |_| Ok(false)),
            Err(CalendarError::WalkLimitExceeded {
                limit: ROLL_MAX_STEPS
            })
        );
    }

    #[test]
    fn date_boundary_and_stalled_walks_error() {
        let max = Date::ymd(Date::MAX_YEAR, 12, 31).unwrap();
        let min = Date::ymd(Date::MIN_YEAR, 1, 1).unwrap();
        assert_eq!(
            adjust_with_predicate(max, Roll::Following, |_| Ok(false)),
            Err(CalendarError::DateBoundary { date: max })
        );
        assert_eq!(
            adjust_with_predicate(min, Roll::Preceding, |_| Ok(false)),
            Err(CalendarError::DateBoundary { date: min })
        );
    }
    #[test]
    fn modified_roll_uses_known_month_before_unavailable_next_year() {
        let date = Date::ymd(2026, 12, 31).unwrap();
        let expected = Date::ymd(2026, 12, 30).unwrap();
        assert_eq!(
            try_adjust(date, Roll::ModifiedFollowing, Calendar::BrazilB3),
            Ok(expected)
        );
        assert_eq!(
            try_adjust(date, Roll::EndOfMonth, Calendar::BrazilB3),
            Ok(expected)
        );
        assert!(matches!(
            try_adjust(date, Roll::Following, Calendar::BrazilB3),
            Err(CalendarError::OutOfCoverage { .. })
        ));
    }

    #[test]
    fn unsupported_holidays_and_weekends_return_coverage_errors() {
        for (calendar, date) in [
            (Calendar::UnitedKingdom, Date::ymd(2045, 12, 25).unwrap()),
            (Calendar::UnitedKingdom, Date::ymd(2015, 12, 25).unwrap()),
            (Calendar::BrazilB3, Date::ymd(2027, 1, 2).unwrap()),
        ] {
            let expected = Err(CalendarError::OutOfCoverage { calendar, date });
            assert_eq!(try_is_holiday(date, calendar), expected);
            assert_eq!(try_is_business_day(date, calendar), expected);
        }
    }
    #[test]
    fn full_finite_b3_year_allows_the_exclusive_successor() {
        let start = Date::ymd(2026, 1, 1).unwrap();
        let end = Date::ymd(2027, 1, 1).unwrap();
        // 261 weekdays, less the 14 official weekday full-day closures.
        assert_eq!(
            try_business_days_between(start, end, Calendar::BrazilB3),
            Ok(247)
        );
        assert_eq!(
            try_bus_252_for_calendar(start, end, Calendar::BrazilB3),
            Ok(247.0 / 252.0)
        );
        assert_eq!(
            try_bus_252_for_calendar(end, start, Calendar::BrazilB3),
            Ok(-247.0 / 252.0)
        );
        // Crossing one day farther really requires the absent 2027 schedule.
        let beyond = Date::ymd(2027, 1, 2).unwrap();
        assert_eq!(
            try_business_days_between(start, beyond, Calendar::BrazilB3),
            Err(CalendarError::OutOfCoverage {
                calendar: Calendar::BrazilB3,
                date: end
            })
        );
        // An empty unsupported interval retains an explicit coverage error.
        assert_eq!(
            try_business_days_between(end, end, Calendar::BrazilB3),
            Err(CalendarError::OutOfCoverage {
                calendar: Calendar::BrazilB3,
                date: end
            })
        );
    }
}
