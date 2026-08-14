//! Thread-local scenario-state scaffolding for GPUI BDD test binaries.
//!
//! Only the scenario binaries that need it include this module, via
//! `#[path = "common/scenario_state.rs"] mod scenario_state;`, so the macro
//! stays out of the general `common` surface.

/// Declare the thread-local scenario state shared by a BDD test binary.
///
/// GPUI scenario bindings cannot thread state through step arguments, so each
/// scenario binary keeps its own thread-local state cell. The scaffolding
/// around that cell is identical everywhere, so this macro generates it for any
/// `Default` state type:
///
/// - `STATE`, the thread-local cell holding the state;
/// - `with_state`, which grants exclusive access to the state;
/// - `reset_state`, which restores the state to its default value;
/// - `ScenarioStateCleanup`, a guard that resets the state when dropped; and
/// - `scenario_state_cleanup`, an rstest fixture that resets the state before a
///   scenario runs and yields the guard so it is reset again afterwards.
///
/// Paths are fully qualified so the macro does not depend on what the calling
/// binary happens to have in scope. Invoke it as `crate::scenario_state!(..)`
/// from a scenario binary's crate root, or pass a visibility after a semicolon
/// when a private support module must expose the generated items to its parent.
#[macro_export]
macro_rules! scenario_state {
    ($state:ty) => {
        thread_local! {
            static STATE: ::std::cell::RefCell<$state> =
                ::std::cell::RefCell::new(<$state as ::core::default::Default>::default());
        }

        /// Run `f` with exclusive access to the scenario state.
        fn with_state<R>(f: impl ::core::ops::FnOnce(&mut $state) -> R) -> R {
            STATE.with(|cell| f(&mut cell.borrow_mut()))
        }

        /// Restore the scenario state to its default value.
        fn reset_state() {
            with_state(|state| *state = <$state as ::core::default::Default>::default());
        }

        /// Guard that clears the scenario state once a scenario finishes.
        struct ScenarioStateCleanup;

        impl ::core::ops::Drop for ScenarioStateCleanup {
            /// Reset the scenario state as the guard goes out of scope.
            fn drop(&mut self) {
                reset_state();
            }
        }

        /// Clear the scenario state before a scenario runs and again afterwards.
        #[::rstest::fixture]
        fn scenario_state_cleanup() -> ScenarioStateCleanup {
            reset_state();
            ScenarioStateCleanup
        }
    };
    ($state:ty; $visibility:vis) => {
        $crate::scenario_state!(@declare [$visibility] $state);
    };
    (@declare [$visibility:vis] $state:ty) => {
        thread_local! {
            static STATE: ::std::cell::RefCell<$state> =
                ::std::cell::RefCell::new(<$state as ::core::default::Default>::default());
        }

        /// Run `f` with exclusive access to the scenario state.
        $visibility fn with_state<R>(f: impl ::core::ops::FnOnce(&mut $state) -> R) -> R {
            STATE.with(|cell| f(&mut cell.borrow_mut()))
        }

        /// Restore the scenario state to its default value.
        $visibility fn reset_state() {
            with_state(|state| *state = <$state as ::core::default::Default>::default());
        }

        /// Guard that clears the scenario state once a scenario finishes.
        $visibility struct ScenarioStateCleanup;

        impl ::core::ops::Drop for ScenarioStateCleanup {
            /// Reset the scenario state as the guard goes out of scope.
            fn drop(&mut self) {
                reset_state();
            }
        }

        /// Clear the scenario state before a scenario runs and again afterwards.
        #[::rstest::fixture]
        $visibility fn scenario_state_cleanup() -> ScenarioStateCleanup {
            reset_state();
            ScenarioStateCleanup
        }
    };
}
