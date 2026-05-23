// Copyright 2026 Regit.io — Nicolas Koenig
// SPDX-License-Identifier: Apache-2.0

//! Allocation-free test helpers, compiled only under `cfg(test)`.
//!
//! The crate is `#![no_std]` and never links `alloc`, so `format!` is not
//! available — not even to the test modules. To assert on `Display` and
//! `Debug` output, a value is rendered into a fixed-capacity stack buffer
//! through [`core::fmt::Write`] and the resulting `&str` is compared
//! directly. This keeps the whole crate, tests included, free of `alloc`.

use core::fmt::{self, Debug, Display, Write};

/// Capacity of a [`FmtBuf`]. Every error and identifier rendering produced
/// by this crate is far shorter than this.
const CAP: usize = 256;

/// A fixed-capacity [`core::fmt::Write`] sink — the no-alloc stand-in for the
/// `String` that `format!` would otherwise build.
pub(crate) struct FmtBuf {
    bytes: [u8; CAP],
    len: usize,
}

impl FmtBuf {
    /// Creates an empty buffer.
    pub(crate) const fn new() -> Self {
        Self {
            bytes: [0; CAP],
            len: 0,
        }
    }

    /// Returns the text written so far.
    pub(crate) fn as_str(&self) -> &str {
        core::str::from_utf8(&self.bytes[..self.len]).unwrap_or("")
    }
}

impl Write for FmtBuf {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let end = self.len + s.len();
        let slot = self.bytes.get_mut(self.len..end).ok_or(fmt::Error)?;
        slot.copy_from_slice(s.as_bytes());
        self.len = end;
        Ok(())
    }
}

/// Renders a value's `Display` form into a stack buffer.
///
/// Used as `assert_eq!(display(value).as_str(), "expected")`.
pub(crate) fn display(value: impl Display) -> FmtBuf {
    let mut buf = FmtBuf::new();
    let _ = write!(buf, "{value}");
    buf
}

/// Renders a value's `Debug` form into a stack buffer.
pub(crate) fn debug(value: impl Debug) -> FmtBuf {
    let mut buf = FmtBuf::new();
    let _ = write!(buf, "{value:?}");
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmt_buf_collects_written_text() {
        let mut buf = FmtBuf::new();
        write!(buf, "ab{}", 12).unwrap();
        assert_eq!(buf.as_str(), "ab12");
    }

    #[test]
    fn display_and_debug_render() {
        assert_eq!(display(42_u32).as_str(), "42");
        assert_eq!(debug("x").as_str(), "\"x\"");
    }

    #[test]
    fn fmt_buf_rejects_overflow() {
        let mut buf = FmtBuf::new();
        // A write that would exceed the capacity fails rather than panics.
        let oversized = [b'x'; CAP + 1];
        let oversized = core::str::from_utf8(&oversized).unwrap_or("");
        assert!(write!(buf, "{oversized}").is_err());
    }
}
