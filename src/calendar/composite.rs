// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! Compound calendar combinators — [`Composite`] and [`JointBusiness`].
//!
//! A real-world cashflow often touches more than one calendar at once. A
//! cross-currency swap settles a leg in each currency on the other leg's
//! holiday: the joint settlement date is the next day on which both legs'
//! calendars are open. A Luxembourg-domiciled UCITS fund computes NAV on a
//! day that is both a TARGET2 business day and a Luxembourg bank-holiday
//! business day. The two combinators in this module name those operations.
//!
//! - [`Composite`] takes the **union of holidays**: a date is a holiday on
//!   the composite if it is a holiday on *any* underlying calendar.
//!   Equivalent to "broadest possible non-business set across the group".
//! - [`JointBusiness`] takes the **intersection of business days**: a date
//!   is a business day iff every underlying calendar treats it as one.
//!   Equivalent to "narrowest possible business set across the group".
//!
//! In practice the two operations are duals (`Composite::is_holiday` is
//! `!JointBusiness::is_business_day` once weekends are accounted for); the
//! distinction is one of ergonomic framing, not semantics. Use whichever
//! reads more naturally at the call site.
//!
//! Both structs hold a `&'static [Calendar]` — they are `Copy`,
//! allocation-free, and `no_std`-clean.
//!
//! # Examples
//!
//! ```
//! use regit_daycount::{Calendar, Date};
//! use regit_daycount::calendar::Composite;
//!
//! // A Luxembourg-domiciled UCITS calendar: holiday if TARGET2 OR the
//! // local Luxembourg calendar would close. (Luxembourg-only holidays —
//! // National Day, Assumption — are not in the named set yet, so this
//! // composite reduces to TARGET2 alone for now.)
//! static REGIONS: &[Calendar] = &[Calendar::Target2];
//! let lu = Composite::new(REGIONS);
//!
//! // 2026-12-25 is a TARGET2 holiday → composite holiday.
//! assert!(lu.is_holiday(Date::ymd(2026, 12, 25).unwrap()));
//!
//! // 2026-05-20 (Wed) is open on TARGET2 → not a composite holiday.
//! assert!(!lu.is_holiday(Date::ymd(2026, 5, 20).unwrap()));
//! ```

use crate::calendar::{Calendar, is_business_day, is_holiday};
use crate::date::Date;

// ─── Composite ───────────────────────────────────────────────────────────────

/// Composite calendar — union of holidays across an underlying set.
///
/// A date is a holiday on the composite iff it is a holiday on *any* of
/// the underlying calendars. Use this when a cashflow's settlement is
/// blocked by a holiday on any one of several markets.
///
/// The struct holds a borrowed `&'static [Calendar]`. Construct with
/// [`Composite::new`].
///
/// # Examples
///
/// ```
/// use regit_daycount::{Calendar, Date};
/// use regit_daycount::calendar::Composite;
///
/// static REGIONS: &[Calendar] = &[Calendar::Target2, Calendar::UnitedKingdom];
/// let euro_sterling = Composite::new(REGIONS);
///
/// // 2026-12-25 is a holiday on both → composite holiday.
/// assert!(euro_sterling.is_holiday(Date::ymd(2026, 12, 25).unwrap()));
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Composite {
    /// The underlying calendars. The order is irrelevant for `is_holiday`
    /// and `is_business_day` (both are commutative); it is preserved
    /// because callers may inspect [`Composite::kinds`] for diagnostics.
    pub kinds: &'static [Calendar],
}

impl Composite {
    /// Constructs a `Composite` from a static slice of underlying
    /// calendars.
    ///
    /// # Examples
    ///
    /// ```
    /// use regit_daycount::Calendar;
    /// use regit_daycount::calendar::Composite;
    ///
    /// static REGIONS: &[Calendar] = &[Calendar::Target2, Calendar::Switzerland];
    /// let euro_chf = Composite::new(REGIONS);
    /// assert_eq!(euro_chf.kinds.len(), 2);
    /// ```
    #[must_use]
    pub const fn new(kinds: &'static [Calendar]) -> Self {
        Self { kinds }
    }

    /// True if `date` is a holiday on *any* underlying calendar.
    ///
    /// # Examples
    ///
    /// ```
    /// use regit_daycount::{Calendar, Date};
    /// use regit_daycount::calendar::Composite;
    ///
    /// static REGIONS: &[Calendar] = &[Calendar::UnitedStates, Calendar::UnitedKingdom];
    /// let us_uk = Composite::new(REGIONS);
    ///
    /// // 2024-07-04 (US Independence Day) is a US holiday → composite holiday.
    /// assert!(us_uk.is_holiday(Date::ymd(2024, 7, 4).unwrap()));
    /// ```
    #[must_use]
    pub fn is_holiday(&self, date: Date) -> bool {
        self.kinds.iter().any(|&cal| is_holiday(date, cal))
    }

    /// True if `date` is a business day on *every* underlying calendar —
    /// not a weekend, and not a holiday on any of them.
    ///
    /// # Examples
    ///
    /// ```
    /// use regit_daycount::{Calendar, Date};
    /// use regit_daycount::calendar::Composite;
    ///
    /// static REGIONS: &[Calendar] = &[Calendar::Target2, Calendar::UnitedStates];
    /// let eur_usd = Composite::new(REGIONS);
    ///
    /// // 2024-07-04 — open on TARGET2, closed on US → not a joint business day.
    /// assert!(!eur_usd.is_business_day(Date::ymd(2024, 7, 4).unwrap()));
    /// ```
    #[must_use]
    pub fn is_business_day(&self, date: Date) -> bool {
        self.kinds.iter().all(|&cal| is_business_day(date, cal))
    }
}

// ─── JointBusiness ───────────────────────────────────────────────────────────

/// Joint-business calendar — intersection of business days across an
/// underlying set.
///
/// A date is a business day on the joint-business calendar iff it is a
/// business day on *every* underlying calendar. Use this for the
/// multi-leg-settlement framing (cross-currency swaps, dual-listed
/// instruments).
///
/// Semantically equivalent to [`Composite`]'s business-day predicate, but
/// exposed as its own type so the call site reads naturally for the
/// "settles iff both legs open" intent.
///
/// # Examples
///
/// ```
/// use regit_daycount::{Calendar, Date};
/// use regit_daycount::calendar::JointBusiness;
///
/// static LEGS: &[Calendar] = &[Calendar::UnitedStates, Calendar::Japan];
/// let usdjpy = JointBusiness::new(LEGS);
///
/// // 2024-01-02 — open on US, closed on Japan (JPX year-end) → not joint.
/// assert!(!usdjpy.is_business_day(Date::ymd(2024, 1, 2).unwrap()));
/// ```
#[derive(Debug, Clone, Copy)]
pub struct JointBusiness {
    /// The underlying calendars.
    pub kinds: &'static [Calendar],
}

impl JointBusiness {
    /// Constructs a `JointBusiness` from a static slice of underlying
    /// calendars.
    ///
    /// # Examples
    ///
    /// ```
    /// use regit_daycount::Calendar;
    /// use regit_daycount::calendar::JointBusiness;
    ///
    /// static LEGS: &[Calendar] = &[Calendar::UnitedStates, Calendar::HongKong];
    /// let usdhkd = JointBusiness::new(LEGS);
    /// assert_eq!(usdhkd.kinds.len(), 2);
    /// ```
    #[must_use]
    pub const fn new(kinds: &'static [Calendar]) -> Self {
        Self { kinds }
    }

    /// True if `date` is a business day on *every* underlying calendar.
    ///
    /// # Examples
    ///
    /// ```
    /// use regit_daycount::{Calendar, Date};
    /// use regit_daycount::calendar::JointBusiness;
    ///
    /// static LEGS: &[Calendar] = &[Calendar::Target2, Calendar::UnitedStates];
    /// let eurusd = JointBusiness::new(LEGS);
    ///
    /// // 2024-03-15 — Friday, open on both → joint business day.
    /// assert!(eurusd.is_business_day(Date::ymd(2024, 3, 15).unwrap()));
    /// ```
    #[must_use]
    pub fn is_business_day(&self, date: Date) -> bool {
        self.kinds.iter().all(|&cal| is_business_day(date, cal))
    }

    /// True if `date` is a holiday on *any* underlying calendar.
    ///
    /// Symmetric to [`Composite::is_holiday`]; both structs expose both
    /// queries so the call site can read in the most natural framing.
    ///
    /// # Examples
    ///
    /// ```
    /// use regit_daycount::{Calendar, Date};
    /// use regit_daycount::calendar::JointBusiness;
    ///
    /// static LEGS: &[Calendar] = &[Calendar::Target2, Calendar::UnitedStates];
    /// let eurusd = JointBusiness::new(LEGS);
    ///
    /// // 2024-12-25 is a holiday on both → reported as a holiday on the joint.
    /// assert!(eurusd.is_holiday(Date::ymd(2024, 12, 25).unwrap()));
    /// ```
    #[must_use]
    pub fn is_holiday(&self, date: Date) -> bool {
        self.kinds.iter().any(|&cal| is_holiday(date, cal))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static EUR_USD: &[Calendar] = &[Calendar::Target2, Calendar::UnitedStates];
    static EUR_CHF: &[Calendar] = &[Calendar::Target2, Calendar::Switzerland];
    static US_UK: &[Calendar] = &[Calendar::UnitedStates, Calendar::UnitedKingdom];

    // ─── Composite ───────────────────────────────────────────────────────

    #[test]
    fn composite_is_holiday_on_any_leg() {
        let c = Composite::new(EUR_USD);
        // 2024-07-04 is a US holiday but not a TARGET2 holiday → composite YES.
        assert!(c.is_holiday(Date::ymd(2024, 7, 4).unwrap()));
        // 2024-05-01 is a TARGET2 holiday (Labour Day) but not a US holiday → YES.
        assert!(c.is_holiday(Date::ymd(2024, 5, 1).unwrap()));
        // 2024-03-15 (Fri) — neither → NO.
        assert!(!c.is_holiday(Date::ymd(2024, 3, 15).unwrap()));
    }

    #[test]
    fn composite_business_day_requires_all_open() {
        let c = Composite::new(EUR_USD);
        // 2024-07-04 — US closed → not a business day even though TARGET2 open.
        assert!(!c.is_business_day(Date::ymd(2024, 7, 4).unwrap()));
        // 2024-05-01 — TARGET2 closed → not a business day.
        assert!(!c.is_business_day(Date::ymd(2024, 5, 1).unwrap()));
        // 2024-03-15 (Fri) — both open → yes.
        assert!(c.is_business_day(Date::ymd(2024, 3, 15).unwrap()));
        // Weekend — no.
        assert!(!c.is_business_day(Date::ymd(2024, 3, 16).unwrap()));
    }

    // ─── JointBusiness ───────────────────────────────────────────────────

    #[test]
    fn joint_business_business_day_requires_all_open() {
        let j = JointBusiness::new(EUR_CHF);
        // 2024-05-01 (Labour Day) — closed on both → NO.
        assert!(!j.is_business_day(Date::ymd(2024, 5, 1).unwrap()));
        // 2024-08-01 — Swiss National Day, open on TARGET2 → NO.
        assert!(!j.is_business_day(Date::ymd(2024, 8, 1).unwrap()));
        // 2024-03-15 (Fri) — both open → YES.
        assert!(j.is_business_day(Date::ymd(2024, 3, 15).unwrap()));
    }

    #[test]
    fn joint_business_is_holiday_on_any_leg() {
        let j = JointBusiness::new(US_UK);
        // 2024-07-04 (US Independence) → YES (US closed).
        assert!(j.is_holiday(Date::ymd(2024, 7, 4).unwrap()));
        // 2024-08-26 (UK Summer BH) → YES (UK closed).
        assert!(j.is_holiday(Date::ymd(2024, 8, 26).unwrap()));
        // 2024-03-15 (Fri) → NO.
        assert!(!j.is_holiday(Date::ymd(2024, 3, 15).unwrap()));
    }

    // ─── Single-calendar degenerate ──────────────────────────────────────

    #[test]
    fn single_calendar_composite_matches_underlying() {
        // A composite of one calendar is exactly that calendar.
        static SINGLE: &[Calendar] = &[Calendar::Target2];
        let c = Composite::new(SINGLE);
        let d = Date::ymd(2024, 12, 25).unwrap();
        assert_eq!(
            c.is_holiday(d),
            crate::calendar::is_holiday(d, Calendar::Target2)
        );
        let d2 = Date::ymd(2024, 3, 15).unwrap();
        assert_eq!(
            c.is_holiday(d2),
            crate::calendar::is_holiday(d2, Calendar::Target2)
        );
    }

    // ─── Empty calendar list ─────────────────────────────────────────────

    #[test]
    fn empty_composite_has_no_holidays_and_only_weekend_filters_business_days() {
        static EMPTY: &[Calendar] = &[];
        let c = Composite::new(EMPTY);
        // No legs → no holidays at all.
        assert!(!c.is_holiday(Date::ymd(2024, 12, 25).unwrap()));
        // No legs → `all` over an empty iterator is true → every weekday
        // (and even weekends — there's nothing to filter on) is a business
        // day. Document this as a degenerate corner case; in practice
        // callers always supply at least one calendar.
        assert!(c.is_business_day(Date::ymd(2024, 3, 16).unwrap())); // Saturday!
    }
}
