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

### Stateful file I/O scenarios

The `gpui_file_io_*` scenario binaries cover Save and export dialog
behaviour, where a `Given` step opens a window that a later `When` or `Then`
step must go on to drive. A `gpui::VisualTestContext` borrows from the
`TestAppContext` it was built against, and the harness hands each step a
fresh `&mut TestAppContext`, so a context saved from one step would be tied
to a stale borrow by the time the next step runs: it cannot cross a step
boundary. These binaries store a `DurableShell` in scenario state instead: a
`gpui::Entity<Phase0Shell>` paired with the window's `AnyWindowHandle`, both
of which are cheap to copy and remain valid across steps.
`DurableShell::with_visual_cx` rebuilds a `VisualTestContext` from that
handle for the lifetime of a closure, giving a subsequent step a live
context to act on.

The `scenario_state!` macro lives in `tests/common/scenario_state.rs`, which
each binary that needs it includes via
`#[path = "common/scenario_state.rs"] mod scenario_state;`, the same pattern
used for the file I/O helper modules described below. Because step
functions cannot thread state through their arguments, each scenario
binary keeps a thread-local state cell instead. Invoke
`crate::scenario_state!(StateType)` once at the binary's
crate root with any `Default`-implementing state type; the macro generates
the `STATE` cell, `with_state` and `reset_state` helpers, the
`ScenarioStateCleanup` drop guard, and the `scenario_state_cleanup` rstest
fixture. Every `#[scenario]` function must accept
`#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup` so the
state is reset both before and after the scenario runs. The simplest case
stores a bare `Option<DurableShell>`:

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
UUID-suffixed temporary SVG reached through a cap-std `Dir` capability
rather than an ambient path. Its owned `TempFileGuard` removes the file on
drop, so the scenario state owns the file's lifetime alongside the shell
handle. Scenario cleanup calls `TempSvgFile::cleanup` so removal failures
propagate, while `Drop` remains an idempotent best-effort fallback. File I/O
must not run while the `with_state` borrow is held; the
save-dialog binary therefore stores an `Option<Rc<TempSvgFile>>` and clones
the `Rc` out of the cell before reading the file, releasing the `RefCell`
borrow first.

The file I/O helpers are deliberately not submodules of `common`. They live
in six focused modules under `tests/common/`, each pulled in only by the
binaries that use it, via `#[path = "common/<name>.rs"] mod <name>;`:

- `durable_shell.rs` — `DurableShell`, included by all four
  `gpui_file_io_*` binaries.
- `path_prompt.rs` — `assert_no_path_prompt` and `assert_path_prompt`,
  included by the click-save-button, open-dialog and save-dialog binaries.
- `temp_svg.rs` — `TempSvgFile`, included by the open-dialog,
  metadata-round-trip and save-dialog binaries.
- `temp_svg_write.rs`, `temp_svg_read.rs` and `temp_svg_exists.rs` — the
  `write`, `read_to_string` and `exists` operations on `TempSvgFile`,
  each included only by the binaries that perform that operation.

Splitting the helpers this finely means every module is fully used by
every binary that includes it, so the compiler still catches a genuinely
unused helper instead of a blanket `dead_code` expectation hiding it. It
also keeps these helpers out of the shared surface that every GPUI
integration test otherwise compiles. `assert_no_path_prompt(cx, context)`
and `assert_path_prompt(cx, context)` share the underlying
`did_prompt_for_new_path` check, while each step binding supplies its own
failure message via `context`.

See `tests/gpui_file_io_save_dialog.rs` for the fullest worked example,
combining a `DurableShell`, a shared `TempSvgFile`, and multiple export
scenarios, and `tests/features/file_io_*.feature` for the corresponding
specifications.
