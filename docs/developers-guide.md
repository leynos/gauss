# Developer's guide

## Purpose and audience

This guide is for contributors adding tests or extending behavioural coverage
in gauss. It documents the project's testing patterns, the new `rstest-bdd`
GPUI harness integration, and the conventions that govern them. For the
upstream `rstest-bdd` API reference and migration notes consult:

- [rstest-bdd user's guide](rstest-bdd-users-guide.md)
- [rstest-bdd v0.6.0 migration guide](rstest-bdd-v0-6-0-migration-guide.md)

## Test taxonomy

Gauss uses three test styles, each with a distinct role:

- **`#[gpui::test]` integration tests** — the default for low-level GPUI
  integration coverage. These tests live under `tests/gpui_*.rs`, receive a
  `&mut TestAppContext`, and use `VisualTestContext` directly. They remain the
  preferred choice when the behaviour fits a single function body and does not
  benefit from Gherkin readability.

- **`rstest`-based unit tests** — pure Rust unit tests (`#[cfg(test)] mod tests`
  blocks) using `rstest` fixtures and `#[rstest::rstest]` parameterisation.
  These test isolated logic in `gauss-core` and `gauss-svg`.

- **`rstest-bdd` behavioural tests** — Gherkin `.feature` files backed by step
  definitions and scenario bindings. The GPUI-backed variant uses
  `rstest_bdd_harness_gpui::GpuiHarness` to inject a `TestAppContext` into
  steps. These tests live under `tests/<name>_bdd.rs` with companion modules
  `tests/<name>_bdd/{steps.rs, world.rs}`.

## When to choose the GPUI BDD harness

Choose `GpuiHarness` when any of the following holds:

- the behaviour can be expressed in three or more observable transitions sharing
  setup,
- the assertions are narrative-friendly (status strings, accessibility output,
  mode flags), or
- the steps will be reused across multiple scenarios.

Otherwise prefer a single `#[gpui::test]`. Do not reach for `GpuiHarness` just
to wrap a one-shot assertion in Gherkin — the ceremony does not pay for itself
there.

## The pattern, end to end

This section walks through the shell mode indicator pilot at
`tests/gpui_shell_mode_indicator_bdd*/`.

### Feature file location

Place `.feature` files under `tests/features/` and name them after the domain
(`shell_mode_indicator.feature`, `history_draw_undo.feature`, etc.). Use
straight `Given-When-Then` scenarios — keep each scenario to one `When` step.

```gherkin
Feature: Shell mode indicator

  Scenario: Initial draw mode
    Given the Phase 0 shell is open
    Then the mode indicator reads "Mode: Draw (Line)"
```

### BDD module split

Each BDD test group needs three files:

```text
tests/<name>_bdd.rs            — entry: scenario bindings
tests/<name>_bdd/world.rs      — world struct + thread-local storage
tests/<name>_bdd/steps.rs      — step definitions
```

The entry file binds scenarios and declares any module aliases needed by the
macro's path resolution:

```rust
mod common;
mod gpui_shell_mode_indicator_bdd {
    pub(crate) mod steps;
    pub(crate) mod world;
}
use rstest_bdd_macros::scenario;

#[scenario(
    path = "tests/features/shell_mode_indicator.feature",
    harness = rstest_bdd_harness_gpui::GpuiHarness
)]
fn initial_draw_mode() {}
```

Scenario function signatures are empty when the world lives in thread-local
storage (see below). The `harness` parameter selects `GpuiHarness`, which runs
each scenario inside `gpui::run_test` and injects the `TestAppContext`.

### Accessing the harness context

Step functions receive the injected `TestAppContext` via the reserved fixture
key `rstest_bdd_harness_context`:

```rust
use rstest_bdd_macros::{given, when, then};

#[given("the Phase 0 shell is open")]
fn given_shell_open(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) {
    // … create window, store handles in world …
}

#[when("I press the \"tab\" key")]
fn when_press_tab(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) {
    // … simulate keystroke …
}

#[then("the mode indicator reads \"{expected}\"")]
fn then_mode_indicator_reads(
    #[from(rstest_bdd_harness_context)] cx: &TestAppContext,
    expected: String,
) -> TestSupportResult<()> {
    // … verify mode status line …
}
```

Use `&mut TestAppContext` in steps that mutate the application state (creating
windows, simulating input, calling GPUI's `update`). Use `&TestAppContext`
(shared) in read-only `Then` steps.

### Thread-local world (and why)

`StepContext::borrow_mut` takes `&mut self`, so a step that requests two
mutable fixtures (e.g. `&mut TestAppContext` for window creation and
`&mut ShellWorld` for storing the handle) produces a borrow conflict at compile
time. The generated code cannot hold two `FixtureRefMut` guards against the
same `StepContext` simultaneously.

**Workaround:** keep the world in a thread-local `RefCell` and access it
through a single `use`-imported function. Each step then borrows only one
mutable fixture from `StepContext` — the harness context. Thread-local storage
is safe because each GPUI integration test runs on a single thread.

```rust
// world.rs
use std::cell::RefCell;
use gpui::{AnyWindowHandle, Entity};
use gauss::ui::Phase0Shell;

pub(crate) struct ShellWorld {
    pub(crate) shell: Option<Entity<Phase0Shell>>,
    pub(crate) window: Option<AnyWindowHandle>,
}

impl Default for ShellWorld {
    fn default() -> Self {
        Self { shell: None, window: None }
    }
}

thread_local! {
    static WORLD: RefCell<ShellWorld> = RefCell::new(ShellWorld::default());
}

pub(crate) fn with_world<F, R>(f: F) -> R
where
    F: FnOnce(&RefCell<ShellWorld>) -> R,
{
    WORLD.with(f)
}
```

Steps use `world::with_world(|w| { … })` to read or mutate the world without
going through `StepContext`. The world resets naturally since each thread
starts with a fresh `RefCell`.

### Reusing helpers from `tests/common/mod.rs`

The shared module `tests/common/mod.rs` contains
`init_test_app(&mut TestAppContext)` and
`ensure_initial_draw(&mut VisualTestContext)`. Always delegate to these rather
than duplicating shell-construction logic in the world module.

### Returning `TestSupportResult` from steps

`Then` steps that can fail should return `TestSupportResult<()>`:

```rust
use test_support::{TestSupportError, TestSupportResult};

#[then("the mode indicator reads \"{expected}\"")]
fn then_mode_indicator_reads(
    #[from(rstest_bdd_harness_context)] cx: &TestAppContext,
    expected: String,
) -> TestSupportResult<()> {
    let shell = world::with_world(|w| {
        w.borrow()
            .shell
            .clone()
            .ok_or_else(|| TestSupportError::expectation("shell not open"))
    })?;
    let actual = cx.read(|app| shell.read(app).mode_status_line_for_tests());
    if actual == expected {
        Ok(())
    } else {
        Err(TestSupportError::expectation(format!(
            "expected mode indicator '{expected}', got '{actual}'",
        )))
    }
}
```

`TestSupportError` formats assertion failures consistently with the existing
`#[gpui::test]` test errors.

## Migrating an existing `#[gpui::test]` test

The recipe applied to the shell mode indicator pilot:

1. **Write the feature file** — capture the existing assertions as
   `Given-When-Then` steps in `tests/features/<name>.feature`.

2. **Create the BDD module skeleton** — entry file, `world.rs`, `steps.rs`.

3. **Port the setup to a Given step** — move `init_test_app` and
   `add_window_view` into a `#[given]` step that stores handles in the
   thread-local world.

4. **Port actions to When steps** — move keystroke simulation and
   `visual_cx.update` calls into `#[when]` steps. Reconstruct
   `VisualTestContext` from the stored `AnyWindowHandle` with
   `VisualTestContext::from_window(window, cx)`.

5. **Port assertions to Then steps** — move `assert_eq!` calls into `#[then]`
   steps that read through `cx.read(|app| …)`. Return `TestSupportResult<()>`
   with descriptive error messages.

6. **Delete the original `#[gpui::test]`** — once the BDD test is green.

The next obvious candidate for this migration is `gpui_shell_quit_button`.

## Breaking changes to remember

A focused summary of the v0.6.0 changes that affect gauss contributors:

- **Underscore fixture normalization.** Parameters named `_world` in scenario or
  step signatures now resolve to the `world` fixture key, not `_world`. Use
  `#[from(_world)]` when a literal underscore-prefixed key is required.

- **`Default` on harness types.** Harness types selected by
  `harness = …` in `#[scenario]` or `scenarios!` are instantiated with
  `Default`. `GpuiHarness` already derives `Default`.

- **`Result`/`StepResult` fixtures.** If a fixture returns `Result<T, E>`, the
  scenario function must also return a fallible type so the generated unwrap
  can use `?`.

- **`HarnessAdapter::run` returns `HarnessResult<T>`.** This only matters when
  writing a custom harness — gauss uses the first-party `GpuiHarness` directly.

## Running the gate

Every change that touches Rust or documentation sources must pass the triad:

```bash
make check-fmt
make lint
make test
```

Capture output to a per-branch log under `/tmp` as prescribed by `AGENTS.md`:

```bash
make test 2>&1 | tee "/tmp/test-gauss-$(git branch --show-current).out"
```

After modifying `docs/`, additionally run `make markdownlint`.
