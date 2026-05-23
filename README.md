<!-- Copyright 2026 Regit.io — Nicolas Koenig -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# regit-daycount

Day-count fractions and business-day calendars. Zero-dependency, pure Rust,
`no_std`.

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)
[![no_std](https://img.shields.io/badge/no__std-yes-success.svg)](https://docs.rust-embedded.org/book/intro/no-std.html)

## What it does

`regit-daycount` computes the year fraction between two dates under every
common **day-count fraction** (Act/360, Act/365F, ActAct ISDA, ActAct ICMA,
30/360 BondBasis, 30E/360, 30E/360 ISDA, Act/365L, NL/365, Bus/252, 1/1),
adjusts a date for a non-business day under every common **date-roll
convention** (Unadjusted, Following, ModifiedFollowing, Preceding,
ModifiedPreceding, Nearest, EndOfMonth), and classifies dates against the
major **holiday calendars** (TARGET2, United States, United Kingdom, Japan,
Switzerland, Hong Kong, Singapore, plus composite and joint-business
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
same audited logic runs in a backend service, a WASM bundle, or on an
embedded device with no change.

This sits within [Regit OS](https://www.regit.io): `regit-daycount` is the
time-and-calendar layer — the component that decides what fraction of a year
a given date interval represents, and whether a given date is one on which a
market is open.

## Quick start

```toml
[dependencies]
regit-daycount = "0.1"
```

```text
use regit_daycount::{Calendar, Date, DayCount, Roll, calendar, day_count};

// Year fraction under Act/360 for a 90-day interval.
let start = Date::ymd(2026, 1, 1).unwrap();
let end   = Date::ymd(2026, 4, 1).unwrap();
let yf    = day_count::fraction(start, end, DayCount::Act360);
assert!((yf - 90.0 / 360.0).abs() < 1e-12);

// Roll a date forward to the next TARGET2 business day.
let saturday = Date::ymd(2026, 5, 2).unwrap();
let monday   = calendar::adjust(saturday, Roll::Following, Calendar::Target2);
assert_eq!(monday, Date::ymd(2026, 5, 4).unwrap());

// Is 2026-12-25 a TARGET2 holiday?
assert!(calendar::is_holiday(Date::ymd(2026, 12, 25).unwrap(), Calendar::Target2));
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
| ActAct ICMA | ICMA Rule 251 | [`day_count::act_act_icma`](src/day_count) |
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
- **Nearest** — the nearest business day (Saturday rolls to Friday, Sunday
  rolls to Monday, ties broken forward).
- **EndOfMonth** — anchor every rolled date to the last business day of the
  month when the input is the last business day of *its* month.

## Holiday calendars

| Calendar | Source | Module |
|---|---|---|
| TARGET2 | ECB-defined fixed rule | [`calendar::target2`](src/calendar) |
| Luxembourg | Loi du 21 juillet 1928 (+ Loi du 28 février 2019) | [`calendar::luxembourg`](src/calendar) |
| United States | NYSE / Federal Reserve | [`calendar::united_states`](src/calendar) |
| United Kingdom | Bank of England | [`calendar::united_kingdom`](src/calendar) |
| Japan | Japan Exchange Group | [`calendar::japan`](src/calendar) |
| Switzerland | SIX Swiss Exchange | [`calendar::switzerland`](src/calendar) |
| Hong Kong | HKEX | [`calendar::hong_kong`](src/calendar) |
| Singapore | SGX | [`calendar::singapore`](src/calendar) |
| Composite | union of holidays | [`calendar::composite`](src/calendar) |
| JointBusiness | intersection of business days | [`calendar::composite`](src/calendar) |

TARGET2 and Luxembourg are *fixed-rule* generators (every date derived from
the year alone — Easter via the Computus, plus a small set of fixed civil
dates); every other calendar ships a dated 2020–2040 snapshot of the
exchange's or central bank's published holiday list cross-verified
row-by-row against the primary published source. Snapshot dates and the
gazetted-vs-derived horizon for each calendar are recorded per file and in
[SPEC.md](SPEC.md). Outside the snapshot window, `is_holiday` returns
`false` rather than panicking.

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
    luxembourg.rs       # Loi du 21 juillet 1928 fixed rule
    united_states.rs    # NYSE / Federal Reserve table
    united_kingdom.rs   # Bank of England table
    japan.rs            # JPX table
    switzerland.rs      # SIX table
    hong_kong.rs        # HKEX table
    singapore.rs        # SGX table
    composite.rs        # Composite / JointBusiness combinators
```

One file, one convention. Every type is `Copy`, allocation-free, and
validated on construction.

## Testing

```bash
cargo test                      # unit + integration + doc-tests
cargo run --example quickstart  # end-to-end tour
cargo bench                     # criterion benchmarks
```

Tests are anchored on **the ISDA 2006 worked examples** (the Act/Act
straddle reference, the 30/360 BondBasis reference, the 30E/360 ISDA
maturity-day specials), the **ICMA Rule 251 examples**, and the published
**QuantLib daycounters** cross-oracle. Holiday tables are spot-checked
against the official published calendars year by year.

## Code quality

- `#![no_std]`, allocation-free — runs in services, WASM, and on embedded
  targets with no `std`
- `#![forbid(unsafe_code)]` crate-wide
- `clippy::pedantic` with zero warnings
- No `unwrap()`, `expect()`, or `panic!()` in library code — every failure
  path is a typed `Result`
- Every public item documented with its governing standard and a runnable
  example
- Deterministic: the same input always produces the same verdict

## Dependencies

**Runtime: zero.** Not `std`, not `alloc`, no FFI. Every day-count fraction,
every date-roll convention, and every calendar lookup is hand-rolled from
its governing standard. Licence and supply-chain policy is enforced via
`cargo-deny` (`deny.toml`).

The default `calendars` feature embeds dated holiday-table snapshots as
static, `no_std`-clean data. Disable it for a structural-only build (date
primitives, day-count fractions, date-roll conventions; TARGET2 still works
because it is rule-based, the other calendars become unavailable):
`cargo build --no-default-features`.

## Standards

All algorithms implemented from their governing standard — no ports from
other implementations.

| Standard | Convention |
|---|---|
| ISDA 2006 Definitions §4.16 | Act/360, Act/365F, ActAct ISDA, 30/360, 30E/360, 30E/360 ISDA |
| ICMA Rule 251 | ActAct ICMA, Act/365L |
| ISO 8601 | Date format, weekday rule |
| ECB TARGET2 closing days regulation | TARGET2 calendar |
| Loi du 21 juillet 1928 / Loi du 28 février 2019 (Luxembourg) | Luxembourg calendar |
| NYSE / Federal Reserve published schedule | United States calendar |
| Bank of England bank holiday schedule | United Kingdom calendar |
| JPX / National Holidays Act (Japan) | Japan calendar |
| SIX Swiss Exchange published schedule | Switzerland calendar |
| HKEX published schedule | Hong Kong calendar |
| SGX published schedule | Singapore calendar |

## Documentation

- [SPEC.md](SPEC.md) — every day-count fraction, date-roll convention, and
  calendar rule traced to its standard, with worked examples
- [CHANGELOG.md](CHANGELOG.md) — release history
- [SECURITY.md](SECURITY.md) — vulnerability disclosure policy

## License

Apache License 2.0. See [LICENSE](LICENSE) and [NOTICE](NOTICE).

```
Copyright 2026 Regit.io — Nicolas Koenig
```

---

Part of [Regit OS](https://www.regit.io) — the operating system for investment products. From Luxembourg.
