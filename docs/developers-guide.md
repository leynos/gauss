# Developer's guide

## GPUI harness behavioural tests

The root package owns the GPUI integration tests under `tests/`. The GPUI
behavioural test pattern uses `rstest-bdd` scenarios with the first-party
`rstest_bdd_harness_gpui::GpuiHarness` to inject a `gpui::TestAppContext`.

The root `Cargo.toml` declares the two supporting development dependencies:

```toml
[dev-dependencies]
rstest-bdd-harness-gpui = "0.6.0-beta3"
serial_test = "3"
```

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

### Stateful file I/O scenarios

The `gpui_file_io_*` scenario binaries cover Save and export dialog
behaviour, where a `Given` step opens a window that a later `When` or `Then`
step must go on to drive. A `gpui::VisualTestContext` cannot cross a step
boundary, so these binaries store a `DurableShell` in scenario state instead:
a `gpui::Entity<Phase0Shell>` paired with the window's `AnyWindowHandle`.
`DurableShell::with_visual_cx` rebuilds a `VisualTestContext` from that
handle for the lifetime of a closure, giving a subsequent step a live
context to act on.

Because step functions cannot thread state through their arguments, each
scenario binary keeps a thread-local state cell instead. Invoke
`crate::scenario_state!(StateType)` once at the binary's crate root with any
`Default`-implementing state type; the macro generates the `STATE` cell,
`with_state` and `reset_state` helpers, the `ScenarioStateCleanup` drop
guard, and the `scenario_state_cleanup` rstest fixture. Every `#[scenario]`
function must accept
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
handle. File I/O must not run while the `with_state` borrow is held; the
save-dialog binary therefore stores an `Option<Rc<TempSvgFile>>` and clones
the `Rc` out of the cell before reading the file, releasing the `RefCell`
borrow first.

`tests/common/file_io.rs` is deliberately not a submodule of `common`. Only
the four `gpui_file_io_*` binaries include it, via
`#[path = "common/file_io.rs"] mod file_io;`, so these helpers stay out of
the shared surface that every GPUI integration test otherwise compiles.
Within it, `assert_no_path_prompt(cx, context)` and
`assert_path_prompt(cx, context)` share the underlying
`did_prompt_for_new_path` check, while each step binding supplies its own
failure message via `context`.

See `tests/gpui_file_io_save_dialog.rs` for the fullest worked example,
combining a `DurableShell`, a shared `TempSvgFile`, and multiple export
scenarios, and `tests/features/file_io_*.feature` for the corresponding
specifications.
