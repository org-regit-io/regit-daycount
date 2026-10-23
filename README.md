<!-- Copyright 2026 Regit.io — Nicolas Koenig -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# regit-daycount

Day-count fractions and business-day calendars. Zero-dependency, pure Rust,
`no_std`.

[![Crates.io](https://img.shields.io/crates/v/regit-daycount.svg)](https://crates.io/crates/regit-daycount)
[![docs.rs](https://docs.rs/regit-daycount/badge.svg)](https://docs.rs/regit-daycount)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)
[![no_std](https://img.shields.io/badge/no__std-yes-success.svg)](https://docs.rust-embedded.org/book/intro/no-std.html)

## What it does

`regit-daycount` computes year fractions under 11 **day-count conventions**
(Act/360, Act/365F, ActAct ISDA, ActAct ICMA,
30/360 BondBasis, 30E/360, 30E/360 ISDA, Act/365L, NL/365, Bus/252, 1/1),
adjusts dates under seven **date-roll conventions**
(Unadjusted, Following, ModifiedFollowing, Preceding,
ModifiedPreceding, Nearest, EndOfMonth), and classifies dates against the
major **holiday calendars** (TARGET2, Luxembourg, NYSE, Federal Reserve,
United Kingdom, Japan, Switzerland, Hong Kong, Singapore, plus composite and joint-business
combinations).

Date primitives — a Gregorian `Date { year, month, day }`, the leap-year and
days-in-month rules, the day-of-week computation, fixed and Easter-based
holiday algorithms — are own-rolled, zero-dependency, `Copy`.

Every algorithm is traced to a citable standard in [SPEC.md](SPEC.md). An
auditor, a structurer, or a new engineer can open any source file and check
it against ISDA 2006 §4.16, ICMA Rule 251, the ISO 8601 weekday rule, or the
published calendar of the exchange or central bank in question.

## Why this crate exists

A coupon is paid on a date. An accrued-interest amount is settled to a date.
A swap leg is discounted from a date. Every one of those operations is
parameterised by two conventions — *which* day-count fraction, *which*
holiday calendar — and a wrong choice, or a correct choice computed wrongly,
silently produces a wrong cashflow.

The algorithms are deceptively easy to get subtly wrong. Act/Act ISDA
straddling a year-end splits the interval at midnight on 1 January and
uses two different denominators on the two sides. 30E/360 ISDA treats the
last day of February specially — but only when it is not the maturity date.
ModifiedFollowing falls back to Preceding when the Following adjustment would
cross a month boundary. TARGET2 is open on every weekday except 1 January,
Good Friday, Easter Monday, 1 May, 25 December, and 26 December — and Easter
is a function of the year. A library that gets any of these wrong reports
a wrong number that downstream systems will trust.

`regit-daycount` implements each convention from its governing standard,
verifies it against worked examples for real instruments, and ships the
verification alongside the code. It is `no_std` and allocation-free, so the
same calculation logic runs in a backend service, a WASM bundle, or on an
embedded device with no change.

This sits within [Regit OS](https://www.regit.io): `regit-daycount` is the
time-and-calendar layer — the component that decides what fraction of a year
a given date interval represents, and whether a given date is one on which a
market is open.

## Quick start

```toml
[dependencies]
regit-daycount = "1.1.0"
```

The library requires Rust 1.85 or newer. Upgrading from 1.0.x? Read the
[migration notes](#migrating-from-10x) before changing dependencies.

```rust
use regit_daycount::{Calendar, Date, DayCount, Roll, calendar, day_count};

// Year fraction under Act/360 for a 90-day interval.
let start = Date::ymd(2026, 1, 1).unwrap();
let end   = Date::ymd(2026, 4, 1).unwrap();
let yf    = day_count::try_fraction(start, end, DayCount::Act360, Default::default()).unwrap();
assert!((yf - 90.0 / 360.0).abs() < 1e-12);

// Roll a date forward to the next TARGET2 business day.
let saturday = Date::ymd(2026, 5, 2).unwrap();
let monday   = calendar::try_adjust(saturday, Roll::Following, Calendar::Target2).unwrap();
assert_eq!(monday, Date::ymd(2026, 5, 4).unwrap());

// Is 2026-12-25 a TARGET2 holiday?
assert!(calendar::try_is_holiday(Date::ymd(2026, 12, 25).unwrap(), Calendar::Target2).unwrap());
```

See [`examples/quickstart.rs`](examples/quickstart.rs) for a complete tour
covering every day-count fraction, every date-roll convention, and every
calendar.

## Day-count fractions covered

| Fraction | Standard | Module |
|---|---|---|
| Act/360 | ISDA 2006 §4.16(e) | [`day_count::act_360`](src/day_count) |
| Act/365F (Fixed) | ISDA 2006 §4.16(d) | [`day_count::act_365f`](src/day_count) |
| ActAct ISDA | ISDA 2006 §4.16(b) | [`day_count::act_act_isda`](src/day_count) |
| ActAct ICMA (one reference period; caller splits stubs) | ICMA Rule 251 | [`day_count::act_act_icma`](src/day_count) |
| 30/360 BondBasis | ISDA 2006 §4.16(f) | [`day_count::thirty_360_bond_basis`](src/day_count) |
| 30E/360 | ISDA 2006 §4.16(g) | [`day_count::thirty_e_360`](src/day_count) |
| 30E/360 ISDA | ISDA 2006 §4.16(h) | [`day_count::thirty_e_360_isda`](src/day_count) |
| Act/365L | ICMA / sterling money-market | [`day_count::act_365l`](src/day_count) |
| NL/365 (No-Leap) | conventional | [`day_count::nl_365`](src/day_count) |
| Bus/252 | Brazilian convention | [`day_count::bus_252`](src/day_count) |
| 1/1 | OIS shortcut | [`day_count::one_one`](src/day_count) |

## Date-roll conventions

- **Unadjusted** — return the date as given, even if it falls on a weekend or
  holiday.
- **Following** — roll forward to the next business day.
- **ModifiedFollowing** — Following, unless that would cross a month boundary,
  in which case roll back to the previous business day.
- **Preceding** — roll back to the previous business day.
- **ModifiedPreceding** — Preceding, unless that would cross a month boundary,
  in which case roll forward to the next business day.
- **Nearest** — choose the closest business day; ties are broken forward.
- **EndOfMonth** — for this single-date API, preserve a business date and
  apply ModifiedFollowing to a non-business date. Month stepping is supplied
  separately through `Date::add_months_eom_aware` (calendar-day clamping).

## Holiday calendars

| Calendar | Source | Module |
|---|---|---|
| TARGET2 | ECB-defined fixed rule | [`calendar::target2`](src/calendar) |
| Luxembourg bank holidays | Government bank-holiday list, including Good Friday | [`calendar::luxembourg`](src/calendar) |
| UnitedStatesNyse | NYSE trading calendar | [`calendar::united_states`](src/calendar) |
| UnitedStatesFederalReserve | Federal Reserve operating calendar | [`calendar::united_states`](src/calendar) |
| United Kingdom | Bank of England | [`calendar::united_kingdom`](src/calendar) |
| Japan | Japan Exchange Group | [`calendar::japan`](src/calendar) |
| Switzerland | SIX Swiss Exchange | [`calendar::switzerland`](src/calendar) |
| Hong Kong | HKEX | [`calendar::hong_kong`](src/calendar) |
| Singapore | SGX | [`calendar::singapore`](src/calendar) |
| Composite | union of holidays | [`calendar::composite`](src/calendar) |
| JointBusiness | intersection of business days | [`calendar::composite`](src/calendar) |

Use the checked `calendar::try_*` APIs for financial calculations. They
reject dates outside supported coverage and, by default, dates whose data
has not been verified against a primary published schedule. An explicit
`CoveragePolicy::AllowProjected` opts into rules or snapshots beyond that
verified horizon. `calendar::coverage` exposes both horizons. Checked date
walks validate the dates they visit and return errors instead of partial
counts or guessed payment dates.

| Calendar | Available with explicit opt-in | Verified by default |
|---|---|---|
| TARGET2 | 2002–9999 rule | 2002–2028 standing rule |
| Luxembourg | 2019–9999 rule | 2026–2027 |
| NYSE | 2020–2040 snapshot | 2026–2028 |
| Federal Reserve | 2020–2040 rule | 2026–2030 |
| Singapore | 2020–2040 snapshot | 2026–2027 |
| UK, Japan, Switzerland, Hong Kong | 2020–2040 snapshots | No complete annual audit retained |

Verification readback: 23 October 2026. The TARGET2 range reflects the
standing ECB rule, effective in 2002. `AllowProjected` also permits
unverified historical rows. It does not extend available coverage.

`UnitedStates` remains a legacy alias for `UnitedStatesNyse`. NYSE and the
Federal Reserve differ on Good Friday, Columbus Day, Veterans Day and
Saturday observance. Choose the market explicitly. Early closes, including
Luxembourg's Christmas Eve afternoon, remain business days in this date-only
model.

Legacy Boolean/calendar walking functions remain available. Snapshot
lookups outside 2020–2040 still return `false` for a holiday, which can make
an unknown weekday appear open. See [SPEC.md](SPEC.md) for the checked
coverage contract and [validation/README.md](validation/README.md) for the
independent validation scope.

## Required day-count context

Prefer `day_count::try_fraction` with `DayCountContext`. Act/365L and
Act/Act ICMA require `CouponPeriod { start, end, frequency }`; 30E/360 ISDA
requires `end_is_maturity: Some(true/false)`. Bus/252 requires an explicit
business-day predicate or a checked calendar wrapper. Missing context and
invalid coupon inputs return typed errors without allocation.

Act/365L uses the next coupon date and frequency. For annual coupons its
leap-day test excludes the accrual start and includes the next coupon date;
for other frequencies the next coupon's year determines the denominator.
NL/365 also omits leap days in `(start, end]`, matching QuantLib NoLeap.

ICMA supports an accrual contained in one reference period; callers must
split irregular coupons into nominal periods themselves.

## Migrating from 1.0.x

Version 1.1.0 corrects financial results and adds checked APIs. These changes
require attention even where existing function signatures are retained:

- Context-free `day_count::fraction` now returns `NaN` for `Act365L` and
  `ThirtyE360Isda`, in addition to ICMA and Bus/252. Use `try_fraction` with
  explicit coupon or maturity context. The two-date `act_365l::fraction`
  also returns `NaN`; `year_fraction_with_freq` now consumes its coupon
  arguments for Act/365L.
- Invalid legacy ICMA calls return `NaN` instead of zero. Use `try_*` APIs
  to distinguish zero frequency, invalid reference periods and accruals
  outside the supplied reference period.
- `Calendar` adds `UnitedStatesNyse` and `UnitedStatesFederalReserve`.
  Update exhaustive matches to handle both. `UnitedStates` continues to
  select NYSE; choose Federal Reserve explicitly for that operating calendar.
- Checked calendar APIs require verified coverage by default. Use
  `CoveragePolicy::AllowProjected` explicitly when accepting available
  unverified or projected data. Calendar corrections and NL/365 leap-day
  inclusion can change previously stored fractions or payment dates; see
  [the 1.1.0 changelog](CHANGELOG.md#110---2026-10-23) for the corrections.

## Architecture

```
src/
  lib.rs                # Module declarations + re-exports
  errors.rs             # Typed errors — ValidationError
  date.rs               # Gregorian Date primitive; leap years, weekday, arithmetic
  roll.rs               # Date-roll conventions — Unadjusted, Following, ...

  day_count/
    mod.rs              # DayCount enum + dispatcher
    act_360.rs          # Act/360
    act_365f.rs         # Act/365F
    act_act_isda.rs     # ActAct ISDA — straddle-aware
    act_act_icma.rs     # ActAct ICMA — reference-period aware
    thirty_360_bond_basis.rs
    thirty_e_360.rs
    thirty_e_360_isda.rs
    act_365l.rs
    nl_365.rs
    bus_252.rs
    one_one.rs

  calendar/
    mod.rs              # Calendar enum + dispatcher (is_holiday,
                        # is_business_day, adjust, next_business_day,
                        # previous_business_day, add_business_days,
                        # business_days_between, bus_252_for_calendar)
    target2.rs          # ECB fixed rule (Easter-derived)
    luxembourg.rs       # Luxembourg bank holidays, including Good Friday
    united_states.rs    # NYSE table and separate Federal Reserve rules
    united_kingdom.rs   # Bank of England table
    japan.rs            # JPX table
    switzerland.rs      # SIX table
    hong_kong.rs        # HKEX table
    singapore.rs        # SGX table
    composite.rs        # Composite / JointBusiness combinators
```

One file, one convention. Public value types are `Copy` and allocation-free.
`Date::ymd` validates construction; `ymd_unchecked` and date arithmetic have
caller preconditions documented on their methods.

## Testing

Development uses Rust 1.99.1; the library MSRV remains 1.85.

```bash
cargo test                      # unit + integration + doc-tests
cargo run --example quickstart  # end-to-end tour
cargo bench                     # criterion benchmarks
```

Run the complete independent reference gate:

```bash
uv run validation/oracle.py --check --rust
```

The 1.1.0 baseline contains **477,265 independent reference cases**
(**208,059** with calendars disabled). The gate executes reference
implementations for all 11 day-count conventions,
all seven date rolls, every named calendar (including the US compatibility
alias), all public date operations, business-day operations, and both calendar
combinators. Every civil date in the calendars' 2020–2040 snapshot window is
compared for named-holiday and business-day classification. Gregorian
leap-year, month-length and Easter cases cover every supported year, 1583–9999.

References are pinned QuantLib 1.44 and FinancePy 1.1.2,
`holidays` 0.107 financial-market calendars, Python's Gregorian date library,
`python-dateutil` 2.9.0.post0 and `convertdate` 2.5.1. Financial fractions use
an absolute tolerance of `1e-12`; dates, flags and integer counts match exactly.
The command fails on a mismatch, failed reference execution or a public
convention/operation missing from the generated cases. It checks the actual
Rust public catalogues so additions cannot silently bypass validation.

Ordinary `cargo test` consumes the same expected results offline. The oracle
and reference libraries run through Python. Rust test adapters remain outside
`src`; the production crate keeps zero runtime dependencies. The exact mappings,
case counts and source provenance are in
[validation/README.md](validation/README.md) and
[validation/oracle.json](validation/oracle.json).

## Code quality

- `#![no_std]`, allocation-free — runs in services, WASM, and on embedded
  targets with no `std`
- `#![forbid(unsafe_code)]` crate-wide
- `clippy::pedantic` with zero warnings
- Checked `try_*` calculation and calendar APIs return typed errors; legacy
  convenience functions retain documented NaN and out-of-coverage behavior
- Public APIs document calculation context, coverage and caller preconditions
  with examples
- Deterministic: the same input always produces the same verdict

## Dependencies

**Runtime: zero.** Not `std`, not `alloc`, no FFI. Every day-count fraction,
every date-roll convention, and every calendar lookup is hand-rolled from
its governing standard. Licence and supply-chain policy is enforced via
`cargo-deny` (`deny.toml`).

The default `calendars` feature embeds dated holiday-table snapshots as
static, `no_std`-clean data. Disable it for a structural-only build (date
primitives, day-count fractions and predicate-based date rolls; the entire
calendar module, including TARGET2, becomes unavailable):
`cargo build --no-default-features`.

## Standards

The specification records each implemented convention, its governing
references and the supported calculation scope.

| Standard | Convention |
|---|---|
| ISDA 2006 Definitions §4.16 | Act/360, Act/365F, ActAct ISDA, 30/360, 30E/360, 30E/360 ISDA |
| ICMA Rule 251 | ActAct ICMA, Act/365L |
| ISO 8601 | Date format, weekday rule |
| ECB TARGET2 closing days regulation | TARGET2 calendar |
| Loi du 21 juillet 1928 / Loi du 28 février 2019 (Luxembourg) | Luxembourg calendar |
| NYSE trading schedule; Federal Reserve operating schedule | Separate US market calendars |
| Bank of England bank holiday schedule | United Kingdom calendar |
| JPX / National Holidays Act (Japan) | Japan calendar |
| SIX Swiss Exchange published schedule | Switzerland calendar |
| HKEX published schedule | Hong Kong calendar |
| SGX published schedule | Singapore calendar |

## Documentation

- [SPEC.md](SPEC.md) — every day-count fraction, date-roll convention, and
  calendar rule traced to its standard, with worked examples
- [CHANGELOG.md](CHANGELOG.md) — release history
- [AUTHORS.md](AUTHORS.md) — authorship
- [CITATION.cff](CITATION.cff) — citation metadata
- [SECURITY.md](SECURITY.md) — vulnerability disclosure policy

## License

Apache License 2.0. See [LICENSE](LICENSE) and [NOTICE](NOTICE).

```
Copyright 2026 Regit.io — Nicolas Koenig
```

---

Part of [Regit OS](https://www.regit.io) — the operating system for investment products. From Luxembourg.
