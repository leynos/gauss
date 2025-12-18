# Phase 0: Build the “Gauss” GPUI PoC vector editor

This ExecPlan is a living document. The sections `Progress`,
`Surprises & discoveries`, `Decision log`, and `Outcomes & retrospective` must
be kept up to date as work proceeds.

There is no `PLANS.md` in this repository at the time of writing, so this plan
is the source of truth for the Phase 0 PoC.

## Purpose / big picture

Implement a proof-of-concept (PoC) vector editor (“Gauss”) with a single
window, a toolbar, and a canvas surface. The PoC is “realistic enough to turn
into code without becoming a full Inkscape clone before lunch”.

After this work, a developer can run the app and:

- Draw paths in “Draw” mode (line segments or auto-smoothed cubic Béziers),
  close or commit the path, and switch to “Manipulate” mode.
- In “Manipulate” mode, select and drag shapes/anchors/handles, toggle segment
  kind (line ↔ cubic), adjust stroke/fill, raise/lower, and do “PoC quality”
  structural edits (insert/delete anchor).
- Open and save SVG via a native file dialog, using GPUI’s platform path
  prompts (preferred over a bespoke dialog).
- Undo/redo edits on two independent stacks (document edits and selection
  changes). The same shortcut set is used; holding Shift selects the selection
  history stack.

Toolchain note:

- This plan initially targeted stable Rust to match GPUI’s guidance, but
  `rstest-bdd` currently requires nightly features.
- For now, we pin `nightly-2025-10-23` (a nightly where stable 1.92.0 was
  forked) so we can write behavioural tests.
- Once `rstest-bdd` works on stable, we should return the toolchain pin to
  stable and re-run the quality gates.

Success is observable by running the GUI and doing the actions above, and by
running `make all` and seeing it pass.

## Progress

    - [x] (2025-12-17) Switch toolchain pin from nightly to stable.
    - [x] (2025-12-17) Switch toolchain pin to nightly-2025-10-23 for
          `rstest-bdd`.
    - [x] (2025-12-17) Pin core dependencies (GPUI, gpui-component, uuid).
    - [x] (2025-12-17) Relax `clippy::float_arithmetic` to `allow`.
    - [x] (2025-12-17) Establish a buildable GPUI “hello window” binary.
    - [x] (2025-12-17) Add `rstest-bdd` dev-dependencies and a smoke BDD test.
    - [x] (2025-12-17) Enable GPUI test support and add a headless GPUI test
          for the “Save…” action.
    - [x] (2025-12-17 19:00Z) Add unit tests asserting `DocOp`/`DocChange`
          inversion restores the document.
    - [x] (2025-12-17) Implement a Phase 0 “Save…” workflow that writes an SVG
          file, with a `#[gpui::test]` verifying the saved contents.
    - [x] (2025-12-17) Implement a Phase 0 “Open…” workflow that loads an SVG
          document from disk, with a `#[gpui::test]` verifying the loaded
          document state.
    - [x] (2025-12-18) Implement model layer (document/path/selection/viewport/
          ops) with unit tests using `rstest` (core types, viewport test, and
          doc op inversion tests).
    - [x] (2025-12-17) Implement SVG export + minimal import with unit tests and
          round-trip tests.
    - [x] (2025-12-17) Paint the current document to a GPUI `Canvas` using
          `PathBuilder`, so Phase 0 visibly renders shapes.
    - [x] (2025-12-18) Implement Phase 0 UI rendering (header + canvas) with
          state stored on the root view entity (`Phase0Shell`) and Canvas-based
          painting.
    - [x] (2025-12-17) Implement viewport input mapping (scroll wheel pan +
          Ctrl/Cmd-wheel zoom) with a headless `#[gpui::test]` asserting pan
          and cursor-centred zoom behaviour.
    - [x] (2025-12-17) Implement Draw mode (click to place points, Tab toggle
          edge mode, close, Esc commit) with document history and a headless
          `#[gpui::test]` asserting point placement and undo behaviour.
    - [x] (2025-12-18) Add manipulate-mode segment hit-testing and selection,
          and allow Tab to toggle selected segment kind (line ↔ cubic) with a
          headless `#[gpui::test]` asserting undo restores the original
          geometry.
    - [x] (2025-12-18) Implement raise/lower reordering in manipulate mode
          (Ctrl/Cmd-`[` and Ctrl/Cmd-`]`) with a headless `#[gpui::test]`
          asserting reordering and undo behaviour.
    - [x] (2025-12-18) Implement manipulate-mode structural edits
          (insert/delete anchor), encoded as PoC-quality “replace shape” ops,
          with headless `#[gpui::test]` coverage and doc undo/redo.
    - [x] (2025-12-17) Implement selection history + Shift-modified undo/redo
          with a headless `#[gpui::test]`.
    - [x] (2025-12-18) Add stroke/fill colour controls via
          `gpui-component`’s colour picker, and a headless `#[gpui::test]`
          asserting style changes apply to selected shapes and are undoable.
    - [x] (2025-12-18) Add a “Quit” button and configure the window to use
          environment decorations (non-resizable), with a headless
          `#[gpui::test]` asserting the quit request is recorded.
    - [x] (2025-12-17) Add behavioural tests (BDD) with `rstest-bdd` exercising
          the controller boundary via observable SVG output.
    - [x] (2025-12-18) Run all gates (`make all`) and document how to run the
          PoC.

## Surprises & discoveries

    - Observation: `Cargo.toml` denies `clippy::indexing_slicing`, and initially
      also denied `clippy::float_arithmetic`.
      Evidence: In `Cargo.toml` under `[lints.clippy]`,
      `indexing_slicing = "deny"`. `float_arithmetic` is now set to `allow`.
      Impact: Geometry and viewport code will be float-heavy, but must avoid
      `vec[i]` indexing. Use checked access (`get`, `get_mut`) and treat “out of
      bounds” as a no-op in `PoC` code paths (or return a typed error if the
      caller can act on it).

    - Observation: `rstest-bdd` currently requires Rust nightly and fails to
      compile on stable.
      Evidence: `rstest-bdd` declares `#![feature(auto_traits, negative_impls)]`
      in its crate root.
      Impact: Behaviour tests cannot use upstream `rstest-bdd` on stable today.
      Phase 0 temporarily pins `nightly-2025-10-23` to enable `rstest-bdd`
      tests, and we plan to return to stable once `rstest-bdd` is fixed.

    - Observation: `clippy::self_named_module_files` prevents introducing
      submodules under `src/model/ops.rs`.
      Evidence: `make lint` fails if `src/model/ops.rs` declares `mod …;`,
      suggesting moving `src/model/ops.rs` to `src/model/ops/mod.rs`.
      Impact: Keep unit tests for `ops` as a sibling `#[cfg(test)]` module
      under `src/model/mod.rs`, unless we decide to convert `ops.rs` into an
      `ops/` directory module.

    - Observation: Rust raw strings of the form `r#"..."#` cannot contain the
      sequence `"#` (for example `stroke="#000000"` in SVG snippets).
      Evidence: `cargo fmt` fails to parse test fixtures until the delimiter is
      widened to `r##"..."##`.
      Impact: Use `r##"..."##` for embedded SVG snippets that include hex
      colour attributes.

    - Observation: GPUI 0.2.2's test platform does not implement
      `prompt_for_paths`.
      Evidence: `gpui::platform::test::platform::TestPlatform::prompt_for_paths`
      is `unimplemented!()`.
      Impact: Phase 0 “Open…” tests use a test-only prompt mode that routes open
      selection through `prompt_for_new_path`, so we can still cover action
      dispatch and async wiring.

    - Observation: `VisualTestContext::debug_bounds` only works for elements
      that set an explicit debug selector.
      Evidence: GPUI’s `div().id("...")` sets an element ID, but does not
      populate the debug bounds map; `div().debug_selector(|| "...".to_owned())`
      does.
      Impact: Headless integration tests that need precise hit-testing (for
      example, scroll wheel events on the canvas) should use `debug_selector`.

    - Observation: Having both `src/ui/phase0_shell.rs` and
      `src/ui/phase0_shell/*` triggers `clippy::self-named-module-files`.
      Evidence: `make lint` fails with a suggestion to move
      `src/ui/phase0_shell.rs` to `src/ui/phase0_shell/mod.rs`.
      Impact: When introducing submodules under `src/ui/phase0_shell`, convert
      `phase0_shell` into a directory module and keep `mod.rs` under 400 lines.

    - Observation: Ending a drag gesture can require handling both “mouse up
      while hovered” and “mouse up outside” events.
      Evidence: Phase 0 binds both `on_mouse_up` and `on_mouse_up_out` for the
      canvas container to ensure the drag state is cleared even if the pointer
      leaves the element between down and up.
      Impact: Manipulate-mode drag handling should not assume the pointer stays
      within the canvas bounds for the full gesture.

    - Observation: In GPUI’s headless test harness, `simulate_click` triggers
      `ClickEvent` handlers but does not exercise `MouseDownEvent` handlers.
      Evidence: Selection is driven by `on_mouse_down` and required
      `simulate_mouse_down` / `simulate_mouse_up` in the selection history
      integration test.
      Impact: Prefer explicit mouse down/up simulation in `#[gpui::test]` cases
      that validate hit-testing or drag behaviour.

## Decision log

    - Decision: Target stable Rust for Phase 0 (goal state).
      Rationale: GPUI expects latest stable on macOS/Linux, and stable toolchains
      generally reduce churn.
      Date/Author: 2025-12-17 / Codex

    - Decision: Temporarily pin `nightly-2025-10-23` for Phase 0.
      Rationale: The user explicitly requested this nightly so we can unblock
      `rstest-bdd` behavioural testing immediately. We accept that this may
      break and will revert to stable once `rstest-bdd` supports stable.
      Date/Author: 2025-12-17 / Codex

    - Decision: Use GPUI platform path prompts for file open/save dialogs.
      Rationale: GPUI already provides a ready-made native prompt API
      (`prompt_for_paths` / `prompt_for_new_path`) via the platform services.
      Date/Author: 2025-12-17 / Codex

    - Decision: Scroll wheel support is sufficient; pinch-to-zoom is deferred.
      Rationale: The user explicitly scoped pinch out of Phase 0.
      Date/Author: 2025-12-17 / Codex

    - Decision: Test strategy is layered:
      - Unit tests: `rstest`, focused on pure model/controller functions.
      - Behavioural tests: planned to use `rstest-bdd`, driving controller APIs
        and asserting observable outcomes such as exported SVG strings. This
        currently runs on `nightly-2025-10-23` and should return to stable once
        `rstest-bdd` supports stable.
      Rationale: GPUI UI testing support exists, but controller-driven tests
      avoid coupling behaviour tests to platform-specific event synthesis.
      Date/Author: 2025-12-17 / Codex

    - Decision: Keep doc op round-trip tests in `src/model/mod.rs` (test-only
      submodule) instead of nesting them under `src/model/ops.rs`.
      Rationale: `clippy::self_named_module_files` is denied in this repo and
      forbids `src/model/ops.rs` from having submodules without converting it
      into `src/model/ops/mod.rs`.
      Date/Author: 2025-12-17 / Codex

    - Decision: Bind both `on_mouse_up` and `on_mouse_up_out` on the canvas
      container for Phase 0 drag gestures.
      Rationale: Drag gestures should end cleanly even if the pointer leaves
      the canvas between down and up, and this also improves test robustness.
      Date/Author: 2025-12-17 / Codex

    - Decision: Segment hit-testing selects the segment but dragging still
      moves the shape.
      Rationale: Segment selection is primarily needed for Tab-based segment
      toggling, and preserving the existing “grab anywhere on shape” drag
      behaviour keeps Phase 0 gestures intuitive.
      Date/Author: 2025-12-18 / Codex

    - Decision: Encode anchor insertion/deletion as “replace shape” document
      edits in Phase 0, and bind them to `i` (insert on selected segment) and
      `Backspace/Delete` (delete selected anchors).
      Rationale: Structural edits are hard to encode as fine-grained ops
      without building a more general patch format. Replacing a shape keeps
      operations invertible and small-scope while still validating the user
      workflow. The chosen shortcuts are easy to exercise in headless tests
      and can be revisited once we have real menu/toolbar affordances.
      Date/Author: 2025-12-18 / Codex

## Outcomes & retrospective

At completion, Phase 0 delivers a working PoC with:

- A GPUI app that opens a window, renders a toolbar and canvas, and supports
  the draw/manipulate workflow.
- A model layer that can be used without GPUI (document/path/selection ops).
- SVG open/save sufficient for round-tripping Gauss-produced output.
- Undo/redo on two independent histories (document and selection).
- Tests that protect core behaviour and prevent regressions.

Phase 0 explicitly does not attempt:

- “True” least-squares curve fitting for freehand input (we use a smooth
  curve-through-points approach).
- Pixel-perfect cubic hit-testing (we use sampling for curve segments).
- A full multi-document workspace, layers, text, gradients, or boolean ops.

## Context and orientation

Work from the repository root: `/mnt/home/leynos/Projects/gauss`.

Repository-wide developer commands are in `Makefile`. The main quality gates
are:

    make check-fmt
    make lint
    make test
    make all

For long outputs, capture logs and preserve exit codes:

    set -o pipefail
    mkdir -p target/logs
    (make all) 2>&1 | tee target/logs/make-all.log
    echo "exit=$?"

Style guidance:

- Follow `AGENTS.md` (Rust style, testing rules, documentation rules).
- For documentation grammar and wrapping, follow
  `docs/documentation-style-guide.md` (en-GB-oxendict).
- Use `rstest` for unit tests. Behaviour tests are planned to use `rstest-bdd`
  following `docs/rstest-bdd-users-guide.md`. This repository is temporarily
  pinned to `nightly-2025-10-23` to enable `rstest-bdd` (see
  `Surprises & discoveries`).
- For GPUI-specific wiring tests, use `#[gpui::test]` and GPUI’s
  `TestAppContext` / `VisualTestContext`.

GPUI documentation is vendored locally under `docs/rustdoc-gpui-0.2.2/`. When
you need an API detail, prefer that local rustdoc over the internet.

## Architecture (mapped to GPUI’s “registers”)

GPUI has three “registers”, and this PoC maps them explicitly:

1. State management and cross-view communication uses GPUI `Entity` values.
   For Phase 0 we keep the editor state directly on the root view entity,
   `gauss::ui::Phase0Shell`. Because views are entities, this still gives us a
   stable `Entity<Phase0Shell>` for headless tests, without introducing a
   second “editor state” entity prematurely.

2. High-level declarative UI uses views. Phase 0 is deliberately a single view
   (`Phase0Shell`) that renders:

   - A header row (Open/Save buttons), and
   - A canvas container that hosts the `Canvas` element.

   Behaviour is implemented in `src/ui/phase0_shell/*` submodules (draw,
   manipulate, input mapping, file dialogs, etc.) so the root view stays small.

3. Low-level imperative UI uses elements where needed. The canvas itself uses
   GPUI’s `Canvas` element to paint via the low-level paint API (see
   `src/ui/canvas_paint.rs`), without building a bespoke element system.

This separation is important for testability: the “controller” logic lives in
pure Rust modules and is called from GPUI event handlers, but it can also be
called from unit tests and BDD steps without a running window.

## Plan of work (milestones)

The milestones are ordered to produce visible progress early, de-risk the
unstable parts, and keep each milestone buildable and testable.

### Milestone 0: toolchain + dependency pinning

Goal: build a window on the pinned Rust toolchain, with pinned dependencies.

Edits:

- Update `rust-toolchain.toml` to `channel = "nightly-2025-10-23"`.
- Update `Cargo.toml` to include pinned, caret-version dependencies:
  - `gpui = "0.2.2"`
  - `gpui-component = "0.5.0"`
  - `uuid = "1.19.0"` (with `v4` feature enabled)
  - `camino = "1.2.2"` (UTF-8 paths when possible)
  - `cap-std = "4.0.0"` (capability-oriented filesystem access, using ambient
    authority at the UI boundary for user-selected paths)
- Add dev-dependencies:
  - `rstest = "0.26.1"`
  - `rstest-bdd` + `rstest-bdd-macros` for behavioural tests (requires the
    pinned nightly toolchain for now; see `Surprises & discoveries`).

Lint policy adjustments (minimal and justified):

- Change `float_arithmetic` from `deny` to `allow` under `[lints.clippy]`.
  Geometry and viewport transforms are inherently float-heavy. We retain other
  numeric foot-gun lints (casts, truncation, etc.) as denied.

Add a minimal runnable binary:

- Add `src/main.rs` with crate-level `//!` docs.
- Keep `src/lib.rs` as the main implementation crate for now, but add the
  missing crate-level `//!` docs immediately so `missing_crate_level_docs` does
  not fail.

Acceptance:

- `make build` succeeds on Linux on the pinned toolchain.
- Running `cargo run` opens a window titled “Gauss” (even if the body is a
  placeholder).

### Milestone 1: model layer (pure data, unit tested)

Goal: a model that is editor-friendly and SVG-shaped.

Create modules (all with module-level `//!` docs) under `src/`:

- `src/model/mod.rs`
- `src/model/document.rs`
- `src/model/path.rs`
- `src/model/selection.rs`
- `src/model/viewport.rs`
- `src/model/ops.rs`

Key design constraints from repository lint policy:

- Avoid indexing (`vec[i]`) in favour of `get`/`get_mut` and error returns.
- Keep public surface minimal and documented. Prefer private fields with
  methods to avoid `missing_docs` on many public fields.

Core types (names are intended to be stable, to reduce churn in later steps):

- `ShapeId` as a newtype wrapper around `uuid::Uuid`.
- `Vec2` with `add`, `sub`, `mul` and `distance` helpers.
- `Rgba` and `PaintStyle`.
- `Anchor` (pos, handle_in, handle_out).
- `SegmentKind` (`Line`, `Cubic`).
- `PathGeom` (anchors, segments, closed).
- `Shape` (id, z, style, path).
- `Document` (shapes) with safe lookup helpers.
- `Selection` over `SelItem` values.
- `Viewport` (pan, zoom) with `world_to_screen`, `screen_to_world`,
  `zoom_around_screen_point`.
- `DocOp` and `DocChange` supporting invertible edits suitable for history.

Unit tests (`rstest`) to add early:

- `Viewport::zoom_around_screen_point` keeps the world point under the cursor
  stable across zoom.
- Basic `DocOp::invert` sanity: applying an op then its invert is a no-op for
  a small document.

Acceptance:

- `make test` passes.
- The above tests fail before implementation and pass after.

### Milestone 2: SVG export + minimal import (testable without GPUI)

Goal: round-trip Gauss output via SVG.

Create:

- `src/svg/mod.rs`
- `src/svg/export.rs`: export `Document` to SVG with `<path>` elements.
- `src/svg/import.rs`: parse a limited subset:
  - only `<path>` elements are imported,
  - `d` commands: `M`, `L`, `C`, `Z` (ignore the rest),
  - stroke/fill/stroke-width and opacity attributes.

Implementation notes:

- Prefer robust parsing over ad hoc string slicing. A small XML parser crate is
  acceptable if it keeps the code clear and bounded (and does not force large
  public APIs).
- Export should include `fill="none"` and `stroke="none"` when absent.
- Alpha is exported using `*-opacity` attributes.

Unit tests (`rstest`):

- Exporting a simple document emits expected SVG fragments.
- Importing an exported document and exporting again produces stable output
  (allowing for whitespace normalisation if needed).

Acceptance:

- A round-trip test exists and passes.
- The exported SVG opens in a browser / Inkscape as expected.

### Milestone 3: GPUI window skeleton + ready-made file dialogs

Goal: a functional shell with “Open…” and “Save…” actions.

UI structure (as implemented):

- `src/ui/mod.rs`
- `src/ui/phase0_shell/mod.rs`: the Phase 0 root view (`Phase0Shell`).
- `src/ui/phase0_shell/file_dialogs.rs`: Open/Save prompt wiring.
- `src/ui/canvas_paint.rs`: `Canvas` rendering for a `Document`.

State:

- Phase 0 state is stored directly on `Phase0Shell` (document, selection,
  viewport, tool mode, and `gpui_component::history::History` instances).

File dialogs:

- Use GPUI’s platform service APIs, as described in the vendored rustdoc.
  Specifically, use `prompt_for_paths(PathPromptOptions { … })` for Open and
  `prompt_for_new_path(directory, suggested_name)` for Save.

The implementation should treat open/save as asynchronous:

- Trigger open/save from an Action or toolbar button.
- Await the returned oneshot receiver task and then apply the result back into
  the editor entity on the UI thread.

Acceptance:

- Clicking “Open…” presents a native path prompt.
- Clicking “Save…” presents a native save prompt.
- Errors are surfaced to the user via an on-screen prompt (use GPUI prompt
  facilities, not `println!`, because stdout/stderr are denied by clippy).

### Milestone 4: rendering (Canvas + PathBuilder)

Goal: paint the document to the canvas, and draw selection overlays.

Rendering rules:

- Shapes are drawn in z-order.
- Fill is drawn first (if closed + fill), then stroke.
- Selection overlays draw last (anchors, handles, hover highlight).

Bridge code:

- Implement a function that converts a `Shape` into GPUI paths using
  `PathBuilder`:
  - Move to first anchor,
  - For each segment: `line_to` or `cubic_bezier_to`,
  - `close()` when closed.

Note on coordinate spaces:

- The model stores everything in world space.
- Rendering converts world to screen via the `Viewport` at paint time.

Acceptance:

- The canvas shows at least one deterministic shape (e.g. a starter demo shape)
  and overlays can be toggled for debugging.

### Milestone 5: viewport input mapping (scroll + zoom)

Goal: make the canvas navigable.

Implement:

- Scroll wheel pan:
  - vertical wheel pans vertically,
  - horizontal wheel pans horizontally,
  - Shift+wheel uses horizontal pan when only vertical wheel input exists.
- Ctrl/Cmd-wheel zoom around cursor position, using
  `Viewport::zoom_around_screen_point`.
- Middle mouse drag pan is deferred (the user scoped Phase 0 to scroll wheel).

Acceptance:

- Panning and zooming behave “normally”: content under cursor stays stable when
  zooming.
- No pinch-to-zoom is required in Phase 0.

### Milestone 6: draw mode (state machine + document history)

Goal: place points, toggle edge mode, close/commit, with undo.

Tool state machine:

- `ToolMode`: `Draw` or `Manipulate`.
- `DrawEdgeMode`: `Line` or `BezierAuto`.
- `DrawState`: active open path (shape id), hover point for previews.
- `ManipState`: hover selection item and drag state.

Draw behaviour:

- Click appends an anchor to the active path.
- `Tab` toggles `Line` ↔ `BezierAuto`.
- Closing the path (click first anchor within a snap radius, or explicit close)
  sets `closed = true`, ensures a fill is present, and switches to Manipulate.
- `Esc` commits the open path as-is (still open) and switches to Manipulate.

Bezier auto:

- Use Catmull–Rom to cubic Bézier conversion to generate “pleasant smooth”
  control points. This is a curve-through-points approximation, not a
  least-squares fit.

History:

- Each click (anchor append) is one document history entry.
- Drag gestures are grouped to produce one undo step per gesture.

Acceptance:

- The user can draw a path and undo point placement with Ctrl-Z.

### Milestone 7: manipulate mode (hit-testing + ops)

Goal: select and edit geometry.

Hit-testing priority:

1. Handle points,
2. Anchor points,
3. Segment (line distance threshold; cubic via polyline sampling),
4. Shape interior (optional for Phase 0).

Operations:

- Drag selected anchors: `MoveAnchor` ops.
- Drag handles: `MoveHandleIn` / `MoveHandleOut` ops.
- Drag selected shape: translate all anchors/handles.
- Toggle segment kind line ↔ cubic:
  - Line → cubic initial handles are 1/3 and 2/3 along the line.
  - Cubic → line clears relevant handles.
- Raise/lower: reorder within document z-order.
- Insert/delete anchors: encode as “replace entire shape” ops in Phase 0 to
  keep ops invertible without complex structural patching.

Acceptance:

- The user can select a shape, drag it, and undo the move.
- The user can drag an anchor and see the path update.

### Milestone 8: selection history (second undo stack)

Goal: selection changes have their own undo/redo history.

Implement:

- When selection changes, push a `SelectionChange { from, to }` into
  `selection_history`.
- Ctrl-Z / Ctrl-Y operate on document history.
- Ctrl-Shift-Z / Ctrl-Shift-Y (or equivalent bindings) operate on selection
  history.

Implementation note:

- GPUI supports Actions and keymaps. Prefer Actions for shortcuts and menu
  integration.
- The “Shift selects selection history” rule should also apply to toolbar/menu
  undo/redo buttons if present.

Acceptance:

- Undoing with Shift reverts selection changes without changing the document.

### Milestone 9: behavioural tests (BDD) for the controller layer

Goal: protect user-visible behaviour without needing a real window.

Toolchain note:

- These tests currently run on `nightly-2025-10-23` because upstream
  `rstest-bdd` uses nightly-only language features.
- Once `rstest-bdd` supports stable, this repository should return to stable and
  keep the same BDD scenarios passing.

Test approach:

- Extract input-driven logic into “controller” functions that operate on a
  plain `EditorState` or `EditorModel` (no GPUI types in signatures).
- BDD steps call those controller APIs, then assert observable outputs such as
  the exported SVG.

Structure:

- Feature files under `tests/features/gauss_poc.feature` (and more as needed).
- Scenario tests can live under `src/` behind `#[cfg(test)]` so they can access
  internal modules without forcing large public APIs.
- Follow `docs/rstest-bdd-users-guide.md` conventions:
  - Step functions use `#[given]`, `#[when]`, `#[then]`.
  - Scenarios are bound with `#[scenario(path = "...", name = "...")]`.
  - Use fixtures for shared state, preferably a mutable “world” struct.

Acceptance:

- A smoke BDD scenario exists and passes (proving the harness and toolchain
  integration works).
- At least one BDD scenario covers: draw three points, close path, export SVG,
  and the SVG contains a `<path>` with `Z`.

### Milestone 9b: GPUI integration tests (headless wiring)

Goal: validate GPUI wiring (actions, platform prompts, input events) without
requiring a real window manager.

Approach:

- Enable GPUI’s test support feature for tests, and write integration tests
  using `#[gpui::test]` with `TestAppContext`.
- Use `VisualTestContext` to simulate actions/keystrokes and verify that the
  view wiring performs platform interactions (such as file prompts).

Acceptance:

- A headless GPUI test exists for “Save…” that asserts:
  - The save action prompts for a new path.
  - Simulated selection is received by Gauss and updates state.

## Concrete steps

Run all commands from `/mnt/home/leynos/Projects/gauss`.

At the end of each milestone, run the relevant gates and keep logs:

    set -o pipefail
    mkdir -p target/logs
    (make check-fmt) 2>&1 | tee target/logs/check-fmt.log
    echo "exit=$?"

    set -o pipefail
    mkdir -p target/logs
    (make lint) 2>&1 | tee target/logs/lint.log
    echo "exit=$?"

    set -o pipefail
    mkdir -p target/logs
    (make test) 2>&1 | tee target/logs/test.log
    echo "exit=$?"

Manual smoke checks (once the binary exists):

    cargo run

Expected outcomes are described in each milestone’s acceptance criteria.

## Validation and acceptance

The Phase 0 PoC is accepted when:

- Quality gates: `make all` passes.
- Manual behaviour:
  - A window opens and draws on a canvas.
  - Draw mode and Manipulate mode behave as described.
  - Open/Save show native dialogs and load/save SVG.
  - Undo/redo works for document edits and selection edits (Shift toggles).
- Tests:
  - Unit tests exist for viewport and ops behaviour.
  - At least one BDD scenario exists and passes, asserting observable output
    (SVG).
  - At least one `#[gpui::test]` integration test exists and passes, asserting
    platform prompt wiring (Save dialog).

## Idempotence and recovery

- Milestones are designed to be re-runnable and to keep the project in a
  buildable state.
- If a gate fails, inspect the saved logs in `target/logs/*.log`.
- Prefer reverting via Git if a milestone needs to be restarted.
- Avoid printing to stdout/stderr in production code; surface errors in the UI.

## Interfaces and dependencies

Dependencies must be pinned (caret requirements) and recorded in `Cargo.lock`.

GPUI references:

- Use `docs/rustdoc-gpui-0.2.2/` as the primary reference for GPUI APIs.
- In particular, platform file prompts exist in the `platform` module and use:
  - `PathPromptOptions`
  - `prompt_for_paths`
  - `prompt_for_new_path`

## Revision note (required when editing an ExecPlan)

Initial creation (2025-12-17):

- Converted the original PoC sketch into a milestone-driven plan aligned with
  stable Rust, native GPUI file dialogs, and the repository’s testing rules
  (`rstest`, with `rstest-bdd` planned for behavioural tests).

Revision (2025-12-17):

- Marked Milestone 0 progress items completed after switching the toolchain to
  stable and landing a minimal GPUI window skeleton.
- Documented the mismatch between stable Rust and upstream `rstest-bdd` (which
  requires nightly today). This was later superseded by a user-driven toolchain
  pivot back to nightly.

Revision (2025-12-17):

- Updated the “Concrete steps” log paths to use `target/logs/` so they work in
  sandboxed environments and can be checked when commands are run via `tee`.
- Recorded current progress on the model layer (core types + viewport unit
  test).

Revision (2025-12-17):

- Updated the toolchain plan to pin `nightly-2025-10-23` temporarily so
  `rstest-bdd` behavioural tests can run.
- Added a minimal `rstest-bdd` smoke scenario to validate that the harness
  works under `cargo test`.

Revision (2025-12-17):

- Enabled GPUI's `test-support` feature in tests and added a headless GPUI test
  that dispatches the Save action and simulates choosing a path.

Revision (2025-12-17):

- Implemented SVG import/export modules (with unit tests and a round-trip
  regression test).
- Updated the Phase 0 “Save…” wiring to write an SVG file to disk and extended
  the headless GPUI test to assert that the saved SVG contains a demo shape.

Revision (2025-12-17):

- Added Phase 0 “Open…” wiring and a headless GPUI test that loads an SVG file.
- Documented the `prompt_for_paths` limitation in GPUI 0.2.2 test support and
  the corresponding test-only prompt-mode workaround.

Revision (2025-12-17):

- Implemented Phase 0 viewport navigation with scroll wheel pan and
  Ctrl/Cmd-wheel zoom around the cursor.
- Added a headless `#[gpui::test]` that simulates `ScrollWheelEvent` and
  asserts both pan updates and cursor-centred zoom (world point stability).

Revision (2025-12-17):

- Started Milestone 7 by adding manipulate-mode shape hit-testing, selection,
  and drag-to-move.
- Added a headless `#[gpui::test]` asserting that dragging a shape translates
  anchors and that undo restores the original geometry.

Revision (2025-12-17):

- Extended manipulate mode with anchor hit-testing and drag-to-move-anchor.
- Added a headless `#[gpui::test]` asserting anchor drag updates the path and
  that undo restores the original anchor position.

Revision (2025-12-17):

- Implemented selection history as a separate undo/redo stack.
- Updated keyboard mapping so Shift selects the selection stack
  (Ctrl-Shift-Z/Y).
- Added a headless `#[gpui::test]` that selects an item, clears selection, and
  asserts Shift-undo/redo restores the selection without mutating the document.

Revision (2025-12-17):

- Extended the `rstest-bdd` harness with an SVG export scenario that models
  drawing a closed triangle and asserts the exported `d` attribute contains a
  close (`Z`) command.

Revision (2025-12-17):

- Refactored Phase 0 manipulate mode into submodules to stay under the
  repository’s per-file line limit.
- Added handle hit-testing and drag-to-move-handle support, along with a
  headless `#[gpui::test]` asserting handle drag mutates only the handle and
  Ctrl/Cmd-Z restores the original geometry.

Revision (2025-12-18):

- Added manipulate-mode segment hit-testing (line distance + sampled cubic) and
  segment selection.
- Updated Tab handling so, in manipulate mode, Tab toggles the selected
  segment’s kind (line ↔ cubic) and seeds/clears handles appropriately.
- Added a headless `#[gpui::test]` that selects a segment, presses Tab, and
  asserts Ctrl/Cmd-Z restores the line segment and clears handles.

Revision (2025-12-18):

- Implemented raise/lower reordering in manipulate mode, bound to Ctrl/Cmd-`[`
  and Ctrl/Cmd-`]`.
- Added a headless `#[gpui::test]` that draws two overlapping shapes and
  asserts lower/raise reorder within the document, plus undo behaviour.

Revision (2025-12-18):

- Added PoC-quality anchor insertion (press `i` on a selected segment) and
  anchor deletion (press `Backspace`/`Delete` on selected anchors) in
  manipulate mode.
- Implemented both operations as “replace shape” document edits for simple
  invertibility and undo/redo behaviour.
- Added a headless `#[gpui::test]` asserting insert/delete behaviour and
  document undo/redo restores the expected anchor counts.

Revision (2025-12-18):

- Updated the Architecture and Milestone 3 notes to match the actual Phase 0
  implementation (`Phase0Shell` as the root view containing editor state).
- Marked the model layer and Phase 0 UI rendering progress items complete, and
  recorded that `make all` passes (logs captured under `target/logs/`).

Revision (2025-12-18):

- Added stroke/fill controls to the Phase 0 header via `gpui-component`’s
  colour picker, including undoable style application when a shape is selected.
- Added a headless `#[gpui::test]` that updates stroke and fill via the
  `Phase0Shell` test hooks and asserts undo restores the original style.
- Added a “Quit” button and configured the window to use environment
  decorations while disabling user resizing.
- Added a headless `#[gpui::test]` that clicks the Quit button and asserts the
  shell recorded the quit request (the GPUI test platform does not necessarily
  exit when `App::quit()` is invoked).
