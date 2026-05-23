// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! Day-count fractions and business-day calendars in pure Rust —
//! `#![no_std]`, no `alloc`, zero dependencies.
//!
//! `regit-daycount` computes the year fraction between two dates under every
//! common day-count convention (Act/360, Act/365F, `ActAct` ISDA, `ActAct`
//! ICMA, 30/360 `BondBasis`, 30E/360, 30E/360 ISDA, Act/365L, NL/365, Bus/252,
//! 1/1), adjusts a date under every common date-roll convention (Unadjusted,
//! Following, `ModifiedFollowing`, Preceding, `ModifiedPreceding`, Nearest,
//! `EndOfMonth`), and classifies dates against the major holiday calendars
//! (TARGET2, US, UK, JP, CH, HK, SG, plus composite and joint-business
//! combinations).
//!
//! A coupon is paid on a date, an accrued-interest amount is settled to a
//! date, and a swap leg is discounted from a date — every one of those
//! operations is parameterised by a day-count convention and a holiday
//! calendar, and a wrong choice silently produces a wrong cashflow. This
//! crate's first duty is therefore to compute every convention exactly as
//! its governing standard prescribes — never to guess, never to round
//! silently, never to interpolate where the standard splits.
//!
//! The crate is `#![no_std]` and allocation-free: dates are 32-bit `Copy`
//! triples, every algorithm is hand-rolled integer arithmetic, holiday
//! tables are static sorted arrays, and there is no runtime dependency. The
//! same audited logic runs in a backend service, a WASM bundle, or on an
//! embedded device.
//!
//! # Architecture
//!
//! ```text
//! errors       typed errors — ValidationError
//! date         Gregorian Date primitive — leap years, weekday, arithmetic
//! roll         date-roll conventions — Unadjusted, Following, ...
//!
//! day_count    catalogue of day-count fractions and their dispatcher
//!              (Act/360, Act/365F, ActAct ISDA, ActAct ICMA, 30/360, ...)
//!
//! calendar     catalogue of holiday calendars and their dispatcher
//!              (TARGET2 fixed rule; US / UK / JP / CH / HK / SG snapshots)
//! ```
//!
//! Every rule is traced to a citable standard — ISDA 2006 Definitions §4.16,
//! ICMA Rule 251, ISO 8601, the ECB TARGET2 closing-days regulation, and the
//! published calendars of NYSE, the Bank of England, JPX, SIX, HKEX, and
//! SGX — in the doc comment of the module that implements it.
//!
//! Part of [Regit OS](https://www.regit.io) — the operating system for
//! investment products. From Luxembourg.

#![no_std]
#![forbid(unsafe_code)]

#[cfg(test)]
mod test_support;

pub mod date;
pub mod day_count;
pub mod errors;
pub mod roll;

/// Holiday calendars and their dispatcher. Behind the default `calendars`
/// feature so a minimal build (`cargo build --no-default-features`) drops
/// the embedded snapshot tables and exposes only the date primitives,
/// day-count fractions, and date-roll conventions.
#[cfg(feature = "calendars")]
pub mod calendar;

pub use date::{Date, Weekday};
pub use day_count::DayCount;
pub use errors::ValidationError;
pub use roll::Roll;

#[cfg(feature = "calendars")]
pub use calendar::Calendar;
