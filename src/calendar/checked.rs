// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! Checked calendar access and bounded, coverage-aware walks.

use super::{Calendar, is_business_day, is_holiday};
use crate::{Date, Roll};
use core::fmt;

/// Whether checked operations may use data without complete primary-source
/// verification. `AllowProjected` includes unverified historical rows as well
/// as future projections; it does not extend the available date range.
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

/// Returns availability and verification metadata for a named calendar.
///
/// Verified NYSE years are 2026–2028; Federal Reserve 2026–2030;
/// Luxembourg and Singapore 2026–2027. Other static tables currently have
/// no complete independently audited year and require projection opt-in.
/// TARGET's standing six-day closure rule is effective from 2002 and has
/// been checked against the ECB decision and current 2026–2028 calendars.
/// Later rule-generated dates require projection opt-in.
#[must_use]
pub const fn coverage(cal: Calendar) -> CalendarCoverage {
    let available_years = match cal {
        Calendar::Target2 => (2002, Date::MAX_YEAR),
        Calendar::Luxembourg => (2019, Date::MAX_YEAR),
        _ => (2020, 2040),
    };
    let verified_years = match cal {
        Calendar::Target2 => Some((2002, 2028)),
        Calendar::UnitedStates | Calendar::UnitedStatesNyse => Some((2026, 2028)),
        Calendar::UnitedStatesFederalReserve => Some((2026, 2030)),
        Calendar::Luxembourg | Calendar::Singapore => Some((2026, 2027)),
        _ => None,
    };
    CalendarCoverage {
        available_years,
        verified_years,
        verified_on: if verified_years.is_some() {
            Some("2026-10-23")
        } else {
            None
        },
    }
}

/// A checked calendar query or walk could not determine a reliable result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalendarError {
    /// The supplied value is not a validated date in the supported date domain.
    InvalidDate { date: Date },
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
            Self::InvalidDate { date } => write!(f, "invalid calendar date: {date:?}"),
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

fn validate(date: Date) -> Result<(), CalendarError> {
    Date::ymd(date.year(), date.month(), date.day())
        .map(|_| ())
        .map_err(|_| CalendarError::InvalidDate { date })
}

fn ensure_coverage(date: Date, cal: Calendar, policy: CoveragePolicy) -> Result<(), CalendarError> {
    validate(date)?;
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
    if (direction > 0 && date == Date::ymd_unchecked(Date::MAX_YEAR, 12, 31))
        || (direction < 0 && date == Date::ymd_unchecked(Date::MIN_YEAR, 1, 1))
    {
        return Err(CalendarError::DateBoundary { date });
    }
    let next = date.add_days(direction);
    if next == date || (direction > 0 && next < date) || (direction < 0 && next > date) {
        return Err(CalendarError::DateBoundary { date });
    }
    validate(next).map_err(|_| CalendarError::DateBoundary { date })?;
    Ok(next)
}

/// Checked holiday query using verified data only.
///
/// # Errors
/// Returns an error for invalid, unsupported, or unverified dates.
pub fn try_is_holiday(date: Date, cal: Calendar) -> Result<bool, CalendarError> {
    try_is_holiday_with_policy(date, cal, CoveragePolicy::VerifiedOnly)
}

/// Checked holiday query with explicit data policy.
///
/// # Errors
/// Returns an error for invalid dates or dates outside the selected coverage.
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
/// Returns an error for invalid, unsupported, or unverified dates.
pub fn try_is_business_day(date: Date, cal: Calendar) -> Result<bool, CalendarError> {
    try_is_business_day_with_policy(date, cal, CoveragePolicy::VerifiedOnly)
}

/// Checked business-day query with explicit data policy.
///
/// # Errors
/// Returns an error for invalid dates or dates outside the selected coverage.
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
            // EndOfMonth is the same operation on a standalone date as the
            // legacy API: a last business day is preserved, otherwise use MF.
            let following = walk(date, 1, &predicate)?;
            if following.month() == date.month() && following.year() == date.year() {
                Ok(following)
            } else {
                walk(date, -1, &predicate)
            }
        }
        Roll::ModifiedPreceding => {
            let preceding = walk(date, -1, &predicate)?;
            if preceding.month() == date.month() && preceding.year() == date.year() {
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

/// Adjust with explicit data policy. All dates examined, including those in a
/// discarded modified-roll direction, must satisfy the policy.
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
    // End is exclusive in the count but is still a declared input and must
    // satisfy coverage; empty/inverted intervals do not bypass validation.
    predicate(start)?;
    predicate(end)?;
    if start >= end {
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
/// Returns errors for endpoints or traversed dates outside verified coverage,
/// a date boundary, or the walk limit.
pub fn try_business_days_between(
    start: Date,
    end: Date,
    cal: Calendar,
) -> Result<u32, CalendarError> {
    try_business_days_between_with_policy(start, end, cal, CoveragePolicy::VerifiedOnly)
}

/// Count business days in `[start, end)` with explicit data policy. Both
/// endpoints, including the exclusive end, must satisfy the selected policy.
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

/// Add signed business days with verified data. Zero rolls Following, matching
/// the legacy API; the starting day is excluded for nonzero additions.
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

/// Bus/252 with explicit calendar data policy. Both endpoints are validated.
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
}
