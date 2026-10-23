"""Independent Gregorian Date and validation-error oracle.

Expectations execute datetime, calendar, dateutil and convertdate; no Rust
implementation or observations are read by this module.
"""

import calendar


import datetime as dt


from itertools import product


from convertdate import holidays


from dateutil.relativedelta import relativedelta, weekday


from dateutil.easter import easter as dateutil_easter


MIN = dt.date(1583, 1, 1)


MAX = dt.date(9999, 12, 31)


LO, HI = MIN.toordinal(), MAX.toordinal()


WEEKDAYS = ("Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun")


SOURCE = "Python public-contract model + datetime/calendar"


def valid_date(year, month, day):
    if year < 1583:
        return "Err:ValidationError:OutOfRange:year < 1583"
    if year > 9999:
        return "Err:ValidationError:OutOfRange:year > 9999"
    if not 1 <= month <= 12:
        return "Err:ValidationError:InvalidDate:month-out-of-range"
    try:
        return dt.date(year, month, day)
    except ValueError:
        return "Err:ValidationError:InvalidDate:day-out-of-range"


def date_result(value):
    return "Ok:date:" + value.isoformat() if isinstance(value, dt.date) else value


def arithmetic(point, amount, months=False):
    kind = "month" if months else "day"
    error = f"Err:ValidationError:OutOfRange:{kind} arithmetic leaves supported date range"
    try:
        result = point + (relativedelta(months=amount) if months else dt.timedelta(days=amount))
    except (ValueError, OverflowError):
        return error
    return date_result(result) if result >= MIN else error


def debug_date(point):
    return f"Date {{ year: {point.year}, month: {point.month}, day: {point.day} }}"


from convertdate import holidays as reference_holidays


DATE_OPERATIONS = (
    "ymd", "year", "month", "day", "is_leap_year",
    "days_in_month", "day_of_week", "try_add_days", "try_add_months_eom_aware",
    "nth_weekday_of_month", "try_easter_sunday", "days_between",
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
        add("try_easter_sunday", first, encoded(easter), source="convertdate")
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
            ("ymd", encoded(point)),
            ("year", point.year), ("month", point.month), ("day", point.day),
        ):
            add(operation, point, value)
        for offset in (-2147483648, -366, -31, -1, 0, 1, 31, 366, 2147483647, rng.randint(-100000, 100000)):
            try:
                end = point + dt.timedelta(days=offset)
            except OverflowError:
                add("try_add_days", point, 0, frequency=offset)
                continue
            add("try_add_days", point, encoded(end) if end.year >= 1583 else 0, frequency=offset)
            if end.year >= 1583:
                add("days_between", point, (end - point).days, end=end)
        for offset in (-2147483648, -120, -13, -1, 0, 1, 13, 120, 2147483647):
            try:
                end = point + relativedelta(months=offset)
            except (ValueError, OverflowError):
                add("try_add_months_eom_aware", point, 0, frequency=offset, source="python-dateutil")
                continue
            add("try_add_months_eom_aware", point, encoded(end) if end.year >= 1583 else 0,
                frequency=offset, source="python-dateutil")
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


def representative_dates():
    return [dt.date.fromisoformat(value) for value in ('1583-01-01', '1583-01-02', '1583-12-31', '1599-12-31', '1600-02-28', '1600-02-29', '1700-02-28', '1900-02-28', '1999-12-31', '2000-02-29', '2024-01-31', '2024-02-29', '2024-03-31', '2025-02-28', '2026-05-13', '2026-05-23', '2026-05-24', '2026-05-25', '2026-05-31', '2026-02-01', '2026-12-31', '2100-02-28', '2400-02-29', '9999-01-01', '9999-12-30', '9999-12-31')]


def cases():
    rows = []
    def add(api, args, expected, source=SOURCE):
        rows.append(dict(id=f"date-contract-{len(rows):06}", api=api, args=args,
                         expected=expected, source=source))
    add("date::Date::MIN_YEAR", {}, "int:1583")
    add("date::Date::MAX_YEAR", {}, "int:9999")
    years = (-2147483648, -1, 0, 1582, 1583, 1600, 1700, 1900, 2000, 2024, 2025, 2100, 9999, 10000, 2147483647)
    for year, month, day in product(years, (0, 1, 2, 4, 12, 13, 255), (0, 1, 28, 29, 30, 31, 32, 255)):
        add("date::Date::ymd", dict(year=year, month=month, day=day), date_result(valid_date(year, month, day)))
    for value in range(256):
        for month, day in ((value, 1), (2, value), (4, value), (1, value)):
            add("date::Date::ymd", dict(year=2024, month=month, day=day), date_result(valid_date(2024, month, day)))
        add("date::Date::days_in_month", dict(year=2024, month=value),
            "int:" + str(calendar.monthrange(2024, value)[1] if 1 <= value <= 12 else 0))
    for year in years + (-400, -100, -4, 4, 100, 400, 2147483600):
        add("date::Date::is_leap_year", dict(year=year), "bool:" + str(calendar.isleap(year)).lower())
        for month in (0, 1, 2, 3, 4, 12, 13, 255):
            add("date::Date::days_in_month", dict(year=year, month=month),
                "int:" + str(calendar.monthrange(year, month)[1] if 1 <= month <= 12 else 0))
        easter = valid_date(year, 1, 1)
        if isinstance(easter, dt.date):
            easter = dt.date(*holidays.easter(year, church="western"))
        add("date::Date::try_easter_sunday", dict(year=year), date_result(easter), "convertdate USNO western Easter + date-domain contract")
    points = representative_dates()
    for point in points:
        arg = dict(date=point.isoformat())
        for method, expected in (("year", point.year), ("month", point.month), ("day", point.day)):
            add("date::Date::" + method, arg, f"int:{expected}")
        add("date::Date::day_of_week", arg, "text:" + WEEKDAYS[point.weekday()], "datetime.date.weekday")
        add("date::Date::Debug", arg, "text:" + debug_date(point), "Public Debug representation contract")
        day_offsets = {-2147483648, -367, -366, -31, -1, 0, 1, 31, 366, 367, 2147483647,
                       LO - point.toordinal() - 1, LO - point.toordinal(),
                       HI - point.toordinal(), HI - point.toordinal() + 1}
        for amount in sorted(day_offsets):
            add("date::Date::try_add_days", dict(**arg, days=amount), arithmetic(point, amount), "datetime.timedelta + date-domain contract")
        month_index = point.year * 12 + point.month - 1
        month_offsets = {-2147483648, -25, -13, -12, -2, -1, 0, 1, 2, 12, 13, 25, 2147483647,
                         1583 * 12 - month_index - 1, 1583 * 12 - month_index,
                         9999 * 12 + 11 - month_index, 9999 * 12 + 12 - month_index}
        for amount in sorted(month_offsets):
            add("date::Date::try_add_months_eom_aware", dict(**arg, months=amount), arithmetic(point, amount, months=True), "python-dateutil.relativedelta + date-domain contract")
        for other in points:
            args = dict(date=point.isoformat(), other=other.isoformat())
            add("date::Date::days_between", args, f"int:{(other - point).days}", "datetime.date subtraction")
            relation = "Less" if point < other else "Greater" if point > other else "Equal"
            add("date::Date::traits", args,
                f"text:eq={str(point == other).lower()},cmp={relation},partial_cmp=Some({relation}),clone=true,copy=true,hash_equal=true",
                "Python tuple ordering/equality + Clone/Copy/Hash laws")
    for year, month, n, day in product((1582, 1583, 1900, 2000, 2024, 2026, 9999, 10000),
                                     (0, 1, 2, 4, 12, 13, 255), (0, 1, 2, 3, 4, 5, 6, 255), range(7)):
        if not 1 <= n <= 5:
            expected = "Err:ValidationError:OutOfRange:n must be 1..=5"
        else:
            expected = valid_date(year, month, 1)
            if isinstance(expected, dt.date):
                try:
                    candidate = expected + relativedelta(weekday=weekday(day)(n))
                    expected = (date_result(candidate) if candidate.month == month else
                                "Err:ValidationError:OutOfRange:nth weekday does not exist in this month")
                except (ValueError, OverflowError):
                    expected = "Err:ValidationError:OutOfRange:nth weekday does not exist in this month"
        add("date::Date::nth_weekday_of_month", dict(year=year, month=month, n=n, weekday=WEEKDAYS[day]),
            expected, "python-dateutil weekday occurrence + validation contract")
    for name, other in product(WEEKDAYS, repeat=2):
        add("date::Weekday::traits", dict(weekday=name, other=other),
            f"text:debug={name},eq={str(name == other).lower()},clone=true,copy=true,hash_equal=true",
            "Public enum names + Clone/Copy/Hash laws")
    for variant, payload in (("InvalidDate", "month-out-of-range"), ("InvalidDate", "day-out-of-range"),
                             ("InvalidDate", "custom-rule"), ("OutOfRange", "year < 1583"),
                             ("OutOfRange", "year > 9999"), ("OutOfRange", "custom-value")):
        args = dict(variant=variant, payload=payload)
        prefix = "invalid date: " if variant == "InvalidDate" else "value out of range: "
        field = "rule" if variant == "InvalidDate" else "what"
        add("errors::ValidationError::Display", args, "text:" + prefix + payload, "Public error message contract")
        add("errors::ValidationError::Debug", args, f'text:{variant} {{ {field}: "{payload}" }}', "Public error Debug contract")
        add("errors::ValidationError::source", args, "text:None", "core::error::Error leaf-error contract")
        add("errors::ValidationError::traits", args, "text:clone=true,copy=true,eq=true,distinct=false", "Clone/Copy/PartialEq laws")
    validation_values = (("InvalidDate", "rule-a"), ("InvalidDate", "rule-b"),
                         ("OutOfRange", "value-a"), ("OutOfRange", "value-b"))
    for (variant, payload), (other_variant, other_payload) in product(validation_values, repeat=2):
        add("errors::ValidationError::PartialEq",
            dict(variant=variant, payload=payload, other_variant=other_variant, other_payload=other_payload),
            "bool:" + str((variant, payload) == (other_variant, other_payload)).lower(),
            "Python tagged-value equality")
    # Seeded ordinal pairs span the complete supported range, including
    # multi-millennial interior moves and their inverse operations.
    from random import Random
    rng = Random(20261023)
    for _ in range(24):
        first, last = (dt.date.fromordinal(rng.randint(LO, HI)) for _ in range(2))
        for start, end in ((first, last), (last, first)):
            offset = (end - start).days
            args = dict(date=start.isoformat(), days=offset)
            add("date::Date::try_add_days", args, date_result(end),
                "Python datetime ordinal pair across full Date domain")
            add("date::Date::days_between", dict(date=start.isoformat(), other=end.isoformat()),
                f"int:{offset}", "Python datetime subtraction across full Date domain")
    return rows
