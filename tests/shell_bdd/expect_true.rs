//! Fallible predicate assertions for shell BDD scenarios.

use test_support::{TestSupportError, TestSupportResult};

/// Require a boolean condition in a fallible BDD step.
pub fn expect_true(condition: bool, context: impl Into<String>) -> TestSupportResult<()> {
    if !condition {
        return Err(TestSupportError::expectation(context));
    }
    Ok(())
}
