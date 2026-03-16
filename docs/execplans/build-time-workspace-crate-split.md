# Split Gauss into core, Scalable Vector Graphics (SVG), and app workspace crates

This Execution Plan (ExecPlan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: COMPLETE (2026-03-15)

No `PLANS.md` exists in this repository.

## Purpose / big picture

Gauss currently keeps the pure editor model, the SVG import/export layer, and
the GPUI (GPU-accelerated UI) application in one package even though
[src/lib.rs](../../src/lib.rs) already describes them as separate layers. A
single-package layout means model-only and SVG-only edits still pay for the
heavy `gpui` and `gpui-component` dependency tree whenever developers run the
default workspace build and test commands.

After this plan is implemented, the repository will contain a Cargo workspace
with a pure `gauss-core` library for model logic, a `gauss-svg` library for
import/export logic, and an app crate that keeps the current user-visible
binary and GPUI shell. Success is observable when:

- `cargo test -p gauss-core` can run model work without compiling GPUI crates;
- `cargo test -p gauss-svg` can validate import/export without compiling the
  GPUI shell;
- `cargo build -p gauss` still produces the current `gauss` desktop
  application; and
- `make fmt`, `make markdownlint`, `make nixie`, `make check-fmt`,
  `make lint`, `make test`, and `git diff --check` all pass after the workspace
  refactor.

## Constraints

- Keep the user-visible application binary named `gauss`.
- Preserve the current public module split described in:
  - `src/lib.rs`,
  - `src/model/mod.rs`,
  - `src/svg/mod.rs`,
  - `src/ui/mod.rs`.
- Keep GPUI, GPUI Component, and AccessKit wiring in the app crate. Do not
  pull those dependencies into `gauss-core` or `gauss-svg`.
- Keep `gauss-core` GPUI-independent. This is already a documented architectural
  requirement in `docs/using-gpui-and-gpui-component.md`.
- Keep `gauss-svg` dependent on `gauss-core`, not the other way around.
- Retain existing behaviour for import/export, accessibility, keybindings,
  history, and tool flows. This refactor changes package boundaries, not user
  semantics.
- Keep `crates/test_support` private to the workspace and re-point it at the
  smallest useful crate instead of the full app package.
- Do not introduce new third-party runtime dependencies.
- The user approved implementation on 2026-03-15.

## Tolerances (exception triggers)

- Scope tolerance: if the refactor requires more than 28 files or 1,500 net
  lines of code, stop and split the work into sub-plans before proceeding.
- API tolerance: if preserving the `gauss` binary name requires a crates.io
  package rename or public import-path changes outside the workspace, stop and
  decide whether compatibility shims are required.
- Cycle tolerance: if `gauss-core`, `gauss-svg`, `gauss`, and
  `crates/test_support` cannot be arranged without circular dependencies, stop
  and redesign the helper-crate boundary before continuing.
- Tooling tolerance: if `whitaker`, `cargo`, or `mdformat-all` behave
  differently under a multi-crate workspace in a way that changes the existing
  `make` contract, stop and document the required make-target updates before
  making partial code changes.
- Validation tolerance: if full workspace gates fail for unrelated pre-existing
  reasons, capture the failing output with tee logs and keep the refactor
  partially complete rather than forcing unrelated repairs into this change.

## Risks

- Risk: the model layer may depend on app-owned helpers through hidden imports.
  Mitigation: inventory `crate::ui`, `crate::svg`, and `crate::test_helpers`
  references before moving files, then replace them with crate-local helpers or
  narrower cross-crate APIs.

- Risk: `crates/test_support` may keep the full app crate in the dependency
  graph, weakening the split. Mitigation: re-point it at `gauss-core` and move
  deterministic fixture helpers out of the current root crate where necessary.

- Risk: import paths across dozens of tests could create a long tail of
  breakage. Mitigation: land the workspace boundary first, then perform import
  rewrites and test migration in ordered commits.

- Risk: `cargo test --workspace` may still compile the full app crate when
  workspace-level gates run. Mitigation: accept that clean full builds remain
  expensive and optimize for targeted `-p gauss-core` and `-p gauss-svg`
  workflows rather than promising a dramatic clean-build collapse.

## Progress

- [x] (2026-03-14) Confirmed branch context: `build-time`.
- [x] (2026-03-14) Verified the current root `Cargo.toml` declares a workspace
  with only `crates/test_support` as a member.
- [x] (2026-03-14) Verified `src/lib.rs` already describes the `model`, `svg`,
  and `ui` seams.
- [x] (2026-03-14) Confirmed `gpui` usage is concentrated in `src/ui/**` and
  `src/main.rs`, making a top-level crate split plausible.
- [x] (2026-03-14) Measured a cold `cargo build` and captured the finding that
  the largest compile units are upstream GUI dependencies, not Gauss itself.
- [x] (2026-03-14) Drafted this ExecPlan.
- [x] (2026-03-15) Created `crates/gauss-core` and moved `src/model/**` plus
  `src/test_helpers.rs` into it.
- [x] (2026-03-15) Created `crates/gauss-svg` and moved `src/svg/**` into it.
- [x] (2026-03-15) Rewired the root `gauss` package to re-export `model` and
  `svg` while keeping the binary name and GPUI shell unchanged.
- [x] (2026-03-15) Re-pointed `crates/test_support` at `gauss-core`.
- [x] (2026-03-15) Replayed `make fmt`, `make markdownlint`, `make nixie`,
  `make check-fmt`, `make lint`, `make test`, and `git diff --check` with tee
  logs.

## Surprises and discoveries

- The existing code comments already describe the right split: `src/lib.rs`
  calls `model` pure logic, `svg` persistence, and `ui` GPUI-specific wiring.
- `crates/test_support` is the only existing workspace member, which means the
  repository already has just enough Cargo workspace structure to absorb more
  crates without inventing a new top-level convention.
- `src/main.rs` is thin and mostly window bootstrapping, so the eventual app
  crate can stay small if most logic moves into libraries.
- The measured cold build cost is dominated by `ash`, `naga`, `gpui`, and
  `gpui-component`; `gauss` itself is comparatively cheap. This means the split
  improves selective builds more than it improves the worst-case clean build.
- Keeping the root package as the app crate let the existing `gauss::model`
  and `gauss::svg` import paths survive as thin re-exports, which sharply
  reduced the UI rewrite surface.
- Moving code across crate boundaries surfaced a few implicit app-to-core
  contracts, notably the selection-overlay bounds helper and `Action`'s
  `#[non_exhaustive]` matches in the app crate.

## Decision Log

- 2026-03-14: Plan around three top-level product crates plus the existing test
  helper crate rather than creating many micro-crates. Rationale: the current
  seams are stable, but finer-grained model sub-crates would add dependency
  management overhead without isolating the real compile-time hotspot.

- 2026-03-14: Keep the application package named `gauss`. Rationale: this
  avoids unnecessary churn in developer commands, CI scripts, and user-facing
  binary naming.

- 2026-03-14: Keep accessibility code in the app crate instead of creating a
  separate `gauss-a11y` crate. Rationale: current AccessKit logic is tightly
  coupled to `Phase0Shell`, and the timing evidence does not justify an
  isolated accessibility refactor.

## Context and orientation

The current repository structure is:

- `src/model/**` for the editor model and command/history/tool logic.
- `src/svg/**` for SVG import, export, and metadata.
- `src/ui/**` plus `src/main.rs` for GPUI app wiring and desktop shell
  behaviour.
- `crates/test_support/**` for workspace-private fixtures.

The intended future structure is:

- `crates/gauss-core/` for `model` plus deterministic non-UI test helpers.
- `crates/gauss-svg/` for SVG import/export built on `gauss-core`.
- the root package (the app crate) for `ui` and the binary entrypoint.
- `crates/test_support/` updated to depend on `gauss-core`.

Relevant files to read before implementation:

- `Cargo.toml`
- `src/lib.rs`
- `src/main.rs`
- `src/model/mod.rs`
- `src/svg/mod.rs`
- `src/ui/mod.rs`
- `crates/test_support/Cargo.toml`

## Plan of work

### Stage A: establish the target Cargo workspace shape

Update the root `Cargo.toml` so the workspace declares the new crate members,
shared metadata, and a stable default member set. The root package remained the
app crate rather than introducing a virtual workspace.

Validation gate:

- `cargo metadata --format-version 1` shows the intended members and no
  dependency cycles.

### Stage B: move model code into `gauss-core`

Create `crates/gauss-core/src/lib.rs`, move `src/model/**` into that crate, and
re-home any pure helper code currently in `src/test_helpers.rs` that is needed
by model tests or downstream crates. Update imports so model code is fully
crate-local and no longer references the old root package.

Validation gate:

- `cargo test -p gauss-core` passes.
- `rg -n "use gpui|gpui::" crates/gauss-core` returns no hits.

### Stage C: move SVG code into `gauss-svg`

Create `crates/gauss-svg/src/lib.rs`, move `src/svg/**` into that crate, and
update imports to depend on `gauss-core`. Keep all SVG metadata and resource
handling inside this crate.

Validation gate:

- `cargo test -p gauss-svg` passes.
- `cargo tree -p gauss-svg` shows a dependency on `gauss-core` and no
  dependency on the app crate.

### Stage D: rewire the app crate and helper crate

Update the app crate to depend on `gauss-core` and `gauss-svg`, then move or
replace any `crate::model` and `crate::svg` imports with the new crate paths.
Update `crates/test_support/Cargo.toml` and its source files to depend on
`gauss-core` instead of the current root package.

Validation gate:

- `cargo build -p gauss` still produces the desktop application.
- `cargo test -p test_support` passes.

### Stage E: synchronize docs and developer workflows

Update architecture and developer documentation so the new crate layout is
described accurately. If commands or examples in `docs/` assume a single crate,
rewrite them to use the new workspace structure and targeted package commands.

Validation gate:

- `make fmt`
- `make markdownlint`
- `make nixie`
- `make check-fmt`
- `make lint`
- `make test`
- `git diff --check`

## Outcomes and retrospective

The landed crate graph is `gauss` -> (`gauss-core`, `gauss-svg`) and
`gauss-svg` -> `gauss-core`, with `test_support` depending only on
`gauss-core`. Focused validation passes for:

- `cargo metadata --format-version 1`
- `cargo test -p gauss-core`
- `cargo test -p gauss-svg`
- `cargo build -p gauss`
- `cargo test -p test_support`

The full workspace gate replay also passed:

- `make fmt`
- `make markdownlint`
- `make nixie`
- `make check-fmt`
- `make lint`
- `make test`
- `git diff --check`

The split delivered the expected selective-build improvement:
`cargo test -p gauss-core` and `cargo test -p gauss-svg` now validate their
respective layers without compiling the GPUI shell, while the root `gauss`
package keeps the existing binary name and public `gauss::model` / `gauss::svg`
import paths via re-exports.
