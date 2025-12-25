# Execution Plan: 0.1.1 Define Typed Action Trait

**Status**: Ready for implementation
**Roadmap reference**: `docs/roadmap.md` §0.1.1

## Overview

Define a typed `Action` enum representing user intent. Actions are despatchable
from UI, scripts, and tests—the foundation for the "Everything is an Action"
guiding principle.

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Trait vs Enum | **Enum with methods** | Exhaustive matching, serialization-ready, simpler dispatch |
| Initial scope | **Minimal starter set** | 5–8 core actions; expand in later tasks |
| Module location | `src/model/action.rs` | GPUI-independent for testability |

## Action Set (Minimal Starter)

```rust
pub enum Action {
    // Document mutations
    DeleteSelection,

    // Selection changes
    SelectAll,
    DeselectAll,

    // Tool activation
    ActivatePenTool,
    ActivateSelectTool,

    // History
    Undo,
    Redo,
}
```

## File Layout

| File | Purpose |
|------|---------|
| `src/model/action.rs` | Action enum, ActionKind, methods |
| `src/model/mod.rs` | Add exports for Action, ActionKind |
| `tests/features/action.feature` | BDD scenarios |
| `tests/action_bdd.rs` | BDD step implementations |

## Implementation Steps

### Step 1: Create `src/model/action.rs`

Define the core structures:

```rust
//! User-intent Actions for the Gauss editor.
//!
//! Actions represent what the user wants to do (e.g., "delete selection")
//! without specifying how. Actions are despatchable from UI, scripts, and
//! tests. They are the public API surface for all editor behaviour.
//!
//! Actions are GPUI-independent for testability and scripting.

/// Categorization of actions for dispatch routing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionKind {
    /// Mutates document state; produces undoable Command.
    Document,
    /// Mutates editor state (selection, viewport, tool).
    Editor,
}

/// User intent representation.
///
/// Actions are the unit of user-visible behaviour. Every feature must be
/// expressible as an Action to satisfy the "Everything is an Action" principle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Action {
    /// Delete currently selected objects.
    DeleteSelection,
    /// Select all selectable objects in the document.
    SelectAll,
    /// Clear the current selection.
    DeselectAll,
    /// Activate the Pen (draw) tool.
    ActivatePenTool,
    /// Activate the Selection (manipulate) tool.
    ActivateSelectTool,
    /// Undo the last document change.
    Undo,
    /// Redo the last undone change.
    Redo,
}
```

Implement methods:

- `kind(&self) -> ActionKind` — categorize for dispatch routing
- `name(&self) -> &'static str` — human-readable name for undo/a11y
- `requires_selection(&self) -> bool` — validation helper

### Step 2: Update `src/model/mod.rs`

```rust
pub mod action;
pub use action::{Action, ActionKind};
```

### Step 3: Unit tests (rstest)

In `src/model/action.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(Action::DeleteSelection, ActionKind::Document)]
    #[case(Action::SelectAll, ActionKind::Editor)]
    #[case(Action::Undo, ActionKind::Editor)]
    fn action_kind_is_correct(#[case] action: Action, #[case] expected: ActionKind) {
        assert_eq!(action.kind(), expected);
    }

    #[rstest]
    #[case(Action::DeleteSelection)]
    #[case(Action::SelectAll)]
    #[case(Action::Undo)]
    fn actions_have_nonempty_names(#[case] action: Action) {
        assert!(!action.name().is_empty());
    }

    #[rstest]
    fn delete_selection_requires_selection() {
        assert!(Action::DeleteSelection.requires_selection());
    }

    #[rstest]
    fn select_all_does_not_require_selection() {
        assert!(!Action::SelectAll.requires_selection());
    }
}
```

### Step 4: BDD tests (rstest-bdd)

Create `tests/features/action.feature`:

```gherkin
Feature: Action categorization
  Actions are categorized for correct dispatch routing.

  Scenario: Document action kind
    Given the action DeleteSelection
    Then its kind should be Document

  Scenario: Editor action kind
    Given the action SelectAll
    Then its kind should be Editor

  Scenario: Action requires selection
    Given the action DeleteSelection
    Then it should require a selection

  Scenario: Action does not require selection
    Given the action SelectAll
    Then it should not require a selection
```

Create `tests/action_bdd.rs`:

```rust
//! BDD tests for Action categorization.

use gauss::model::{Action, ActionKind};
use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then};
use test_support::TestSupportResult;

struct ActionWorld {
    action: Option<Action>,
}

#[fixture]
fn world() -> ActionWorld {
    ActionWorld { action: None }
}

#[given(regex = r"^the action (\w+)$")]
fn given_action(world: &mut ActionWorld, name: String) {
    world.action = Some(match name.as_str() {
        "DeleteSelection" => Action::DeleteSelection,
        "SelectAll" => Action::SelectAll,
        "DeselectAll" => Action::DeselectAll,
        "Undo" => Action::Undo,
        "Redo" => Action::Redo,
        _ => panic!("Unknown action: {name}"),
    });
}

#[then(regex = r"^its kind should be (\w+)$")]
fn then_kind_is(world: &ActionWorld, kind_name: String) -> TestSupportResult<()> {
    let expected = match kind_name.as_str() {
        "Document" => ActionKind::Document,
        "Editor" => ActionKind::Editor,
        _ => return Err(test_support::TestSupportError::expectation(
            format!("Unknown kind: {kind_name}")
        )),
    };
    let actual = world.action.expect("action not set").kind();
    if actual != expected {
        return Err(test_support::TestSupportError::expectation(
            format!("Expected {expected:?}, got {actual:?}")
        ));
    }
    Ok(())
}

// ... additional step definitions

rstest_bdd_macros::scenarios!("tests/features/action.feature");
```

### Step 5: Quality gates

```bash
make check-fmt
make lint
make test
```

### Step 6: Documentation updates

Update `docs/users-guide.md` if there are user-facing changes. For this
foundational task, no user-guide changes are expected unless Actions become
visible through scripting or UI.

Update `docs/gauss-architecture-design.md` to record the design decision that
Actions are an enum rather than a trait.

### Step 7: Mark roadmap complete

Edit `docs/roadmap.md`:

```diff
-### 0.1. Action/Command registry

-- [ ] 0.1.1. Define typed Action trait.
-  - [ ] Actions represent user intent (e.g., "Delete Selection").
-  - [ ] Actions are dispatchable from UI, scripts, and tests.
+- [x] 0.1.1. Define typed Action trait.
+  - [x] Actions represent user intent (e.g., "Delete Selection").
+  - [x] Actions are dispatchable from UI, scripts, and tests.
```

## Integration Notes

### Relationship to existing code

- **DocOp** (`src/model/ops.rs`): Actions sit above DocOp. Actions represent
  user intent; DocOps are atomic invertible mutations. Task 0.1.2 (Command
  dispatch) bridges Actions to DocOps.

- **GPUI Actions** (`src/ui/phase0_shell/mod.rs`): Current GPUI actions
  (`OpenSvg`, `CloseWindow`, etc.) remain unchanged for now. They will become
  thin wrappers calling model-layer Actions in task 0.1.2.

### Future extensibility

- `#[non_exhaustive]` allows adding variants without breaking downstream.
- Serde derivation can be added later for macro recording.
- ActionKind can gain a `Window` variant when window actions are added.

## Critical Files

| File | Purpose |
|------|---------|
| `src/model/mod.rs` | Add exports |
| `src/model/ops.rs` | Pattern reference for invertible operations |
| `docs/roadmap.md` | Update status on completion |
| `docs/gauss-architecture-design.md` | Record design decision |

## Estimated Scope

~150 lines of new code, ~100 lines of tests.
