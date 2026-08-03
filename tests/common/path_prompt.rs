//! Shared file-path-prompt assertions for the dialog scenarios.
//!
//! Only the scenario binaries that assert on prompts include this module, and
//! each uses both helpers, so nothing here needs a `dead_code` expectation.

use gpui::TestAppContext;
use test_support::{TestSupportError, TestSupportResult};

/// Assert that no file path prompt has been displayed.
///
/// `context` becomes the expectation failure message, so each scenario binary
/// keeps its own wording while sharing the check.
///
/// # Errors
///
/// Returns an expectation error when a prompt has in fact been displayed.
pub fn assert_no_path_prompt(cx: &mut TestAppContext, context: &str) -> TestSupportResult<()> {
    if cx.did_prompt_for_new_path() {
        return Err(TestSupportError::expectation(context.to_owned()));
    }
    Ok(())
}

/// Assert that a file path prompt has been displayed.
///
/// `context` becomes the expectation failure message, so each scenario binary
/// keeps its own wording while sharing the check.
///
/// # Errors
///
/// Returns an expectation error when no prompt has been displayed.
pub fn assert_path_prompt(cx: &mut TestAppContext, context: &str) -> TestSupportResult<()> {
    if !cx.did_prompt_for_new_path() {
        return Err(TestSupportError::expectation(context.to_owned()));
    }
    Ok(())
}
