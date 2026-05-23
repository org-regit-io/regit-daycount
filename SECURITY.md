<!-- Copyright 2026 Regit.io — Nicolas Koenig -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 1.x     | Yes       |

## Reporting a Vulnerability

If you discover a security vulnerability in `regit-daycount`, please report
it responsibly:

1. **Do not** open a public GitHub issue
2. Email **nicolas.koenig@regit.io** with a description of the vulnerability
3. Include steps to reproduce if possible
4. We will acknowledge receipt within 48 hours and provide a timeline for a fix

## Scope

This crate computes day-count fractions, applies date-roll conventions, and
classifies business days against published holiday calendars. It is `no_std`,
allocation-free, and performs no network I/O, no file I/O, no authentication,
and no external communication. It has zero runtime dependencies.

The primary security concern is **correctness of the numeric / business-day
verdict**. A holiday classified as a trading day, a trading day classified as
a holiday, or a day-count fraction computed from a misstated convention
silently produces a wrong cashflow — a coupon paid on the wrong date, an
accrued-interest amount understated or overstated, a settlement booked on a
closed market. The downstream system has no way to catch the error: it acted
on a verdict the crate certified.

If you find any input where a year-fraction value, a business-day verdict, or
a holiday classification is incorrect with respect to the governing standard,
please report it using the process above. Each rule is traced to its standard
in [SPEC.md](SPEC.md).

## Dependencies

The crate has no runtime dependencies. Licence and supply-chain concerns for
development dependencies are policed via `cargo-deny` (`deny.toml` in the
repository root), checked in CI on every push. Dependency changes that
introduce a non-allowed licence or an active advisory are rejected at the gate.
