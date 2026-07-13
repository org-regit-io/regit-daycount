<!-- Copyright 2026 Regit.io — Nicolas Koenig -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.1] - 2026-07-13

### Changed

- Bumped the `criterion` dev/bench dependency to 0.8. This is a dev-only
  and bench-only dependency; there are no runtime or public API changes.

## [1.0.0] - 2026-05-23

First public release. The crate is `#![no_std]`, allocation-free, and has
zero runtime dependencies; every day-count fraction, date-roll convention,
and calendar lookup is hand-rolled from its governing standard and traced
in [SPEC.md](SPEC.md).

### Added — date primitives (`date`, `errors`)

- `Date` — Gregorian `(year, month, day)` triple over the supported window
  `1583..=9999`, validated on construction. `Copy`, 6 bytes (i32 year,
  two u8 fields), allocation-free. Accessors `year` / `month` / `day`,
  classification helpers `is_leap_year`, `days_in_month`, weekday via
  `day_of_week` (Hinnant `days_from_civil` modulo 7), signed arithmetic
  `days_between` / `add_days`, month-stepping `add_months_eom_aware` with
  end-of-month clamping, structured anchors `nth_weekday_of_month` and
  `easter_sunday` (Meeus / Butcher / Anonymous-Gregorian Computus).
- `Weekday` — Monday-first ISO 8601 ordering, `Copy` enum.
- `ValidationError` — single `Copy` enum carrying `InvalidDate { rule }`
  and `OutOfRange { what }`; implements `core::fmt::Display` and
  `core::error::Error`; chains through `?` even under `#![no_std]`.

### Added — day-count fractions (`day_count`)

- `Act/360` — ISDA 2006 §4.16(e). Money-market default; constant 360
  denominator; signed `days_between` numerator.
- `Act/365F` — ISDA 2006 §4.16(d). Sterling-bloc money-market default;
  constant 365 denominator.
- `ActAct ISDA` — ISDA 2006 §4.16(b). Straddle-aware: splits at every
  1 January, divides each per-year sub-interval by 366 or 365 according
  to that year's leap-year status, sums the contributions. Same-year
  fast path.
- `ActAct ICMA` — ICMA Rule 251 (regular-period case). Takes
  `(start, end, ref_start, ref_end, freq)`; returns `0.0` defensively on
  zero `freq` or degenerate reference period rather than emit a silent
  `NaN`. Irregular-period (split-into-sub-periods) variant deliberately
  deferred to a future revision.
- `30/360 BondBasis` — ISDA 2006 §4.16(f). Endpoint adjustments
  `D1 = 31 → 30` and `D2 = 31 → 30` (the latter conditional on `D1`
  being 30 or 31), then `360 * Δy + 30 * Δm + Δd` over 360.
- `30E/360` — ISDA 2006 §4.16(g). Like Bond Basis but the `D2 = 31 → 30`
  adjustment is unconditional; the 1/360 disagreement with Bond Basis on
  `D2 = 31`-only intervals is pinned by test.
- `30E/360 ISDA` — ISDA 2006 §4.16(h). Last-day-of-month-aware adjustments
  with the February-at-maturity suppression carve-out — the entry point
  takes an `end_is_maturity` flag.
- `Act/365L` — ICMA / sterling money-market. Numerator is signed
  `days_between`; denominator is 366 if any 29 February falls inside the
  half-open period and 365 otherwise. Orientation-symmetric.
- `NL/365` — conventional "No-Leap". Numerator subtracts 29-February dates
  in the half-open interval from the actual day count; denominator is 365.
- `Bus/252` — Brazilian / B3 / ANBIMA convention. Takes a caller-supplied
  `is_business_day: impl Fn(Date) -> bool` predicate to break the
  `day_count → calendar` import cycle. Counts business days in
  `[start, end)`; denominator is 252.
- `1/1` — OIS shortcut. Returns `1.0` for every interval, including the
  degenerate same-day and inverted cases; the placeholder on compounded-
  rate legs.
- `DayCount` — dispatcher enum, with two free functions:
  - `fraction(start, end, basis)` — common entry; returns `NaN` for the
    two variants that need extra arguments (`ActActIcma` needs a reference
    period; `Bus252` needs a predicate). `NaN` propagates loudly through
    arithmetic rather than silently absorbing as `0.0`.
  - `year_fraction_with_freq(start, end, basis, freq, ref_start, ref_end)`
    — routes `ActActIcma` to its full five-argument form; delegates the
    other variants to `fraction`.

### Added — date-roll conventions (`roll`)

- `Roll` — `Copy` enum with seven variants: `Unadjusted`, `Following`,
  `ModifiedFollowing` (ISDA 2006 §4.12 — falls back when crossing a month
  boundary), `Preceding`, `ModifiedPreceding`, `Nearest` (ties to
  Following), `EndOfMonth` (anchors to the last business day of the input
  month when the input is itself the last business day of its month;
  falls back to `ModifiedFollowing` otherwise).
- `roll::apply(date, conv, is_business_day)` — single free function taking
  a caller-supplied business-day predicate (the same cycle-breaking shape
  Bus/252 uses). Defensively bounded at 366 one-day steps in either
  direction; a pathological predicate returns the input unchanged.

### Added — holiday calendars (`calendar`, default feature `calendars`)

- `Calendar` — `Copy` enum naming eight primary calendars (two
  rule-based, six dated-snapshot).
- `Target2` — ECB fixed rule. Six dates a year derived from the year alone
  (Easter via the Computus); rule-exact for every year in the supported
  `Date` range. New Year's Day, Good Friday, Easter Monday, Labour Day,
  Christmas Day, Christmas Holiday. Whit Monday is not a TARGET2 closing
  day post-2002.
- `Luxembourg` — labour-law fixed rule (Loi du 21 juillet 1928 plus the
  Loi du 28 février 2019 that added Europe Day). Eleven dates a year:
  Jan 1, Easter Monday, May 1, May 9 (Europe Day, from 2019), Ascension,
  Whit Monday, Jun 23 (National Day), Aug 15 (Assumption), Nov 1 (All
  Saints), Dec 25, Dec 26. Rule-exact, no table. The natural pairing
  with TARGET2 for Luxembourg-domiciled UCITS / SIF NAV schedules
  (compose them via `JointBusiness`).
- `UnitedStates` — NYSE / Federal Reserve embedded snapshot. 208 observed
  holiday rows over 2020–2040, including Juneteenth from 2022 and every
  weekend-shift substitute. Discretionary closures (Hurricane Sandy, state
  funerals) are out of scope and documented as such.
- `UnitedKingdom` — Bank of England (England & Wales) embedded snapshot.
  171 rows, including the **2020-05-08 VE Day 75th-anniversary**
  displaced Early May Bank Holiday, the 2022 Platinum Jubilee displaced
  Spring Bank Holiday, the extra 3 June 2022 Jubilee day, the 19
  September 2022 Elizabeth II state funeral, and the 8 May 2023
  Coronation of King Charles III.
- `Switzerland` — SIX Swiss Exchange embedded snapshot. 210 rows over
  2020–2040 (ten holidays per year; no weekend-observance shifts).
- `Japan` — JPX / National Holidays Act embedded snapshot. 432 rows
  covering the Cabinet-Office-gazetted equinoxes for 2020–2027 and the
  Aoki/NAOJ-astronomical-inferred equinoxes for 2028–2040 (every row
  annotated with its provenance), the Happy-Monday rule, the *furikae
  kyūjitsu* substitutes, the *kokumin no shukujitsu* People's-Holiday
  rule (firing in 2026, 2032, 2037 within coverage), and the Olympic-
  year moves in 2020 and 2021.
- `HongKong` — HKEX embedded snapshot. 332 rows including the lunar New
  Year (three days), Buddha's Birthday, Tuen Ng, Day-after-Mid-Autumn,
  Chung Yeung, plus the Easter triad and the fixed civil dates. Lunar
  dates 2020–2027 are gazetted; 2028–2040 are HKO-conversion-derived
  with the leap-month structure (intercalary 2028 / 2031 / 2033 / 2036
  / 2039) documented inline.
- `Singapore` — SGX embedded snapshot. 230 rows covering the
  multi-religious public-holiday schedule: Chinese New Year (two days),
  Vesak, Hari Raya Puasa, Hari Raya Haji, Deepavali, plus Good Friday,
  Christmas, and the civil dates with Holidays-Act-§4 Sunday-only
  in-lieu observance (Saturday-falling holidays NOT shifted). 2020–2026
  rows are MOM-gazetted; 2027–2040 are projected (Hijri via Umm
  al-Qura ±MUIS-margin; lunar via HKO; Hindu via Kartik-Amavasya).
- `Composite` — union of holidays across a `&'static [Calendar]`.
- `JointBusiness` — intersection of business days across a
  `&'static [Calendar]` (the cross-currency-settlement framing). Both
  combinators are `Copy`, allocation-free, and reduce to the underlying
  calendar dispatcher.
- Free functions: `is_weekend`, `is_holiday`, `is_business_day`, `adjust`
  (closes `roll::apply` over a `Calendar`), `next_business_day` and
  `previous_business_day` (ergonomic wrappers around the Following /
  Preceding rolls), `add_business_days` (the T+N settlement primitive
  — signed `n`, walks one calendar day at a time counting business days
  against `n`, defensively bounded at ~200 years),
  `business_days_between` (half-open count, defensively bounded at
  ~200 years), `bus_252_for_calendar` (closes `bus_252::fraction` over
  a `Calendar`).
- Each per-snapshot calendar submodule exposes `SNAPSHOT_DATE`,
  `COVERAGE`, and the sorted `HOLIDAYS` slice. Out-of-coverage queries
  return `false` without panicking.

### Verification (load-bearing trust step)

Every embedded snapshot calendar (US, UK, JP, CH, HK, SG) was
cross-verified row-by-row on 2026-05-23 against its primary published
source. The verification pass caught and fixed several real defects
shipped by the initial generation pass:

- **UK** — `(2020, 5, 4)` corrected to `(2020, 5, 8)`: the 2020 Early
  May Bank Holiday was moved by UK government to Friday 8 May for the
  75th anniversary of VE Day. The original row was a missed dated
  special.
- **JP** — `(2039, 3, 20)` corrected to `(2039, 3, 21)`: the 2039
  vernal equinox per the Aoki/NAOJ astronomical formula is Monday
  21 March, not Sunday 20 March. The spurious Sunday-substitute row
  was removed. Total row count corrected 433 → 432.
- **HK** — Buddha's Birthday `(2030, 5, 10)` corrected to
  `(2030, 5, 9)`; Buddha's Birthday `(2031, 5, 29)` corrected to
  `(2031, 5, 28)`. Both off-by-one lunar-conversion errors.
- **SG** — sixteen corrections, including six Saturday-falling
  holidays that were incorrectly Monday-shifted (the Holidays Act §4
  shifts Sunday-falling holidays only, not Saturday); the 2026
  Deepavali Sun → Mon in-lieu; the 2030 CNY-day-2 / Hari Raya Puasa
  collision; and the 2033 New Year's Day on Saturday re-added.
- **US** and **CH** — zero corrections; the original tables matched
  their rule derivation exactly. Both files gained a permanent
  regression test that re-derives every row from the documented rules
  on each run.

After the verification pass, ZERO `// 1.x VERIFY` markers remain in
the crate. The JP, HK, and SG tables additionally carry inline
row-by-row provenance annotations distinguishing gazetted dates from
astronomically- or rule-derived projections.

### Added — examples and benchmarks

- `examples/quickstart.rs` — guided eight-section tour: every `Date`
  primitive, the eleven day-count fractions side-by-side on the same
  interval, the seven date-roll conventions on a single Saturday, the
  seven named calendars on a known closing date each, `adjust` and
  `business_days_between` under TARGET2, a `JointBusiness` EUR/USD
  settlement filter, and a worked accrued-interest computation on a
  hypothetical 5.00% Act/360 EUR bond.
- `benches/daycount.rs` — Criterion benchmarks for the date primitives,
  every day-count dispatcher arm (incl. `ActActIcma`), TARGET2 / US /
  Japan / HK calendar lookups, every roll convention, and the Bus/252
  fortnight walk. Indicative targets: sub-30 ns fraction, sub-200 ns
  calendar lookup, sub-500 ns calendar-aware roll.

### Crate metadata

- `edition = "2024"`, MSRV `1.85`, pinned toolchain `1.95.0`.
- `#![no_std]`, `#![forbid(unsafe_code)]`, no `alloc`.
- `clippy::pedantic` clean across the whole workspace and all targets.
- Builds for `wasm32-unknown-unknown` and `thumbv7em-none-eabi` (a target
  with no `std` at all — proof of `no_std`-ness).
- Zero runtime dependencies; licence and supply-chain policy enforced via
  `cargo-deny` (`deny.toml`).
- Default feature `calendars` embeds the eight calendar layer
  (TARGET2 and Luxembourg as rule generators; the six dated-snapshot
  calendars hold 1,583 holiday rows over 2020–2040 inclusive). Disable
  it (`cargo build --no-default-features`) for a minimal build that
  exposes only the date primitives, day-count fractions, and date-roll
  conventions.

### Numbers

- **371 tests** with default features (238 unit + 74 integration +
  59 doctests); **239 tests** with `--no-default-features` (148 + 59 +
  32). All seven QuantLib v1.34 `daycounters.cpp` cross-oracle vectors
  that apply to this crate's catalogue pass at the documented tolerance.
- **1,583 embedded holiday rows** across US / UK / CH / JP / HK / SG
  spanning 2020–2040 inclusive; TARGET2 and Luxembourg are fully
  rule-based.
- **~10,000 lines** of library Rust (`src/`), **~1,800 lines** of
  example, benchmark, and integration tests.

[1.0.0]: https://github.com/org-regit-io/regit-daycount/releases/tag/v1.0.0
