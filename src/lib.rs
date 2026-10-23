// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! Financial day counts, business-day calendars and date rolls.
//!
//! The crate is `no_std`, allocation-free and has no runtime dependencies.
//! [`Date`] values always lie in `1583..=9999`. Public construction and date
//! arithmetic validate that domain. Calendar operations return errors when
//! the selected market's data does not cover a date.
//!
//! # Day counts
//!
//! Use [`day_count::try_fraction`] for dispatch. Schedule-dependent conventions
//! require explicit [`day_count::DayCountContext`]; errors are returned as
//! [`day_count::DayCountError`], never NaN sentinels. Infallible per-convention
//! functions remain available for conventions that need only two valid dates.
//!
//! ```
//! use regit_daycount::{Date, DayCount, day_count::{self, DayCountContext}};
//!
//! let start = Date::ymd(2026, 1, 1).unwrap();
//! let end = Date::ymd(2026, 4, 1).unwrap();
//! let fraction = day_count::try_fraction(
//!     start, end, DayCount::Act360, DayCountContext::default(),
//! ).unwrap();
//! assert_eq!(fraction, 0.25);
//! assert!(Date::ymd(9999, 12, 31).unwrap().try_add_days(1).is_err());
//! ```
//!
//! # Calendar data
//!
//! The default `calendars` feature exposes `calendar`. Its `try_*` operations
//! require verified coverage by default; explicit projection opt-in accepts
//! available unverified data without extending its horizon. Inspect
//! `calendar::coverage` before choosing a market or historical period.
//!
//! # Public date invariant
//!
//! The unchecked constructor and raw arithmetic are internal implementation
//! details. They cannot be used to manufacture an invalid public date.
//!
//! ```compile_fail
//! use regit_daycount::Date;
//! let invalid = Date::ymd_unchecked(2026, 2, 30);
//! ```
//!
//! ```compile_fail
//! use regit_daycount::Date;
//! let escaped = Date::ymd(9999, 12, 31).unwrap().add_days(1);
//! ```
//!
//! Removed permissive entry points cannot be called by downstream code.
//!
//! ```compile_fail
//! use regit_daycount::{Date, DayCount, day_count};
//! let date = Date::ymd(2026, 1, 1).unwrap();
//! let value = day_count::fraction(date, date, DayCount::Act365L);
//! ```
//!
//! ```compile_fail
//! use regit_daycount::{Calendar, Date, calendar};
//! let date = Date::ymd(2045, 12, 25).unwrap();
//! let open = calendar::is_business_day(date, Calendar::UnitedKingdom);
//! ```

#![no_std]
#![forbid(unsafe_code)]

pub mod date;
pub mod day_count;
pub mod errors;
pub mod roll;

/// Checked market calendars and business-day operations.
#[cfg(feature = "calendars")]
pub mod calendar;

pub use date::{Date, Weekday};
pub use day_count::DayCount;
pub use errors::ValidationError;
pub use roll::{Roll, RollError};

#[cfg(feature = "calendars")]
pub use calendar::Calendar;
