<!-- Copyright 2026 Regit.io — Nicolas Koenig -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# SPEC.md — regit-daycount

> The reference specification for every day-count fraction, date-roll
> convention, and holiday calendar this crate computes. For each it states
> the structure (inputs, outputs, denominator), the algorithm in plain-text
> procedure form, one worked example computed by hand against an ISDA / ICMA
> reference case, and the governing standard cited by number.
>
> This document is the public, citable distillation of the crate's internal
> verified algorithm reference. The crate is the executable form of this
> specification — every rule stated here is enforced by the `src/` module
> named in its heading, and a computation never trusts a supplied convention:
> it dispatches on the typed enum and runs the algorithm of the governing
> standard.

---

## Table of contents

1. [Conventions](#conventions)
2. [Date primitives — `src/date.rs`](#date-primitives--srcdaters)
3. [Act/360 — ISDA 2006 §4.16(e) — `src/day_count/act_360.rs`](#act360--isda-2006-416e--srcday_countact_360rs)
4. [Act/365F — ISDA 2006 §4.16(d) — `src/day_count/act_365f.rs`](#act365f--isda-2006-416d--srcday_countact_365frs)
5. [ActAct ISDA — ISDA 2006 §4.16(b) — `src/day_count/act_act_isda.rs`](#actact-isda--isda-2006-416b--srcday_countact_act_isdars)
6. [ActAct ICMA — ICMA Rule 251 — `src/day_count/act_act_icma.rs`](#actact-icma--icma-rule-251--srcday_countact_act_icmars)
7. [30/360 BondBasis — ISDA 2006 §4.16(f) — `src/day_count/thirty_360_bond_basis.rs`](#30360-bondbasis--isda-2006-416f--srcday_countthirty_360_bond_basisrs)
8. [30E/360 — ISDA 2006 §4.16(g) — `src/day_count/thirty_e_360.rs`](#30e360--isda-2006-416g--srcday_countthirty_e_360rs)
9. [30E/360 ISDA — ISDA 2006 §4.16(h) — `src/day_count/thirty_e_360_isda.rs`](#30e360-isda--isda-2006-416h--srcday_countthirty_e_360_isdars)
10. [Act/365L — ICMA — `src/day_count/act_365l.rs`](#act365l--icma--srcday_countact_365lrs)
11. [NL/365 — conventional — `src/day_count/nl_365.rs`](#nl365--conventional--srcday_countnl_365rs)
12. [Bus/252 — Brazilian convention — `src/day_count/bus_252.rs`](#bus252--brazilian-convention--srcday_countbus_252rs)
13. [1/1 — OIS shortcut — `src/day_count/one_one.rs`](#11--ois-shortcut--srcday_countone_oners)
14. [Unadjusted date roll — `src/roll.rs`](#unadjusted-date-roll--srcrollrs)
15. [Following date roll — `src/roll.rs`](#following-date-roll--srcrollrs)
16. [ModifiedFollowing date roll — `src/roll.rs`](#modifiedfollowing-date-roll--srcrollrs)
17. [Preceding date roll — `src/roll.rs`](#preceding-date-roll--srcrollrs)
18. [ModifiedPreceding date roll — `src/roll.rs`](#modifiedpreceding-date-roll--srcrollrs)
19. [Nearest date roll — `src/roll.rs`](#nearest-date-roll--srcrollrs)
20. [EndOfMonth date roll — `src/roll.rs`](#endofmonth-date-roll--srcrollrs)
21. [TARGET2 calendar — ECB fixed rule — `src/calendar/target2.rs`](#target2-calendar--ecb-fixed-rule--srccalendartarget2rs)
22. [Luxembourg calendar — Loi du 21 juillet 1928 — `src/calendar/luxembourg.rs`](#luxembourg-calendar--loi-du-21-juillet-1928--srccalendarluxembourgrs)
23. [United States calendar — NYSE / Federal Reserve — `src/calendar/united_states.rs`](#united-states-calendar--nyse--federal-reserve--srccalendarunited_statesrs)
24. [United Kingdom calendar — Bank of England — `src/calendar/united_kingdom.rs`](#united-kingdom-calendar--bank-of-england--srccalendarunited_kingdomrs)
25. [Japan calendar — JPX — `src/calendar/japan.rs`](#japan-calendar--jpx--srccalendarjapanrs)
26. [Switzerland calendar — SIX — `src/calendar/switzerland.rs`](#switzerland-calendar--six--srccalendarswitzerlandrs)
27. [Hong Kong calendar — HKEX — `src/calendar/hong_kong.rs`](#hong-kong-calendar--hkex--srccalendarhong_kongrs)
28. [Singapore calendar — SGX — `src/calendar/singapore.rs`](#singapore-calendar--sgx--srccalendarsingaporers)
29. [Calendar dispatchers and business-day primitives — `src/calendar/mod.rs`](#calendar-dispatchers-and-business-day-primitives--srccalendarmodrs)
30. [Composite / JointBusiness combinators — `src/calendar/composite.rs`](#composite--jointbusiness-combinators--srccalendarcompositers)
31. [Standards index](#standards-index)

---

## Conventions

The conventions below hold for every fraction, roll, and calendar in this
document.

- **Date primitive.** The single date type is `Date`, a proleptic Gregorian
  `(year, month, day)` triple. The accepted year range is
  `1583..=9999` — the Gregorian calendar took effect in October 1582, and
  from 1583 onward every day-count and holiday-calendar rule this crate
  implements is defined uniformly. Construction is validated by
  `Date::ymd`; out-of-range or impossible triples (29 February in a
  non-leap year, day 31 of April) are rejected with the typed
  `ValidationError`. There is no second date type — every fraction, roll
  and calendar takes and returns `Date` values.
- **Half-open interval `[start, end)`.** Every day count in this crate is
  end-exclusive, start-inclusive. `days_between(2026-01-01, 2026-01-02)`
  returns `1`, not `2`. The same convention applies to leap-day inclusion
  (Act/365L, NL/365), to business-day counts (Bus/252,
  `business_days_between`), and to the comparison `start <= ymd(y, 2, 29) <
  end` that decides whether a given 29 February belongs to a period. The
  half-open form is the single convention the standards (ISDA §4.16, ICMA
  Rule 251) themselves use; mixing in a closed-interval count anywhere is a
  bug.
- **Inverted intervals.** `start > end` is not rejected — some callers
  compute reverse-period accruals. Every fraction returns the natural
  negation of the forward result: `fraction(end, start) == -fraction(start,
  end)`, exactly. The two exceptions are `1/1` (which returns `1.0` for
  every input, including the inverted one, because the convention is
  definitional) and the dispatcher arms that return `f64::NAN` (which
  propagate NaN by IEEE-754).
- **Weekday rule.** Saturday and Sunday are the universal weekend; every
  calendar in this crate uses the Western Sat/Sun convention. `is_weekend`
  reports this independently of any calendar; `is_business_day(date, cal)`
  is the documented composition `!is_weekend(date) && !is_holiday(date,
  cal)`. Markets with a Fri/Sat weekend (Saudi Arabia, the UAE pre-2022)
  are out of scope.
- **Holiday-on-weekend reporting.** A fixed-date holiday that falls on a
  Saturday or Sunday is still reported by `is_holiday` for its rule date.
  The schedule does not "shift" it; the weekend test composes with the
  holiday test through `is_business_day`. For the dated-snapshot calendars
  (US, UK, JP, CH, HK, SG) the *observed* shift is encoded into the
  snapshot itself (so e.g. the snapshot lists 2026-07-03 — the observed
  Independence Day — alongside 2026-07-04 only when the published list
  records both).
- **Holiday detection is calendar-only.** `is_holiday(date, cal)` reports
  true *only* for calendar holidays; weekends are out of scope here.
  Combine with `is_weekend` (or use the `is_business_day` wrapper) for the
  full "market open?" predicate.
- **Out-of-coverage calendar lookups.** The six snapshot calendars (US, UK,
  JP, CH, HK, SG) embed a dated table whose coverage range is `(2020,
  2040)` inclusive, with `SNAPSHOT_DATE = "2026-05-23"` recorded in the
  source. A lookup for a date outside the coverage window returns `false`
  — the conservative answer — never a panic and never an extrapolation.
  Auditors should pin the snapshot date when reporting calendar validity,
  because the published schedules are revised continually. TARGET2 is the
  exception: it is fully rule-based and exact for every year in the
  supported range.
- **Verification, not trust.** The `DayCount` enum dispatches a typed
  variant to the algorithm of the governing standard. The caller cannot
  pass a free-form rule and have it executed — every computation runs the
  cited procedure verbatim. A wrong cashflow caused by a silently-misapplied
  convention is the worst possible failure, and this crate is the
  executable form of the specification so that the rule and the code
  cannot drift.
- **NaN as the cannot-compute sentinel.** Two `DayCount` variants need
  information the two-argument dispatcher cannot supply on its own:
  `ActActIcma` needs a reference period and frequency, and `Bus252` needs
  a business-day predicate. For both, `day_count::fraction(start, end,
  basis)` returns `f64::NAN`. NaN is chosen deliberately: it propagates
  loudly through arithmetic — any downstream multiplication produces NaN
  in turn — where a silent `0.0` would absorb the misuse and produce a
  wrong cashflow with no warning. Callers route through
  `year_fraction_with_freq` (for ICMA) or the per-module
  `bus_252::fraction` (for Bus/252) with the missing input.
  `ThirtyE360Isda` similarly takes an extra `end_is_maturity` flag the
  dispatcher cannot infer; the dispatcher defaults it to `false` and the
  full signature lives at `thirty_e_360_isda::fraction`.
- **Tolerance.** The crate's working numeric slack is `1e-12` — loose
  enough that a benign last-bit rounding never causes a spurious test
  failure, tight enough that any drift large enough to mis-state a cashflow
  at the cent level (≥ 1e-9 on a unit notional) is caught. Worked examples
  in this document quote the value to the precision the implementation
  returns, which is the result of one or two `f64` divisions of exact
  rationals — the worst-case relative error is a few ulps of the printed
  number. Cross-oracle vectors transcribed verbatim from QuantLib's
  `daycounters.cpp` are compared at a slightly wider `1e-10` because
  QuantLib prints the expected value with twelve decimal digits and the
  trailing digits are a round-and-truncate of QuantLib's own `f64`.
- **Citation form.** Every section cites the governing standard by number
  and edition (e.g. "ISDA 2006 Definitions §4.16(e)" or "ICMA Rule 251");
  every worked example transcribed from QuantLib also cites the source
  line in `test-suite/daycounters.cpp` at release tag `v1.34`. The Standards
  index in §29 maps each cited standard to the section that references it.
- **No allocation, no `std`.** The crate is `#![no_std]` and
  `#![forbid(unsafe_code)]`. `Date` is a `Copy` triple, every algorithm is
  hand-rolled integer arithmetic, holiday tables are static sorted arrays,
  and there is no runtime dependency. The same audited code runs in a
  backend service, a WASM bundle, or on an embedded device.

---

## Date primitives — `src/date.rs`

The `Date` type is the only date primitive used in this crate. It is a
`Copy`, three-field struct — a signed 32-bit year and two `u8` fields —
holding a proleptic Gregorian `(year, month, day)` triple. Five operations
on `Date` are load-bearing for the rest of the specification and are
specified individually below.

### Structure

```text
Length: 3 fields.

  ┌──────┬───────┬─────┐
  │ year │ month │ day │
  └──┬───┴───┬───┴──┬──┘
     │       │       └ 1..=days_in_month(year, month)
     │       └───────── 1..=12
     └───────────────── 1583..=9999  (Date::MIN_YEAR..=Date::MAX_YEAR)

Validation:  Date::ymd(year, month, day) -> Result<Date, ValidationError>
Trust-me:    Date::ymd_unchecked(year, month, day) -> Date  (const)
Accessors:   year() / month() / day()   (all const)
```

`Date::ymd` validates in three steps: year in `MIN_YEAR..=MAX_YEAR`, month
in `1..=12`, day in `1..=days_in_month(year, month)`. The unchecked form
exists for `const`-context construction and for reconstructing a `Date`
from fields validated earlier; any untrusted input must go through
`Date::ymd`.

`PartialOrd` and `Ord` follow the natural chronological order — equivalent
to lexicographic order on `(year, month, day)`, which is the same thing for
Gregorian dates.

### Epoch conversion — Hinnant

Every arithmetic operation on `Date` (`add_days`, `days_between`,
`day_of_week`) is grounded in **Howard Hinnant's `days_from_civil` /
`civil_from_days`** algorithm — a public-domain, branchless, integer-only
conversion between a `(year, month, day)` triple and a signed count of
days from the civil epoch 1970-01-01 — transcribed verbatim in `i64`. The
original derivation is at
<http://howardhinnant.github.io/date_algorithms.html>. The two conversion
helpers are `const fn` private to the module; every public arithmetic
operation factors through them.

### `is_leap_year(year)`

```text
is_leap_year(y) = (y % 4 == 0 AND y % 100 != 0) OR (y % 400 == 0)
```

So 1900 is not a leap year (divisible by 100 but not 400), 2000 is
(divisible by 400), and 2100 is not.

**Worked example.** `is_leap_year(2024) = true` (`2024 % 4 == 0` and
`2024 % 100 == 24 ≠ 0`). `is_leap_year(1900) = false` (`1900 % 4 == 0`
and `1900 % 100 == 0` and `1900 % 400 == 300 ≠ 0`). `is_leap_year(2025) =
false` (`2025 % 4 == 1`).

### `day_of_week`

```text
z   = to_civil_days(year, month, day)   // signed count from 1970-01-01
idx = (z + 3).rem_euclid(7)             // 1970-01-01 was a Thursday
return (Mon, Tue, Wed, Thu, Fri, Sat, Sun)[idx]
```

The `+ 3` offsets Thursday (the civil epoch's weekday) to index 3 in the
Monday-first variant table; `rem_euclid` lifts the signed civil-day count
into the non-negative weekday index without a manual shift.

**Worked example.** `Date::ymd(2026, 5, 23).unwrap().day_of_week() ==
Weekday::Sat`. Cross-verified against the Apollo 11 anchor: 1969-07-20
was a Sunday, and the crate's per-anchor test set pins both.

### `easter_sunday(year)`

```text
Anonymous Gregorian computus (Meeus / Butcher form). Variable names follow
Meeus, Astronomical Algorithms, 2nd ed., §8:

  a = y mod 19
  b = y / 100
  c = y mod 100
  d = b / 4
  e = b mod 4
  f = (b + 8) / 25
  g = (b - f + 1) / 3
  h = (19a + b - d - g + 15) mod 30
  i = c / 4
  k = c mod 4
  l = (32 + 2e + 2i - h - k) mod 7
  m = (a + 11h + 22l) / 451
  month = (h + l - 7m + 114) / 31      // 3 = March, 4 = April
  day   = ((h + l - 7m + 114) mod 31) + 1
```

The algorithm is integer-only and exact for every Gregorian year. It
underlies the TARGET2 holiday rule (Good Friday = Easter − 2, Easter
Monday = Easter + 1) and the Good Friday entry of the US and UK snapshot
calendars.

**Worked example.** `Date::easter_sunday(2026) == Date::ymd(2026, 4, 5)`.
Other anchors pinned by the unit tests: 2024-03-31, 2025-04-20,
2027-03-28, 2038-04-25.

### `add_months_eom_aware(months)`

```text
total     = year * 12 + (month - 1) + months
new_year  = total div 12
new_month = (total mod 12) + 1
new_day   = min(self.day, days_in_month(new_year, new_month))
return Date(new_year, new_month, new_day)
```

The clamp on `new_day` is the load-bearing rule: a date already at the end
of its month rolls forward to the end of the new month, not "the same
numeric day, even though that day does not exist". So `2026-01-31 + 1
month = 2026-02-28` (non-leap) and `2024-01-31 + 1 month = 2024-02-29`
(leap). The arithmetic is performed in `i64` to avoid overflow on the
extremes of the supported year range.

**Worked example.** `Date::ymd(2024, 1, 31).unwrap().add_months_eom_aware(1)
== Date::ymd(2024, 2, 29).unwrap()`. The same call on 2026 (non-leap) lands
on 2026-02-28, demonstrating the leap-year-aware clamp.

### Governing standards

ISO 8601, *Date and time — Representations for information interchange*,
§3.4.1 (calendar date) and §3.4.2 (proleptic Gregorian calendar). Howard
E. Hinnant, *chrono-Compatible Low-Level Date Algorithms*,
<http://howardhinnant.github.io/date_algorithms.html> (public domain).
Jean Meeus, *Astronomical Algorithms*, 2nd ed., Willmann-Bell, 1998, §8
"The date of Easter".

---

## Act/360 — ISDA 2006 §4.16(e) — `src/day_count/act_360.rs`

Act/360 is the money-market default: the year fraction between two dates
is the actual count of calendar days separating them, divided by the fixed
denominator 360. Every major short-term floating-rate index settled in
USD or EUR — SOFR, €STR, the legacy USD-LIBOR fixings, the Fed Funds
effective rate — accrues on Act/360, and so do the EUR and USD legs of the
overwhelming majority of interest-rate swaps. It is the simplest of the
eleven conventions in this crate: a signed day count over the period,
then one division.

### Structure

```text
fraction(start, end) = days_between(start, end) / 360.0

where days_between is signed, end-exclusive, start-inclusive.
```

The denominator is the literal constant `360`. The numerator is signed —
an inverted interval (`start > end`) yields a negative fraction.

### Worked example — 90-day interval

```text
start = 2026-01-01
end   = 2026-04-01
days  = 31 (Jan) + 28 (Feb, 2026 non-leap) + 31 (Mar) = 90
f     = 90 / 360 = 0.25
```

Asserted in `tests/integration.rs::golden::act_360_90_days` to within the
crate-wide `TOL = 1e-12`.

### Governing standard

ISDA 2006 Definitions §4.16(e), *Actual/360*.

---

## Act/365F — ISDA 2006 §4.16(d) — `src/day_count/act_365f.rs`

Act/365F (Actual/365 Fixed) is the sterling-bloc money-market default: the
year fraction between two dates is the actual count of calendar days
separating them, divided by the fixed denominator 365. GBP, JPY, AUD, CAD,
HKD and several other markets quote their short-term floating-rate indices
on this basis — SONIA, TONA, the legacy GBP- and JPY-LIBOR fixings, BBSW
and CDOR all accrue on Act/365F. The procedure is identical to Act/360
except for the denominator.

### Structure

```text
fraction(start, end) = days_between(start, end) / 365.0

where days_between is signed, end-exclusive, start-inclusive.
```

The defining property: a full one-year non-leap-year span (365 calendar
days) returns exactly `1.0`; a leap-year span (366 days) returns
`366/365 ≈ 1.0027`.

### Worked example — 90-day interval

```text
start = 2026-01-01
end   = 2026-04-01
days  = 31 (Jan) + 28 (Feb, 2026 non-leap) + 31 (Mar) = 90
f     = 90 / 365 = 0.246575342465753
```

Asserted in `tests/integration.rs::golden::act_365f_90_days` to within
`TOL = 1e-12`.

### Governing standard

ISDA 2006 Definitions §4.16(d), *Actual/365 (Fixed)*.

---

## ActAct ISDA — ISDA 2006 §4.16(b) — `src/day_count/act_act_isda.rs`

Actual/Actual (ISDA) is the leap-year-aware day-count fraction used by
every standard ISDA-documented interest-rate swap whose floating leg
references an "Actual/Actual" basis. Unlike Act/360 and Act/365F — both
of which use a single, fixed denominator — Act/Act ISDA splits the
calculation period at every January 1 it crosses and divides each
sub-period's actual day count by 366 if its containing calendar year is a
Gregorian leap year and 365 otherwise. The sub-fractions are summed. The
construction is the one specified by the standard verbatim and is the one
printed in the ISDA 2006 Definitions appendix.

### Structure

```text
if start.year == end.year:
    d     = days_between(start, end)
    denom = 366 if is_leap_year(start.year) else 365
    return d / denom

else (multi-year span):
    sum = 0.0
    for y in start.year ..= end.year:
        sub_start = max(start, Date(y,     1, 1))
        sub_end   = min(end,   Date(y + 1, 1, 1))
        d_y       = days_between(sub_start, sub_end)
        if d_y > 0:
            denom = 366 if is_leap_year(y) else 365
            sum  += d_y / denom
    return sum
```

The half-open clamp `[Date(y, 1, 1), Date(y + 1, 1, 1))` per year is the
shape the standard prescribes and matches the crate-wide end-exclusive
convention. The same-year case is fast-pathed.

### Worked example — ISDA 2006 §4.16(b) printed example

```text
start = 2007-12-28
end   = 2008-02-29

Days in 2007: 2007-12-28 → 2008-01-01 (excl.) = 4 days
              (28, 29, 30, 31 December). 2007 non-leap → /365.
Days in 2008: 2008-01-01 → 2008-02-29 (excl.) = 31 + 28 = 59 days.
              2008 leap → /366.

f = 4/365 + 59/366
  = 0.010958904109589 + 0.161202185792350
  = 0.172161089901939
```

Asserted in `tests/integration.rs::golden::isda_act_act_2007_to_2008` to
`(f - 0.172_161_089_901_939).abs() < 1e-12`, matching the printed ISDA
2006 Definitions worked example exactly. Cross-verified against QuantLib
v1.34 `test-suite/daycounters.cpp:145` (1999-02-01 → 1999-07-01 =
0.410958904110), `:157` (1999-07-01 → 2000-07-01 = 1.001377348600), and
four further QuantLib vectors covering the same-year-leap and
multi-year-span paths.

### Governing standard

ISDA 2006 Definitions §4.16(b), *Actual/Actual* / *Actual/Actual (ISDA)*
/ *Act/Act* / *Act/Act (ISDA)*.

---

## ActAct ICMA — ICMA Rule 251 — `src/day_count/act_act_icma.rs`

Actual/Actual (ICMA) is the bond-market day-count convention prescribed by
the International Capital Market Association in Rule 251 and used to
compute accrued interest on fixed-rate bonds the world over. Unlike the
money-market fractions, the denominator is not a constant: it is the
actual length, in calendar days, of the *reference* coupon period
multiplied by the coupon frequency. A semi-annual bond whose calculation
period coincides with one full coupon period therefore always returns
exactly `0.5`, regardless of whether that period happens to be 181 or 184
days long — the property the bond market wants from a "fair fraction of a
coupon".

The two-argument dispatcher `day_count::fraction` cannot compute this
variant because it lacks the reference period and frequency; it returns
`f64::NAN`. Callers route through `day_count::year_fraction_with_freq` (or
call `act_act_icma::fraction` directly) with the five-argument signature.

### Structure

```text
fraction(start, end, ref_start, ref_end, freq) =
    days_between(start, end) / (freq * days_between(ref_start, ref_end))
```

`freq` is the integer coupon frequency: 1 (annual), 2 (semi-annual), 4
(quarterly), 12 (monthly). The implementation covers the **regular-period**
case — the calculation period sits inside (or coincides with) one
reference coupon period. Irregular (long / short) first or last coupons
require the caller to pre-decompose into regular sub-periods and sum.

Defensive behaviour: `freq == 0` or `ref_start == ref_end` returns `0.0`
rather than NaN — both are caller-side precondition violations and the
chosen sentinel avoids propagating an undefined value through downstream
cashflow arithmetic.

### Worked example — full semi-annual period

```text
start     = 2026-03-01     end     = 2026-09-01
ref_start = 2026-03-01     ref_end = 2026-09-01
freq      = 2 (semi-annual)

days(calc) = 31 + 30 + 31 + 30 + 31 + 31 = 184
days(ref)  = 184
f          = 184 / (2 * 184) = 0.5
```

Asserted in `tests/integration.rs::cross_oracle::
quantlib_act_act_icma_full_semiannual_period_is_half` to `1e-12`. The same
test file also pins the annual full-period (= 1.0) and the 1999-02-01 →
1999-07-01 vector (= 0.410958904110) against the same QuantLib reference.

### Governing standard

International Capital Market Association, *ICMA Rule 251 — Accrued
Interest Calculation*.

---

## 30/360 BondBasis — ISDA 2006 §4.16(f) — `src/day_count/thirty_360_bond_basis.rs`

30/360 Bond Basis is the US bond-market default: every month is treated as
30 days long and every year as 360 days, with two specific day-of-month
adjustments applied to the endpoints before differencing. The convention
underpins coupon accrual on US Treasury notes and bonds, US agency debt,
and most US corporate fixed-rate bonds; it is also the historic basis for
the swap-leg "30/360" tag in pre-ISDA confirmations.

### Structure

With `(Y1, M1, D1)` from `start` and `(Y2, M2, D2)` from `end`:

```text
1. If D1 == 31, set D1 = 30.
2. If D2 == 31 AND D1 in {30, 31}, set D2 = 30.
   (After step 1 has run, this is equivalent to "D2 == 31 AND D1 == 30".)
3. numerator = 360 * (Y2 - Y1) + 30 * (M2 - M1) + (D2 - D1)
4. f         = numerator / 360.0
```

The conditionality of step 2 on `D1` is the single rule that distinguishes
30/360 Bond Basis from 30E/360 (§4.16(g)) — Bond Basis suppresses the `D2`
collapse when `D1 ∉ {30, 31}`; 30E/360 collapses unconditionally.

### Worked example — `D2 = 31` with `D1 = 15`, no adjustment

```text
start = 2026-01-15, end = 2026-07-31
D1 = 15 (no change). D2 = 31 but D1 = 15 ∉ {30, 31} → no adjust.
numerator = 360 * 0 + 30 * (7-1) + (31-15) = 180 + 16 = 196
f         = 196 / 360
```

Asserted in `tests/integration.rs::golden::thirty_e_360_vs_bond_basis_d2_31`
to `1e-12`. The same test pairs the BondBasis result against the 30E/360
value on the same input (195/360) so the 1/360 gap between the two
conventions is auditable in a single place. Cross-verified against
QuantLib v1.34 `test-suite/daycounters.cpp:514` (2006-08-31 → 2007-02-28 =
178/360), `:515` (2007-02-28 → 2007-08-31 = 183/360), `:534` (2008-02-29 →
2009-02-28 = 359/360), and three other QuantLib vectors.

### Governing standard

ISDA 2006 Definitions §4.16(f), *30/360* (also known as "Bond Basis").

---

## 30E/360 — ISDA 2006 §4.16(g) — `src/day_count/thirty_e_360.rs`

30E/360 — the "Eurobond Basis" — is the day-count fraction the European
corporate-bond market uses on its fixed legs. It is a thirty-day-month
convention: every month is treated as having exactly 30 days and every
year as having exactly 360, with two end-of-month adjustments applied to
the raw `(year, month, day)` triples before the arithmetic. The result is
a clean, semi-annual coupon period that always lands on `0.5` regardless
of the actual day count.

### Structure

```text
1. If D1 == 31, set D1 = 30.
2. If D2 == 31, set D2 = 30.        (UNCONDITIONAL on D1)
3. numerator = 360 * (Y2 - Y1) + 30 * (M2 - M1) + (D2 - D1)
4. f         = numerator / 360.0
```

Step 2 fires whenever `D2 == 31`, regardless of `D1` — the single
divergence from 30/360 Bond Basis. The two conventions agree on every
input whose `end.day` is not 31; they disagree by `1/360` on the
`start.day ∉ {30, 31}, end.day == 31` case.

### Worked example — `D1 = 15, D2 = 31` collapses

```text
start = 2026-01-15, end = 2026-07-31
D1 = 15 (no change). D2 = 31 → D2 = 30 unconditionally.
numerator = 360 * 0 + 30 * (7-1) + (30-15) = 180 + 15 = 195
f         = 195 / 360
```

Asserted in `tests/integration.rs::golden::thirty_e_360_vs_bond_basis_d2_31`.
Cross-verified against QuantLib v1.34 `test-suite/daycounters.cpp:557`
(2006-02-28 → 2006-08-31 = 182/360), `:561` (2008-02-29 → 2008-08-31 =
181/360, where the BondBasis answer would be 182/360), `:585` (2008-02-28
→ 2008-03-31 = 32/360), and three other QuantLib vectors.

### Governing standard

ISDA 2006 Definitions §4.16(g), *30E/360* (also marketed as the "Eurobond
Basis").

---

## 30E/360 ISDA — ISDA 2006 §4.16(h) — `src/day_count/thirty_e_360_isda.rs`

30E/360 ISDA is the maturity-day-aware sibling of 30E/360. The
30-day-month / 360-day-year arithmetic is identical to that of the plain
30E/360 fraction (§4.16(g)), but the day-of-month adjustments are governed
by a different rule: a day is collapsed to 30 when it is the **last
calendar day of its month** — not merely when it is the 31st. So a 28
February in a non-leap year, a 29 February in a leap year, and a 30 April
all qualify, where the plain 30E/360 fraction adjusts only the four
31-day-month endings.

On top of that, the convention carries one deliberate carve-out: when the
end date is the **maturity date** of the instrument *and* the end month
is February, the end-day adjustment is **suppressed** — `D2` is left at
the actual last day of February (28 or 29). This is the load-bearing case
of the convention; it is the entire reason §4.16(h) exists alongside
§4.16(g). Mishandling it silently over- or under-accrues the final coupon
of every February-maturing instrument that quotes on 30E/360 ISDA — a
1/360 to 2/360 absolute drift in the year fraction, well above any
oracle-matching tolerance.

The signature carries an extra `end_is_maturity` flag the two-argument
dispatcher cannot infer; `day_count::fraction` defaults it to `false`, so
the maturity case must call `thirty_e_360_isda::fraction` directly.

### Structure

```text
1. If D1 == days_in_month(Y1, M1), set D1 = 30.
2. If D2 == days_in_month(Y2, M2)
       AND NOT (M2 == 2 AND end_is_maturity),
   set D2 = 30.
3. numerator = 360 * (Y2 - Y1) + 30 * (M2 - M1) + (D2 - D1)
4. f         = numerator / 360.0
```

`days_in_month` returns 28/29/30/31 for any valid `(year, month)`, so the
"last day" test is exact (and leap-year aware in February).

### Worked example 1 — not at maturity, end-day collapses

```text
start = 2007-08-31, end = 2008-02-29, end_is_maturity = false
D1 = 31 = days_in_month(2007, 8) → D1 = 30
D2 = 29 = days_in_month(2008, 2), M2 = 2 but NOT maturity → D2 = 30
numerator = 360 + 30 * (2 - 8) + (30 - 30) = 360 - 180 = 180
f         = 180 / 360 = 0.5
```

Asserted in `tests/integration.rs::cross_oracle::
quantlib_eurobond_isda_2007_08_31_to_2008_02_29_not_maturity` to `1e-12`,
matching QuantLib v1.34 `test-suite/daycounters.cpp:622`.

### Worked example 2 — at maturity, end-day suppressed

```text
start = 2011-08-31, end = 2012-02-29, end_is_maturity = true
D1 = 31 = days_in_month(2011, 8) → D1 = 30
D2 = 29 = days_in_month(2012, 2), M2 = 2 AND maturity → SUPPRESS, D2 stays 29
numerator = 360 + 30 * (2 - 8) + (29 - 30) = 360 - 180 - 1 = 179
f         = 179 / 360
```

Asserted in `tests/integration.rs::cross_oracle::
quantlib_eurobond_isda_2011_08_31_to_2012_02_29_at_maturity`, matching
QuantLib v1.34 `test-suite/daycounters.cpp:630`. The 1/360 gap between the
two worked examples (180 vs 179) is exactly the swing the February
suppression introduces.

### Governing standard

ISDA 2006 Definitions §4.16(h), *30E/360 (ISDA)*.

---

## Act/365L — ICMA — `src/day_count/act_365l.rs`

Act/365L (Actual/365 Leap-year-sensitive) is the "leap-aware" cousin of
Act/365F: the numerator is the same signed count of actual calendar days
separating the two dates, but the denominator switches to 366 whenever a
29 February falls inside the period and stays at 365 otherwise. The
convention is used for sterling floating-rate notes whose coupon basis
must reflect the presence of a leap day inside the accrual window.

### Structure

```text
numerator   = days_between(start, end)                   // signed
denominator = 366 if any 29 February d satisfies start <= d < end
                  (or end <= d < start for inverted intervals)
              else 365
f           = numerator / denominator
```

The leap-day inclusion test is **start-inclusive, end-exclusive** —
`start <= ymd(y, 2, 29) < end` — exactly matching the half-open
convention `days_between` uses for the numerator. So a period anchored on
a leap-day boundary picks up that 29 February only when it sits on the
`start` edge, never when it sits on the `end` edge. For an inverted
interval the test is symmetric (`end <= ymd(y, 2, 29) < start`), keeping
`fraction(end, start) == -fraction(start, end)` exact.

### Worked example — straddling 2024-02-29

```text
start = 2023-12-01, end = 2024-06-01
days  = 31 (Dec) + 31 (Jan) + 29 (Feb 2024) + 31 (Mar) + 30 (Apr)
      + 31 (May) = 183
2024-02-29 ∈ [2023-12-01, 2024-06-01) → denominator = 366
f     = 183 / 366 = 0.5
```

Asserted in `tests/integration.rs::cross_oracle::
hand_act_365l_straddle_leap_day_2024`. The same module also pins
`hand_act_365l_full_non_leap_year` (= 1.0 for a 365-day non-leap year)
and `hand_act_365l_q3_2024_after_leap` (= 92/365 for an interval that
does *not* contain 2024-02-29, exercising the half-open inclusion).

### Governing standard

ICMA Rule 251 and the sterling money-market convention literature on the
*Actual/365L* leap-aware day count.

---

## NL/365 — conventional — `src/day_count/nl_365.rs`

NL/365 — "No-Leap / 365" — is a money-market variant of Act/365F that
excludes 29 February from the numerator: the fraction's numerator is the
actual count of calendar days between `start` and `end`, *minus* the
number of 29 February dates that fall in the half-open interval `[start,
end)`; the denominator is the fixed constant 365. The effect is that a
full leap-year span and a full non-leap-year span both return exactly
`1.0` — the convention is "leap-year-blind" by construction, in the same
way Act/360 is leap-year-blind by virtue of a constant 360 denominator,
but here every year contributes exactly `365/365` to the fraction.

The convention has no entry in the ISDA 2006 Definitions §4.16 catalogue;
it is a market-practice convention found principally in some legacy
money-market and fixed-income contracts.

### Structure

```text
numerator   = days_between(start, end) - leap_days_in_interval(start, end)
denominator = 365
f           = numerator / 365.0

leap_days_in_interval(lo, hi) = count of ymd(y, 2, 29) for leap year y
                                satisfying lo <= ymd(y, 2, 29) < hi
```

For an inverted interval (`start > end`) the leap-day count is taken over
the forward half-open interval and the sign is carried by the signed
`days_between`, preserving `fraction(end, start) == -fraction(start,
end)`.

### Worked example — full leap year is exactly 1.0

```text
start = 2024-01-01, end = 2025-01-01
days  = 366 (2024 is a leap year)
2024-02-29 ∈ [start, end) → subtract 1
num   = 365
f     = 365 / 365 = 1.0
```

Asserted in `tests/integration.rs::cross_oracle::
hand_nl_365_full_leap_year_is_one`. The same module also pins
`hand_nl_365_eight_year_span_two_leaps` (2020-01-01 → 2028-01-01 =
`2920/365 = 8.0` exactly, covering the 2020 and 2024 leap days) and
`hand_nl_365_q1_non_leap` (90/365 for a non-leap-year first quarter).

### Governing standard

Market-convention day-count catalogues that list NL/365 alongside the
ISDA / ICMA fractions — the convention is non-statutory; no single
primary reference applies.

---

## Bus/252 — Brazilian convention — `src/day_count/bus_252.rs`

Bus/252 is the day-count fraction used for almost every BRL-denominated
fixed-income instrument: federal CDI / Selic-indexed notes, ANBIMA
reference yields, B3-listed DI futures, and the corporate debentures
priced off them. The numerator is the count of **business days** in the
half-open interval `[start, end)` under the caller-supplied calendar; the
denominator is the fixed constant 252 — the conventional Brazilian
business-year length.

Bus/252 cannot share the two-argument signature of the other fractions:
by definition it needs to know which dates are business days, and that
knowledge lives in `crate::calendar`. The signature therefore takes a
caller-supplied `is_business_day: impl Fn(Date) -> bool` predicate; the
two-argument dispatcher `day_count::fraction` returns `f64::NAN` for the
`Bus252` variant. In practice the caller either passes the closure
`|d| calendar::is_business_day(d, cal)` for their chosen `cal`, or uses
the convenience helper `calendar::bus_252_for_calendar(start, end, cal)`
that builds the closure for them.

### Structure

```text
For start <= end:
    count = number of dates d with start <= d < end and is_business_day(d)
    return count as f64 / 252.0
For start > end:
    return -fraction(end, start, is_business_day)
```

The walk is one day at a time, bounded defensively at `MAX_DAYS = 73_000`
steps (~200 years) so a pathological predicate cannot loop indefinitely.

### Worked example — two weeks under TARGET2

```text
start = 2026-05-01 (Fri), end = 2026-05-15 (Fri), calendar = TARGET2
Business days in [2026-05-01, 2026-05-15):
    May  1 (Fri) — TARGET2 Labour Day, NOT a business day
    May  4 (Mon), May  5 (Tue), May  6 (Wed), May  7 (Thu), May  8 (Fri)
    May 11 (Mon), May 12 (Tue), May 13 (Wed), May 14 (Thu)
= 9 business days
f = 9 / 252
```

Asserted in `tests/integration.rs::golden::bus_252_target2_two_weeks`. The
same input is also pinned end-to-end in
`tests/integration.rs::cross_module::bus_252_for_calendar_matches_direct_bus_252_call`,
which checks that the convenience helper and the direct
`bus_252::fraction` call produce bit-exact equal `f64` values.

### Governing standards

ANBIMA, *Caderno de Fórmulas — Títulos Públicos Federais* (Bus/252
numerator definition for LTN, NTN-B, NTN-F). B3, *Manual de Apreçamento
— Derivativos de Renda Fixa* (Bus/252 denominator = 252 for DI futures).

---

## 1/1 — OIS shortcut — `src/day_count/one_one.rs`

1/1 is the degenerate convention: every interval, regardless of length,
direction, or endpoint placement, maps to the year-fraction `1.0`. The
function ignores its two arguments entirely — they are accepted only so
the signature lines up with every other fraction in the dispatcher.

The convention exists as a placeholder inside the schedule of an
overnight-indexed-swap (OIS) contract. In an OIS, the floating coupon is
the geometric compound of the overnight rate over the accrual period; the
daily compounding does *all* the time-weighting work internally. Once
that compounded rate has been computed, multiplying it by the period's
year-fraction would double-count the time dimension. Setting the
day-count to 1/1 keeps the uniform cashflow formula
`payment = notional * rate * fraction(start, end)` with `fraction ≡ 1`
on the OIS leg.

### Structure

```text
fraction(_, _) = 1.0
```

This is the one place in the crate where `fraction(d, d) != 0.0` and where
`fraction(end, start) != -fraction(start, end)`. Both are deliberate.

### Worked example

```text
start = 2026-01-01, end = 2026-04-01 → 1.0
start = 2026-05-23, end = 2026-05-23 → 1.0  (zero-length)
start = 2026-04-01, end = 2026-01-01 → 1.0  (inverted)
```

Asserted bit-exactly (no tolerance) in `src/day_count/one_one.rs`'s unit
tests and via `tests/integration.rs::properties::one_one_is_one_on_zero_length`.

### Governing standard

ISDA OIS terminology notes (the "1/1" / "Act/Act" placeholder on
compounded-rate legs).

---

## Unadjusted date roll — `src/roll.rs`

`Roll::Unadjusted` is the no-op roll: it returns the input date as given,
even when the date is a weekend or holiday. It exists because some term
sheets quote an unadjusted accrual date alongside an adjusted payment
date, and the schedule generator needs to distinguish the two.

### Structure

```text
apply(date, Unadjusted, _) = date
```

The business-day predicate is never consulted.

### Worked example

```text
apply(2026-05-23, Unadjusted, weekends_only) = 2026-05-23 (Sat)
```

Asserted in `src/roll.rs`'s `unadjusted_returns_input` unit test.

### Governing standard

ISDA 2006 Definitions §4.12 (*Business Day Convention*) — the "Unadjusted"
option of the Business Day Convention.

---

## Following date roll — `src/roll.rs`

`Roll::Following` rolls a non-business date forward to the next business
day. It is the default settlement-date convention in most floating-rate
contracts.

### Structure

```text
apply(date, Following, is_biz):
    d = date
    while not is_biz(d):
        d = d.add_days(1)
        (bounded by MAX_STEPS = 366 one-day steps)
    return d
```

A defensive cap of 366 steps in either direction protects against a
pathological predicate. If the cap is reached `apply` silently returns
the input unchanged — the safest fallback for a function without a
`Result` return.

### Worked example — Saturday under a weekends-only calendar

```text
2026-05-23 (Sat)
  -> not a business day → step to 2026-05-24 (Sun)
  -> not a business day → step to 2026-05-25 (Mon)
  -> business day → return 2026-05-25
```

Asserted in `src/roll.rs::following_from_saturday`. The same pattern is
exercised end-to-end under TARGET2 in
`tests/integration.rs::cross_module::adjust_then_fraction_under_target2`
(2026-05-23 (Sat) → 2026-05-25 (Mon) under `Calendar::Target2`).

### Governing standard

ISDA 2006 Definitions §4.12 (*Business Day Convention*) — the "Following
Business Day Convention".

---

## ModifiedFollowing date roll — `src/roll.rs`

`Roll::ModifiedFollowing` rolls forward like `Following`, but if the
forward roll would cross into the next calendar month it falls back to
`Preceding` from the original date. It is the standard convention for
fixed-leg coupon dates on interest-rate swaps and for almost every
month-end-anchored fixing schedule.

### Structure

```text
apply(date, ModifiedFollowing, is_biz):
    f = walk_forward(date, is_biz)
    if f.month() == date.month():
        return f
    else:
        return walk_backward(date, is_biz)
```

### Worked example — Sunday at month-end

```text
2026-05-31 (Sun) under a weekends-only calendar
  Following → 2026-06-01 (Mon)        // crosses into June
  fall back → Preceding from 2026-05-31
              → 2026-05-30 (Sat) → 2026-05-29 (Fri) → return 2026-05-29
```

Asserted in `src/roll.rs::modified_following_falls_back_when_month_changes`.

### Governing standard

ISDA 2006 Definitions §4.12 (*Business Day Convention*) — the "Modified
Following Business Day Convention".

---

## Preceding date roll — `src/roll.rs`

`Roll::Preceding` rolls a non-business date backward to the previous
business day. It is the mirror of `Following` and is occasionally used
for fixing dates that must precede the accrual start.

### Structure

```text
apply(date, Preceding, is_biz):
    d = date
    while not is_biz(d):
        d = d.add_days(-1)
        (bounded by MAX_STEPS = 366 one-day steps)
    return d
```

### Worked example — Saturday under a weekends-only calendar

```text
2026-05-23 (Sat)
  -> not a business day → step to 2026-05-22 (Fri)
  -> business day → return 2026-05-22
```

Asserted in `src/roll.rs::preceding_from_saturday`.

### Governing standard

ISDA 2006 Definitions §4.12 (*Business Day Convention*) — the "Preceding
Business Day Convention".

---

## ModifiedPreceding date roll — `src/roll.rs`

`Roll::ModifiedPreceding` rolls backward like `Preceding`, but if the
backward roll would cross into the previous calendar month it falls back
to `Following` from the original date. It is the symmetric counterpart
to `ModifiedFollowing` and is used in jurisdictions whose conventions
anchor on the *start* of the month.

### Structure

```text
apply(date, ModifiedPreceding, is_biz):
    p = walk_backward(date, is_biz)
    if p.month() == date.month():
        return p
    else:
        return walk_forward(date, is_biz)
```

### Worked example — Sunday at month-start

```text
2026-02-01 (Sun) under a weekends-only calendar
  Preceding → 2026-01-30 (Fri)        // crosses into January
  fall back → Following from 2026-02-01
              → 2026-02-02 (Mon) → return 2026-02-02
```

Asserted in `src/roll.rs::modified_preceding_falls_back_when_month_changes`.

### Governing standard

ISDA 2006 Definitions §4.12 (*Business Day Convention*) — the "Modified
Preceding Business Day Convention".

---

## Nearest date roll — `src/roll.rs`

`Roll::Nearest` rolls a non-business date to the closer of the next and
previous business days; ties go to the forward direction. It is the
market convention used in some short-dated money-market instruments.

### Structure

```text
apply(date, Nearest, is_biz):
    if is_biz(date): return date
    fwd     = walk_forward(date, is_biz)
    bwd     = walk_backward(date, is_biz)
    fwd_d   = |date.days_between(fwd)|
    bwd_d   = |date.days_between(bwd)|
    return fwd if fwd_d <= bwd_d else bwd
```

The tie predicate `fwd_d <= bwd_d` breaks forward, as documented.

### Worked example — Saturday goes to Friday

```text
2026-05-23 (Sat) under a weekends-only calendar
  Friday  2026-05-22 = 1 day back
  Monday  2026-05-25 = 2 days forward
  → return 2026-05-22 (Fri)
```

Asserted in `src/roll.rs::nearest_on_saturday_goes_friday`.

### Governing standard

Market convention — the "Nearest Business Day Convention" is not in
ISDA §4.12 but is widely listed in money-market practitioner references
alongside the four ISDA conventions.

---

## EndOfMonth date roll — `src/roll.rs`

`Roll::EndOfMonth` anchors every monthly roll-forward that started on the
last business day of a month to the last business day of the new month it
lands on. In ISDA wording: "if the date is the last business day of its
month, every rolled coupon date is also the last business day of its
month".

### Structure

```text
apply(date, EndOfMonth, is_biz):
    if is_last_business_day_of_month(date, is_biz):
        return last_business_day_of_month(date, is_biz)
    else:
        return ModifiedFollowing(date, is_biz)
```

`is_last_business_day_of_month` walks forward from `date` to the end of
its calendar month and reports `true` iff no later business day exists.
`last_business_day_of_month` walks backward from the calendar last-day to
find the latest business day inside the month. When `date` is not itself
the last business day of its month, the roll falls through to
`ModifiedFollowing` so the day-of-month behaviour is unsurprising.

The intended usage is "EOM-aware schedule generation": pair this roll
with `Date::add_months_eom_aware` to roll a coupon period that started at
month-end forward by a fixed number of months and land again at the
month-end.

### Worked example — last Friday of a month under TARGET2

```text
2026-05-29 (Fri) is the last business day of May 2026 under TARGET2.
apply(2026-05-29, EndOfMonth, is_business_day(_, Target2)) = 2026-05-29.

2026-05-31 (Sun) is NOT the last business day of May (the predicate
returns false on Sunday). EndOfMonth falls through to ModifiedFollowing,
which lands back on the last business day Fri 2026-05-29.
```

Both branches are asserted in `src/roll.rs::end_of_month_*` and in
`tests/integration.rs::cross_module` (indirectly, via the TARGET2
adjustment helpers).

### Governing standard

ISDA 2006 Definitions §4.12 (*Business Day Convention*) — the "EOM"
qualifier on a Business Day Convention, paired with the "End of Month"
schedule-rolling rule.

---

## TARGET2 calendar — ECB fixed rule — `src/calendar/target2.rs`

TARGET2 — Trans-European Automated Real-time Gross settlement Express
Transfer system — is the euro-area large-value payment system, and its
closing-days schedule defines the "TARGET2 business day" used by every
EUR-denominated cashflow this crate processes. Unlike the six dated-
snapshot calendars catalogued below, TARGET2 is **fully rule-based**: the
European Central Bank fixes exactly six closing dates per year, derivable
from the year alone, and that schedule has not changed since TARGET2 went
live in 2008. No dated table is stored — the rule is exact for every year
in the supported `Date` range.

### Structure

```text
easter        = Date::easter_sunday(date.year())   // Meeus / Computus
good_friday   = easter.add_days(-2)
easter_monday = easter.add_days(+1)

is_holiday(date) =
    date == ymd(year,  1,  1)   ||   // New Year's Day        (fixed)
    date == good_friday          ||   // Easter − 2            (computed)
    date == easter_monday        ||   // Easter + 1            (computed)
    date == ymd(year,  5,  1)   ||   // Labour Day             (fixed)
    date == ymd(year, 12, 25)   ||   // Christmas Day          (fixed)
    date == ymd(year, 12, 26)        // Christmas Holiday      (fixed)
```

Whit Monday (Easter + 50 days) was a TARGET closing day pre-2002 and was
**removed** when TARGET2 superseded TARGET; it is *not* a TARGET2
holiday. The regression test `whit_monday_is_not_a_holiday` pins this.
Weekend dates are not classified as holidays here; the `is_business_day`
dispatcher composes the weekend test with this holiday test.

### Worked example — 2024

```text
Date::easter_sunday(2024) = 2024-03-31
  → Good Friday    = 2024-03-29
  → Easter Monday  = 2024-04-01

holidays(2024) = { 2024-01-01,   // New Year's Day (Mon)
                   2024-03-29,   // Good Friday
                   2024-04-01,   // Easter Monday
                   2024-05-01,   // Labour Day (Wed)
                   2024-12-25,   // Christmas Day (Wed)
                   2024-12-26 }  // Christmas Holiday (Thu)
```

Asserted in `tests/integration.rs::golden::target2_known_holidays` (the
four non-fixed entries: Good Friday, Easter Monday, Christmas Day, Boxing
Day) and in `src/calendar/target2.rs::holidays_2024` (all six).

### Governing standard

European Central Bank, *Decision of the European Central Bank on the
TARGET2 closing days* — the six fixed closing dates, unchanged since
TARGET2 went live on 19 November 2007 and replaced TARGET on 19 May 2008.
Easter is computed by Jean Meeus, *Astronomical Algorithms*, 2nd ed.,
Willmann-Bell, 1998, §8.

---

## Luxembourg calendar — Loi du 21 juillet 1928 — `src/calendar/luxembourg.rs`

Luxembourg's bank-holiday calendar is the schedule under which Luxembourg
banks, the Banque centrale du Luxembourg, and the BCEE close, and which
every Luxembourg-domiciled UCITS or SIF fund uses (typically composited
with TARGET2 via `JointBusiness`) to determine NAV-publication dates.
Like TARGET2 it is **fully rule-based** — every date is either a fixed
civil date or an Easter-derived date that `Date::easter_sunday` already
gives us, so no snapshot table is stored and the calendar is exact for
every year in the supported `Date` range.

The list differs from TARGET2 in five entries: Luxembourg observes
Whit Monday (TARGET2 dropped it in 2002), Ascension Day, Europe Day (from
2019), Luxembourg National Day, the Assumption of Mary, and All Saints'
Day; it does NOT observe Good Friday (which is a religious observance but
not a gazetted Luxembourg bank holiday). The Luxembourg-specific dates
add five extra closures per year relative to TARGET2 — the gap a UCITS /
SIF NAV calendar fills via composition.

### Structure

```text
Eleven holidays per year:

  1.  1 January       (New Year's Day)              fixed
  2.  Easter Monday   = Easter Sunday + 1 day       Easter-derived
  3.  1 May           (Labour Day)                  fixed
  4.  9 May           (Europe Day)                  fixed (from 2019 onwards)
  5.  Ascension Day   = Easter Sunday + 39 days     Easter-derived
  6.  Whit Monday     = Easter Sunday + 50 days     Easter-derived
  7. 23 June          (National Day)                fixed
  8. 15 August        (Assumption of Mary)          fixed
  9.  1 November      (All Saints' Day)             fixed
 10. 25 December      (Christmas Day)               fixed
 11. 26 December      (Boxing Day / St. Stephen)    fixed
```

No weekend-observance shift — like TARGET2, the function reports the
**rule date** in every year. A holiday that falls on a Sunday is reported
on the Sunday; the weekend filter is handled separately by
`is_business_day`.

### Worked example — 2026

```text
Easter Sunday 2026 = 5 April  (computed by Date::easter_sunday)

  2026-01-01  Thu   New Year's Day
  2026-04-06  Mon   Easter Monday   (Easter + 1)
  2026-05-01  Fri   Labour Day
  2026-05-09  Sat   Europe Day
  2026-05-14  Thu   Ascension Day   (Easter + 39)
  2026-05-25  Mon   Whit Monday     (Easter + 50)
  2026-06-23  Tue   National Day
  2026-08-15  Sat   Assumption
  2026-11-01  Sun   All Saints'
  2026-12-25  Fri   Christmas Day
  2026-12-26  Sat   Boxing Day
```

Asserted in `src/calendar/luxembourg.rs::tests::holidays_2026`.

### Europe Day (the 2019 cutover)

Europe Day (the anniversary of the Schuman Declaration, 9 May 1950) was
added to the Luxembourg public-holiday list by the **Loi du 28 février
2019 fixant les jours fériés légaux**, with first observance in 2019.
The implementation guards this exactly:

```text
europe_day = (date == ymd(year, 5, 9)) && year >= 2019
```

so `is_holiday(2018-05-09)` returns `false` and `is_holiday(2019-05-09)`
returns `true`. The cutover is asserted in
`src/calendar/luxembourg.rs::tests::europe_day_added_in_2019`.

### Composition with TARGET2

The canonical Luxembourg-fund NAV calendar is

```rust
use regit_daycount::{Calendar, calendar::JointBusiness};

static LU_FUND: &[Calendar] = &[Calendar::Target2, Calendar::Luxembourg];
let nav = JointBusiness::new(LU_FUND);
```

which closes on any date either TARGET2 or Luxembourg considers
non-business — the broadest correct schedule for a SICAV.

### Governing standards

Loi du 21 juillet 1928 sur le droit du travail (Luxembourg), as amended;
Loi du 28 février 2019 fixant les jours fériés légaux (the law that added
Europe Day). Easter is the Anonymous Gregorian / Meeus Computus shared
with TARGET2.

---

## United States calendar — NYSE / Federal Reserve — `src/calendar/united_states.rs`

The US calendar is an embedded, dated snapshot of the holidays observed
by the New York Stock Exchange and the US Federal Reserve. The NYSE
schedule is a superset of the Federal Reserve operating-day list (the
NYSE adds Good Friday); this module models the NYSE list, which is the
conservative — and standard — choice for US bond and equity settlement
dates.

### Structure

```text
Snapshot date: SNAPSHOT_DATE = "2026-05-23"
Coverage:      COVERAGE      = (2020, 2040) inclusive
Entries:       208 observed holiday dates (10 holidays × 21 years − 2
               edge gaps for Juneteenth before 2022)
```

The ten observed holidays follow these rules (the *observed* date for a
fixed-date holiday falling on a weekend is shifted per the published
schedule; the snapshot records the observed date, not the rule date):

```text
New Year's Day        Jan 1   (observed Fri ← Sat, Mon → Sun)
MLK Day               3rd Monday of January
Presidents' Day       3rd Monday of February
Good Friday           Easter Sunday − 2 days
Memorial Day          last Monday of May
Juneteenth (from 2022) Jun 19 (observed Mon → Sun, Fri ← Sat); NYSE since 2022
Independence Day      Jul 4   (observed Fri ← Sat, Mon → Sun)
Labor Day             1st Monday of September
Thanksgiving          4th Thursday of November
Christmas Day         Dec 25  (observed Fri ← Sat, Mon → Sun)
```

The 208-entry table is a sorted `&'static [(year, month, day)]` array in
the source file; it is not reproduced inline here. Lookups outside
`COVERAGE` return `false` — never a panic, never an extrapolation.

### Worked example

```text
is_holiday(2024-07-04, UnitedStates) = true   // Independence Day (Thu)
is_holiday(2024-11-28, UnitedStates) = true   // Thanksgiving Day (4th Thu)
```

Asserted in `tests/integration.rs::golden::us_known_holidays`. The
out-of-coverage behaviour is asserted in
`tests/integration.rs::invalid::out_of_coverage_calendar_returns_false_not_panic`
(`is_holiday(2019-12-25, UnitedStates) = false`).

### Governing standards

NYSE Holiday Calendar, the published trading-floor schedule. Federal
Reserve Bank Services holiday schedule. SEC / NYSE notices for
discretionary closures (hurricanes, state funerals).

---

## United Kingdom calendar — Bank of England — `src/calendar/united_kingdom.rs`

The UK calendar is an embedded, dated snapshot of the Bank of England's
bank-holiday schedule for England & Wales — the schedule that governs
sterling money-market and gilt settlement dates.

### Structure

```text
Snapshot date: SNAPSHOT_DATE = "2026-05-23"
Coverage:      COVERAGE      = (2020, 2040) inclusive
Entries:       171 entries (8 per year × 21 years − 1 displaced Spring
               Bank Holiday in 2022 + 2 extra Royal-proclamation days)
```

The eight base holidays per year:

```text
New Year's Day      Jan 1   (observed Mon → Sun)
Good Friday         Easter Sunday − 2 days
Easter Monday       Easter Sunday + 1 day
Early May Bank Hol. 1st Monday of May
Spring Bank Hol.    Last Monday of May (DISPLACED in 2022)
Summer Bank Hol.    Last Monday of August (England & Wales)
Christmas Day       Dec 25  (observed Mon → Sun, Tue → Sat)
Boxing Day          Dec 26  (observed Tue → Sun, Mon → Sat)
```

The snapshot also records the Royal proclamation specials — the 2022
Platinum Jubilee (Spring Bank moved to Thu 2 June, plus an extra Friday
3 June), the 2023 Coronation, and the 2022 funeral of Queen Elizabeth II
— which are statutory dated additions, not rule-based.

The 171-entry table lives in the source file; it is not reproduced inline.

### Worked example

```text
is_holiday(2022-06-02, UnitedKingdom) = true  // Platinum Jubilee Spring Bank
is_holiday(2022-06-03, UnitedKingdom) = true  // Platinum Jubilee extra
```

Asserted in `tests/integration.rs::golden::uk_platinum_jubilee_2022`. This
test pins the dated specials that no closed-form rule would generate.

### Governing standards

Bank of England, *UK bank holidays*. HM Government, *UK bank holidays*
(gov.uk). Banking and Financial Dealings Act 1971 (c. 80), Schedule 1.

---

## Japan calendar — JPX — `src/calendar/japan.rs`

The Japan calendar is an embedded, dated snapshot of the Japan Exchange
Group trading calendar for the Tokyo Stock Exchange — which composes the
16 statutory national holidays under the Act on National Holidays
(国民の祝日に関する法律, Law No. 178 of 1948) with the year-end / new-year
exchange closure (typically 2 January and 31 December).

### Structure

```text
Snapshot date: SNAPSHOT_DATE = "2026-05-23"
Coverage:      COVERAGE      = (2020, 2040) inclusive
```

Several Japanese holidays are date-shifted by the "happy Monday" law, and
the vernal and autumnal equinox holidays depend on the equinox tables
published by the National Astronomical Observatory; both make the
snapshot the only reliable form. Substitute holidays (振替休日) for
holidays that fall on a Sunday are also encoded.

The table lives in the source file; it is not reproduced inline.

### Worked example

```text
is_holiday(2024-01-01, Japan) = true   // Ganjitsu (New Year's Day)
```

Asserted in `src/calendar/mod.rs::dispatches_to_every_named_calendar` and
in the `src/calendar/japan.rs` per-year unit tests.

### Governing standards

*Act on National Holidays* (国民の祝日に関する法律), Law No. 178 of 1948.
Cabinet Office (Government of Japan), annual *koyomi yōkō*. Japan
Exchange Group, *Trading Calendar of the Tokyo Stock Exchange*.

---

## Switzerland calendar — SIX — `src/calendar/switzerland.rs`

The Switzerland calendar is an embedded, dated snapshot of the SIX Swiss
Exchange trading calendar — combining the Swiss federal holidays
(Neujahrstag, Bundesfeier on 1 August) with the cantonal holidays the
exchange observes (Berchtoldstag on 2 January, Auffahrt = Ascension Day
on Easter + 39, Pfingstmontag = Whit Monday on Easter + 50).

### Structure

```text
Snapshot date: SNAPSHOT_DATE = "2026-05-23"
Coverage:      COVERAGE      = (2020, 2040) inclusive
Entries:       210 (10 holidays × 21 years; no extraordinary closures)
```

The ten observed holidays per year are: Neujahrstag (Jan 1), Berchtoldstag
(Jan 2), Karfreitag (Easter − 2), Ostermontag (Easter + 1), Tag der Arbeit
(May 1), Auffahrt (Easter + 39), Pfingstmontag (Easter + 50), Bundesfeier
(Aug 1), Weihnachten (Dec 25), Stephanstag (Dec 26).

The table lives in the source file; it is not reproduced inline.

### Worked example

```text
is_holiday(2024-08-01, Switzerland) = true   // Bundesfeier
```

Asserted in `src/calendar/mod.rs::dispatches_to_every_named_calendar`.

### Governing standards

SIX Swiss Exchange, *Trading calendar*. Federal Constitution of the
Swiss Confederation, Art. 110 (1 August National Day).

---

## Hong Kong calendar — HKEX — `src/calendar/hong_kong.rs`

The Hong Kong calendar is an embedded, dated snapshot of the Hong Kong
Exchanges and Clearing trading calendar. The HKEX schedule combines the
Western fixed-date holidays (Western New Year, Christmas, Boxing Day)
with several **lunar / Chinese-calendar holidays** — Lunar New Year (3
days), Ching Ming Festival, Buddha's Birthday, Tuen Ng (Dragon Boat),
Mid-Autumn Festival, Chung Yeung Festival — none of which have a closed-
form Gregorian rule and which therefore force the snapshot form.

### Structure

```text
Snapshot date: SNAPSHOT_DATE = "2026-05-23"
Coverage:      COVERAGE      = (2020, 2040) inclusive
```

The table lives in the source file; it is not reproduced inline.

### Worked example

```text
is_holiday(2024-12-25, HongKong) = true   // Christmas (Western fixed)
```

Asserted in `src/calendar/mod.rs::dispatches_to_every_named_calendar`.

### Governing standards

Hong Kong Exchanges and Clearing Limited, *HKEX Trading Calendar*. Hong
Kong Government Gazette, *General Holidays Ordinance* (Cap. 149).

---

## Singapore calendar — SGX — `src/calendar/singapore.rs`

The Singapore calendar is an embedded, dated snapshot of the Singapore
Exchange trading calendar. The SGX schedule combines the Western fixed-
date holidays with the multi-faith Singaporean public holidays — Chinese
New Year (lunar), Vesak Day (Buddhist), Hari Raya Puasa and Hari Raya
Haji (Islamic), Deepavali (Hindu / lunar) — and applies the §4 Holidays
Act 1998 Monday-in-lieu rule for fixed-date holidays falling on a Sunday.

### Structure

```text
Snapshot date: SNAPSHOT_DATE = "2026-05-23"
Coverage:      COVERAGE      = (2020, 2040) inclusive
```

The table lives in the source file; it is not reproduced inline.

### Worked example

```text
is_holiday(2024-08-09, Singapore) = true   // National Day
```

Asserted in `src/calendar/mod.rs::dispatches_to_every_named_calendar`.

### Governing standards

Singapore Ministry of Manpower, *Public Holidays in Singapore*. Singapore
Exchange (SGX), *SGX Securities and Derivatives Trading Calendar*.
Holidays Act 1998 (Singapore), §4 (Monday-in-lieu rule).

---

## Calendar dispatchers and business-day primitives — `src/calendar/mod.rs`

The `calendar` module exposes a small, uniform set of free functions
that fan out over the named [`Calendar`] enum. Every function is `Copy`-
by-value over `Date` + `Calendar`, allocation-free, and `no_std`-clean.

### Classification

```text
is_weekend(date)             -> bool   // true iff Sat / Sun
is_holiday(date, cal)        -> bool   // true iff date is a holiday under cal
                                       //   (does NOT include weekends)
is_business_day(date, cal)   -> bool   // !is_weekend(date) && !is_holiday(date, cal)
```

### Rolling and walking

```text
adjust(date, conv, cal)           -> Date  // roll::apply(date, conv,
                                           //             |d| is_business_day(d, cal))
next_business_day(date, cal)      -> Date  // adjust(date, Roll::Following, cal)
previous_business_day(date, cal)  -> Date  // adjust(date, Roll::Preceding, cal)
add_business_days(date, n, cal)   -> Date  // T+N settlement primitive
business_days_between(start, end, cal) -> u32   // half-open [start, end)
bus_252_for_calendar(start, end, cal)  -> f64   // Bus/252 closed over cal
```

### `add_business_days(date, n, cal)`

The settlement-date primitive. Walks one calendar day at a time,
counting only business days (under `cal`) against `n`. Weekends and
holidays are skipped without consuming a count.

- `n > 0`  — walk forward; T+N settlement on T = `date`.
- `n < 0`  — walk backward; the "n business days before" form.
- `n == 0` — return `date` unchanged if it is itself a business day;
  otherwise walk forward to the next business day (the market convention
  for T+0 on a non-business start).

```text
add_business_days(Mon 2026-05-25, 2, Target2)   = Wed 2026-05-27
add_business_days(Thu 2026-05-28, 2, Target2)   = Mon 2026-06-01   // across weekend
add_business_days(Thu 2026-05-28, -2, Target2)  = Tue 2026-05-26
```

The walk is bounded at roughly 200 years of calendar walking — a
defensive cap that no real settlement convention approaches. If the
cap is reached, the partial result is returned.

### `business_days_between(start, end, cal)`

Counts business days in `[start, end)` (start-inclusive, end-exclusive
— the same half-open convention used everywhere in this crate). For
`start >= end` the count is `0`; the function does not return signed
counts. Walk is one calendar day at a time, defensively bounded at
~200 years.

```text
business_days_between(2026-05-01, 2026-05-15, Target2) = 9
  // May 1 is Labour Day; business days are 4, 5, 6, 7, 8, 11, 12, 13, 14
```

### Worked example — TARGET2 fortnight

Asserted in `src/calendar/mod.rs::tests::business_days_between_two_weeks_with_target2_labour_day`
and the matching `bus_252_for_calendar_matches_manual_count` test.

### Governing standards

ISDA 2006 Definitions §4.12 (business-day conventions) for the roll
fall-through to [`crate::roll::apply`]. The T+N settlement form is
market convention; the half-open `[start, end)` interval is consistent
with the day-count fractions in §3.

---

## Composite / JointBusiness combinators — `src/calendar/composite.rs`

A real-world cashflow often touches more than one calendar at once. A
cross-currency swap settles a leg in each currency on the other leg's
holiday: the joint settlement date is the next day on which both legs'
calendars are open. A Luxembourg-domiciled UCITS fund computes NAV on a
day that is both a TARGET2 business day and a Luxembourg bank-holiday
business day. The two combinators in this module name those operations.

- **`Composite`** takes the **union of holidays**: a date is a holiday on
  the composite if it is a holiday on *any* underlying calendar.
  Equivalent to "broadest possible non-business set across the group".
- **`JointBusiness`** takes the **intersection of business days**: a
  date is a business day iff *every* underlying calendar treats it as
  one. Equivalent to "narrowest possible business set across the group".

The two operations are semantic duals (`Composite::is_holiday` is the
union of holidays; `JointBusiness::is_business_day` is the intersection of
business days, which is the same set once weekends are accounted for);
the distinction is one of ergonomic framing, not of semantics. Both
structs hold a `&'static [Calendar]`, are `Copy`, allocation-free, and
`no_std`-clean.

### Structure

```text
Composite { kinds: &'static [Calendar] }
    is_holiday(d)      = any(is_holiday(d, k) for k in kinds)
    is_business_day(d) = all(is_business_day(d, k) for k in kinds)

JointBusiness { kinds: &'static [Calendar] }
    is_business_day(d) = all(is_business_day(d, k) for k in kinds)
    is_holiday(d)      = !is_business_day(d) && !is_weekend(d)
```

`is_business_day` on both types is the intersection — it is `false` if
*any* underlying calendar would close on `d`, whether through a weekend or
a named holiday.

### Worked example — EUR/USD joint business calendar, 4 July 2024

```text
JointBusiness { kinds: &[Target2, UnitedStates] }

2024-07-04 (Thu)
  is_holiday(_, Target2)       = false    // not a TARGET2 holiday
  is_business_day(_, Target2)  = true     // Thursday, not a holiday → open
  is_holiday(_, UnitedStates)  = true     // Independence Day
  is_business_day(_, UnitedStates) = false

is_business_day(2024-07-04) = all([true, false]) = false
is_holiday(2024-07-04)      = true                    // composite holiday
```

Asserted in `tests/integration.rs::golden::joint_us_eur_4_july_not_business`
and in `tests/integration.rs::cross_module::composite_then_business_days_between`
(which walks the full week 2024-07-01 → 2024-07-08 under
`Composite([Target2, UnitedStates])` and counts 4 business days,
correctly dropping Thu 4 July as the only US-only holiday in that
window).

### Governing standards

The combinators have no single governing standard; the union-of-holidays
and intersection-of-business-days framings are the multi-calendar
practitioner conventions that real-world cross-currency, multi-leg and
joint-jurisdiction settlement schedules use.

---

## Standards index

| Standard / source | Section | File |
|---|---|---|
| ISO 8601 (calendar date, proleptic Gregorian) | §2 | `src/date.rs` |
| Howard Hinnant, *chrono-Compatible Low-Level Date Algorithms* | §2 | `src/date.rs` |
| Jean Meeus, *Astronomical Algorithms*, 2nd ed., §8 (Computus) | §2, §21, §25 | `src/date.rs`, `src/calendar/target2.rs`, `src/calendar/switzerland.rs` |
| ISDA 2006 Definitions §4.16(e), *Actual/360* | §3 | `src/day_count/act_360.rs` |
| ISDA 2006 Definitions §4.16(d), *Actual/365 (Fixed)* | §4 | `src/day_count/act_365f.rs` |
| ISDA 2006 Definitions §4.16(b), *Actual/Actual (ISDA)* | §5 | `src/day_count/act_act_isda.rs` |
| ICMA Rule 251, *Accrued Interest Calculation* | §6, §10 | `src/day_count/act_act_icma.rs`, `src/day_count/act_365l.rs` |
| ISDA 2006 Definitions §4.16(f), *30/360* (Bond Basis) | §7 | `src/day_count/thirty_360_bond_basis.rs` |
| ISDA 2006 Definitions §4.16(g), *30E/360* (Eurobond Basis) | §8 | `src/day_count/thirty_e_360.rs` |
| ISDA 2006 Definitions §4.16(h), *30E/360 (ISDA)* | §9 | `src/day_count/thirty_e_360_isda.rs` |
| ANBIMA, *Caderno de Fórmulas — Títulos Públicos Federais* | §12 | `src/day_count/bus_252.rs` |
| B3, *Manual de Apreçamento — Derivativos de Renda Fixa* | §12 | `src/day_count/bus_252.rs` |
| ISDA OIS terminology notes (1/1 placeholder) | §13 | `src/day_count/one_one.rs` |
| ISDA 2006 Definitions §4.12, *Business Day Convention* | §14–§20 | `src/roll.rs` |
| ECB *Decision on the TARGET2 closing days* | §21 | `src/calendar/target2.rs` |
| NYSE Holiday Calendar | §22 | `src/calendar/united_states.rs` |
| Federal Reserve Bank Services holiday schedule | §22 | `src/calendar/united_states.rs` |
| Bank of England, *UK bank holidays* | §23 | `src/calendar/united_kingdom.rs` |
| Banking and Financial Dealings Act 1971 (c. 80), Schedule 1 | §23 | `src/calendar/united_kingdom.rs` |
| *Act on National Holidays* (Japan), Law No. 178 of 1948 | §24 | `src/calendar/japan.rs` |
| Japan Exchange Group, *Trading Calendar of the Tokyo Stock Exchange* | §24 | `src/calendar/japan.rs` |
| SIX Swiss Exchange, *Trading calendar* | §25 | `src/calendar/switzerland.rs` |
| Hong Kong Exchanges and Clearing, *HKEX Trading Calendar* | §26 | `src/calendar/hong_kong.rs` |
| Hong Kong *General Holidays Ordinance* (Cap. 149) | §26 | `src/calendar/hong_kong.rs` |
| Singapore Exchange (SGX), *Trading Calendar* | §27 | `src/calendar/singapore.rs` |
| Singapore Ministry of Manpower, *Public Holidays in Singapore* | §27 | `src/calendar/singapore.rs` |
| Singapore *Holidays Act 1998*, §4 (Monday-in-lieu rule) | §27 | `src/calendar/singapore.rs` |
| QuantLib v1.34, `test-suite/daycounters.cpp` (cross-oracle reference) | §5, §6, §7, §8, §9 | `tests/integration.rs` |

---

*Part of [Regit OS](https://www.regit.io) — the operating system for
investment products. From Luxembourg.*
