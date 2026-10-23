"""Independent full-window market references for the Python oracle.

Financial sets execute pinned holidays 0.107 market implementations. QuantLib
1.44 additionally checks rule-complete calendars. Its regional calendar tables
and Swiss banking calendar are not substituted for the advertised exchanges.
Dates after official publication remain explicit
projections, even where independent implementations agree.
"""

import datetime as dt
import hashlib
import inspect
from pathlib import Path

import holidays
from holidays.constants import BANK, GOVERNMENT, OPTIONAL, PUBLIC
import QuantLib as ql


HOLIDAYS_VERSION = "0.107"
START = dt.date(2020, 1, 1)
END = dt.date(2040, 12, 31)
VARIANTS = {
    "target2": "Target2", "usnyse": "UnitedStatesNyse",
    "usfed": "UnitedStatesFederalReserve", "uslegacy": "UnitedStates",
    "uk": "UnitedKingdom", "japan": "Japan", "switzerland": "Switzerland",
    "hongkong": "HongKong", "singapore": "Singapore", "luxembourg": "Luxembourg",
}


class CalendarReference:
    """A named-date set with explicit bounds, never an out-of-range default-open."""

    def __init__(self, name, named_dates, metadata):
        self.name = name
        self.named_dates = frozenset(d for d in named_dates if START <= d <= END)
        self.metadata = metadata
        for year in range(2020, 2041):
            count = sum(date.year == year for date in self.named_dates)
            assert count >= metadata["minimum_named_holidays_per_year"], (name, year, count)

    def _check(self, date):
        if not START <= date <= END:
            raise ValueError(f"{self.name} reference outside explicit 2020–2040 coverage: {date}")

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
    """Return `(refs, metadata)` for all nine unique markets and the US alias.

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

    def add(key, entity, named_dates=None, mapping="native PUBLIC financial named-date set", minimum=6, primary=()):
        dates = set(entity) if named_dates is None else set(named_dates)
        sources[key] = _source(entity)
        meta = {
            "rust_variant": VARIANTS[key], "reference": type(entity).__name__,
            "mapping": mapping, "available_years": [2020, 2040],
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

    refs["uslegacy"] = refs["usnyse"]
    metadata = {
        "package": "holidays==0.107", "package_url": "https://pypi.org/project/holidays/0.107/",
        "wheel_sha256": "5e4582d543b145d8ebb80ce8943e60ea5c8b6a30b74201ad0f95af0cbe67cb14",
        "source_archive_sha256": "2f02cdfbe41a5f14e053590f2385a932fe42f0c692aed75cb55601ad37cebb4d",
        "source_tag": "https://github.com/vacanza/holidays/tree/v0.107",
        "source_files": sources,
        "calendars": {key: dict(ref.metadata, rust_variant=VARIANTS[key]) for key, ref in refs.items()},
        "coverage": "Every civil day2020–2040 inclusive, independently named holidays and business-day classifications; future dates remain projections.",
        "known_incomplete_alternatives": {
            "QuantLib.HongKong": "Annual lunar tables stop 2026 in v1.44; later missing rows cannot validate the full 2020–2040 horizon.",
            "QuantLib.Singapore": "Annual lunar/religious tables stop 2027 in v1.44; later missing rows cannot validate the full 2020–2040 horizon.",
            "QuantLib.Switzerland": "Banking settlement calendars omit SIX full trading closures on December24 and December31.",
        },
    }
    assert set(refs) == set(VARIANTS)
    return refs, metadata
