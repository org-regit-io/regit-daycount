# /// script
# requires-python = ">=3.12,<3.14"
# dependencies = [
#   "QuantLib==1.44", "financepy==1.1.2", "python-dateutil==2.9.0.post0",
#   "convertdate==2.5.1", "holidays==0.107", "PyMeeus==0.5.12", "six==1.17.0",
#   "numpy==2.3.5", "numba==0.62.1", "llvmlite==0.45.1", "scipy==1.16.3",
#   "pandas==2.3.3", "matplotlib==3.10.9", "contourpy==1.4.0", "cycler==0.12.1",
#   "fonttools==4.67.0", "kiwisolver==1.5.1", "packaging==26.3", "pillow==13.0.0",
#   "pyparsing==3.3.3", "pytz==2026.5", "tzdata==2026.5",
# ]
# ///
"""Regenerate independent fixtures, check their bytes, and optionally compare Rust.

Run with `uv run validation/oracle.py --check --rust`. Python reference libraries
execute independently; offline Rust tests read committed fixtures.
"""

import argparse
import collections
import datetime as dt
import hashlib
from importlib.metadata import version
import json
from pathlib import Path
import random
import re
import subprocess
import sys
import tomllib

import QuantLib as ql


ROOT = Path(__file__).resolve().parent.parent
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
    "target2": "Target2", "luxembourg": "Luxembourg", "uslegacy": "UnitedStates",
    "usnyse": "UnitedStatesNyse", "usfed": "UnitedStatesFederalReserve",
    "uk": "UnitedKingdom", "japan": "Japan", "switzerland": "Switzerland",
    "hongkong": "HongKong", "singapore": "Singapore",
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
    if isinstance(date, str):
        return date
    return date.ISO() if isinstance(date, ql.Date) else date.isoformat()


def qdate(date):
    return ql.Date(date.day, date.month, date.year)


def add_months(date, months):
    # Uses QuantLib date arithmetic only to select the input reference period.
    shifted = qdate(date) + ql.Period(months, ql.Months)
    return dt.date(shifted.year(), shifted.month(), shifted.dayOfMonth())


def reference_packages():
    script = Path(__file__).read_text().split("# /// script\n", 1)[1].split("# ///", 1)[0]
    metadata = tomllib.loads("\n".join(line.removeprefix("# ") for line in script.splitlines()))
    requirements = [line.strip() for line in (ROOT / "validation/requirements.txt").read_text().splitlines()
                    if line.strip() and not line.startswith("#")]
    assert sorted(requirements) == sorted(metadata["dependencies"]), "requirements.txt differs from PEP 723 pins"
    return dict(requirement.split("==", 1) for requirement in requirements)


def enum_inventory(path, name):
    source = (ROOT / path).read_text()
    body = re.search(r"pub enum " + re.escape(name) + r"\s*\{(.*?)\n\}", source, re.S)
    assert body is not None, f"Cannot inventory public enum {name} from {path}"
    variants = re.findall(r"^\s*([A-Za-z][A-Za-z0-9_]*)\s*,", body.group(1), re.M)
    assert variants and len(variants) == len(set(variants)), (name, variants)
    return variants


def enforce_catalogue_coverage(rows, operation_metadata):
    inventories = {
        "DayCount": enum_inventory("src/day_count/mod.rs", "DayCount"),
        "Calendar": enum_inventory("src/calendar/mod.rs", "Calendar"),
        "Roll": enum_inventory("src/roll.rs", "Roll"),
    }
    covered = {key: collections.Counter() for key in inventories}
    calendar_dates = collections.defaultdict(set)
    for row in rows:
        kind, name = row[1:3]
        if kind in ("fraction", "bus252"):
            covered["DayCount"][DAY_COUNTS["bus252" if kind == "bus252" else name]] += 1
        elif kind in ("business", "holiday"):
            covered["Calendar"][CALENDAR_VARIANTS[name]] += 1
            calendar_dates[(name, kind)].add(row[3])
        elif kind == "roll_op":
            covered["Roll"][name.rsplit(":", 1)[1]] += 1
    for catalogue, variants in inventories.items():
        assert set(covered[catalogue]) == set(variants), (catalogue, variants, dict(covered[catalogue]))
    date_methods = set(re.findall(r"pub (?:const )?fn ([A-Za-z0-9_]+)\(", (ROOT / "src/date.rs").read_text()))
    date_cases = collections.Counter(row[2] for row in rows if row[1] == "date_op")
    assert set(date_cases) == date_methods, ("Date", sorted(date_methods), dict(date_cases))
    def public_operations(paths, ignored=()):
        names = set()
        for path in paths:
            for name in re.findall(r"pub (?:const )?fn ([A-Za-z0-9_]+)\(", (ROOT / path).read_text()):
                if name not in ignored:
                    names.add(name.removeprefix("try_").removesuffix("_with_policy"))
        return names

    expected_calendar_ops = public_operations(
        ("src/calendar/mod.rs", "src/calendar/checked.rs"), ignored=("coverage",))
    expected_composite_ops = public_operations(("src/calendar/composite.rs",), ignored=("new",))
    calendar_operations = collections.Counter()
    composite_operations = collections.defaultdict(collections.Counter)
    for row in rows:
        if row[1] == "business":
            calendar_operations["is_business_day"] += 1
        elif row[1] == "holiday":
            calendar_operations["is_holiday"] += 1
        elif row[1] == "roll_op" and not row[2].startswith("weekends:"):
            calendar_operations["adjust"] += 1
        elif row[1] == "calendar_op":
            operation = row[2].split(":", 1)[1]
            if "." in operation:
                combinator, operation = operation.split(".", 1)
                composite_operations[combinator][operation] += 1
            else:
                calendar_operations[operation] += 1
    assert set(calendar_operations) == expected_calendar_ops, ("Calendar operations", sorted(expected_calendar_ops), dict(calendar_operations))
    assert set(composite_operations) == {"Composite", "JointBusiness"}, composite_operations
    for combinator, cases in composite_operations.items():
        assert set(cases) == expected_composite_ops, (combinator, sorted(expected_composite_ops), dict(cases))
    for name in CALENDAR_VARIANTS:
        for kind in ("business", "holiday"):
            days = {dt.date.fromisoformat(value) for value in calendar_dates[(name, kind)]}
            required = {dt.date(2020, 1, 1) + dt.timedelta(days=offset) for offset in range(7671)}
            assert days >= required, f"Incomplete {name} {kind} reference horizon"
    return {
        catalogue: {"public_variants": inventories[catalogue], "cases": dict(sorted(covered[catalogue].items()))}
        for catalogue in inventories
    } | {"Date": {"public_methods": sorted(date_methods), "cases": dict(sorted(date_cases.items()))},
         "CalendarOperations": {"public_operations": sorted(expected_calendar_ops), "cases": dict(sorted(calendar_operations.items()))},
         "CompositeOperations": {"public_operations": sorted(expected_composite_ops),
                                 "cases": {name: dict(sorted(cases.items())) for name, cases in sorted(composite_operations.items())}},
         "operations": operation_metadata}


def generate():
    from calendar_references import build_calendar_references
    from operations_references import append_references
    from python_daycounts import execute as execute_daycount_references

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
        date = start
        while date <= end:
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
        "generator": "validation/oracle.py",
        "fixture": "validation/oracle.tsv",
        "fixture_sha256": hashlib.sha256(tsv.encode()).hexdigest(),
        "rows": len(rows),
        "counts_by_convention": dict(sorted(counts.items())),
        "counts_by_kind": dict(sorted(collections.Counter(row[1] for row in rows).items())),
        "day_count_variants": 11,
        "coverage": enforce_catalogue_coverage(rows, operation_metadata),
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


def compare_rust(tsv):
    result = subprocess.run(["cargo", "run", "--quiet", "--example", "oracle_probe"],
                            cwd=ROOT, input=tsv, text=True, capture_output=True)
    if result.returncode:
        print(result.stderr, file=sys.stderr)
        raise SystemExit(result.returncode)
    values = result.stdout.splitlines()
    expected_rows = tsv.splitlines()[1:]
    assert len(values) == len(expected_rows), (len(values), len(expected_rows), result.stderr)
    failures = []
    for row, response in zip(expected_rows, values):
        fields = row.split("\t")
        case_id, actual = response.split("\t", 1)
        assert case_id == fields[0], (case_id, fields[0])
        try:
            difference = abs(float(actual) - float(fields[9]))
            if not difference <= 1e-12:
                failures.append((row, response))
        except ValueError:
            failures.append((row, response))
    for row, response in failures[:30]:
        print(f"MISMATCH {row}\n    Rust: {response}", file=sys.stderr)
    if failures:
        categories = collections.Counter(row.split("\t")[1] for row, _ in failures)
        operations = collections.Counter(":".join(row.split("\t")[1:3]) for row, _ in failures)
        print("Failure kinds: " + json.dumps(dict(categories), sort_keys=True), file=sys.stderr)
        print("Failure operations: " + json.dumps(dict(operations), sort_keys=True), file=sys.stderr)
        raise SystemExit(f"{len(failures)} of {len(values)} independent oracle comparisons failed")
    print(f"Rust matches {len(values)} independent oracle fixtures (tolerance 1e-12)")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="Verify committed fixtures without writing")
    parser.add_argument("--rust", action="store_true", help="Execute Rust probe and compare every fixture")
    args = parser.parse_args()
    tsv, manifest = generate()
    for name, content in [("oracle.tsv", tsv), ("oracle.json", manifest)]:
        path = ROOT / "validation" / name
        if args.check:
            assert path.read_text() == content, f"Fixture differs; run uv run validation/oracle.py: {path}"
        else:
            path.write_text(content)
    print(f"{'Verified' if args.check else 'Generated'} {len(tsv.splitlines()) - 1} fixtures with executed Python reference libraries; seed {SEED}")
    print("DayCount reference cases: " + json.dumps(json.loads(manifest)["counts_by_day_count"], sort_keys=True))
    if args.rust:
        compare_rust(tsv)


if __name__ == "__main__":
    main()
