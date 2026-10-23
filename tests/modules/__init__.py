"""Independent Python reference process: inputs and expected observations.

Only tests/run.py launches and compares implementations. This package never
calls Rust, reads Rust implementation tables, or consumes Rust results.
"""

import collections
import datetime as dt
import hashlib
from importlib.metadata import version
import json
from pathlib import Path
import random
import tomllib



ROOT = Path(__file__).resolve().parents[2]
SEED = 20261023
QL_VERSION = "1.44"
COLUMNS = ["id", "kind", "convention", "start", "end", "ref_start", "ref_end", "frequency", "maturity", "expected", "source"]
DAY_COUNTS = {
    "act360": "Act360", "act365f": "Act365F", "act365l": "Act365L",
    "actactisda": "ActActIsda", "actacticma": "ActActIcma", "nl365": "Nl365",
    "thirty360bond": "Thirty360BondBasis", "thirtye360": "ThirtyE360",
    "thirtye360isda": "ThirtyE360Isda", "oneone": "OneOne", "bus252": "Bus252",
}
CALENDAR_VARIANTS = {
    "target2": "Target2", "luxembourg": "Luxembourg", "usnyse": "UnitedStatesNyse", "usfed": "UnitedStatesFederalReserve",
    "uk": "UnitedKingdom", "japan": "Japan", "switzerland": "Switzerland",
    "hongkong": "HongKong", "singapore": "Singapore", "brazilb3": "BrazilB3",
}

# Selected partial coupons with next coupons before, on and after leap day.
# These are input scenarios only; expected results execute FinancePy.
COUPON_ENDPOINT_CASES = [
    (2011, 12, 28, 2012, 2, 28, 12, 2012, 2, 28),
    (2011, 12, 28, 2012, 2, 28, 12, 2012, 2, 29),
    (2011, 12, 28, 2012, 2, 28, 12, 2012, 3, 1),
    (2011, 12, 28, 2012, 2, 29, 12, 2012, 2, 29),
    (2011, 12, 28, 2012, 2, 29, 12, 2012, 3, 1),
    (2011, 12, 28, 2012, 2, 28, 6, 2012, 2, 28),
    (2011, 12, 28, 2012, 2, 28, 6, 2012, 2, 29),
    (2011, 12, 28, 2012, 2, 28, 6, 2012, 3, 1),
    (2011, 12, 28, 2012, 2, 29, 6, 2012, 2, 29),
    (2011, 12, 28, 2012, 2, 29, 6, 2012, 3, 1),
    (2010, 12, 28, 2011, 2, 28, 6, 2011, 2, 28),
    (2010, 12, 28, 2011, 2, 28, 6, 2011, 3, 1),
]


def iso(date):
    import QuantLib as ql
    if isinstance(date, str):
        return date
    return date.ISO() if isinstance(date, ql.Date) else date.isoformat()


def qdate(date):
    import QuantLib as ql
    return ql.Date(date.day, date.month, date.year)


def add_months(date, months):
    import QuantLib as ql
    # Uses QuantLib date arithmetic only to select the input reference period.
    shifted = qdate(date) + ql.Period(months, ql.Months)
    return dt.date(shifted.year(), shifted.month(), shifted.dayOfMonth())


def reference_packages():
    script = (ROOT / "tests/run.py").read_text().split("# /// script\n", 1)[1].split("# ///", 1)[0]
    metadata = tomllib.loads("\n".join(line.removeprefix("# ") for line in script.splitlines()))
    requirements = [line.strip() for line in (ROOT / "tests/requirements.txt").read_text().splitlines()
                    if line.strip() and not line.startswith("#")]
    assert sorted(requirements) == sorted(metadata["dependencies"]), "requirements.txt differs from PEP 723 pins"
    return dict(requirement.split("==", 1) for requirement in requirements)


def generate():
    import QuantLib as ql
    from .calendar import build_calendar_references, append_references
    from .day_count import execute as execute_daycount_references

    python_packages = reference_packages()
    for package, pinned in python_packages.items():
        assert version(package) == pinned, (package, version(package), pinned)
    assert ql.__version__ == QL_VERSION, (ql.__version__, QL_VERSION)
    rng = random.Random(SEED)
    rows = []

    def emit(kind, convention, start, end=None, ref_start=None, ref_end=None,
             frequency=None, maturity=None, expected=None, source="quantlib"):
        row = [str(len(rows)), kind, convention, iso(start),
               iso(end) if end is not None else "-",
               iso(ref_start) if ref_start is not None else "-",
               iso(ref_end) if ref_end is not None else "-",
               str(frequency) if frequency is not None else "-",
               str(int(maturity)) if maturity is not None else "-",
               format(float(expected), ".17g") if expected is not None else "-", source]
        rows.append(row)
        return row

    counters = {
        "act360": ql.Actual360(),
        "act365f": ql.Actual365Fixed(),
        "actactisda": ql.ActualActual(ql.ActualActual.ISDA),
        "nl365": ql.Actual365Fixed(ql.Actual365Fixed.NoLeap),
        "thirty360bond": ql.Thirty360(ql.Thirty360.BondBasis),
        "thirtye360": ql.Thirty360(ql.Thirty360.European),
    }
    # Dense endpoint cases cover leap-day starts/ends, month ends, year ends,
    # reversed intervals and equal dates. Both century leap rules are included.
    edges = [dt.date.fromisoformat(value) for value in [
        "1999-12-31", "2000-02-28", "2000-02-29", "2000-03-01",
        "2023-02-28", "2023-12-31", "2024-01-01", "2024-01-30",
        "2024-01-31", "2024-02-28", "2024-02-29", "2024-03-01",
        "2024-08-30", "2024-08-31", "2025-02-28", "2025-03-01",
        "2099-12-31", "2100-02-28", "2100-03-01",
    ]]
    intervals = [(a, b) for a in edges for b in edges]
    first = dt.date(1901, 1, 1)
    span = (dt.date(2199, 12, 31) - first).days
    intervals += [(first + dt.timedelta(days=rng.randrange(span + 1)),
                   first + dt.timedelta(days=rng.randrange(span + 1))) for _ in range(512)]
    for start, end in intervals:
        for name, counter in counters.items():
            emit("fraction", name, start, end, expected=counter.yearFraction(qdate(start), qdate(end)))
        for maturity in (False, True):
            counter = ql.Thirty360(ql.Thirty360.ISDA, qdate(end) if maturity else ql.Date())
            emit("fraction", "thirtye360isda", start, end, maturity=maturity,
                 expected=counter.yearFraction(qdate(start), qdate(end)))
        emit("fraction", "oneone", start, end, source="quantlib-mapped")

    icma = ql.ActualActual(ql.ActualActual.ISMA)
    for frequency in (1, 2, 3, 4, 6, 12):
        for _ in range(32):
            ref_start = dt.date(rng.randrange(1999, 2040), rng.randrange(1, 13), 15)
            ref_end = add_months(ref_start, 12 // frequency)
            days = (ref_end - ref_start).days
            start = ref_start + dt.timedelta(days=rng.randrange(days + 1))
            end = ref_start + dt.timedelta(days=rng.randrange(days + 1))
            emit("fraction", "actacticma", start, end, ref_start, ref_end, frequency,
                 expected=icma.yearFraction(qdate(start), qdate(end), qdate(ref_start), qdate(ref_end)))
    for row in COUPON_ENDPOINT_CASES:
        start, end = dt.date(*row[:3]), dt.date(*row[3:6])
        months, ref_end = row[6], dt.date(*row[7:10])
        emit("fraction", "act365l", start, end, add_months(ref_end, -months), ref_end,
             12 // months, source="financepy")

    coupon_ends = [dt.date.fromisoformat(value) for value in [
        "1999-12-31", "2000-02-29", "2020-02-29", "2023-02-28",
        "2024-02-28", "2024-02-29", "2024-03-01", "2024-08-31",
        "2024-12-31", "2025-02-28", "2040-01-01", "2100-02-28",
    ]]
    # Every pair of selected positions inside each reference coupon includes
    # partial, full, reversed and equal accruals. Leap positions are explicit.
    for frequency in (1, 2, 3, 4, 6, 12):
        for ref_end in coupon_ends:
            ref_start = add_months(ref_end, -(12 // frequency))
            days = (ref_end - ref_start).days
            points = {ref_start + dt.timedelta(days=offset)
                      for offset in (0, 1, days // 2, days - 1, days)}
            for year in range(ref_start.year, ref_end.year + 1):
                try:
                    leap = dt.date(year, 2, 29)
                except ValueError:
                    continue
                for delta in (-1, 0, 1):
                    position = leap + dt.timedelta(days=delta)
                    if ref_start <= position <= ref_end:
                        points.add(position)
            for start in sorted(points):
                for end in sorted(points):
                    emit("fraction", "act365l", start, end, ref_start, ref_end, frequency,
                         source="financepy")
        for _ in range(64):
            ref_start = dt.date(rng.randrange(1902, 2198), rng.randrange(1, 13), rng.randrange(1, 29))
            ref_end = add_months(ref_start, 12 // frequency)
            days = (ref_end - ref_start).days
            start = ref_start + dt.timedelta(days=rng.randrange(days + 1))
            end = ref_start + dt.timedelta(days=rng.randrange(days + 1))
            for first, second in ((start, end), (end, start), (start, start)):
                emit("fraction", "act365l", first, second, ref_start, ref_end, frequency,
                     source="financepy")

    calendar_references, calendar_metadata = build_calendar_references()
    assert set(calendar_references) == set(CALENDAR_VARIANTS), "Missing named calendar references"
    start, end = dt.date(2020, 1, 1), dt.date(2040, 12, 31)
    for name, calendar in calendar_references.items():
        date = calendar.start
        while date <= calendar.end:
            emit("business", name, date, expected=calendar.is_business_day(date), source="calendar-reference")
            emit("holiday", name, date, expected=calendar.is_holiday(date), source="calendar-reference")
            date += dt.timedelta(days=1)

    bus252 = ql.Business252(ql.TARGET())
    bus_intervals = [(dt.date.fromisoformat(a), dt.date.fromisoformat(b)) for a, b in [
        ("2026-04-03", "2026-04-07"), ("2026-04-04", "2026-04-07"),
        ("2026-04-07", "2026-04-07"), ("2026-04-02", "2026-04-07"),
        ("2024-02-28", "2024-03-01"),
    ]]
    span = (end - start).days
    for _ in range(256):
        first = start + dt.timedelta(days=rng.randrange(span + 1))
        second = start + dt.timedelta(days=rng.randrange(span + 1))
        bus_intervals.append((min(first, second), max(first, second)))
    for first, second in bus_intervals:
        emit("bus252", "target2", first, second,
             expected=bus252.yearFraction(qdate(first), qdate(second)))
        for lower, upper in ((first, second), (second, first)):
            emit("bus252", "target2", lower, upper, source="quantlib-mapped")

    # Selected primary-source facts, separate from QuantLib comparisons.
    # MOM's complete 2026 and 2027 schedules were read back on 2026-10-23.
    for value in ["2026-01-01", "2026-02-17", "2026-02-18", "2026-03-21",
                  "2026-04-03", "2026-05-01", "2026-05-27", "2026-06-01",
                  "2026-08-10", "2026-11-09", "2026-12-25", "2027-01-01",
                  "2027-02-08", "2027-03-10", "2027-03-26", "2027-05-17",
                  "2027-05-20", "2027-08-09", "2027-10-28"]:
        emit("business", "singapore", dt.date.fromisoformat(value), expected=False, source="mom")
    emit("business", "singapore", dt.date(2026, 3, 20), expected=True, source="mom")
    # Raw holiday query distinguishes the published Saturday closure from the
    # weekend business-day rule and would catch a wrong Friday substitution.
    emit("holiday", "singapore", dt.date(2026, 3, 21), expected=True, source="mom")

    # Government and ING corroborate Ascension on May 14 (the BCL page prints
    # May 15). Partial Christmas/New Year Eve closures remain business days.
    for value in ["2026-01-01", "2026-04-03", "2026-04-06", "2026-05-01",
                  "2026-05-09", "2026-05-14", "2026-05-25", "2026-06-23",
                  "2026-08-15", "2026-11-01", "2026-12-25", "2026-12-26"]:
        emit("business", "luxembourg", dt.date.fromisoformat(value), expected=False, source="lux-bank")
    for value in ["2026-05-15", "2026-06-22", "2026-12-24", "2026-12-31"]:
        emit("business", "luxembourg", dt.date.fromisoformat(value), expected=True, source="lux-bank")

    operation_metadata = append_references(emit, ql, rng, calendar_references)

    python_reference_metadata = execute_daycount_references(rows)
    tsv = "\t".join(COLUMNS) + "\n" + "".join("\t".join(row) + "\n" for row in rows)
    counts = collections.Counter(row[2] for row in rows)
    day_count_sources = collections.defaultdict(collections.Counter)
    for row in rows:
        if row[1] in ("fraction", "bus252"):
            convention = "bus252" if row[1] == "bus252" else row[2]
            day_count_sources[DAY_COUNTS[convention]][row[10]] += 1
    assert set(day_count_sources) == set(DAY_COUNTS.values()), "Missing DayCount reference coverage"
    manifest = {
        "format_version": 1,
        "python_reference_packages": python_packages,
        "seed": SEED,
        "generator": "tests/run.py (independent Python worker)",
        "reference_sha256": hashlib.sha256(tsv.encode()).hexdigest(),
        "rows": len(rows),
        "counts_by_convention": dict(sorted(counts.items())),
        "counts_by_kind": dict(sorted(collections.Counter(row[1] for row in rows).items())),
        "day_count_variants": 11,
        "operations": operation_metadata,
        "counts_by_day_count": {name: sum(values.values()) for name, values in sorted(day_count_sources.items())},
        "day_count_reference_counts": {name: dict(sorted(values.items())) for name, values in sorted(day_count_sources.items())},
        "sources": {
            "quantlib": {
                "package": f"QuantLib=={QL_VERSION}",
                "package_url": f"https://pypi.org/project/QuantLib/{QL_VERSION}/",
                "upstream": f"https://github.com/lballabio/QuantLib/tree/v{QL_VERSION}",
                "mappings": {
                    "nl365": "Actual365Fixed(Actual365Fixed.NoLeap)",
                    "actacticma": "ActualActual(ISMA), one contained regular reference coupon period",
                    "thirty360bond": "Thirty360(BondBasis)",
                    "thirtye360": "Thirty360(European)",
                    "thirtye360isda": "Thirty360(ISDA, terminationDate=end if maturity else Date())",
                    "usnyse": "UnitedStates(NYSE)",
                    "usfed": "UnitedStates(FederalReserve)",
                    "target2": "TARGET()",
                    "bus252": "Business252(TARGET()), forward and equal intervals only",
                },
            },
            **python_reference_metadata,
            "calendar-reference": calendar_metadata,
            "mom": {
                "source": "https://www.mom.gov.sg/employment-practices/public-holidays",
                "readback_date": "2026-10-23",
                "scope": "Selected 2026/2027 full-day public holiday and Sunday-observation facts",
            },
            "lux-bank": {
                "sources": [
                    "https://luxembourg.public.lu/fr/vivre/qualite-de-vie/jours-feries-legaux-luxembourg.html",
                    "https://www.ing.lu/webing/content/siteing/fr/private-banking/contacts-and-information/information/more-information/bank-closing-days.html",
                    "https://www.bcl.lu/fr/apropos/jours_douverture/index.html",
                ],
                "readback_date": "2026-10-23",
                "scope": "2026 bank full-day closures and selected adjacent/partial-day business dates",
                "source_discrepancy": "BCL prints Ascension 2026-05-15; government and ING confirm 2026-05-14.",
                "partial_days": "Christmas Eve and New Year's Eve have partial closures; both remain business days in a full-day model.",
            },
        },
        "limits": [
            "QuantLib day-count comparisons span 1901–2199; Date primitive references cover the full 1583–9999 supported year range.",
            "ICMA cases are contained in one regular reference period; multi-period stubs are unsupported.",
            "2020-2040 calendar agreement does not confer official publication or verified status on future dates.",
            "Signed Bus/252 explicitly maps normalized QuantLib inputs; its raw reversed endpoint inclusion is a different API convention.",
            "Primary-source selected dates are regression facts, not complete calendar certification.",
        ],
    }
    return tsv, json.dumps(manifest, indent=2, sort_keys=True) + "\n"


def cases():
    """Emit inputs and independently calculated outcomes to the controller.

    This worker never invokes the Rust implementation, reads its tables, or
    receives its outputs. The controller sends Rust only id/api/args.
    """
    from . import date, roll, calendar, day_count
    tsv, metadata = generate()
    for line in tsv.splitlines()[1:]:
        fields = line.split("\t")
        kind, operation = fields[1:3]
        needs_calendar = (kind in ("business", "holiday", "bus252", "calendar_op")
                          or (kind == "roll_op" and not operation.startswith("weekends:")))
        yield {"id": "reference-" + fields[0], "api": "@reference",
               "args": {"row": "|".join(fields[:9])},
               "expected": "Ok:f64:" + fields[9], "source": fields[10],
               "calendars": needs_calendar}
    for module in (date, roll, calendar, day_count):
        for case in module.cases():
            yield {**case, "calendars": case["api"].startswith("calendar::")}
    yield {"metadata": json.loads(metadata)}
