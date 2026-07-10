# Using GPUI and GPUI Component in Gauss

GPUI is a hybrid immediate/retained, GPU-accelerated user interface (UI)
framework for Rust. GPUI Component sits on top of GPUI and provides a themed,
Tailwind-style set of UI controls and layout helpers.

This document captures the lessons from the Phase 0 proof of concept (PoC) and
provides guidance for Phase 1, which will introduce a functional user
interface. Gauss now uses a split workspace: `gauss-core` owns the pure editor
model, `gauss-svg` owns SVG persistence, and the root `gauss` package keeps the
GPUI application shell. The guidance here applies to that root app package and
uses GPUI 0.2.2 and GPUI Component 0.5.1 as pinned in this repository.

## Why this exists

GPUI is pre-1.0 and changes frequently. The best way to stay productive is to
standardize how the app boots, how state is organized, how the UI is rendered,
and how behaviour is tested. This guide records the patterns that worked in
Phase 0, highlights pitfalls, and outlines how to extend the UI safely.

## Version and platform constraints

- GPUI is pre-1.0, so API changes are expected between versions.
- This repo pins `gpui = "0.2.2"` and `gpui-component = "0.5.1"` in
  `Cargo.toml`. When either changes, update this guide.
- Track versions intentionally and expect occasional update work.
- GPUI targets macOS and Linux and expects the latest stable Rust. This repo
  uses a nightly toolchain to support behavioural testing, but the API patterns
  are stable-friendly.
- On macOS, GPUI uses Metal and requires Xcode and the command line tools. Run
  the following once after installing Xcode:

  ```sh
  xcode-select --install
  sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer
  ```

- Local rustdoc copies live under `docs/rustdoc-gpui-0.2.2` and
  `docs/rustdoc-gpui-component-0.5.1`. They are not committed, so a fresh clone
  will not include them.

## Getting started from scratch

### Add dependencies

The pinned versions below match the repo. The optional assets crate should stay
on the same version as `gpui-component`.

```toml
[dependencies]
gpui = "0.2.2"
gpui-component = "0.5.1"
# Optional, for bundled icon assets
# gpui-component-assets = "0.5.1"
anyhow = "1.0"
```

### Entry point skeleton

All GPUI apps follow the same boot sequence:

1. Create an `Application`.
2. Call `gpui_component::init(app)` inside the `Application::run` closure.
3. Open a window and install a `Root` view as the first view in the window.
4. Create the root view entity and pass it to `Root::new`.

Minimal example:

```rust
use gpui::*;
use gpui_component::Root;

struct HelloWorld;

impl Render for HelloWorld {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().child("Hello, World!")
    }
}

fn main() {
    let app = Application::new();
    // If gpui-component-assets is used:
    // let app = Application::new().with_assets(gpui_component_assets::Assets);

    app.run(|app| {
        gpui_component::init(app);

        if app
            .open_window(WindowOptions::default(), |window, cx| {
                let view = cx.new(|_| HelloWorld);
                cx.new(|root_cx| Root::new(view, window, root_cx))
            })
            .is_err()
        {
            app.quit();
        }
    });
}
```

Gauss centralizes this setup in `gauss::ui::init`, and the root app entrypoint
in `src/main.rs` calls it before opening the window.

## Mental model: three registers

GPUI supports three primary registers. The PoC uses all three.

1. **Entities (state and communication)**
   - `Entity<T>` is owned by the app. State lives here.
   - Entities suit shared editor state or view state that needs to be read or
     updated across components.
   - Access entity data via a context (`Context<T>` or `App`) and call
     `Context::notify` when state changes need to wake observers.

2. **Views (declarative UI)**
   - Views are entities that implement `Render`.
   - Views compose the UI tree (`div`, components, etc).
   - GPUI calls `render` on the root view each frame.

3. **Elements (imperative UI)**
   - Elements give direct control over layout and rendering.
   - Elements handle custom drawing (for example, `Canvas`) or specialized
     layout.

The PoC uses a view (`Phase0Shell`) that renders a toolbar and a canvas. The
canvas uses GPUI's low-level drawing surface via `Canvas` and `PathBuilder`.

## Contexts used in Gauss

- `Application` / `App`: create windows and access global state.
- `Context<T>`: entity context, dereferences to `App`.
- `Window`: the active window; required by `Render`.
- `AsyncApp` and `AsyncWindowContext`: created via `to_async`, can be held
  across `.await` points but become fallible because the app or window may
  disappear.
- `TestAppContext`: test-only contexts that panic if the app or window is
  missing and provide extra helpers for input simulation.

## GPUI Component in practice

### When to use components

GPUI Component is the default for standard UI controls (buttons, inputs,
dropdowns, colour pickers, etc). It provides consistent theming and sizing and
keeps custom drawing focused on the canvas.

### Initialize once

`gpui_component::init(app)` must be called inside the `Application::run`
closure before using any components. This initializes theme state and registers
component services.

### Root wrapper is required

The first view in every window must be a `gpui_component::Root`. This wrapper
hosts theming, dialog layers, and other global component plumbing.

### Stateless vs stateful components

- Most components are stateless `RenderOnce` elements used directly in
  `render`.
- Some components manage internal state and therefore implement `Render`. These
  must be created as entities and stored on the view struct.

Example (stateful input):

```rust
struct MyView {
    input: Entity<gpui_component::input::InputState>,
}

impl MyView {
    fn new(window: &Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| {
            gpui_component::input::InputState::new(window, cx).default_value("Hello")
        });
        Self { input }
    }
}

impl Render for MyView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        self.input.clone()
    }
}
```

### Theming

GPUI Component exposes theme data via the `ActiveTheme` trait, implemented for
`App`. Because `Context<T>` dereferences to `App`, `cx.theme()` is available in
views and elements.

### Sizing and variants

Most components support sizes and variants with fluent methods:

```rust
Button::new("btn").small();
Button::new("btn").medium(); // default
Button::new("btn").large();

Button::new("btn").primary();
Button::new("btn").danger();
Button::new("btn").warning();
Button::new("btn").success();
Button::new("btn").ghost();
Button::new("btn").outline();
```

### Icons and assets

GPUI Component includes an `Icon` element and an `IconName` enum, but it does
not bundle Scalable Vector Graphics (SVG) icons by default. Icons require
either custom SVG assets or the optional `gpui-component-assets` crate plus
`Application::with_assets`.

```rust
use gpui_component::{Icon, IconName};

Icon::new(IconName::Check);
```

## Actions and keyboard input

GPUI is keyboard-first. Actions should be defined and bound in a key context to
expose keyboard behaviour.

Actions can be unit structs:

```rust
mod menu {
    actions!(gpui, [MoveUp, MoveDown]);
}
```

Or richer types:

```rust
mod menu {
    #[gpui::action]
    struct Move {
        direction: Direction,
        select: bool,
    }
}
```

Bind actions on elements with `on_action` and scope key bindings with
`key_context`:

```rust
div()
    .key_context("menu")
    .on_action(|this: &mut Menu, _: &menu::MoveUp, _window, _cx| {
        // ...
    });
```

Key bindings can be expressed as JSON keymaps (action names are fully qualified
Rust paths). Gauss currently registers bindings in code via `App::bind_keys` in
`src/ui/phase0_shell/mod.rs`. Key context strings must use letters, digits,
`_`, or `-`.

## Canvas and drawing

For the PoC, rendering stays inside a `Canvas` element with `PathBuilder`. This
provides low-level control without writing a full custom element.

Recommended patterns:

- Convert model-space points to screen-space through a `Viewport` transform.
- Build separate fill and stroke paths for each shape.
- Draw fill first, then stroke, then selection overlays.
- Use a small, predictable marker size for anchors and handles.

GPUI's `PathBuilder` supports lines and cubic Bezier curves, which match the
model representation. Use `PathBuilder::fill()` and `PathBuilder::stroke()` to
build `Path` objects for the canvas.

## File dialogs and platform prompts

The PoC uses native file dialogs for Open/Save via GPUI platform prompts. The
headless test backend does not implement `prompt_for_paths`, so tests route
Open via `prompt_for_new_path`. `Phase0Shell::new_for_tests` exists to support
headless testing and should remain.

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

Behaviour-heavy tests stay at the controller/model boundary using `rstest-bdd`,
with a few targeted `#[gpui::test]` integrations for wiring and input
behaviour. This is the recommended split for Phase 1.

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

## Pitfalls encountered (and how to avoid them)

- **Unfulfilled lint expectations**: Only add `#[expect]` when the lint is
  actually triggered in that module.
- **`let _ =` with `#[must_use]`**: Use a named binding (for example,
  `let _cleanup =`) to satisfy the lint.
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

- GPUI docs: `docs/rustdoc-gpui-0.2.2/gpui/index.html`
- GPUI Component docs:
  `docs/rustdoc-gpui-component-0.5.1/gpui_component/index.html`

These local docs should be consulted before introducing new GPUI APIs so the
project stays aligned with the pinned versions.
