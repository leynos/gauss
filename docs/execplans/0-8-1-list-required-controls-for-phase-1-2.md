# List required controls for Phase 1-2 widget audit (0.8.1)

This ExecPlan (execution plan) is a living document. The sections `Constraints`,
`Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`, `Decision Log`,
and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Status: DRAFT (2026-03-14)

## Purpose / big picture

Roadmap item `0.8.1` in [docs/roadmap.md](../roadmap.md) asks for the Phase 1
and Phase 2 control surfaces to be enumerated before broad user-interface (UI)
work accelerates. The goal is not to ship the controls yet. The goal is to
establish a trustworthy inventory of which controls Gauss must support for the
Phase 1 and Phase 2 roadmap, what each control must do, and which
accessibility, keyboard, and action-routing requirements each one carries.

This matters because architecture section `14.1` in
[docs/gauss-architecture-design.md](../gauss-architecture-design.md) says the
widget capability audit must happen early and must distinguish between stock
`gpui-component` coverage and future custom-control pressure. If this audit is
left implicit, later UI work will drift between the roadmap, the architecture
document, and the actual shell seams in `src/ui/phase0_shell/`.

Success is observable when:

- the repository contains one canonical Phase 1-2 control inventory that lists
  all required toolbars, panels, layer controls, properties controls, and
  colour-related controls implied by the roadmap and feature plan;
- every listed control records its user-facing purpose, core states, keyboard
  expectations, accessibility expectations, and required Action-to-Command
  linkage;
- unit tests (`rstest`), behaviour tests (`rstest-bdd` 0.6.0-beta3), and GPUI
  tests prove that the audit artefact is complete, internally consistent, and
  grounded in the current shell seams rather than being a free-form note;
- `docs/gauss-architecture-design.md` records the design decision that keeps
  this audit maintainable;
- `docs/roadmap.md` marks `0.8.1` done only after the audit artefact,
  documentation updates, tests, and full repository gates pass;
- implementation does not begin until the user explicitly approves this draft.

## Context and orientation

The current UI root remains `Phase0Shell` in `src/ui/phase0_shell/mod.rs`,
rendered by `src/ui/phase0_shell/view.rs`. The existing shell already exposes
early seams that the audit must reference by name:

- `src/ui/phase0_shell/tool_rail.rs` for tool selection chrome;
- `src/ui/phase0_shell/style_controls.rs` for fill/stroke colour pickers;
- `src/ui/phase0_shell/chrome_panels.rs` for document-header and status-bar
  panel chrome;
- `src/ui/action_bridge/mod.rs` and `src/model/action.rs` for action routing;
- `src/ui/phase0_shell/accessibility.rs` and
  `src/ui/phase0_shell/a11y_service/` for accessibility expectations.

The roadmap spreads the relevant requirements across several items:

- `0.8.1` asks for the control list and requirements;
- `1.3`, `1.4`, `1.5`, `1.6.2`, `1.8.1`, and `1.9` define Phase 1 transform,
  style, layers, history, widget-audit, and accessibility pressure;
- `2.2` defines the Phase 2 Character and Paragraph panels plus text-colour
  reuse.

The feature plan expands that into concrete control families: toolbox, canvas
adjacent controls, fill/stroke panels, layers, a text/character panel,
paragraph controls, and options-bar style affordances. Architecture section
`14.1` then turns that into an explicit audit obligation.

Because the user requested test coverage for this roadmap item, the
implementation described below does not treat the audit as prose-only
documentation. Instead, it proposes a typed inventory in Rust that can be
validated by unit, BDD, and GPUI tests, with human-readable documentation
derived from or kept in lockstep with that inventory.

## Agent team

This planning draft uses a Logisphere-style agent team so the implementation
can proceed with explicit ownership rather than an undifferentiated to-do list.

- Pandalump (architecture lead) owns the inventory shape, file boundaries, and
  separation between roadmap requirements, typed audit data, and UI modules.
- Telefono (contracts lead) owns the schema for each control entry so invalid
  or incomplete requirement records are hard to express.
- Doggylump (reliability and accessibility lead) owns failure modes,
  accessibility expectations, and the rule that unsupported controls stay
  documented rather than silently implied.
- Dinolump (documentation and maintainability lead) owns the narrative docs,
  cross-links, and keeping the audit understandable to a new contributor.
- Wafflecat (alternatives lead) owns the rejected alternatives and the rule
  that docs-only solutions are not enough for this repository because they do
  not satisfy the requested automated validation.
- Buzzy Bee (validation lead) owns the test layering and the gate replay plan.

The same team structure should be used during implementation, even if one human
or one coding agent performs the edits sequentially.

## Constraints

- Scope is limited to roadmap item `0.8.1`: list required controls for
  Phase 1-2 and document each control's requirements.
- Do not perform widget-to-`gpui-component` mapping in the implementation of
  this plan except where a requirement record needs to say that current shell
  evidence already exists. Full mapping belongs to roadmap item `0.8.2`.
- Do not design or implement new custom widgets here. Custom-widget planning
  belongs to roadmap item `0.8.3`.
- Preserve the existing Action-to-Command architecture. This audit must record
  the requirement that each future control route through actions and commands;
  it must not introduce UI-only behaviour.
- Keep the current `Phase0Shell` root view and current shell modules stable.
  The audit may reference or lightly exercise them in tests, but it must not
  refactor the shell as part of `0.8.1`.
- Any Rust additions must obey repository rules:
  module-level `//!` comments, public API docs where applicable, no file above
  400 lines, and no lint suppressions except as a last resort.
- Required validation for the final implementation remains:
  `rstest` unit tests, `rstest-bdd` 0.6.0-beta3 behaviour tests, `#[gpui::test]`
  coverage, `make check-fmt`, `make lint`, and `make test`.
- Update `docs/gauss-architecture-design.md` with any design decision required
  to explain the chosen audit source of truth.
- Update `docs/users-guide.md` only if the implementation introduces a new
  user-visible UI or behaviour. If the final change remains internal and
  documentary, record explicitly why no user's guide delta is needed.
- Do not mark `docs/roadmap.md` item `0.8.1` done until the implementation is
  complete, approved, and fully validated.
- Do not begin implementation until the user explicitly approves this
  ExecPlan.

## Tolerances (exception triggers)

- Scope tolerance: if completing `0.8.1` requires shipping the widget mapping
  for `0.8.2` or a custom control prototype for `0.8.3`, stop and split the
  work instead of rolling multiple roadmap items together.
- File tolerance: if the implementation grows beyond 12 files or 550 net lines
  of changes, pause and re-scope before proceeding.
- Interface tolerance: if a typed audit catalogue would need to become part of
  a public library API rather than an internal UI-planning artefact, stop and
  re-evaluate the shape.
- Documentation tolerance: if the audit cannot be kept self-consistent across
  the typed inventory, design document, and roadmap without introducing
  generation tooling or a new dependency, stop and ask whether that extra
  tooling is acceptable.
- Test tolerance: if GPUI coverage would require introducing a synthetic audit
  viewer or other new UI just to satisfy the GPUI layer, stop and confirm
  whether a shell-seam validation test is sufficient.
- Dependency tolerance: if a new external crate appears necessary, stop and
  escalate rather than widening the blast radius of a documentation-centric
  roadmap item.
- Tooling tolerance: if `grepai` remains unavailable in the environment during
  implementation, proceed with `leta` plus `rg` only if the evidence remains
  locally verifiable; otherwise stop and request the missing tool.
- Gate tolerance: if `make check-fmt`, `make lint`, or `make test` fail for
  unrelated pre-existing reasons, capture tee-log evidence and leave roadmap
  closure undone.

## Risks

- Risk: the audit could become a prose document that immediately drifts from
  the roadmap. Severity: high. Likelihood: medium. Mitigation: keep one typed
  source-of-truth inventory and test it.
- Risk: the implementation could accidentally slide into `0.8.2` mapping work.
  Severity: high. Likelihood: medium. Mitigation: keep `0.8.1` entries focused
  on requirements, current evidence, and deferred mapping notes only.
- Risk: tests could become fake ceremony for a documentation task. Severity:
  medium. Likelihood: high. Mitigation: test completeness, consistency, and
  alignment with actual shell seams rather than trivial string snapshots.
- Risk: the audit may undercount controls because requirements are spread
  across the roadmap, feature plan, and architecture document. Severity: high.
  Likelihood: medium. Mitigation: extract requirements from all three sources
  and encode source anchors per control entry.
- Risk: accessibility and keyboard requirements may be forgotten for panels and
  inspectors that do not exist yet. Severity: high. Likelihood: medium.
  Mitigation: make accessibility and keyboard fields mandatory in the typed
  schema.
- Risk: `grepai` is not currently available in this environment. Severity:
  low. Likelihood: high. Mitigation: record the limitation, use `leta` for
  symbol-aware discovery, and use `rg` for text-level coverage checks.

## Progress

- [x] (2026-03-14) Loaded `AGENTS.md`, the required project memory notes, and
  the `execplans` skill before drafting.
- [x] (2026-03-14) Loaded the Logisphere experts skill and used it as the
  planning agent team for this draft.
- [x] (2026-03-14) Reviewed `docs/roadmap.md`,
  `docs/gauss-architecture-design.md`, `docs/gauss-feature-plan.md`,
  `docs/using-gpui-and-gpui-component.md`,
  `docs/accesskit-based-accessibility-in-gpui.md`,
  `docs/rust-testing-with-rstest-fixtures.md`, `docs/rust-doctest-dry-guide.md`,
  `docs/reliable-testing-in-rust-via-dependency-injection.md`, and
  `docs/rstest-bdd-users-guide.md`.
- [x] (2026-03-14) Verified that `leta` is installed and usable for codebase
  understanding.
- [x] (2026-03-14) Verified that no `grepai` binary or MCP resource is present
  in this environment; planned fallback discovery uses `leta` plus `rg`.
- [x] (2026-03-14) Inspected the current shell seams in
  `src/ui/phase0_shell/view.rs`, `src/ui/phase0_shell/tool_rail.rs`,
  `src/ui/phase0_shell/style_controls.rs`, and
  `src/ui/phase0_shell/chrome_panels.rs`.
- [x] (2026-03-14) Drafted this ExecPlan at
  `docs/execplans/0-8-1-list-required-controls-for-phase-1-2.md`.
- [ ] Await user approval.
- [ ] Implement the typed control inventory, documentation updates, tests, and
  roadmap closure described below.

## Surprises & Discoveries

- `docs/gauss-architecture-design.md` section `14.1` is narrower and more
  actionable than the roadmap wording: it explicitly expects a three-step flow
  of list, widget mapping, and small internal widget library. That confirms
  `0.8.1`, `0.8.2`, and `0.8.3` should remain separate.
- The current shell already has concrete anchors for some audit rows:
  `tool_rail.rs` proves toolbar pressure exists now, `style_controls.rs` proves
  `gpui-component` colour pickers are already in use, and `chrome_panels.rs`
  shows document-header and status-bar panels.
- The roadmap's `0.8.1` bullet list is not exhaustive by itself. Phase 1-2
  requirements also imply transform fields, history presentation, Character and
  Paragraph controls, and text-editing affordances.
- `grepai` is not available in this environment. `leta` is available and works
  as the symbol-aware discovery tool. `rg` remains necessary for multi-file
  roadmap and documentation coverage checks.
- A docs-only implementation would be difficult to validate with the required
  `rstest`, BDD, and GPUI layers. A typed audit catalogue is the most credible
  way to satisfy the repository's testing bar without inventing runtime
  behaviour.

## Decision Log

- 2026-03-14: Use a typed internal audit catalogue plus companion
  documentation, not prose-only notes. Rationale: the user explicitly requires
  automated validation, and the repository policy expects unit, BDD, and GPUI
  coverage for delivered work.
- 2026-03-14: Keep the current shell as evidence, not as the audit's source of
  truth. Rationale: the current shell only covers a subset of Phase 1-2
  controls, so the audit must be broader than the current implementation.
- 2026-03-14: Record keyboard, accessibility, and action-routing requirements
  as mandatory fields per control. Rationale: the architecture document treats
  those as non-negotiable, and omitting them here would merely defer the real
  audit.
- 2026-03-14: Treat `grepai` as unavailable for this planning turn and rely on
  `leta` plus `rg`. Rationale: there is no installed binary or MCP resource to
  invoke, and the draft must stay grounded in verifiable local evidence.
- 2026-03-14: No implementation work begins until the user approves this
  draft. Rationale: the `execplans` approval gate is mandatory for this task.

## Plan of work

Implementation should proceed in four milestones.

1. Establish the audit source of truth.

Create a new internal module such as `src/ui/widget_audit/` that defines a typed
`RequiredControl` record and supporting enums. Keep the schema focused on
requirements rather than implementation mapping. A novice should be able to
open that module and answer, for each control, all of the following:

- which phase requires it;
- which surface it belongs to, such as toolbar, panel, popover, inspector
  section, on-canvas text edit affordance, or layer-row control;
- what user job it supports;
- what states it must expose;
- what keyboard and accessibility behaviour it must support;
- which existing source documents justify its inclusion;
- whether current shell evidence already exists.

This module should stay internal to the UI crate. It is a planning artefact,
not a public product API.

1. Populate the Phase 1-2 inventory.

Encode the concrete controls implied by the roadmap and feature plan. At
minimum this should cover:

- tool-selection toolbar entries for select, pen/path, rectangle, ellipse,
  line, and type;
- properties-panel controls for `X`, `Y`, `Width`, `Height`, and `Rotation`;
- alignment and distribution controls required by `1.3.4` and `1.3.5`;
- stroke controls: colour, width, opacity;
- fill controls: colour, opacity, no-fill state;
- layers-panel rows, visibility toggle, lock toggle, rename affordance, and
  reorder affordance;
- optional history-panel list if it remains in-scope for Phase 1 requirements;
- Character-panel controls: font family, font size, basic styles, alignment,
  text colour;
- Paragraph-panel controls required by the Phase 2 roadmap wording;
- direct text-editing affordances that must exist on canvas even if they are
  not classic stock widgets, because the audit is about required controls, not
  only panel chrome.

Each entry must cite the roadmap or feature-plan anchor that requires it.

1. Publish the human-readable audit and architecture note.

Add or update a documentation page that presents the inventory in prose or a
compact table, keeping it understandable for humans who do not want to read the
Rust source. This can live in a new document under `docs/`, or in a focused
section of `docs/gauss-architecture-design.md` if that keeps the source of
truth clearer. Also update architecture section `14.1` to record the design
decision that the typed inventory is the canonical audit source for `0.8.1` and
the input to later `0.8.2` mapping work.

Update `docs/users-guide.md` only if implementation introduces a user-visible
inspection view or any changed UI behaviour. If the implementation remains
internal plus documentation-only, add no speculative user's guide text.

1. Validate and close the roadmap item.

Add a unit-test module that uses `rstest` parameterization to assert that every
required control family exists and that mandatory requirement fields are not
missing.

Add a BDD feature such as `tests/features/widget_capability_audit.feature` and
bindings such as `tests/widget_capability_audit_bdd.rs` that express the audit
from the roadmap perspective. The scenarios should prove observable planning
behaviour, for example that every Phase 1-2 control category named in the
roadmap resolves to one or more typed inventory entries with documented
requirements.

Add a focused GPUI test only for the part that truly requires GPUI wiring. The
best candidate is a shell-seam consistency check, such as proving that the
currently shipped tool rail and colour-picker controls correspond to audit
entries that mark them as having current shell evidence. Do not invent a new UI
surface solely to satisfy the GPUI layer.

After tests and documentation are complete, mark roadmap item `0.8.1` done in
`docs/roadmap.md`.

## Validation and evidence

Before implementation, establish the red-green path by writing tests that fail
because the inventory is incomplete or missing. Then fill the inventory until
they pass. Use the existing test layering guidance in the repository:

- `rstest` for the data-contract checks and parameterized requirement coverage;
- `rstest-bdd` for roadmap-language scenarios;
- `#[gpui::test]` only where a real shell seam or GPUI entity must be
  exercised.

When implementation is complete, run the required gates from the repository
root with `tee` and `set -o pipefail`:

```plaintext
set -o pipefail
make check-fmt 2>&1 | tee /tmp/0-8-1-check-fmt.log
```

```plaintext
set -o pipefail
make lint 2>&1 | tee /tmp/0-8-1-lint.log
```

```plaintext
set -o pipefail
make test 2>&1 | tee /tmp/0-8-1-test.log
```

If documentation outside the Rust tree changes, also run:

```plaintext
set -o pipefail
make fmt 2>&1 | tee /tmp/0-8-1-fmt.log
```

```plaintext
set -o pipefail
make markdownlint 2>&1 | tee /tmp/0-8-1-markdownlint.log
```

```plaintext
set -o pipefail
make nixie 2>&1 | tee /tmp/0-8-1-nixie.log
```

The implementation is complete only when the inventory exists, docs are
updated, tests are green, and the roadmap entry is marked done.

## Outcomes & Retrospective

Pending. This section must remain in place and be completed after the user
approves the plan and implementation finishes.
