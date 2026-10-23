# Executed crate oracle

`uv run validation/oracle.py --check --rust` executes the pinned reference
libraries, regenerates expected results in memory, verifies the committed
fixtures, and compares the Rust crate at `1e-12` absolute tolerance.

The gate covers every advertised calculation family:

| Capability | Independent reference cases |
| --- | ---: |
| All 11 `DayCount` variants | 11,936 |
| All 9 unique calendars plus the legacy US alias, every 2020–2040 date for both holidays and business days, plus primary-source facts | 153,457 |
| All 7 rolls, across every named calendar and weekends only | 29,799 |
| All 13 public `Date` methods | 194,197 |
| Business-day operations and both composite calendar types | 87,876 |
| Total | 477,265 |

The generator inventories the public `DayCount`, `Calendar` and `Roll` enums and
`Date` methods from Rust source, using signatures only. It compares that inventory
with actual generated cases and fails on missing capabilities. It also inventories
calendar/composite financial operations and requires complete daily calendar
coverage. Rust's exhaustive enum mappings provide a second compile-time check.
Unknown fixture kinds fail; minimal builds skip only calendar-feature cases.
Expected financial results never come from Rust source.

```sh
# Generate fixtures using independently executed reference libraries.
uv run validation/oracle.py

# Run the complete gate without changing committed fixtures.
uv run validation/oracle.py --check --rust

# Consume committed results without Python, reference libraries or network.
cargo test --test oracle_fixtures
cargo test --test oracle_fixtures --no-default-features
```

The oracle runs through Python 3.12–3.13, using QuantLib 1.44, FinancePy 1.1.2,
holidays 0.107, python-dateutil 2.9.0.post0 and convertdate 2.5.1. All reference
packages and transitive dependencies are pinned in `requirements.txt` and the
script's PEP 723 metadata; the gate verifies both lists agree and checks installed
versions. FinancePy financial source files are checked against their pinned
SHA-256 hashes. The first run installs the development packages through `uv`;
subsequent runs reuse its cache. Missing references and incomplete horizons fail
the gate.

Pins use the latest mutually compatible stable releases, checked against PyPI on
23 October 2026. FinancePy 1.1.2 requires NumPy `<2.4`, SciPy `<1.17`, pandas
`<=2.4`, matplotlib `<3.11`, Numba `<0.63` and llvmlite `<0.46`; holidays 0.107
requires python-dateutil `<3`. These upstream bounds explain the seven pins below
their unconstrained latest releases. FinancePy also requires Python `<3.14`.

| Rust day count | Executed reference |
| --- | --- |
| `Act360` | QuantLib `Actual360()` |
| `Act365F` | QuantLib `Actual365Fixed()` |
| `ActActIsda` | QuantLib `ActualActual(ISDA)` |
| `ActActIcma` | QuantLib `ActualActual(ISMA)` with a reference coupon |
| `Thirty360BondBasis` | QuantLib `Thirty360(BondBasis)` |
| `ThirtyE360` | QuantLib `Thirty360(European)` |
| `ThirtyE360Isda` | QuantLib `Thirty360(ISDA, terminationDate)`, both maturity flags |
| `Act365L` | FinancePy `DayCount(ACT_365L).year_frac` with next coupon and frequency |
| `Nl365` | QuantLib `Actual365Fixed(NoLeap)` |
| `Bus252` | QuantLib `Business252(TARGET)`, with explicit signed interval mapping |
| `OneOne` | QuantLib `OneDayCounter` with explicitly ordered reference inputs |

Day-count samples include leap, century, month-end and year-end endpoints,
reversed/equal intervals, and deterministic random intervals. ICMA cases supply
regular reference coupons at six frequencies. Act/365L covers annual and
nonannual frequencies 1, 2, 3, 4, 6 and 12, full/partial coupons, leap endpoints,
reversed/equal accruals and random periods. FinancePy exposes annual versus
nonannual denominator classes for Act/365L, without a bimonthly enum: the adapter
selects its nonannual class for six coupons/year while preserving the contractual
next coupon and fixture frequency. Both interval orientations execute an ordered
reference call, negating the result for reversed accruals. For 1/1, only reference
inputs are ordered to execute its constant basis; Rust receives the original
endpoints, including reversals. Bus/252 explicitly maps QuantLib's ordered call
to the signed lower-inclusive, upper-exclusive interval contract. Both raw Rust
functions and checked calendar wrappers are compared.

Calendar references execute each market's exact financial calendar in the pinned
holidays library, with additional QuantLib comparisons where its calendar matches
the market and full horizon. The helper checks horizon bounds and every annual
holiday population. Raw named-holiday mappings distinguish weekends from holidays
and preserve market-specific observation rules. SIX trading closures are distinct
from Swiss banking settlement; Hong Kong/Singapore projections use complete
reference algorithms rather than truncated QuantLib lunar tables. Source hashes,
market mappings, published facts and projection assumptions appear in the manifest.
Future dates remain explicitly projected even when independent libraries agree.

Date references execute Python `datetime`/`calendar`, dateutil month and weekday
arithmetic, and convertdate's independent Western Easter implementation. Every
supported year 1583–9999 is checked for leap years, every month length, Easter and
weekday anchors, with selected/random constructor and arithmetic cases. Rolls and
business-day operations execute QuantLib algorithms over the independent market
holiday data. Standalone `EndOfMonth` maps to zero-month EOM advance; joint
calendars execute native QuantLib `JointCalendar`, covering empty, single-leg and
multi-market combinations for both `Composite` and `JointBusiness`.

`oracle.json` records per-capability counts, source inventories, exact method
mappings, pinned packages and artifacts, reference source hashes, the seed and the
fixture checksum. `oracle.tsv` stores expected results for offline Rust tests;
The production crate remains dependency free, `no_std` and allocation free.
