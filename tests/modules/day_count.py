"""Independent financial oracle: vendor SDK references and public contracts.

Vendor calculations execute pinned QuantLib and FinancePy. Full-date-domain
formulas use Python datetime/calendar, exact Fraction arithmetic and math.fsum,
with independent definitions documented at https://strata.opengamma.io/day_counts/.
No Rust implementation or Rust-generated expected results are consumed here.
"""

import bisect
import calendar
from fractions import Fraction
from itertools import product
import math
from dateutil.relativedelta import relativedelta

import contextlib
import datetime as dt
import hashlib
import inspect
from importlib.metadata import version
import io
from pathlib import Path

import QuantLib as ql

# FinancePy prints a startup banner on import; keep the oracle's output focused.
with contextlib.redirect_stdout(io.StringIO()):
    from financepy.utils.date import Date as FinanceDate
    from financepy.utils.day_count import DayCount, DayCountTypes
    from financepy.utils.frequency import FrequencyTypes
    import financepy.utils.date as finance_date_module
    import financepy.utils.day_count as finance_day_count_module
    import financepy.utils.frequency as finance_frequency_module


FINANCEPY_VERSION = "1.1.2"
SOURCE_COMMIT = "80b1e8c5dad16788393f788f6b9aa67019823138"
SOURCE_HASHES = {
    "day_count.py": "0b113dc2eb5925d637c3a76ec8a189b8e0361b8d28927c8396c52c1e041d8403",
    "frequency.py": "27f95b53dd1b99df9ba7e34077b89b7354b77f56fd307095c8f6a5cbacfbf2ad",
    "date.py": "46a3dfd4632726206b567424a8d85203e8686e0e51a8bb2f7a6807570dbcbf0f",
}
ACT_365L = DayCount(DayCountTypes.ACT_365L)
ONE_ONE = ql.OneDayCounter()
BUS_252 = ql.Business252(ql.TARGET())


def finance_date(value):
    return FinanceDate(value.day, value.month, value.year)


def quantlib_date(value):
    return ql.Date(value.day, value.month, value.year)


def evaluate(row):
    start, end = (dt.date.fromisoformat(row[index]) for index in (3, 4))
    lower, upper = min(start, end), max(start, end)
    if row[1] == "bus252":
        result = BUS_252.yearFraction(quantlib_date(lower), quantlib_date(upper))
        return -result if start > end else result
    if row[2] == "oneone":
        # The Rust basis is constant in either orientation. Only reference
        # inputs are ordered; Rust still receives both original endpoints.
        return ONE_ONE.yearFraction(quantlib_date(lower), quantlib_date(upper))
    if row[2] == "act365l":
        frequency = int(row[7])
        # FinancePy exposes no bimonthly enum. Its Act/365L method distinguishes
        # annual from nonannual frequency, so MONTHLY selects the correct
        # reference denominator class for six coupons/year. The contractual
        # next coupon and original frequency remain unchanged in the fixture.
        reference_frequency = FrequencyTypes.MONTHLY if frequency == 6 else FrequencyTypes(frequency)
        next_coupon = dt.date.fromisoformat(row[6])
        result, _, _ = ACT_365L.year_frac(
            finance_date(lower), finance_date(upper), dt3=finance_date(next_coupon),
            freq_type=reference_frequency,
        )
        return -result if start > end else result
    raise ValueError("Unsupported Python day-count reference: " + repr(row[:4]))


def provenance():
    assert version("financepy") == FINANCEPY_VERSION, version("financepy")
    sources = {}
    for module in (finance_date_module, finance_day_count_module, finance_frequency_module):
        path = Path(inspect.getfile(module))
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        assert digest == SOURCE_HASHES[path.name], (path.name, digest)
        sources[path.name] = {
            "sha256": digest,
            "url": f"https://github.com/domokane/FinancePy/blob/{SOURCE_COMMIT}/financepy/utils/{path.name}",
        }
    return {
        "financepy": {
            "package": "financepy==1.1.2",
            "package_url": "https://pypi.org/project/financepy/1.1.2/",
            "wheel_sha256": "3c32578b81f338741ac135bb05ff9aa9164d75f6aa89c4d5e7f6a96c8b6f37d9",
            "source_archive_sha256": "ab239301cdab901dff5ac781979d3e192db63151b11b768335d3a28b0ebed289",
            "source_commit": SOURCE_COMMIT,
            "source_files": sources,
            "mapping": "DayCount(DayCountTypes.ACT_365L).year_frac(ordered_start, ordered_end, dt3=next_coupon, freq_type=frequency)[0]; reverse accruals negate the ordered result",
            "frequency_6_mapping": "MONTHLY selects FinancePy's nonannual Act/365L class; supplied next coupon remains contractual and fixture frequency remains 6",
        },
        "quantlib-mapped": {
            "package": "QuantLib==1.44",
            "mappings": {
                "oneone": "OneDayCounter().yearFraction(min(start,end),max(start,end)); original endpoints are preserved for Rust's constant-one basis",
                "bus252": "Business252(TARGET()).yearFraction(min(start,end),max(start,end)), negated for reversed intervals to map the signed lower-inclusive/upper-exclusive contract",
            },
        },
    }


def execute(rows):
    metadata = provenance()
    counts = {"financepy": 0, "quantlib-mapped": 0}
    for row in rows:
        if row[10] in counts:
            row[9] = format(float(evaluate(row)), ".17g")
            counts[row[10]] += 1
    print(f"Executed FinancePy for {counts['financepy']} Act/365L cases and mapped QuantLib for {counts['quantlib-mapped']} 1/1/signed Bus/252 cases")
    return metadata




BASIS_MODULES = {
    "Act360": "act_360", "Act365F": "act_365f", "ActActIsda": "act_act_isda",
    "ActActIcma": "act_act_icma", "Thirty360BondBasis": "thirty_360_bond_basis",
    "ThirtyE360": "thirty_e_360", "ThirtyE360Isda": "thirty_e_360_isda",
    "Act365L": "act_365l", "Nl365": "nl_365", "Bus252": "bus_252", "OneOne": "one_one",
}
ERROR_MESSAGES = {
    "MissingCouponPeriod": "day count requires a reference coupon period and frequency",
    "MissingMaturity": "day count requires an explicit end-is-maturity flag",
    "MissingBusinessDayPredicate": "Bus/252 requires a business-day predicate",
    "ZeroFrequency": "coupon frequency must be positive",
    "InvalidReferencePeriod": "reference coupon end must be after its start",
    "AccrualOutsideReferencePeriod": "accrual lies outside the single reference coupon period",
}
SOURCE = "Python Fraction/datetime/calendar; ISDA/ICMA definitions via https://strata.opengamma.io/day_counts/"
LEAPS = tuple(dt.date(year, 2, 29).toordinal() for year in range(1583, 10000) if calendar.isleap(year))


def date(value):
    return dt.date.fromisoformat(value)


def leap_days(lower, upper):
    """Number of leap dates in the convention's (lower, upper] interval."""
    return bisect.bisect_right(LEAPS, upper.toordinal()) - bisect.bisect_right(LEAPS, lower.toordinal())


def weekdays(lower, upper):
    days = (upper - lower).days
    weeks, extra = divmod(days, 7)
    return weeks * 5 + sum((lower.weekday() + offset) % 7 < 5 for offset in range(extra))


def business_days(lower, upper, args):
    predicate = args.get("predicate", "weekends")
    if predicate == "all":
        return (upper - lower).days
    if predicate == "none":
        return 0
    if predicate == "single":
        return int(lower <= date(args["target"]) < upper)
    if predicate == "counted_none":
        return 0
    count = weekdays(lower, upper)
    if predicate == "no_sep7":
        # September 7 is excluded in every year, independently of weekends.
        count -= sum(lower <= (holiday := dt.date(year, 9, 7)) < upper and holiday.weekday() < 5
                     for year in range(lower.year, upper.year + 1))
    else:
        assert predicate == "weekends", predicate
    return count


def fraction(args):
    """A financial value or the precise public validation error."""
    start, end = date(args["start"]), date(args["end"])
    lower, upper = min(start, end), max(start, end)
    signed_days = (end - start).days
    orientation = -1 if start > end else 1
    basis = args["basis"]
    if basis in ("ActActIcma", "Act365L"):
        if args.get("coupon", "none") == "none":
            return "MissingCouponPeriod"
        frequency = args["frequency"]
        ref_start, ref_end = date(args["ref_start"]), date(args["ref_end"])
        if frequency == 0:
            return "ZeroFrequency"
        if ref_start >= ref_end:
            return "InvalidReferencePeriod"
        if lower < ref_start or upper > ref_end:
            return "AccrualOutsideReferencePeriod"
        if basis == "ActActIcma":
            return float(Fraction(signed_days, (ref_end - ref_start).days * frequency))
        denominator = (366 if leap_days(lower, ref_end) else 365) if frequency == 1 else (366 if calendar.isleap(ref_end.year) else 365)
        return float(Fraction(signed_days, denominator))
    if basis == "ThirtyE360Isda" and args.get("maturity", "none") == "none":
        return "MissingMaturity"
    if basis == "Act360":
        return float(Fraction(signed_days, 360))
    if basis == "Act365F":
        return float(Fraction(signed_days, 365))
    if basis == "OneOne":
        return 1.0
    if basis == "Nl365":
        return float(Fraction(orientation * ((upper - lower).days - leap_days(lower, upper)), 365))
    if basis == "ActActIsda":
        # Intersect [lower,upper) with each Gregorian year. This partition and
        # fsum do not share the Rust closed-form implementation's arithmetic.
        pieces = []
        for year in range(lower.year, upper.year + 1):
            year_start = dt.date(year, 1, 1).toordinal()
            year_end = year_start + (366 if calendar.isleap(year) else 365)
            overlap = max(0, min(upper.toordinal(), year_end) - max(lower.toordinal(), year_start))
            pieces.append(float(Fraction(overlap, year_end - year_start)))
        return orientation * math.fsum(pieces)
    if basis == "Bus252":
        return float(Fraction(orientation * business_days(lower, upper, args), 252))
    # Assign 30-day month coordinates from the published adjustment rules.
    first, last = start.day, end.day
    if basis == "Thirty360BondBasis":
        last = 30 if last == 31 and first >= 30 else last
        first = min(first, 30)
    elif basis == "ThirtyE360":
        first, last = min(first, 30), min(last, 30)
    else:
        assert basis == "ThirtyE360Isda", basis
        first_feb_eom = start.month == 2 and start.day == calendar.monthrange(start.year, 2)[1]
        last_feb_eom = end.month == 2 and end.day == calendar.monthrange(end.year, 2)[1]
        first = 30 if first == 31 or first_feb_eom else first
        last = 30 if last == 31 or (last_feb_eom and args["maturity"] == "false") else last
    return float(Fraction(360 * (end.year - start.year) + 30 * (end.month - start.month) + last - first, 360))


def qdate(value):
    return ql.Date(value.day, value.month, value.year)


def crosscheck(args, expected):
    """Execute vendor Python SDKs wherever the supplied contract maps exactly."""
    if isinstance(expected, str):
        return "validation contract"
    start, end = date(args["start"]), date(args["end"])
    lower, upper = min(start, end), max(start, end)
    if not (1901 <= lower.year <= upper.year <= 2199):
        return "full Date domain; independent Python financial formula"
    orientation = -1 if start > end else 1
    basis = args["basis"]
    counters = {
        "Act360": ql.Actual360(), "Act365F": ql.Actual365Fixed(),
        "ActActIsda": ql.ActualActual(ql.ActualActual.ISDA),
        "Nl365": ql.Actual365Fixed(ql.Actual365Fixed.NoLeap),
        "Thirty360BondBasis": ql.Thirty360(ql.Thirty360.BondBasis),
        "ThirtyE360": ql.Thirty360(ql.Thirty360.European),
    }
    source = "QuantLib==1.44"
    if basis in counters:
        actual = counters[basis].yearFraction(qdate(start), qdate(end))
    elif basis == "ThirtyE360Isda":
        actual = ql.Thirty360(ql.Thirty360.ISDA, qdate(end) if args["maturity"] == "true" else ql.Date()).yearFraction(qdate(start), qdate(end))
    elif basis == "OneOne":
        actual = ql.OneDayCounter().yearFraction(qdate(lower), qdate(upper))
        source += " OneDayCounter mapped constant-one orientation"
    elif basis == "ActActIcma":
        if not args.get("regular", False):
            return "ICMA supplied-reference exact Fraction definition; SDK does not accept explicit arbitrary frequency"
        ref_start, ref_end = date(args["ref_start"]), date(args["ref_end"])
        actual = orientation * ql.ActualActual(ql.ActualActual.ISMA).yearFraction(qdate(lower), qdate(upper), qdate(ref_start), qdate(ref_end))
    elif basis == "Act365L":
        ref_end = date(args["ref_end"])
        frequency = FrequencyTypes.ANNUAL if args["frequency"] == 1 else FrequencyTypes.MONTHLY
        actual = orientation * ACT_365L.year_frac(finance_date(lower), finance_date(upper),
                                                  dt3=finance_date(ref_end), freq_type=frequency)[0]
        source = "FinancePy==1.1.2 annual/nonannual Act365L classes; supplied coupon end"
    else:
        assert basis == "Bus252"
        predicate = args.get("predicate", "weekends")
        if predicate == "counted_none":
            return "equal Bus252: Python empty interval, predicate must not be called"
        cal = ql.BespokeCalendar("contract-" + predicate)
        if predicate in ("weekends", "no_sep7"):
            cal.addWeekend(ql.Saturday)
            cal.addWeekend(ql.Sunday)
        elif predicate == "none":
            for wd in (ql.Monday, ql.Tuesday, ql.Wednesday, ql.Thursday, ql.Friday, ql.Saturday, ql.Sunday):
                cal.addWeekend(wd)
        elif predicate == "single":
            # QuantLib businessDaysBetween is used as a one-date set measure.
            target = date(args["target"])
            actual = orientation * int(lower <= target < upper) / 252
            assert math.isclose(actual, expected, rel_tol=2e-12, abs_tol=2e-12)
            return "Python exact singleton-set membership"
        else:
            assert predicate == "all"
        if predicate == "no_sep7":
            for year in range(lower.year, upper.year + 1):
                cal.addHoliday(ql.Date(7, 9, year))
        actual = orientation * ql.Business252(cal).yearFraction(qdate(lower), qdate(upper))
    assert math.isclose(actual, expected, rel_tol=2e-12, abs_tol=2e-12), (basis, args, actual, expected)
    return source


def outcome(value):
    return "Err:DayCountError:" + value if isinstance(value, str) else "Ok:f64:" + format(value, ".17g")


def debug_date(point):
    return f"Date {{ year: {point.year}, month: {point.month}, day: {point.day} }}"


def debug_coupon(args, prefix=""):
    return ("CouponPeriod { start: " + debug_date(date(args[prefix + "ref_start"])) + ", end: " +
            debug_date(date(args[prefix + "ref_end"])) + f", frequency: {args[prefix + 'frequency']}" + " }")


def debug_context(args, prefix=""):
    coupon = "None" if args[prefix + "coupon"] == "none" else "Some(" + debug_coupon(args, prefix) + ")"
    flag = args[prefix + "maturity"]
    maturity = "None" if flag == "none" else "Some(" + flag + ")"
    return f"DayCountContext {{ coupon: {coupon}, end_is_maturity: {maturity} }}"


def cases():
    rows = []

    def add(api, args, expected, source=SOURCE):
        if args.get("predicate") == "counted_none" and api in (
                "day_count::bus_252::fraction", "day_count::try_fraction_with_business_days"):
            expected = "predicate_calls:0|" + expected
        rows.append(dict(id=f"daycount-contract-{len(rows):06}", api=api,
                         args=args, expected=expected, source=source))

    def emit(args, direct=True):
        expected = fraction(args)
        source = SOURCE + "; checked against " + crosscheck(args, expected)
        if direct:
            module = BASIS_MODULES[args["basis"]]
            method = "try_fraction" if args["basis"] in ("ActActIcma", "Act365L") else "fraction"
            # Direct schedule functions take all fields; a missing dispatcher
            # context is exercised by dispatch rather than an impossible call.
            if expected != "MissingCouponPeriod" and expected != "MissingMaturity":
                add("day_count::" + module + "::" + method, args, outcome(expected), source)
        plain = "MissingBusinessDayPredicate" if args["basis"] == "Bus252" else expected
        add("day_count::try_fraction", args, outcome(plain), source)
        with_predicate = dict(args)
        if args["basis"] != "Bus252":
            with_predicate["predicate"] = "counted_none"
        add("day_count::try_fraction_with_business_days", with_predicate, outcome(expected), source)

    # Distinct arithmetic and month-end classes including exact full-domain
    # boundary values, rather than merely checking finiteness there.
    intervals = [("1583-01-01", "9999-12-31"), ("1583-01-01", "1583-01-02"),
                 ("9999-12-30", "9999-12-31"), ("1900-02-28", "1900-03-01"),
                 ("2000-02-28", "2000-02-29"), ("2000-02-29", "2000-03-01"),
                 ("1999-12-31", "2001-01-01"), ("2024-01-29", "2024-01-31"),
                 ("2024-01-30", "2024-01-31"), ("2024-01-31", "2024-02-29"),
                 ("2024-02-29", "2025-02-28"), ("2025-02-28", "2025-03-31"),
                 ("2024-04-30", "2024-05-31"), ("2024-01-15", "2024-07-31"),
                 ("2026-09-01", "2026-09-10"), ("2026-09-07", "2026-09-08"),
                 ("2100-02-28", "2100-03-01"), ("2400-02-28", "2400-03-01")]
    context_free = [basis for basis in BASIS_MODULES if basis not in ("Act365L", "ActActIcma", "Bus252")]
    for start, end in intervals:
        for first, last in ((start, end), (end, start), (start, start), (end, end)):
            for basis in context_free:
                for maturity in (("false", "true") if basis == "ThirtyE360Isda" else ("none",)):
                    emit(dict(start=first, end=last, basis=basis, coupon="none", maturity=maturity))

    # Coupon validation failure precedence is independent of endpoint equality
    # and orientation. Irrelevant context must be ignored by other bases.
    start, end = "2024-04-01", "2024-07-01"
    coupon_shapes = [dict(coupon="none"),
                     dict(coupon="some", ref_start=start, ref_end=end, frequency=0),
                     dict(coupon="some", ref_start=end, ref_end=start, frequency=0),
                     dict(coupon="some", ref_start=end, ref_end=start, frequency=2),
                     dict(coupon="some", ref_start=start, ref_end=start, frequency=2),
                     dict(coupon="some", ref_start="2024-04-02", ref_end=end, frequency=4),
                     dict(coupon="some", ref_start=start, ref_end="2024-06-30", frequency=4),
                     dict(coupon="some", ref_start=start, ref_end=end, frequency=4)]
    for basis, coupon, orientation, maturity in product(BASIS_MODULES, coupon_shapes,
                                                       ((start, end), (end, start), (start, start), (end, end)),
                                                       ("none", "false", "true")):
        emit(dict(start=orientation[0], end=orientation[1], basis=basis, maturity=maturity,
                  predicate="weekends", **coupon))

    # All u8 positive frequencies are legal in the supplied-reference contract.
    # Regular schedule examples also execute QuantLib ICMA, including six/year.
    for frequency in range(1, 256):
        ref_start = dt.date(2023, 12, 31)
        regular = frequency in (1, 2, 3, 4, 6, 12)
        ref_end = ref_start + relativedelta(months=12 // frequency) if regular else dt.date(2024, 12, 31)
        for basis in ("ActActIcma", "Act365L"):
            for lower, upper in ((ref_start, ref_end), (ref_start + dt.timedelta(days=1), ref_end),
                                 (ref_end, ref_start), (ref_end, ref_end)):
                emit(dict(start=lower.isoformat(), end=upper.isoformat(), basis=basis,
                          coupon="some", ref_start=ref_start.isoformat(), ref_end=ref_end.isoformat(),
                          frequency=frequency, maturity="none", regular=int(regular)))

    # Annual denominator endpoint inclusion, next coupon distinct from accrual
    # end, century leap rules and the complete supported date-domain endpoints.
    for ref_start, ref_end, start, end in (
        ("2023-02-28", "2024-02-28", "2023-12-28", "2024-02-28"),
        ("2023-02-28", "2024-02-29", "2023-12-28", "2024-02-28"),
        ("2023-03-01", "2024-03-01", "2024-02-29", "2024-03-01"),
        ("2099-03-01", "2100-03-01", "2100-02-28", "2100-03-01"),
        ("2399-03-01", "2400-03-01", "2400-02-28", "2400-03-01"),
        ("1583-01-01", "9999-12-31", "1583-01-01", "9999-12-31"),
        ("1583-01-01", "1584-01-01", "1583-01-01", "1583-02-01"),
        ("9998-12-31", "9999-12-31", "9999-01-01", "9999-12-31"),
    ):
        for basis, frequency, orientation in product(("Act365L", "ActActIcma"), (1, 2, 6, 255),
                                                     ((start, end), (end, start), (end, end))):
            emit(dict(start=orientation[0], end=orientation[1], basis=basis, coupon="some",
                      ref_start=ref_start, ref_end=ref_end, frequency=frequency, maturity="none"))

    # Bus252 custom predicates: independent O(1) weekday counts and set measures,
    # including a multi-century interval without a vendor date-range limit.
    for start, end in intervals + [("1800-01-01", "2100-01-01")]:
        for first, last in ((start, end), (end, start), (start, start), (end, end)):
            for predicate in ("weekends", "all", "none", "no_sep7", "single"):
                emit(dict(start=first, end=last, basis="Bus252", coupon="none", maturity="none",
                          predicate=predicate, target="2026-09-07"))
    for point in ("1583-01-01", "2024-02-29", "9999-12-31"):
        emit(dict(start=point, end=point, basis="Bus252", coupon="none", maturity="none", predicate="counted_none"))

    for basis in BASIS_MODULES:
        add("day_count::DayCount::Debug", dict(basis=basis), "text:" + basis,
            "Public enum Debug contract")
    for basis, other in product(BASIS_MODULES, repeat=2):
        add("day_count::DayCount::traits", dict(basis=basis, other=other),
            f"text:debug={basis},eq={str(basis == other).lower()},clone=true,copy=true,hash_equal=true",
            "Public enum names + Python tagged-value equality + Clone/Copy/Hash laws")
    for error, message in ERROR_MESSAGES.items():
        args = dict(error=error)
        add("day_count::error::DayCountError::Display", args, "text:" + message, "Public error message contract")
        add("day_count::error::DayCountError::Debug", args, "text:" + error, "Public enum Debug contract")
        add("day_count::error::DayCountError::source", args, "text:None", "core::error::Error leaf-error contract")
        add("day_count::error::DayCountError::traits", args, "text:clone=true,copy=true,eq=true,hash_equal=true", "Clone/Copy/Eq/Hash laws")
        for other in ERROR_MESSAGES:
            add("day_count::error::DayCountError::PartialEq", dict(error=error, other=other),
                "bool:" + str(error == other).lower(), "Python tagged-value equality")

    coupons = [dict(ref_start="2024-01-01", ref_end="2024-07-01", frequency=value) for value in (0, 1, 2, 255)]
    coupons += [dict(ref_start="1583-01-01", ref_end="9999-12-31", frequency=6),
                dict(ref_start="2024-07-01", ref_end="2024-01-01", frequency=2)]
    for coupon in coupons:
        add("day_count::CouponPeriod::Debug", coupon, "text:" + debug_coupon(coupon), "Public struct Debug contract")
        for field, expected in (("start", "Ok:date:" + coupon["ref_start"]),
                                ("end", "Ok:date:" + coupon["ref_end"]), ("frequency", f"int:{coupon['frequency']}")):
            add("day_count::CouponPeriod::" + field, coupon, expected, "Public field value contract")
        for other in coupons:
            args = dict(coupon, **{"other_" + key: value for key, value in other.items()})
            add("day_count::CouponPeriod::traits", args,
                f"text:eq={str(coupon == other).lower()},clone=true,copy=true,hash_equal=true",
                "Python struct equality + Clone/Copy/Hash laws")
    contexts = [dict(coupon="none", maturity=flag) for flag in ("none", "false", "true")]
    contexts += [dict(coupon="some", maturity=flag, **coupon) for coupon, flag in product(coupons, ("none", "false", "true"))]
    add("day_count::DayCountContext::default", {}, "text:DayCountContext { coupon: None, end_is_maturity: None }", "Default optional-context contract")
    for context in contexts:
        add("day_count::DayCountContext::Debug", context, "text:" + debug_context(context), "Public struct Debug contract")
        add("day_count::DayCountContext::coupon", context,
            "text:None" if context["coupon"] == "none" else "text:Some(" + debug_coupon(context) + ")", "Public optional field contract")
        add("day_count::DayCountContext::end_is_maturity", context,
            "text:None" if context["maturity"] == "none" else "text:Some(" + context["maturity"] + ")", "Public optional field contract")
        for other in contexts:
            args = dict(context, **{"other_" + key: value for key, value in other.items()})
            add("day_count::DayCountContext::traits", args,
                f"text:eq={str(context == other).lower()},clone=true,copy=true,hash_equal=true",
                "Python optional struct equality + Clone/Copy/Hash laws")
    # Preserve the old additive-accrual property's full-domain reach with
    # independently known amounts for all three legs of seeded date triples.
    from random import Random
    rng = Random(20261023)
    lo, hi = dt.date(1583, 1, 1).toordinal(), dt.date(9999, 12, 31).toordinal()
    for _ in range(16):
        a, b, c = sorted(dt.date.fromordinal(rng.randint(lo, hi)) for _ in range(3))
        for first, last in ((a, b), (b, c), (a, c), (c, a)):
            emit(dict(start=first.isoformat(), end=last.isoformat(), basis="Act360",
                      coupon="none", maturity="none"))
    return rows
