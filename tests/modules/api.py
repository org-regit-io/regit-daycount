"""Compiler-derived, fail-closed inventory of the crate's public API.

Names come from rustdoc's expanded item graph, including methods emitted by
macros and exports from private modules. Coverage comes from concrete oracle
cases and their evaluated IDs; the inventory is never evidence
that an operation ran. A public alias identifies the same compiler item, but
similarly named methods and default/explicit-policy APIs remain separate.
"""

from __future__ import annotations

import collections
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile


TOOLCHAIN = "1.95.0"
AUTO_TRAITS = frozenset(("Send", "Sync", "Unpin", "UnwindSafe", "RefUnwindSafe"))
# Compiler implementation details, not stable traits downstream can name.
COMPILER_TRAITS = frozenset(("Freeze", "UnsafeUnpin", "StructuralPartialEq"))
TYPE_TRAITS = {
    "date::Date": {"Clone", "Copy", "Debug", "Eq", "Hash", "Ord", "PartialEq", "PartialOrd"},
    "date::Weekday": {"Clone", "Copy", "Debug", "Eq", "Hash", "PartialEq"},
    "errors::ValidationError": {"Clone", "Copy", "Debug", "Display", "Eq", "Error", "PartialEq"},
    "day_count::DayCount": {"Clone", "Copy", "Debug", "Eq", "Hash", "PartialEq"},
    "day_count::error::DayCountError": {"Clone", "Copy", "Debug", "Display", "Eq", "Error", "Hash", "PartialEq"},
    "day_count::CouponPeriod": {"Clone", "Copy", "Debug", "Eq", "Hash", "PartialEq"},
    "day_count::DayCountContext": {"Clone", "Copy", "Debug", "Default", "Eq", "Hash", "PartialEq"},
    "roll::Roll": {"Clone", "Copy", "Debug", "Eq", "Hash", "PartialEq"},
    "roll::RollError": {"Clone", "Copy", "Debug", "Display", "Eq", "Error", "PartialEq"},
    "calendar::Calendar": {"Clone", "Copy", "Debug", "Eq", "Hash", "PartialEq"},
    "calendar::CalendarError": {"Clone", "Copy", "Debug", "Display", "Eq", "Error", "PartialEq"},
    "calendar::CoveragePolicy": {"Clone", "Copy", "Debug", "Default", "Eq", "Hash", "PartialEq"},
    "calendar::CalendarCoverage": {"Clone", "Copy", "Debug", "Eq", "PartialEq"},
    "calendar::composite::Composite": {"Clone", "Copy", "Debug"},
    "calendar::composite::JointBusiness": {"Clone", "Copy", "Debug"},
}
# These exact traits are exercised together by the runtime ::traits probes.
# This deliberately does not infer a bundle from the current source: adding a
# trait requires deciding how its behavior will be checked independently.
RUNTIME_BUNDLES = {
    name: traits & {"Clone", "Eq", "Hash", "Ord", "PartialEq", "PartialOrd"}
    for name, traits in TYPE_TRAITS.items()
}
# These two adapters also format the enum value with {:?} in their observed
# output. Other ::traits bundles do not format Debug and receive no such credit.
RUNTIME_BUNDLES["date::Weekday"].add("Debug")
RUNTIME_BUNDLES["roll::Roll"].add("Debug")


def inventory(root: Path, *, calendars: bool = True) -> dict:
    """Read macro-expanded public items from a pinned Rust compiler."""
    root = Path(root)
    with tempfile.TemporaryDirectory(prefix="regit-oracle-rustdoc-") as scratch:
        command = ["cargo", f"+{TOOLCHAIN}", "rustdoc", "--lib", "--locked", "--offline",
                   "--target-dir", scratch,
                   "--all-features" if calendars else "--no-default-features", "--",
                   "-Z", "unstable-options", "--output-format", "json"]
        environment = dict(os.environ, RUSTC_BOOTSTRAP="1")
        completed = subprocess.run(command, cwd=root, env=environment, text=True,
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        assert completed.returncode == 0, f"Public API extraction failed:\n{completed.stderr}"
        graph = json.loads((Path(scratch) / "doc/regit_daycount.json").read_text())
    assert graph["format_version"] == 57, (
        "Pinned rustdoc format changed; review the API graph parser", graph["format_version"])
    index = graph["index"]
    paths = collections.defaultdict(set)

    def walk(item_id, parent, override=None, ancestors=frozenset()):
        item_id = str(item_id)
        if item_id in ancestors:
            raise AssertionError(f"Cycle in rustdoc public exports at {parent}")
        item = index[item_id]
        if item["crate_id"] != 0 or item["visibility"] != "public":
            return
        kind, body = next(iter(item["inner"].items()))
        if kind == "use":
            target = body["id"]
            assert target is not None and str(target) in index, ("Unresolved export", item)
            if body["is_glob"]:
                module = index[str(target)]["inner"]["module"]
                for child in module["items"]:
                    walk(child, parent, ancestors=ancestors | {item_id})
            else:
                walk(target, parent, body["name"], ancestors | {item_id})
            return
        name = override or item["name"]
        path = "::".join((*parent, name)) if name else "::".join(parent)
        if kind == "module":
            for child in body["items"]:
                walk(child, (*parent, name) if name else parent, ancestors=ancestors | {item_id})
        else:
            paths[item_id].add(path)

    root_item = index[str(graph["root"])]
    for child in root_item["inner"]["module"]["items"]:
        walk(child, ())
    items = {}
    types = {}
    aliases = {}

    def add(identity, kind, names, item, **extra):
        assert identity not in items, ("Duplicate public identity", identity)
        items[identity] = {"kind": kind, "aliases": sorted(names),
                           "source": item.get("span"), **extra}
        for name in names:
            assert name not in aliases or aliases[name] == identity, (name, identity)
            aliases[name] = identity

    for item_id, names in sorted(paths.items()):
        item = index[item_id]
        kind, body = next(iter(item["inner"].items()))
        definition = "::".join(graph["paths"].get(item_id, {}).get("path", [])[1:])
        canonical = definition if definition in names else min(names, key=lambda name: (name.count("::"), name))
        assert kind in ("struct", "enum", "function", "constant"), (
            "New public item kind needs an oracle policy", canonical, kind)
        add(canonical, kind, names, item)
        if kind not in ("struct", "enum", "union"):
            continue
        types[canonical] = {"kind": kind, "variants": [], "traits": [], "non_exhaustive": "non_exhaustive" in item["attrs"]}
        if kind == "enum":
            for variant_id in body["variants"]:
                variant = index[str(variant_id)]
                variant_name = variant["name"]
                variant_kind = variant["inner"]["variant"]["kind"]
                types[canonical]["variants"].append({"name": variant_name, "kind": variant_kind})
                add(f"{canonical}::{variant_name}", "variant",
                    {f"{name}::{variant_name}" for name in names}, variant, owner=canonical)
        else:
            struct_kind = body["kind"]
            fields = struct_kind.get("plain", {}).get("fields", []) if isinstance(struct_kind, dict) else []
            for field_id in fields:
                field = index[str(field_id)]
                if field["visibility"] == "public":
                    add(f"{canonical}::{field['name']}", "field",
                        {f"{name}::{field['name']}" for name in names}, field, owner=canonical)
        for impl_id in body.get("impls", []):
            impl_item = index[str(impl_id)]
            implementation = impl_item["inner"]["impl"]
            trait = implementation.get("trait")
            if trait is not None:
                if implementation["blanket_impl"] is not None:
                    continue  # std blanket implementations do not belong to this crate.
                trait_name = trait["path"]
                if trait_name in COMPILER_TRAITS:
                    continue
                assert not implementation["is_negative"], ("Negative trait requires explicit oracle", canonical, trait_name)
                assert not implementation["is_synthetic"] or trait_name in AUTO_TRAITS, (
                    "New auto trait requires an explicit oracle", canonical, trait_name)
                types[canonical]["traits"].append(trait_name)
                add(f"{canonical}::{trait_name}", "trait_impl",
                    {f"{name}::{trait_name}" for name in names}, impl_item,
                    owner=canonical, trait=trait_name)
                continue
            for method_id in implementation["items"]:
                method = index[str(method_id)]
                if method["visibility"] != "public":
                    continue
                method_kind = next(iter(method["inner"]))
                assert method_kind in ("function", "assoc_const"), (canonical, method_kind)
                name = method["name"]
                add(f"{canonical}::{name}", method_kind,
                    {f"{prefix}::{name}" for prefix in names}, method, owner=canonical)
    return {"toolchain": TOOLCHAIN, "format_version": graph["format_version"],
            "feature": "default" if calendars else "minimal", "items": items,
            "aliases": aliases, "types": types}


def _arguments(value):
    if isinstance(value, dict):
        return value
    assert isinstance(value, str), ("Unexpected oracle args", value)
    return dict(part.split("=", 1) for part in value.split(";") if part)


def enforce(root: Path, cases: list[dict], *, executed_ids=None, compile_results=None,
            api_inventory=None) -> dict:
    """Require concrete independent cases for every public item and variant.

    ``executed_ids`` must come from the Rust adapter's actual evaluated case
    stream, after comparing its observations with Python. Omitting it performs
    a fixture catalogue check only and is explicitly recorded as unexecuted.
    Compiler results are likewise accepted only after the Python runner has
    checked expected compiler success/failure, not from a static declaration.
    """
    api_inventory = api_inventory or inventory(root)
    items, aliases = api_inventory["items"], api_inventory["aliases"]
    evidence = collections.defaultdict(set)
    ids = set()
    observed = set(executed_ids) if executed_ids is not None else None
    for case in cases:
        case_id = case["id"]
        assert case_id not in ids, ("Duplicate oracle case ID", case_id)
        ids.add(case_id)
        assert case.get("source"), ("Missing independent source", case_id)
        assert "expected" in case, ("Missing Python outcome", case_id)
        api = case["api"]
        args = _arguments(case["args"])
        # Exact aggregate trait probes are a reviewed implementation of these
        # known contracts; they do not grant coverage to a newly derived trait.
        if api.endswith("::traits"):
            type_alias = api.removesuffix("::traits")
            assert type_alias in aliases, ("Unknown trait-probe type", api)
            owner = aliases[type_alias]
            assert owner in RUNTIME_BUNDLES, ("New type needs behavioral trait probes", api)
            targets = [f"{owner}::{trait}" for trait in RUNTIME_BUNDLES[owner]]
        elif api.endswith("::source"):
            owner = aliases.get(api.removesuffix("::source"))
            targets = [f"{owner}::Error"]
        elif api.endswith("::default"):
            owner = aliases.get(api.removesuffix("::default"))
            targets = [f"{owner}::Default"]
        else:
            assert api in aliases, ("Oracle API is not a public compiler item", api, case_id)
            targets = [aliases[api]]
        for target in targets:
            assert target in items, ("Probe targets a missing trait/item", api, target)
            evidence[target].add(case_id)
        # Variants require an observed case with that exact input/output token.
        # Restrict to known enum argument roles so a random diagnostic message
        # mentioning a variant cannot count as exercised calendar/day-count data.
        role_types = {"basis": "day_count::DayCount", "day_count": "day_count::DayCount",
                      "calendar": "calendar::Calendar", "roll": "roll::Roll",
                      "weekday": "date::Weekday", "policy": "calendar::CoveragePolicy"}
        for key, owner in role_types.items():
            if owner not in api_inventory["types"] or key not in args:
                continue
            candidate = str(args[key])
            if f"{owner}::{candidate}" in items:
                evidence[f"{owner}::{candidate}"].add(case_id)
        if "markets" in args:
            for candidate in str(args["markets"]).split(","):
                variant = f"calendar::Calendar::{candidate}"
                if variant in items:
                    evidence[variant].add(case_id)
        for variant, metadata in items.items():
            if metadata["kind"] != "variant":
                continue
            owner = metadata["owner"]
            name = variant.rsplit("::", 1)[1]
            if owner.endswith("Error"):
                # Error fixtures assert typed Rust outcomes, or explicitly
                # instantiate the variant for Display/source/trait probes.
                error_token = f"Err:{owner.rsplit('::', 1)[1]}:{name}"
                expected = str(case["expected"])
                if expected == error_token or expected.startswith(error_token + ":"):
                    evidence[variant].add(case_id)
                elif args.get("variant") == name and any(target.startswith(owner + "::") for target in targets):
                    evidence[variant].add(case_id)
            elif owner == "date::Weekday" and api in ("date::Date::day_of_week", "Date::day_of_week"):
                if str(case["expected"]) == name:
                    evidence[variant].add(case_id)
        if observed is not None:
            assert case_id in observed, ("Declared Python case was not evaluated by Rust", case_id)
    if observed is not None:
        assert observed == ids, ("Rust/Python case IDs differ", sorted(observed - ids), sorted(ids - observed))
    compile_results = compile_results or []
    for result in compile_results:
        assert result.get("executed") is True and result.get("passed") is True, (
            "Compiler coverage requires successful execution", result)
        for target in result.get("covers", []):
            assert target in items, ("Compile probe refers to an unknown public item", target)
            evidence[target].add("compile:" + result["id"])
    missing = []
    for identity, item in items.items():
        if not evidence[identity]:
            missing.append(identity)
    assert not missing, "Public API lacks independent oracle evidence:\n" + "\n".join(sorted(missing))
    return {"schema": 1, "extractor": f"rustdoc {api_inventory['toolchain']} JSON {api_inventory['format_version']}",
            "feature": api_inventory["feature"], "runtime_execution_verified": observed is not None,
            "public_items": len(items), "cases": len(cases),
            "items": {identity: {**item, "case_ids": sorted(evidence[identity])}
                      for identity, item in sorted(items.items())}}


def check_inventory_guard(root: Path, cases: list[dict], *, executed_ids=None,
                          compile_results=None, api_inventory=None) -> dict:
    """Prove a new real public Rust function fails without an oracle case.

    The source mutation happens in a temporary copy outside the checkout.
    This tests both expanded-source discovery and the missing-case gate; a
    synthetic inventory declaration alone would test only the latter.
    """
    root = Path(root)
    baseline = api_inventory or inventory(root)
    enforce(root, cases, executed_ids=executed_ids, compile_results=compile_results,
            api_inventory=baseline)
    marker = "oracle_gate_uncovered_public_function"
    with tempfile.TemporaryDirectory(prefix="regit-oracle-mutation-") as temporary:
        scratch = Path(temporary)
        shutil.copytree(root / "src", scratch / "src")
        for name in ("Cargo.toml", "Cargo.lock", "README.md"):
            shutil.copyfile(root / name, scratch / name)
        with (scratch / "src/lib.rs").open("a") as source:
            source.write(f"\n/// Deliberately uncovered oracle-gate mutation.\npub fn {marker}() -> u32 {{ 17 }}\n")
        mutated = inventory(scratch, calendars=baseline["feature"] == "default")
        assert marker in mutated["items"], "Expanded API extraction missed the injected public function"
        try:
            enforce(scratch, cases, executed_ids=executed_ids, compile_results=compile_results,
                    api_inventory=mutated)
        except AssertionError as error:
            assert marker in str(error), ("Mutation failed for an unrelated reason", str(error))
        else:
            raise AssertionError("Public API gate accepted a new function without an oracle case")
    return {"id": "new_public_function_requires_oracle", "executed": True, "passed": True,
            "source": "Temporary source mutation + independent rustdoc extraction"}


# Independent downstream compiler probes.

TRAIT_PATHS = {
    "Clone": "core::clone::Clone", "Copy": "core::marker::Copy",
    "Debug": "core::fmt::Debug", "Default": "core::default::Default",
    "Display": "core::fmt::Display", "Eq": "core::cmp::Eq",
    "Error": "core::error::Error", "Hash": "core::hash::Hash",
    "Ord": "core::cmp::Ord", "PartialEq": "core::cmp::PartialEq",
    "PartialOrd": "core::cmp::PartialOrd", "Send": "core::marker::Send",
    "Sync": "core::marker::Sync", "Unpin": "core::marker::Unpin",
    "UnwindSafe": "core::panic::UnwindSafe", "RefUnwindSafe": "core::panic::RefUnwindSafe",
}


def _probes(api_inventory):
    probes = []
    declarations, calls, covers = [], [], []
    for number, (name, metadata) in enumerate(sorted(api_inventory["types"].items())):
        assert name in TYPE_TRAITS, f"New public type needs explicit compile and behavior contracts: {name}"
        expected = TYPE_TRAITS[name] | AUTO_TRAITS
        assert set(metadata["traits"]) == expected, (
            "Public trait contract changed; add independent behavior or preserve compatibility",
            name, sorted(expected), sorted(metadata["traits"]))
        bounds = "+".join(TRAIT_PATHS[trait] for trait in sorted(expected))
        declarations.append(f"fn traits_{number}<T: {bounds}>() {{}}")
        calls.append(f"traits_{number}::<regit_daycount::{name}>();")
        covers.append(name)
        covers.extend(f"{name}::{trait}" for trait in sorted(AUTO_TRAITS | ({"Copy", "Eq"} & expected)))
    probes.append({"id": "public_type_and_trait_bounds", "body": "\n".join(calls),
                   "declarations": "\n".join(declarations), "success": True, "codes": [], "covers": covers})

    for name, metadata in sorted(api_inventory["types"].items()):
        if metadata["kind"] != "enum":
            continue
        patterns = []
        for variant in metadata["variants"]:
            kind = variant["kind"]
            suffix = "" if kind == "plain" else " { .. }" if "struct" in kind else "(..)"
            patterns.append(f"regit_daycount::{name}::{variant['name']}{suffix} => 0")
        # Weekday is deliberately closed; all extensible market/convention/
        # error enums require downstream wildcard matches in the 2.0 contract.
        success = name == "date::Weekday"
        probes.append({"id": "exhaustiveness_" + name.replace("::", "_"),
                       "declarations": f"fn classify(value: regit_daycount::{name}) -> u8 {{ match value {{ {', '.join(patterns)} }} }}",
                       "body": "", "success": success, "codes": [] if success else ["E0004"], "covers": []})

    date = "let d = regit_daycount::Date::ymd(2026, 1, 1).unwrap();"
    negative = [
        ("private_unchecked_constructor", "let _ = regit_daycount::Date::ymd_unchecked(2026,2,30);", ["E0624"]),
        ("private_date_fields", "let _ = regit_daycount::Date { year: 2026, month: 2, day: 30 };", ["E0451"]),
        ("private_raw_day_arithmetic", date + " let _ = d.add_days(1);", ["E0624"]),
        ("removed_raw_month_arithmetic", date + " let _ = d.add_months_eom_aware(1);", ["E0599", "E0624"]),
        ("no_date_add_operator", date + " let _ = d + 1;", ["E0369"]),
        ("removed_nan_fraction", date + " let _ = regit_daycount::day_count::fraction(d,d,regit_daycount::DayCount::Act365L);", ["E0425"]),
        ("removed_nan_frequency_wrapper", date + " let _ = regit_daycount::day_count::year_fraction_with_freq(d,d,regit_daycount::DayCount::Act365L,2);", ["E0425"]),
        ("date_arithmetic_returns_result", date + " let _: regit_daycount::Date = d.try_add_days(1);", ["E0308"]),
        ("day_count_returns_result", date + " let _: f64 = regit_daycount::day_count::try_fraction(d,d,regit_daycount::DayCount::Act360,Default::default());", ["E0308"]),
        ("roll_returns_result", date + " let _: regit_daycount::Date = regit_daycount::roll::try_apply(d,regit_daycount::Roll::Following, |_| true);", ["E0308"]),
        ("date_result_must_be_handled", date + " d.try_add_days(1);", ["unused_must_use"]),
        ("day_count_result_must_be_handled", date + " regit_daycount::day_count::try_fraction(d,d,regit_daycount::DayCount::Act360,Default::default());", ["unused_must_use"]),
        ("roll_result_must_be_handled", date + " regit_daycount::roll::try_apply(d,regit_daycount::Roll::Following, |_| true);", ["unused_must_use"]),
    ]
    if api_inventory["feature"] == "default":
        cal = "regit_daycount::Calendar::Target2"
        for method in ("is_holiday", "is_business_day", "adjust", "next_business_day",
                       "previous_business_day", "business_days_between", "advance_business_days"):
            negative.append(("removed_unchecked_calendar_" + method,
                             f"use regit_daycount::calendar::{method};", ["E0603", "E0432"]))
        negative.extend([
            ("calendar_returns_result", date + f" let _: bool = regit_daycount::calendar::try_is_business_day(d,{cal});", ["E0308"]),
            ("calendar_result_must_be_handled", date + f" regit_daycount::calendar::try_is_business_day(d,{cal});", ["unused_must_use"]),
            ("composite_constructor_returns_result", f"let _: regit_daycount::calendar::Composite = regit_daycount::calendar::Composite::new(&[{cal}]);", ["E0308"]),
        ])
    else:
        negative.extend([
            ("calendar_feature_is_absent", "use regit_daycount::calendar;", ["E0432"]),
            ("calendar_reexport_is_absent", "use regit_daycount::Calendar;", ["E0432"]),
        ])
    for name, body, codes in negative:
        probes.append({"id": name, "declarations": "", "body": body,
                       "success": False, "codes": codes, "covers": []})
    return probes


def run_compile_contracts(root: Path, *, api_inventory=None, calendars=True) -> list[dict]:
    """Execute compiler probes and return only verified observation records."""
    root = Path(root)
    api_inventory = api_inventory or inventory(root, calendars=calendars)
    results = []
    with tempfile.TemporaryDirectory(prefix="regit-oracle-consumer-") as temporary:
        scratch = Path(temporary)
        library = scratch / "libregit_daycount.rlib"
        build = ["rustc", f"+{TOOLCHAIN}", "--crate-name", "regit_daycount", "--edition=2024",
                 "--crate-type=rlib", str(root / "src/lib.rs"), "-o", str(library)]
        if api_inventory["feature"] == "default":
            build.extend(("--cfg", 'feature="calendars"'))
        built = subprocess.run(build, cwd=root, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        assert built.returncode == 0, f"Independent consumer library build failed:\n{built.stderr}"
        for probe in _probes(api_inventory):
            source = ("#![deny(unused_must_use)]\n#![allow(dead_code, unused_imports)]\n"
                      + probe["declarations"] + "\nfn main() {\n" + probe["body"] + "\n}\n")
            path = scratch / (probe["id"] + ".rs")
            path.write_text(source)
            command = ["rustc", f"+{TOOLCHAIN}", "--edition=2024", "--emit=metadata",
                       "--crate-name", "oracle_consumer", "--extern", f"regit_daycount={library}",
                       "--error-format=json", str(path), "-o", str(scratch / "probe.rmeta")]
            completed = subprocess.run(command, cwd=root, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            diagnostics = [json.loads(line) for line in completed.stderr.splitlines() if line.startswith("{")]
            codes = {diagnostic["code"]["code"] for diagnostic in diagnostics if diagnostic.get("code")}
            assert (completed.returncode == 0) == probe["success"], (
                f"Compile contract {probe['id']} expected {'success' if probe['success'] else 'failure'}:\n"
                + completed.stderr)
            if not probe["success"]:
                assert codes & set(probe["codes"]), (
                    "Compiler rejected a probe for the wrong reason", probe["id"], sorted(codes), probe["codes"], completed.stderr)
            results.append({"id": probe["id"], "executed": True, "passed": True,
                            "expected_success": probe["success"], "observed_success": completed.returncode == 0,
                            "diagnostic_codes": sorted(codes), "covers": probe["covers"],
                            "source": "Independent downstream rustc consumer"})
    return results


# Kept concise for controller imports.
run = run_compile_contracts
