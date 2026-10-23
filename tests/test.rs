// Copyright 2026 Regit.io — SPDX-License-Identifier: Apache-2.0
//! Independent Rust executable: input scenarios in, crate results out.

use regit_daycount::Date;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;
use std::io::{self, BufRead, Write};
use std::str::FromStr;

struct Args<'a>(BTreeMap<&'a str, &'a str>);
impl<'a> Args<'a> {
    fn new(value: &'a str) -> Self {
        Self(
            value
                .split(';')
                .filter(|part| !part.is_empty())
                .map(|part| part.split_once('=').expect("key=value input"))
                .collect(),
        )
    }
    fn get(&self, key: &str) -> &'a str {
        self.0
            .get(key)
            .unwrap_or_else(|| panic!("missing argument {key}"))
    }
    fn has(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }
    fn parse<T: FromStr>(&self, key: &str) -> T
    where
        T::Err: Debug,
    {
        self.get(key).parse().unwrap()
    }
    fn date(&self, key: &str) -> Date {
        let parts: Vec<_> = self.get(key).split('-').collect();
        Date::ymd(
            parts[0].parse().unwrap(),
            parts[1].parse().unwrap(),
            parts[2].parse().unwrap(),
        )
        .unwrap()
    }
}
fn observe(api: &str, args: &Args<'_>) -> Result<String, String> {
    if api == "@reference" {
        let mut fields: Vec<_> = args.get("row").split('|').collect();
        fields.extend(["-", "-"]);
        return numeric::evaluate(&fields).map(|value| format!("Ok:f64:{value:.17}"));
    }
    if let Some(result) = date_roll::evaluate(api, args) {
        return result;
    }
    if let Some(result) = day_counts::evaluate(api, args) {
        return result;
    }
    #[cfg(feature = "calendars")]
    if let Some(result) = calendars::evaluate(api, args) {
        return result;
    }
    Err(format!("unknown observation API: {api}"))
}
fn execute() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::var("REGIT_TEST_OUTPUT")?;
    let mut output = io::BufWriter::new(std::fs::File::create(path)?);
    let mut ids = BTreeSet::new();
    for line in io::stdin().lock().lines() {
        let line = line?;
        let fields: Vec<_> = line.splitn(3, '\t').collect();
        if fields.len() != 3 || !ids.insert(fields[0].to_owned()) {
            return Err(format!("invalid or duplicate input: {line}").into());
        }
        let actual = observe(fields[1], &Args::new(fields[2]))?;
        writeln!(output, "{}\t{actual}", fields[0])?;
    }
    if ids.is_empty() {
        return Err("no observation inputs".into());
    }
    output.flush()?;
    Ok(())
}

mod numeric {

    // Copyright 2026 Regit.io — SPDX-License-Identifier: Apache-2.0

    use regit_daycount::day_count::{self, CouponPeriod, DayCountContext};
    use regit_daycount::{Date, DayCount};

    mod operations {

        // Copyright 2026 Regit.io — SPDX-License-Identifier: Apache-2.0

        use regit_daycount::{Date, Roll, Weekday, roll};

        pub fn evaluate(fields: &[&str]) -> Option<Result<f64, String>> {
            match fields[1] {
                "date_op" => Some(date_operation(fields)),
                "roll_op" => Some(roll_operation(fields)),
                "calendar_op" => Some(calendar_operation(fields)),
                _ => None,
            }
        }

        fn parts(value: &str) -> (i32, u8, u8) {
            let mut parts = value.split('-');
            (
                parts.next().unwrap().parse().unwrap(),
                parts.next().unwrap().parse().unwrap(),
                parts.next().unwrap().parse().unwrap(),
            )
        }

        fn date(value: &str) -> Date {
            super::date(value)
        }

        fn encoded(date: Date) -> f64 {
            f64::from(date.year() * 10000 + i32::from(date.month()) * 100 + i32::from(date.day()))
        }

        fn weekday(index: u8) -> Weekday {
            [
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
                Weekday::Sat,
                Weekday::Sun,
            ][usize::from(index)]
        }

        fn date_operation(fields: &[&str]) -> Result<f64, String> {
            let (year, month, day) = parts(fields[3]);
            if fields[2] == "ymd" {
                return Ok(Date::ymd(year, month, day).map_or(0.0, encoded));
            }
            let start = date(fields[3]);
            Ok(match fields[2] {
                "year" => f64::from(start.year()),
                "month" => f64::from(start.month()),
                "day" => f64::from(start.day()),
                "is_leap_year" => f64::from(u8::from(Date::is_leap_year(year))),
                "days_in_month" => f64::from(Date::days_in_month(year, month)),
                "day_of_week" => f64::from(match start.day_of_week() {
                    Weekday::Mon => 1,
                    Weekday::Tue => 2,
                    Weekday::Wed => 3,
                    Weekday::Thu => 4,
                    Weekday::Fri => 5,
                    Weekday::Sat => 6,
                    Weekday::Sun => 7,
                }),
                "try_add_days" => start
                    .try_add_days(fields[7].parse().unwrap())
                    .map_or(0.0, encoded),
                "try_add_months_eom_aware" => start
                    .try_add_months_eom_aware(fields[7].parse().unwrap())
                    .map_or(0.0, encoded),
                "days_between" => f64::from(start.days_between(date(fields[4]))),
                "try_easter_sunday" => Date::try_easter_sunday(year).map_or(0.0, encoded),
                "nth_weekday_of_month" => Date::nth_weekday_of_month(
                    year,
                    month,
                    fields[7].parse().unwrap(),
                    weekday(fields[5].parse().unwrap()),
                )
                .map_or(0.0, encoded),
                other => return Err(format!("Unknown Date operation {other}")),
            })
        }

        fn convention(value: &str) -> Result<Roll, String> {
            Ok(match value {
                "Unadjusted" => Roll::Unadjusted,
                "Following" => Roll::Following,
                "ModifiedFollowing" => Roll::ModifiedFollowing,
                "Preceding" => Roll::Preceding,
                "ModifiedPreceding" => Roll::ModifiedPreceding,
                "Nearest" => Roll::Nearest,
                "EndOfMonth" => Roll::EndOfMonth,
                other => return Err(format!("Unknown Roll {other}")),
            })
        }

        fn roll_operation(fields: &[&str]) -> Result<f64, String> {
            let (name, operation) = fields[2].split_once(':').unwrap();
            let start = date(fields[3]);
            let conv = convention(operation)?;
            if name == "weekends" {
                return roll::try_apply(start, conv, |d| {
                    !matches!(d.day_of_week(), Weekday::Sat | Weekday::Sun)
                })
                .map(encoded)
                .map_err(|e| format!("{e:?}"));
            }
            named_roll(start, conv, name)
        }

        #[cfg(feature = "calendars")]
        pub fn calendar(value: &str) -> Result<regit_daycount::Calendar, String> {
            use regit_daycount::Calendar;
            Ok(match value {
                "target2" => Calendar::Target2,
                "usnyse" => Calendar::UnitedStatesNyse,
                "usfed" => Calendar::UnitedStatesFederalReserve,
                "uk" => Calendar::UnitedKingdom,
                "japan" => Calendar::Japan,
                "switzerland" => Calendar::Switzerland,
                "hongkong" => Calendar::HongKong,
                "singapore" => Calendar::Singapore,
                "luxembourg" => Calendar::Luxembourg,
                "brazilb3" => Calendar::BrazilB3,
                other => return Err(format!("Unknown calendar {other}")),
            })
        }

        #[cfg(feature = "calendars")]
        fn named_roll(start: Date, conv: Roll, name: &str) -> Result<f64, String> {
            use regit_daycount::calendar::{self, CoveragePolicy};
            calendar::try_adjust_with_policy(
                start,
                conv,
                calendar(name)?,
                CoveragePolicy::AllowProjected,
            )
            .map(encoded)
            .map_err(|e| format!("{e:?}"))
        }

        #[cfg(not(feature = "calendars"))]
        fn named_roll(_start: Date, _conv: Roll, _name: &str) -> Result<f64, String> {
            Err("calendars feature is disabled".to_owned())
        }

        #[cfg(feature = "calendars")]
        fn calendar_operation(fields: &[&str]) -> Result<f64, String> {
            use regit_daycount::calendar::{self, CoveragePolicy};
            let (name, operation) = fields[2].split_once(':').unwrap();
            if operation.contains('.') {
                return composite_operation(fields, name, operation);
            }
            let start = date(fields[3]);
            let cal = calendar(name)?;
            let policy = CoveragePolicy::AllowProjected;
            Ok(match operation {
                "is_weekend" => f64::from(u8::from(calendar::is_weekend(start))),
                "next_business_day" => encoded(
                    calendar::try_next_business_day_with_policy(start, cal, policy)
                        .map_err(|e| format!("{e:?}"))?,
                ),
                "previous_business_day" => encoded(
                    calendar::try_previous_business_day_with_policy(start, cal, policy)
                        .map_err(|e| format!("{e:?}"))?,
                ),
                "add_business_days" => encoded(
                    calendar::try_add_business_days_with_policy(
                        start,
                        fields[7].parse().unwrap(),
                        cal,
                        policy,
                    )
                    .map_err(|e| format!("{e:?}"))?,
                ),
                "business_days_between" => f64::from(
                    calendar::try_business_days_between_with_policy(
                        start,
                        date(fields[4]),
                        cal,
                        policy,
                    )
                    .map_err(|e| format!("{e:?}"))?,
                ),
                "bus_252_for_calendar" => calendar::try_bus_252_for_calendar_with_policy(
                    start,
                    date(fields[4]),
                    cal,
                    policy,
                )
                .map_err(|e| format!("{e:?}"))?,
                other => return Err(format!("Unknown calendar operation {other}")),
            })
        }

        #[cfg(feature = "calendars")]
        fn composite_operation(
            fields: &[&str],
            name: &str,
            operation: &str,
        ) -> Result<f64, String> {
            use regit_daycount::Calendar;
            use regit_daycount::calendar::{Composite, CoveragePolicy, JointBusiness};
            let kinds: &'static [Calendar] = match name {
                "single-target2" => &[Calendar::Target2],
                "target2+usnyse" => &[Calendar::Target2, Calendar::UnitedStatesNyse],
                "target2+luxembourg" => &[Calendar::Target2, Calendar::Luxembourg],
                "usnyse+uk" => &[Calendar::UnitedStatesNyse, Calendar::UnitedKingdom],
                "japan+hongkong+singapore" => {
                    &[Calendar::Japan, Calendar::HongKong, Calendar::Singapore]
                }
                other => return Err(format!("Unknown composite {other}")),
            };
            let (kind, operation) = operation.split_once('.').unwrap();
            let start = date(fields[3]);
            let policy = CoveragePolicy::AllowProjected;
            macro_rules! query {
        ($method:ident, $($args:expr),*) => {
            match kind {
                "Composite" => Composite::new(kinds).unwrap().$method($($args),*),
                "JointBusiness" => JointBusiness::new(kinds).unwrap().$method($($args),*),
                other => return Err(format!("Unknown combinator {other}")),
            }.map_err(|e| format!("{e:?}"))
        };
    }
            match operation {
                "is_business_day" => query!(try_is_business_day_with_policy, start, policy)
                    .map(|v| f64::from(u8::from(v))),
                "is_holiday" => query!(try_is_holiday_with_policy, start, policy)
                    .map(|v| f64::from(u8::from(v))),
                "adjust" => query!(
                    try_adjust_with_policy,
                    start,
                    Roll::ModifiedFollowing,
                    policy
                )
                .map(encoded),
                "add_business_days" => query!(
                    try_add_business_days_with_policy,
                    start,
                    fields[7].parse().unwrap(),
                    policy
                )
                .map(encoded),
                "business_days_between" => query!(
                    try_business_days_between_with_policy,
                    start,
                    date(fields[4]),
                    policy
                )
                .map(f64::from),
                "bus_252" => query!(try_bus_252_with_policy, start, date(fields[4]), policy),
                other => Err(format!("Unknown composite operation {other}")),
            }
        }

        #[cfg(not(feature = "calendars"))]
        fn calendar_operation(_fields: &[&str]) -> Result<f64, String> {
            Err("calendars feature is disabled".to_owned())
        }
    }

    pub fn date(value: &str) -> Date {
        let mut parts = value.split('-');
        Date::ymd(
            parts.next().unwrap().parse().unwrap(),
            parts.next().unwrap().parse().unwrap(),
            parts.next().unwrap().parse().unwrap(),
        )
        .unwrap()
    }

    pub fn evaluate(fields: &[&str]) -> Result<f64, String> {
        if let Some(result) = operations::evaluate(fields) {
            return result;
        }
        let start = date(fields[3]);
        if matches!(fields[1], "business" | "holiday") {
            return business(start, fields[2], fields[1] == "holiday");
        }
        if fields[1] == "bus252" {
            return bus252(start, date(fields[4]));
        }
        let basis = match fields[2] {
            "act360" => DayCount::Act360,
            "act365f" => DayCount::Act365F,
            "act365l" => DayCount::Act365L,
            "actactisda" => DayCount::ActActIsda,
            "actacticma" => DayCount::ActActIcma,
            "nl365" => DayCount::Nl365,
            "oneone" => DayCount::OneOne,
            "thirty360bond" => DayCount::Thirty360BondBasis,
            "thirtye360" => DayCount::ThirtyE360,
            "thirtye360isda" => DayCount::ThirtyE360Isda,
            name => return Err(format!("Unknown basis {name}")),
        };
        let coupon = (fields[5] != "-").then(|| CouponPeriod {
            start: date(fields[5]),
            end: date(fields[6]),
            frequency: fields[7].parse().unwrap(),
        });
        let context = DayCountContext {
            coupon,
            end_is_maturity: match fields[8] {
                "0" => Some(false),
                "1" => Some(true),
                _ => None,
            },
        };
        day_count::try_fraction(start, date(fields[4]), basis, context)
            .map_err(|e| format!("{e:?}"))
    }

    #[cfg(feature = "calendars")]
    fn business(date: Date, name: &str, holiday: bool) -> Result<f64, String> {
        use regit_daycount::calendar::{self, CoveragePolicy};
        let cal = operations::calendar(name)?;
        let result = if holiday {
            calendar::try_is_holiday_with_policy(date, cal, CoveragePolicy::AllowProjected)
        } else {
            calendar::try_is_business_day_with_policy(date, cal, CoveragePolicy::AllowProjected)
        };
        let actual = result.map_err(|e| format!("{e:?}"))?;
        Ok(f64::from(u8::from(actual)))
    }

    #[cfg(not(feature = "calendars"))]
    fn business(_date: Date, _name: &str, _holiday: bool) -> Result<f64, String> {
        Err("calendars feature is disabled".to_owned())
    }

    #[cfg(feature = "calendars")]
    fn bus252(start: Date, end: Date) -> Result<f64, String> {
        use regit_daycount::calendar::{self, CoveragePolicy};
        calendar::try_bus_252_for_calendar_with_policy(
            start,
            end,
            regit_daycount::Calendar::Target2,
            CoveragePolicy::AllowProjected,
        )
        .map_err(|e| format!("{e:?}"))
    }

    #[cfg(not(feature = "calendars"))]
    fn bus252(_start: Date, _end: Date) -> Result<f64, String> {
        Err("calendars feature is disabled".to_owned())
    }
}

mod date_roll {

    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    use regit_daycount::errors::ValidationError;
    use regit_daycount::roll::{self, RollError};
    use regit_daycount::{Date, Roll, Weekday};

    use super::Args;

    fn iso(date: Date) -> String {
        format!("{:04}-{:02}-{:02}", date.year(), date.month(), date.day())
    }

    fn validation_result(result: Result<Date, ValidationError>) -> String {
        match result {
            Ok(date) => format!("Ok:date:{}", iso(date)),
            Err(ValidationError::InvalidDate { rule }) => {
                format!("Err:ValidationError:InvalidDate:{rule}")
            }
            Err(ValidationError::OutOfRange { what }) => {
                format!("Err:ValidationError:OutOfRange:{what}")
            }
            Err(error) => panic!("unmapped validation error: {error:?}"),
        }
    }

    fn roll_result(result: Result<Date, RollError>) -> String {
        match result {
            Ok(date) => format!("Ok:date:{}", iso(date)),
            Err(RollError::DateBoundary { date }) => {
                format!("Err:RollError:DateBoundary:{}", iso(date))
            }
            Err(RollError::WalkLimitExceeded { limit }) => {
                format!("Err:RollError:WalkLimitExceeded:{limit}")
            }
            Err(error) => panic!("unmapped roll error: {error:?}"),
        }
    }

    fn weekday(value: &str) -> Weekday {
        match value {
            "Mon" => Weekday::Mon,
            "Tue" => Weekday::Tue,
            "Wed" => Weekday::Wed,
            "Thu" => Weekday::Thu,
            "Fri" => Weekday::Fri,
            "Sat" => Weekday::Sat,
            "Sun" => Weekday::Sun,
            _ => panic!("unknown weekday {value}"),
        }
    }

    fn convention(value: &str) -> Roll {
        match value {
            "Unadjusted" => Roll::Unadjusted,
            "Following" => Roll::Following,
            "ModifiedFollowing" => Roll::ModifiedFollowing,
            "Preceding" => Roll::Preceding,
            "ModifiedPreceding" => Roll::ModifiedPreceding,
            "Nearest" => Roll::Nearest,
            "EndOfMonth" => Roll::EndOfMonth,
            _ => panic!("unknown roll {value}"),
        }
    }

    fn hash<T: Hash>(value: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    fn validation_error(args: &Args) -> ValidationError {
        validation_error_fields(args.get("variant"), args.get("payload"))
    }

    fn validation_error_fields(variant: &str, payload: &str) -> ValidationError {
        // Public errors accept arbitrary &'static str. Fixtures own their payloads;
        // this short-lived integration-test process can give those a static life.
        let payload: &'static str = Box::leak(payload.to_owned().into_boxed_str());
        match variant {
            "InvalidDate" => ValidationError::InvalidDate { rule: payload },
            "OutOfRange" => ValidationError::OutOfRange { what: payload },
            value => panic!("unknown validation error {value}"),
        }
    }

    fn roll_error(args: &Args) -> RollError {
        match args.get("variant") {
            "DateBoundary" => RollError::DateBoundary {
                date: args.date("date"),
            },
            "WalkLimitExceeded" => RollError::WalkLimitExceeded {
                limit: args.parse("limit"),
            },
            value => panic!("unknown roll error {value}"),
        }
    }

    // Dispatch stays together; Clone is called explicitly to observe its contract.
    #[allow(clippy::clone_on_copy, clippy::too_many_lines)]
    pub fn evaluate(api: &str, args: &Args) -> Option<Result<String, String>> {
        let output = match api {
            "date::Date::MIN_YEAR" => format!("int:{}", Date::MIN_YEAR),
            "date::Date::MAX_YEAR" => format!("int:{}", Date::MAX_YEAR),
            "date::Date::ymd" => validation_result(Date::ymd(
                args.parse("year"),
                args.parse("month"),
                args.parse("day"),
            )),
            "date::Date::year" => format!("int:{}", args.date("date").year()),
            "date::Date::month" => format!("int:{}", args.date("date").month()),
            "date::Date::day" => format!("int:{}", args.date("date").day()),
            "date::Date::is_leap_year" => {
                format!("bool:{}", Date::is_leap_year(args.parse("year")))
            }
            "date::Date::days_in_month" => format!(
                "int:{}",
                Date::days_in_month(args.parse("year"), args.parse("month"))
            ),
            "date::Date::day_of_week" => {
                format!("text:{:?}", args.date("date").day_of_week())
            }
            "date::Date::try_add_days" => {
                validation_result(args.date("date").try_add_days(args.parse("days")))
            }
            "date::Date::try_add_months_eom_aware" => validation_result(
                args.date("date")
                    .try_add_months_eom_aware(args.parse("months")),
            ),
            "date::Date::nth_weekday_of_month" => validation_result(Date::nth_weekday_of_month(
                args.parse("year"),
                args.parse("month"),
                args.parse("n"),
                weekday(args.get("weekday")),
            )),
            "date::Date::try_easter_sunday" => {
                validation_result(Date::try_easter_sunday(args.parse("year")))
            }
            "date::Date::days_between" => {
                format!("int:{}", args.date("date").days_between(args.date("other")))
            }
            "date::Date::Debug" => format!("text:{:?}", args.date("date")),
            "date::Date::traits" => {
                let date = args.date("date");
                let other = args.date("other");
                let cloned = date.clone();
                let copied = date;
                let cmp = date.cmp(&other);
                format!(
                    "text:eq={},cmp={cmp:?},partial_cmp={:?},clone={},copy={},hash_equal={}",
                    date == other,
                    date.partial_cmp(&other),
                    date == cloned,
                    date == copied,
                    hash(&date) == hash(&cloned)
                )
            }
            "date::Weekday::traits" => {
                let day = weekday(args.get("weekday"));
                let other = weekday(args.get("other"));
                let cloned = day.clone();
                let copied = day;
                format!(
                    "text:debug={day:?},eq={},clone={},copy={},hash_equal={}",
                    day == other,
                    day == cloned,
                    day == copied,
                    hash(&day) == hash(&cloned)
                )
            }
            "roll::Roll::traits" => {
                let roll = convention(args.get("roll"));
                let other = convention(args.get("other"));
                let cloned = roll.clone();
                let copied = roll;
                format!(
                    "text:debug={roll:?},eq={},clone={},copy={},hash_equal={}",
                    roll == other,
                    roll == cloned,
                    roll == copied,
                    hash(&roll) == hash(&cloned)
                )
            }
            "roll::try_apply" => {
                let calls = std::cell::Cell::new(0_u32);
                let predicate = |date: Date| {
                    calls.set(calls.get() + 1);
                    match args.get("predicate") {
                        "all" => true,
                        "none" | "counted_none" => false,
                        "weekends" => !matches!(date.day_of_week(), Weekday::Sat | Weekday::Sun),
                        "single" => date == args.date("target"),
                        "except" => date != args.date("target"),
                        "pair" => date == args.date("target") || date == args.date("target2"),
                        "before" => date < args.date("target"),
                        "after" => date > args.date("target"),
                        value => panic!("unknown predicate {value}"),
                    }
                };
                let actual = roll_result(roll::try_apply(
                    args.date("date"),
                    convention(args.get("roll")),
                    predicate,
                ));
                if args.get("predicate") == "counted_none" {
                    format!("predicate_calls:{}|{actual}", calls.get())
                } else {
                    actual
                }
            }
            "errors::ValidationError::PartialEq" => format!(
                "bool:{}",
                validation_error(args)
                    == validation_error_fields(
                        args.get("other_variant"),
                        args.get("other_payload")
                    )
            ),
            "roll::RollError::PartialEq" => {
                let other = match args.get("other_variant") {
                    "DateBoundary" => RollError::DateBoundary {
                        date: args.date("other_date"),
                    },
                    "WalkLimitExceeded" => RollError::WalkLimitExceeded {
                        limit: args.parse("other_limit"),
                    },
                    value => panic!("unknown roll error {value}"),
                };
                format!("bool:{}", roll_error(args) == other)
            }
            "errors::ValidationError::Display" => {
                format!("text:{}", validation_error(args))
            }
            "errors::ValidationError::Debug" => {
                format!("text:{:?}", validation_error(args))
            }
            "errors::ValidationError::source" => {
                let error = validation_error(args);
                let object: &dyn std::error::Error = &error;
                if object.source().is_none() {
                    "text:None".into()
                } else {
                    "text:Some".into()
                }
            }
            "errors::ValidationError::traits" => {
                let error = validation_error(args);
                let cloned = error.clone();
                let copied = error;
                let distinct = match error {
                    ValidationError::InvalidDate { .. } => {
                        ValidationError::OutOfRange { what: "other" }
                    }
                    _ => ValidationError::InvalidDate { rule: "other" },
                };
                format!(
                    "text:clone={},copy={},eq={},distinct={}",
                    error == cloned,
                    error == copied,
                    error == validation_error(args),
                    error == distinct
                )
            }
            "roll::RollError::Display" => format!("text:{}", roll_error(args)),
            "roll::RollError::Debug" => format!("text:{:?}", roll_error(args)),
            "roll::RollError::source" => {
                let error = roll_error(args);
                let object: &dyn std::error::Error = &error;
                if object.source().is_none() {
                    "text:None".into()
                } else {
                    "text:Some".into()
                }
            }
            "roll::RollError::traits" => {
                let error = roll_error(args);
                let cloned = error.clone();
                let copied = error;
                let distinct = match error {
                    RollError::DateBoundary { .. } => RollError::WalkLimitExceeded { limit: 7 },
                    _ => RollError::DateBoundary {
                        date: Date::ymd(2026, 1, 1).unwrap(),
                    },
                };
                format!(
                    "text:clone={},copy={},eq={},distinct={}",
                    error == cloned,
                    error == copied,
                    error == roll_error(args),
                    error == distinct
                )
            }
            _ => return None,
        };
        Some(Ok(output))
    }
}

mod day_counts {

    use super::Args;
    use regit_daycount::day_count::{self, CouponPeriod, DayCountContext, DayCountError};
    use regit_daycount::{Date, DayCount, Weekday};
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    fn basis(value: &str) -> DayCount {
        match value {
            "Act360" => DayCount::Act360,
            "Act365F" => DayCount::Act365F,
            "ActActIsda" => DayCount::ActActIsda,
            "ActActIcma" => DayCount::ActActIcma,
            "Thirty360BondBasis" => DayCount::Thirty360BondBasis,
            "ThirtyE360" => DayCount::ThirtyE360,
            "ThirtyE360Isda" => DayCount::ThirtyE360Isda,
            "Act365L" => DayCount::Act365L,
            "Nl365" => DayCount::Nl365,
            "Bus252" => DayCount::Bus252,
            "OneOne" => DayCount::OneOne,
            value => panic!("unknown basis {value}"),
        }
    }
    fn coupon(args: &Args, prefix: &str) -> CouponPeriod {
        CouponPeriod {
            start: args.date(&format!("{prefix}ref_start")),
            end: args.date(&format!("{prefix}ref_end")),
            frequency: args.parse(&format!("{prefix}frequency")),
        }
    }
    fn context(args: &Args, prefix: &str) -> DayCountContext {
        DayCountContext {
            coupon: match args.get(&format!("{prefix}coupon")) {
                "none" => None,
                "some" => Some(coupon(args, prefix)),
                value => panic!("unknown coupon flag {value}"),
            },
            end_is_maturity: match args.get(&format!("{prefix}maturity")) {
                "none" => None,
                "true" => Some(true),
                "false" => Some(false),
                value => panic!("unknown maturity flag {value}"),
            },
        }
    }
    fn error(value: &str) -> DayCountError {
        match value {
            "MissingCouponPeriod" => DayCountError::MissingCouponPeriod,
            "MissingMaturity" => DayCountError::MissingMaturity,
            "MissingBusinessDayPredicate" => DayCountError::MissingBusinessDayPredicate,
            "ZeroFrequency" => DayCountError::ZeroFrequency,
            "InvalidReferencePeriod" => DayCountError::InvalidReferencePeriod,
            "AccrualOutsideReferencePeriod" => DayCountError::AccrualOutsideReferencePeriod,
            value => panic!("unknown day-count error {value}"),
        }
    }
    fn result(value: Result<f64, DayCountError>) -> String {
        match value {
            Ok(number) => format!("Ok:f64:{number:.17}"),
            Err(error) => format!("Err:DayCountError:{error:?}"),
        }
    }
    fn numeric(value: f64) -> String {
        result(Ok(value))
    }
    fn iso(value: Date) -> String {
        format!(
            "{:04}-{:02}-{:02}",
            value.year(),
            value.month(),
            value.day()
        )
    }
    fn predicate(args: &Args, value: Date) -> bool {
        match args.get("predicate") {
            "all" => true,
            "none" | "counted_none" => false,
            "weekends" => !matches!(value.day_of_week(), Weekday::Sat | Weekday::Sun),
            "no_sep7" => {
                !(matches!(value.day_of_week(), Weekday::Sat | Weekday::Sun)
                    || value.month() == 9 && value.day() == 7)
            }
            "single" => value == args.date("target"),
            value => panic!("unknown predicate {value}"),
        }
    }
    fn hash<T: Hash>(value: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }
    // Dispatch stays together; Clone is called explicitly to observe its contract.
    #[allow(clippy::clone_on_copy, clippy::too_many_lines)]
    pub fn evaluate(api: &str, args: &Args) -> Option<Result<String, String>> {
        let calls = std::cell::Cell::new(0_u32);
        let observed_predicate = |date| {
            calls.set(calls.get() + 1);
            predicate(args, date)
        };
        let out = match api {
            "day_count::act_360::fraction" => numeric(day_count::act_360::fraction(
                args.date("start"),
                args.date("end"),
            )),
            "day_count::act_365f::fraction" => numeric(day_count::act_365f::fraction(
                args.date("start"),
                args.date("end"),
            )),
            "day_count::act_act_isda::fraction" => numeric(day_count::act_act_isda::fraction(
                args.date("start"),
                args.date("end"),
            )),
            "day_count::thirty_360_bond_basis::fraction" => numeric(
                day_count::thirty_360_bond_basis::fraction(args.date("start"), args.date("end")),
            ),
            "day_count::thirty_e_360::fraction" => numeric(day_count::thirty_e_360::fraction(
                args.date("start"),
                args.date("end"),
            )),
            "day_count::thirty_e_360_isda::fraction" => {
                numeric(day_count::thirty_e_360_isda::fraction(
                    args.date("start"),
                    args.date("end"),
                    args.parse("maturity"),
                ))
            }
            "day_count::nl_365::fraction" => numeric(day_count::nl_365::fraction(
                args.date("start"),
                args.date("end"),
            )),
            "day_count::one_one::fraction" => numeric(day_count::one_one::fraction(
                args.date("start"),
                args.date("end"),
            )),
            "day_count::bus_252::fraction" => numeric(day_count::bus_252::fraction(
                args.date("start"),
                args.date("end"),
                observed_predicate,
            )),
            "day_count::act_365l::try_fraction" => result(day_count::act_365l::try_fraction(
                args.date("start"),
                args.date("end"),
                args.date("ref_start"),
                args.date("ref_end"),
                args.parse("frequency"),
            )),
            "day_count::act_act_icma::try_fraction" => {
                result(day_count::act_act_icma::try_fraction(
                    args.date("start"),
                    args.date("end"),
                    args.date("ref_start"),
                    args.date("ref_end"),
                    args.parse("frequency"),
                ))
            }
            "day_count::try_fraction" => result(day_count::try_fraction(
                args.date("start"),
                args.date("end"),
                basis(args.get("basis")),
                context(args, ""),
            )),
            "day_count::try_fraction_with_business_days" => {
                result(day_count::try_fraction_with_business_days(
                    args.date("start"),
                    args.date("end"),
                    basis(args.get("basis")),
                    context(args, ""),
                    observed_predicate,
                ))
            }
            "day_count::DayCount::Debug" => format!("text:{:?}", basis(args.get("basis"))),
            "day_count::DayCount::traits" => {
                let value = basis(args.get("basis"));
                let other = basis(args.get("other"));
                let cloned = value.clone();
                let copied = value;
                format!(
                    "text:debug={value:?},eq={},clone={},copy={},hash_equal={}",
                    value == other,
                    value == cloned,
                    value == copied,
                    hash(&value) == hash(&cloned)
                )
            }
            "day_count::CouponPeriod::start" => format!("Ok:date:{}", iso(coupon(args, "").start)),
            "day_count::CouponPeriod::end" => format!("Ok:date:{}", iso(coupon(args, "").end)),
            "day_count::CouponPeriod::frequency" => format!("int:{}", coupon(args, "").frequency),
            "day_count::CouponPeriod::Debug" => format!("text:{:?}", coupon(args, "")),
            "day_count::CouponPeriod::traits" => {
                let value = coupon(args, "");
                let other = coupon(args, "other_");
                let cloned = value.clone();
                let copied = value;
                format!(
                    "text:eq={},clone={},copy={},hash_equal={}",
                    value == other,
                    value == cloned,
                    value == copied,
                    hash(&value) == hash(&cloned)
                )
            }
            "day_count::DayCountContext::default" => {
                format!("text:{:?}", DayCountContext::default())
            }
            "day_count::DayCountContext::coupon" => format!("text:{:?}", context(args, "").coupon),
            "day_count::DayCountContext::end_is_maturity" => {
                format!("text:{:?}", context(args, "").end_is_maturity)
            }
            "day_count::DayCountContext::Debug" => format!("text:{:?}", context(args, "")),
            "day_count::DayCountContext::traits" => {
                let value = context(args, "");
                let other = context(args, "other_");
                let cloned = value.clone();
                let copied = value;
                format!(
                    "text:eq={},clone={},copy={},hash_equal={}",
                    value == other,
                    value == cloned,
                    value == copied,
                    hash(&value) == hash(&cloned)
                )
            }
            "day_count::error::DayCountError::Display" => {
                format!("text:{}", error(args.get("error")))
            }
            "day_count::error::DayCountError::Debug" => {
                format!("text:{:?}", error(args.get("error")))
            }
            "day_count::error::DayCountError::source" => {
                let value = error(args.get("error"));
                let object: &dyn std::error::Error = &value;
                if object.source().is_none() {
                    "text:None".into()
                } else {
                    "text:Some".into()
                }
            }
            "day_count::error::DayCountError::PartialEq" => format!(
                "bool:{}",
                error(args.get("error")) == error(args.get("other"))
            ),
            "day_count::error::DayCountError::traits" => {
                let value = error(args.get("error"));
                let cloned = value.clone();
                let copied = value;
                format!(
                    "text:clone={},copy={},eq={},hash_equal={}",
                    value == cloned,
                    value == copied,
                    value == error(args.get("error")),
                    hash(&value) == hash(&cloned)
                )
            }
            _ => return None,
        };
        if args.has("predicate")
            && args.get("predicate") == "counted_none"
            && matches!(
                api,
                "day_count::bus_252::fraction" | "day_count::try_fraction_with_business_days"
            )
        {
            Some(Ok(format!("predicate_calls:{}|{out}", calls.get())))
        } else {
            Some(Ok(out))
        }
    }
}

#[cfg(feature = "calendars")]
mod calendars {

    // Copyright 2026 Regit.io — SPDX-License-Identifier: Apache-2.0

    use regit_daycount::calendar::{self, Calendar, CalendarError, CoveragePolicy};
    use regit_daycount::{Date, Roll};

    fn calendar(value: &str) -> Calendar {
        match value {
            "Target2" => Calendar::Target2,
            "Luxembourg" => Calendar::Luxembourg,
            "UnitedStatesNyse" => Calendar::UnitedStatesNyse,
            "UnitedStatesFederalReserve" => Calendar::UnitedStatesFederalReserve,
            "UnitedKingdom" => Calendar::UnitedKingdom,
            "Japan" => Calendar::Japan,
            "Switzerland" => Calendar::Switzerland,
            "HongKong" => Calendar::HongKong,
            "Singapore" => Calendar::Singapore,
            "BrazilB3" => Calendar::BrazilB3,
            other => panic!("unknown oracle calendar {other}"),
        }
    }

    fn policy(args: &super::Args) -> CoveragePolicy {
        if args.has("policy") && args.get("policy") == "AllowProjected" {
            CoveragePolicy::AllowProjected
        } else {
            CoveragePolicy::VerifiedOnly
        }
    }

    fn roll(args: &super::Args) -> Roll {
        match args.get("roll") {
            "Unadjusted" => Roll::Unadjusted,
            "Following" => Roll::Following,
            "ModifiedFollowing" => Roll::ModifiedFollowing,
            "Preceding" => Roll::Preceding,
            "ModifiedPreceding" => Roll::ModifiedPreceding,
            "Nearest" => Roll::Nearest,
            "EndOfMonth" => Roll::EndOfMonth,
            other => panic!("unknown oracle roll {other}"),
        }
    }

    fn date_text(date: Date) -> String {
        format!("{:04}-{:02}-{:02}", date.year(), date.month(), date.day())
    }

    fn error_text(error: CalendarError) -> String {
        match error {
            CalendarError::EmptyComposite => "Err:CalendarError:EmptyComposite".into(),
            CalendarError::OutOfCoverage { calendar, date } => {
                format!(
                    "Err:CalendarError:OutOfCoverage:{calendar:?}:{}",
                    date_text(date)
                )
            }
            CalendarError::UnverifiedDate { calendar, date } => {
                format!(
                    "Err:CalendarError:UnverifiedDate:{calendar:?}:{}",
                    date_text(date)
                )
            }
            CalendarError::DateBoundary { date } => {
                format!("Err:CalendarError:DateBoundary:{}", date_text(date))
            }
            CalendarError::WalkLimitExceeded { limit } => {
                format!("Err:CalendarError:WalkLimitExceeded:{limit}")
            }
            _ => panic!("unhandled public calendar error"),
        }
    }

    fn bool_result(result: Result<bool, CalendarError>) -> String {
        result.map_or_else(error_text, |value| format!("Ok:bool:{value}"))
    }

    fn date_result(result: Result<Date, CalendarError>) -> String {
        result.map_or_else(error_text, |date| format!("Ok:date:{}", date_text(date)))
    }

    fn count_result(result: Result<u32, CalendarError>) -> String {
        result.map_or_else(error_text, |value| format!("Ok:int:{value}"))
    }

    fn fraction_result(result: Result<f64, CalendarError>) -> String {
        result.map_or_else(error_text, |value| format!("Ok:f64:{value:.17}"))
    }

    fn pair_text(pair: (i32, i32)) -> String {
        format!("text:{},{}", pair.0, pair.1)
    }

    fn parse_error(args: &super::Args) -> CalendarError {
        match args.get("kind") {
            "EmptyComposite" => CalendarError::EmptyComposite,
            "OutOfCoverage" => CalendarError::OutOfCoverage {
                calendar: calendar(args.get("calendar")),
                date: args.date("date"),
            },
            "UnverifiedDate" => CalendarError::UnverifiedDate {
                calendar: calendar(args.get("calendar")),
                date: args.date("date"),
            },
            "DateBoundary" => CalendarError::DateBoundary {
                date: args.date("date"),
            },
            "WalkLimitExceeded" => CalendarError::WalkLimitExceeded {
                limit: args.parse("limit"),
            },
            other => panic!("unknown oracle calendar error {other}"),
        }
    }

    fn hash_contract<T: std::hash::Hash + PartialEq>(left: &T, right: &T) -> bool {
        use std::hash::Hasher;
        let mut left_hash = std::collections::hash_map::DefaultHasher::new();
        let mut right_hash = std::collections::hash_map::DefaultHasher::new();
        left.hash(&mut left_hash);
        right.hash(&mut right_hash);
        left != right || left_hash.finish() == right_hash.finish()
    }

    // Dispatch stays together; Clone is called explicitly to observe its contract.
    #[allow(clippy::clone_on_copy, clippy::too_many_lines)]
    fn named_operation(api: &str, args: &super::Args) -> Option<String> {
        let cal = if args.has("calendar") {
            calendar(args.get("calendar"))
        } else {
            Calendar::Target2
        };
        let output = match api {
            "calendar::is_weekend" => {
                format!("Ok:bool:{}", calendar::is_weekend(args.date("date")))
            }
            "calendar::CALENDAR_MAX_STEPS" => format!("Ok:int:{}", calendar::CALENDAR_MAX_STEPS),
            "calendar::CoveragePolicy::default" => format!("text:{:?}", CoveragePolicy::default()),
            "calendar::coverage" => {
                let value = calendar::coverage(cal);
                let verified = value
                    .verified_years
                    .map_or("None".into(), |(lo, hi)| format!("{lo},{hi}"));
                format!(
                    "text:available={},{}|verified={verified}|verified_on={}",
                    value.available_years.0,
                    value.available_years.1,
                    value.verified_on.unwrap_or("None")
                )
            }
            "calendar::CalendarCoverage::available_years" => {
                pair_text(calendar::coverage(cal).available_years)
            }
            "calendar::CalendarCoverage::verified_years" => calendar::coverage(cal)
                .verified_years
                .map_or("text:None".into(), pair_text),
            "calendar::CalendarCoverage::verified_on" => format!(
                "text:{}",
                calendar::coverage(cal).verified_on.unwrap_or("None")
            ),
            "calendar::Calendar::Debug" => format!("text:{cal:?}"),
            "calendar::Calendar::Clone" => format!("text:{:?}", Clone::clone(&cal)),
            "calendar::Calendar::PartialEq" => {
                format!("Ok:bool:{}", cal == calendar(args.get("other")))
            }
            "calendar::Calendar::Hash" => format!(
                "Ok:bool:{}",
                hash_contract(&cal, &calendar(args.get("other")))
            ),
            "calendar::CalendarCoverage::Debug" => format!("text:{:?}", calendar::coverage(cal)),
            "calendar::CalendarCoverage::Clone" => {
                let value = calendar::coverage(cal);
                format!("Ok:bool:{}", Clone::clone(&value) == value)
            }
            "calendar::CalendarCoverage::PartialEq" => format!(
                "Ok:bool:{}",
                calendar::coverage(cal) == calendar::coverage(calendar(args.get("other")))
            ),
            "calendar::CoveragePolicy::Debug" => format!("text:{:?}", policy(args)),
            "calendar::CoveragePolicy::Clone" => format!("text:{:?}", Clone::clone(&policy(args))),
            "calendar::CoveragePolicy::PartialEq" | "calendar::CoveragePolicy::Hash" => {
                let other = if args.get("other") == "AllowProjected" {
                    CoveragePolicy::AllowProjected
                } else {
                    CoveragePolicy::VerifiedOnly
                };
                let value = if api.ends_with("Hash") {
                    hash_contract(&policy(args), &other)
                } else {
                    policy(args) == other
                };
                format!("Ok:bool:{value}")
            }
            "calendar::try_is_holiday" => {
                bool_result(calendar::try_is_holiday(args.date("date"), cal))
            }
            "calendar::try_is_holiday_with_policy" => bool_result(
                calendar::try_is_holiday_with_policy(args.date("date"), cal, policy(args)),
            ),
            "calendar::try_is_business_day" => {
                bool_result(calendar::try_is_business_day(args.date("date"), cal))
            }
            "calendar::try_is_business_day_with_policy" => bool_result(
                calendar::try_is_business_day_with_policy(args.date("date"), cal, policy(args)),
            ),
            "calendar::try_adjust" => {
                date_result(calendar::try_adjust(args.date("date"), roll(args), cal))
            }
            "calendar::try_adjust_with_policy" => date_result(calendar::try_adjust_with_policy(
                args.date("date"),
                roll(args),
                cal,
                policy(args),
            )),
            "calendar::try_next_business_day" => {
                date_result(calendar::try_next_business_day(args.date("date"), cal))
            }
            "calendar::try_next_business_day_with_policy" => date_result(
                calendar::try_next_business_day_with_policy(args.date("date"), cal, policy(args)),
            ),
            "calendar::try_previous_business_day" => {
                date_result(calendar::try_previous_business_day(args.date("date"), cal))
            }
            "calendar::try_previous_business_day_with_policy" => {
                date_result(calendar::try_previous_business_day_with_policy(
                    args.date("date"),
                    cal,
                    policy(args),
                ))
            }
            "calendar::try_add_business_days" => date_result(calendar::try_add_business_days(
                args.date("date"),
                args.parse("n"),
                cal,
            )),
            "calendar::try_add_business_days_with_policy" => {
                date_result(calendar::try_add_business_days_with_policy(
                    args.date("date"),
                    args.parse("n"),
                    cal,
                    policy(args),
                ))
            }
            "calendar::try_business_days_between" => count_result(
                calendar::try_business_days_between(args.date("start"), args.date("end"), cal),
            ),
            "calendar::try_business_days_between_with_policy" => {
                count_result(calendar::try_business_days_between_with_policy(
                    args.date("start"),
                    args.date("end"),
                    cal,
                    policy(args),
                ))
            }
            "calendar::try_bus_252_for_calendar" => fraction_result(
                calendar::try_bus_252_for_calendar(args.date("start"), args.date("end"), cal),
            ),
            "calendar::try_bus_252_for_calendar_with_policy" => {
                fraction_result(calendar::try_bus_252_for_calendar_with_policy(
                    args.date("start"),
                    args.date("end"),
                    cal,
                    policy(args),
                ))
            }
            "calendar::CalendarError::Display" => format!("text:{}", parse_error(args)),
            "calendar::CalendarError::Debug" => format!("text:{:?}", parse_error(args)),
            "calendar::CalendarError::Clone" => {
                let error = parse_error(args);
                format!("Ok:bool:{}", Clone::clone(&error) == error)
            }
            "calendar::CalendarError::PartialEq" => {
                let error = parse_error(args);
                let other = if args.parse::<u8>("same") == 1 {
                    error
                } else if error == CalendarError::EmptyComposite {
                    CalendarError::WalkLimitExceeded { limit: 0 }
                } else {
                    CalendarError::EmptyComposite
                };
                format!("Ok:bool:{}", error == other)
            }
            "calendar::CalendarError::source" => {
                use std::error::Error;
                format!(
                    "text:{}",
                    if parse_error(args).source().is_none() {
                        "None"
                    } else {
                        "Some"
                    }
                )
            }
            _ => return None,
        };
        Some(output)
    }

    fn market_operation(api: &str, args: &super::Args) -> Option<String> {
        macro_rules! constants {
        ($module:ident, $($name:ident),+ $(,)?) => {
            $(if api == concat!("calendar::", stringify!($module), "::", stringify!($name)) {
                return Some(pair_text(calendar::$module::$name));
            })+
        };
    }
        macro_rules! texts {
        ($module:ident, $($name:ident),+ $(,)?) => {
            $(if api == concat!("calendar::", stringify!($module), "::", stringify!($name)) {
                return Some(format!("text:{}", calendar::$module::$name));
            })+
        };
    }
        macro_rules! market {
            ($module:ident) => {
                if api == concat!("calendar::", stringify!($module), "::try_is_holiday") {
                    return Some(bool_result(calendar::$module::try_is_holiday(
                        args.date("date"),
                    )));
                }
            };
        }
        constants!(target2, VERIFIED_YEARS);
        constants!(luxembourg, VERIFIED_YEARS);
        constants!(
            united_states,
            COVERAGE,
            NYSE_VERIFIED_YEARS,
            FEDERAL_RESERVE_VERIFIED_YEARS
        );
        constants!(united_kingdom, COVERAGE, VERIFIED_YEARS);
        constants!(japan, COVERAGE, VERIFIED_YEARS);
        constants!(switzerland, COVERAGE, VERIFIED_YEARS);
        constants!(hong_kong, COVERAGE, VERIFIED_YEARS);
        constants!(singapore, COVERAGE, VERIFIED_YEARS);
        constants!(brazil, COVERAGE, VERIFIED_YEARS);
        texts!(target2, VERIFIED_ON);
        texts!(luxembourg, VERIFIED_ON);
        texts!(
            united_states,
            SNAPSHOT_DATE,
            NYSE_VERIFIED_ON,
            FEDERAL_RESERVE_VERIFIED_ON
        );
        texts!(united_kingdom, SNAPSHOT_DATE, VERIFIED_ON);
        texts!(japan, SNAPSHOT_DATE, VERIFIED_ON);
        texts!(switzerland, SNAPSHOT_DATE, VERIFIED_ON);
        texts!(hong_kong, SNAPSHOT_DATE, VERIFIED_ON);
        texts!(singapore, SNAPSHOT_DATE, VERIFIED_ON);
        texts!(brazil, VERIFIED_ON);
        market!(target2);
        market!(luxembourg);
        market!(united_kingdom);
        market!(japan);
        market!(switzerland);
        market!(hong_kong);
        market!(singapore);
        market!(brazil);
        match api {
            "calendar::united_states::try_is_nyse_holiday" => Some(bool_result(
                calendar::united_states::try_is_nyse_holiday(args.date("date")),
            )),
            "calendar::united_states::try_is_federal_reserve_holiday" => Some(bool_result(
                calendar::united_states::try_is_federal_reserve_holiday(args.date("date")),
            )),
            _ => None,
        }
    }

    fn markets(args: &super::Args) -> &'static [Calendar] {
        if args.get("markets").is_empty() {
            return &[];
        }
        Box::leak(
            args.get("markets")
                .split(',')
                .map(calendar)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        )
    }

    fn kinds_text(kinds: &[Calendar]) -> String {
        format!(
            "text:{}",
            kinds
                .iter()
                .map(|kind| format!("{kind:?}"))
                .collect::<Vec<_>>()
                .join(",")
        )
    }

    fn composite_operation(api: &str, args: &super::Args) -> Option<String> {
        macro_rules! composite {
            ($kind:ident) => {
                if let Some(operation) =
                    api.strip_prefix(concat!("calendar::composite::", stringify!($kind), "::"))
                {
                    let value = match calendar::$kind::new(markets(args)) {
                        Ok(value) => value,
                        Err(error) => return Some(error_text(error)),
                    };
                    return Some(match operation {
                        "new" | "kinds" => kinds_text(value.kinds()),
                        "try_is_holiday" => bool_result(value.try_is_holiday(args.date("date"))),
                        "try_is_holiday_with_policy" => bool_result(
                            value.try_is_holiday_with_policy(args.date("date"), policy(args)),
                        ),
                        "try_is_business_day" => {
                            bool_result(value.try_is_business_day(args.date("date")))
                        }
                        "try_is_business_day_with_policy" => bool_result(
                            value.try_is_business_day_with_policy(args.date("date"), policy(args)),
                        ),
                        "try_adjust" => {
                            date_result(value.try_adjust(args.date("date"), roll(args)))
                        }
                        "try_adjust_with_policy" => date_result(value.try_adjust_with_policy(
                            args.date("date"),
                            roll(args),
                            policy(args),
                        )),
                        "try_add_business_days" => date_result(
                            value.try_add_business_days(args.date("date"), args.parse("n")),
                        ),
                        "try_add_business_days_with_policy" => {
                            date_result(value.try_add_business_days_with_policy(
                                args.date("date"),
                                args.parse("n"),
                                policy(args),
                            ))
                        }
                        "try_business_days_between" => count_result(
                            value.try_business_days_between(args.date("start"), args.date("end")),
                        ),
                        "try_business_days_between_with_policy" => {
                            count_result(value.try_business_days_between_with_policy(
                                args.date("start"),
                                args.date("end"),
                                policy(args),
                            ))
                        }
                        "try_bus_252" => {
                            fraction_result(value.try_bus_252(args.date("start"), args.date("end")))
                        }
                        "try_bus_252_with_policy" => {
                            fraction_result(value.try_bus_252_with_policy(
                                args.date("start"),
                                args.date("end"),
                                policy(args),
                            ))
                        }
                        "Debug" => format!("text:{value:?}"),
                        "Clone" => kinds_text(Clone::clone(&value).kinds()),
                        _ => return None,
                    });
                }
            };
        }
        composite!(Composite);
        composite!(JointBusiness);
        None
    }

    pub fn evaluate(api: &str, args: &super::Args) -> Option<Result<String, String>> {
        named_operation(api, args)
            .or_else(|| market_operation(api, args))
            .or_else(|| composite_operation(api, args))
            .map(Ok)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().any(|arg| arg == "--observe") {
        execute()?;
    }
    Ok(())
}
