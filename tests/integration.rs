// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for regit-daycount.
//!
//! Structure (see doc/WORKING.md §5):
//!
//! - mod golden — every fraction / calendar / roll against a named
//!   ISDA / ICMA / exchange-published example
//! - mod invalid — every fallible Date::ymd path and degenerate interval
//!   returns the right typed error / NaN
//! - mod roundtrip — Date::ymd(d.year(), d.month(), d.day()) round-trips;
//!   add_days(n).add_days(-n) is identity
//! - mod properties — proptest invariants
//! - mod cross_oracle — QuantLib daycounters test-suite vectors
//!   transcribed verbatim (with citing line numbers) plus hand-derived
//!   vectors for the conventions QuantLib does not carry (Act/365L,
//!   NL/365, our Bus/252 against a weekends-only / TARGET2 predicate)
//! - mod cross_module — Date ⇄ roll ⇄ calendar ⇄ day_count interactions
//!   (the integration the unit tests do not exercise)

// Test docstrings reference financial conventions (TARGET2, ISDA, ICMA,
// QuantLib, Act/360, ...) and module/file names (doc/ALGORITHMS.md,
// day_count) that look item-like to clippy::doc_markdown but are not
// Rust identifiers. Suppress at file scope so the per-comment noise of
// backticking every such reference doesn't drown out the actual prose.
#![allow(clippy::doc_markdown)]

use regit_daycount::day_count::{act_360, act_act_isda, thirty_e_360_isda};
use regit_daycount::errors::ValidationError;
use regit_daycount::{Date, DayCount, day_count};

#[cfg(feature = "calendars")]
use regit_daycount::calendar::{self, Composite, JointBusiness};
#[cfg(feature = "calendars")]
use regit_daycount::day_count::bus_252;
#[cfg(feature = "calendars")]
use regit_daycount::{Calendar, Roll, Weekday, roll};

/// Working numeric tolerance for the integration suite.
///
/// 1e-12 is the same slack used throughout the crate's per-module unit
/// tests: loose enough that a benign last-bit rounding never causes a
/// spurious failure, tight enough that any drift large enough to mis-state
/// a cashflow at the cent level (≥ 1e-9 on a unit notional) is caught.
const TOL: f64 = 1e-12;

// ─── Golden vectors ──────────────────────────────────────────────────────────

mod golden {
    use super::*;

    // ─── day-count fractions ────────────────────────────────────────────

    /// ISDA 2006 §4.16(b), the printed worked example.
    ///
    /// `start = 2007-12-28`, `end = 2008-02-29`. Days in 2007 = 4 (28, 29,
    /// 30, 31 December); 2007 is not a leap year, contribution = 4/365.
    /// Days in 2008 = 59 (31 Jan + 28 Feb); 2008 is a leap year,
    /// contribution = 59/366. Sum = 4/365 + 59/366 = 0.172_161_089_901_939.
    #[test]
    fn isda_act_act_2007_to_2008() {
        let start = Date::ymd(2007, 12, 28).unwrap();
        let end = Date::ymd(2008, 2, 29).unwrap();
        let f = act_act_isda::fraction(start, end);
        let want = 4.0_f64 / 365.0 + 59.0_f64 / 366.0;
        assert!((f - want).abs() < TOL, "Act/Act ISDA: {f} vs {want}");
        assert!((f - 0.172_161_089_901_939).abs() < TOL);
    }

    /// Act/360, doc/ALGORITHMS.md §A.1 worked example #1.
    #[test]
    fn act_360_90_days() {
        let start = Date::ymd(2026, 1, 1).unwrap();
        let end = Date::ymd(2026, 4, 1).unwrap();
        let f = act_360::fraction(start, end);
        assert!((f - 0.25).abs() < TOL, "Act/360 90-day: {f}");
    }

    /// Act/365F, doc/ALGORITHMS.md §A.2 worked example.
    #[test]
    fn act_365f_90_days() {
        let start = Date::ymd(2026, 1, 1).unwrap();
        let end = Date::ymd(2026, 4, 1).unwrap();
        let f = day_count::fraction(start, end, DayCount::Act365F);
        assert!((f - 0.246_575_342_465_753).abs() < TOL, "Act/365F: {f}");
    }

    /// 30/360 BondBasis, doc/ALGORITHMS.md §A.5 worked example.
    #[test]
    fn thirty_360_6m() {
        let start = Date::ymd(2026, 1, 15).unwrap();
        let end = Date::ymd(2026, 7, 15).unwrap();
        let f = day_count::fraction(start, end, DayCount::Thirty360BondBasis);
        assert!((f - 0.5).abs() < TOL, "30/360 6M: {f}");
    }

    /// The two 30-day-month variants disagree on a `D2 = 31` endpoint when
    /// `D1 != 30, 31`. 30E/360 unconditionally collapses `D2` to 30 →
    /// numerator = 30*(7-1) + (30-15) = 195. 30/360 BondBasis leaves
    /// `D2 = 31` because `D1 = 15` is neither 30 nor 31 → numerator =
    /// 30*(7-1) + (31-15) = 196. The 1/360 swing is the documented
    /// convention difference.
    #[test]
    fn thirty_e_360_vs_bond_basis_d2_31() {
        let start = Date::ymd(2026, 1, 15).unwrap();
        let end = Date::ymd(2026, 7, 31).unwrap();
        let e_360 = day_count::fraction(start, end, DayCount::ThirtyE360);
        let bond = day_count::fraction(start, end, DayCount::Thirty360BondBasis);
        assert!((e_360 - 195.0_f64 / 360.0).abs() < TOL, "30E/360: {e_360}");
        assert!((bond - 196.0_f64 / 360.0).abs() < TOL, "BondBasis: {bond}");
        assert!((bond - e_360 - 1.0_f64 / 360.0).abs() < TOL);
    }

    /// 30E/360 ISDA load-bearing case — the February-at-maturity carve-out
    /// from §4.16(h). `start = 2024-01-31`, `end = 2025-02-28`,
    /// `end_is_maturity = true`. D1 = 31 → 30 (last day of January);
    /// D2 = 28 is the last day of Feb but the suppression rule keeps D2 at
    /// 28. Numerator = 360 + 30 - 2 = 388.
    #[test]
    fn thirty_e_360_isda_maturity_feb_28() {
        let start = Date::ymd(2024, 1, 31).unwrap();
        let end = Date::ymd(2025, 2, 28).unwrap();
        let f = thirty_e_360_isda::fraction(start, end, true);
        assert!((f - 388.0_f64 / 360.0).abs() < TOL, "30E/360 ISDA: {f}");
    }

    // ─── holiday calendars ──────────────────────────────────────────────

    /// Known TARGET2 holidays for 2024. Good Friday 2024 = 2024-03-29
    /// (Easter Sunday 2024 = 2024-03-31), Easter Monday = 2024-04-01,
    /// plus the fixed-date Christmas and Boxing Day.
    #[cfg(feature = "calendars")]
    #[test]
    fn target2_known_holidays() {
        for &(y, m, d) in &[
            (2024, 3, 29),  // Good Friday
            (2024, 4, 1),   // Easter Monday
            (2024, 12, 25), // Christmas
            (2024, 12, 26), // Boxing Day
        ] {
            assert!(
                calendar::is_holiday(Date::ymd(y, m, d).unwrap(), Calendar::Target2),
                "TARGET2 holiday {y}-{m}-{d}",
            );
        }
    }

    /// Two named US holidays from the published Federal Reserve list.
    #[cfg(feature = "calendars")]
    #[test]
    fn us_known_holidays() {
        for &(y, m, d) in &[
            (2024, 7, 4),   // Independence Day
            (2024, 11, 28), // Thanksgiving
        ] {
            assert!(
                calendar::is_holiday(Date::ymd(y, m, d).unwrap(), Calendar::UnitedStates),
                "US holiday {y}-{m}-{d}",
            );
        }
    }

    /// 2022 UK dated specials — the Spring Bank Holiday displaced to Thu
    /// 2 June for the Platinum Jubilee, and the extra Friday 3 June
    /// granted for the occasion.
    #[cfg(feature = "calendars")]
    #[test]
    fn uk_platinum_jubilee_2022() {
        assert!(calendar::is_holiday(
            Date::ymd(2022, 6, 2).unwrap(),
            Calendar::UnitedKingdom,
        ));
        assert!(calendar::is_holiday(
            Date::ymd(2022, 6, 3).unwrap(),
            Calendar::UnitedKingdom,
        ));
    }

    /// `JointBusiness([Target2, UnitedStates])` — 2024-07-04 is a US
    /// holiday but a TARGET2 business day; the joint-business calendar
    /// must report it as not a business day.
    #[cfg(feature = "calendars")]
    #[test]
    fn joint_us_eur_4_july_not_business() {
        static LEGS: &[Calendar] = &[Calendar::Target2, Calendar::UnitedStates];
        let jb = JointBusiness::new(LEGS);
        assert!(!jb.is_business_day(Date::ymd(2024, 7, 4).unwrap()));
        // And it agrees that the date is a holiday on the joint.
        assert!(jb.is_holiday(Date::ymd(2024, 7, 4).unwrap()));
    }

    /// Bus/252 via the `calendar::bus_252_for_calendar` convenience —
    /// reproduces the per-module worked example. `[2026-05-01, 2026-05-15)`
    /// under TARGET2 has 9 business days (May 1 is Labour Day, a TARGET2
    /// holiday); fraction = 9 / 252.
    #[cfg(feature = "calendars")]
    #[test]
    fn bus_252_target2_two_weeks() {
        let s = Date::ymd(2026, 5, 1).unwrap();
        let e = Date::ymd(2026, 5, 15).unwrap();
        let f = calendar::bus_252_for_calendar(s, e, Calendar::Target2);
        assert!((f - 9.0_f64 / 252.0).abs() < TOL, "Bus/252 TARGET2: {f}");
    }
}

// ─── Invalid vectors ─────────────────────────────────────────────────────────

mod invalid {
    use super::*;

    #[test]
    fn ymd_rejects_february_29_in_non_leap() {
        assert_eq!(
            Date::ymd(2025, 2, 29),
            Err(ValidationError::InvalidDate {
                rule: "day-out-of-range",
            }),
        );
    }

    #[test]
    fn ymd_rejects_year_below_1583() {
        assert_eq!(
            Date::ymd(1582, 1, 1),
            Err(ValidationError::OutOfRange {
                what: "year < 1583",
            }),
        );
    }

    #[test]
    fn ymd_rejects_month_zero_and_thirteen() {
        assert_eq!(
            Date::ymd(2026, 0, 1),
            Err(ValidationError::InvalidDate {
                rule: "month-out-of-range",
            }),
        );
        assert_eq!(
            Date::ymd(2026, 13, 1),
            Err(ValidationError::InvalidDate {
                rule: "month-out-of-range",
            }),
        );
    }

    /// `ActActIcma` requires a reference period; the no-frequency
    /// dispatcher cannot compute and returns NaN so the loud failure
    /// propagates through downstream arithmetic.
    #[test]
    fn dispatcher_returns_nan_for_act_act_icma_without_freq() {
        let s = Date::ymd(2026, 3, 1).unwrap();
        let e = Date::ymd(2026, 9, 1).unwrap();
        assert!(day_count::fraction(s, e, DayCount::ActActIcma).is_nan());
    }

    /// `Bus252` requires a business-day predicate; the dispatcher returns
    /// NaN. The full-frequency variant likewise propagates NaN.
    #[test]
    fn dispatcher_returns_nan_for_bus_252_without_predicate() {
        let s = Date::ymd(2026, 5, 1).unwrap();
        let e = Date::ymd(2026, 5, 15).unwrap();
        assert!(day_count::fraction(s, e, DayCount::Bus252).is_nan());
        // The freq-aware dispatcher also returns NaN for Bus252.
        assert!(day_count::year_fraction_with_freq(s, e, DayCount::Bus252, 1, s, e).is_nan(),);
    }

    /// A snapshot calendar queried outside its coverage window returns
    /// `false`, not a panic. The US snapshot starts in 2020; 2019-12-25
    /// is a real US holiday but lies outside the embedded table, and the
    /// lookup must report the conservative `false`.
    #[cfg(feature = "calendars")]
    #[test]
    fn out_of_coverage_calendar_returns_false_not_panic() {
        assert!(!calendar::is_holiday(
            Date::ymd(2019, 12, 25).unwrap(),
            Calendar::UnitedStates,
        ));
    }
}

// ─── Round-trips ─────────────────────────────────────────────────────────────

mod roundtrip {
    use super::*;

    /// `Date::ymd(d.year(), d.month(), d.day()) == Ok(d)` for every named
    /// real-world reference. The set spans leap days, century boundaries,
    /// market-moving anniversaries, and dates the rest of this suite uses.
    #[test]
    fn ymd_roundtrip_for_named_dates() {
        for &(y, m, d) in &[
            (1969, 7, 20),  // Apollo 11 Moon landing
            (1980, 12, 12), // Apple's first trading day (ISIN birthday)
            (1999, 12, 31), // Last day of the 1900s
            (2000, 1, 1),   // Y2K
            (2007, 12, 28), // ISDA worked-example start
            (2008, 2, 29),  // ISDA worked-example end (leap day 2008)
            (2020, 2, 29),  // Leap day 2020
            (2024, 2, 29),  // Leap day 2024
            (2026, 5, 23),  // The repository's "today" anchor
            (2038, 1, 19),  // Y2K38 (32-bit time_t rollover date)
            (9999, 12, 31), // Largest accepted date
        ] {
            let d0 = Date::ymd(y, m, d).expect("named date should parse");
            let d1 = Date::ymd(d0.year(), d0.month(), d0.day());
            assert_eq!(d1, Ok(d0), "round-trip {y}-{m}-{d}");
        }
    }

    /// `add_days(n).add_days(-n) == d` over a small deterministic grid.
    #[test]
    fn add_days_round_trips() {
        let anchors = [
            Date::ymd(2026, 5, 23).unwrap(),
            Date::ymd(2024, 2, 29).unwrap(),
        ];
        for &d in &anchors {
            for &n in &[-1000_i32, -1, 0, 1, 1000] {
                assert_eq!(d.add_days(n).add_days(-n), d, "anchor {d:?} offset {n}");
            }
        }
    }

    /// `day_of_week()` is a pure function of the date — two calls return
    /// the same weekday. The test exercises a span that crosses several
    /// month boundaries and a leap day.
    #[test]
    fn day_of_week_is_deterministic() {
        let mut d = Date::ymd(2024, 2, 25).unwrap();
        for _ in 0..10 {
            assert_eq!(d.day_of_week(), d.day_of_week(), "stable for {d:?}");
            d = d.add_days(1);
        }
    }
}

// ─── Property-based invariants ───────────────────────────────────────────────

mod properties {
    use super::*;
    use proptest::prelude::*;

    // A `Date` strategy that stays inside the supported `[1583, 9999]`
    // window. Built by generating `(year, month, day-of-year)` and
    // resolving via `add_days` so every output is a real, in-range
    // Gregorian date — `Date::ymd` would reject roughly a third of
    // random `(year, month, day)` triples in February alone.
    prop_compose! {
        fn arb_date()
            (year in 1583i32..=9999, doy in 0u32..366)
            -> Date
        {
            // Jan 1 of `year` is always in range; adding [0, 364] days
            // stays inside the same calendar year for the longest month
            // sequence. For `year = 9999` we cap `doy` at 364 so the
            // walk lands on 9999-12-31 at worst. For every other year,
            // the cap is 365 (366 next-year-Jan-1 dates also stay in
            // the supported window).
            let base = Date::ymd_unchecked(year, 1, 1);
            let cap = if year == 9999 { 364 } else { 365 };
            let n = core::cmp::min(doy, cap);
            // `add_days` accepts an `i32`; `n` is at most 365.
            #[allow(clippy::cast_possible_wrap)]
            let n_i32 = n as i32;
            base.add_days(n_i32)
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            ..ProptestConfig::default()
        })]

        /// For every day-count basis except `OneOne` (definitionally 1.0),
        /// `Bus252` (NaN through the dispatcher), and `ActActIcma` (NaN
        /// through the dispatcher), `fraction(d, d, basis) == 0.0`.
        #[test]
        fn prop_fraction_is_zero_on_zero_length(d in arb_date()) {
            for &basis in &[
                DayCount::Act360,
                DayCount::Act365F,
                DayCount::ActActIsda,
                DayCount::Thirty360BondBasis,
                DayCount::ThirtyE360,
                DayCount::ThirtyE360Isda,
                DayCount::Act365L,
                DayCount::Nl365,
            ] {
                let f = day_count::fraction(d, d, basis);
                prop_assert!(f.abs() < TOL, "basis {basis:?}: {f}");
            }
        }

        /// Act/360 is exactly additive: for any `a <= b <= c`,
        /// `fraction(a, c, Act360) == fraction(a, b, Act360) +
        /// fraction(b, c, Act360)`. The day count is an integer, so the
        /// equality holds within rounding.
        #[test]
        fn prop_act_360_is_additive(
            a in arb_date(),
            n1 in 0i32..3_650,
            n2 in 0i32..3_650,
        ) {
            let b = a.add_days(n1);
            let c = b.add_days(n2);
            let lhs = day_count::fraction(a, c, DayCount::Act360);
            let rhs = day_count::fraction(a, b, DayCount::Act360)
                    + day_count::fraction(b, c, DayCount::Act360);
            prop_assert!((lhs - rhs).abs() < TOL, "lhs={lhs}, rhs={rhs}");
        }

        /// `add_days(n).add_days(-n) == d` for any `n` in
        /// `[-10_000, 10_000]` on any in-range anchor.
        #[test]
        fn prop_add_days_round_trip(
            d in arb_date(),
            n in -10_000i32..10_000,
        ) {
            prop_assert_eq!(d.add_days(n).add_days(-n), d);
        }
    }

    // `is_holiday` is a pure function of `(date, calendar)` — two calls
    // return the same value. Property-checked across every named calendar.
    // The neighbouring `prop_is_weekend_xor_business_day_or_holiday`
    // property is grouped into the same `proptest!` block because both
    // depend on the `calendars` feature.
    #[cfg(feature = "calendars")]
    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            ..ProptestConfig::default()
        })]

        #[test]
        fn prop_is_holiday_is_deterministic(d in arb_date()) {
            for &cal in &[
                Calendar::Target2,
                Calendar::UnitedStates,
                Calendar::UnitedKingdom,
                Calendar::Japan,
                Calendar::Switzerland,
                Calendar::HongKong,
                Calendar::Singapore,
            ] {
                prop_assert_eq!(
                    calendar::is_holiday(d, cal),
                    calendar::is_holiday(d, cal),
                );
            }
        }

        // Exactly one of {business_day, non-business} holds for every
        // date — and `non-business` decomposes as `weekend OR holiday`.
        // The two cases (weekend and holiday) may both fire on the same
        // date when a fixed-date holiday falls on a Saturday or Sunday
        // (e.g. Christmas Day in years where 25 December is on a Sat),
        // so we partition into business vs non-business, then check the
        // non-business set is exactly `weekend OR holiday`.
        #[test]
        fn prop_is_weekend_xor_business_day_or_holiday(d in arb_date()) {
            for &cal in &[
                Calendar::Target2,
                Calendar::UnitedStates,
                Calendar::UnitedKingdom,
                Calendar::Japan,
                Calendar::Switzerland,
                Calendar::HongKong,
                Calendar::Singapore,
            ] {
                let wknd = calendar::is_weekend(d);
                let hol = calendar::is_holiday(d, cal);
                let biz = calendar::is_business_day(d, cal);
                // Exactly one of `business_day` / `non-business` is true.
                prop_assert!(
                    biz != (wknd || hol),
                    "biz/non-biz partition fails at {:?} ({:?})", d, cal,
                );
                // `is_business_day` is the crate's documented
                // composition `!is_weekend && !is_holiday`.
                prop_assert_eq!(biz, !wknd && !hol);
            }
        }
    }

    /// `OneOne` returns `1.0` even on a zero-length interval — the
    /// definitional case the broader zero-length property has to skip.
    #[test]
    fn one_one_is_one_on_zero_length() {
        let d = Date::ymd(2026, 5, 23).unwrap();
        assert!((day_count::fraction(d, d, DayCount::OneOne) - 1.0).abs() < TOL);
    }
}

// ─── Cross-oracle vectors against QuantLib ───────────────────────────────────

/// Cross-oracle vectors transcribed from the QuantLib daycounters test
/// suite at `test-suite/daycounters.cpp` (pinned to release tag `v1.34`,
/// commit reachable from
/// <https://github.com/lballabio/QuantLib/blob/v1.34/test-suite/daycounters.cpp>).
///
/// Each test below cites the QuantLib source line for every vector it
/// asserts against. The QuantLib reference numbers themselves are printed
/// to 12 decimal places in `daycounters.cpp`; the comparison tolerance is
/// the crate-wide `TOL = 1e-12`, the same slack the per-module unit tests
/// use.
///
/// Conventions transcribed:
///
/// - Act/Act ISDA  ↔ `ActualActual::ISDA` (daycounters.cpp lines 133–213)
/// - Act/Act ICMA  ↔ `ActualActual::ISMA` (daycounters.cpp lines 137–209,
///   regular-period vectors only — the irregular-period schedule tests at
///   lines 273–841 use the QuantLib `Schedule` infrastructure we do not
///   replicate here)
/// - 30/360 Bond Basis    ↔ `Thirty360::BondBasis`    (lines 506–536)
/// - 30E/360 Eurobond     ↔ `Thirty360::EurobondBasis`(lines 549–585)
/// - 30E/360 ISDA         ↔ `Thirty360::ISDA`         (lines 606–647)
///
/// Conventions NOT covered by direct QuantLib vectors (and the rationale):
///
/// - Act/360, Act/365F — no dedicated QuantLib test (these are exercised
///   via `Actual360` / `Actual365Fixed` consistency checks rather than
///   hard-coded vectors). The hand-derived vectors below are computed
///   directly from the §4.16(e) / §4.16(d) algorithms and round-trip
///   through `days_between`.
/// - Act/365L, NL/365 — neither is a QuantLib day counter; vectors are
///   hand-derived per the module-level docstrings.
/// - Bus/252 — QuantLib carries `Business252(Brazil)`, which depends on
///   Brazilian B3 / ANBIMA holidays. This crate does not ship a Brazilian
///   calendar (the snapshot tables cover TARGET2 / US / UK / JP / CH / HK
///   / SG only), so QuantLib's Bus/252 numbers cannot be reproduced
///   without bolting on a Brazil snapshot. We cross-check the algorithm
///   against a weekends-only predicate instead, which is verifiable by
///   inspection.
/// - 1/1 — definitionally constant; the per-module test covers it.
mod cross_oracle {
    use super::*;

    /// Comparison tolerance for vectors transcribed from the QuantLib test
    /// suite.
    ///
    /// QuantLib's `daycounters.cpp` prints each expected value with twelve
    /// decimal digits — e.g. `0.497724380567`. The trailing digits are a
    /// round-and-truncate of the C++ `double` QuantLib itself produces, so
    /// the printed constant differs from the exact value by up to a few
    /// units of the twelfth decimal place. Comparing our internally-exact
    /// `f64` to the printed constant at the crate-wide `1e-12` tolerance
    /// occasionally fails on a vector whose printed digits were rounded
    /// down rather than to nearest — `0.502737956204` vs the true
    /// `0.502737956204...`, plus a few `× 1e-12` from QuantLib's own
    /// formatting. Use `1e-10` for the QuantLib comparisons, which is
    /// loose enough to absorb the printf noise without admitting any
    /// drift relevant to a cashflow: 1e-10 on a unit notional is still
    /// well below a cent.
    ///
    /// The non-QuantLib hand-derived vectors below use the standard
    /// crate-wide `TOL = 1e-12`.
    const QL_TOL: f64 = 1e-10;

    // ─── Act/Act ISDA (QuantLib ActualActual::ISDA) ──────────────────────

    /// QuantLib daycounters.cpp:133 — 2003-11-01 → 2004-05-01 = 0.497_724_380_567.
    /// Hand-check: 2003 contributes (61/365); 2004 contributes (121/366).
    #[test]
    fn quantlib_act_act_isda_2003_11_01_to_2004_05_01() {
        let s = Date::ymd(2003, 11, 1).unwrap();
        let e = Date::ymd(2004, 5, 1).unwrap();
        let f = day_count::fraction(s, e, DayCount::ActActIsda);
        assert!((f - 0.497_724_380_567).abs() < TOL, "{f}");
    }

    /// QuantLib daycounters.cpp:145 — 1999-02-01 → 1999-07-01 = 0.410_958_904_110.
    /// Single non-leap year: 150 days / 365 = 0.410958904109589.
    #[test]
    fn quantlib_act_act_isda_1999_02_01_to_1999_07_01() {
        let s = Date::ymd(1999, 2, 1).unwrap();
        let e = Date::ymd(1999, 7, 1).unwrap();
        let f = day_count::fraction(s, e, DayCount::ActActIsda);
        assert!((f - 0.410_958_904_110).abs() < TOL, "{f}");
    }

    /// QuantLib daycounters.cpp:157 — 1999-07-01 → 2000-07-01 = 1.001_377_348_600.
    /// 1999 (non-leap) contributes 184/365; 2000 (leap) contributes 182/366.
    /// QuantLib's constant is printed to twelve decimal digits; the exact
    /// value is `184/365 + 182/366 = 1.0013773486039374...`. The printed
    /// constant differs from the exact value by ~4e-12 — within the
    /// printf-truncation envelope but outside `TOL = 1e-12`. We assert
    /// against the exact rational AND, separately, against the printed
    /// constant at the wider QuantLib tolerance.
    #[test]
    fn quantlib_act_act_isda_1999_07_01_to_2000_07_01() {
        let s = Date::ymd(1999, 7, 1).unwrap();
        let e = Date::ymd(2000, 7, 1).unwrap();
        let f = day_count::fraction(s, e, DayCount::ActActIsda);
        let exact = 184.0_f64 / 365.0 + 182.0_f64 / 366.0;
        assert!((f - exact).abs() < TOL, "{f}");
        assert!((f - 1.001_377_348_600).abs() < QL_TOL, "{f}");
    }

    /// QuantLib daycounters.cpp:169 — 2002-08-15 → 2003-07-15 = 0.915_068_493_151.
    /// Both years non-leap: (139 + 195) / 365 = 334/365.
    #[test]
    fn quantlib_act_act_isda_2002_08_15_to_2003_07_15() {
        let s = Date::ymd(2002, 8, 15).unwrap();
        let e = Date::ymd(2003, 7, 15).unwrap();
        let f = day_count::fraction(s, e, DayCount::ActActIsda);
        assert!((f - 0.915_068_493_151).abs() < TOL, "{f}");
    }

    /// QuantLib daycounters.cpp:181 — 2003-07-15 → 2004-01-15 = 0.504_004_790_778.
    #[test]
    fn quantlib_act_act_isda_2003_07_15_to_2004_01_15() {
        let s = Date::ymd(2003, 7, 15).unwrap();
        let e = Date::ymd(2004, 1, 15).unwrap();
        let f = day_count::fraction(s, e, DayCount::ActActIsda);
        assert!((f - 0.504_004_790_778).abs() < TOL, "{f}");
    }

    /// QuantLib daycounters.cpp:205 — 2000-01-30 → 2000-06-30 = 0.415_300_546_448.
    /// Same-year leap fast path: 152/366.
    #[test]
    fn quantlib_act_act_isda_2000_01_30_to_2000_06_30() {
        let s = Date::ymd(2000, 1, 30).unwrap();
        let e = Date::ymd(2000, 6, 30).unwrap();
        let f = day_count::fraction(s, e, DayCount::ActActIsda);
        assert!((f - 0.415_300_546_448).abs() < TOL, "{f}");
    }

    // ─── Act/Act ICMA (QuantLib ActualActual::ISMA, regular period) ──────

    /// QuantLib daycounters.cpp:149 — 1999-02-01 → 1999-07-01 with ISMA
    /// equals the ISDA value 0.410_958_904_110 when the reference period
    /// is the full year and freq = 1 (annual). QuantLib's ISMA, when fed
    /// a calculation period sitting inside a single annual reference
    /// period, computes `calc_days / (1 * ref_days) = 150 / 365`.
    #[test]
    fn quantlib_act_act_icma_1999_02_01_to_1999_07_01_annual_ref() {
        let s = Date::ymd(1999, 2, 1).unwrap();
        let e = Date::ymd(1999, 7, 1).unwrap();
        let ref_start = Date::ymd(1999, 1, 1).unwrap();
        let ref_end = Date::ymd(2000, 1, 1).unwrap();
        let f =
            day_count::year_fraction_with_freq(s, e, DayCount::ActActIcma, 1, ref_start, ref_end);
        assert!((f - 0.410_958_904_110).abs() < TOL, "{f}");
    }

    /// QuantLib daycounters.cpp:137 — 2003-11-01 → 2004-05-01 with a
    /// semi-annual reference period that coincides exactly with the
    /// calculation period yields exactly 0.5 by the ICMA `days / (freq *
    /// days)` cancellation.
    #[test]
    fn quantlib_act_act_icma_full_semiannual_period_is_half() {
        let s = Date::ymd(2003, 11, 1).unwrap();
        let e = Date::ymd(2004, 5, 1).unwrap();
        let f = day_count::year_fraction_with_freq(s, e, DayCount::ActActIcma, 2, s, e);
        assert!((f - 0.500_000_000_000).abs() < TOL, "{f}");
    }

    /// QuantLib daycounters.cpp:161 — 1999-07-01 → 2000-07-01 with an
    /// annual reference period spanning the same dates is exactly 1.0
    /// (calc_days == ref_days, freq = 1).
    #[test]
    fn quantlib_act_act_icma_full_annual_period_is_one() {
        let s = Date::ymd(1999, 7, 1).unwrap();
        let e = Date::ymd(2000, 7, 1).unwrap();
        let f = day_count::year_fraction_with_freq(s, e, DayCount::ActActIcma, 1, s, e);
        assert!((f - 1.000_000_000_000).abs() < TOL, "{f}");
    }

    // ─── 30/360 Bond Basis (QuantLib Thirty360::BondBasis) ───────────────

    /// QuantLib daycounters.cpp:514 — 2006-08-31 → 2007-02-28 = 178/360.
    /// D1 = 31 → 30; D2 = 28 (not 31) → stays. Num = 360 - 180 - 2 = 178.
    #[test]
    fn quantlib_bond_basis_2006_08_31_to_2007_02_28() {
        let s = Date::ymd(2006, 8, 31).unwrap();
        let e = Date::ymd(2007, 2, 28).unwrap();
        let f = day_count::fraction(s, e, DayCount::Thirty360BondBasis);
        assert!((f - 178.0_f64 / 360.0).abs() < TOL, "{f}");
    }

    /// QuantLib daycounters.cpp:515 — 2007-02-28 → 2007-08-31 = 183/360.
    /// D1 = 28; D2 = 31 with D1 ∉ {30, 31} → D2 stays 31.
    /// Num = 30*6 + (31 - 28) = 183.
    #[test]
    fn quantlib_bond_basis_2007_02_28_to_2007_08_31() {
        let s = Date::ymd(2007, 2, 28).unwrap();
        let e = Date::ymd(2007, 8, 31).unwrap();
        let f = day_count::fraction(s, e, DayCount::Thirty360BondBasis);
        assert!((f - 183.0_f64 / 360.0).abs() < TOL, "{f}");
    }

    /// QuantLib daycounters.cpp:516 — 2007-08-31 → 2008-02-29 = 179/360.
    /// D1 = 31 → 30; D2 = 29 (not 31) → stays. Num = 360 - 180 - 1 = 179.
    #[test]
    fn quantlib_bond_basis_2007_08_31_to_2008_02_29() {
        let s = Date::ymd(2007, 8, 31).unwrap();
        let e = Date::ymd(2008, 2, 29).unwrap();
        let f = day_count::fraction(s, e, DayCount::Thirty360BondBasis);
        assert!((f - 179.0_f64 / 360.0).abs() < TOL, "{f}");
    }

    /// QuantLib daycounters.cpp:531 — 2008-02-28 → 2008-08-31 = 183/360.
    /// D1 = 28; D2 = 31 with D1 ∉ {30, 31} → D2 stays 31. The signature
    /// of the BondBasis vs 30E/360 distinction: 30E/360 collapses to 182.
    #[test]
    fn quantlib_bond_basis_2008_02_28_to_2008_08_31() {
        let s = Date::ymd(2008, 2, 28).unwrap();
        let e = Date::ymd(2008, 8, 31).unwrap();
        let f = day_count::fraction(s, e, DayCount::Thirty360BondBasis);
        assert!((f - 183.0_f64 / 360.0).abs() < TOL, "{f}");
    }

    /// QuantLib daycounters.cpp:534 — 2008-02-29 → 2009-02-28 = 359/360.
    /// D1 = 29; D2 = 28; no collapses fire. Num = 360 + 0 + (28-29) = 359.
    #[test]
    fn quantlib_bond_basis_2008_02_29_to_2009_02_28() {
        let s = Date::ymd(2008, 2, 29).unwrap();
        let e = Date::ymd(2009, 2, 28).unwrap();
        let f = day_count::fraction(s, e, DayCount::Thirty360BondBasis);
        assert!((f - 359.0_f64 / 360.0).abs() < TOL, "{f}");
    }

    /// QuantLib daycounters.cpp:536 — 2008-02-28 → 2008-03-31 = 33/360.
    /// D1 = 28; D2 = 31 with D1 ∉ {30, 31} → D2 stays. Num = 30 + 3 = 33.
    #[test]
    fn quantlib_bond_basis_2008_02_28_to_2008_03_31() {
        let s = Date::ymd(2008, 2, 28).unwrap();
        let e = Date::ymd(2008, 3, 31).unwrap();
        let f = day_count::fraction(s, e, DayCount::Thirty360BondBasis);
        assert!((f - 33.0_f64 / 360.0).abs() < TOL, "{f}");
    }

    // ─── 30E/360 Eurobond (QuantLib Thirty360::EurobondBasis) ────────────

    /// QuantLib daycounters.cpp:557 — 2006-02-28 → 2006-08-31 = 182/360.
    /// 30E/360 unconditionally collapses D2 = 31 → 30. Num = 30*6 + 2 = 182.
    #[test]
    fn quantlib_eurobond_2006_02_28_to_2006_08_31() {
        let s = Date::ymd(2006, 2, 28).unwrap();
        let e = Date::ymd(2006, 8, 31).unwrap();
        let f = day_count::fraction(s, e, DayCount::ThirtyE360);
        assert!((f - 182.0_f64 / 360.0).abs() < TOL, "{f}");
    }

    /// QuantLib daycounters.cpp:559 — 2007-02-28 → 2007-08-31 = 182/360.
    /// Mirrors the prior test (different year, same shape).
    #[test]
    fn quantlib_eurobond_2007_02_28_to_2007_08_31() {
        let s = Date::ymd(2007, 2, 28).unwrap();
        let e = Date::ymd(2007, 8, 31).unwrap();
        let f = day_count::fraction(s, e, DayCount::ThirtyE360);
        assert!((f - 182.0_f64 / 360.0).abs() < TOL, "{f}");
    }

    /// QuantLib daycounters.cpp:561 — 2008-02-29 → 2008-08-31 = 181/360.
    /// D1 = 29 (NOT 31 → no collapse under 30E/360), D2 = 31 → 30.
    /// Num = 30*6 + (30-29) = 181. The BondBasis answer would be 182.
    #[test]
    fn quantlib_eurobond_2008_02_29_to_2008_08_31() {
        let s = Date::ymd(2008, 2, 29).unwrap();
        let e = Date::ymd(2008, 8, 31).unwrap();
        let f = day_count::fraction(s, e, DayCount::ThirtyE360);
        assert!((f - 181.0_f64 / 360.0).abs() < TOL, "{f}");
    }

    /// QuantLib daycounters.cpp:580 — 2008-02-28 → 2008-08-31 = 182/360.
    /// 30E/360 collapses D2 unconditionally; the BondBasis answer would
    /// be 183.
    #[test]
    fn quantlib_eurobond_2008_02_28_to_2008_08_31() {
        let s = Date::ymd(2008, 2, 28).unwrap();
        let e = Date::ymd(2008, 8, 31).unwrap();
        let f = day_count::fraction(s, e, DayCount::ThirtyE360);
        assert!((f - 182.0_f64 / 360.0).abs() < TOL, "{f}");
    }

    /// QuantLib daycounters.cpp:585 — 2008-02-28 → 2008-03-31 = 32/360.
    /// D1 = 28; D2 = 31 → 30 unconditionally. Num = 30 + (30-28) = 32.
    #[test]
    fn quantlib_eurobond_2008_02_28_to_2008_03_31() {
        let s = Date::ymd(2008, 2, 28).unwrap();
        let e = Date::ymd(2008, 3, 31).unwrap();
        let f = day_count::fraction(s, e, DayCount::ThirtyE360);
        assert!((f - 32.0_f64 / 360.0).abs() < TOL, "{f}");
    }

    /// QuantLib daycounters.cpp:583 — 2008-02-29 → 2009-02-28 = 359/360.
    /// Neither endpoint is 31; both stay. Num = 360 + 0 + (28-29) = 359.
    #[test]
    fn quantlib_eurobond_2008_02_29_to_2009_02_28() {
        let s = Date::ymd(2008, 2, 29).unwrap();
        let e = Date::ymd(2009, 2, 28).unwrap();
        let f = day_count::fraction(s, e, DayCount::ThirtyE360);
        assert!((f - 359.0_f64 / 360.0).abs() < TOL, "{f}");
    }

    // ─── 30E/360 ISDA (QuantLib Thirty360::ISDA) ─────────────────────────
    //
    // QuantLib's ISDA variant takes a `terminationDate` constructor
    // parameter; an interval whose `end == terminationDate` and whose
    // `end.month == February` triggers the §4.16(h) suppression. The
    // cases below pin specific (start, end, end_is_maturity) triples and
    // cite the data block + line in daycounters.cpp where the vector
    // appears.

    /// QuantLib daycounters.cpp:622 — 2007-08-31 → 2008-02-29 = 180/360,
    /// end_is_maturity = false (data2's terminationDate is 2012-02-29,
    /// so this end is NOT the maturity date). D1 = 31 = last day → 30;
    /// D2 = 29 = last day of Feb 2008 → 30 (not suppressed). Num = 180.
    #[test]
    fn quantlib_eurobond_isda_2007_08_31_to_2008_02_29_not_maturity() {
        let s = Date::ymd(2007, 8, 31).unwrap();
        let e = Date::ymd(2008, 2, 29).unwrap();
        let f = thirty_e_360_isda::fraction(s, e, false);
        assert!((f - 180.0_f64 / 360.0).abs() < TOL, "{f}");
    }

    /// QuantLib daycounters.cpp:630 — 2011-08-31 → 2012-02-29 = 179/360,
    /// end_is_maturity = true (data2's terminationDate is 2012-02-29).
    /// D1 = 31 → 30; D2 = 29 IS last day of Feb but suppression fires
    /// → D2 stays 29. Num = 360 + 30*(2-8) + (29-30) = 179.
    #[test]
    fn quantlib_eurobond_isda_2011_08_31_to_2012_02_29_at_maturity() {
        let s = Date::ymd(2011, 8, 31).unwrap();
        let e = Date::ymd(2012, 2, 29).unwrap();
        let f = thirty_e_360_isda::fraction(s, e, true);
        assert!((f - 179.0_f64 / 360.0).abs() < TOL, "{f}");
    }

    /// QuantLib daycounters.cpp:633 — 2006-01-31 → 2006-02-28 = 30/360,
    /// end_is_maturity = false (data3's terminationDate is 2008-02-29).
    /// D1 = 31 = last day → 30; D2 = 28 = last day of Feb 2006 → 30.
    /// Num = 30 + (30-30) = 30.
    #[test]
    fn quantlib_eurobond_isda_2006_01_31_to_2006_02_28() {
        let s = Date::ymd(2006, 1, 31).unwrap();
        let e = Date::ymd(2006, 2, 28).unwrap();
        let f = thirty_e_360_isda::fraction(s, e, false);
        assert!((f - 30.0_f64 / 360.0).abs() < TOL, "{f}");
    }

    /// QuantLib daycounters.cpp:639 — 2007-08-31 → 2008-02-28 = 178/360,
    /// end_is_maturity = false. D1 = 31 → 30; D2 = 28 is NOT last day
    /// of Feb 2008 (which is 29) → stays. Num = 360-180-2 = 178.
    #[test]
    fn quantlib_eurobond_isda_2007_08_31_to_2008_02_28_not_last_day() {
        let s = Date::ymd(2007, 8, 31).unwrap();
        let e = Date::ymd(2008, 2, 28).unwrap();
        let f = thirty_e_360_isda::fraction(s, e, false);
        assert!((f - 178.0_f64 / 360.0).abs() < TOL, "{f}");
    }

    /// QuantLib daycounters.cpp:644 — 2007-02-28 → 2008-02-29 = 359/360,
    /// end_is_maturity = true. D1 = 28 = last day Feb 2007 → 30; D2 = 29
    /// IS last day of Feb 2008 but suppressed → stays. Num = 360 + (29-30)
    /// = 359.
    #[test]
    fn quantlib_eurobond_isda_2007_02_28_to_2008_02_29_at_maturity() {
        let s = Date::ymd(2007, 2, 28).unwrap();
        let e = Date::ymd(2008, 2, 29).unwrap();
        let f = thirty_e_360_isda::fraction(s, e, true);
        assert!((f - 359.0_f64 / 360.0).abs() < TOL, "{f}");
    }

    /// QuantLib daycounters.cpp:645 — 2008-02-29 → 2009-02-28 = 360/360,
    /// end_is_maturity = false (data3's terminationDate is 2008-02-29,
    /// not 2009-02-28). D1 = 29 = last day Feb 2008 → 30; D2 = 28 = last
    /// day of Feb 2009 → 30. Num = 360 + 0 + 0 = 360.
    #[test]
    fn quantlib_eurobond_isda_2008_02_29_to_2009_02_28_not_maturity() {
        let s = Date::ymd(2008, 2, 29).unwrap();
        let e = Date::ymd(2009, 2, 28).unwrap();
        let f = thirty_e_360_isda::fraction(s, e, false);
        assert!((f - 360.0_f64 / 360.0).abs() < TOL, "{f}");
    }

    // ─── Hand-derived vectors for conventions without QuantLib coverage ─

    /// Act/360 — non-QuantLib vector. 1 Mar 2020 → 1 Mar 2024 spans 4
    /// years including the 2020 leap day, so 366 + 365 + 365 + 365 =
    /// 1461 days. f = 1461/360 = 4.058333333333333.
    #[test]
    fn hand_act_360_four_year_span_includes_leap() {
        let s = Date::ymd(2020, 3, 1).unwrap();
        let e = Date::ymd(2024, 3, 1).unwrap();
        let f = day_count::fraction(s, e, DayCount::Act360);
        assert!((f - 1461.0_f64 / 360.0).abs() < TOL, "{f}");
    }

    /// Act/360 — full non-leap-year-and-a-half span. 2025-01-01 →
    /// 2026-07-01 is 365 + 181 = 546 days; f = 546/360 = 1.516666...
    #[test]
    fn hand_act_360_eighteen_months_non_leap() {
        let s = Date::ymd(2025, 1, 1).unwrap();
        let e = Date::ymd(2026, 7, 1).unwrap();
        let f = day_count::fraction(s, e, DayCount::Act360);
        assert!((f - 546.0_f64 / 360.0).abs() < TOL, "{f}");
    }

    /// Act/360 — 28-day February in a non-leap year. 2025-02-01 →
    /// 2025-03-01 = 28 days; f = 28/360.
    #[test]
    fn hand_act_360_february_non_leap() {
        let s = Date::ymd(2025, 2, 1).unwrap();
        let e = Date::ymd(2025, 3, 1).unwrap();
        let f = day_count::fraction(s, e, DayCount::Act360);
        assert!((f - 28.0_f64 / 360.0).abs() < TOL, "{f}");
    }

    /// Act/365F — same four-year span as the Act/360 hand vector above.
    /// f = 1461/365 = 4.002739726027397.
    #[test]
    fn hand_act_365f_four_year_span_includes_leap() {
        let s = Date::ymd(2020, 3, 1).unwrap();
        let e = Date::ymd(2024, 3, 1).unwrap();
        let f = day_count::fraction(s, e, DayCount::Act365F);
        assert!((f - 1461.0_f64 / 365.0).abs() < TOL, "{f}");
    }

    /// Act/365F — quarter-of-a-year in a non-leap year (Q3 2025).
    /// 2025-07-01 → 2025-10-01 = 31 + 31 + 30 = 92 days; f = 92/365.
    #[test]
    fn hand_act_365f_third_quarter_non_leap() {
        let s = Date::ymd(2025, 7, 1).unwrap();
        let e = Date::ymd(2025, 10, 1).unwrap();
        let f = day_count::fraction(s, e, DayCount::Act365F);
        assert!((f - 92.0_f64 / 365.0).abs() < TOL, "{f}");
    }

    /// Act/365F — leap day on the start boundary. 2024-02-29 →
    /// 2025-02-28 = 365 days (29 Feb is on the start, last counted day
    /// is 2025-02-27 → 366 days no, let me redo: days_between is
    /// end-exclusive: 2024-02-29 → 2024-03-01 is 1 day, then full year
    /// to 2025-03-01 would be 1+365 = 366, so 2024-02-29 → 2025-02-28
    /// is 366 - 1 = 365 days). f = 365/365 = 1.0 exactly.
    #[test]
    fn hand_act_365f_leap_day_on_start_to_28_feb_next_year() {
        let s = Date::ymd(2024, 2, 29).unwrap();
        let e = Date::ymd(2025, 2, 28).unwrap();
        let f = day_count::fraction(s, e, DayCount::Act365F);
        assert!((f - 1.0).abs() < TOL, "{f}");
    }

    /// Act/365L — full non-leap year. 2025-01-01 → 2026-01-01: no
    /// 29-Feb inside → denominator 365; numerator 365; f = 1.0.
    #[test]
    fn hand_act_365l_full_non_leap_year() {
        let s = Date::ymd(2025, 1, 1).unwrap();
        let e = Date::ymd(2026, 1, 1).unwrap();
        let f = day_count::fraction(s, e, DayCount::Act365L);
        assert!((f - 1.0).abs() < TOL, "{f}");
    }

    /// Act/365L — Q3 2024 (after the leap day). 2024-07-01 →
    /// 2024-10-01 = 92 days; 2024-02-29 NOT in [start, end) →
    /// denominator 365. f = 92/365.
    #[test]
    fn hand_act_365l_q3_2024_after_leap() {
        let s = Date::ymd(2024, 7, 1).unwrap();
        let e = Date::ymd(2024, 10, 1).unwrap();
        let f = day_count::fraction(s, e, DayCount::Act365L);
        assert!((f - 92.0_f64 / 365.0).abs() < TOL, "{f}");
    }

    /// Act/365L — straddles 2024-02-29. 2023-12-01 → 2024-06-01 = 31 +
    /// 31 + 29 + 31 + 30 + 31 = 183 days. 2024-02-29 IS in [start, end)
    /// → denominator 366. f = 183/366 = 0.5.
    #[test]
    fn hand_act_365l_straddle_leap_day_2024() {
        let s = Date::ymd(2023, 12, 1).unwrap();
        let e = Date::ymd(2024, 6, 1).unwrap();
        let f = day_count::fraction(s, e, DayCount::Act365L);
        assert!((f - 183.0_f64 / 366.0).abs() < TOL, "{f}");
        // Exact half — the leap-aware denominator is the whole point.
        assert!((f - 0.5).abs() < TOL, "{f}");
    }

    /// NL/365 — full leap year. 2024-01-01 → 2025-01-01 = 366 days, of
    /// which 2024-02-29 is dropped → numerator 365; f = 1.0.
    #[test]
    fn hand_nl_365_full_leap_year_is_one() {
        let s = Date::ymd(2024, 1, 1).unwrap();
        let e = Date::ymd(2025, 1, 1).unwrap();
        let f = day_count::fraction(s, e, DayCount::Nl365);
        assert!((f - 1.0).abs() < TOL, "{f}");
    }

    /// NL/365 — eight-year span 2020-01-01 → 2028-01-01 covers two leap
    /// years (2020, 2024). Days = 2*366 + 6*365 = 2922; minus 2 leap
    /// days → numerator 2920; f = 2920/365 = 8.0 exactly.
    #[test]
    fn hand_nl_365_eight_year_span_two_leaps() {
        let s = Date::ymd(2020, 1, 1).unwrap();
        let e = Date::ymd(2028, 1, 1).unwrap();
        let f = day_count::fraction(s, e, DayCount::Nl365);
        assert!((f - 8.0).abs() < TOL, "{f}");
    }

    /// NL/365 — Q1 in a non-leap year. 2025-01-01 → 2025-04-01 = 90
    /// days, no leap day. f = 90/365.
    #[test]
    fn hand_nl_365_q1_non_leap() {
        let s = Date::ymd(2025, 1, 1).unwrap();
        let e = Date::ymd(2025, 4, 1).unwrap();
        let f = day_count::fraction(s, e, DayCount::Nl365);
        assert!((f - 90.0_f64 / 365.0).abs() < TOL, "{f}");
    }

    /// Act/Act ISDA — full non-leap year. 2025-01-01 → 2026-01-01 = 365
    /// days, all in 2025 (non-leap). f = 365/365 = 1.0.
    #[test]
    fn hand_act_act_isda_full_non_leap_year() {
        let s = Date::ymd(2025, 1, 1).unwrap();
        let e = Date::ymd(2026, 1, 1).unwrap();
        let f = day_count::fraction(s, e, DayCount::ActActIsda);
        assert!((f - 1.0).abs() < TOL, "{f}");
    }

    /// Act/Act ISDA — full leap year. 2024-01-01 → 2025-01-01 = 366
    /// days, all in 2024 (leap). f = 366/366 = 1.0.
    #[test]
    fn hand_act_act_isda_full_leap_year() {
        let s = Date::ymd(2024, 1, 1).unwrap();
        let e = Date::ymd(2025, 1, 1).unwrap();
        let f = day_count::fraction(s, e, DayCount::ActActIsda);
        assert!((f - 1.0).abs() < TOL, "{f}");
    }

    // ─── Bus/252 — hand-derived against a weekends-only calendar ─────────

    /// Bus/252 hand vector — first business week of 2026 under a
    /// weekends-only predicate. 2026-01-05 (Mon) → 2026-01-12 (Mon):
    /// Mon, Tue, Wed, Thu, Fri (in the half-open interval) = 5 business
    /// days. f = 5/252.
    #[cfg(feature = "calendars")]
    #[test]
    fn hand_bus_252_one_business_week_weekends_only() {
        use regit_daycount::day_count::bus_252;
        let s = Date::ymd(2026, 1, 5).unwrap();
        let e = Date::ymd(2026, 1, 12).unwrap();
        let f = bus_252::fraction(s, e, |d| !calendar::is_weekend(d));
        assert!((f - 5.0_f64 / 252.0).abs() < TOL, "{f}");
    }

    /// Bus/252 hand vector — three calendar weeks. 2026-01-05 (Mon) →
    /// 2026-01-26 (Mon): three full Mon–Fri weeks = 15 business days.
    /// f = 15/252.
    #[cfg(feature = "calendars")]
    #[test]
    fn hand_bus_252_three_business_weeks_weekends_only() {
        use regit_daycount::day_count::bus_252;
        let s = Date::ymd(2026, 1, 5).unwrap();
        let e = Date::ymd(2026, 1, 26).unwrap();
        let f = bus_252::fraction(s, e, |d| !calendar::is_weekend(d));
        assert!((f - 15.0_f64 / 252.0).abs() < TOL, "{f}");
    }

    /// Bus/252 hand vector — TARGET2 around Christmas 2024. 2024-12-23
    /// (Mon) → 2024-12-30 (Mon): in [start, end), Mon 23, Tue 24, Wed
    /// 25 (TARGET2 Christmas), Thu 26 (TARGET2 Boxing Day), Fri 27, Sat
    /// 28, Sun 29 — business days are 23 (Mon), 24 (Tue), 27 (Fri) =
    /// 3 business days. f = 3/252.
    #[cfg(feature = "calendars")]
    #[test]
    fn hand_bus_252_target2_christmas_week_2024() {
        let s = Date::ymd(2024, 12, 23).unwrap();
        let e = Date::ymd(2024, 12, 30).unwrap();
        let f = calendar::bus_252_for_calendar(s, e, Calendar::Target2);
        assert!((f - 3.0_f64 / 252.0).abs() < TOL, "{f}");
    }
}

// ─── Cross-module integration ────────────────────────────────────────────────

#[cfg(feature = "calendars")]
mod cross_module {
    use super::*;

    /// Date ⇄ roll ⇄ calendar ⇄ day_count. Take a Saturday, adjust
    /// Following under TARGET2, then compute Act/360 from a fixed start
    /// to the adjusted-end. The hand-computation: 2026-05-23 (Sat) →
    /// Following → 2026-05-25 (Mon). From 2026-05-01 the days_between is
    /// 24; Act/360 fraction = 24 / 360.
    #[test]
    fn adjust_then_fraction_under_target2() {
        let saturday = Date::ymd(2026, 5, 23).unwrap();
        assert_eq!(saturday.day_of_week(), Weekday::Sat);
        let adjusted = calendar::adjust(saturday, Roll::Following, Calendar::Target2);
        assert_eq!(adjusted, Date::ymd(2026, 5, 25).unwrap());

        let start = Date::ymd(2026, 5, 1).unwrap();
        let f = day_count::fraction(start, adjusted, DayCount::Act360);
        assert!((f - 24.0_f64 / 360.0).abs() < TOL, "Act/360 1→25 May: {f}");
    }

    /// Composite holiday over [TARGET2, UnitedStates] in the week of
    /// 4 July 2024 — the only US-only holiday in this window.
    ///
    /// `[2024-07-01, 2024-07-08)`: Mon, Tue, Wed, Thu (4 July =
    /// composite holiday because US closed), Fri, Sat, Sun. The
    /// composite drops 2024-07-04 → 4 business days remain (Jul 1, 2,
    /// 3, 5). `Composite::is_business_day` provides the predicate;
    /// `business_days_between` for `Calendar` only takes a single
    /// named calendar, so the walk is done by hand here.
    #[test]
    fn composite_then_business_days_between() {
        static REGIONS: &[Calendar] = &[Calendar::Target2, Calendar::UnitedStates];
        let comp = Composite::new(REGIONS);

        let start = Date::ymd(2024, 7, 1).unwrap();
        let end = Date::ymd(2024, 7, 8).unwrap();

        // Walk one day at a time using the composite predicate.
        let mut count = 0u32;
        let mut d = start;
        while d < end {
            if comp.is_business_day(d) {
                count += 1;
            }
            d = d.add_days(1);
        }
        assert_eq!(count, 4, "composite business days in week of 4 Jul 2024");

        // Spot-check the predicate at each weekday in the window.
        assert!(comp.is_business_day(Date::ymd(2024, 7, 1).unwrap())); // Mon
        assert!(comp.is_business_day(Date::ymd(2024, 7, 2).unwrap())); // Tue
        assert!(comp.is_business_day(Date::ymd(2024, 7, 3).unwrap())); // Wed
        assert!(!comp.is_business_day(Date::ymd(2024, 7, 4).unwrap())); // US Independence
        assert!(comp.is_business_day(Date::ymd(2024, 7, 5).unwrap())); // Fri
        assert!(!comp.is_business_day(Date::ymd(2024, 7, 6).unwrap())); // Sat
        assert!(!comp.is_business_day(Date::ymd(2024, 7, 7).unwrap())); // Sun
    }

    /// The `bus_252_for_calendar` convenience wraps the same
    /// `bus_252::fraction` call with the same predicate. The two must
    /// agree exactly (not within tolerance) on every input.
    #[test]
    fn bus_252_for_calendar_matches_direct_bus_252_call() {
        let s = Date::ymd(2026, 5, 1).unwrap();
        let e = Date::ymd(2026, 5, 15).unwrap();
        let via_helper = calendar::bus_252_for_calendar(s, e, Calendar::Target2);
        let via_direct =
            bus_252::fraction(s, e, |d| calendar::is_business_day(d, Calendar::Target2));
        // Bit-exact equality: both paths execute identical arithmetic.
        assert_eq!(
            via_helper.to_bits(),
            via_direct.to_bits(),
            "{via_helper} vs {via_direct}",
        );
    }

    /// `roll::apply` reaches the same answer as `calendar::adjust` when
    /// the predicate is `|d| calendar::is_business_day(d, cal)` — by
    /// construction the latter delegates to the former, but the
    /// integration test pins the contract.
    #[test]
    fn roll_apply_matches_calendar_adjust() {
        let saturday = Date::ymd(2026, 5, 23).unwrap();
        let via_roll = roll::apply(saturday, Roll::Following, |d| {
            calendar::is_business_day(d, Calendar::Target2)
        });
        let via_cal = calendar::adjust(saturday, Roll::Following, Calendar::Target2);
        assert_eq!(via_roll, via_cal);
    }
}
