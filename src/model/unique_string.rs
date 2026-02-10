//! Shared helper for generating unique string identifiers.
//!
//! This module centralizes the "base + numeric suffix" strategy used by model
//! stores so naming behaviour stays consistent across resource and style types.

use std::collections::HashMap;

const MAX_UNIQUE_SUFFIX: u32 = 10_000;

pub(super) fn make_unique_string<T>(
    requested: &str,
    default_base: &str,
    separator: &str,
    existing: &HashMap<String, T>,
) -> String {
    let trimmed = requested.trim();
    let base = if trimmed.is_empty() {
        default_base.to_owned()
    } else {
        trimmed.to_owned()
    };

    if !existing.contains_key(base.as_str()) {
        return base;
    }

    for suffix in 1..=MAX_UNIQUE_SUFFIX {
        let candidate = format!("{base}{separator}{suffix}");
        if !existing.contains_key(candidate.as_str()) {
            return candidate;
        }
    }

    panic!(
        "failed to generate a unique identifier for base '{base}' within {MAX_UNIQUE_SUFFIX} attempts"
    );
}
