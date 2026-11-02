//! Named literal Rustup discovery-failure inputs for preflight contract cases.
//!
//! Each toolchain check issues its own list and component query, so the cases
//! stay here in call order and keep `preflight_cases` within its line budget.

use super::{
    super::install_oracles as oracle,
    CallOccurrence,
    CheckFailureCase,
    Failure,
    FailureTrace,
    InvocationSelection,
};

/// The declared Rustup toolchain-list query fails before its component query.
pub(crate) const DECLARED_TOOLCHAIN_LIST_FAILURE: CheckFailureCase = CheckFailureCase {
    failure: Failure::DeclaredToolchainList,
    selection: InvocationSelection {
        tool: "rustup",
        arguments: &["toolchain", "list"],
        occurrence: CallOccurrence::First,
    },
    trace: FailureTrace::MissingDeclaredComponentQuery,
    diagnostic: "could not list installed Rust toolchains",
};

/// The declared Rustup component query fails after listing toolchains.
pub(crate) const DECLARED_COMPONENT_LIST_FAILURE: CheckFailureCase = CheckFailureCase {
    failure: Failure::DeclaredComponentList,
    selection: InvocationSelection {
        tool: "rustup",
        arguments: &[
            "component",
            "list",
            "--toolchain",
            oracle::DECLARED,
            "--installed",
        ],
        occurrence: CallOccurrence::First,
    },
    trace: FailureTrace::Complete,
    diagnostic: "could not list components for toolchain",
};

/// The second Rustup toolchain-list query fails before its component query.
pub(crate) const STABLE_TOOLCHAIN_LIST_FAILURE: CheckFailureCase = CheckFailureCase {
    failure: Failure::StableToolchainList,
    selection: InvocationSelection {
        tool: "rustup",
        arguments: &["toolchain", "list"],
        occurrence: CallOccurrence::Second,
    },
    trace: FailureTrace::MissingStableComponentQuery,
    diagnostic: "could not list installed Rust toolchains",
};

/// The stable-channel Rustup component query fails after listing its toolchain.
///
/// The declared channel differs from the stable pin, so exactly one expected
/// component query names the stable toolchain even though it is the second one
/// issued in source order.
pub(crate) const STABLE_COMPONENT_LIST_FAILURE: CheckFailureCase = CheckFailureCase {
    failure: Failure::StableComponentList,
    selection: InvocationSelection {
        tool: "rustup",
        arguments: &[
            "component",
            "list",
            "--toolchain",
            oracle::STABLE,
            "--installed",
        ],
        occurrence: CallOccurrence::First,
    },
    trace: FailureTrace::Complete,
    diagnostic: "could not list components for toolchain",
};

/// The third Rustup toolchain-list query fails before its component query.
pub(crate) const NIGHTLY_TOOLCHAIN_LIST_FAILURE: CheckFailureCase = CheckFailureCase {
    failure: Failure::NightlyToolchainList,
    selection: InvocationSelection {
        tool: "rustup",
        arguments: &["toolchain", "list"],
        occurrence: CallOccurrence::Third,
    },
    trace: FailureTrace::MissingNightlyComponentQuery,
    diagnostic: "could not list installed Rust toolchains",
};

/// The nightly formatter component query fails after listing its toolchain.
pub(crate) const NIGHTLY_COMPONENT_LIST_FAILURE: CheckFailureCase = CheckFailureCase {
    failure: Failure::NightlyComponentList,
    selection: InvocationSelection {
        tool: "rustup",
        arguments: &[
            "component",
            "list",
            "--toolchain",
            oracle::NIGHTLY,
            "--installed",
        ],
        occurrence: CallOccurrence::First,
    },
    trace: FailureTrace::Complete,
    diagnostic: "could not list components for toolchain",
};
