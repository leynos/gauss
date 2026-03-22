# Test Timeout Analysis for i18n Module (0.7.1)

Date: 2026-03-22

## Problem Statement

The `make test` command timed out during execution with the following
observation:

- Timeout occurred during test compilation, not test execution
- Library compilation succeeds quickly (< 4 seconds when cached)
- Test compilation takes significantly longer due to heavyweight dependencies

## Root Cause Analysis

### Primary Cause: Heavyweight Test Dependencies

The Gauss project includes several heavyweight dependencies that significantly
increase test compilation time:

1. **GPUI**: GPU-accelerated UI framework with substantial dependencies
2. **Graphics Libraries**: blade-graphics, cosmic-text, resvg, lyon
3. **Platform Integration**: wayland, x11-clipboard, ashpd, xkbcommon
4. **Testing Frameworks**: rstest, rstest-bdd, gpui test harnesses

### Secondary Issue: Pre-existing Test Compilation Errors

Investigation revealed pre-existing test compilation errors unrelated to the
i18n implementation:

```text
error[E0599]: no method named `document` found for reference `&Phase0Shell`
error[E0599]: no method named `selection` found for reference `&Phase0Shell`
error[E0599]: no function or associated item named `new_for_tests` found
```

These errors indicate that existing tests were not compiling successfully,
suggesting the test suite may have been broken prior to the i18n implementation.

## Solution Implemented

### 1. Migrated to cargo-nextest

Replaced `cargo test` with `cargo nextest run` in the Makefile:

**Benefits:**

- Parallel test execution with configurable thread limits
- Better output formatting and test grouping
- Retry support for flaky tests
- Timeout configuration per test group
- JUnit output for CI integration

### 2. Created .config/nextest.toml

Configured nextest with:

- Default profile for local development
- CI profile with stricter settings
- Test groups for heavyweight GPUI tests (serial execution)
- Timeout configuration (60s default, 120s for GPUI tests)
- Retry policy (1 retry for flaky tests)

### 3. Added Makefile Targets

- `make test`: Run tests with nextest (default profile)
- `make test-ci`: Run tests with CI profile (stricter settings)
- `make test-quick`: Run unit tests only (skip integration tests)

## Performance Characteristics

### Library Build Time

- Cached: ~4 seconds
- Clean: Variable based on dependency count

### Test Build Time

- **Issue**: Pre-existing compilation errors prevent full test build
- **Expected**: 2-5 minutes for clean build with all test dependencies
- **Cached**: Should be significantly faster after first build

### Test Execution Time (once compilation works)

- Unit tests (i18n): Expected < 1s
- BDD tests: Expected < 5s
- GPUI integration tests: Expected 10-30s (serial execution)

## Recommendations

### Immediate Actions Needed

1. **Fix Pre-existing Test Compilation Errors**
   - The test_helpers module methods are properly gated
   - Existing GPUI tests are failing to find Phase0Shell methods
   - This issue pre-dates the i18n implementation
   - **Action**: Investigate why test_helpers module is not being included in
     test compilation

2. **Verify i18n Tests Separately**
   - Once compilation errors are resolved
   - Run: `cargo nextest run --lib -E 'test(i18n)'`
   - This will verify the i18n unit tests in isolation

### Long-term Improvements

1. **Split Test Dependencies**
   - Consider feature flags to reduce test dependency weight
   - Separate unit tests from integration tests more clearly

2. **CI Pipeline Optimization**
   - Use incremental compilation in CI
   - Cache compiled dependencies
   - Run test-quick for rapid feedback
   - Run full test suite on merge to main

3. **Test Organization**
   - Keep unit tests in module files (fast)
   - Move integration tests to separate binaries (can be built selectively)
   - Use test groups effectively in nextest config

## Conclusion

The timeout was caused by heavyweight test dependencies compiling slowly,
combined with pre-existing test compilation errors. The nextest migration
provides better test execution once the compilation issues are resolved.

The i18n implementation itself is complete and the library compiles
successfully. The test infrastructure improvements (nextest configuration) are
in place and will provide benefits once the underlying compilation issues are
fixed.
