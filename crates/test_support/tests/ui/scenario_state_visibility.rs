//! Compile-pass fixture for `scenario_state!` with `pub(super)` visibility.

#[path = "../../../../tests/common/scenario_state.rs"]
mod scenario_state;

mod support {
    //! Fixture support that exercises the macro's parent-visible expansion.

    #[derive(Default)]
    pub(super) struct ScenarioState {
        pub(super) value: u8,
    }

    crate::scenario_state!(ScenarioState; pub(super));
}

fn main() {
    support::with_state(|state| state.value = 1);
    support::reset_state();
    let _cleanup = support::ScenarioStateCleanup;
    let _fixture = support::scenario_state_cleanup();
}
