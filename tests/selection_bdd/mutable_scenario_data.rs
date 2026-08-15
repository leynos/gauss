//! Mutable scenario-payload access for selection BDD harnesses that record state.
//!
//! Only harnesses that capture press-time values include this module. Keeping
//! the accessor separate preserves dead-code reporting for read-only harnesses.

use test_support::TestSupportResult;

use crate::support::{ScenarioContext, with_typed_scenario_data};

/// Mutate the scenario-specific payload for a typed scenario context.
///
/// # Errors
///
/// Returns an error when the scenario payload is absent or has a different
/// concrete type.
pub(super) fn with_mut_scenario_data<T: 'static, R>(
    context: ScenarioContext,
    f: impl FnOnce(&mut T) -> R,
) -> TestSupportResult<R> {
    with_typed_scenario_data(context, |data| data.downcast_mut::<T>().map(f))
}
