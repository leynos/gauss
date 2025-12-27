# Command Dispatch: Testing

**Parent**: [0-1-2-command-dispatch.md](../0-1-2-command-dispatch.md)

## Step 7: Unit tests (rstest)

Add comprehensive unit tests. See
[snippets/step7-unit-tests.rs](snippets/step7-unit-tests.rs).

## Step 8: Behaviour-driven development (BDD) tests (rstest-bdd)

Create `tests/features/command.feature`. See
[snippets/step8-command.feature](snippets/step8-command.feature).

Create `tests/command_bdd.rs`. See
[snippets/step8-command-bdd.rs](snippets/step8-command-bdd.rs).

## Step 9: GPUI integration tests

Create `tests/gpui_command_integration.rs`. See
[snippets/step9-gpui-integration.rs](snippets/step9-gpui-integration.rs).

## Step 10: Quality gates

```bash
make check-fmt
make lint
make test
```

## Step 11: Documentation updates

**Architecture document**: Add Command design decision to
`docs/gauss-architecture-design.md` §7.1. See
[snippets/step11-architecture-update.md](snippets/step11-architecture-update.md).

**User's guide**: No user guide changes are needed for this foundational task,
as commands are not directly user-visible. Users interact with Actions via
keyboard shortcuts and UI; the Command layer is internal.

## Step 12: Mark roadmap complete

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
