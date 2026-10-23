// Copyright 2026 Regit.io — SPDX-License-Identifier: Apache-2.0

#[path = "support/oracle.rs"]
mod oracle_support;

#[test]
fn matches_committed_independent_oracle_fixtures() {
    let fixture = include_str!("../validation/oracle.tsv");
    let mut compared = 0;
    for line in fixture.lines().skip(1) {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 11, "Invalid fixture row: {line}");
        if !oracle_support::supported(&fields) {
            continue;
        }
        let expected: f64 = fields[9].parse().unwrap();
        let actual = oracle_support::evaluate(&fields)
            .unwrap_or_else(|error| panic!("Oracle row {line}: {error}"));
        assert!(
            (actual - expected).abs() <= 1e-12,
            "Oracle row {line}: Rust={actual:.17}, expected={expected:.17}"
        );
        compared += 1;
    }
    // Independent baseline sizes catch omitted fixture rows. Update these
    // deliberately when the reference corpus changes.
    let expected_count = if cfg!(feature = "calendars") {
        477_265
    } else {
        208_059
    };
    assert_eq!(
        compared, expected_count,
        "Fixture set unexpectedly truncated"
    );
}
