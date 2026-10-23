// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "calendars")]

use regit_daycount::calendar::{self, CalendarError, Composite, CoveragePolicy, JointBusiness};
use regit_daycount::{Calendar, Date, Roll};

fn d(year: i32, month: u8, day: u8) -> Date {
    Date::ymd(year, month, day).unwrap()
}

#[test]
fn official_nyse_saturday_new_year_exception_covers_every_affected_table_year() {
    // NYSE's published 2022 and 2028 calendars explicitly say Saturday
    // New Year's Day has no holiday observation; audit the same table rule
    // in every year where it affects a Friday within 2020–2040.
    for year in [2021, 2027, 2032, 2038] {
        for cal in [Calendar::UnitedStates, Calendar::UnitedStatesNyse] {
            assert!(
                calendar::try_is_business_day_with_policy(
                    d(year, 12, 31),
                    cal,
                    CoveragePolicy::AllowProjected,
                )
                .unwrap()
            );
        }
    }
    assert!(calendar::try_is_business_day(d(2027, 12, 31), Calendar::UnitedStatesNyse).unwrap());
}

#[test]
fn independently_published_market_specific_us_closures() {
    let nyse = Calendar::UnitedStatesNyse;
    let fed = Calendar::UnitedStatesFederalReserve;
    // NYSE 2026 calendar and FRB Services 2026–2030 table disagree on
    // Good Friday, Saturday-holiday Friday observations, and these two
    // Reserve Bank holidays. Expected values are source facts, not formulas.
    for (date, nyse_open, fed_open) in [
        (d(2026, 4, 3), false, true),
        (d(2026, 7, 3), false, true),
        (d(2026, 10, 12), true, false),
        (d(2026, 11, 11), true, false),
        (d(2027, 6, 18), false, true),
        (d(2027, 12, 24), false, true),
    ] {
        assert_eq!(
            calendar::try_is_business_day(date, nyse),
            Ok(nyse_open),
            "NYSE {date:?}"
        );
        assert_eq!(
            calendar::try_is_business_day(date, fed),
            Ok(fed_open),
            "Fed {date:?}"
        );
    }
    // ICE's 2024-12-30 exceptional closure announcement. A single verified
    // date does not upgrade the complete 2025 year to verified coverage.
    assert!(
        !calendar::try_is_business_day_with_policy(
            d(2025, 1, 9),
            nyse,
            CoveragePolicy::AllowProjected
        )
        .unwrap()
    );
}

#[test]
fn singapore_and_luxembourg_source_regressions_and_partial_day_scope() {
    // MOM annual list: Hari Raya Puasa is Saturday 21 March, not Friday20.
    assert_eq!(
        calendar::try_is_business_day(d(2026, 3, 20), Calendar::Singapore),
        Ok(true)
    );
    assert_eq!(
        calendar::try_is_holiday(d(2026, 3, 21), Calendar::Singapore),
        Ok(true)
    );
    assert_eq!(
        calendar::try_add_business_days(d(2026, 3, 20), 1, Calendar::Singapore),
        Ok(d(2026, 3, 23))
    );
    // Luxembourg government/ING: Good Friday bank closure; Christmas Eve
    // afternoon-only closure leaves a partial business day in this model.
    assert_eq!(
        calendar::try_is_business_day(d(2026, 4, 3), Calendar::Luxembourg),
        Ok(false)
    );
    assert_eq!(
        calendar::try_is_business_day(d(2026, 12, 24), Calendar::Luxembourg),
        Ok(true)
    );
    assert_eq!(
        calendar::try_next_business_day(d(2026, 4, 3), Calendar::Luxembourg),
        Ok(d(2026, 4, 7))
    );
    assert_eq!(
        calendar::try_business_days_between(d(2026, 4, 2), d(2026, 4, 8), Calendar::Luxembourg),
        Ok(2)
    );
}

#[test]
fn coverage_is_checked_on_weekends_and_empty_and_reversed_intervals() {
    let cal = Calendar::UnitedStatesNyse;
    for date in [d(2019, 12, 31), d(2041, 12, 25), d(2041, 12, 28)] {
        assert_eq!(
            calendar::try_is_business_day(date, cal),
            Err(CalendarError::OutOfCoverage {
                calendar: cal,
                date
            })
        );
        assert!(calendar::try_business_days_between(date, date, cal).is_err());
        assert!(calendar::try_bus_252_for_calendar(date, date, cal).is_err());
        assert!(calendar::try_add_business_days(date, 0, cal).is_err());
    }
    assert!(calendar::try_business_days_between(d(2026, 1, 5), d(2019, 12, 31), cal).is_err());
    assert_eq!(
        calendar::try_business_days_between(d(2026, 1, 5), d(2026, 1, 2), cal),
        Ok(0)
    );
}

#[test]
fn projections_require_explicit_opt_in_and_never_expand_availability() {
    let cal = Calendar::Singapore;
    let projected = d(2028, 1, 3);
    assert_eq!(
        calendar::try_is_business_day(projected, cal),
        Err(CalendarError::UnverifiedDate {
            calendar: cal,
            date: projected
        })
    );
    assert!(
        calendar::try_is_business_day_with_policy(projected, cal, CoveragePolicy::AllowProjected)
            .is_ok()
    );
    assert!(
        calendar::try_is_business_day_with_policy(
            d(2041, 1, 1),
            cal,
            CoveragePolicy::AllowProjected
        )
        .is_err()
    );
    // We do not promote old prose into an externally verified horizon.
    assert_eq!(calendar::coverage(Calendar::Japan).verified_years, None);
    assert!(calendar::try_is_business_day(d(2026, 1, 5), Calendar::Japan).is_err());
    assert!(
        calendar::try_is_business_day_with_policy(
            d(2026, 1, 5),
            Calendar::Japan,
            CoveragePolicy::AllowProjected
        )
        .is_ok()
    );
}

#[test]
fn checked_walks_validate_every_date_including_horizon_crossings() {
    let cal = Calendar::Singapore;
    // First verified day is a holiday: Preceding would touch an unverified
    // earlier year. The starting date alone cannot validate this result.
    assert_eq!(
        calendar::try_previous_business_day(d(2026, 1, 1), cal),
        Err(CalendarError::UnverifiedDate {
            calendar: cal,
            date: d(2025, 12, 31)
        })
    );
    assert_eq!(
        calendar::try_add_business_days(d(2027, 12, 31), 1, cal),
        Err(CalendarError::UnverifiedDate {
            calendar: cal,
            date: d(2028, 1, 1)
        })
    );
    assert!(calendar::try_business_days_between(d(2027, 12, 31), d(2028, 1, 1), cal).is_err());
    assert!(calendar::try_bus_252_for_calendar(d(2028, 1, 1), d(2027, 12, 31), cal).is_err());
    // Availability is separate from verification: opt-in cannot cross2040.
    assert!(
        calendar::try_add_business_days_with_policy(
            d(2040, 12, 31),
            1,
            cal,
            CoveragePolicy::AllowProjected
        )
        .is_err()
    );
}

#[test]
fn rolls_signed_counts_zero_add_and_nearest_ties_use_expected_dates() {
    let cal = Calendar::Target2;
    assert_eq!(
        calendar::try_adjust(d(2026, 5, 31), Roll::ModifiedFollowing, cal),
        Ok(d(2026, 5, 29))
    );
    assert_eq!(
        calendar::try_adjust(d(2026, 3, 1), Roll::ModifiedPreceding, cal),
        Ok(d(2026, 3, 2))
    );
    // May1 holiday: Thu Apr30 and Mon May4 are both two days from May2.
    assert_eq!(
        calendar::try_adjust(d(2026, 5, 2), Roll::Nearest, cal),
        Ok(d(2026, 5, 4))
    );
    assert_eq!(
        calendar::try_add_business_days(d(2026, 5, 2), 0, cal),
        Ok(d(2026, 5, 4))
    );
    assert_eq!(
        calendar::try_add_business_days(d(2026, 5, 4), -1, cal),
        Ok(d(2026, 4, 30))
    );
    assert_eq!(
        calendar::try_bus_252_for_calendar(d(2026, 5, 4), d(2026, 4, 30), cal),
        Ok(-1.0 / 252.0)
    );
}

#[test]
fn checked_composites_do_not_hide_later_leg_coverage_errors() {
    static REGIONS: &[Calendar] = &[Calendar::Target2, Calendar::Japan];
    static LU_TARGET: &[Calendar] = &[Calendar::Target2, Calendar::Luxembourg];
    let composite = Composite::new(REGIONS);
    let joint = JointBusiness::new(REGIONS);
    let holiday = d(2026, 12, 25);
    // The first calendar is already closed, but Japan lacks audited annual
    // coverage. The error must survive the union/intersection computation.
    assert!(composite.try_is_holiday(holiday).is_err());
    assert!(composite.try_is_business_day(holiday).is_err());
    assert!(joint.try_adjust(holiday, Roll::Following).is_err());
    assert!(
        composite
            .try_is_holiday_with_policy(holiday, CoveragePolicy::AllowProjected)
            .unwrap()
    );
    let lu = JointBusiness::new(LU_TARGET);
    assert_eq!(
        lu.try_adjust(d(2026, 4, 3), Roll::Following),
        Ok(d(2026, 4, 7))
    );
    assert_eq!(
        lu.try_add_business_days(d(2026, 4, 2), 1),
        Ok(d(2026, 4, 7))
    );
    assert_eq!(
        lu.try_business_days_between(d(2026, 4, 2), d(2026, 4, 8)),
        Ok(2)
    );
    assert_eq!(
        lu.try_bus_252(d(2026, 4, 8), d(2026, 4, 2)),
        Ok(-2.0 / 252.0)
    );
}

#[test]
fn invalid_dates_date_boundaries_and_walk_caps_are_typed_errors() {
    let invalid = Date::ymd_unchecked(2026, 2, 30);
    assert_eq!(
        calendar::try_adjust(invalid, Roll::Unadjusted, Calendar::Target2),
        Err(CalendarError::InvalidDate { date: invalid })
    );
    let max = d(Date::MAX_YEAR, 12, 31);
    assert_eq!(
        calendar::try_add_business_days_with_policy(
            max,
            1,
            Calendar::Target2,
            CoveragePolicy::AllowProjected
        ),
        Err(CalendarError::DateBoundary { date: max })
    );
    assert_eq!(
        calendar::try_business_days_between_with_policy(
            d(2002, 1, 1),
            d(2203, 1, 1),
            Calendar::Target2,
            CoveragePolicy::AllowProjected
        ),
        Err(CalendarError::WalkLimitExceeded {
            limit: calendar::CALENDAR_MAX_STEPS
        })
    );
    // i32::MIN must not overflow when converted into a remaining count.
    assert_eq!(
        calendar::try_add_business_days_with_policy(
            d(5000, 1, 1),
            i32::MIN,
            Calendar::Target2,
            CoveragePolicy::AllowProjected
        ),
        Err(CalendarError::WalkLimitExceeded {
            limit: calendar::CALENDAR_MAX_STEPS
        })
    );
}
