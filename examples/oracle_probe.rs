// Copyright 2026 Regit.io — SPDX-License-Identifier: Apache-2.0

//! Development oracle line protocol; used by `validation/oracle.py --rust`.

#[path = "../tests/support/oracle.rs"]
mod oracle_support;

use std::io::{self, BufRead, Write};

fn main() -> io::Result<()> {
    let mut output = io::BufWriter::new(io::stdout().lock());
    for line in io::stdin().lock().lines().skip(1) {
        let line = line?;
        let fields: Vec<_> = line.split('\t').collect();
        if !oracle_support::supported(&fields) {
            writeln!(output, "{}\tunsupported", fields[0])?;
            continue;
        }
        match oracle_support::evaluate(&fields) {
            Ok(value) => writeln!(output, "{}\t{value:.17}", fields[0])?,
            Err(error) => writeln!(output, "{}\terror:{error}", fields[0])?,
        }
    }
    output.flush()
}
