# Migrate the Gauss workspace to rstest-bdd v0.6.0

| Item                | Value                                                             |
| ------------------- | ----------------------------------------------------------------- |
| Branch              | `adopt-rstest-bdd-v0-6-0`                                         |
| Base commit         | `c8c4a89` ("Bump the github-actions group with 6 updates (#178)") |
| Baseline dependency | `rstest-bdd` family at `0.6.0-beta3`                              |
| Target              | Published `0.6.0` release from crates.io                          |

## Purpose

Move every existing `rstest-bdd` consumer in this workspace from the
`0.6.0-beta3` prerelease to the published `0.6.0` release, fix the real defects
the upgrade exposes, and preserve existing behaviour and test coverage. This is
a migration of existing consumers; no new BDD suites are introduced.

## Big picture / constraints

- Preserve behaviour and coverage. Do not weaken assertions, add ignores,
  or disable compile-time validation to make the build pass.
- Keep the diff focused; avoid unrelated dependency churn.
- Follow the repository's normal version-constraint convention (caret
  requirements; see `AGENTS.md`).
- The workspace already uses the canonical
  `harness = rstest_bdd_harness_gpui::GpuiHarness` path, so no harness path
  reshaping is required.
- Necessary MSRV/nightly bumps are authorised, but must be validated and
  recorded rather than assumed.

## 1. Imported documentation provenance

Imported **byte-for-byte** from upstream `leynos/rstest-bdd`.

| Item                   | Value                                                                   |
| ---------------------- | ----------------------------------------------------------------------- |
| Upstream repository    | `https://github.com/leynos/rstest-bdd`                                  |
| Tag                    | `v0.6.0`                                                                |
| Tag resolves to commit | `72fb22635670e456545ca368805ba4c1c9d7bd69`                              |
| Tag type               | Lightweight (resolves directly to the commit; no annotated-object peel) |

The requested tag resolved **exactly** to the commit named in the task
briefing. No substitution or discrepancy was found.

Imported files (both checksums verified after copy):

| Upstream path                    | Destination                                 | sha256                                                             |
| -------------------------------- | ------------------------------------------- | ------------------------------------------------------------------ |
| `docs/users-guide.md`            | `docs/rstest-bdd-users-guide.md`            | `1e9f4d1b6607fdf83df979676f1a682218cd6751381d374cb98b701778b798b2` |
| `docs/v0-6-0-migration-guide.md` | `docs/rstest-bdd-v0-6-0-migration-guide.md` | `6e76c10028f9962134f6158732cf1645ae1b0beba3018bccd3bc3e88b2c38985` |
| `docs/v0-5-0-migration-guide.md` | `docs/rstest-bdd-v0-5-0-migration-guide.md` | `508d5b55cb2025915ced366b917a2327e572a6aa920fd290684b858b7afa2abc` |

**Important finding:** the copies previously in this repository were *not* from
the final release. Their checksums differed from the tag:

| Destination                                 | Previous sha256 (beta3-era) | Superseded by       |
| ------------------------------------------- | --------------------------- | ------------------- |
| `docs/rstest-bdd-users-guide.md`            | `2d1048ff9056b114…`         | `1e9f4d1b6607fdf8…` |
| `docs/rstest-bdd-v0-6-0-migration-guide.md` | `8fcf9611f858c4b6…`         | `6e76c10028f99621…` |
| `docs/rstest-bdd-v0-5-0-migration-guide.md` | `32c655ed4e756fcc…`         | `508d5b55cb202591…` |

The v0.5.0 guide was imported from the same commit because a complete migration
audit needs the intervening guide. Upstream relative links in the imported
guides point at sibling upstream documents (for example `rstest-bdd-design.md`,
`adr-012-…md`) that are not vendored here; they are left intact and
byte-for-byte, per the "keep the imported text byte-for-byte intact"
constraint. Local migration notes live in this ExecPlan, not in the imported
copies.

### Version boundary respected

The v0.6.0 text contains sections labelled **v0.6.1 beta** and **v0.7.0**.
These describe later releases and are explicitly **not** implemented here. In
particular:

- `#[harness_context]` (v0.6.1 beta) is **not** adopted. The v0.6-compatible
  `#[from(rstest_bdd_harness_context)]` spelling is retained.
- The guard-based `StepContext` borrowing model (v0.7.0) is **not**
  adopted. The interim v0.6 thread-local scenario-state pattern is retained
  where it already exists.

## 2. Repository inventory

Single Cargo workspace, `gauss`, with members:

- root package `gauss` (the binary/library, `src/`)
- `crates/gauss-core`
- `crates/gauss-svg`
- `crates/test_support` (test-support crate)

`default-members` excludes `crates/test_support`.

Before the migration, `rstest-bdd` dependencies were declared in three
manifests as **direct** `dev-dependencies` (no workspace inheritance):

| Manifest                       | Declared                                                                     |
| ------------------------------ | ---------------------------------------------------------------------------- |
| `Cargo.toml`                   | `rstest-bdd`, `rstest-bdd-macros`, `rstest-bdd-harness-gpui` @ `0.6.0-beta3` |
| `crates/gauss-core/Cargo.toml` | `rstest-bdd`, `rstest-bdd-macros` @ `0.6.0-beta3`                            |
| `crates/gauss-svg/Cargo.toml`  | `rstest-bdd`, `rstest-bdd-macros` @ `0.6.0-beta3`                            |

No `[patch]`, no Git/path overrides, no package aliases for these crates, no
vendored or submodule copies. One lockfile (`Cargo.lock`). `rstest-bdd-harness`
is **not** a direct dependency and does not need to be: no consumer imports the
base harness API directly, and the first-party GPUI adapter re-exports what
generated code needs.

Consumer surface:

- 591 step functions across the workspace.
- 85 uses of `harness = rstest_bdd_harness_gpui::GpuiHarness`.
- No `tokio` dependency, no `runtime = "..."` scenarios form.
- No `attributes = (...)` macro arguments.
- No custom `HarnessAdapter` implementations.
- No direct `insert_value`, `record_bypassed_steps`, `BypassedScenario`,
  or `RustStepIndexResult` callers.
- No JSON/JUnit reporting or LSP integrations.

Toolchain: `rust-toolchain.toml` pins `1.92.0` (stable) with `rustfmt`,
`clippy`, `rust-analyzer`. **Stable channel, not nightly.** No `rust-version`
(MSRV) is declared in any manifest.

## 3. Version alignment

Declared requirements moved from `0.6.0-beta3` to `0.6.0` (caret semantics, per
repository convention) in the three manifests above.

Lockfile updated with **targeted** updates, not an unrestricted refresh:

```sh
cargo update -p rstest-bdd --precise 0.6.0
cargo update -p rstest-bdd-macros --precise 0.6.0
cargo update -p rstest-bdd-harness-gpui --precise 0.6.0
```

Resolved package set after migration (single version each; no duplicates):

`rstest-bdd`, `rstest-bdd-harness`, `rstest-bdd-harness-gpui`,
`rstest-bdd-macros`, `rstest-bdd-patterns`, `rstest-bdd-policy` — all `0.6.0`.
`rstest` remains `0.27.0`.

Transitive additions, all traced to the published 0.6.0 manifests:

| Added/changed                                                                  | Pulled by                                                                  |
| ------------------------------------------------------------------------------ | -------------------------------------------------------------------------- |
| `ctor` 0.4.3 → 1.0.13, plus `link-section`, `linktime-proc-macro`              | `rstest-bdd` 0.6.0 (`ctor = "1.0.13"`)                                     |
| `proc-macro-error3`, `proc-macro-error-attr3` (replace `proc-macro-error` 1.x) | `rstest-bdd-macros` 0.6.0                                                  |
| `convert_case` 0.6.0 → 0.12.0                                                  | `rstest-bdd-macros` 0.6.0                                                  |
| `cargo_metadata` 0.18.1 → 0.23.1                                               | `rstest-bdd-harness` 0.6.0                                                 |
| `hashbrown` 0.17.1, `derive_more` 2.1.1, `typed-builder` 0.23.2                | `rstest-bdd` 0.6.0                                                         |
| `tokio-macros` 2.6.1                                                           | pre-existing `tokio` 1.48.0 in the graph (via GPUI); newly enabled feature |

No prerelease versions remain in the lockfile for the `rstest-bdd` family.

### MSRV / toolchain

Upstream `0.6.0` declares `rust-version = "1.88"` (the `gherkin 0.16` parser
requires it). This repository declares **no** `rust-version` and pins the
toolchain at `1.92.0`, which is already above the upstream floor. **No MSRV
bump and no nightly pin change is required or made.** The repository is not a
nightly repository, so no rustfmt/nightly pin work applies.

## 4. Behavioural changes applied

### 4.1 Step-return alias classification (the significant one)

v0.6.0 classifies unhinted non-unit step returns by their **concrete type** at
runtime (`rstest_bdd::step_return`), rather than only recognising syntactically
spelled `Result` paths at macro-expansion time. Under beta3, an unhinted *local
alias* of `Result<T, E>` was treated as an opaque value: the whole `Result` was
boxed as a payload and any `Err` was silently discarded — the "former false
green" the migration guide describes.

This repository had **165 step functions returning `TestSupportResult<()>`**, a
local alias of `Result<(), TestSupportError>`. Compiling against 0.6.0 surfaces
the defects those aliases were hiding.

The remaining 327 step functions spell `Result<(), TestSupportError>`
explicitly and were already classified as fallible; 99 return unit.

`TestSupportError` derives `thiserror::Error`, so it already implements
`Display`; the migration guide's `Display` requirement is satisfied with no
change.

### 4.2 Defect exposed and fixed in the accessibility scenario

`adding_a_shape_emits_one_inserted_accessibility_node_update`

- **Test:** `gauss::a11y_service_bdd` /
  `adding_a_shape_emits_one_inserted_accessibility_node_update`
- **Step:** `inserted_node_list_contains_appended_shape_id` in
  `tests/a11y_service_bdd.rs`
- **Symptom under 0.6.0:** the scenario now fails, reporting that the
  published inserted-node IDs `[12884901889]` (that is `0x3_0000_0001`) do not
  contain the expected `0x2_0000_0001`.
- **Root cause (test setup, not production code):** the step asserted
  against the raw seed constant `APPENDED_SHAPE_ID = 0x2_0000_0001`, but
  `ShapeId::from_accesskit_node_id` decodes through
  `slotmap::KeyData::from_ffi`, which keeps only the **low 32 bits** as the
  index and forces the version field **odd**:

  ```rust
  // slotmap 1.1.1
  pub const fn from_ffi(value: u64) -> Self {
      let idx = value & 0xffff_ffff;
      let version = (value >> 32) | 1; // Ensure version is odd.
      Self::new(idx as u32, version as u32)
  }
  ```

  The seed `0x2_0000_0001` decodes to index `1`, version `3`, and therefore
  re-encodes to `0x3_0000_0001`. The assertion could **never** hold; the step
  always returned `Err`, which beta3 swallowed.
- **Fix:** derive the expected value by round-tripping the seed through
  `ShapeId`, matching the pattern the passing unit tests in
  `src/ui/phase0_shell/a11y_service/tests/mod.rs` already use
  (`shape_id(0x2_0000_0002).to_accesskit_node_id()`).
- **No production code changed.** The service was correct; the test's
  expectation was not. The false green was *not* restored.

This is exactly the failure mode the migration guide warns about, and the fix
follows its guidance: fix the incorrect setup rather than restore the old
behaviour.

### 4.3 Regression coverage for the aliased-error contract

The defect above was exposed by an existing scenario, but nothing in the suite
pinned the *alias classification* contract itself. Adding such a test is the
difference between "this migration happened to be caught" and "this migration
cannot regress silently".

`crates/gauss-core/tests/result_alias_bdd.rs` (with
`crates/gauss-core/tests/features/result_alias_classification.feature`) pins
both directions of the contract using step returns written as the
`test_support::TestSupportResult` alias:

1. a step that returns `Ok(())` through the alias still injects its
   value and the scenario passes;
2. a step that returns `Err` through the alias fails the scenario, run
   under `#[should_panic(expected = "Step failed at index")]`.

Direction 2 is the regression guard: under beta3's classification the `Err` was
boxed as an opaque payload, the step appeared to succeed, the scenario passed,
and the `#[should_panic]` would therefore have failed. The test asserts the
*converse* of the old behaviour, so it cannot pass by accident if the defect
returns.

This target lives in `crates/gauss-core/tests/`. The integration-test inventory
gate (`scripts/check_integration_test_inventory.py`) is scoped to the **root
`gauss` package only** — it selects the package whose `manifest_path` is the
root `Cargo.toml` — so adding a `gauss-core` target does not perturb the
documented `total=52` counts in `tests/CONSOLIDATION_MAP.md` or the three
execplans that mirror them.

`#[should_panic]` was verified to survive to the generated test: the scenario
macro re-emits user attributes (`codegen/scenario/mod.rs`, which appends
`#(#attrs)*` after the generated `#[rstest]` and harness attributes), `rstest`
passes `#[should_panic]` through, and the observed panic message is the
framework's own
`Step failed at index 1: When a step returns Err through the result alias …`.

### 4.4 Documentation drift corrected

Two live documents and one source comment asserted beta-era behaviour that
0.6.0 no longer exhibits:

| Location                      | Was                                                                                                  | Now                                                                                                                          |
| ----------------------------- | ---------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| `docs/developers-guide.md`    | `rstest-bdd-harness-gpui = "0.6.0-beta3"`                                                            | `"0.6.0"`                                                                                                                    |
| `docs/developers-guide.md`    | "rstest-bdd 0.6.0-beta3's injected …"                                                                | "rstest-bdd 0.6.0's injected …"                                                                                              |
| `tests/gpui_draw_undo_bdd.rs` | claimed the step macro classifies return types syntactically and cannot see through a `Result` alias | records that 0.6.0 resolves the alias by concrete type, and that the spelled-out signatures are retained only to avoid churn |

The `tests/gpui_draw_undo_bdd.rs` header also carried two TODOs. The alias TODO
(#573) is **resolved by 0.6.0** and is now covered by the regression test
above. The fallible-scenario-return TODO (#574) is also **resolved**: 0.6.0
gained `adapt_fallible_gpui_boundary`, which rewrites a fallible scenario body
for the GPUI unit-returning test boundary and panics on `Err`. It was verified
empirically — converting `draw_undo_scenario` to return
`Result<(), TestSupportError>` compiles and passes under `-D warnings` with no
`unused_must_use`. The one rough edge is that the macro erases the declared
return type, so the trailing `Ok(())` requires an explicit
`Ok::<(), TestSupportError>(())`. The scenario was left unit-returning because
the turbofish is no clearer than the existing form, and the header now says so
rather than repeating a claim that is no longer true.

### 4.5 Items audited and found inapplicable

| Migration item                                      | Status in this repository                                                                                    |
| --------------------------------------------------- | ------------------------------------------------------------------------------------------------------------ |
| Underscore-prefixed implicit fixtures               | No implicit underscore-named step/scenario parameters. `#[from(...)]` names remain exact. Nothing to change. |
| `runtime = "tokio-current-thread"` → `TokioHarness` | Not used; no `tokio` dependency.                                                                             |
| Custom `HarnessAdapter` → `HarnessResult<T>`        | No custom harness implementations.                                                                           |
| `insert_value` → `InsertOutcome`                    | No direct callers.                                                                                           |
| `record_bypassed_steps` → `BypassedScenario`        | No direct callers.                                                                                           |
| `RustStepIndexResult` / LSP entry points            | Not used.                                                                                                    |
| `find_feature_files` error propagation              | Not used.                                                                                                    |
| JUnit `classname` / JSON `feature_path`             | No reporting integrations; no snapshots to re-baseline.                                                      |
| Guard-based borrowing (v0.7.0)                      | Deliberately not adopted; version boundary respected.                                                        |
| `#[harness_context]` marker (v0.6.1 beta)           | Deliberately not adopted; `#[from(rstest_bdd_harness_context)]` retained.                                    |
| Feature-file rebuild invalidation                   | Beneficial; no `touch` workarounds present to remove.                                                        |

## 5. Validation

### 5.1 Gates run

Run sequentially from the worktree root, each with output teed to a log so the
real exit status is preserved:

```sh
make check-fmt
make markdownlint
make nixie
make lint
make test
```

`make test` runs the full nextest suite, then the doctest equivalent
`cargo test --workspace --doc --all-features` with `RUSTFLAGS="-D warnings"` and
`RUSTDOCFLAGS="--cfg docsrs -D warnings"`.

### 5.2 Inventory reconciliation

Counts were taken with a single consistent scope (all `*.rs` and `*.feature`
under `tests/`, `crates/`, `src/` and `docs/`) so the baseline and migrated
figures are directly comparable. The baseline column is the base commit
`c8c4a89`; the migrated column includes the uncommitted regression test and its
feature file.

| Measure                                          | Baseline `c8c4a89` | Migrated | Difference                            |
| ------------------------------------------------ | ------------------ | -------- | ------------------------------------- |
| Feature files                                    | 47                 | 48       | +1 (the new regression feature)       |
| `#[scenario]` attributes                         | 182                | 184      | +2 (the two new regression scenarios) |
| Step functions (`#[given]`/`#[when]`/`#[then]`)  | 598                | 602      | +4 (the four new regression steps)    |
| `harness = rstest_bdd_harness_gpui::GpuiHarness` | 85                 | 85       | 0                                     |

**Every difference is accounted for by the new regression coverage.** The
migration itself changes no scenario, step, or feature file: the committed
migration commit `2023cfb` touches only the three manifests, the lockfile, and
`tests/a11y_service_bdd.rs`.

An earlier pair of counts appeared to show the working tree *losing* scenarios
relative to `HEAD`. That was an artefact of the grep scope: one tracked
scenario lives in `docs/execplans/0-1-2-command-dispatch/snippets`, which the
first working-tree scan omitted. The table above uses the corrected, consistent
scope.

`make check-fmt` cannot be affected by the regression-test comment edits: the
repository has no `rustfmt.toml`, so `wrap_comments` is off and comment reflow
is not a formatting concern.

### 5.3 Baseline equivalence

Because the baseline worktree under `/tmp` was reclaimed before the final gate
run, baseline test figures are cited from the earlier verified baseline runs
recorded during this task rather than re-run. The baseline was measured in a
pristine detached worktree at `c8c4a89` with a separate `CARGO_TARGET_DIR` so
that baseline and migrated runs could not contaminate each other.

## 6. Results

### 6.1 Gate results

All five gates pass on the migrated tree. Each was run sequentially from the
worktree root with output teed to a log under `/tmp`, so the real exit status
is preserved.

| Gate                | Exit | Evidence                                                                                                                                       |
| ------------------- | ---- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| `make check-fmt`    | 0    | `cargo fmt --all -- --check` clean; mdtablefix `74 files left unchanged`                                                                       |
| `make markdownlint` | 0    | `Summary: 0 error(s)` over 74 files; spelling gate exit 0; inventory checker `23 passed`; inventory markers unchanged (`total: 52`)            |
| `make nixie`        | 0    | All Mermaid diagrams validated successfully                                                                                                    |
| `make lint`         | 0    | `cargo doc` clean; `cargo clippy --workspace --all-targets --all-features -- -D warnings` clean; Whitaker (Dylint, `nightly-2026-05-28`) clean |
| `make test`         | 0    | nextest `954 tests run: 954 passed, 1 skipped` across 74 binaries; doctests green                                                              |

`make check-fmt` needed one fix during validation: the ExecPlan itself was not
mdtablefix-normalised. That is a fault in this document, not in the migration,
and was repaired before the final run.

### 6.2 Test counts, baseline versus migrated

| Measure             | Baseline `c8c4a89` | Migrated | Difference                                  |
| ------------------- | ------------------ | -------- | ------------------------------------------- |
| Tests run (nextest) | 952                | 954      | +2                                          |
| Passed              | 952                | 954      | +2                                          |
| Failed              | 0                  | 0        | 0                                           |
| Skipped             | 1                  | 1        | 0                                           |
| Test binaries       | 73                 | 74       | +1 (the new `gauss-core::result_alias_bdd`) |
| Doctests (passed)   | 64                 | 64       | 0                                           |
| Doctests (ignored)  | 3                  | 3        | 0                                           |

The single skipped test is present in both columns, so the skip set is
unchanged. The two added tests are exactly the new aliased-error regression
pair; the added binary is the one new integration-test target that carries
them. No baseline test was renamed, removed, or disabled.

### 6.3 Proof-points

The scenarios this migration turns on, all passing:

```text
PASS [ 0.075s] (156/954) gauss::a11y_service_bdd
     adding_a_shape_emits_one_inserted_accessibility_node_update
PASS [ 0.106s] (182/954) gauss::gpui_draw_undo_bdd draw_undo_scenario
PASS [ 0.009s] (827/954) gauss-core::result_alias_bdd
     step_returning_ok_through_result_alias_succeeds
PASS [ 0.011s] (828/954) gauss-core::result_alias_bdd
     step_returning_err_through_result_alias_fails_scenario
```

### 6.4 False-green proof

The most important single result of this migration is differential, and was
captured before the final gate sweep:

- At baseline `c8c4a89` (beta3, unfixed test), the a11y scenario
  **passes** — that is the false green, where the step's `Err` was silently
  discarded.
- Against 0.6.0 with the test still unfixed, the same scenario
  **fails**, reporting
  `expected inserted node IDs [12884901889] to contain 0x200000001`.
- With the setup defect fixed, 0.6.0 passes again.

Three states, two of them distinguishable only because 0.6.0 classifies the
return by its concrete type. That sequence is what establishes the defect was
real and that the fix addressed the cause rather than the symptom.

## 7. Outstanding items and risks

### 7.1 Nothing blocking

Every gate in section 6.1 exits 0 on the migrated tree. No check is claimed as
passed that was not run, no warning is suppressed, and no validation was
relaxed to make the build compile.

### 7.2 Accepted, disclosed items

- **Baseline figures were not re-measured in the final sweep.** The
  baseline worktree under `/tmp` was reclaimed by the environment before the
  final gate run. The baseline column in section 6.2 cites the earlier verified
  baseline runs from this task. Those runs were made in a pristine detached
  worktree at `c8c4a89` with a separate `CARGO_TARGET_DIR`. If the reader wants
  fresh parity, re-running the same commands at `c8c4a89` reproduces them.
- **`ropey 2.0.0-beta.1` remains in the lockfile.** Pre-existing at
  `c8c4a89`, unrelated to `rstest-bdd`, untouched here. Recorded so the
  remaining prerelease is not mistaken for migration residue.
- **Two open Dependabot PRs are superseded, not closed.** PR #179 and
  PR #180 propose the lockfile-only bump this branch performs properly. Closing
  them is a maintainer decision and was deliberately not taken.
- **`proc-macro-error2 v2.0.1` future-incompatibility warning.** Emitted
  by cargo during `make lint`, from a transitive dependency outside the
  `rstest-bdd` family. It is a warning, not an error, it is not suppressed, and
  it does not affect the gate's exit status. It is not caused by this migration
  and is out of scope to fix here.
- **No guard-based borrowing adopted.** Upstream's v0.6.0 notes describe
  guard-based `StepContext` borrowing as a *future* (v0.7.0) direction. This
  migration deliberately stops at the v0.6 contract, so the existing
  thread-local scenario-state pattern is retained unchanged.

### 7.3 Deliberately not done

- No MSRV bump and no nightly pin change. Upstream `0.6.0` requires Rust
  1.88; this repository has no `rust-version` and pins `1.92.0`, already above
  the floor. Neither change was necessary, so neither was made.
- No compatibility facades, dual-version branches, or wrappers for the
  pre-1.0 API.
- No GPUI, Bevy, or custom harness introduced.
- No v0.6.1-beta `#[harness_context]` and no v0.7.0 guard-based
  borrowing.
- Not merged, and no release published.

## Decision log

- **Branch name reuse.** The name `adopt-rstest-bdd-v0-6-0` was already
  occupied by a stale local branch holding PR #95's head (`ec4acbc`, a closed
  beta1-era planning branch). To satisfy the rename request without destroying
  history, that branch was preserved as `archive/pr95-adopt-rstest-bdd-v0-6-0`
  before the session branch took the name. `origin/adopt-rstest-bdd-v0-6-0`
  does not exist (PR #95's branch was deleted on close), so tracking is
  configured to that exact remote ref and the push will establish it.
- **Documentation imported from the commit, not the tag ref.** Extraction
  used `git show 72fb226…:path` so the immutable commit, not a movable ref, is
  the source of truth.
- **Open Dependabot PRs superseded, not duplicated.** Before starting,
  the existing-work check found PR #179 ("Bump rstest-bdd-macros from
  0.6.0-beta3 to 0.6.0") and PR #180 ("Bump rstest-bdd from 0.6.0-beta3 to
  0.6.0"), both open, both **lockfile-only** diffs. This migration performs the
  same dependency bump *and* fixes the defect the bump exposes, so it
  supersedes both. Nothing was closed or commented on them as part of this
  work; that is a maintainer decision. PR #95 (the stale
  `adopt-rstest-bdd-v0-6-0` beta1 planning branch) is CLOSED, and PR #113
  (`adopt-rstest-bdd-v0-6-0-beta3`) is MERGED, so no branch-name collision was
  introduced.
- **`ropey 2.0.0-beta.1` is pre-existing, not migration churn.** The
  lockfile contains exactly one remaining prerelease. It is present identically
  at the base commit `c8c4a89`, is untouched by the migration commit, and is
  unrelated to the `rstest-bdd` family. It is therefore out of scope and left
  alone.
- **Regression test locale comment corrected during review.** The
  `#[should_panic]` comment initially described the expected message as the
  `en-GB` rendering. The published `rstest-bdd-0.6.0` loader pins `en-US` and
  never consults the environment, so the comment was factually wrong even
  though the two templates are byte-identical and the assertion was unaffected.
  Corrected to name `en-US` and to record that the message is not
  locale-configurable.
