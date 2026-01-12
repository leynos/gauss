# Migrate Phase0 edits to Commands (Issue 18)

This ExecPlan is a living document. The sections Progress, Surprises &
Discoveries, Decision Log, and Outcomes & Retrospective must be kept up to date
as work proceeds.

No PLANS.md file is present in the repository.

## Purpose / Big Picture

Phase 0 editing actions should flow through the Action to Command pipeline so
undo and redo are based on CommandInverse rather than DocChange. After this
work, inserting anchors, closing paths, moving shapes, moving anchors, moving
handles, setting styles, reordering, and toggling segment kinds will all emit
Commands. A user can verify this by performing those edits, then undoing and
redoing them, and by running the test suite with the new command coverage.

## Progress

- [x] (2026-01-02 23:48Z) Drafted ExecPlan with the issue requirements and
  Phase 0 migration scope.
- [x] (2026-01-03 01:19Z) Implemented command-based document history and
  undo/redo wiring.
- [x] (2026-01-03 01:19Z) Routed Phase 0 actions and keybindings through the
  Action to Command pipeline.
- [x] (2026-01-03 01:19Z) Migrated drag, style, and draw workflows to create
  and apply Commands.
- [x] (2026-01-03 01:19Z) Updated command behaviour for reorder and delete
  anchors parity.
- [x] (2026-01-03 01:19Z) Added unit and BDD tests for new Commands and action
  preparation.
- [x] (2026-01-03 02:38Z) Split the command module into feature-focused files
  and migrated draw-mode anchor insertion to `Command::InsertAnchor`.
- [x] (2026-01-03 03:42Z) Corrected secondary modifier GPUI encoding and
  updated keystroke/keybinding expectations.
- [x] (2026-01-03 03:44Z) Ran formatting, lint, and test gateways and recorded
  outcomes.
- [x] (2026-01-03 03:55Z) Ran Markdown formatting, markdownlint, and nixie
  validation for documentation updates.
- [x] (2026-01-04 18:12Z) Refactored movement command anchor handling with a
  shared helper and grouped anchor identity inputs.
- [x] (2026-01-04 18:12Z) Reworked command editing test helpers to propagate
  typed errors and avoid oversized error variants.
- [x] (2026-01-04 18:12Z) Ran check-fmt, typecheck, lint, and test gateways for
  the refactors and recorded logs.

## Surprises & Discoveries

- Observation: The command module exceeded the 400-line limit after the
  migration, so it was split into feature submodules. Evidence:
  `wc -l src/model/command/mod.rs` reported more than 400 lines before the
  split.
- Observation: GPUI treats `cmd-` as the platform modifier, so emitting `cmd-`
  for secondary shortcuts on non-macOS caused Ctrl-based bindings to miss.
  Evidence: `gpui_reorder_undo` passed after switching to `secondary-` and
  updating keystroke expectations.
- Observation: `make nixie` requires an escalated run because Mermaid rendering
  launches a browser process that the sandbox blocks. Evidence: Puppeteer
  failed with "Failed to launch the browser process" until rerun with elevated
  permissions.
- Observation: Clippy flagged `result_large_err` for test helper errors when
  they stored full shapes by value. Evidence: `make lint` failed until the
  errors stored shapes behind `Arc`.

## Decision Log

- Decision: Phase 0 tools will emit Command and CommandInverse instead of
  DocOp and DocChange. Rationale: Issue requirement and ensures all edits are
  undoable via Commands. Date/Author: 2026-01-02, Codex.
- Decision: Raise and lower must act on selections that include anchors or
  segments and preserve the current block move behaviour. Rationale: Maintain
  Phase 0 behaviour and avoid regressions in selection semantics. Date/Author:
  2026-01-02, Codex.
- Decision: DeleteAnchors must preserve existing handles and segment kinds.
  Rationale: Required by issue guidance; avoid geometry degradation.
  Date/Author: 2026-01-02, Codex.
- Decision: Mode-specific keybindings will be enabled by switching the GPUI
  key context based on the current tool mode. Rationale: Allows Manipulate-only
  shortcuts without global collisions. Date/Author: 2026-01-02, Codex.
- Decision: The DeleteSelection handler will fall back to DeleteSelectedAnchors
  when no shapes are selected, and DeleteSelectedAnchors will not have a
  default keybinding to avoid collisions with DeleteSelection. Rationale:
  Preserves existing delete-key behaviour without conflicting keybindings.
  Date/Author: 2026-01-03, Codex.
- Decision: ToggleEdgeMode is bound only in DrawMode, with Tab bound to
  ToggleSegmentKind in ManipulateMode via the action bridge. Rationale: Keeps
  Tab behaviour context-specific while routing segment toggles through
  Commands. Date/Author: 2026-01-03, Codex.
- Decision: Draw-mode anchor insertion uses `Command::InsertAnchor` rather
  than introducing a new command variant. Rationale: Shape replacement already
  captures the change, keeping the command surface minimal. Date/Author:
  2026-01-03, Codex.
- Decision: Segment-based anchor insertion uses a dedicated
  `Command::InsertAnchorOnSegment` variant while keeping draw-mode insertions
  on `Command::InsertAnchor`. Rationale: Preserve action intent in command
  history and future serialization, superseding the earlier decision for
  segment insertions. Date/Author: 2026-01-12, Codex.
- Decision: Split `src/model/command` into feature submodules. Rationale:
  Enforces the 400-line file limit and keeps command logic grouped by feature.
  Date/Author: 2026-01-03, Codex.
- Decision: Encode secondary keybindings as `secondary-` for GPUI so platform
  mapping resolves to Cmd on macOS and Ctrl elsewhere. Rationale: Ensures
  cross-platform action bindings such as raise/lower work with
  `Modifiers::secondary_key()` in tests and the UI. Date/Author: 2026-01-03,
  Codex.
- Decision: Group anchor identity inputs into a small struct for movement
  helpers. Rationale: Keeps helper signatures within the clippy argument limit
  while preserving debug diagnostics. Date/Author: 2026-01-04, Codex.
- Decision: Store shapes behind `Arc` in test helper error variants. Rationale:
  Avoids oversized error results while keeping detailed mismatch diagnostics.
  Date/Author: 2026-01-04, Codex.

## Outcomes & Retrospective

- Completed the Phase 0 command migration and aligned undo/redo. Fixed
  secondary keybinding encoding so cross-platform shortcuts fire reliably. All
  quality gates pass.

## Context and Orientation

The model layer defines Actions, Commands, and user-facing errors in
`src/model/action.rs` and `src/model/command/`. Commands are applied through
`Command::apply`, and undo uses `CommandInverse::apply`. The Phase 0 UI
currently mutates documents via DocOp and DocChange in `src/ui/phase0_shell/`
and stores undo history as `DocHistoryItem` in
`src/ui/phase0_shell/draw/mod.rs`. Keybindings are defined in
`src/model/keybinding/mod.rs` and bridged to GPUI actions in
`src/ui/action_bridge/mod.rs`, then wired in `src/ui/phase0_shell/view.rs`.

The operations in Issue 18 are implemented today by Phase 0 helpers such as
`insert_anchor_on_selected_segment` in `src/ui/phase0_shell/anchor_edit.rs`,
drag helpers in `src/ui/phase0_shell/manipulate/drag.rs` and
`src/ui/phase0_shell/manipulate/handle_drag.rs`, style changes in
`src/ui/phase0_shell/style_controls.rs`, and reorder or segment toggle helpers
in `src/ui/phase0_shell/reorder.rs` and
`src/ui/phase0_shell/segment_toggle.rs`. These must be migrated to Commands.

## Plan of Work

First, replace document history with Command-based history. Introduce a new
history item that stores the applied Command and its CommandInverse so undo and
redo do not depend on DocChange. Add a helper on Phase0Shell to apply a
Command, update the document, and push history. Update undo and redo to use the
command history.

Next, route Phase 0 action handlers through the Action to Command pipeline. Add
default keybindings for the new Actions, update the GPUI action handlers in
`src/ui/phase0_shell/view.rs` to call `prepare_command` for document Actions,
and remove the direct key handling in `src/ui/phase0_shell/input.rs` that
currently swallows these keys. Switch the root element key context to the
current tool mode using `action_bridge::context_for_tool_mode` so
Manipulate-only bindings work.

Then, migrate Phase 0 editing flows to Commands. For drag operations, keep the
preview mutations but return a Command from `finish_drag` and apply it via the
new command helper. For style changes, build a `Command::SetStyle` with a list
of `StyleChange` entries and apply it. For drawing, use `Command::InsertAnchor`
when appending anchors to an open path and `Command::ClosePath` when closing a
path, both using shape replacement data. For anchor insertion and deletion,
call the model Action for insert on segment and delete anchors, then apply the
resulting Commands.

Finally, align command behaviour with Phase 0 semantics. Update reorder
preparation in `src/model/command/reorder.rs` to compute operations using the
same block move algorithm as `src/ui/phase0_shell/reorder.rs` and to include
shapes selected via anchors, handles, or segments. Update anchor deletion logic
in `src/model/command/anchor.rs` to preserve segment kinds and handles while
maintaining path invariants.

## Concrete Steps

All commands should be run from the repository root.

1. Inspect the current Phase 0 history and action wiring so you can update
   them with minimal churn.

    rg "DocHistoryItem" src/ui/phase0_shell
    rg "bind_model_actions" -n src/ui/phase0_shell/view.rs

2. Implement command history, then adjust undo and redo to use it. Expect the
   compiler to point to the updated history types.

3. Add keybindings for new Actions and update view action handlers to apply
   Commands. Remove the direct key handling for these Actions in
   `src/ui/phase0_shell/input.rs` so the action bridge receives the keystrokes.

4. Migrate drag, style, draw, and anchor edit workflows to create and apply
   Commands. Use the existing preview mutations but ensure the final commit is
   a Command.

5. Update command preparation and apply logic for reorder and delete anchors to
   match the required behaviour.

6. Add unit and BDD tests for the new Commands and preparation paths.

7. Run the quality gates with 300 second timeouts and capture output logs.

    set -o pipefail
    timeout 300s make check-fmt |& tee /tmp/gauss-check-fmt.log
    timeout 300s make lint |& tee /tmp/gauss-lint.log
    timeout 300s make test |& tee /tmp/gauss-test.log

   Expected result: each command exits 0 and the logs contain no errors.

## Validation and Acceptance

The change is accepted when all of the following are true:

- Editing actions for insert anchor on segment, draw-mode anchor insertion,
  delete anchors, raise, lower, toggle segment kind, move shapes, move anchors,
  move handles, set style, and close path produce Commands and populate command
  history.
- Undo and redo apply CommandInverse and Command respectively, and each edit
  round-trips correctly.
- Raise and lower operate on selections that include anchors or segments and
  preserve the block move behaviour used in Phase 0.
- DeleteAnchors preserves existing handles and segment kinds on the remaining
  path.
- `make check-fmt`, `make lint`, and `make test` pass with clean logs.

## Idempotence and Recovery

All steps are safe to re-run. If a migration step fails, revert the partial
changes in that file and re-apply the step. The test and lint commands are
read-only and can be re-run until they pass. If the command history update
breaks undo or redo, revert to the last known-good state and re-apply the
history changes in smaller increments.

## Artifacts and Notes

Keep the test logs for review:

    /tmp/gauss-check-fmt.log
    /tmp/gauss-lint.log
    /tmp/gauss-test.log

## Interfaces and Dependencies

Update or add the following interfaces so the migration is explicit and
consistent:

- In `src/ui/phase0_shell/draw/mod.rs`, replace `DocHistoryItem` with a new
  `CommandHistoryItem` that stores `Command` and `CommandInverse` and
  implements `HistoryItem`.
- In `src/ui/phase0_shell/mod.rs`, change `document_history` to
  `History<CommandHistoryItem>`.
- In `src/ui/phase0_shell/draw/mod.rs` (or a new helper module), add
  `Phase0Shell::apply_command(command: Command) -> Result<(), UserError>` that
  applies the command, updates the document, and pushes history.
- In `src/ui/phase0_shell/view.rs`, update action handlers to call
  `prepare_command` for document Actions and then `apply_command`.
- In `src/model/keybinding/mod.rs`, add default bindings for
  `InsertAnchorOnSegment`, `RaiseSelection`, `LowerSelection`, and
  `ToggleSegmentKind` with ManipulateMode contexts, keeping DeleteSelection on
  Backspace/Delete and routing anchor deletion via the handler fallback.
- In `src/ui/phase0_shell/input.rs`, remove key handling for those Actions so
  the action bridge receives the keystrokes.
- In `src/ui/phase0_shell/manipulate/drag.rs` and
  `src/ui/phase0_shell/manipulate/handle_drag.rs`, return Commands from drag
  completion and apply them via `apply_command`.
- In `src/ui/phase0_shell/style_controls.rs`, replace `DocOp::SetStyle` usage
  with `Command::SetStyle`.
- In `src/ui/phase0_shell/draw/mod.rs`, use `Command::InsertAnchor` for
  appending anchors to an open path and `Command::ClosePath` for path closure,
  both with shape replacement data.
- In `src/model/command/reorder.rs`, update `prepare_raise_selection` and
  `prepare_lower_selection` to use selection items beyond whole shapes and to
  apply the block move reorder algorithm.
- In `src/model/command/anchor.rs`, update delete anchors logic to preserve
  handles and segment kinds and keep path invariants intact.

## Revision note (required when editing an ExecPlan)

Replaced the earlier draft with a self-contained ExecPlan that follows the
execplans skill format, includes the mandatory living sections, and reflects
user decisions to migrate Phase 0 to Commands, keep raise/lower behaviour, and
preserve handles during anchor deletion.

Updated progress and decisions to reflect completed command migration work, the
DeleteSelection fallback behaviour, and Tab binding changes for draw versus
manipulate contexts.

Added progress and decision entries for splitting the command module into
feature submodules and routing draw-mode anchor insertion through
`Command::InsertAnchor`. This keeps file sizes within limits and ensures draw
edits participate in command history; the remaining work is still to run and
record the quality gates.

Revised the plan after identifying GPUI's `secondary` modifier parsing: the
keystroke encoder now emits `secondary-` instead of `cmd-`, tests were updated,
and quality gates were re-run. This resolves Ctrl-based shortcuts on non-macOS
and completes the remaining work.

Updated the progress and discoveries after running Markdown formatting,
markdownlint, and nixie validation; added the sandbox requirement for nixie to
the discoveries log.

Updated the progress log to capture the movement helper refactor, the test
helper error propagation adjustments, and the latest quality gate runs. Added
decision and discovery entries for the clippy-driven adjustments to the helper
signatures and error storage.

Updated the decision log and plan notes to adopt a dedicated
`Command::InsertAnchorOnSegment` variant for segment insertions, addressing
issue 26's command specificity concerns.
