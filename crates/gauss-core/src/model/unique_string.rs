//! Shared helper for generating unique string identifiers.
//!
//! This module centralizes the "base + numeric suffix" strategy used by model
//! stores so naming behaviour stays consistent across resource and style types.

use std::collections::HashMap;

/// Bound suffix probing so a corrupted or exhausted namespace cannot loop forever.
const MAX_UNIQUE_SUFFIX: u32 = 10_000;

/// Trim a requested name, choose a fallback for blank input, and avoid collisions.
///
/// If the base already exists, the first available `{base}{separator}{n}` name
/// is returned. The function panics only if all 10,000 suffixes are occupied.
/// For `requested = "Primary"` and `separator = "-"`, an occupied `Primary`
/// and free `Primary-1` produce `Primary-1`.
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
