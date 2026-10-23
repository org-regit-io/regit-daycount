# Calculation contract

This specification defines the supported behavior. Detailed algorithms, source
citations and examples live beside the implementation in `src/`.

## Date domain

`Date` is a private-field Gregorian date in `1583..=9999`. `Date::ymd` validates
year, month and day and is usable in const contexts. External callers cannot
construct a date without validation. Ordering is chronological.

`try_add_days` adds signed calendar days. `try_add_months_eom_aware` adds signed
calendar months, clamping the original day to the target month's last day.
Despite its historical name, it does not preserve month-end when the target
month has more days. Both return errors when the result leaves the date domain,
including extreme `i32` offsets. `days_between` returns the exact signed integer
difference; all valid endpoint pairs fit `i32`.

`try_easter_sunday` uses the Gregorian Meeus/Butcher computus and validates its
year. `nth_weekday_of_month` validates the year, month, occurrence and existence
of the requested weekday. Weekdays use the Gregorian calendar throughout the
supported domain, without modeling historical national calendar adoption.

References: ISO 8601; Howard Hinnant's Gregorian civil-date algorithms; Meeus,
*Astronomical Algorithms*, Gregorian Easter computus.

## Financial conventions

The dispatcher is `day_count::try_fraction(start, end, basis, context)`.
Missing or invalid context returns a typed error. Primitive convention functions
that require only two validated dates return `f64` directly. No API uses NaN to
signal missing context or invalid data.

| Basis | Definition |
| --- | --- |
| Act/360 | Actual signed days / 360 |
| Act/365F | Actual signed days / 365 |
| Act/Act ISDA | Split at calendar-year boundaries; days in each year divided by 365 or 366 |
| Act/Act ICMA | Signed accrual days / (frequency × reference coupon days), for one supplied regular/notional reference period |
| 30/360 Bond Basis | ISDA endpoint rules: start 31→30; end 31→30 only when adjusted start is 30 |
| 30E/360 | Both endpoint day numbers 31→30 |
| 30E/360 ISDA | Month-end endpoint adjustments; February end at maturity is exempt from the second endpoint adjustment |
| Act/365L | Annual: denominator 366 if `(accrual_start, next_coupon]` includes February 29; nonannual: denominator follows the next coupon's calendar year |
| NL/365 | QuantLib NoLeap serial-day difference / 365; leap-day removal follows `(start, end]` |
| Bus/252 | Signed business-day count over the normalized `[lower, upper)` interval / 252 |
| 1/1 | Always one |

For 30-day conventions, the numerator is
`360 × (end_year − start_year) + 30 × (end_month − start_month) + adjusted_end_day − adjusted_start_day`.

ICMA and Act/365L require a positive frequency and an ordered reference period
containing both accrual endpoints. The caller owns the contractual schedule.
ICMA does not infer or split stubs spanning multiple reference periods. Reversed
accruals retain a signed numerator and use an ordered reference period.

Actual-day conventions are antisymmetric. Endpoint-adjusted 30/360 conventions
need not be; equal February maturity endpoints under 30E/360 ISDA may be nonzero.
The 1/1 shortcut is constant even for equal or reversed dates.

Bus/252 with a caller predicate uses the caller's market definition. Use the
checked calendar wrapper to propagate missing-calendar-data errors. BrazilB3
means B3 listed-market full-day trading closures, not a universal Brazilian
settlement calendar.

References: ISDA 2006 Definitions §4.16; ICMA Rule 251; ANBIMA/B3 business-day
conventions. Each implementation cites its applicable rule and precise scope.

## Date rolls

`roll::try_apply` applies a convention using a caller-owned business-day predicate.
It returns a date-boundary or search-limit error rather than a partial answer.
Named-calendar rolls additionally validate coverage on every examined date.

| Convention | Behavior |
| --- | --- |
| Unadjusted | Return input unchanged |
| Following / Preceding | Search on or after / on or before the input |
| Modified Following | Following unless it crosses the month, then Preceding |
| Modified Preceding | Preceding unless it crosses the month, then Following |
| Nearest | Minimum calendar-day distance; ties choose Following |
| EndOfMonth | Standalone-date compatibility contract: Modified Following |

`EndOfMonth` does not generate a schedule or advance to the last business day.
Calendar search bounds and coverage checks are part of the API contract; a
successful adjustment never certifies a date outside the selected policy.

Reference: ISDA 2006 Definitions §4.12; explicit standalone EndOfMonth contract.

## Calendar data

A business day is neither Saturday/Sunday nor a full-day closure of the selected
market. Holiday queries report the market's named/observed holiday set, which can
include nominal Saturday holidays in some calendars. Half-day sessions remain
business days. Trading and banking calendars are separate markets.

`calendar::coverage` exposes available and verified years plus a per-calendar
verification timestamp. Every default checked query uses `VerifiedOnly`.
`AllowProjected` accepts available unverified years, including history; it does
not extend the available range. Independent-library agreement is not official
verification. Complete primary-source annual audits are retained in the calendar
modules. TARGET2's verified standing rule has its own effective-date scope.

Every date examined by an adjustment, count or walk must satisfy the selected
coverage policy. Nonempty counts require coverage of every counted date in `[start, end)`;
the exclusive end need not have calendar data. Empty or reversed business-day
count intervals return zero after validating both endpoints. Bus/252 normalizes
reversed intervals and negates the forward count.
Zero business-day additions roll Following; nonzero additions exclude the start.
Walk caps return errors and cannot produce partial financial results.

`Composite` and `JointBusiness` both intersect constituent business days and
unite their holidays. Constructors reject empty constituents. Every leg is
checked, including after another leg has already determined a closure.

## Public compatibility

Extensible convention, market, policy and error enums are non-exhaustive.
`Weekday` is the closed seven-day set. No public numeric casts are promised as a
serialization format. Version 2.0 removes permissive legacy APIs; the migration
map is in README.md. Subsequent compatible releases are checked against the
latest released public API.

## Verification scope

`uv run tests/run.py --coverage` launches independent Python reference
calculations and Rust computations as separate processes, then compares their
outputs. Python domain modules live in `tests/modules/`; `tests/test.rs` builds
the standalone Rust executable outside the production library. It receives test
inputs and returns observations without Python expectations. Neither computation
process consumes the other's implementation or results. The launcher retains
generated observations, source provenance and `report.json` under ignored
`target/test-results/`; no expected fixtures are committed.

Financial references execute pinned QuantLib and FinancePy. Calendar references
execute Python holidays and QuantLib for the named market; date references use
Python datetime/calendar, python-dateutil and convertdate. Explicit Python
contract models supply expected errors, context validation, coverage policies,
walk limits, date boundaries, metadata and trait behavior. Compiler probes check
contracts that cannot be represented as runtime values. A compiler-derived API
inventory requires evidence for the public surface in both feature
configurations, and a temporary added public function verifies that the inventory
gate rejects an uncovered addition.

Third-party financial comparisons span 1901–2199; Gregorian primitive references
span 1583–9999. Established calendar snapshots cover 2020–2040 and B3 covers
2026; rule-calendar boundaries are checked independently beyond those snapshots.
ICMA comparisons stay within one regular/notional reference coupon period.
Signed Bus/252 orders the endpoints and negates the forward result, preserving
the crate's half-open interval rather than QuantLib's raw reversed-interval
rule. Projected library agreement does not certify an official annual calendar.

Date, flag, integer and error observations compare exactly. Floating results
must be finite and differ by no more than the larger of `1e-12` and two ULPs of
the expected value. Production coverage is measured from the independently
generated inputs; the report records the measured scope rather than inferring
it from a fixture count. The 2.0 release run reached 100% of instrumented
production functions and 98.7% of production lines; it does not establish
exhaustive branch or path coverage.

Python dependency pins are shared by `tests/requirements.txt` and PEP 723
metadata in the entry point. The 2.0 references run with QuantLib 1.44,
FinancePy 1.1.2, python-dateutil 3.0.0, convertdate 2.5.1, holidays 0.107,
PyMeeus 0.5.12, six 1.17.0, NumPy 2.5.4, Numba 0.68.0, llvmlite 0.50.0 and
SciPy 1.18.1, checked against current stable releases on 2026-10-23. The uv
entry point explicitly overrides older dependency bounds declared by FinancePy
and holidays; unused pandas and Matplotlib are excluded. The original FinancePy
reference source hashes remain checked, and the complete comparison is run in
this overridden environment. These development packages are not Rust runtime
dependencies or part of the published archive.
