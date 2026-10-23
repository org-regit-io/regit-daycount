# regit-daycount

Financial day-count fractions, market calendars and date rolls in Rust.
`no_std`, no allocation, no runtime dependencies. Requires Rust 1.85 or later.

```toml
[dependencies]
regit-daycount = "2"
```

```rust
use regit_daycount::{Date, DayCount};
use regit_daycount::day_count::{self, CouponPeriod, DayCountContext};

fn main() -> Result<(), regit_daycount::day_count::DayCountError> {
    let start = Date::ymd(2026, 3, 1).unwrap();
    let end = Date::ymd(2026, 9, 1).unwrap();
    let context = DayCountContext {
        coupon: Some(CouponPeriod { start, end, frequency: 2 }),
        ..DayCountContext::default()
    };
    let fraction = day_count::try_fraction(start, end, DayCount::ActActIcma, context)?;
    assert_eq!(fraction, 0.5);
    Ok(())
}
```

## Public contract

- `Date::ymd` validates Gregorian dates in `1583..=9999`, including in const contexts.
- `try_add_days`, `try_add_months_eom_aware` and `try_easter_sunday` return range errors. Safe public methods cannot create out-of-domain dates.
- `day_count::try_fraction` requires explicit context where the convention needs it. Missing or invalid context returns `DayCountError`.
- Calendar queries, rolls and walks return `CalendarError` for unavailable data, unverified years, date boundaries or exhausted walk limits. They never return partial results.
- `roll::try_apply` supports caller-owned business-day predicates and returns `RollError` for date boundaries or exhausted search limits.
- Extensible public enums are `#[non_exhaustive]`; downstream matches need a fallback arm. `Weekday` is closed.

There are no public NaN error wrappers, unchecked date constructors or permissive calendar lookups.

## Day-count conventions

| Convention | Required context |
| --- | --- |
| Act/360, Act/365F, Act/Act ISDA | Dates |
| 30/360 Bond Basis, 30E/360 | Dates |
| 30E/360 ISDA | Explicit `end_is_maturity` |
| Act/365L | Ordered reference coupon and frequency |
| Act/Act ICMA | One regular/notional reference coupon containing the accrual, and frequency |
| NL/365 | Dates; leap-day treatment follows QuantLib NoLeap |
| Bus/252 | A named calendar or caller-owned business-day predicate |
| 1/1 | Constant one, including equal and reversed dates |

ICMA does not infer schedules or split multi-period stubs. Supply the contractual
reference periods and sum their contained accruals. Bond Basis is the ISDA
30/360 algorithm; it must not be substituted for a contract specifying US/NASD.
See [SPEC.md](SPEC.md) and the module rustdoc for formulas and standards.

## Calendars

The default `calendars` feature includes TARGET2, Luxembourg, explicit NYSE and
Federal Reserve calendars, UK (England and Wales), Japan (JPX), Switzerland
(SIX), Hong Kong (HKEX), Singapore and Brazil (B3 listed-market trading).
Calendar names identify different market operations: trading closures are not
interchangeable with banking settlement holidays. Partial sessions remain
business days in this date-only model.

```rust
use regit_daycount::{Calendar, Date, Roll};
use regit_daycount::calendar::{self, JointBusiness};

fn main() -> Result<(), regit_daycount::calendar::CalendarError> {
    let start = Date::ymd(2026, 9, 1).unwrap();
    let end = Date::ymd(2026, 9, 10).unwrap();
    assert_eq!(calendar::try_business_days_between(start, end, Calendar::BrazilB3)?, 6);

    let joint = JointBusiness::new(&[Calendar::Target2, Calendar::UnitedStatesNyse])?;
    let payment = joint.try_adjust(Date::ymd(2026, 4, 3).unwrap(), Roll::Following)?;
    assert_eq!(payment, Date::ymd(2026, 4, 7).unwrap());
    Ok(())
}
```

| Calendar | Available years | Verified years |
| --- | --- | --- |
| TARGET2 | 2002–9999 | 2002–2028, standing rule |
| Luxembourg | 2019–9999 | 2026–2027 |
| UK (England and Wales) | 2020–2040 | 2020–2028 |
| NYSE | 2020–2040 | 2026–2028 |
| Federal Reserve | 2020–2040 | 2026–2030 |
| JPX, SIX, HKEX, Singapore | 2020–2040 | 2026–2027 |
| Brazil B3 | 2026 | 2026 |

`calendar::coverage(calendar)` returns the available years, independently
primary-source verified years, and that calendar's verification date. Checked
operations use `CoveragePolicy::VerifiedOnly`. `AllowProjected` explicitly accepts
available unaudited historical rows and future projections; it never extends
availability. Combinations require at least one calendar and validate every leg.
Sources and complete audited annual closure lists are retained with the relevant
calendar module.

Disable embedded calendars with `default-features = false`; date primitives,
day counts and caller-owned date rolls remain available.

## Migrating from 1.x

Version 2.0 intentionally changes the public contract:

| Removed 1.x API | 2.0 replacement |
| --- | --- |
| `day_count::fraction`, frequency dispatchers | `day_count::try_fraction` with `DayCountContext` |
| `act_365l::fraction`, `act_act_icma::fraction` | Their explicit-context `try_fraction` functions |
| Boolean/date-returning calendar functions | Calendar `try_*` functions, optionally with an explicit coverage policy |
| `Calendar::UnitedStates` | `UnitedStatesNyse` or `UnitedStatesFederalReserve` |
| `Date::ymd_unchecked` | Const-capable `Date::ymd` |
| `Date::add_days`, `add_months_eom_aware`, `easter_sunday` | Their fallible `try_*` counterparts |
| `roll::apply` | `roll::try_apply` |
| Infallible composite constructors | `Composite::new` / `JointBusiness::new` returning `Result` |

Handle returned errors explicitly. Matching extensible enums now requires a
fallback arm. Infallible primitive day-count functions retain their numeric
signatures because validated dates make their results well-defined.

## Development

Production modules live in `src/`. Run the independent validation through one
entry point, `tests/run.py`; its Python modules live in `tests/modules/` and
follow the library's date, roll, day-count and calendar domains.

```sh
cargo test --locked
cargo test --locked --no-default-features
cargo clippy --all-targets --all-features --locked -- -D warnings
uv run tests/run.py --coverage
```

The launcher runs Python references and the standalone `tests/test.rs` executable
in separate processes, then compares their outputs for both feature
configurations. Reference calculations use independent libraries and explicit
public contracts; they do not read Rust implementations or results. Generated
evidence is written under
`target/test-results/`, including `report.json`; expected results are not committed.

Use Python 3.12–3.13, Rust 1.95.0 for the API inventory, and
`cargo-llvm-cov` 0.8.7 with the pinned toolchain's `llvm-tools` component.
Python dependencies are pinned in `tests/requirements.txt` and the entry point's
PEP 723 metadata. The entry point explicitly overrides FinancePy and holidays
dependency bounds to run their unchanged references against the latest pinned
libraries. Its uv configuration excludes unused pandas and Matplotlib.
The 2.0 release run exercised all instrumented production functions and 98.7%
of production lines. This is measured execution coverage, not proof of every
possible path. The [verification scope](SPEC.md#verification-scope) describes reference
limits; library agreement does not establish primary-source calendar verification.
Development tools and generated evidence are excluded from the crates.io archive.

CI checks financial regressions, public API compilation, compatibility with the
latest release, minimum Rust, WASM and an embedded target without `std`.

Apache-2.0. [Regit](https://www.regit.io).
