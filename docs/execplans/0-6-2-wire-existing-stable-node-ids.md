# Wire existing stable node IDs into AccessKit Chrome semantics (0.6.2)

This Execution Plan (ExecPlan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: COMPLETE (2026-03-02)

No `PLANS.md` exists in this repository.

## Purpose / big picture

Roadmap item `0.6.2` requires wiring pre-defined stable node IDs from
`src/ui/phase0_shell/accessibility.rs` into the AccessKit tree and exposing
window-chrome roles and labels in that tree.

After this milestone is implemented, accessibility consumers should observe a
stable, deterministic chrome subtree with correct semantics across renders,
matching the architectural intent in `docs/gauss-architecture-design.md` §11.1
and §20.

Success is observable when:

- AccessKit chrome nodes use the pre-defined node IDs in
  `src/ui/phase0_shell/accessibility.rs`.
- Window chrome nodes expose clear roles and labels (and documented shortcut
  hints) in initial and incremental updates.
- Unit tests (`rstest`), behaviour-driven development (BDD) tests
  (`rstest-bdd` v0.5.0), and GPU-accelerated UI (GPUI) tests cover happy paths,
  unhappy paths, and key edge cases.
- `docs/gauss-architecture-design.md` records design decisions taken in `0.6.2`.
- `docs/users-guide.md` reflects any user-visible accessibility behaviour
  change.
- `docs/roadmap.md` marks `0.6.2` done only after implementation and green
  required gates.
- `make check-fmt`, `make lint`, and `make test` succeed with tee-logged
  evidence.

## Constraints

- Scope is strictly roadmap item `0.6.2`:
  - Connect pre-defined IDs in `accessibility.rs` to AccessKit.
  - Expose roles and labels for window chrome.
- Preserve behaviour delivered by `0.6.1`:
  deterministic snapshots, incremental updates, and focus fallback handling.
- Keep stable ID ownership centralized in
  `src/ui/phase0_shell/accessibility.rs`; avoid duplicate ID literals in
  unrelated modules.
- Keep `A11yService` architecture boundaries intact: tree projection logic stays
  in `src/ui/phase0_shell/a11y_service/`, while UI interaction wiring remains
  in shell/chrome modules.
- Test coverage must include:
  - unit tests with `rstest`,
  - behaviour tests with `rstest-bdd` v0.5.0,
  - GPUI integration tests using `#[gpui::test]` and test contexts.
- Do not mark roadmap `0.6.2` done until code, docs, and required gates are
  complete.
- Follow repository guidance from
  - `docs/gauss-architecture-design.md`,
  - `docs/using-gpui-and-gpui-component.md`,
  - `docs/accesskit-based-accessibility-in-gpui.md`,
  - `docs/rust-testing-with-rstest-fixtures.md`,
  - `docs/rust-doctest-dry-guide.md`,
  - `docs/reliable-testing-in-rust-via-dependency-injection.md`,
  - `docs/rstest-bdd-users-guide.md`.

## Tolerances (exception triggers)

- Scope tolerance: if implementation requires new platform adapter crates or
  platform-specific runtime integration beyond existing `accesskit` usage,
  pause and re-scope (that work belongs to later milestones).
- Blast-radius tolerance: if >12 files or >500 net LOC are required for `0.6.2`,
  pause and decompose before continuing.
- Semantics tolerance: if AccessKit role/state APIs in the pinned version cannot
  express required chrome semantics without breaking compatibility, pause, and
  document fallback options.
- Test tolerance: if test changes require broad fixture redesign outside
  accessibility suites, pause, and split into a follow-up work item.
- Gate tolerance: if `make check-fmt`, `make lint`, or `make test` fail due to
  unrelated pre-existing issues, record evidence and keep completion status
  partial rather than marking roadmap work done.

## Risks

- Risk: chrome semantics drift between UI text in `chrome.rs` and AccessKit
  labels in `accessibility.rs`. Mitigation: make `accessibility.rs` the single
  source for a11y label metadata consumed by tree construction and, where
  appropriate, shared UI text.

- Risk: maximize/fullscreen semantics may require toggle-style role/state that
  differs from current `Role::Button` mapping. Mitigation: add explicit tests
  for role/label/state expectations and document final mapping decision in
  `docs/gauss-architecture-design.md`.

- Risk: roadmap/documentation inconsistency if one section is updated and
  another remains stale. Mitigation: include roadmap checkbox, roadmap
  accessibility summary block, and users guide in one doc update pass before
  final gate run.

- Risk: false confidence from existence-only node-ID assertions.
  Mitigation: extend tests to assert role/label semantics, not only inserted ID
  presence.

## Progress

- [x] (2026-03-02) Confirmed branch context:
  `0-6-2-wire-existing-stable-node-ids`.
- [x] (2026-03-02) Loaded required skills: `execplans`, `grepai`, `leta`
  (plus `rust-router` for Rust-facing decisions).
- [x] (2026-03-02) Ran direct `grepai` and `leta` discovery over accessibility
  code paths and test surfaces.
- [x] (2026-03-02) Used an agent team for parallel discovery and synthesized
  findings through context pack `pk_nbpidj6d`:
  - Reimu (roadmap/architecture anchors),
  - Marisa (leta symbol/test map),
  - Axel (grepai-first implementation gap analysis).
- [x] (2026-03-02) Drafted this ExecPlan document.
- [x] (2026-03-02) Implemented `0.6.2` code changes:
  canonical chrome semantics in `accessibility.rs`, tree wiring in
  `tree_builder.rs`, and GPUI payload access for semantic assertions.
- [x] (2026-03-02) Added/adjusted unit, BDD, and GPUI tests for `0.6.2`
  semantics, including restore-label edge coverage for maximized windows.
- [x] (2026-03-02) Updated architecture and user documentation.
- [x] (2026-03-02) Ran required gates with tee logs:
  `/tmp/check-fmt-gauss-0-6-2-wire-existing-stable-node-ids.out`,
  `/tmp/lint-gauss-0-6-2-wire-existing-stable-node-ids.out`,
  `/tmp/test-gauss-0-6-2-wire-existing-stable-node-ids.out`.
- [x] (2026-03-02) Marked roadmap item `0.6.2` done after all gates passed.

## Surprises & Discoveries

- `0.6.1` already wired stable chrome IDs into `tree_builder.rs` and has broad
  coverage for initial/incremental/no-op/error paths.
- Existing tests validated node presence and update shape, but required explicit
  semantic assertions (role/label/shortcut metadata) across all test layers to
  close `0.6.2`.
- `docs/roadmap.md` contains both checklist status (`0.6.1` done, `0.6.2`
  open) and a narrative accessibility-wiring section that should be reconciled
  during completion updates.
- Clippy style policies created a helper-function tension between
  `no_unwrap_or_else_panic`, `expect_used`, and
  `unnecessary_option_map_or_else`; the stable pattern was returning
  `Option<&Node>` from helpers and using explicit `let ... else` assertions at
  call sites.

## Decision Log

- 2026-03-02: Moved from plan-only to implementation after explicit user
  approval. Rationale: follow execplan approval gate and keep roadmap status
  aligned with delivered behaviour.

- 2026-03-02: Treat `accessibility.rs` as the canonical stable-ID and
  chrome-metadata source. Rationale: avoids drift and aligns with roadmap
  wording (“pre-defined IDs in accessibility.rs”).

- 2026-03-02: Require semantic assertions (role/label/hint) across unit, BDD,
  and GPUI layers before marking `0.6.2` complete. Rationale: checklist closure
  depends on behaviour, not only code structure.

- 2026-03-02: Keep chrome role mapping as `Role::TitleBar` for the drag region
  plus `Role::Button` for controls, while enriching nodes with label,
  description, keyboard shortcut, and click action metadata. Rationale: this
  matches current AccessKit role affordances and keeps parity with existing
  window-control behaviour.

## Outcomes & Retrospective

Current outcome: roadmap item `0.6.2` is fully delivered with green required
gates.

Delivered outcomes:

- canonical chrome semantics are centralized in
  `src/ui/phase0_shell/accessibility.rs` and consumed directly by tree
  construction;
- AccessKit chrome nodes expose stable IDs plus explicit role/label/shortcut
  metadata in initial and incremental updates;
- unit, BDD, and GPUI coverage now verifies semantic behaviour, not just node
  presence;
- roadmap and architecture/user docs are synchronized with delivered state.

Retrospective summary:

- Worker-agent parallelization accelerated implementation but required one
  follow-up polish commit to satisfy strict cross-lint helper constraints.
- Keeping documentation closure in the same pass as gate replay prevented
  roadmap status drift.

## Context and orientation

Roadmap and architecture anchors:

- `docs/roadmap.md` lines 172-180 (`0.6.2` scope and acceptance bullets).
- `docs/gauss-architecture-design.md` lines 1214-1241 (`A11yService` contract,
  stable IDs) and 1524-1543 (immediate next steps + remaining accessibility
  milestones).

Current code surfaces:

- `src/ui/phase0_shell/accessibility.rs` (stable IDs, names, shortcut hints).
- `src/ui/phase0_shell/a11y_service/tree_builder.rs` (chrome node map,
  reserved IDs, role/label wiring).
- `src/ui/phase0_shell/a11y_service/mod.rs` and
  `src/ui/phase0_shell/view.rs` (render-time sync pipeline).
- `src/ui/phase0_shell/chrome.rs` (window-control UI surface that must remain
  semantically aligned with accessibility metadata).

Validation surfaces:

- Unit: `src/ui/phase0_shell/a11y_service/tests.rs`.
- Behaviour: `tests/a11y_service_bdd.rs` and
  `tests/features/a11y_service.feature`.
- GPUI: `tests/gpui_a11y_service.rs`.
- BDD version anchor: `Cargo.toml` dev dependencies (`rstest-bdd = "0.5.0"`).

Documentation surfaces:

- `docs/gauss-architecture-design.md` (`0.6.2` decisions and status text).
- `docs/users-guide.md` accessibility status section.
- `docs/roadmap.md` checklist and accessibility-wiring summary block.

## Agent-team execution plan

Use a small team during implementation, coordinated by context pack
`pk_nbpidj6d`.

1. Reimu (explorer): validate roadmap/design text changes and closure wording.
2. Marisa (explorer): validate `leta` symbol graph before/after refactors and
   produce line-specific test target checks.
3. Axel (worker/explorer): implement chrome metadata wiring and test additions,
   then replay required gates.

If parallel edits conflict, resolve by preferring `accessibility.rs` metadata
as source of truth and documenting the final decision in the plan
`Decision Log`.

## Plan of work

Stage A: Canonicalize chrome accessibility metadata

1. Audit `accessibility.rs` metadata completeness for all chrome nodes
   (`WINDOW_MENU`, `MINIMIZE_BUTTON`, `MAXIMIZE_BUTTON`, `FULLSCREEN_BUTTON`,
   `CLOSE_BUTTON`, `TITLEBAR`).
2. Introduce (or refine) helper accessors so tree construction pulls IDs and
   labels from one canonical place rather than ad hoc literals.
3. Keep stable numeric IDs unchanged.

Acceptance for Stage A:

- No duplicated chrome node ID literals outside intended constants.
- Metadata retrieval APIs are straightforward and testable.

Stage B: Wire AccessKit chrome roles and labels

1. Update `tree_builder.rs` to consume canonical metadata for node IDs, labels,
   and shortcut hints.
2. Confirm/adjust role mapping for title bar and chrome controls; document
   maximize/fullscreen semantics decision (button vs toggle-like semantics).
3. Ensure root/chrome child ordering remains deterministic.

Acceptance for Stage B:

- Initial tree includes all required chrome nodes with expected roles and
  labels.
- Incremental updates preserve stable IDs and semantics.

Stage C: Maintain UI-chrome and accessibility parity

1. Audit `chrome.rs` button text/tooltip/IDs against accessibility metadata.
2. Remove divergence where practical in `0.6.2` scope (or document deferred
   follow-up if non-trivial).
3. Keep platform-specific shortcut differences explicit and intentional.

Acceptance for Stage C:

- No contradictory naming between UI chrome and accessibility labels for the
  same control.
- Any intentional mismatch is documented.

Stage D: Expand validation (unit + BDD + GPUI)

1. Unit tests (`rstest`):
   - assert chrome node role/label/description mappings,
   - assert maximize/restore label switching behaviour,
   - retain unhappy-path collision tests.
2. Behaviour tests (`rstest-bdd` v0.5.0):
   - add/extend scenarios to verify chrome semantic exposure,
   - keep duplicate-ID unhappy path assertions.
3. GPUI tests:
   - assert initial draw includes expected chrome semantics,
   - preserve no-op and stale-selection edge cases.

Acceptance for Stage D:

- Happy, unhappy, and edge-path coverage exists across all three layers.
- Tests fail before and pass after semantic wiring changes where applicable.

Stage E: Documentation and roadmap closure

1. Update `docs/gauss-architecture-design.md` with `0.6.2` decisions and status.
2. Update `docs/users-guide.md` if behaviour or announcements visible to users
   changed.
3. Update `docs/roadmap.md`:
   - mark `0.6.2` checklist item and child bullets done,
   - reconcile accessibility-wiring summary text with delivered state.

Acceptance for Stage E:

- Docs reflect actual behaviour and milestone status with no stale claims.

Stage F: Gates, evidence, and finalization

1. Run required gates with tee logs:

```plaintext
set -o pipefail
make check-fmt | tee /tmp/check-fmt-$(get-project)-$(git branch --show-current).out
make lint | tee /tmp/lint-$(get-project)-$(git branch --show-current).out
make test | tee /tmp/test-$(get-project)-$(git branch --show-current).out
```

2. If docs changed materially, run documentation gates before final sign-off:

```plaintext
set -o pipefail
make fmt | tee /tmp/fmt-$(get-project)-$(git branch --show-current).out
make markdownlint | tee /tmp/markdownlint-$(get-project)-$(git branch --show-current).out
make nixie | tee /tmp/nixie-$(get-project)-$(git branch --show-current).out
```

3. Record gate outcomes and keep status truthful (`complete` vs `partial`).

Acceptance for Stage F:

- All required gates pass, logs exist, and roadmap status reflects verified
  completion.

## Validation checklist (implementation-time)

- [x] Unit tests assert chrome roles/labels/shortcut hints and stable IDs.
- [x] BDD scenarios cover happy/unhappy semantics for chrome nodes.
- [x] GPUI tests verify runtime wiring and unchanged-state no-op behaviour.
- [x] `make check-fmt` passes.
- [x] `make lint` passes.
- [x] `make test` passes.
- [x] `docs/gauss-architecture-design.md` updated with design decisions.
- [x] `docs/users-guide.md` updated for user-visible accessibility semantics.
- [x] `docs/roadmap.md` `0.6.2` marked done only after gates pass.

## Approval gate

Satisfied: explicit approval received on 2026-03-02, then implementation
proceeded to completion.
