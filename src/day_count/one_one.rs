// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! Constant-one shortcut: every interval has fraction `1.0`.
//!
//! This is an explicitly defined arithmetic shortcut, rather than an actual
//! day-count calculation. It can be used when a supplied return already
//! includes time weighting and the caller's cashflow formula needs an identity
//! multiplier. The instrument's terms determine whether that shortcut applies.
//!
//! Equal and reversed endpoints also return `1.0`. This differs from a signed
//! 1/1 counter that returns `-1.0` for reversed endpoints. The result is not
//! additive over subintervals.

use crate::date::Date;

/// Returns `1.0` regardless of the endpoints or their ordering.
///
/// # Examples
///
/// ```
/// use regit_daycount::{Date, day_count::one_one};
/// let start = Date::ymd(2026, 1, 1).unwrap();
/// let end = Date::ymd(2026, 4, 1).unwrap();
/// assert_eq!(one_one::fraction(start, end), 1.0);
/// assert_eq!(one_one::fraction(end, start), 1.0);
/// assert_eq!(one_one::fraction(start, start), 1.0);
/// ```
#[must_use]
pub fn fraction(_start: Date, _end: Date) -> f64 {
    1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_multiplier_includes_equal_reverse_and_boundary_dates() {
        let first = Date::ymd(1583, 1, 1).unwrap();
        let last = Date::ymd(9999, 12, 31).unwrap();
        let start = Date::ymd(2026, 1, 1).unwrap();
        let end = Date::ymd(2026, 4, 1).unwrap();
        for (start, end) in [
            (start, end),
            (end, start),
            (start, start),
            (first, last),
            (last, first),
        ] {
            assert_eq!(fraction(start, end).to_bits(), 1.0_f64.to_bits());
        }
    }
}
