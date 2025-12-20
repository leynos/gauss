# Using GPUI and GPUI Component in Gauss

This document captures what we learned while building the Phase 0 PoC and
provides guidance for Phase 1, which will introduce a functional user
interface. It assumes the current project structure and uses GPUI 0.2.2 and
GPUI Component 0.5.0 as pinned in this repository.

## Why this exists

GPUI is pre-1.0 and changes frequently. The best way to stay productive is to
standardise how we boot the app, organise state, render the UI, and test our
behaviour. This guide records the patterns that worked in Phase 0, highlights
pitfalls, and outlines how to extend the UI safely.

## Version and platform constraints

- GPUI is pre-1.0, so API changes are expected between versions.
- It targets macOS and Linux and expects the latest stable Rust. In this repo
  we currently use a nightly toolchain to support behavioural testing, but the
  API patterns are stable-friendly.
- Track versions intentionally and expect occasional update work.

## Mental model: three registers

GPUI supports three primary registers. We use all three in the PoC.

1. **Entities (state and communication)**
   - `Entity<T>` is owned by the app. State lives here.
   - Use entities for shared editor state or view state that needs to be read
     or updated across components.
   - Access entity data via context (`Context<T>` or `App`) with `read` and
     `update`.

2. **Views (declarative UI)**
   - Views are entities that implement `Render`.
   - Use views to compose the UI tree (`div`, components, etc).
   - GPUI calls `render` on the root view each frame.

3. **Elements (imperative UI)**
   - Elements give direct control over layout and rendering.
   - Use elements when you need custom drawing (e.g. `Canvas`) or specialised
     layout.

The PoC uses a view (`Phase0Shell`) that renders a toolbar and a canvas. The
canvas uses GPUI's low-level drawing surface via `Canvas` and `PathBuilder`.

## Contexts you will touch

- `Application` / `App`: create windows and access global state.
- `Context<T>`: entity context, dereferences to `App`.
- `Window`: the active window; required by `Render`.
- `TestAppContext`: headless testing and input simulation.

In async workflows, GPUI provides `to_async` contexts. We do not rely on async
contexts in Phase 0, but the API exists for background tasks.

## Bootstrapping the app

The minimal pattern used in this repo:

- Create an `Application`.
- Call `gpui_component::init(cx)` inside `app.run`.
- Open a window and set the root view to a `Root` wrapper.

Example (trimmed):

```rust
let app = Application::new();
app.run(|cx| {
    gpui_component::init(cx);
    cx.open_window(WindowOptions::default(), |window, cx| {
        let view = cx.new(|_| Phase0Shell::new(cx));
        cx.new(|cx| Root::new(view, window, cx))
    });
});
```

The `Root` wrapper is required by GPUI Component for theming and layout.

## GPUI Component in practice

### When to use components

Use GPUI Component for standard UI controls: buttons, inputs, dropdowns, colour
pickers, etc. It provides consistent theming and sizes and lets the canvas
remain the only custom-drawn piece.

### Initialise once

`gpui_component::init(cx)` must be called inside the `app.run` closure before
using any components. This ensures theme state and global settings are ready.

### Stateless vs stateful

- Most components are stateless and are used directly in `render`.
- A few are stateful (e.g. inputs, lists) and must be created as entities and
  stored on the view struct.

### Colour picker and history

We use GPUI Component's colour picker module and `History` for undo/redo
stacks. This is a central dependency for Phase 1 UI work.

### Icons and assets

Icons are not bundled by default. If we want iconography in Phase 1, we must
add assets and use `Icon` with a supported `IconName`.

## Canvas and drawing

For the PoC we kept rendering inside a `Canvas` element with `PathBuilder`.
This provides low-level control without writing a full custom element.

Patterns that worked well:

- Convert model-space points to screen-space through a `Viewport` transform.
- Build separate fill and stroke paths for each shape.
- Draw fill first, then stroke, then selection overlays.
- Use a small, predictable marker size for anchors and handles.

GPUI's `PathBuilder` supports lines and cubic Bézier curves, which match our
model representation. Use `PathBuilder::fill()` and `PathBuilder::stroke()` to
build `Path` objects for the canvas.

## Actions and keyboard input

GPUI uses actions for key bindings. The reliable flow is:

1. Define actions (unit structs are easiest).
2. Set a `key_context` on the element subtree that should handle the bindings.
3. Register `on_action` handlers on that subtree.
4. Provide key bindings in the keymap file.

In Phase 0 we also handle some keys directly in view-level logic for
simplicity, but Phase 1 should move fully to actions so keyboard behaviour is
centralised and testable.

## File dialogs and platform prompts

The PoC uses native file dialogs for Open/Save via GPUI platform prompts. The
headless test backend does not implement `prompt_for_paths`, so tests route
Open via `prompt_for_new_path`. This is why `Phase0Shell::new_for_tests` exists
and should remain for headless testing.

Guidance for Phase 1:

- Keep dialog behaviour behind a thin view-level adapter so tests can swap the
  prompt implementation.
- Always record the last opened/saved path for test assertions and debugging.
- Clean up temporary files in tests using a small drop guard.

## Headless testing notes

GPUI's `#[gpui::test]` macro is viable for integration tests. Practical notes
from the PoC:

- Use `TestAppContext` and `VisualTestContext` for input simulation.
- Call `run_until_parked` after simulated input so the UI settles.
- Prefer test helpers in `tests/common` for repeated setup.
- When simulating platform prompts, assert prompt state before and after.

We kept behaviour-heavy tests at the controller/model boundary using
`rstest-bdd` and a few targeted `#[gpui::test]` integrations for wiring and
input behaviour. This is the recommended split for Phase 1.

## Model + controller separation

The PoC uses a clean split:

- **Model**: pure data types, operations, viewport transforms.
- **Controller**: tool state machines and input mapping.
- **View**: GPUI and GPUI Component rendering and event handlers.

Phase 1 should keep the controller logic independent from GPUI types where
possible. This keeps tests fast and reduces UI coupling.

## Patterns worth keeping for Phase 1

- Tool mode state machines are explicit enums.
- Viewport transform is isolated in `model::viewport`.
- Undo/redo uses GPUI Component `History` for grouping.
- Selection history is separate and uses a modifier to switch stacks.
- Hit-testing is kept deterministic and uses model-space geometry.
- Selection overlays and hit-testing share geometry utilities.

## Pitfalls we hit (and how to avoid them)

- **Unfulfilled lint expectations**: Only add `#[expect]` when the lint is
  actually triggered in that module.
- **`let _ =` with `#[must_use]`**: Use a named binding (e.g. `let _cleanup =`)
  to satisfy the lint.
- **Headless prompt differences**: `prompt_for_paths` is not available in the
  test platform; use `prompt_for_new_path` when testing Open.
- **Stale history after Open**: When swapping documents, clear undo/redo and
  selection histories so Undo cannot apply edits from an old file.

## Guidance for Phase 1 UI work

1. **Expand the toolbar first**
   - Keep it in a single view module and use GPUI Component controls.
   - Wire actions for tool switching and key bindings.

2. **Solidify action routing**
   - Move key handling to actions and key contexts.
   - Keep mouse input in view handlers, but have them invoke controller logic.

3. **Document-view boundaries**
   - Introduce a small adapter between UI events and model ops to reduce
     coupling.
   - Keep selection and document history changes in one place.

4. **Testing**
   - Add `#[gpui::test]` only for behaviour that requires platform wiring.
   - Keep core behaviours covered by unit and BDD tests.

## References

- GPUI docs: `docs/rustdoc-gpui-0.2.2`
- GPUI Component docs: `docs/rustdoc-gpui-component-0.5.0`

These local docs should be consulted before introducing new GPUI APIs so we
stay aligned with the pinned versions.
