// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! Typed error enum for date construction and convention dispatch.
//!
//! All failure paths return a typed `Result` — no `panic!()`, no `unwrap()`,
//! no string errors. Each variant carries enough context for the caller to
//! report precisely what was wrong and where.
//!
//! A single enum covers the two failure domains the crate can produce:
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
/// The crate has a single error type because every failure mode reduces to
/// one of two shapes: an input that does not name a real date, or an input
/// that is well-formed in isolation but outside the supported range.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{debug, display};

    #[test]
    fn validation_error_display_invalid_date() {
        let err = ValidationError::InvalidDate {
            rule: "month-out-of-range",
        };
        assert_eq!(display(err).as_str(), "invalid date: month-out-of-range");
    }

    #[test]
    fn validation_error_display_out_of_range() {
        let err = ValidationError::OutOfRange {
            what: "year < 1583",
        };
        assert_eq!(display(err).as_str(), "value out of range: year < 1583");
    }

    #[test]
    fn validation_error_display_has_no_trailing_period() {
        for err in [
            ValidationError::InvalidDate { rule: "r" },
            ValidationError::OutOfRange { what: "w" },
        ] {
            assert!(!display(err).as_str().ends_with('.'));
        }
    }

    #[test]
    fn validation_error_is_error_trait() {
        let err: &dyn core::error::Error = &ValidationError::InvalidDate { rule: "r" };
        assert!(err.source().is_none());
    }

    #[test]
    fn validation_error_copy_eq() {
        let err = ValidationError::OutOfRange { what: "year" };
        let copy = err;
        assert_eq!(err, copy);
    }

    #[test]
    fn errors_debug() {
        assert!(
            debug(ValidationError::InvalidDate { rule: "r" })
                .as_str()
                .contains("InvalidDate")
        );
        assert!(
            debug(ValidationError::OutOfRange { what: "w" })
                .as_str()
                .contains("OutOfRange")
        );
    }
}
