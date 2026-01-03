//! Undoable Commands for the Gauss editor.
//!
//! Commands are concrete, undoable state changes. They sit between Actions
//! (user intent) and `DocOps` (atomic mutations). Commands capture pre-
//! conditions, required context, and sufficient data for undo.
//!
//! Commands are GPUI-independent for testability and scripting.
//!
//! # Error Handling
//!
//! Command errors are separated into two categories:
//!
//! - **[`UserError`]**: Semantic errors that should be presented to users
//!   (e.g., empty selection, shape not found). These are returned from
//!   [`prepare_command`] and command application methods.
//!
//! - **Internal errors**: Dispatcher bugs and invariant violations that
//!   indicate programming errors. These use `unreachable!()` or `debug_assert!()`
//!   to fail fast during development while maintaining safety in release builds.
//!
//! This separation clarifies error handling responsibilities: UI code handles
//! [`UserError`] gracefully, while internal errors are caught during testing.
//!
//! # Design
//!
//! Commands are implemented as an enum rather than a trait for several reasons:
//!
//! - **Exhaustive matching**: All command variants can be matched exhaustively,
//!   making dispatch tables complete and verifiable at compile time.
//! - **Serialization**: Enums are trivially serializable, enabling future macro
//!   recording and playback (see roadmap §0.1.2).
//! - **Simplicity**: No type erasure or dynamic dispatch complexity.
//! - **Consistency**: Matches the Action enum design from task 0.1.1.
//!
//! # Relationship to Actions and `DocOps`
//!
//! ```text
//! Action (user intent)       e.g., DeleteSelection
//!    │
//!    ▼  prepare_command()
//! Command (undoable mutation) e.g., DeleteShapes { targets: [...] }
//!    │
//!    ▼  apply()
//! Document mutation          Direct shape removal with inverse capture
//! ```
//!
//! # Examples
//!
//! ```rust
//! use gauss::model::{Action, EngineState, prepare_command};
//!
//! // Prepare a command from an action using unified engine state
//! let state = EngineState::new();
//! let result = prepare_command(Action::DeleteSelection, &state);
//!
//! // Empty selection produces an error
//! assert!(result.is_err());
//! ```
#![allow(
    clippy::float_arithmetic,
    reason = "geometry calculations require arithmetic"
)]

mod anchor;
mod command_def;
mod delete_shapes;
mod error;
mod inverse;
mod movement;
mod prepare;
mod reorder;
mod segment;
mod style;
mod types;

pub use command_def::Command;
pub use error::UserError;
pub use inverse::CommandInverse;
pub use prepare::prepare_command;
pub use types::{
    AnchorDeletion, AnchorDeletionResult, AnchorMovement, AnchorRestoration, AnchorRestorationKind,
    DeletedShape, HandleKind, HandleMovement, ReorderOp, SegmentChange, ShapeMovement,
    ShapeReplacement, StyleChange,
};
