"""Independent Date, roll and calendar-operation reference cases.

No expected value is derived from Rust source. Calendar arithmetic executes
QuantLib on independent reference-calendar data; Gregorian arithmetic executes
Python's standard library, python-dateutil and convertdate.
"""

import calendar
import datetime as dt

from convertdate import holidays as reference_holidays
from dateutil.easter import easter as dateutil_easter
from dateutil.relativedelta import relativedelta, weekday


DATE_OPERATIONS = (
    "ymd", "ymd_unchecked", "year", "month", "day", "is_leap_year",
    "days_in_month", "day_of_week", "add_days", "add_months_eom_aware",
    "nth_weekday_of_month", "easter_sunday", "days_between",
)
ROLL_VARIANTS = (
    "Unadjusted", "Following", "ModifiedFollowing", "Preceding",
    "ModifiedPreceding", "Nearest", "EndOfMonth",
)


def encoded(date):
    return date.year * 10000 + date.month * 100 + date.day


def qdate(ql, date):
    return ql.Date(date.day, date.month, date.year)


def decoded(date):
    return dt.date(date.year(), date.month(), date.dayOfMonth())


def append_date_references(emit, rng):
    counts = {operation: 0 for operation in DATE_OPERATIONS}

    def add(operation, start, expected, **kwargs):
        counts[operation] += 1
        return emit("date_op", operation, start, expected=expected,
                    source=kwargs.pop("source", "python-calendar"), **kwargs)

    # Every supported Gregorian year and every month, not just modern dates.
    for year in range(1583, 10000):
        first = dt.date(year, 1, 1)
        add("is_leap_year", first, calendar.isleap(year))
        for month in range(1, 13):
            add("days_in_month", dt.date(year, month, 1), calendar.monthrange(year, month)[1])
        # convertdate executes the USNO implementation independently of Rust's
        # Meeus/Butcher implementation. dateutil's published scope ends4099.
        easter = dt.date(*reference_holidays.easter(year, church="western"))
        add("easter_sunday", first, encoded(easter), source="convertdate")
        if year <= 4099:
            assert easter == dateutil_easter(year), (year, easter, dateutil_easter(year))
        # Year-by-year weekday anchors include both sides of February.
        for point in (first, dt.date(year, 2, 28), dt.date(year, 3, 1)):
            add("day_of_week", point, point.isoweekday())

    selected = [dt.date.fromisoformat(value) for value in (
        "1583-01-01", "1583-12-31", "1599-12-31", "1600-02-28",
        "1600-02-29", "1700-02-28", "1900-02-28", "1999-12-31",
        "2000-02-29", "2024-01-31", "2024-02-29", "2024-04-30",
        "2025-02-28", "2100-02-28", "2400-02-29", "4099-12-31",
        "4100-02-28", "9900-02-28", "9999-01-01", "9999-12-31",
    )]
    lo, hi = dt.date(1583, 1, 1).toordinal(), dt.date(9999, 12, 31).toordinal()
    selected += [dt.date.fromordinal(rng.randint(lo, hi)) for _ in range(1024)]
    for point in selected:
        for operation, value in (
            ("ymd", encoded(point)), ("ymd_unchecked", encoded(point)),
            ("year", point.year), ("month", point.month), ("day", point.day),
        ):
            add(operation, point, value)
        for offset in (-366, -31, -1, 0, 1, 31, 366, rng.randint(-100000, 100000)):
            try:
                end = point + dt.timedelta(days=offset)
            except OverflowError:
                continue
            if end.year >= 1583:
                add("add_days", point, encoded(end), frequency=offset)
                add("days_between", point, (end - point).days, end=end)
        for offset in (-120, -13, -1, 0, 1, 13, 120):
            try:
                end = point + relativedelta(months=offset)
            except (ValueError, OverflowError):
                continue
            if end.year >= 1583:
                add("add_months_eom_aware", point, encoded(end), frequency=offset,
                    source="python-dateutil")
        # dateutil's weekday adjustment may leave the month for a missing
        # fifth occurrence; the crate's documented contract returns an error.
        first = point.replace(day=1)
        for day_of_week in range(7):
            for occurrence in (1, 3, 5):
                try:
                    end = first + relativedelta(weekday=weekday(day_of_week)(occurrence))
                    expected = encoded(end) if end.month == first.month else 0
                except (ValueError, OverflowError):
                    expected = 0
                add("nth_weekday_of_month", first, expected,
                    frequency=occurrence, ref_start=str(day_of_week), source="python-dateutil")

    for value in ("1582-12-31", "10000-01-01", "2024-00-01", "2024-13-01",
                  "2024-01-00", "2024-04-31", "2023-02-29", "2100-02-29",
                  "2024-02-30", "2024-01-32"):
        year, month, day = map(int, value.split("-"))
        try:
            reference = dt.date(year, month, day)
            expected = encoded(reference) if reference.year >= 1583 else 0
        except ValueError:
            expected = 0
        add("ymd", value, expected)
    for occurrence in (0, 6, 255):
        add("nth_weekday_of_month", dt.date(2026, 2, 1), 0,
            frequency=occurrence, ref_start="0", source="python-calendar")
    return counts


def reference_calendars(ql, refs):
    """Use QuantLib's algorithms over independent external holiday data."""
    result = {"weekends": ql.WeekendsOnly()}
    first, last = dt.date(2020, 1, 1), dt.date(2040, 12, 31)
    for name, reference in refs.items():
        cal = ql.BespokeCalendar("regit-independent-" + name)
        cal.addWeekend(ql.Saturday)
        cal.addWeekend(ql.Sunday)
        date = first
        while date <= last:
            if not reference.is_business_day(date):
                cal.addHoliday(qdate(ql, date))
            date += dt.timedelta(days=1)
        result[name] = cal
    return result


def append_references(emit, ql, rng, refs=None):
    """Append cases through the main oracle emitter and return provenance."""
    if refs is None:
        from calendar_references import build_calendar_references
        refs, _ = build_calendar_references()
    counts = append_date_references(emit, rng)
    calendars = reference_calendars(ql, refs)
    roll_codes = {
        "Unadjusted": ql.Unadjusted, "Following": ql.Following,
        "ModifiedFollowing": ql.ModifiedFollowing, "Preceding": ql.Preceding,
        "ModifiedPreceding": ql.ModifiedPreceding, "Nearest": ql.Nearest,
    }
    selected = set()
    for year in (2021, 2024, 2026, 2027, 2032, 2039):
        for month in range(1, 13):
            first = dt.date(year, month, 1)
            last = dt.date(year, month, calendar.monthrange(year, month)[1])
            for point in (first, first + dt.timedelta(days=1), last - dt.timedelta(days=1), last):
                selected.add(point)
        easter = dateutil_easter(year)
        selected.update(easter + dt.timedelta(days=offset) for offset in range(-4, 5))
    start, span = dt.date(2021, 1, 1), (dt.date(2039, 12, 31) - dt.date(2021, 1, 1)).days
    selected.update(start + dt.timedelta(days=rng.randint(0, span)) for _ in range(64))
    roll_counts = {}
    operation_counts = {}

    def expected_adjust(cal, point, variant):
        date = qdate(ql, point)
        if variant == "EndOfMonth":
            # This crate's standalone-date operation supplies no schedule
            # origin. QL's zero-month EOM advance is exactly that contract.
            return cal.advance(date, ql.Period(0, ql.Months), ql.ModifiedFollowing, True)
        return cal.adjust(date, roll_codes[variant])

    for name, cal in calendars.items():
        for point in sorted(selected):
            for variant in ROLL_VARIANTS:
                expected = encoded(decoded(expected_adjust(cal, point, variant)))
                emit("roll_op", name + ":" + variant, point, expected=expected,
                     source="quantlib-operations")
                roll_counts[variant] = roll_counts.get(variant, 0) + 1
            if name == "weekends":
                continue
            emit("calendar_op", name + ":is_weekend", point,
                 expected=point.weekday() >= 5, source="python-calendar")
            operation_counts["is_weekend"] = operation_counts.get("is_weekend", 0) + 1
            for operation, result in (
                ("next_business_day", cal.adjust(qdate(ql, point), ql.Following)),
                ("previous_business_day", cal.adjust(qdate(ql, point), ql.Preceding)),
            ):
                emit("calendar_op", name + ":" + operation, point,
                     expected=encoded(decoded(result)), source="quantlib-operations")
                operation_counts[operation] = operation_counts.get(operation, 0) + 1
            for amount in (-10, -1, 0, 1, 10):
                result = cal.advance(qdate(ql, point), amount, ql.Days, ql.Following)
                emit("calendar_op", name + ":add_business_days", point, frequency=amount,
                     expected=encoded(decoded(result)), source="quantlib-operations")
                operation_counts["add_business_days"] = operation_counts.get("add_business_days", 0) + 1
            for offset in (0, 1, 7, 31):
                end = point + dt.timedelta(days=offset)
                result = cal.businessDaysBetween(qdate(ql, point), qdate(ql, end), True, False)
                emit("calendar_op", name + ":business_days_between", point, end=end,
                     expected=result, source="quantlib-operations")
                operation_counts["business_days_between"] = operation_counts.get("business_days_between", 0) + 1
                bus_fraction = ql.Business252(cal).yearFraction(qdate(ql, point), qdate(ql, end))
                for first, last, sign in ((point, end, 1), (end, point, -1)):
                    emit("calendar_op", name + ":bus_252_for_calendar", first, end=last,
                         expected=sign * bus_fraction, source="quantlib-operations")
                    operation_counts["bus_252_for_calendar"] = operation_counts.get("bus_252_for_calendar", 0) + 1

    composite_sets = (
        (), ("target2",), ("target2", "usnyse"), ("target2", "luxembourg"),
        ("usnyse", "uk"), ("japan", "hongkong", "singapore"),
    )
    composite_counts = {}
    for names in composite_sets:
        if any(name not in calendars for name in names):
            raise RuntimeError("Missing composite reference calendars: " + repr(names))
        # Native QL joint-calendar evaluation determines joint business days.
        if not names:
            joint, token = ql.NullCalendar(), "empty"
        elif len(names) == 1:
            joint, token = calendars[names[0]], "single-" + names[0]
        else:
            joint = ql.JointCalendar([calendars[name] for name in names], ql.JoinHolidays)
            token = "+".join(names)
        for point in sorted(selected)[::4]:
            for kind in ("Composite", "JointBusiness"):
                prefix = token + ":" + kind + "."
                cases = [
                    ("is_business_day", int(joint.isBusinessDay(qdate(ql, point))), {}),
                    ("is_holiday", int(any(refs[name].is_holiday(point) for name in names)), {}),
                    ("adjust", encoded(decoded(joint.adjust(qdate(ql, point), ql.ModifiedFollowing))), {}),
                ]
                for amount in (-2, 0, 2):
                    result = joint.advance(qdate(ql, point), amount, ql.Days, ql.Following)
                    cases.append(("add_business_days", encoded(decoded(result)), {"frequency": amount}))
                end = point + dt.timedelta(days=14)
                count = joint.businessDaysBetween(qdate(ql, point), qdate(ql, end), True, False)
                cases.append(("business_days_between", count, {"end": end}))
                # Apply the documented signed lower-inclusive/upper-exclusive
                # contract to the executed QuantLib joint-calendar result.
                bus_fraction = ql.Business252(joint).yearFraction(qdate(ql, point), qdate(ql, end))
                cases.append(("bus_252", bus_fraction, {"end": end}))
                cases.append(("bus_252", -bus_fraction, {"start": end, "end": point}))
                for operation, expected, kwargs in cases:
                    first = kwargs.pop("start", point)
                    emit("calendar_op", prefix + operation, first, expected=expected,
                         source="quantlib-joint-operations", **kwargs)
                    composite_counts[operation] = composite_counts.get(operation, 0) + 1
    return {
        "date_operations": {name: name for name in DATE_OPERATIONS},
        "date_operation_counts": counts,
        "date_reference_range": [1583, 9999],
        "date_references": {
            "python": "datetime.date, timedelta, calendar.isleap/monthrange",
            "python-dateutil": "2.9.0.post0 relativedelta months and nth-weekday; Easter overlap1583–4099",
            "convertdate": "2.5.1 holidays.easter(church='western'), USNO algorithm, all1583–9999",
        },
        "roll_variants": list(ROLL_VARIANTS),
        "roll_counts": roll_counts,
        "roll_calendars": list(calendars),
        "roll_reference": "QuantLib Calendar.adjust; standalone EndOfMonth=Calendar.advance(0 Months, ModifiedFollowing, endOfMonth=true)",
        "calendar_operations": {name: name for name in operation_counts},
        "calendar_operation_counts": operation_counts,
        "composite_operations": {name: name for name in composite_counts},
        "composite_operation_counts": composite_counts,
        "calendar_operation_reference": "QuantLib BespokeCalendar over independent reference data; native JointCalendar(JoinHolidays)",
    }
