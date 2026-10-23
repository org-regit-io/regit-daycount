"""Independent market references, calendar contracts and bulk operation cases.

Expected values execute Python holidays/QuantLib, datetime, and dateutil; this
module neither imports Rust nor reads Rust source, tables, binaries or results.
The reviewed coverage contract is asserted independently of implementation data.
"""

import calendar
import datetime as dt
import hashlib
import inspect
from functools import lru_cache
from pathlib import Path

import holidays
from holidays.constants import BANK, GOVERNMENT, OPTIONAL, PUBLIC
import QuantLib as ql
from convertdate import holidays as western_holidays
from dateutil.easter import easter as dateutil_easter

from .roll import ROLL_VARIANTS

from .date import (
    DATE_OPERATIONS, append_date_references, decoded, encoded, qdate,
)


HOLIDAYS_VERSION = "0.107"
START = dt.date(2020, 1, 1)
END = dt.date(2040, 12, 31)
VARIANTS = {
    "target2": "Target2", "usnyse": "UnitedStatesNyse",
    "usfed": "UnitedStatesFederalReserve", "brazilb3": "BrazilB3",
    "uk": "UnitedKingdom", "japan": "Japan", "switzerland": "Switzerland",
    "hongkong": "HongKong", "singapore": "Singapore", "luxembourg": "Luxembourg",
}


class CalendarReference:
    """A named-date set with explicit bounds, never an out-of-range default-open."""

    def __init__(self, name, named_dates, metadata):
        self.name = name
        lo, hi = metadata["available_years"]
        self.start, self.end = dt.date(lo, 1, 1), dt.date(hi, 12, 31)
        self.named_dates = frozenset(d for d in named_dates if self.start <= d <= self.end)
        self.metadata = metadata
        for year in range(lo, hi + 1):
            count = sum(date.year == year for date in self.named_dates)
            assert count >= metadata["minimum_named_holidays_per_year"], (name, year, count)

    def _check(self, date):
        if not self.start <= date <= self.end:
            raise ValueError(f"{self.name} reference outside explicit coverage: {date}")

    def is_holiday(self, date):
        self._check(date)
        return date in self.named_dates

    def is_business_day(self, date):
        self._check(date)
        return date.weekday() < 5 and date not in self.named_dates


def _dates():
    date = START
    while date <= END:
        yield date
        date += dt.timedelta(days=1)


def _source(entity):
    path = Path(inspect.getfile(type(entity)))
    package = Path(holidays.__file__).resolve().parent
    relative = str(path.relative_to(package)).replace("\\", "/")
    return {
        "module": "holidays/" + relative,
        "source": "https://github.com/vacanza/holidays/blob/v0.107/holidays/" + relative,
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
    }


def build_calendar_references():
    """Return `(refs, metadata)` for all ten explicitly named markets.

    Every object provides is_holiday(date) and is_business_day(date). Raw
    named-date mappings are retained explicitly instead of equating an external
    calendar's `isHoliday` (which includes every weekend) to Rust's predicate.
    """
    assert holidays.__version__ == HOLIDAYS_VERSION, holidays.__version__
    assert ql.__version__ == "1.44", ql.__version__
    years = range(2020, 2041)
    kwargs = dict(years=years, language="en_US", expand=False)
    sources = {}
    refs = {}

    def add(key, entity, named_dates=None, mapping="native PUBLIC financial named-date set", minimum=6, primary=(), available=(2020, 2040)):
        dates = set(entity) if named_dates is None else set(named_dates)
        sources[key] = _source(entity)
        meta = {
            "rust_variant": VARIANTS[key], "reference": type(entity).__name__,
            "mapping": mapping, "available_years": list(available),
            "minimum_named_holidays_per_year": minimum,
            "primary_sources": list(primary),
            "annual_named_holiday_counts": {str(year): sum(date.year == year for date in dates) for year in years},
            "projection_policy": "Future unpublished years are independent library projections, not official or verified calendars.",
        }
        refs[key] = CalendarReference(key, dates, meta)

    add("target2", holidays.financial_holidays("XECB", **kwargs), minimum=6,
        primary=["https://www.ecb.europa.eu/press/pr/date/2000/html/pr001214_4.en.html"])
    add("usnyse", holidays.financial_holidays("XNYS", **kwargs), minimum=8,
        primary=["https://www.nyse.com/trade/hours-calendars"])
    add("uk", holidays.financial_holidays("XLON", **kwargs), minimum=7,
        primary=["https://www.gov.uk/bank-holidays", "https://www.londonstockexchange.com/equities-trading/business-days"])
    add("japan", holidays.financial_holidays("XJPX", **kwargs), minimum=17,
        primary=["https://www.jpx.co.jp/english/corporate/about-jpx/calendar/index.html"])
    add("switzerland", holidays.financial_holidays("XSWX", **kwargs), minimum=6,
        mapping="Native SIX PUBLIC full-day trading closures; weekend-only nominal dates are omitted, partial sessions excluded.",
        primary=["https://www.six-group.com/en/market-data/news-tools/trading-currency-holiday-calendar.html"])
    add("hongkong", holidays.financial_holidays("XHKG", **kwargs), minimum=10,
        mapping="Native HKEX PUBLIC full-day trading closures including listed severe-weather cancellations; weekend-only nominal dates and HALF_DAY category omitted.",
        primary=["https://www.hkex.com.hk/Services/Trading-hours-and-Severe-Weather-Arrangements/Trading-Hours/Securities-Market"])

    sg = holidays.Singapore(**kwargs)
    sgx = holidays.financial_holidays("XSES", **kwargs)
    add("singapore", sg, (date for date in sg if date.weekday() != 6), minimum=9,
        mapping="Singapore PUBLIC named dates including polling holidays; Sunday nominal dates replaced by library's observed weekday, Saturday nominal dates retained. Native XSES independently confirms every business-day classification.",
        primary=["https://www.mom.gov.sg/employment-practices/public-holidays", "https://www.sgx.com/stock-exchange/trading"])
    sources["singapore_exchange"] = _source(sgx)
    for date in _dates():
        assert refs["singapore"].is_business_day(date) == (date.weekday() < 5 and date not in sgx), date
    refs["singapore"].metadata["estimated_named_dates"] = [
        date.isoformat() for date, name in sorted(sg.items()) if "estimated" in name.lower()
    ]

    lu = holidays.Luxembourg(categories=(PUBLIC, BANK), **kwargs)
    # The provider's BANK category also lists partial closures. BCL and bank
    # opening schedules identify Christmas Eve afternoon and New Year's Eve
    # afternoon; neither is a whole-day closure in the crate's date-only model.
    partial = {date for date, name in lu.items() if name in ("Christmas Eve (afternoon)", "New Year's Eve")}
    add("luxembourg", lu, set(lu) - partial, minimum=10,
        mapping="Luxembourg PUBLIC+BANK named holidays, excluding explicitly partial Christmas Eve and New Year's Eve closures; Good Friday retained.",
        primary=["https://www.bcl.lu/fr/apropos/jours_douverture/index.html", "https://www.ing.lu/webing/content/siteing/fr/private-banking/contacts-and-information/information/more-information/bank-closing-days.html"])

    federal = holidays.UnitedStates(categories=GOVERNMENT, observed=False, **kwargs)
    federal_names = {
        "New Year's Day", "Birthday of Martin Luther King, Jr.", "Washington's Birthday",
        "Memorial Day", "Juneteenth National Independence Day", "Independence Day",
        "Labor Day", "Columbus Day", "Veterans Day", "Thanksgiving Day", "Christmas Day",
    }
    # Government office closures such as Carter's mourning day do not close
    # Federal Reserve payment services. Match the Reserve Bank's named eleven
    # holidays, then apply its explicitly published Sunday-only observation.
    fed_dates = set()
    for date, name in federal.items():
        if name in federal_names:
            fed_dates.add(date + dt.timedelta(days=1) if date.weekday() == 6 else date)
    add("usfed", federal, fed_dates, minimum=10,
        mapping="US GOVERNMENT nominal federal holiday dates filtered to the Reserve Bank's eleven names; Sunday moves to Monday, Saturday retained with no prior-Friday observation. Juneteenth raw holiday begins2021; first extra weekday closure2022.",
        primary=["https://www.frbservices.org/about/holiday-schedules/", "https://www.federalreserve.gov/newsevents/pressreleases/other20210617b.htm"])

    # QuantLib supplies a second independently executed reference where the
    # selected market and full horizon actually match its implementation.
    cross = {
        "target2": ql.TARGET(),
        "usnyse": ql.UnitedStates(ql.UnitedStates.NYSE),
        "usfed": ql.UnitedStates(ql.UnitedStates.FederalReserve),
        "uk": ql.UnitedKingdom(ql.UnitedKingdom.Exchange),
        "japan": ql.Japan(),
    }
    for key, calendar in cross.items():
        for date in _dates():
            actual = calendar.isBusinessDay(ql.Date(date.day, date.month, date.year))
            assert refs[key].is_business_day(date) == actual, (key, date, actual)
        refs[key].metadata["additional_business_reference"] = "QuantLib 1.44 " + calendar.name() + ": all2020–2040 dates agree"

    # BVMF's PUBLIC set omits B3's full-day December 24/31 trading
    # closures. Use the exact exchange calendar, checked against B3's complete
    # official 2026 circular, rather than silently treating those dates as open.
    b3 = ql.Brazil(ql.Brazil.Exchange)
    b3_first = dt.date(2026, 1, 1)
    b3_dates = {b3_first + dt.timedelta(days=offset) for offset in range(365)
                if (b3_first + dt.timedelta(days=offset)).weekday() < 5
                and not b3.isBusinessDay(ql.Date((b3_first + dt.timedelta(days=offset)).day,
                                               (b3_first + dt.timedelta(days=offset)).month, 2026))}
    refs["brazilb3"] = CalendarReference("brazilb3", b3_dates, {
        "reference": "QuantLib 1.44 Brazil.Exchange",
        "available_years": [2026, 2026],
        "minimum_named_holidays_per_year": 14,
        "annual_named_holiday_counts": {"2026": len(b3_dates)},
        "mapping": "B3 published weekday full-day trading closures; nominal weekend holidays omitted and partial sessions remain open.",
        "primary_sources": ["https://www.b3.com.br/pt_br/noticias/calendario-de-negociacao-da-b3-confira-o-funcionamento-da-bolsa-em-2026.htm"],
        "alternative_reference_limitation": "holidays 0.107 BVMF PUBLIC set omits December 24 and December 31 listed-market full-day closures.",
    })
    sources["brazilb3"] = {
        "module": "ql/time/calendars/brazil.cpp",
        "source": "https://github.com/lballabio/QuantLib/blob/v1.44/ql/time/calendars/brazil.cpp",
    }
    metadata = {
        "package": "holidays==0.107", "package_url": "https://pypi.org/project/holidays/0.107/",
        "wheel_sha256": "5e4582d543b145d8ebb80ce8943e60ea5c8b6a30b74201ad0f95af0cbe67cb14",
        "source_archive_sha256": "2f02cdfbe41a5f14e053590f2385a932fe42f0c692aed75cb55601ad37cebb4d",
        "source_tag": "https://github.com/vacanza/holidays/tree/v0.107",
        "source_files": sources,
        "calendars": {key: dict(ref.metadata, rust_variant=VARIANTS[key]) for key, ref in refs.items()},
        "coverage": "Each available civil day: 2020–2040 for established snapshots, 2026 only for B3. Future unpublished dates remain projections.",
        "known_incomplete_alternatives": {
            "QuantLib.HongKong": "Annual lunar tables stop 2026 in v1.44; later missing rows cannot validate the full 2020–2040 horizon.",
            "QuantLib.Singapore": "Annual lunar/religious tables stop 2027 in v1.44; later missing rows cannot validate the full 2020–2040 horizon.",
            "QuantLib.Switzerland": "Banking settlement calendars omit SIX full trading closures on December24 and December31.",
        },
    }
    assert set(refs) == set(VARIANTS)
    return refs, metadata


MIN_DATE = dt.date(1583, 1, 1)
MAX_DATE = dt.date(9999, 12, 31)
MAX_STEPS = 73_000
ROLL_LIMIT = 366
POLICIES = ("VerifiedOnly", "AllowProjected")
ROLLS = ("Unadjusted", "Following", "ModifiedFollowing", "Preceding",
         "ModifiedPreceding", "Nearest", "EndOfMonth")
# Bounds are the documented contract, independently asserted against the Rust
# metadata and against queries immediately inside/outside each boundary.
MARKETS = {
    "Target2": ("target2", "target2", (2002, 9999), (2002, 2028)),
    "Luxembourg": ("luxembourg", "luxembourg", (2019, 9999), (2026, 2027)),
    "UnitedStatesNyse": ("usnyse", "united_states", (2020, 2040), (2026, 2028)),
    "UnitedStatesFederalReserve": ("usfed", "united_states", (2020, 2040), (2026, 2030)),
    "UnitedKingdom": ("uk", "united_kingdom", (2020, 2040), (2020, 2028)),
    "Japan": ("japan", "japan", (2020, 2040), (2026, 2027)),
    "Switzerland": ("switzerland", "switzerland", (2020, 2040), (2026, 2027)),
    "HongKong": ("hongkong", "hong_kong", (2020, 2040), (2026, 2027)),
    "Singapore": ("singapore", "singapore", (2020, 2040), (2026, 2027)),
    "BrazilB3": ("brazilb3", "brazil", (2026, 2026), (2026, 2026)),
}
SOURCE = "Python datetime + holidays 0.107 / QuantLib 1.44 independent market sets + public coverage contract"
RULE_SOURCE = "Python datetime + python-dateutil Easter + ECB / Luxembourg standing holiday rules"
CONTRACT_SOURCE = "Explicit reviewed 2.0 public contract; independent Python expected values (no Rust imports or parsing)"


class CalendarFailure(Exception):
    def __init__(self, kind, *payload):
        self.kind, self.payload = kind, payload
        super().__init__(kind, *payload)

    def encoded(self):
        return ":".join(("Err", "CalendarError", self.kind, *map(str, self.payload)))


@lru_cache(maxsize=1)
def _references():
    return build_calendar_references()[0]


@lru_cache(maxsize=None)
def _rule_dates(calendar, year):
    # Published named closures as a set, with Easter supplied by dateutil's
    # algorithm rather than the crate's Meeus implementation.
    offsets = (-2, 1) if calendar == "Target2" else (-2, 1, 39, 50)
    fixed = ((1, 1), (5, 1), (12, 25), (12, 26))
    if calendar == "Luxembourg":
        fixed += ((5, 9), (6, 23), (8, 15), (11, 1))
    sunday = dt.date(*western_holidays.easter(year, church="western"))
    return frozenset({dt.date(year, month, day) for month, day in fixed}
                     | {sunday + dt.timedelta(days=offset) for offset in offsets})


def _ensure(calendar, date, policy):
    _, _, (lo, hi), (verified_lo, verified_hi) = MARKETS[calendar]
    if not lo <= date.year <= hi:
        raise CalendarFailure("OutOfCoverage", calendar, date)
    if policy == "VerifiedOnly" and not verified_lo <= date.year <= verified_hi:
        raise CalendarFailure("UnverifiedDate", calendar, date)


def _holiday(calendar, date, policy):
    _ensure(calendar, date, policy)
    key = MARKETS[calendar][0]
    reference = _references()[key]
    if reference.start <= date <= reference.end:
        return reference.is_holiday(date)
    # Only the documented rule calendars have availability outside 2020–2040.
    assert calendar in ("Target2", "Luxembourg"), (calendar, date)
    return date in _rule_dates(calendar, date.year)


def _query(markets, date, policy, business=False):
    # Evaluate all markets before deciding whether a union is closed. The
    # second leg's absence must never be masked by a first-leg holiday/weekend.
    named_closures = [_holiday(market, date, policy) for market in markets]
    holiday = any(named_closures)
    return date.weekday() < 5 and not holiday if business else holiday


def _move(date, direction):
    ordinal = date.toordinal() + direction
    if ordinal < MIN_DATE.toordinal() or ordinal > MAX_DATE.toordinal():
        raise CalendarFailure("DateBoundary", date)
    return dt.date.fromordinal(ordinal)


def _following(markets, date, policy, direction):
    for offset in range(ROLL_LIMIT + 1):
        if _query(markets, date, policy, True):
            return date
        if offset != ROLL_LIMIT:
            date = _move(date, direction)
    raise CalendarFailure("WalkLimitExceeded", ROLL_LIMIT)


def _adjust(markets, date, roll, policy):
    business = _query(markets, date, policy, True)
    if roll == "Unadjusted":
        return date
    if roll in ("Following", "Preceding"):
        return _following(markets, date, policy, 1 if roll == "Following" else -1)
    if roll == "Nearest":
        if business:
            return date
        forward, backward = date, date
        for _ in range(ROLL_LIMIT):
            forward = _move(forward, 1)
            if _query(markets, forward, policy, True):
                return forward
            backward = _move(backward, -1)
            if _query(markets, backward, policy, True):
                return backward
        raise CalendarFailure("WalkLimitExceeded", ROLL_LIMIT)
    direction = -1 if roll == "ModifiedPreceding" else 1
    # Search only this month in the preferred direction. Calendar data beyond
    # the month is irrelevant to the contractual fallback.
    candidate = date
    while candidate.month == date.month:
        if _query(markets, candidate, policy, True):
            return candidate
        ordinal = candidate.toordinal() + direction
        if not MIN_DATE.toordinal() <= ordinal <= MAX_DATE.toordinal():
            break
        candidate = dt.date.fromordinal(ordinal)
    return _following(markets, date, policy, -direction)


def _count(markets, start, end, policy):
    _query(markets, start, policy, True)
    if start >= end:
        _query(markets, end, policy, True)
        return 0
    # Half-open interval: validate exactly those dates that can be counted.
    # The exclusive endpoint may be outside the market's available horizon.
    business = 0
    for offset in range((end - start).days):
        if offset == MAX_STEPS:
            raise CalendarFailure("WalkLimitExceeded", MAX_STEPS)
        date = start + dt.timedelta(days=offset)
        business += _query(markets, date, policy, True)
    return business


def _add(markets, date, number, policy):
    _query(markets, date, policy, True)
    if number == 0:
        return _following(markets, date, policy, 1)
    direction, remaining = (1 if number > 0 else -1), abs(number)
    for _ in range(MAX_STEPS):
        date = _move(date, direction)
        if _query(markets, date, policy, True):
            remaining -= 1
            if not remaining:
                return date
    raise CalendarFailure("WalkLimitExceeded", MAX_STEPS)


def _expected(operation, args, markets=None):
    markets = markets or [args["calendar"]]
    policy = args.get("policy", "VerifiedOnly")
    date = dt.date.fromisoformat(args["date"]) if "date" in args else None
    try:
        if operation == "try_is_holiday":
            return "Ok:bool:" + str(_query(markets, date, policy)).lower()
        if operation == "try_is_business_day":
            return "Ok:bool:" + str(_query(markets, date, policy, True)).lower()
        if operation in ("try_adjust", "try_next_business_day", "try_previous_business_day"):
            roll = args.get("roll", "Following" if operation == "try_next_business_day" else "Preceding")
            return "Ok:date:" + str(_adjust(markets, date, roll, policy))
        if operation == "try_add_business_days":
            return "Ok:date:" + str(_add(markets, date, int(args["n"]), policy))
        start, end = (dt.date.fromisoformat(args[key]) for key in ("start", "end"))
        if operation == "try_business_days_between":
            return "Ok:int:" + str(_count(markets, start, end, policy))
        assert operation in ("try_bus_252_for_calendar", "try_bus_252"), operation
        lo, hi, sign = (end, start, -1) if end < start else (start, end, 1)
        return "Ok:f64:" + repr(sign * _count(markets, lo, hi, policy) / 252.0)
    except CalendarFailure as error:
        return error.encoded()


def _error_value(kind, calendar="Target2", date="2029-01-01", limit=MAX_STEPS):
    date = dt.date.fromisoformat(date)
    debug_date = f"Date {{ year: {date.year}, month: {date.month}, day: {date.day} }}"
    if kind == "EmptyComposite":
        return "a composite calendar requires at least one market", "EmptyComposite"
    if kind == "WalkLimitExceeded":
        return f"calendar walk exceeded {limit} steps", f"WalkLimitExceeded {{ limit: {limit} }}"
    if kind == "DateBoundary":
        return f"calendar walk reached date boundary at {debug_date}", f"DateBoundary {{ date: {debug_date} }}"
    reason = "has no coverage" if kind == "OutOfCoverage" else "is unverified"
    return (f"{calendar} {reason} for {debug_date}",
            f"{kind} {{ calendar: {calendar}, date: {debug_date} }}")


def cases():
    """Every checked calendar API; success, rejection and traversal contracts."""
    result = []

    def emit(api, args, expected=None, operation=None, markets=None, source=SOURCE):
        if expected is None:
            expected = _expected(operation or api.rsplit("::", 1)[-1].removesuffix("_with_policy"), args, markets)
        result.append(dict(id=f"calendar-contract-{len(result):05d}", api=api,
                           args=args, expected=expected, source=source))

    # Every metadata field is checked separately as well as the compound value.
    emit("calendar::CALENDAR_MAX_STEPS", {}, "Ok:int:73000", source=CONTRACT_SOURCE)
    emit("calendar::CoveragePolicy::default", {}, "text:VerifiedOnly", source=CONTRACT_SOURCE)
    constant_seen = set()
    for calendar, (_, module, available, verified) in MARKETS.items():
        emit("calendar::coverage", {"calendar": calendar},
             f"text:available={available[0]},{available[1]}|verified={verified[0]},{verified[1]}|verified_on=2026-10-23",
             source=CONTRACT_SOURCE)
        for field, value in (("available_years", f"text:{available[0]},{available[1]}"),
                             ("verified_years", f"text:{verified[0]},{verified[1]}"),
                             ("verified_on", "text:2026-10-23")):
            emit(f"calendar::CalendarCoverage::{field}", {"calendar": calendar}, value, source=CONTRACT_SOURCE)
        prefix = "NYSE_" if calendar == "UnitedStatesNyse" else "FEDERAL_RESERVE_" if calendar == "UnitedStatesFederalReserve" else ""
        values = {prefix + "VERIFIED_YEARS": f"text:{verified[0]},{verified[1]}",
                  prefix + "VERIFIED_ON": "text:2026-10-23"}
        if module not in ("target2", "luxembourg"):
            values["COVERAGE"] = f"text:{available[0]},{available[1]}"
        if module not in ("target2", "luxembourg", "brazil"):
            snapshot = "2026-05-23" if module in ("united_states", "united_kingdom", "japan") else "2026-10-23"
            values["SNAPSHOT_DATE"] = "text:" + snapshot
        for name, value in values.items():
            api = f"calendar::{module}::{name}"
            if api not in constant_seen:
                emit(api, {}, value, source=CONTRACT_SOURCE)
                constant_seen.add(api)

        # Deliberately include weekday, weekend, named holiday, observed holiday,
        # exceptional closure, month boundary and verified/available boundaries.
        dates = {dt.date(2026, 1, 1), dt.date(2026, 1, 2), dt.date(2026, 1, 3),
                 dt.date(2026, 2, 17), dt.date(2026, 2, 18), dt.date(2026, 4, 3),
                 dt.date(2026, 4, 30), dt.date(2026, 5, 1), dt.date(2026, 5, 2),
                 dt.date(2026, 5, 3), dt.date(2026, 5, 31), dt.date(2026, 7, 3),
                 dt.date(2026, 7, 4), dt.date(2026, 8, 1), dt.date(2026, 9, 7),
                 dt.date(2026, 12, 24), dt.date(2026, 12, 25), dt.date(2026, 12, 26),
                 dt.date(2026, 12, 31), dt.date(2025, 1, 9), MIN_DATE, MAX_DATE}
        if calendar in ("Target2", "Luxembourg"):
            # convertdate's USNO Gregorian calculation supports the full Date
            # domain; dateutil documents Western Easter only through 4099.
            for year in (4100, 9999):
                sunday = dt.date(*western_holidays.easter(year, church="western"))
                dates.update(sunday + dt.timedelta(days=offset)
                             for offset in (-3, -2, -1, 0, 1, 2, 38, 39, 40, 49, 50, 51))
        for lo, hi in (available, verified):
            dates.update((dt.date(lo, 1, 1), dt.date(hi, 12, 31)))
            if lo > MIN_DATE.year:
                dates.add(dt.date(lo - 1, 12, 31))
            if hi < MAX_DATE.year:
                dates.add(dt.date(hi + 1, 1, 1))
        # Include at least one real holiday and an open weekday from every year
        # in the independently provided table horizon, not just the audit year.
        reference = _references()[MARKETS[calendar][0]]
        for year in range(reference.start.year, reference.end.year + 1):
            annual = sorted(date for date in reference.named_dates if date.year == year)
            dates.update((annual[0], annual[-1], dt.date(year, 6, 15)))

        for date in sorted(dates):
            base = {"calendar": calendar, "date": str(date)}
            for operation in ("try_is_holiday", "try_is_business_day"):
                emit("calendar::" + operation, base)
                for policy in POLICIES:
                    emit("calendar::" + operation + "_with_policy", dict(base, policy=policy))
            convenience = "try_is_nyse_holiday" if calendar == "UnitedStatesNyse" else "try_is_federal_reserve_holiday" if calendar == "UnitedStatesFederalReserve" else "try_is_holiday"
            emit(f"calendar::{module}::{convenience}", {"date": str(date)},
                 _expected("try_is_holiday", base))

        walks = ["2026-01-01", "2026-01-03", "2026-04-30", "2026-05-01",
                 "2026-05-31", "2026-09-07", "2026-12-31",
                 str(dt.date(available[0], 1, 1)), str(dt.date(available[1], 12, 31)),
                 str(dt.date(verified[0], 1, 1)), str(dt.date(verified[1], 12, 31)),
                 "2025-12-25", "2045-12-25", str(MIN_DATE), str(MAX_DATE)]
        for date in dict.fromkeys(walks):
            for policy in (None, *POLICIES):
                suffix = "_with_policy" if policy else ""
                base = {"calendar": calendar, "date": date}
                if policy:
                    base["policy"] = policy
                for roll in ROLLS:
                    emit("calendar::try_adjust" + suffix, dict(base, roll=roll), operation="try_adjust")
                for operation in ("try_next_business_day", "try_previous_business_day"):
                    emit("calendar::" + operation + suffix, base, operation=operation)
                for n in (-5, -1, 0, 1, 5):
                    emit("calendar::try_add_business_days" + suffix, dict(base, n=n), operation="try_add_business_days")

        intervals = [("2026-09-01", "2026-09-10"), ("2026-09-10", "2026-09-01"),
                     ("2026-01-03", "2026-01-03"), ("2026-12-31", "2027-01-01"),
                     ("2026-01-01", "2027-01-01"), ("2026-12-30", "2027-01-02"),
                     ("2027-01-01", "2027-01-01"), ("2027-01-01", "2026-12-31"),
                     (str(dt.date(available[0], 1, 1)), str(dt.date(available[0], 1, 2))),
                     (str(dt.date(available[1], 12, 30)), str(dt.date(available[1], 12, 31))),
                     ("2025-12-31", "2026-01-03"), ("2040-12-30", "2041-01-02"),
                     (str(MIN_DATE), str(MIN_DATE)), (str(MAX_DATE), str(MAX_DATE))]
        for start, end in intervals:
            for operation in ("try_business_days_between", "try_bus_252_for_calendar"):
                for policy in (None, *POLICIES):
                    args = {"calendar": calendar, "start": start, "end": end}
                    if policy:
                        args["policy"] = policy
                    emit("calendar::" + operation + ("_with_policy" if policy else ""), args, operation=operation)

    # Weekday classification is independent of every market's coverage.
    for start in (MIN_DATE, dt.date(2000, 2, 27), dt.date(2026, 5, 18), dt.date(9999, 12, 25)):
        for offset in range(min(7, (MAX_DATE - start).days + 1)):
            date = start + dt.timedelta(days=offset)
            emit("calendar::is_weekend", {"date": str(date)},
                 "Ok:bool:" + str(date.weekday() >= 5).lower(), source="Python datetime.date.weekday")

    # Limits are exercised through public named calendars, not private test
    # predicates: rule calendars provide sufficient real date coverage.
    for calendar in ("Target2", "Luxembourg"):
        for n in (2_147_483_647, -2_147_483_648):
            args = {"calendar": calendar, "date": "2300-01-02", "n": n, "policy": "AllowProjected"}
            emit("calendar::try_add_business_days_with_policy", args, source=RULE_SOURCE)
        start = dt.date(2300, 1, 2)
        for length in (MAX_STEPS - 1, MAX_STEPS, MAX_STEPS + 1):
            end = start + dt.timedelta(days=length)
            for operation in ("try_business_days_between", "try_bus_252_for_calendar"):
                for reverse in (False, True):
                    a, b = (end, start) if reverse else (start, end)
                    emit("calendar::" + operation + "_with_policy",
                         {"calendar": calendar, "start": str(a), "end": str(b), "policy": "AllowProjected"},
                         source=RULE_SOURCE)
        for date, n in ((MAX_DATE, 1), (MAX_DATE, 2_147_483_647)):
            emit("calendar::try_add_business_days_with_policy",
                 {"calendar": calendar, "date": str(date), "n": n, "policy": "AllowProjected"}, source=RULE_SOURCE)

    # Every composite method is called on BOTH concrete public types. Repeated
    # markets and supplied order are part of the constructor/accessor contract.
    combinations = ([], ["Target2"], ["Target2", "UnitedStatesNyse"],
                    ["UnitedStatesNyse", "Target2", "UnitedStatesNyse"],
                    ["Target2", "BrazilB3"], ["BrazilB3", "Target2"])
    for kind in ("Composite", "JointBusiness"):
        prefix = f"calendar::composite::{kind}::"
        for markets in combinations:
            encoded = ",".join(markets)
            constructor = "text:" + encoded if markets else "Err:CalendarError:EmptyComposite"
            emit(prefix + "new", {"markets": encoded}, constructor, source=CONTRACT_SOURCE)
            if not markets:
                continue
            emit(prefix + "kinds", {"markets": encoded}, "text:" + encoded, source=CONTRACT_SOURCE)
            emit(prefix + "Clone", {"markets": encoded}, "text:" + encoded, source=CONTRACT_SOURCE)
            emit(prefix + "Debug", {"markets": encoded},
                 "text:" + kind + " { kinds: [" + ", ".join(markets) + "] }", source=CONTRACT_SOURCE)
            for policy in (None, *POLICIES):
                suffix = "_with_policy" if policy else ""
                for date in ("2026-01-01", "2026-07-03", "2026-07-04", "2026-07-06",
                             "2026-09-07", "2026-12-31", "2027-12-25", "2025-12-25", str(MAX_DATE)):
                    base = {"markets": encoded, "date": date}
                    if policy:
                        base["policy"] = policy
                    for operation in ("try_is_holiday", "try_is_business_day"):
                        emit(prefix + operation + suffix, base, operation=operation, markets=markets)
                    for roll in ROLLS:
                        emit(prefix + "try_adjust" + suffix, dict(base, roll=roll), operation="try_adjust", markets=markets)
                    for n in (-1, 0, 1):
                        emit(prefix + "try_add_business_days" + suffix, dict(base, n=n), operation="try_add_business_days", markets=markets)
                for start, end in (("2026-09-01", "2026-09-10"), ("2026-09-10", "2026-09-01"),
                                   ("2026-01-01", "2027-01-01"), ("2026-12-31", "2027-01-01"),
                                   ("2026-12-31", "2027-01-02"), ("2027-01-01", "2027-01-01"),
                                   ("2026-07-04", "2026-07-04"), ("2027-01-01", "2026-12-31")):
                    args = {"markets": encoded, "start": start, "end": end}
                    if policy:
                        args["policy"] = policy
                    for operation in ("try_business_days_between", "try_bus_252"):
                        emit(prefix + operation + suffix, args, operation=operation, markets=markets)

        # The shared implementation does not exempt either public facade from
        # observable cap and date-boundary contracts.
        for n in (2_147_483_647, -2_147_483_648):
            emit(prefix + "try_add_business_days_with_policy",
                 {"markets": "Target2", "date": "2300-01-02", "n": n, "policy": "AllowProjected"},
                 operation="try_add_business_days", markets=["Target2"], source=RULE_SOURCE)
        emit(prefix + "try_add_business_days_with_policy",
             {"markets": "Target2", "date": str(MAX_DATE), "n": 1, "policy": "AllowProjected"},
             operation="try_add_business_days", markets=["Target2"], source=RULE_SOURCE)
        for length in (MAX_STEPS, MAX_STEPS + 1):
            start = dt.date(2300, 1, 2)
            end = start + dt.timedelta(days=length)
            for operation in ("try_business_days_between", "try_bus_252"):
                emit(prefix + operation + "_with_policy",
                     {"markets": "Target2", "start": str(start), "end": str(end), "policy": "AllowProjected"},
                     operation=operation, markets=["Target2"], source=RULE_SOURCE)

    for kind in ("EmptyComposite", "OutOfCoverage", "UnverifiedDate", "DateBoundary", "WalkLimitExceeded"):
        args = {"kind": kind, "calendar": "UnitedKingdom", "date": "2045-12-25", "limit": MAX_STEPS}
        display, debug = _error_value(kind, args["calendar"], args["date"], args["limit"])
        emit("calendar::CalendarError::Display", args, "text:" + display, source=CONTRACT_SOURCE)
        emit("calendar::CalendarError::Debug", args, "text:" + debug, source=CONTRACT_SOURCE)
        emit("calendar::CalendarError::source", args, "text:None", source=CONTRACT_SOURCE)
        emit("calendar::CalendarError::Clone", args, "Ok:bool:true", source=CONTRACT_SOURCE)
        for same in (0, 1):
            emit("calendar::CalendarError::PartialEq", dict(args, same=same),
                 "Ok:bool:" + str(bool(same)).lower(), source=CONTRACT_SOURCE)
    for calendar, (_, _, available, verified) in MARKETS.items():
        emit("calendar::Calendar::Debug", {"calendar": calendar}, "text:" + calendar, source=CONTRACT_SOURCE)
        emit("calendar::Calendar::Clone", {"calendar": calendar}, "text:" + calendar, source=CONTRACT_SOURCE)
        for other in MARKETS:
            emit("calendar::Calendar::PartialEq", {"calendar": calendar, "other": other},
                 "Ok:bool:" + str(calendar == other).lower(), source=CONTRACT_SOURCE)
            emit("calendar::Calendar::Hash", {"calendar": calendar, "other": other},
                 "Ok:bool:true", source=CONTRACT_SOURCE)
        debug = (f"CalendarCoverage {{ available_years: ({available[0]}, {available[1]}), "
                 f"verified_years: Some(({verified[0]}, {verified[1]})), verified_on: Some(\"2026-10-23\") }}")
        emit("calendar::CalendarCoverage::Debug", {"calendar": calendar}, "text:" + debug, source=CONTRACT_SOURCE)
        emit("calendar::CalendarCoverage::Clone", {"calendar": calendar}, "Ok:bool:true", source=CONTRACT_SOURCE)
        for other in MARKETS:
            emit("calendar::CalendarCoverage::PartialEq", {"calendar": calendar, "other": other},
                 "Ok:bool:" + str(MARKETS[calendar][2:] == MARKETS[other][2:]).lower(), source=CONTRACT_SOURCE)
    for policy in POLICIES:
        emit("calendar::CoveragePolicy::Debug", {"policy": policy}, "text:" + policy, source=CONTRACT_SOURCE)
        emit("calendar::CoveragePolicy::Clone", {"policy": policy}, "text:" + policy, source=CONTRACT_SOURCE)
        for other in POLICIES:
            emit("calendar::CoveragePolicy::PartialEq", {"policy": policy, "other": other},
                 "Ok:bool:" + str(policy == other).lower(), source=CONTRACT_SOURCE)
            emit("calendar::CoveragePolicy::Hash", {"policy": policy, "other": other},
                 "Ok:bool:true", source=CONTRACT_SOURCE)
    return result


def reference_calendars(ql, refs):
    """Use QuantLib's algorithms over independent external holiday data."""
    result = {"weekends": ql.WeekendsOnly()}
    for name, reference in refs.items():
        first, last = reference.start, reference.end
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
            if name != "weekends" and not refs[name].start + dt.timedelta(days=40) <= point <= refs[name].end - dt.timedelta(days=40):
                continue
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
        ("target2",), ("target2", "usnyse"), ("target2", "luxembourg"),
        ("usnyse", "uk"), ("japan", "hongkong", "singapore"),
    )
    composite_counts = {}
    for names in composite_sets:
        if any(name not in calendars for name in names):
            raise RuntimeError("Missing composite reference calendars: " + repr(names))
        # Native QL joint-calendar evaluation determines joint business days.
        if len(names) == 1:
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
