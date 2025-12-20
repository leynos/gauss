# Build Phase 1 basic UI shell

This ExecPlan is a living document. The sections `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must
be kept up to date as work proceeds.

No `PLANS.md` file exists in the repository at the time of writing, so this
plan is self-contained and is the single source of truth for the Phase 1 basic
UI work.

## Purpose / Big Picture

Deliver a clean, Zed-inspired UI shell for Gauss that wraps the existing Phase
0 editor functionality. The result should present a top chrome row, a left tool
rail, a central canvas area, and a bottom status bar with a calm, neutral
palette. Icons in `icons/` should appear in the UI, and any feature that is not
implemented yet must show a greyed-out placeholder icon rather than an active
control. Success is visible by running the app and seeing the layout closely
match the provided screenshot while core actions (open/save, undo/redo, tool
mode switching) still work.

## Progress

- [x] (2025-12-20 19:57Z) Create the initial ExecPlan.
- [x] (2025-12-20 20:06Z) Survey the current UI entrypoints and identify which
  elements can remain stable for tests (for example the canvas selector).
- [x] (2025-12-20 20:16Z) Prototyping milestone: confirm the best GPUI API for
  rendering custom SVG icons from `icons/` in a way that works on both desktop
  and headless tests.
- [x] (2025-12-20 20:31Z) Implement the Zed-inspired chrome layout around the
  existing editor canvas, keeping interaction behaviour unchanged.
- [x] (2025-12-20 20:31Z) Wire buttons to available actions and render
  placeholder icons for unavailable functionality.
- [x] (2025-12-20 20:39Z) Update or extend tests to validate the new layout
  where it is observable (completed: existing selectors and behaviours were
  preserved, so no new tests were required).
- [x] (2025-12-20 20:39Z) Run formatting, linting, and tests using the Makefile
  gateways.
- [x] (2025-12-20 22:15Z) Apply the polish pass: remove the top-left menu
  icon, clip the canvas contents to the viewport bounds, add tooltips to icon
  buttons, move vertical alignment controls to the same toolbar as the
  horizontal ones, and switch window controls to the new icons.
- [x] (2025-12-20 22:33Z) Re-run the formatting, Markdown validation, linting,
  and test gateways after the polish adjustments.
- [x] (2025-12-20 23:10Z) Split the chrome helpers into dedicated modules so
  the main chrome layout file stays under 400 lines.

## Surprises & Discoveries

- Observation: GPUI tests depend on the `Phase0Shell` root view and the
  `#phase0-canvas` debug selector. Evidence: `tests/common/mod.rs` uses
  `debug_bounds("#phase0-canvas")`, and most GPUI tests construct
  `Phase0Shell::new` as the view.
- Observation: GPUI's `img()` element accepts an `Image` built from SVG bytes,
  which avoids runtime filesystem access and should work in headless tests.
  Evidence: `gpui::Image::from_bytes` accepts `ImageFormat::Svg`, and
  `ImageSource` supports `Image` inputs via `img()`.

## Decision Log

- Decision: Keep `Phase0Shell` as the root view and evolve its layout rather
  than introducing a brand-new root view for Phase 1. Rationale: Most GPUI
  tests are written against `Phase0Shell`, so this avoids widespread test churn
  while still allowing a complete UI refresh. Date/Author: 2025-12-20, Codex.
- Decision: Render icons with
  `gpui::img(Image::from_bytes(ImageFormat::Svg, ...))` using `include_bytes!`
  for the `icons/` assets. Rationale: `img()` accepts `Image` sources and SVG
  bytes directly, avoiding runtime file access and keeping headless tests
  stable. Date/Author: 2025-12-20, Codex.
- Decision: Keep the existing colour pickers in the chrome so style tooling
  remains available while the UI is restructured. Rationale: This avoids dead
  code warnings and preserves Phase 0 style editing without adding a new
  inspector panel. Date/Author: 2025-12-20, Codex.
- Decision: Add tooltips to chrome icon buttons and move all alignment icons
  into the document header toolbar. Rationale: Tooltips provide clarity for the
  newly icon-heavy UI, and colocating alignment controls keeps layout tools
  predictable. Date/Author: 2025-12-20, Codex.
- Decision: Replace the text close control with window control icons and clip
  the canvas container with `overflow_hidden`. Rationale: The icons match the
  desired chrome styling and clipping prevents shapes from drawing outside the
  canvas bounds. Date/Author: 2025-12-20, Codex.

## Outcomes & Retrospective

Pending. This will be completed once the Phase 1 UI ships and is verified.

## Context and Orientation

Gauss is a GPUI application. The entrypoint in `src/main.rs` creates a window
and installs `gauss::ui::Phase0Shell` inside a `gpui_component::Root`. UI code
lives under `src/ui/phase0_shell/`. The existing layout is defined in
`src/ui/phase0_shell/view.rs`, with the chrome layout assembled in
`src/ui/phase0_shell/chrome.rs`. The canvas element is identified by the
`phase0-canvas` debug selector and is referenced by GPUI tests under `tests/`.

The Zed-like chrome should be built using GPUI Component where possible. The
reference guidance for component usage and theming is
`docs/using-gpui-and-gpui-component.md`. Custom SVG icons are stored in the
repository at `icons/` and are Apache-licensed from IBM Carbon. Use
placeholders (greyed out) for any icon that represents functionality not yet
implemented.

## Plan of Work

First, confirm which UI elements are relied on by tests and must retain stable
selectors (for example `phase0-canvas`). Next, decide how to render the SVG
icons. Prefer embedding icons using `include_str!` or `include_bytes!` so no
runtime filesystem access is required; consult the local GPUI rustdoc to choose
between `gpui::Svg`, `gpui::Image`, or GPUI Component icon APIs. If this cannot
be determined confidently, introduce a small prototyping module that renders a
single icon in the window and delete it once the approach is validated.

Update the Phase 0 shell layout to match the Zed-like chrome. Extract the new
layout into new modules under `src/ui/phase0_shell/` (for example `chrome.rs`,
`tool_rail.rs`, `status_bar.rs`) to keep file sizes under the 400-line limit
and to maintain clarity. Each new module must begin with a module-level `//!`
comment describing its purpose. Keep existing model and controller behaviour
intact; this change is visual and structural, not a behavioural rewrite.

Add tooltips to icon buttons and ensure the canvas container clips its content
to the visible bounds. Replace the textual window close control with the new
window control icons, keeping placeholders for any unimplemented behaviours.

Implement a small icon/button helper that takes an icon identifier, button
state (active, enabled, placeholder), and optional action handler. The helper
should apply a consistent size, padding, and hover styling to match the Zed
feel. Placeholder buttons should render the correct icon at reduced opacity and
should not respond to input. Where a feature is already available (open/save,
undo/redo, tool mode switching, draw edge toggle), wire the button to existing
handlers in `Phase0Shell` so behaviour is unchanged.

The new layout should include:

- A top chrome row with recent project text, document title, and right-aligned
  quick actions. Align spacing and typography to feel light and restrained,
  with thin separators between groups.
- A left tool rail with the primary selection/draw tools and mode toggle,
  using the provided icons. Use placeholders for tools that are not wired yet.
- A central canvas area that keeps the same event handlers and debug selector
  as today.
- A bottom status bar that shows zoom controls, snapping, and status text.
  Unavailable controls should use placeholder styling.

Update any tests that rely on layout text or positions. Prefer to keep existing
selectors stable so tests remain resilient. If new UI elements need testing,
add focused `#[gpui::test]` cases to assert that placeholders render in a
disabled state and that wired buttons still dispatch actions.

## Concrete Steps

Work from the repository root:

    cd /mnt/home/leynos/Projects/gauss

Review the current shell layout and references:

    rg -n "phase0-canvas|Phase0Shell" src/ui src/main.rs tests

Enumerate the available icons to map them to UI slots:

    ls -1 icons

Prototype the icon rendering if needed by adding a temporary view or helper,
then delete the prototype once the API is confirmed.

Implement the new chrome modules and update `Phase0Shell::render` to assemble
that layout.

If you update documentation, run the Markdown tooling through the Makefile:

    (set -o pipefail; timeout 300s make fmt 2>&1 | tee /tmp/gauss-fmt.log)
    (set -o pipefail; timeout 300s make markdownlint 2>&1 | \
      tee /tmp/gauss-markdownlint.log)
    (set -o pipefail; timeout 300s make nixie 2>&1 | tee /tmp/gauss-nixie.log)

Before committing code changes, run the full quality gate via Makefile commands
and inspect the exit codes:

    (set -o pipefail; timeout 300s make check-fmt 2>&1 | \
      tee /tmp/gauss-check-fmt.log)
    (set -o pipefail; timeout 300s make lint 2>&1 | tee /tmp/gauss-lint.log)
    (set -o pipefail; timeout 300s make test 2>&1 | tee /tmp/gauss-test.log)

To visually confirm the UI, run the app in another terminal after the build
passes:

    timeout 300s cargo run

## Validation and Acceptance

- Running `cargo run` opens a window with the new chrome layout that visually
  matches the Zed-inspired screenshot: top bar, left tool rail, central canvas,
  and bottom status bar with a calm grey palette.
- Buttons that are wired (open/save, undo/redo, tool mode switching, draw edge
  toggle) still work as before and update the editor state.
- Buttons that represent unimplemented features are rendered as greyed-out
  placeholders and do not respond to input.
- The canvas still responds to input, and the `phase0-canvas` selector remains
  valid for existing tests.
- Hovering any icon button shows a tooltip describing its function, and shapes
  do not render outside the canvas bounds when moved.
- `make check-fmt`, `make lint`, and `make test` all pass.
- If documentation changed, `make fmt`, `make markdownlint`, and `make nixie`
  pass.

## Idempotence and Recovery

The UI changes are safe to re-run; rebuilding or re-rendering does not mutate
external state. If a step goes wrong, revert file edits with version control
and re-apply the plan. Prototype code added for icon rendering should be
removed once the final approach is chosen.

## Artifacts and Notes

Icon mapping should be kept explicit. For example, map the following icons to
their UI groups and mark placeholders where behaviour is missing:

    Top chrome: file-new.svg, file-open.svg, file-save.svg, edit-undo.svg,
    edit-redo.svg, settings.svg, window-minimize.svg, window-maximize.svg,
    window-close.svg
    Tool rail: select.svg, draw-path.svg, draw-curve.svg, draw-square.svg,
    draw-circle.svg, mode-manipulate.svg, mode-draw.svg
    Status bar: zoom-in.svg, zoom-out.svg, zoom-area.svg, snap-to-grid.svg
    Arrange/align: align-*.svg, arrange-vertical.svg, move-*.svg
    Boolean ops: shape-*.svg

## Interfaces and Dependencies

- `src/ui/phase0_shell/view.rs` should continue to host
  `impl Render for Phase0Shell`, but it should delegate layout construction to
  new helpers so the file remains under 400 lines.
- New modules such as `src/ui/phase0_shell/chrome.rs` or
  `src/ui/phase0_shell/tool_rail.rs` should contain functions returning
  `impl gpui::IntoElement` and accept `&mut Phase0Shell`, `&mut Window`, and
  `&mut Context<Phase0Shell>` where they need event handlers.
- A shared icon helper module (for example `src/ui/icon_assets.rs`) should
  provide a stable API such as:

    pub enum UiIcon { Open, Save, Undo, Redo, Select, DrawPath, WindowClose, … }

    pub fn icon_element(icon: UiIcon, size: f32) -> impl gpui::IntoElement

  The helper should be implemented using the GPUI SVG API discovered during the
  prototyping milestone. Avoid runtime filesystem reads; embed SVG assets at
  compile time.
- Use GPUI Component theme values via `cx.theme()` where possible to keep the
  palette consistent with the rest of the app. If explicit colours are needed,
  define them once and reuse them across the chrome modules.

## Revision note (required when editing an ExecPlan)

Updated the Progress section to mark the UI entrypoint survey as complete and
added a discovery noting the `Phase0Shell` and `#phase0-canvas` test
dependencies. This clarifies which selectors must remain stable before the UI
layout work begins.

Updated the Progress section to mark the SVG icon API prototyping as complete
and recorded the decision to use `gpui::img` with embedded SVG bytes. This
narrows the implementation choices for the upcoming chrome work.

Adjusted the icon rendering decision text to keep line lengths within the
documentation style guide limits.

Recorded completion of the initial chrome layout and button wiring work, and
noted the decision to retain the existing colour picker controls in the new
layout.

Marked the test and quality gate steps complete after confirming the updated
layout kept existing test selectors intact and all Makefile checks passed.

Documented the follow-up polish work to remove the menu icon, add tooltips,
clip the canvas, move alignment controls into the header toolbar, and switch to
the new window control icons, updating the progress, plan, and decision log to
match the latest UI adjustments.

Recorded the chrome layout refactor that moved tool rail and panel helpers into
dedicated modules to keep `chrome.rs` below the 400-line threshold.
