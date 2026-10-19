# Changelog

All notable changes to this project will be documented in this file.

The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- **Breaking:** Stop exporting Make's `RUST_FLAGS` input; callers that need
  compiler flags for child tools must set `RUSTFLAGS` or pass
  `RUST_FLAGS=value` to recursive Make. Add `NEXTEST_FLAGS` for direct Nextest
  runs in `test`, `test-ci` and `test-quick`; it does not affect the
  availability probe, Cargo fallback or doctests
  ([#245](https://github.com/leynos/gauss/pull/245)).

- **Breaking:** `UserError::InvalidOperation` now accepts `String` instead of
  `&'static str`. This enables dynamic error messages with contextual
  information such as indices and lengths for better debugging.

  **Migration example:**

  ```rust
  // Before (static string):
  Err(UserError::InvalidOperation("operation failed"))

  // After (convert to String):
  Err(UserError::InvalidOperation("operation failed".into()))
  // or
  Err(UserError::InvalidOperation("operation failed".to_string()))
  // or with dynamic content:
  Err(UserError::InvalidOperation(format!(
      "index {} out of range (len = {})",
      index,
      len
  )))
  ```

  **Internal changes:** All callsites within gauss have been updated to use
  `.into()` for string literals or `format!()` for dynamic messages.

  **Compatibility note:** Downstream consumers using
  `UserError::InvalidOperation` must update their code to pass a `String`
  instead of `&'static str`. The simplest migration is to append `.into()` or
  `.to_string()` to existing string literals.

- **Breaking:** The paint type and action variants now use British spelling.
  Existing source callers must update these names; no compatibility aliases are
  provided. See [Colour API migration][colour-migration] and [PR #240][pr-240].

| Previous Rust name       | Current Rust name         |
| ------------------------ | ------------------------- |
| `Color`                  | `Colour`                  |
| `Action::SetStrokeColor` | `Action::SetStrokeColour` |
| `Action::SetFillColor`   | `Action::SetFillColour`   |

`Action::identifier()` intentionally retains the recorded strings
`SetStrokeColor` and `SetFillColor`; persisted identifiers are unchanged.

[colour-migration]: docs/developers-guide.md#colour-api-migration
[pr-240]: https://github.com/leynos/gauss/pull/240
