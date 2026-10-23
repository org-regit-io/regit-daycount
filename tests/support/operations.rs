// Copyright 2026 Regit.io — SPDX-License-Identifier: Apache-2.0

//! Rust API evaluation for independently generated date/roll/calendar cases.

use regit_daycount::{Date, Roll, Weekday, roll};

pub fn handles(fields: &[&str]) -> bool {
    matches!(fields[1], "date_op" | "roll_op" | "calendar_op")
}

pub fn supported(fields: &[&str]) -> bool {
    fields[1] == "date_op"
        || (fields[1] == "roll_op" && fields[2].starts_with("weekends:"))
        || (handles(fields) && cfg!(feature = "calendars"))
}

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
    let (year, month, day) = parts(value);
    Date::ymd(year, month, day).unwrap()
}

fn encoded(date: Date) -> f64 {
    f64::from(date.year() * 10000 + i32::from(date.month()) * 100 + i32::from(date.day()))
}

fn weekday(index: u8) -> Result<Weekday, String> {
    match index {
        0 => Ok(Weekday::Mon),
        1 => Ok(Weekday::Tue),
        2 => Ok(Weekday::Wed),
        3 => Ok(Weekday::Thu),
        4 => Ok(Weekday::Fri),
        5 => Ok(Weekday::Sat),
        6 => Ok(Weekday::Sun),
        _ => Err(format!("Unknown weekday {index}")),
    }
}

fn weekday_number(weekday: Weekday) -> u8 {
    match weekday {
        Weekday::Mon => 1,
        Weekday::Tue => 2,
        Weekday::Wed => 3,
        Weekday::Thu => 4,
        Weekday::Fri => 5,
        Weekday::Sat => 6,
        Weekday::Sun => 7,
    }
}

fn date_operation(fields: &[&str]) -> Result<f64, String> {
    let (year, month, day) = parts(fields[3]);
    if fields[2] == "ymd" {
        return Ok(Date::ymd(year, month, day).map_or(0.0, encoded));
    }
    let start = Date::ymd(year, month, day).unwrap();
    match fields[2] {
        "ymd_unchecked" => Ok(encoded(Date::ymd_unchecked(year, month, day))),
        "year" => Ok(f64::from(start.year())),
        "month" => Ok(f64::from(start.month())),
        "day" => Ok(f64::from(start.day())),
        "is_leap_year" => Ok(f64::from(u8::from(Date::is_leap_year(year)))),
        "days_in_month" => Ok(f64::from(Date::days_in_month(year, month))),
        "day_of_week" => Ok(f64::from(weekday_number(start.day_of_week()))),
        "add_days" => Ok(encoded(start.add_days(fields[7].parse().unwrap()))),
        "add_months_eom_aware" => Ok(encoded(
            start.add_months_eom_aware(fields[7].parse().unwrap()),
        )),
        "days_between" => Ok(f64::from(start.days_between(date(fields[4])))),
        "easter_sunday" => Ok(encoded(Date::easter_sunday(year))),
        "nth_weekday_of_month" => Ok(Date::nth_weekday_of_month(
            year,
            month,
            fields[7].parse().unwrap(),
            weekday(fields[5].parse().unwrap())?,
        )
        .map_or(0.0, encoded)),
        other => Err(format!("Unknown Date operation {other}")),
    }
}

// Exhaustive named mapping makes future public Roll additions a compile-time
// coverage decision, in addition to the Python source-inventory acceptance gate.
fn roll_name(roll: Roll) -> &'static str {
    match roll {
        Roll::Unadjusted => "Unadjusted",
        Roll::Following => "Following",
        Roll::ModifiedFollowing => "ModifiedFollowing",
        Roll::Preceding => "Preceding",
        Roll::ModifiedPreceding => "ModifiedPreceding",
        Roll::Nearest => "Nearest",
        Roll::EndOfMonth => "EndOfMonth",
    }
}

fn convention(value: &str) -> Result<Roll, String> {
    let roll = match value {
        "Unadjusted" => Roll::Unadjusted,
        "Following" => Roll::Following,
        "ModifiedFollowing" => Roll::ModifiedFollowing,
        "Preceding" => Roll::Preceding,
        "ModifiedPreceding" => Roll::ModifiedPreceding,
        "Nearest" => Roll::Nearest,
        "EndOfMonth" => Roll::EndOfMonth,
        other => return Err(format!("Unknown Roll {other}")),
    };
    assert_eq!(roll_name(roll), value);
    Ok(roll)
}

fn roll_operation(fields: &[&str]) -> Result<f64, String> {
    let (calendar, operation) = fields[2].split_once(':').unwrap();
    let start = date(fields[3]);
    let conv = convention(operation)?;
    if calendar == "weekends" {
        return Ok(encoded(roll::apply(start, conv, |date| {
            !matches!(date.day_of_week(), Weekday::Sat | Weekday::Sun)
        })));
    }
    named_roll(start, conv, calendar)
}

#[cfg(feature = "calendars")]
fn calendar(value: &str) -> Result<regit_daycount::Calendar, String> {
    use regit_daycount::Calendar;
    match value {
        "target2" => Ok(Calendar::Target2),
        "usnyse" => Ok(Calendar::UnitedStatesNyse),
        "uslegacy" => Ok(Calendar::UnitedStates),
        "usfed" => Ok(Calendar::UnitedStatesFederalReserve),
        "uk" => Ok(Calendar::UnitedKingdom),
        "japan" => Ok(Calendar::Japan),
        "switzerland" => Ok(Calendar::Switzerland),
        "hongkong" => Ok(Calendar::HongKong),
        "singapore" => Ok(Calendar::Singapore),
        "luxembourg" => Ok(Calendar::Luxembourg),
        other => Err(format!("Unknown calendar {other}")),
    }
}

#[cfg(feature = "calendars")]
fn named_roll(start: Date, conv: Roll, name: &str) -> Result<f64, String> {
    use regit_daycount::calendar::{self, CoveragePolicy};
    let cal = calendar(name)?;
    let checked =
        calendar::try_adjust_with_policy(start, conv, cal, CoveragePolicy::AllowProjected)
            .map_err(|error| format!("{error:?}"))?;
    let raw = roll::apply(start, conv, |date| calendar::is_business_day(date, cal));
    if raw != checked || raw != calendar::adjust(start, conv, cal) {
        return Err(format!(
            "Roll {name} {conv:?}: raw={raw:?} checked={checked:?}"
        ));
    }
    Ok(encoded(checked))
}

#[cfg(not(feature = "calendars"))]
fn named_roll(_start: Date, _conv: Roll, _name: &str) -> Result<f64, String> {
    Err("calendars feature is disabled".to_owned())
}

#[cfg(feature = "calendars")]
fn calendar_operation(fields: &[&str]) -> Result<f64, String> {
    use regit_daycount::calendar::{self, CoveragePolicy};
    let (name, operation) = fields[2].split_once(':').unwrap();
    if name.contains('+') || name == "empty" || name.starts_with("single-") {
        return composite_operation(fields, name, operation);
    }
    let start = date(fields[3]);
    let policy = CoveragePolicy::AllowProjected;
    if operation == "is_weekend" {
        return Ok(f64::from(u8::from(calendar::is_weekend(start))));
    }
    let cal = calendar(name)?;
    match operation {
        "next_business_day" => {
            let checked = calendar::try_next_business_day_with_policy(start, cal, policy)
                .map_err(|error| format!("{error:?}"))?;
            let raw = calendar::next_business_day(start, cal);
            if raw != checked {
                return Err(format!("next raw={raw:?} checked={checked:?}"));
            }
            Ok(encoded(checked))
        }
        "previous_business_day" => {
            let checked = calendar::try_previous_business_day_with_policy(start, cal, policy)
                .map_err(|error| format!("{error:?}"))?;
            let raw = calendar::previous_business_day(start, cal);
            if raw != checked {
                return Err(format!("previous raw={raw:?} checked={checked:?}"));
            }
            Ok(encoded(checked))
        }
        "add_business_days" => {
            let amount = fields[7].parse().unwrap();
            let checked = calendar::try_add_business_days_with_policy(start, amount, cal, policy)
                .map_err(|error| format!("{error:?}"))?;
            let raw = calendar::add_business_days(start, amount, cal);
            if raw != checked {
                return Err(format!("add raw={raw:?} checked={checked:?}"));
            }
            Ok(encoded(checked))
        }
        "business_days_between" => {
            let end = date(fields[4]);
            let checked = calendar::try_business_days_between_with_policy(start, end, cal, policy)
                .map_err(|error| format!("{error:?}"))?;
            let raw = calendar::business_days_between(start, end, cal);
            if raw != checked {
                return Err(format!("count raw={raw} checked={checked}"));
            }
            Ok(f64::from(checked))
        }
        "bus_252_for_calendar" => {
            let end = date(fields[4]);
            let checked = calendar::try_bus_252_for_calendar_with_policy(start, end, cal, policy)
                .map_err(|error| format!("{error:?}"))?;
            let raw = calendar::bus_252_for_calendar(start, end, cal);
            if (raw - checked).abs() > 1e-12 {
                return Err(format!("Bus252 raw={raw} checked={checked}"));
            }
            Ok(checked)
        }
        other => Err(format!("Unknown calendar operation {other}")),
    }
}

#[cfg(feature = "calendars")]
fn composite_operation(fields: &[&str], name: &str, operation: &str) -> Result<f64, String> {
    use regit_daycount::Calendar;
    use regit_daycount::calendar::{Composite, CoveragePolicy, JointBusiness};
    static EMPTY: &[Calendar] = &[];
    static SINGLE: &[Calendar] = &[Calendar::Target2];
    static EUR_USD: &[Calendar] = &[Calendar::Target2, Calendar::UnitedStatesNyse];
    static EUR_LU: &[Calendar] = &[Calendar::Target2, Calendar::Luxembourg];
    static USD_GBP: &[Calendar] = &[Calendar::UnitedStatesNyse, Calendar::UnitedKingdom];
    static ASIA: &[Calendar] = &[Calendar::Japan, Calendar::HongKong, Calendar::Singapore];
    let kinds = match name {
        "empty" => EMPTY,
        "single-target2" => SINGLE,
        "target2+usnyse" => EUR_USD,
        "target2+luxembourg" => EUR_LU,
        "usnyse+uk" => USD_GBP,
        "japan+hongkong+singapore" => ASIA,
        other => return Err(format!("Unknown composite calendar {other}")),
    };
    let (kind, operation) = operation.split_once('.').unwrap();
    let start = date(fields[3]);
    let policy = CoveragePolicy::AllowProjected;
    macro_rules! query {
        ($method:ident, $($args:expr),*) => {
            match kind {
                "Composite" => Composite::new(kinds).$method($($args),*),
                "JointBusiness" => JointBusiness::new(kinds).$method($($args),*),
                other => return Err(format!("Unknown combinator {other}")),
            }.map_err(|error| format!("{error:?}"))
        };
    }
    match operation {
        "is_business_day" => {
            let checked = query!(try_is_business_day_with_policy, start, policy)?;
            let raw = if kind == "Composite" {
                Composite::new(kinds).is_business_day(start)
            } else {
                JointBusiness::new(kinds).is_business_day(start)
            };
            if raw != checked {
                return Err(format!("{kind} business raw={raw} checked={checked}"));
            }
            let all_verified = kinds.iter().all(|&cal| {
                regit_daycount::calendar::coverage(cal)
                    .verified_years
                    .is_some_and(|(lo, hi)| (lo..=hi).contains(&start.year()))
            });
            if all_verified && query!(try_is_business_day, start)? != checked {
                return Err(format!("{kind} default checked business query disagrees"));
            }
            Ok(f64::from(u8::from(checked)))
        }
        "is_holiday" => {
            let checked = query!(try_is_holiday_with_policy, start, policy)?;
            let raw = if kind == "Composite" {
                Composite::new(kinds).is_holiday(start)
            } else {
                JointBusiness::new(kinds).is_holiday(start)
            };
            if raw != checked {
                return Err(format!("{kind} holiday raw={raw} checked={checked}"));
            }
            let all_verified = kinds.iter().all(|&cal| {
                regit_daycount::calendar::coverage(cal)
                    .verified_years
                    .is_some_and(|(lo, hi)| (lo..=hi).contains(&start.year()))
            });
            if all_verified && query!(try_is_holiday, start)? != checked {
                return Err(format!("{kind} default checked holiday query disagrees"));
            }
            Ok(f64::from(u8::from(checked)))
        }
        "adjust" => Ok(encoded(query!(
            try_adjust_with_policy,
            start,
            Roll::ModifiedFollowing,
            policy
        )?)),
        "add_business_days" => Ok(encoded(query!(
            try_add_business_days_with_policy,
            start,
            fields[7].parse().unwrap(),
            policy
        )?)),
        "business_days_between" => Ok(f64::from(query!(
            try_business_days_between_with_policy,
            start,
            date(fields[4]),
            policy
        )?)),
        "bus_252" => query!(try_bus_252_with_policy, start, date(fields[4]), policy),
        other => Err(format!("Unknown composite operation {other}")),
    }
}

#[cfg(not(feature = "calendars"))]
fn calendar_operation(_fields: &[&str]) -> Result<f64, String> {
    Err("calendars feature is disabled".to_owned())
}
