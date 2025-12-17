//! User interface views and wiring.
//!
//! Phase 0 intentionally keeps UI minimal. The intent is to create a place for
//! GPUI-specific code (views, actions, and platform interactions) while keeping
//! the data model in `crate::model` usable without a running window.

pub mod phase0_shell;

pub use phase0_shell::{OpenSvg, Phase0Shell, SaveSvg};
