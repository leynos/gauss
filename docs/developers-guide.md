# Developer's guide

## Validate lint and documentation policy

Install the pinned Rust toolchain from the repository root before running the
validation targets:

```sh
rustup install
```

The `rust-toolchain.toml` manifest installs `rustfmt`, `clippy`, and
`rust-analyzer` for the pinned compiler. Keep all three components in the
manifest so formatting, linting, and Language Server Protocol (LSP) analysis
use the same Rust release.

Run the required commit gates sequentially:

```sh
make check-fmt
make lint
make test
```

`make lint` performs three checks. Rustdoc builds the workspace documentation
with `RUSTDOC_FLAGS` set to `--cfg docsrs -D warnings` by default. Clippy then
checks the workspace with all targets and features, including integration
tests, before Whitaker applies its Dylint rules to the same target set.

The workspace denies undocumented public APIs, unsafe code, direct environment
access, panic-prone operations, lossy numerical conversions, debug output, and
the configured Clippy hygiene rules. Add documentation or repair the underlying
code instead of suppressing a lint.

`make test` runs the full nextest suite, falling back to `cargo test` when
nextest is unavailable. It then runs the equivalent of the following doctest
command, so examples compile with all workspace features:

```sh
RUSTFLAGS="-D warnings" RUSTDOCFLAGS="--cfg docsrs -D warnings" cargo test --workspace --doc --all-features
```

The target passes the configured `RUST_FLAGS` value to both test stages. When
supplying coverage configuration, sanitizer options, or other compiler flags,
retain `-D warnings` in the override so doctests use the same build
configuration and warning policy as the main suite.

## Namespace GitHub Actions runners

Gauss's repository-owned Linux CI and main-branch coverage jobs run on
`namespace-profile-default`: the shared Ubuntu 22.04 Linux/amd64 profile with
4 vCPU and 16 GB memory. Its Namespace cache volume is disabled for this
baseline rollout. Existing workflow cache actions remain unchanged; they are
not backed by a Namespace cache volume.

### Inject environment readers

Clippy rejects direct calls to `std::env::var`, `var_os`, `vars`, and `vars_os`
in production code. Define a narrow environment-reader port and inject its
implementation at the application boundary instead. Inject a stub reader in
tests, so cases remain deterministic.

Clippy also rejects `std::env::set_var` and `remove_var`. Do not mutate the
process environment in tests; configure the stub reader with the values needed
by each case.

## GPUI harness behavioural tests

The root package owns the GPUI integration tests under `tests/`. The GPUI
behavioural test pattern uses `rstest-bdd` scenarios with the first-party
`rstest_bdd_harness_gpui::GpuiHarness` to inject a `gpui::TestAppContext`.

The root `Cargo.toml` declares the three supporting development dependencies:

```toml
[dev-dependencies]
proptest = "1.11.0"
rstest-bdd-harness-gpui = "0.6.0-beta3"
serial_test = "3"
```

Use `proptest` for property-based tests where helper contracts must hold across
generated inputs, such as non-finite vector values and repeated temporary-file
cleanup. Keep those properties focused on the invariant under test, with
example-based tests covering representative scenario behaviour.

Use the harness only for integration tests that need a GPUI test context. A
scenario selects it through the canonical path, so the macro supplies the GPUI
test attribute:

```rust,no_run
use rstest_bdd_macros::scenario;
use serial_test::serial;

#[scenario(
    path = "tests/features/draw_undo.feature",
    name = "Draw clicks add anchors and undo removes them",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn draw_undo_scenario() {}
```

`#[serial]` protects stateful GPUI scenarios that share process-local test
state. Keep the scenario binding small; place the fixture-backed step
definitions and any durable `Entity` or window-handle state in the same test
module. See `tests/gpui_draw_undo_bdd.rs` for the complete working pattern.

Selection scenarios share their durable window lifecycle through
`tests/selection_bdd/support.rs`. That module stores only handles and screen
points needed by every selection test binary. Each binary defines its own typed
scenario payload and installs it with `set_scenario_data`; this keeps unused
fields out of binaries that do not need them. Framework-independent shape and
selection queries live in `test_support::selection`, where other integration
suites can reuse them without depending on GPUI. Include
`tests/common/selection_coordinates.rs` only in binaries that convert selection
coordinates, and keep pointer interaction helpers in `tests/common`.

Test support is organized around per-harness, capability-sized facades. This
replaces the deleted `tests/common/shared_helpers.rs`: each integration-test
binary owns a `tests/common/<harness>.rs` facade that declares only the focused
capability modules it needs and re-exports the narrow surface it consumes.
Shared implementations remain in focused modules under `tests/common/`; the
facade owns their composition for its harness.

For a new harness, add `tests/common/<harness>.rs` and include it with
`#[path = "common/<harness>.rs"] mod common;`. Declare only the capabilities
used by that harness and re-export only its helpers; do not recreate a global
helper module.

### Shell BDD support and test classification

Shell tests describe a user-visible action or observable state transition as a
Gherkin scenario under `tests/features/`. Keep render-tree presence, geometry,
and test-plumbing assertions as raw `#[gpui::test]` tests: Given/When/Then
language would hide that these checks depend on implementation structure rather
than behaviour. The retained structural inventory and rationale are recorded in
the
[v0.6.0 migration guide](rstest-bdd-v0-6-0-migration-guide.md#retain-structural-gauss-shell-tests-as-raw-gpui-tests).

The focused modules under `tests/shell_bdd/` provide shared support for the
behavioural shell scenarios:

- `support.rs` keeps shell-specific scenario state and rebuilds a
  `VisualTestContext` for each step. It uses `DurableShell` from
  `tests/common/durable_shell.rs` and the `scenario_state!` macro from
  `tests/common/scenario_state.rs`.
- `lifecycle.rs` re-exports the canonical, focused common helpers for
  application initialization and the initial draw.
- `click.rs` performs fallible selector-based clicks and drains pending GPUI
  work.
- `expect_equal.rs` and `expect_true.rs` return `TestSupportError` values from
  step assertions instead of panicking.

Include only the support modules that a test binary uses. Use a path attribute
such as `#[path = "shell_bdd/support.rs"]`. Selective inclusion keeps
unused-helper checks effective and avoids adding the complete shell support
surface to every GPUI integration test. Each stateful scenario must accept the
cleanup fixture and remain `#[serial]`.

### Integration-test inventory validation

The authoritative integration-test inventory comes from the root `gauss`
package in Cargo metadata. The checker reads the target source markers,
classifies each target, and verifies the documented inventory markers in the
four inventory documents. Run it through the Make target:

```sh
make check-integration-test-inventory
```

This target invokes the repository script with `uv` using the following
equivalent command:

```sh
UV_CACHE_DIR=.uv-cache UV_TOOL_DIR=.uv-tools uv run \
  scripts/check_integration_test_inventory.py
```

`markdownlint` depends on this target, so documentation validation also fails
when the documented inventory differs from the current Cargo metadata.

### Stateful history scenarios

The `gpui_history_bdd` binaries combine rstest-bdd 0.6.0-beta3's injected
`&mut TestAppContext` with state that must survive BDD step boundaries. This
combination requires a thread-local, resettable state workaround. Do not use
`ScenarioState` or `Slot` fixture injection for these scenarios: the borrowed
GPUI context cannot cross steps through those fixture values.

Store a `DurableShell` in the thread-local state instead of a
`VisualTestContext`. `DurableShell` keeps the `Entity<Phase0Shell>` and
`AnyWindowHandle` that remain valid across steps. Use `DurableShell::open` for
normal interaction scenarios and `DurableShell::open_for_tests` when a history
step needs the test-only shell seams. Each step receives a fresh context and
rebuilds its short-lived visual context with
`shell.with_visual(cx, |visual_cx, entity| { ... })`. Use `shell.entity()` only
for reads that do not need a visual context.

Mark every history scenario with `#[serial]`. The GPUI harness and the
thread-local state are process-local, so serial execution keeps scenarios from
overlapping. Reset the state before a scenario through an rstest fixture, and
reset it again when that fixture's cleanup guard is dropped. Setup steps may
also call `reset_state()` before replacing the state, but they must never hold
the state-cell borrow while using the injected `TestAppContext`.

The essential shape is:

```rust,ignore
thread_local! {
    static STATE: RefCell<HistoryState> = RefCell::new(HistoryState::default());
}

fn reset_state() {
    STATE.with(|state| *state.borrow_mut() = HistoryState::default());
}

#[fixture]
fn state_cleanup() -> StateCleanup {
    reset_state();
    StateCleanup
}

#[scenario(
    path = "tests/features/history_drag_anchor_undo.feature",
    name = "Dragging an anchor creates one undo entry and undo restores it",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn drag_anchor_history(#[from(state_cleanup)] _cleanup: StateCleanup) {}
```

The step that creates the shell stores the returned `DurableShell`; later steps
accept `#[from(rstest_bdd_harness_context)] cx: &mut TestAppContext` and call
`with_visual` for the duration of each interaction or assertion.

### Stateful file I/O scenarios

The `gpui_file_io_*` scenario binaries cover Save and export dialog behaviour,
where a `Given` step opens a window that a later `When` or `Then` step must go
on to drive. A `gpui::VisualTestContext` borrows from the `TestAppContext` it
was built against, and the harness hands each step a fresh
`&mut TestAppContext`, so a context saved from one step would be tied to a
stale borrow by the time the next step runs: it cannot cross a step boundary.
These binaries store a `DurableShell` in scenario state instead: a
`gpui::Entity<Phase0Shell>` paired with the window's `AnyWindowHandle`, both of
which are cheap to copy and remain valid across steps.
`DurableShell::with_visual_cx` rebuilds a `VisualTestContext` from that handle
for the lifetime of a closure, giving a subsequent step a live context to act
on.

The `scenario_state!` macro lives in `tests/common/scenario_state.rs`, which
each binary that needs it includes via
`#[path = "common/scenario_state.rs"] mod scenario_state;`, the same pattern
used for the file I/O helper modules described below. Because step functions
cannot thread state through their arguments, each scenario binary keeps a
thread-local state cell instead. Invoke `crate::scenario_state!(StateType)`
once at the binary's crate root with any `Default`-implementing state type; the
macro generates the `STATE` cell, `with_state` and `reset_state` helpers, the
`ScenarioStateCleanup` drop guard, and the `scenario_state_cleanup` rstest
fixture. Every `#[scenario]` function must accept
`#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup` so the state
is reset both before and after the scenario runs. The simplest case stores a
bare `Option<DurableShell>`:

```rust,no_run
crate::scenario_state!(Option<DurableShell>);

fn shell() -> Result<DurableShell, TestSupportError> {
    with_state(|state| state.clone())
        .ok_or_else(|| TestSupportError::missing("shell handles", "set by the Given step"))
}

#[scenario(
    path = "tests/features/file_io_click_save_button.feature",
    name = "Clicking Save opens the save prompt",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn clicking_save_opens_prompt(#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}
```

Scenarios that also exercise the filesystem hold a `TempSvgFile`: a
UUID-suffixed temporary SVG reached through a cap-std `Dir` capability rather
than an ambient path. Its owned `TempFileGuard` removes the file on drop, so
the scenario state owns the file's lifetime alongside the shell handle.
Scenario cleanup calls `TempSvgFile::cleanup` so removal failures propagate,
while `Drop` remains an idempotent best-effort fallback. File I/O must not run
while the `with_state` borrow is held; the save-dialog binary therefore stores
an `Option<Rc<TempSvgFile>>` and clones the `Rc` out of the cell before reading
the file, releasing the `RefCell` borrow first.

The file I/O helpers are deliberately not submodules of `common`. They live in
six focused modules under `tests/common/`, each pulled in only by the binaries
that use it, via `#[path = "common/<name>.rs"] mod <name>;`:

- `durable_shell.rs` — `DurableShell`, included by all four
  `gpui_file_io_*` binaries.
- `path_prompt.rs` — `assert_no_path_prompt` and `assert_path_prompt`,
  included by the click-save-button, open-dialog and save-dialog binaries.
- `temp_svg.rs` — `TempSvgFile`, included by the open-dialog,
  metadata-round-trip and save-dialog binaries.
- `temp_svg_write.rs`, `temp_svg_read.rs` and `temp_svg_exists.rs` — the
  `write`, `read_to_string` and `exists` operations on `TempSvgFile`, each
  included only by the binaries that perform that operation.

Splitting the helpers this finely means every module is fully used by every
binary that includes it, so the compiler still catches a genuinely unused
helper instead of a blanket `dead_code` expectation hiding it. It also keeps
these helpers out of the shared surface that every GPUI integration test
otherwise compiles. `assert_no_path_prompt(cx, context)` and
`assert_path_prompt(cx, context)` share the underlying
`did_prompt_for_new_path` check, while each step binding supplies its own
failure message via `context`.

See `tests/gpui_file_io_save_dialog.rs` for the fullest worked example,
combining a `DurableShell`, a shared `TempSvgFile`, and multiple export
scenarios, and `tests/features/file_io_*.feature` for the corresponding
specifications.
