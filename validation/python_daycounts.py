"""Execute Python reference libraries for schedule-aware and mapped day counts.

The adapter selects conventions, supplies contractual dates and maps interval
orientation. All day-count calculations execute FinancePy or QuantLib.
"""

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
