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
scenario selects it through the canonical path so the macro supplies the GPUI
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
