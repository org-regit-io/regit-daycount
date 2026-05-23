// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! Quickstart example for `regit-daycount`.
//!
//! A guided tour of the crate, in eight labelled sections, for anyone
//! evaluating the API: every `Date` primitive, the eleven day-count
//! fractions side-by-side over the same interval, the seven date-roll
//! conventions applied to a single Saturday, the seven named holiday
//! calendars on a known closing date each, `adjust` and
//! `business_days_between` under TARGET2, a `JointBusiness` EUR/USD
//! settlement filter, and a worked accrued-interest computation on a
//! hypothetical EUR bond that ties the pieces together. Run with
//! `cargo run --example quickstart`.

use regit_daycount::calendar::{
    self, JointBusiness, adjust, bus_252_for_calendar, business_days_between, is_holiday,
};
use regit_daycount::day_count;
use regit_daycount::{Calendar, Date, DayCount, Roll, Weekday};

// Section 7 builds a `JointBusiness` over a `'static` slice — `JointBusiness`
// borrows its kinds, so the slice must outlive the value. A module-level
// `static` is the simplest such storage.
static EUR_USD: &[Calendar] = &[Calendar::Target2, Calendar::UnitedStates];

// A guided tour is one long, deliberately linear `main` — eight labelled
// sections that read top to bottom; splitting it into helpers would hide
// the narrative.
#[allow(clippy::too_many_lines)]
fn main() {
    // ── 1. Date primitives — Gregorian arithmetic over a 32-bit triple ──
    let today = Date::ymd(2026, 5, 23).expect("1. Date primitives");
    println!(
        "Date  {:04}-{:02}-{:02}  ({:?})",
        today.year(),
        today.month(),
        today.day(),
        today.day_of_week(),
    );
    println!("  leap year 2024 = {}", Date::is_leap_year(2024));
    println!("  leap year 2025 = {}", Date::is_leap_year(2025));
    let plus_seven = today.add_days(7);
    println!(
        "  +7 days        = {:04}-{:02}-{:02}",
        plus_seven.year(),
        plus_seven.month(),
        plus_seven.day(),
    );
    let jan31 = Date::ymd(2026, 1, 31).expect("1. Date primitives");
    let feb = jan31.add_months_eom_aware(1);
    println!(
        "  Jan 31 + 1 month (EOM-aware) = {:04}-{:02}-{:02}",
        feb.year(),
        feb.month(),
        feb.day(),
    );
    let third_friday_june = Date::nth_weekday_of_month(2026, 6, 3, Weekday::Fri)
        .expect("1. Date primitives — 3rd Friday of June 2026");
    println!(
        "  3rd Friday of June 2026     = {:04}-{:02}-{:02}",
        third_friday_june.year(),
        third_friday_june.month(),
        third_friday_june.day(),
    );
    let easter = Date::easter_sunday(2026);
    println!(
        "  Easter Sunday 2026          = {:04}-{:02}-{:02}",
        easter.year(),
        easter.month(),
        easter.day(),
    );

    // ── 2. Day-count fractions — eleven conventions, same interval ──────
    // 2026-01-01 → 2026-04-01 is exactly 90 calendar days (31 + 28 + 31).
    // Printing every basis side-by-side makes the conventions visibly
    // disagree by amounts that move a real cashflow.
    let s = Date::ymd(2026, 1, 1).expect("2. Day-count fractions — start");
    let e = Date::ymd(2026, 4, 1).expect("2. Day-count fractions — end");
    println!("\nYear fractions for [2026-01-01, 2026-04-01) (90 calendar days)");
    for basis in [
        DayCount::Act360,
        DayCount::Act365F,
        DayCount::ActActIsda,
        DayCount::Thirty360BondBasis,
        DayCount::ThirtyE360,
        DayCount::ThirtyE360Isda,
        DayCount::Act365L,
        DayCount::Nl365,
        DayCount::OneOne,
    ] {
        println!(
            "  {:<20?} -> {:.12}",
            basis,
            day_count::fraction(s, e, basis)
        );
    }
    // ActActIcma needs a reference period — supplied via the freq-aware
    // dispatcher. A regular quarterly period over the same interval gives
    // exactly 1 / freq = 0.25.
    let icma = day_count::year_fraction_with_freq(s, e, DayCount::ActActIcma, 4, s, e);
    println!(
        "  {:<20?} -> {icma:.12}  (freq=4, regular period)",
        DayCount::ActActIcma,
    );
    // Bus/252 needs a business-day predicate — the calendar dispatcher
    // closes over one for us. Result = TARGET2 business days in
    // [Jan 1, Apr 1) / 252.
    let bus = bus_252_for_calendar(s, e, Calendar::Target2);
    println!("  {:<20?} -> {bus:.12}  (TARGET2)", DayCount::Bus252);

    // ── 3. Date-roll conventions — every roll on the same Saturday ──────
    // A weekend-only predicate (no holidays) makes the seven rolls easy to
    // compare: Sat 2026-05-23 rolls forward to Mon 05-25, backward to Fri
    // 05-22, etc.
    let weekend_only = |d: Date| !matches!(d.day_of_week(), Weekday::Sat | Weekday::Sun);
    let sat = Date::ymd(2026, 5, 23).expect("3. Date-roll conventions");
    println!("\nRolls applied to Sat 2026-05-23 (weekends-only predicate)");
    for conv in [
        Roll::Unadjusted,
        Roll::Following,
        Roll::ModifiedFollowing,
        Roll::Preceding,
        Roll::ModifiedPreceding,
        Roll::Nearest,
        Roll::EndOfMonth,
    ] {
        let r = regit_daycount::roll::apply(sat, conv, weekend_only);
        println!(
            "  {:<18?} -> {:04}-{:02}-{:02}",
            conv,
            r.year(),
            r.month(),
            r.day(),
        );
    }

    // ── 4. Holiday calendars — one known closing date per named calendar ─
    // Each calendar's first dispatcher (`is_holiday`) is checked against a
    // single recognisable date from its published list — the same smoke
    // test the unit suite uses, surfaced here as a sanity print.
    println!("\nis_holiday across the eight named calendars (2024)");
    let cases: [(Calendar, Date); 8] = [
        (
            Calendar::Target2,
            Date::ymd(2024, 12, 25).expect("4. TARGET2 Christmas"),
        ),
        (
            Calendar::Luxembourg,
            Date::ymd(2024, 6, 23).expect("4. Luxembourg National Day"),
        ),
        (
            Calendar::UnitedStates,
            Date::ymd(2024, 7, 4).expect("4. US Independence Day"),
        ),
        (
            Calendar::UnitedKingdom,
            Date::ymd(2024, 12, 26).expect("4. UK Boxing Day"),
        ),
        (
            Calendar::Japan,
            Date::ymd(2024, 1, 1).expect("4. Japan New Year"),
        ),
        (
            Calendar::Switzerland,
            Date::ymd(2024, 8, 1).expect("4. Swiss National Day"),
        ),
        (
            Calendar::HongKong,
            Date::ymd(2024, 12, 25).expect("4. HK Christmas"),
        ),
        (
            Calendar::Singapore,
            Date::ymd(2024, 8, 9).expect("4. SG National Day"),
        ),
    ];
    for (cal, d) in cases {
        println!(
            "  {:<15?} {:04}-{:02}-{:02} -> holiday={}",
            cal,
            d.year(),
            d.month(),
            d.day(),
            is_holiday(d, cal),
        );
    }

    // ── 5. adjust — rolling a holiday off the calendar ──────────────────
    // 2026-12-25 (Friday) is Christmas Day on TARGET2; 26 Dec (Sat) is also
    // a TARGET2 holiday, 27 Dec is Sunday, so Following lands on Mon
    // 2026-12-28.
    let xmas = Date::ymd(2026, 12, 25).expect("5. adjust — Christmas 2026");
    let next_biz = adjust(xmas, Roll::Following, Calendar::Target2);
    println!(
        "\nadjust(2026-12-25, Following, TARGET2) -> {:04}-{:02}-{:02}  ({:?})",
        next_biz.year(),
        next_biz.month(),
        next_biz.day(),
        next_biz.day_of_week(),
    );

    // ── 6. business_days_between — counting an open interval ────────────
    // [2026-05-01, 2026-05-15) under TARGET2: May 1 is Labour Day, so the
    // ten weekdays in the window minus that one give 9 business days.
    let may_start = Date::ymd(2026, 5, 1).expect("6. business_days_between — start");
    let may_end = Date::ymd(2026, 5, 15).expect("6. business_days_between — end");
    let bd = business_days_between(may_start, may_end, Calendar::Target2);
    println!(
        "\nbusiness_days_between([2026-05-01, 2026-05-15), TARGET2) = {bd}  (May 1 is Labour Day)"
    );

    // ── 7. JointBusiness — settling only when every leg is open ─────────
    // A cross-currency EUR/USD swap settles iff TARGET2 and the US calendar
    // are both open. July 4 (US closed) and May 1 (TARGET2 closed) both
    // fail; March 15 (both open) passes. `EUR_USD` is the module-level
    // `static` slice from the top of the file.
    let eurusd = JointBusiness::new(EUR_USD);
    let jul4 = Date::ymd(2024, 7, 4).expect("7. JointBusiness — US Independence");
    let may1 = Date::ymd(2024, 5, 1).expect("7. JointBusiness — TARGET2 Labour Day");
    let mar15 = Date::ymd(2024, 3, 15).expect("7. JointBusiness — clean Friday");
    println!("\nJointBusiness EUR/USD (TARGET2 + UnitedStates)");
    println!(
        "  2024-07-04 -> joint biz day = {}  (US closed)",
        eurusd.is_business_day(jul4),
    );
    println!(
        "  2024-05-01 -> joint biz day = {}  (TARGET2 closed)",
        eurusd.is_business_day(may1),
    );
    println!(
        "  2024-03-15 -> joint biz day = {}  (both open)",
        eurusd.is_business_day(mar15),
    );
    // Weekends are filtered by the underlying calendars' business-day rule;
    // the joint inherits that, as a sanity check:
    let sat_check = calendar::is_weekend(today);
    println!("  weekend filter is shared: 2026-05-23 is_weekend = {sat_check}");

    // ── 8. Worked example — accrued interest on a hypothetical EUR bond ─
    // Notional 1 000 000 EUR, fixed coupon 5.00 % p.a., Act/360, period
    // 2026-01-15 → 2026-04-15. This is what every primitive above exists
    // for: turn a date interval into a cashflow with no silent rounding.
    let notional = 1_000_000.0_f64;
    let coupon = 0.05_f64;
    let period_start = Date::ymd(2026, 1, 15).expect("8. bond — period start");
    let period_end = Date::ymd(2026, 4, 15).expect("8. bond — period end");
    let yf = day_count::fraction(period_start, period_end, DayCount::Act360);
    let accrued = notional * coupon * yf;
    println!("\nAccrued interest — EUR bond, 5.00% Act/360, 2026-01-15 to 2026-04-15");
    println!(
        "  days  = {} (start-inclusive, end-exclusive)",
        period_start.days_between(period_end),
    );
    println!("  yf    = {yf:.12}");
    println!("  accr  = {accrued:.2} EUR  (notional * coupon * yf)");
}
