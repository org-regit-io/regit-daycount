"""Independent caller-calendar roll and roll-error oracle.

The model ranks permitted Gregorian ordinal candidates and boundary events
against the documented search contract. No Rust outputs enter this model.
"""

import calendar
import datetime as dt
from itertools import product

from .date import MIN, MAX, LO, HI, SOURCE, date_result, debug_date, representative_dates

ROLLS = ("Unadjusted", "Following", "ModifiedFollowing", "Preceding",
         "ModifiedPreceding", "Nearest", "EndOfMonth")


def business(ordinal, args):
    predicate = args["predicate"]
    if predicate == "all":
        return True
    if predicate == "none":
        return False
    if predicate == "weekends":
        return dt.date.fromordinal(ordinal).weekday() < 5
    target = dt.date.fromisoformat(args["target"]).toordinal()
    if predicate == "single":
        return ordinal == target
    if predicate == "except":
        return ordinal != target
    if predicate == "pair":
        return ordinal in (target, dt.date.fromisoformat(args["target2"]).toordinal())
    if predicate == "before":
        return ordinal < target
    if predicate == "after":
        return ordinal > target
    raise AssertionError(f"unexpected predicate evaluation: {predicate}")


def roll_result(args):
    point = dt.date.fromisoformat(args["date"])
    origin = point.toordinal()
    roll = args["roll"]
    if roll == "Unadjusted":
        return date_result(point)
    if business(origin, args):
        return date_result(point)

    def directed(direction):
        candidates = [origin + direction * distance for distance in range(1, 367)
                      if LO <= origin + direction * distance <= HI]
        matches = [candidate for candidate in candidates if business(candidate, args)]
        if matches:
            return date_result(dt.date.fromordinal(min(matches, key=lambda value: abs(value - origin))))
        if not LO <= origin + direction * 366 <= HI:
            return "Err:RollError:DateBoundary:" + (MAX if direction > 0 else MIN).isoformat()
        return "Err:RollError:WalkLimitExceeded:366"

    if roll in ("ModifiedFollowing", "ModifiedPreceding", "EndOfMonth"):
        direction = -1 if roll == "ModifiedPreceding" else 1
        month_first = point.replace(day=1).toordinal()
        month_last = point.replace(day=calendar.monthrange(point.year, point.month)[1]).toordinal()
        same_month = [ordinal for ordinal in range(month_first, month_last + 1)
                      if (ordinal - origin) * direction > 0 and business(ordinal, args)]
        if same_month:
            return date_result(dt.date.fromordinal(min(same_month, key=lambda value: abs(value - origin))))
        return directed(-direction)
    if roll in ("Following", "Preceding"):
        return directed(1 if roll == "Following" else -1)
    assert roll == "Nearest"
    # Rank business candidates and boundary events by distance, forward first.
    # A required boundary event is an error even if the other direction would
    # contain an answer at an equal or larger distance.
    events = []
    for direction, priority in ((1, 0), (-1, 1)):
        available = HI - origin if direction > 0 else origin - LO
        for distance in range(1, min(366, available) + 1):
            ordinal = origin + direction * distance
            if business(ordinal, args):
                events.append((distance, priority, date_result(dt.date.fromordinal(ordinal))))
        if available < 366:
            events.append((available + 1, priority,
                           "Err:RollError:DateBoundary:" + (MAX if direction > 0 else MIN).isoformat()))
    return min(events)[2] if events else "Err:RollError:WalkLimitExceeded:366"


ROLL_VARIANTS = (
    "Unadjusted", "Following", "ModifiedFollowing", "Preceding",
    "ModifiedPreceding", "Nearest", "EndOfMonth",
)


def cases():
    rows = []
    def add(api, args, expected, source=SOURCE):
        rows.append(dict(id=f"roll-contract-{len(rows):06}", api=api, args=args,
                         expected=expected, source=source))
    points = representative_dates()
    for name, other in product(ROLLS, repeat=2):
        add("roll::Roll::traits", dict(roll=name, other=other),
            f"text:debug={name},eq={str(name == other).lower()},clone=true,copy=true,hash_equal=true",
            "Public enum names + Clone/Copy/Hash laws")
    roll_points = points + [dt.date(2026, 1, 1), dt.date(2026, 2, 28)]
    for point in roll_points:
        for roll, predicate in product(ROLLS, ("weekends", "all", "none", "except", "before", "after")):
            args = dict(date=point.isoformat(), roll=roll, predicate=predicate, target=point.isoformat())
            add("roll::try_apply", args, roll_result(args), "Independent ranked ordinal candidate contract")
    for point in (dt.date(2026, 1, 1), MIN, MAX):
        for offset in (-367, -366, -1, 0, 1, 366, 367):
            ordinal = point.toordinal() + offset
            if not LO <= ordinal <= HI:
                continue
            target = dt.date.fromordinal(ordinal).isoformat()
            for roll in ROLLS:
                args = dict(date=point.isoformat(), roll=roll, predicate="single", target=target)
                add("roll::try_apply", args, roll_result(args), "Independent single-business-date ordinal model")
    for distance in (1, 2, 31, 366, 367):
        point = dt.date(2026, 6, 15)
        args = dict(date=point.isoformat(), roll="Nearest", predicate="pair",
                    target=(point - dt.timedelta(days=distance)).isoformat(),
                    target2=(point + dt.timedelta(days=distance)).isoformat())
        add("roll::try_apply", args, roll_result(args), "Independent equal-distance candidate ranking; forward tie")
    for point in (MIN, MAX, dt.date(2026, 5, 23)):
        add("roll::try_apply", dict(date=point.isoformat(), roll="Unadjusted", predicate="counted_none"),
            "predicate_calls:0|" + date_result(point), "Unadjusted contract: predicate must not be evaluated")
    for point in (MIN, dt.date(2026, 5, 23), MAX):
        args = dict(variant="DateBoundary", date=point.isoformat())
        add("roll::RollError::Display", args, "text:date roll reached date boundary at " + debug_date(point), "Public error message contract")
        add("roll::RollError::Debug", args, "text:DateBoundary { date: " + debug_date(point) + " }", "Public error Debug contract")
        add("roll::RollError::source", args, "text:None", "core::error::Error leaf-error contract")
        add("roll::RollError::traits", args, "text:clone=true,copy=true,eq=true,distinct=false", "Clone/Copy/PartialEq laws")
    for limit in (0, 1, 366, 4294967295):
        args = dict(variant="WalkLimitExceeded", limit=limit)
        add("roll::RollError::Display", args, f"text:date roll exceeded {limit} steps", "Public error message contract")
        add("roll::RollError::Debug", args, f"text:WalkLimitExceeded {{ limit: {limit} }}", "Public error Debug contract")
        add("roll::RollError::source", args, "text:None", "core::error::Error leaf-error contract")
        add("roll::RollError::traits", args, "text:clone=true,copy=true,eq=true,distinct=false", "Clone/Copy/PartialEq laws")
    roll_errors = (("DateBoundary", MIN.isoformat()), ("DateBoundary", MAX.isoformat()),
                   ("WalkLimitExceeded", 0), ("WalkLimitExceeded", 366))
    for (variant, payload), (other_variant, other_payload) in product(roll_errors, repeat=2):
        args = dict(variant=variant, other_variant=other_variant)
        args["date" if variant == "DateBoundary" else "limit"] = payload
        args["other_date" if other_variant == "DateBoundary" else "other_limit"] = other_payload
        add("roll::RollError::PartialEq", args,
            "bool:" + str((variant, payload) == (other_variant, other_payload)).lower(),
            "Python tagged-value equality")
    return rows
