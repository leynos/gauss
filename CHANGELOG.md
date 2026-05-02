# Changelog

All notable changes to this project will be documented in this file.

The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

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

### Internal

- Adopt `rstest-bdd` v0.6.0-beta1 with GPUI harness
  (`rstest-bdd-harness-gpui::GpuiHarness`) for BDD tests that need a
  `TestAppContext`. The Phase 0 shell mode indicator test is migrated as a
  pilot. See the [Developer's guide](docs/developers-guide.md) for the pattern.
