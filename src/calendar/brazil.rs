// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! B3 listed-market full-day trading closures for 2026.
//!
//! This is a B3 trading calendar, suitable when a Bus/252 contract names that
//! market. ANBIMA, banking and OTC settlement calendars are distinct and must
//! not be inferred from this variant. Ash Wednesday's partial session remains
//! a business day. The São Paulo 9 July local holiday is a normal B3 session.
//! No unverified past or projected future years are generated.
//!
//! Complete official schedule checked on 2026-10-23:
//! - B3 circular 054/2025-VNC, Annex 1, Table A and section 1.1:
//!   <https://www.b3.com.br/data/files/21/F3/6B/17/6FAEA9105B12E5A9AC094EA8/CL%20054-2025-VNC%20CALENDARIO%20DE%20FERIADOS%20EM%202026%20E%20FUNCIONAMENTO%20DA%20B3%20EM%2018022026%20QUARTAFEIRA%20DE%20CINZAS_EN.pdf>.
//! - Official summary, including partial-session and local-holiday distinctions:
//!   <https://www.b3.com.br/pt_br/noticias/calendario-de-negociacao-da-b3-confira-o-funcionamento-da-bolsa-em-2026.htm>.

use crate::Date;

/// Inclusive available year range. Checked queries reject dates outside it.
pub const COVERAGE: (i32, i32) = (2026, 2026);
/// Complete primary-source verified annual horizon (inclusive).
pub const VERIFIED_YEARS: (i32, i32) = (2026, 2026);
/// Date the primary annual schedule was checked.
pub const VERIFIED_ON: &str = "2026-10-23";

// B3 circular 054/2025-VNC, Annex 1, Table A. Each row closes listed trading.
pub(crate) const HOLIDAYS: &[(i32, u8, u8)] = &[
    (2026, 1, 1),
    (2026, 2, 16),
    (2026, 2, 17),
    (2026, 4, 3),
    (2026, 4, 21),
    (2026, 5, 1),
    (2026, 6, 4),
    (2026, 9, 7),
    (2026, 10, 12),
    (2026, 11, 2),
    (2026, 11, 20),
    (2026, 12, 24),
    (2026, 12, 25),
    (2026, 12, 31),
];

pub(crate) fn is_holiday(date: Date) -> bool {
    HOLIDAYS
        .binary_search(&(date.year(), date.month(), date.day()))
        .is_ok()
}

/// Query a B3 full-day trading closure using verified coverage.
///
/// # Errors
/// Returns a coverage error outside 2026, including with projection opt-in.
pub fn try_is_holiday(date: Date) -> Result<bool, super::CalendarError> {
    super::try_is_holiday(date, super::Calendar::BrazilB3)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calendar::{self, Calendar, CalendarError, CoveragePolicy};

    #[test]
    fn independence_day_changes_bus252_and_signed_direction() {
        let start = Date::ymd(2026, 9, 1).unwrap();
        let end = Date::ymd(2026, 9, 10).unwrap();
        assert_eq!(
            calendar::try_business_days_between(start, end, Calendar::BrazilB3),
            Ok(6)
        );
        assert_eq!(
            calendar::try_business_days_between(start, end, Calendar::Target2),
            Ok(7)
        );
        assert_eq!(
            calendar::try_bus_252_for_calendar(start, end, Calendar::BrazilB3),
            Ok(6.0 / 252.0)
        );
        assert_eq!(
            calendar::try_bus_252_for_calendar(end, start, Calendar::BrazilB3),
            Ok(-6.0 / 252.0)
        );
    }

    #[test]
    fn local_holiday_and_partial_session_remain_open() {
        for (month, day) in [(2, 18), (7, 9)] {
            assert_eq!(
                calendar::try_is_business_day(
                    Date::ymd(2026, month, day).unwrap(),
                    Calendar::BrazilB3
                ),
                Ok(true)
            );
        }
    }

    #[test]
    fn absent_years_are_never_projected_as_open() {
        for year in [2025, 2027] {
            let date = Date::ymd(year, 12, 25).unwrap();
            assert_eq!(
                calendar::try_is_holiday_with_policy(
                    date,
                    Calendar::BrazilB3,
                    CoveragePolicy::AllowProjected
                ),
                Err(CalendarError::OutOfCoverage {
                    calendar: Calendar::BrazilB3,
                    date
                })
            );
        }
    }
    // Complete primary schedules linked in the module documentation, checked
    // 2026-10-23. This compares every date, including open days and weekends.
    #[test]
    fn complete_verified_schedules_match_primary_sources() {
        let expected = &[
            // 2026
            (2026, 1, 1),
            (2026, 2, 16),
            (2026, 2, 17),
            (2026, 4, 3),
            (2026, 4, 21),
            (2026, 5, 1),
            (2026, 6, 4),
            (2026, 9, 7),
            (2026, 10, 12),
            (2026, 11, 2),
            (2026, 11, 20),
            (2026, 12, 24),
            (2026, 12, 25),
            (2026, 12, 31),
        ];
        super::super::assert_verified_schedule(
            super::super::Calendar::BrazilB3,
            2026,
            2026,
            expected,
        );
    }
}
