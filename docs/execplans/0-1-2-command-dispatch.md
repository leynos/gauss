# Execution Plan: 0.1.2 Implement Command Dispatch

**Status**: Complete
**Roadmap reference**: `docs/roadmap.md` §0.1.2
**Depends on**: 0.1.1 (Action enum — complete)

## Overview

Commands are concrete, undoable state changes that bridge user intent (Actions)
to atomic document mutations (DocOps). This task implements the command
dispatch layer described in the architecture document §7.

The relationship between Actions, Commands, and DocOps is:

```text
Action (user intent)       e.g., DeleteSelection
   │
   ▼  dispatch()
Command (undoable mutation) e.g., DeleteSelectionCommand { ids: [...] }
   │
   ▼  apply()
DocChange / DocOp          e.g., RemoveShape { index, shape }
```

## Design Decisions

| Decision               | Choice                                 | Rationale                                                                 |
| ---------------------- | -------------------------------------- | ------------------------------------------------------------------------- |
| Command representation | **Enum with data**                     | Exhaustive matching, serialization-ready, consistent with Action design   |
| Inverse storage        | **Pre-computed at apply time**         | Undo does not require re-deriving inverse from current state              |
| Serialization          | **`serde` derives (optional feature)** | Enables macro recording without blocking initial implementation           |
| Error handling         | **`thiserror` enum**                   | Semantic errors for caller inspection, consistent with project guidelines |
| Module location        | `src/model/command.rs`                 | GPUI (Zed's UI framework)-independent for testability                     |

## Design Rationale

### Why Commands, not just DocOps?

DocOps are low-level, atomic mutations (move anchor, insert shape). Commands
operate at the user-intent level (delete selection, move selected objects). A
single Command may produce multiple DocOps grouped into one undo entry.

Commands capture:

- **Pre-conditions**: Can the command execute? (e.g., is there a selection?)
- **Context**: What data is needed? (e.g., which shape IDs are selected?)
- **Inverse**: How to undo? (store sufficient data at apply time)
- **Name**: Human-readable description for undo/redo menu

### Serialization for Macro Recording

Commands are designed for future macro recording per the architecture document.
The `serde` derives are gated behind a `serde` feature flag to avoid mandatory
serialization overhead. Initial implementation does not require macros to work.

## Command Set (Initial Scope)

The initial command set mirrors the existing Document-kind Actions:

```rust
pub enum Command {
    /// Delete the specified shapes.
    DeleteShapes {
        /// Shape IDs to delete, with their indices and data for undo.
        targets: Vec<DeletedShape>,
    },
    // Future commands:
    // MoveShapes { shape_ids: Vec<ShapeId>, delta: Vec2 },
    // SetStyle { shape_ids: Vec<ShapeId>, from: PaintStyle, to: PaintStyle },
    // InsertShape { index: usize, shape: Shape },
    // ...
}
```

The `DeletedShape` captures everything needed to restore the shape on undo:

```rust
pub struct DeletedShape {
    /// Original index in the document's shape list.
    pub index: usize,
    /// The deleted shape data.
    pub shape: Shape,
}
```

## File Layout

| File                             | Purpose                                            |
| -------------------------------- | -------------------------------------------------- |
| `src/model/command.rs`           | Command enum, CommandInverse, dispatch logic       |
| `src/model/command/error.rs`     | CommandError enum (optional: inline in command.rs) |
| `src/model/mod.rs`               | Add exports for Command types                      |
| `tests/features/command.feature` | BDD scenarios for command dispatch                 |
| `tests/command_bdd.rs`           | BDD step implementations                           |

## Implementation Steps

### Step 1: Create `src/model/command.rs`

Define the core structures:

```rust
//! Undoable Commands for the Gauss editor.
//!
//! Commands are concrete, undoable state changes. They sit between Actions
//! (user intent) and DocOps (atomic mutations). Commands capture pre-
//! conditions, required context, and sufficient data for undo.
//!
//! Commands are GPUI-independent for testability and scripting.

use crate::model::{Document, DocChange, DocOp, Selection, Shape, ShapeId};

/// Errors that can occur during command execution.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum CommandError {
    /// The command requires a non-empty selection, but nothing is selected.
    #[error("command requires a selection, but nothing is selected")]
    EmptySelection,

    /// A referenced shape does not exist in the document.
    #[error("shape {0:?} not found in document")]
    ShapeNotFound(ShapeId),
}

/// A shape that was deleted, with data needed for restoration.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DeletedShape {
    /// Original index in the document's shape list.
    pub index: usize,
    /// The deleted shape data.
    pub shape: Shape,
}

/// Concrete, undoable state changes.
///
/// Commands are the unit of undo/redo. Each command captures sufficient
/// data to apply and reverse the operation.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Command {
    /// Delete the specified shapes from the document.
    DeleteShapes {
        /// Shapes to delete, with their indices and data for undo.
        targets: Vec<DeletedShape>,
    },
}
```

### Step 2: Implement Command methods

```rust
impl Command {
    /// Return a human-readable name for this command.
    ///
    /// This name is suitable for undo/redo menu entries and accessibility.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::DeleteShapes { .. } => "Delete",
        }
    }

    /// Apply the command to the document, returning the inverse for undo.
    ///
    /// # Errors
    ///
    /// Returns `CommandError` if the command cannot be executed (e.g.,
    /// referenced shapes do not exist).
    pub fn apply(&self, doc: &mut Document) -> Result<CommandInverse, CommandError> {
        match self {
            Self::DeleteShapes { targets } => apply_delete_shapes(doc, targets),
        }
    }
}
```

### Step 3: Implement CommandInverse

```rust
/// The inverse of an applied command, used for undo.
///
/// CommandInverse captures everything needed to reverse a command.
/// It is produced by `Command::apply` and stored in the undo stack.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum CommandInverse {
    /// Restore deleted shapes to their original positions.
    RestoreShapes {
        /// Shapes to restore, with their original indices.
        targets: Vec<DeletedShape>,
    },
}

impl CommandInverse {
    /// Return a human-readable name for this inverse command.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::RestoreShapes { .. } => "Delete",
        }
    }

    /// Apply the inverse command to restore previous state.
    ///
    /// # Errors
    ///
    /// Returns `CommandError` if the inverse cannot be applied.
    pub fn apply(&self, doc: &mut Document) -> Result<(), CommandError> {
        match self {
            Self::RestoreShapes { targets } => apply_restore_shapes(doc, targets),
        }
    }
}
```

### Step 4: Implement helper functions

```rust
fn apply_delete_shapes(
    doc: &mut Document,
    targets: &[DeletedShape],
) -> Result<CommandInverse, CommandError> {
    // Remove shapes in reverse index order to preserve indices
    let mut sorted_targets = targets.to_vec();
    sorted_targets.sort_by(|a, b| b.index.cmp(&a.index));

    for target in &sorted_targets {
        if target.index < doc.shapes.len() {
            doc.shapes.remove(target.index);
        }
    }

    Ok(CommandInverse::RestoreShapes {
        targets: targets.to_vec(),
    })
}

fn apply_restore_shapes(
    doc: &mut Document,
    targets: &[DeletedShape],
) -> Result<(), CommandError> {
    // Insert shapes in forward index order to preserve indices
    let mut sorted_targets = targets.to_vec();
    sorted_targets.sort_by(|a, b| a.index.cmp(&b.index));

    for target in sorted_targets {
        if target.index <= doc.shapes.len() {
            doc.shapes.insert(target.index, target.shape);
        }
    }

    Ok(())
}
```

### Step 5: Implement command preparation from Action

```rust
/// Prepare a command from an action and current editor state.
///
/// This function bridges user intent (Action) to concrete command (Command).
/// It captures required context (selection, document state) at the moment
/// the action is invoked.
///
/// # Errors
///
/// Returns `CommandError` if the action cannot produce a valid command
/// (e.g., DeleteSelection with empty selection).
pub fn prepare_command(
    action: Action,
    doc: &Document,
    selection: &Selection,
) -> Result<Command, CommandError> {
    match action {
        Action::DeleteSelection => prepare_delete_selection(doc, selection),
        // Other Document actions would be handled here
        _ => unreachable!("only Document actions should be dispatched to prepare_command"),
    }
}

fn prepare_delete_selection(
    doc: &Document,
    selection: &Selection,
) -> Result<Command, CommandError> {
    if selection.is_empty() {
        return Err(CommandError::EmptySelection);
    }

    // Collect selected shape IDs
    let shape_ids: Vec<ShapeId> = selection.selected_shapes().collect();

    if shape_ids.is_empty() {
        return Err(CommandError::EmptySelection);
    }

    // Build DeletedShape entries with indices and data
    let mut targets = Vec::with_capacity(shape_ids.len());
    for &id in &shape_ids {
        let Some(index) = doc.find_index(id) else {
            return Err(CommandError::ShapeNotFound(id));
        };
        let shape = doc.shapes[index].clone();
        targets.push(DeletedShape { index, shape });
    }

    Ok(Command::DeleteShapes { targets })
}
```

### Step 6: Update `src/model/mod.rs`

```rust
pub mod command;

pub use command::{Command, CommandError, CommandInverse, DeletedShape, prepare_command};
```

### Step 7: Unit tests (rstest)

Add comprehensive unit tests in `src/model/command.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Document, Selection, SelItem, Shape};
    use rstest::{fixture, rstest};

    #[fixture]
    fn empty_doc() -> Document {
        Document::default()
    }

    #[fixture]
    fn doc_with_two_shapes() -> Document {
        let mut doc = Document::default();
        doc.shapes.push(Shape::default());
        doc.shapes.push(Shape::default());
        doc
    }

    #[rstest]
    fn delete_shapes_removes_from_document(mut doc_with_two_shapes: Document) {
        let id = doc_with_two_shapes.shapes[0].id;
        let cmd = Command::DeleteShapes {
            targets: vec![DeletedShape {
                index: 0,
                shape: doc_with_two_shapes.shapes[0].clone(),
            }],
        };

        let result = cmd.apply(&mut doc_with_two_shapes);
        assert!(result.is_ok());
        assert_eq!(doc_with_two_shapes.shapes.len(), 1);
    }

    #[rstest]
    fn delete_shapes_inverse_restores(mut doc_with_two_shapes: Document) {
        let original_len = doc_with_two_shapes.shapes.len();
        let shape = doc_with_two_shapes.shapes[0].clone();

        let cmd = Command::DeleteShapes {
            targets: vec![DeletedShape { index: 0, shape }],
        };

        let inverse = cmd.apply(&mut doc_with_two_shapes).expect("apply succeeded");
        assert_eq!(doc_with_two_shapes.shapes.len(), original_len - 1);

        inverse.apply(&mut doc_with_two_shapes).expect("undo succeeded");
        assert_eq!(doc_with_two_shapes.shapes.len(), original_len);
    }

    #[rstest]
    fn prepare_delete_selection_fails_with_empty_selection(doc_with_two_shapes: Document) {
        let selection = Selection::default();

        let result = prepare_command(
            Action::DeleteSelection,
            &doc_with_two_shapes,
            &selection,
        );

        assert!(matches!(result, Err(CommandError::EmptySelection)));
    }

    #[rstest]
    fn command_name_is_nonempty() {
        let cmd = Command::DeleteShapes { targets: vec![] };
        assert!(!cmd.name().is_empty());
    }
}
```

### Step 8: Behaviour-driven development (BDD) tests (rstest-bdd)

Create `tests/features/command.feature`:

```gherkin
Feature: Command dispatch

  Commands are concrete, undoable state changes that bridge user intent
  (Actions) to atomic document mutations (DocOps).

  Background:
    Given a document with two shapes
    And the first shape is selected

  Scenario: Delete selection produces valid command
    When I prepare DeleteSelection action
    Then the command should be DeleteShapes
    And the command should target one shape

  Scenario: Delete selection command removes shape
    When I prepare DeleteSelection action
    And I apply the command
    Then the document should have one shape

  Scenario: Delete selection is undoable
    When I prepare DeleteSelection action
    And I apply the command
    And I apply the inverse
    Then the document should have two shapes

  Scenario: Delete selection requires selection
    Given nothing is selected
    When I prepare DeleteSelection action
    Then the command should fail with EmptySelection

  Scenario: Command has human-readable name
    When I prepare DeleteSelection action
    Then the command name should be "Delete"
```

Create `tests/command_bdd.rs`:

```rust
//! Behaviour tests for Command dispatch.

use gauss::model::{
    Action, Command, CommandError, CommandInverse, Document, Selection, SelItem,
    Shape, prepare_command,
};
use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use test_support::{TestSupportError, TestSupportResult};

/// World state for command BDD tests.
#[derive(Default)]
struct CommandWorld {
    doc: Document,
    selection: Selection,
    command: Option<Result<Command, CommandError>>,
    inverse: Option<CommandInverse>,
}

#[fixture]
fn world() -> CommandWorld {
    CommandWorld::default()
}

// === Given steps ===

#[given("a document with two shapes")]
fn given_doc_with_two_shapes(world: &mut CommandWorld) {
    world.doc = Document::default();
    world.doc.shapes.push(Shape::default());
    world.doc.shapes.push(Shape::default());
}

#[given("the first shape is selected")]
fn given_first_shape_selected(world: &mut CommandWorld) {
    if let Some(shape) = world.doc.shapes.first() {
        world.selection.toggle(SelItem::Shape(shape.id));
    }
}

#[given("nothing is selected")]
fn given_nothing_selected(world: &mut CommandWorld) {
    world.selection = Selection::default();
}

// === When steps ===

#[when("I prepare DeleteSelection action")]
fn when_prepare_delete_selection(world: &mut CommandWorld) {
    world.command = Some(prepare_command(
        Action::DeleteSelection,
        &world.doc,
        &world.selection,
    ));
}

#[when("I apply the command")]
fn when_apply_command(world: &mut CommandWorld) -> TestSupportResult<()> {
    let cmd = world
        .command
        .as_ref()
        .ok_or_else(|| TestSupportError::missing("command", "apply"))?
        .as_ref()
        .map_err(|e| TestSupportError::expectation(format!("command failed: {e}")))?;

    let inverse = cmd
        .apply(&mut world.doc)
        .map_err(|e| TestSupportError::expectation(format!("apply failed: {e}")))?;

    world.inverse = Some(inverse);
    Ok(())
}

#[when("I apply the inverse")]
fn when_apply_inverse(world: &mut CommandWorld) -> TestSupportResult<()> {
    let inverse = world
        .inverse
        .as_ref()
        .ok_or_else(|| TestSupportError::missing("inverse", "undo"))?;

    inverse
        .apply(&mut world.doc)
        .map_err(|e| TestSupportError::expectation(format!("undo failed: {e}")))?;

    Ok(())
}

// === Then steps ===

#[then("the command should be DeleteShapes")]
fn then_command_is_delete_shapes(world: &CommandWorld) -> TestSupportResult<()> {
    let cmd = world
        .command
        .as_ref()
        .ok_or_else(|| TestSupportError::missing("command", "check"))?
        .as_ref()
        .map_err(|e| TestSupportError::expectation(format!("command failed: {e}")))?;

    match cmd {
        Command::DeleteShapes { .. } => Ok(()),
        #[allow(unreachable_patterns)]
        _ => Err(TestSupportError::expectation("expected DeleteShapes")),
    }
}

#[then("the command should target one shape")]
fn then_command_targets_one_shape(world: &CommandWorld) -> TestSupportResult<()> {
    let cmd = world
        .command
        .as_ref()
        .ok_or_else(|| TestSupportError::missing("command", "check"))?
        .as_ref()
        .map_err(|e| TestSupportError::expectation(format!("command failed: {e}")))?;

    match cmd {
        Command::DeleteShapes { targets } if targets.len() == 1 => Ok(()),
        Command::DeleteShapes { targets } => Err(TestSupportError::expectation(format!(
            "expected 1 target, got {}",
            targets.len()
        ))),
        #[allow(unreachable_patterns)]
        _ => Err(TestSupportError::expectation("expected DeleteShapes")),
    }
}

#[then("the document should have one shape")]
fn then_doc_has_one_shape(world: &CommandWorld) -> TestSupportResult<()> {
    if world.doc.shapes.len() == 1 {
        Ok(())
    } else {
        Err(TestSupportError::expectation(format!(
            "expected 1 shape, got {}",
            world.doc.shapes.len()
        )))
    }
}

#[then("the document should have two shapes")]
fn then_doc_has_two_shapes(world: &CommandWorld) -> TestSupportResult<()> {
    if world.doc.shapes.len() == 2 {
        Ok(())
    } else {
        Err(TestSupportError::expectation(format!(
            "expected 2 shapes, got {}",
            world.doc.shapes.len()
        )))
    }
}

#[then("the command should fail with EmptySelection")]
fn then_command_fails_empty_selection(world: &CommandWorld) -> TestSupportResult<()> {
    match &world.command {
        Some(Err(CommandError::EmptySelection)) => Ok(()),
        Some(Err(e)) => Err(TestSupportError::expectation(format!(
            "expected EmptySelection, got {e}"
        ))),
        Some(Ok(_)) => Err(TestSupportError::expectation(
            "expected error, got success",
        )),
        None => Err(TestSupportError::missing("command", "check")),
    }
}

#[then(r#"the command name should be "Delete""#)]
fn then_command_name_is_delete(world: &CommandWorld) -> TestSupportResult<()> {
    let cmd = world
        .command
        .as_ref()
        .ok_or_else(|| TestSupportError::missing("command", "check"))?
        .as_ref()
        .map_err(|e| TestSupportError::expectation(format!("command failed: {e}")))?;

    if cmd.name() == "Delete" {
        Ok(())
    } else {
        Err(TestSupportError::expectation(format!(
            r#"expected "Delete", got "{}""#,
            cmd.name()
        )))
    }
}

// === Scenario bindings ===

#[scenario(
    path = "tests/features/command.feature",
    name = "Delete selection produces valid command"
)]
fn delete_selection_produces_valid_command(world: CommandWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/command.feature",
    name = "Delete selection command removes shape"
)]
fn delete_selection_command_removes_shape(world: CommandWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/command.feature",
    name = "Delete selection is undoable"
)]
fn delete_selection_is_undoable(world: CommandWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/command.feature",
    name = "Delete selection requires selection"
)]
fn delete_selection_requires_selection(world: CommandWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/command.feature",
    name = "Command has human-readable name"
)]
fn command_has_human_readable_name(world: CommandWorld) {
    let _ = world;
}
```

### Step 9: GPUI integration tests

Create `tests/gpui_command_integration.rs` to verify command dispatch
integrates correctly with the existing undo/redo system:

```rust
//! GPUI integration tests for command dispatch.
//!
//! These tests verify that commands integrate correctly with the
//! existing undo/redo system in the Phase0Shell.

use gpui::{TestAppContext, VisualTestContext};
use gauss::ui::{init, Phase0Shell};

#[gpui::test]
async fn delete_selection_adds_to_undo_stack(cx: &mut TestAppContext) {
    // Test that executing DeleteSelection via the command system
    // adds an entry to the undo stack
    // Implementation depends on Phase0Shell integration
}

#[gpui::test]
async fn undo_after_delete_restores_shape(cx: &mut TestAppContext) {
    // Test that Cmd+Z after delete restores the shape
}
```

### Step 10: Quality gates

```bash
make check-fmt
make lint
make test
```

### Step 11: Documentation updates

**Architecture document**: Add Command design decision to
`docs/gauss-architecture-design.md` §7.1:

```markdown
### 7.1 Command design (implemented 2025-12)

**Design decision:** Commands are implemented as an **enum with data** rather
than a trait, for the following reasons:

- **Exhaustive matching**: All command variants can be matched exhaustively.
- **Serialization**: Enums are serializable for macro recording.
- **Consistency**: Matches the Action enum design.

Commands capture:
- Pre-conditions (can the command execute?)
- Context (what data is needed?)
- Inverse (how to undo?)
- Name (human-readable description)

The relationship is:
- Actions represent user intent
- Commands capture concrete mutations with undo data
- DocOps are atomic invertible document mutations
```

**User's guide**: No user guide changes are needed for this foundational task, as
commands are not directly user-visible. Users interact with Actions via
keyboard shortcuts and UI; the Command layer is internal.

### Step 12: Mark roadmap complete

Edit `docs/roadmap.md`:

```diff
 ### 0.1. Action/Command registry

 - [x] 0.1.1. Define typed Action enum.
   - [x] Actions represent user intent (e.g., "Delete Selection").
   - [x] Actions are dispatchable from UI, scripts, and tests.
-- [ ] 0.1.2. Implement Command dispatch.
--   - [ ] Commands are concrete, undoable state changes.
--   - [ ] Commands are serialisable for macro recording (optional initially).
+- [x] 0.1.2. Implement Command dispatch.
+  - [x] Commands are concrete, undoable state changes.
+  - [x] Commands are serialisable for macro recording (optional initially).
```

## Integration Notes

### Relationship to Existing Code

- **Action** (`src/model/action.rs`): Actions represent user intent. The
  `prepare_command` function bridges Actions to Commands.

- **DocOp/DocChange** (`src/model/ops.rs`): Commands can internally use DocOps
  for complex mutations. For simple cases like DeleteShapes, direct document
  manipulation is sufficient.

- **Selection** (`src/model/selection.rs`): Commands require selection state
  to determine which objects to operate on.

- **GPUI undo/redo**: The existing Phase0Shell uses `gpui_component::History`.
  Commands and their inverses integrate with this system.

### Future Extensibility

- `#[non_exhaustive]` allows adding command variants without breaking
  downstream.
- `serde` feature flag enables macro recording when needed.
- Additional commands (MoveShapes, SetStyle, etc.) follow the same pattern.
- Command grouping (transactions) can be added in task 0.3.2.

## Testing Strategy

| Test Type  | Location                  | Coverage                           |
| ---------- | ------------------------- | ---------------------------------- |
| Unit tests | `src/model/command.rs`    | Command apply/inverse, error cases |
| BDD tests  | `tests/command_bdd.rs`    | Command preparation, dispatch flow |
| GPUI tests | `tests/gpui_command_*.rs` | Undo/redo integration              |

### Test Scenarios

1. **Happy path**: DeleteSelection with valid selection produces command,
   applies correctly, inverse restores state.

2. **Error cases**: DeleteSelection with empty selection returns
   `CommandError::EmptySelection`.

3. **Round-trip**: Apply + inverse returns document to original state.

4. **Multi-shape delete**: Deleting multiple shapes preserves index order
   for correct undo.

5. **Integration**: Commands integrate with existing undo/redo via
   Phase0Shell history.

## Critical Files

| File                                | Purpose                                 |
| ----------------------------------- | --------------------------------------- |
| `src/model/action.rs`               | Action enum (already complete)          |
| `src/model/ops.rs`                  | DocOp/DocChange for reference           |
| `src/model/selection.rs`            | Selection state for command preparation |
| `src/model/document.rs`             | Document structure                      |
| `docs/roadmap.md`                   | Update status on completion             |
| `docs/gauss-architecture-design.md` | Record design decision                  |

## Estimated Scope

~250 lines of new code in `src/model/command.rs`, ~150 lines of unit tests,
~200 lines of BDD tests.
