# /// script
# requires-python = ">=3.12,<3.14"
# dependencies = [
#   "QuantLib==1.44",
#   "financepy==1.1.2",
#   "python-dateutil==3.0.0",
#   "convertdate==2.5.1",
#   "holidays==0.107",
#   "PyMeeus==0.5.12",
#   "six==1.17.0",
#   "numpy==2.5.4",
#   "numba==0.68.0",
#   "llvmlite==0.50.0",
#   "scipy==1.18.1",
# ]
# [tool.uv]
# exclude-dependencies = ["matplotlib", "pandas"]
# override-dependencies = [
#   "python-dateutil==3.0.0",
#   "numpy==2.5.4",
#   "numba==0.68.0",
#   "llvmlite==0.50.0",
#   "scipy==1.18.1",
# ]
# ///
"""Launch independent Python/Rust processes and compare their observations."""

import argparse
import contextlib
import hashlib
import json
import math
import os
from pathlib import Path
import subprocess
import sys

if sys.flags.optimize:
    raise SystemExit("Run the test launcher without Python optimization; checks must stay enabled.")
sys.dont_write_bytecode = True

ROOT = Path(__file__).resolve().parent.parent
ARTIFACTS = ROOT / "target/test-results"


def reference_worker():
    # Save the original stream; reference-library startup messages go to stderr.
    output = sys.stdout
    from modules import cases
    with contextlib.redirect_stdout(sys.stderr):
        for case in cases():
            output.write(json.dumps(case, separators=(",", ":")) + "\n")


def inputs(cases):
    rows = []
    for case in cases:
        fields = [case["id"], case["api"], ";".join(f"{key}={value}" for key, value in sorted(case["args"].items()))]
        assert all("\t" not in field and "\n" not in field for field in fields)
        assert all(";" not in str(value) for value in case["args"].values())
        rows.append("\t".join(fields) + "\n")
    return "".join(rows)


def equal(actual, expected):
    if "f64:" not in actual or "f64:" not in expected:
        return actual == expected
    actual_tag, a = actual.rsplit("f64:", 1)
    expected_tag, b = expected.rsplit("f64:", 1)
    if actual_tag != expected_tag:
        return False
    a, b = float(a), float(b)
    return math.isfinite(a) and math.isfinite(b) and abs(a - b) <= max(1e-12, 2 * math.ulp(b))


def compare(path, cases):
    expected = {case["id"]: case for case in cases}
    assert len(expected) == len(cases), "duplicate reference case IDs"
    observed = set()
    with path.open() as stream:
        for line in stream:
            case_id, actual = line.rstrip("\n").split("\t", 1)
            assert case_id in expected, f"Rust emitted an unexpected case: {case_id}"
            assert case_id not in observed, f"Rust emitted a duplicate case: {case_id}"
            case = expected[case_id]
            assert equal(actual, case["expected"]), (
                f"Independent mismatch {case_id}: {case['api']}({case['args']})\n"
                f"Python: {case['expected']}\nRust: {actual}\nReference: {case['source']}")
            observed.add(case_id)
    assert observed == set(expected), f"Rust omitted cases: {sorted(set(expected) - observed)[:10]}"
    return observed


def build(calendars):
    command = ["cargo", "test", "--locked", "--test", "test", "--no-run", "--message-format=json"]
    command.append("--all-features" if calendars else "--no-default-features")
    completed = subprocess.run(command, cwd=ROOT, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    assert completed.returncode == 0, completed.stderr
    executables = [row["executable"] for line in completed.stdout.splitlines()
                   if (row := json.loads(line)).get("reason") == "compiler-artifact"
                   and row.get("executable") and row["target"]["name"] == "test"]
    assert len(executables) == 1, executables
    return executables[0]


def execute(cases, calendars):
    executable = build(calendars)
    path = ARTIFACTS / ("rust-default.tsv" if calendars else "rust-minimal.tsv")
    environment = dict(os.environ, REGIT_TEST_OUTPUT=str(path))
    completed = subprocess.run([executable, "--observe"], input=inputs(cases), text=True,
                               cwd=ROOT, env=environment, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    assert completed.returncode == 0, completed.stderr
    return compare(path, cases)


def measure(cases):
    """Measure the production code exercised by oracle inputs alone."""
    output = ARTIFACTS / "rust-instrumented.tsv"
    command = ["cargo", "llvm-cov", "--locked", "--test", "test", "--all-features", "--no-report", "--", "--observe"]
    environment = dict(os.environ, REGIT_TEST_OUTPUT=str(output))
    completed = subprocess.run(command, cwd=ROOT, input=inputs(cases), text=True,
                               env=environment, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    assert completed.returncode == 0, completed.stderr
    compare(output, cases)
    report = ARTIFACTS / "llvm.json"
    subprocess.run(["cargo", "llvm-cov", "report", "--json", "--output-path", str(report)], cwd=ROOT, check=True)
    data = json.loads(report.read_text())["data"]
    files = {row["filename"]: row["summary"] for report in data for row in report["files"]
             if str(ROOT / "src") + "/" in row["filename"]}
    assert files, "no production Rust coverage recorded"
    return files


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--coverage", action="store_true", help="Measure production coverage from oracle inputs only")
    parser.add_argument("--reference-worker", action="store_true", help=argparse.SUPPRESS)
    args = parser.parse_args()
    if args.reference_worker:
        reference_worker()
        return
    ARTIFACTS.mkdir(parents=True, exist_ok=True)
    (ARTIFACTS / "report.json").unlink(missing_ok=True)
    print("Running the independent Python reference process...", flush=True)
    environment = dict(os.environ, NUMBA_CACHE_DIR=str(ARTIFACTS / "numba"), PYTHONDONTWRITEBYTECODE="1")
    python_output = ARTIFACTS / "python.jsonl"
    with python_output.open("w") as output:
        subprocess.run([sys.executable, str(Path(__file__).resolve()), "--reference-worker"], cwd=ROOT,
                       env=environment, stdout=output, check=True)
    cases, metadata = [], None
    with python_output.open() as stream:
        for line in stream:
            row = json.loads(line)
            if "metadata" in row:
                assert metadata is None
                metadata = row["metadata"]
            else:
                cases.append(row)
    assert cases and metadata, "the Python reference process did not complete"
    from modules import api as checks
    evidence = {}
    for calendars in (True, False):
        print("Checking Rust " + ("with calendars..." if calendars else "without calendars..."), flush=True)
        selected = [case for case in cases if calendars or not case["calendars"]]
        observed = execute(selected, calendars)
        contracts = [case for case in selected if case["api"] != "@reference"]
        inventory = checks.inventory(ROOT, calendars=calendars)
        compiler = checks.run_compile_contracts(ROOT, api_inventory=inventory)
        report = checks.enforce(ROOT, contracts, executed_ids=observed & {case['id'] for case in contracts},
                                compile_results=compiler, api_inventory=inventory)
        checks.check_inventory_guard(ROOT, contracts, executed_ids=observed & {case['id'] for case in contracts},
                                     compile_results=compiler, api_inventory=inventory)
        evidence["default" if calendars else "minimal"] = report
        print(("Default" if calendars else "Minimal") + ": Python/Rust observations and public API contracts agree.")
    report = {"reference": metadata, "api": evidence,
              "python_output_sha256": hashlib.sha256(python_output.read_bytes()).hexdigest()}
    if args.coverage:
        report["production_coverage"] = measure(cases)
    (ARTIFACTS / "report.json").write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    print("Independent comparison passed. Evidence: target/test-results/report.json")


if __name__ == "__main__":
    main()
