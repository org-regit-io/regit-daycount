// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! Criterion benchmarks for regit-daycount.
//!
//! Every operation is fixed-size, allocation-free integer / f64 arithmetic
//! over a `Copy` Date triple, so all of it is comfortably sub-microsecond.
//!
//! Performance targets (indicative, native release on commodity hardware):
//!
//! | Operation                                | Target   |
//! |------------------------------------------|----------|
//! | Date::ymd / day_of_week / add_days       | < 20 ns  |
//! | Act/360 / Act/365F / 30/360 fraction     | < 30 ns  |
//! | ActAct ISDA fraction (year-split)        | < 60 ns  |
//! | ActAct ICMA / 30E/360 ISDA fraction      | < 50 ns  |
//! | day_count::fraction (dispatch + arm)     | < 60 ns  |
//! | calendar::is_holiday (binary search)     | < 200 ns |
//! | calendar::adjust (Following, 1 holiday)  | < 500 ns |
//! | calendar::business_days_between (14 d)   | < 5 µs   |

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use regit_daycount::{Date, DayCount, Weekday, day_count, roll};

#[cfg(feature = "calendars")]
use regit_daycount::{Calendar, Roll, calendar};

// ─── Date primitives ─────────────────────────────────────────────────────────

fn bench_date(c: &mut Criterion) {
    let mut group = c.benchmark_group("date");

    // Construction with full validation.
    group.bench_function("ymd", |b| {
        b.iter(|| Date::ymd(black_box(2026), black_box(5), black_box(23)));
    });

    // Weekday — Hinnant civil-day path.
    let d = Date::ymd(2026, 5, 23).expect("bench: 2026-05-23 valid");
    group.bench_function("day_of_week", |b| {
        b.iter(|| black_box(d).day_of_week());
    });

    // One-day forward step — round-trip through civil days.
    group.bench_function("add_days", |b| {
        b.iter(|| black_box(d).add_days(black_box(1)));
    });

    // Closed-form Anonymous Gregorian / Meeus computus.
    group.bench_function("easter_sunday", |b| {
        b.iter(|| Date::easter_sunday(black_box(2026)));
    });

    // 3rd Friday of June 2026 — the standard nth-weekday lookup.
    group.bench_function("nth_weekday_of_month", |b| {
        b.iter(|| {
            Date::nth_weekday_of_month(
                black_box(2026),
                black_box(6),
                black_box(3),
                black_box(Weekday::Fri),
            )
        });
    });

    group.finish();
}

// ─── Day-count fractions (via dispatcher) ────────────────────────────────────

fn bench_day_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("day_count");

    // Apple-style semi-annual bond period: 2026-01-15 → 2026-07-15.
    let start = Date::ymd(2026, 1, 15).expect("bench: 2026-01-15 valid");
    let end = Date::ymd(2026, 7, 15).expect("bench: 2026-07-15 valid");

    group.bench_function("act_360", |b| {
        b.iter(|| {
            day_count::fraction(
                black_box(start),
                black_box(end),
                black_box(DayCount::Act360),
            )
        });
    });
    group.bench_function("act_365f", |b| {
        b.iter(|| {
            day_count::fraction(
                black_box(start),
                black_box(end),
                black_box(DayCount::Act365F),
            )
        });
    });
    group.bench_function("act_act_isda", |b| {
        b.iter(|| {
            day_count::fraction(
                black_box(start),
                black_box(end),
                black_box(DayCount::ActActIsda),
            )
        });
    });
    group.bench_function("thirty_360_bond_basis", |b| {
        b.iter(|| {
            day_count::fraction(
                black_box(start),
                black_box(end),
                black_box(DayCount::Thirty360BondBasis),
            )
        });
    });
    group.bench_function("thirty_e_360", |b| {
        b.iter(|| {
            day_count::fraction(
                black_box(start),
                black_box(end),
                black_box(DayCount::ThirtyE360),
            )
        });
    });
    group.bench_function("thirty_e_360_isda", |b| {
        b.iter(|| {
            day_count::fraction(
                black_box(start),
                black_box(end),
                black_box(DayCount::ThirtyE360Isda),
            )
        });
    });
    group.bench_function("act_365l", |b| {
        b.iter(|| {
            day_count::fraction(
                black_box(start),
                black_box(end),
                black_box(DayCount::Act365L),
            )
        });
    });
    group.bench_function("nl_365", |b| {
        b.iter(|| {
            day_count::fraction(black_box(start), black_box(end), black_box(DayCount::Nl365))
        });
    });
    group.bench_function("one_one", |b| {
        b.iter(|| {
            day_count::fraction(
                black_box(start),
                black_box(end),
                black_box(DayCount::OneOne),
            )
        });
    });

    // ActAct ICMA — semi-annual reference period coincident with the
    // accrual period (regular coupon).
    group.bench_function("act_act_icma", |b| {
        b.iter(|| {
            day_count::year_fraction_with_freq(
                black_box(start),
                black_box(end),
                black_box(DayCount::ActActIcma),
                black_box(2),
                black_box(start),
                black_box(end),
            )
        });
    });

    group.finish();
}

// ─── Calendar holiday lookup ─────────────────────────────────────────────────

#[cfg(feature = "calendars")]
fn bench_calendar(c: &mut Criterion) {
    let mut group = c.benchmark_group("calendar");

    // TARGET2 — fixed-rule generator (Easter-derived; six dates a year).
    let target2_hit = Date::ymd(2026, 1, 1).expect("bench: 2026-01-01 valid");
    group.bench_function("is_holiday_target2", |b| {
        b.iter(|| calendar::is_holiday(black_box(target2_hit), black_box(Calendar::Target2)));
    });

    // Luxembourg — fixed-rule generator (Easter-derived; eleven dates a
    // year). Whit Monday distinguishes LU from TARGET2 — pick that date.
    let lu_hit = Date::ymd(2026, 5, 25).expect("bench: 2026-05-25 valid");
    group.bench_function("is_holiday_luxembourg", |b| {
        b.iter(|| calendar::is_holiday(black_box(lu_hit), black_box(Calendar::Luxembourg)));
    });

    // United States — snapshot table; a hit (Independence Day observed)
    // and a miss (an ordinary Friday) to bound the binary search both ways.
    let us_hit = Date::ymd(2026, 7, 3).expect("bench: 2026-07-03 valid");
    group.bench_function("is_holiday_united_states_hit", |b| {
        b.iter(|| calendar::is_holiday(black_box(us_hit), black_box(Calendar::UnitedStates)));
    });
    let us_miss = Date::ymd(2026, 5, 22).expect("bench: 2026-05-22 valid");
    group.bench_function("is_holiday_united_states_miss", |b| {
        b.iter(|| calendar::is_holiday(black_box(us_miss), black_box(Calendar::UnitedStates)));
    });

    // Japan — the largest table at 433 rows; a hit (Coming of Age Day).
    let jp_hit = Date::ymd(2026, 1, 12).expect("bench: 2026-01-12 valid");
    group.bench_function("is_holiday_japan_hit", |b| {
        b.iter(|| calendar::is_holiday(black_box(jp_hit), black_box(Calendar::Japan)));
    });

    // Hong Kong — Lunar New Year Day 1.
    let hk_hit = Date::ymd(2026, 2, 17).expect("bench: 2026-02-17 valid");
    group.bench_function("is_holiday_hong_kong", |b| {
        b.iter(|| calendar::is_holiday(black_box(hk_hit), black_box(Calendar::HongKong)));
    });

    group.finish();
}

// ─── Date-roll application ───────────────────────────────────────────────────

fn bench_roll(c: &mut Criterion) {
    let mut group = c.benchmark_group("roll");

    // Weekend-only business-day predicate — keeps the bench independent of
    // any calendar so it runs under `--no-default-features` too.
    let is_biz = |d: Date| {
        let wd = d.day_of_week();
        wd != Weekday::Sat && wd != Weekday::Sun
    };

    // Saturday 2026-05-23 — exercises all three rolls below.
    let saturday = Date::ymd(2026, 5, 23).expect("bench: 2026-05-23 valid");

    group.bench_function("apply_following", |b| {
        b.iter(|| {
            roll::apply(
                black_box(saturday),
                black_box(roll::Roll::Following),
                is_biz,
            )
        });
    });
    group.bench_function("apply_modified_following", |b| {
        b.iter(|| {
            roll::apply(
                black_box(saturday),
                black_box(roll::Roll::ModifiedFollowing),
                is_biz,
            )
        });
    });
    group.bench_function("apply_nearest", |b| {
        b.iter(|| roll::apply(black_box(saturday), black_box(roll::Roll::Nearest), is_biz));
    });

    group.finish();

    // The TARGET2-driven adjust lives in its own (feature-gated) group below
    // so the rest of `bench_roll` still compiles under `--no-default-features`.
    #[cfg(feature = "calendars")]
    {
        let mut group = c.benchmark_group("roll_calendar");
        // Fri 2026-05-01 is TARGET2 Labour Day — Following lands on Mon
        // 2026-05-04 after one one-day forward step.
        let labour = Date::ymd(2026, 5, 1).expect("bench: 2026-05-01 valid");
        group.bench_function("adjust_following_target2", |b| {
            b.iter(|| {
                calendar::adjust(
                    black_box(labour),
                    black_box(Roll::Following),
                    black_box(Calendar::Target2),
                )
            });
        });
        group.finish();
    }
}

// ─── Bus/252 walk ────────────────────────────────────────────────────────────

#[cfg(feature = "calendars")]
fn bench_bus_252(c: &mut Criterion) {
    let mut group = c.benchmark_group("bus_252");

    // [2026-05-01, 2026-05-15) under TARGET2 — 14 calendar days, 9 business
    // days (May 1 is Labour Day; the two weekends drop the remaining four).
    let start = Date::ymd(2026, 5, 1).expect("bench: 2026-05-01 valid");
    let end = Date::ymd(2026, 5, 15).expect("bench: 2026-05-15 valid");

    group.bench_function("target2_fortnight", |b| {
        b.iter(|| {
            calendar::bus_252_for_calendar(
                black_box(start),
                black_box(end),
                black_box(Calendar::Target2),
            )
        });
    });

    group.finish();
}

// ─── Harness ─────────────────────────────────────────────────────────────────

#[cfg(feature = "calendars")]
criterion_group!(
    benches,
    bench_date,
    bench_day_count,
    bench_calendar,
    bench_roll,
    bench_bus_252,
);

#[cfg(not(feature = "calendars"))]
criterion_group!(benches, bench_date, bench_day_count, bench_roll);

criterion_main!(benches);
