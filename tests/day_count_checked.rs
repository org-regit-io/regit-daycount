// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! Public API regressions for explicit contractual context and leap boundaries.

use regit_daycount::day_count::{
    self, CouponPeriod, DayCountContext, DayCountError, act_act_icma, bus_252, nl_365,
};
use regit_daycount::{Date, DayCount, Weekday};

fn d(year: i32, month: u8, day: u8) -> Date {
    Date::ymd(year, month, day).unwrap()
}

fn coupon(start: Date, end: Date, frequency: u8) -> DayCountContext {
    DayCountContext {
        coupon: Some(CouponPeriod {
            start,
            end,
            frequency,
        }),
        ..DayCountContext::default()
    }
}

#[test]
fn missing_context_is_typed_and_legacy_wrappers_return_nan() {
    let start = d(2024, 1, 1);
    let end = d(2024, 7, 1);
    for (basis, error) in [
        (DayCount::Act365L, DayCountError::MissingCouponPeriod),
        (DayCount::ActActIcma, DayCountError::MissingCouponPeriod),
        (DayCount::ThirtyE360Isda, DayCountError::MissingMaturity),
        (DayCount::Bus252, DayCountError::MissingBusinessDayPredicate),
    ] {
        for accrual_end in [start, end] {
            assert_eq!(
                day_count::try_fraction(start, accrual_end, basis, DayCountContext::default()),
                Err(error)
            );
            assert!(day_count::fraction(start, accrual_end, basis).is_nan());
        }
    }
}

#[test]
fn checked_coupon_conventions_reject_invalid_context() {
    let start = d(2024, 1, 1);
    let end = d(2024, 7, 1);
    for basis in [DayCount::Act365L, DayCount::ActActIcma] {
        for (context, error) in [
            (coupon(start, end, 0), DayCountError::ZeroFrequency),
            (
                coupon(start, start, 2),
                DayCountError::InvalidReferencePeriod,
            ),
            (coupon(end, start, 2), DayCountError::InvalidReferencePeriod),
            (
                coupon(d(2024, 2, 1), end, 2),
                DayCountError::AccrualOutsideReferencePeriod,
            ),
            (
                coupon(start, d(2024, 6, 1), 2),
                DayCountError::AccrualOutsideReferencePeriod,
            ),
        ] {
            assert_eq!(
                day_count::try_fraction(start, end, basis, context),
                Err(error)
            );
            assert_eq!(
                day_count::try_fraction(end, start, basis, context),
                Err(error)
            );
        }
    }
}

#[test]
fn icma_preserves_reverse_and_equal_accruals_inside_ordered_reference() {
    let rs = d(2026, 3, 1);
    let re = d(2026, 9, 1);
    let end = d(2026, 6, 1);
    assert_eq!(act_act_icma::try_fraction(rs, end, rs, re, 2), Ok(0.25));
    assert_eq!(act_act_icma::try_fraction(end, rs, rs, re, 2), Ok(-0.25));
    assert_eq!(act_act_icma::try_fraction(end, end, rs, re, 2), Ok(0.0));
    assert_eq!(act_act_icma::try_fraction(re, re, rs, re, 2), Ok(0.0));
    assert_eq!(
        act_act_icma::try_fraction(d(2027, 1, 1), d(2027, 1, 1), rs, re, 2),
        Err(DayCountError::AccrualOutsideReferencePeriod)
    );
}

#[test]
fn legacy_icma_invalid_input_never_becomes_a_fake_zero() {
    let start = d(2024, 1, 1);
    let end = d(2024, 7, 1);
    for (rs, re, freq) in [(start, end, 0), (start, start, 2), (end, start, 2)] {
        assert!(act_act_icma::fraction(start, end, rs, re, freq).is_nan());
        assert!(
            day_count::year_fraction_with_freq(start, end, DayCount::ActActIcma, freq, rs, re)
                .is_nan()
        );
    }
    assert!(act_act_icma::fraction(start, d(2025, 1, 1), start, end, 2).is_nan());
}

#[test]
fn frequency_dispatcher_consumes_act365l_coupon_end_and_frequency() {
    let start = d(2024, 4, 1);
    let end = d(2024, 7, 1);
    let annual =
        day_count::year_fraction_with_freq(start, end, DayCount::Act365L, 1, start, d(2025, 4, 1));
    let quarterly =
        day_count::year_fraction_with_freq(start, end, DayCount::Act365L, 4, start, end);
    assert!((annual - 91.0 / 365.0).abs() < 1e-12);
    assert!((quarterly - 91.0 / 366.0).abs() < 1e-12);
    assert_eq!(
        day_count::try_year_fraction_with_freq(start, end, DayCount::Act365L, 0, start, end),
        Err(DayCountError::ZeroFrequency)
    );
}

#[test]
fn explicit_maturity_changes_the_final_february_coupon() {
    let start = d(2024, 1, 31);
    let end = d(2025, 2, 28);
    let result = |maturity| {
        day_count::try_fraction(
            start,
            end,
            DayCount::ThirtyE360Isda,
            DayCountContext {
                end_is_maturity: Some(maturity),
                ..DayCountContext::default()
            },
        )
        .unwrap()
    };
    assert!((result(true) - 388.0 / 360.0).abs() < 1e-12);
    assert!((result(false) - 390.0 / 360.0).abs() < 1e-12);
    // Equal/reverse arithmetic is convention-specific. This equal-date
    // maturity case intentionally matches QuantLib 1.44's ISDA convention.
    let leap = d(2024, 2, 29);
    let equal = day_count::try_fraction(
        leap,
        leap,
        DayCount::ThirtyE360Isda,
        DayCountContext {
            end_is_maturity: Some(true),
            ..DayCountContext::default()
        },
    )
    .unwrap();
    assert!((equal + 1.0 / 360.0).abs() < 1e-12);
}

// These exact integer-day ratios and zero/whole-year identities must agree.
#[test]
#[allow(clippy::float_cmp)]
fn nl365_matches_quantlib_no_leap_boundary_semantics() {
    for year in [2000, 2024, 2400] {
        let before = d(year, 2, 28);
        let leap = d(year, 2, 29);
        let after = d(year, 3, 1);
        assert_eq!(nl_365::fraction(before, leap), 0.0);
        assert_eq!(nl_365::fraction(leap, before), 0.0);
        assert_eq!(nl_365::fraction(leap, after), 1.0 / 365.0);
        assert_eq!(nl_365::fraction(after, leap), -1.0 / 365.0);
        assert_eq!(nl_365::fraction(leap, leap), 0.0);
        assert_eq!(nl_365::fraction(before, after), 1.0 / 365.0);
    }
    for year in [1900, 2100] {
        assert_eq!(nl_365::fraction(d(year, 2, 28), d(year, 3, 1)), 1.0 / 365.0);
    }
    assert_eq!(nl_365::fraction(d(2000, 2, 29), d(2400, 2, 29)), 400.0);
}

#[test]
fn explicit_business_predicate_dispatches_bus252() {
    let start = d(2026, 5, 1);
    let end = d(2026, 5, 15);
    let weekdays = |date: Date| !matches!(date.day_of_week(), Weekday::Sat | Weekday::Sun);
    let context = DayCountContext::default();
    let result =
        day_count::try_fraction_with_business_days(start, end, DayCount::Bus252, context, weekdays)
            .unwrap();
    assert!((result - 10.0 / 252.0).abs() < 1e-12);
    assert_eq!(
        day_count::try_fraction_with_business_days(
            start,
            end,
            DayCount::Act365L,
            context,
            weekdays
        ),
        Err(DayCountError::MissingCouponPeriod)
    );
}

// The all-open predicate gives exact integer-day ratios, not approximations.
#[test]
#[allow(clippy::float_cmp)]
fn bus252_counts_valid_intervals_beyond_the_old_two_century_cap() {
    let start = d(1800, 1, 1);
    let end = d(2100, 1, 1);
    let expected = f64::from(start.days_between(end)) / 252.0;
    assert_eq!(bus_252::fraction(start, end, |_| true), expected);
    assert_eq!(bus_252::fraction(end, start, |_| true), -expected);
    assert_eq!(bus_252::fraction(start, start, |_| true), 0.0);
    let last = d(Date::MAX_YEAR, 12, 31);
    assert_eq!(
        bus_252::fraction(last.add_days(-1), last, |_| true),
        1.0 / 252.0
    );
}

#[test]
fn bus252_signed_easter_endpoints_match_reference_relative_intervals() {
    // Executed QuantLib Business252(TARGET) with ordered endpoints and explicit sign:
    // Easter Friday April 3 and Monday April 6 are closed. Tuesday April 7
    // is excluded after normalizing, even when it is the supplied start.
    let thursday = d(2026, 4, 2);
    let saturday = d(2026, 4, 4);
    let tuesday = d(2026, 4, 7);
    let target_easter = |date: Date| {
        !matches!(date.day_of_week(), Weekday::Sat | Weekday::Sun)
            && date != d(2026, 4, 3)
            && date != d(2026, 4, 6)
    };
    for (start, end, days) in [
        (saturday, tuesday, 0),
        (tuesday, saturday, 0),
        (thursday, tuesday, 1),
        (tuesday, thursday, -1),
    ] {
        let actual = bus_252::fraction(start, end, target_easter);
        assert!((actual - f64::from(days) / 252.0).abs() < 1e-12);
    }
}
