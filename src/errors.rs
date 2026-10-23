// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! Typed error enum for date construction and convention dispatch.
//!
//! Date construction reports structural and range failures as typed errors.
//! Checked day-count and calendar operations have their own error types in
//! [`crate::day_count`] and, with the feature enabled, `crate::calendar`.
//!
//! This enum covers two date-validation failures:
//!
//! - [`ValidationError::InvalidDate`] — a `(year, month, day)` triple does
//!   not name a real Gregorian date (e.g. 31 February, a 0 month).
//! - [`ValidationError::OutOfRange`] — a value is well-formed in isolation
//!   but falls outside the range a convention or calendar accepts.
//!
//! It implements [`core::fmt::Display`] and [`core::error::Error`], so it
//! composes with `?` and with `dyn Error` even under `#![no_std]`.
//!
//! # References
//!
//! - ISO 8601 (Gregorian calendar date format) — the date grammar these
//!   errors report against.

use core::fmt;

// ─── Validation errors ───────────────────────────────────────────────────────

/// Error returned when an input cannot be accepted as a valid date or when a
/// value falls outside the range a convention accepts.
///
/// Distinguishes a structurally invalid date from a value outside the
/// supported date range. Convention and calendar checks use their domain
/// error types.
///
/// # Examples
///
/// ```
/// use regit_daycount::errors::ValidationError;
///
/// let err = ValidationError::InvalidDate { rule: "day-out-of-range" };
/// assert_eq!(err, ValidationError::InvalidDate { rule: "day-out-of-range" });
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ValidationError {
    /// The `(year, month, day)` triple does not name a real Gregorian date;
    /// `rule` names the violated structural rule.
    InvalidDate {
        /// A short, human-readable description of the violated rule.
        rule: &'static str,
    },
    /// A value is well-formed in isolation but falls outside the range the
    /// caller's convention or calendar accepts; `what` names the value.
    OutOfRange {
        /// A short, human-readable description of the out-of-range value.
        what: &'static str,
    },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDate { rule } => write!(f, "invalid date: {rule}"),
            Self::OutOfRange { what } => write!(f, "value out of range: {what}"),
        }
    }
}

impl core::error::Error for ValidationError {}
