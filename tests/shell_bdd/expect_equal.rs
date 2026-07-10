//! Fallible equality assertions for shell BDD scenarios.

use std::fmt::Debug;

use test_support::{TestSupportError, TestSupportResult};

/// Compare values while returning a step error rather than panicking.
pub fn expect_equal<T: Debug + PartialEq>(
    actual: &T,
    expected: &T,
    context: impl Into<String>,
) -> TestSupportResult<()> {
    if actual != expected {
        return Err(TestSupportError::expectation(format!(
            "{}: expected {expected:?}, found {actual:?}",
            context.into()
        )));
    }
    Ok(())
}
