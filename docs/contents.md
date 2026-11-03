# Documentation contents

This index groups Gauss documentation by its purpose and audience.

## Start here

- [Documentation contents](contents.md) — Find documentation by subject.
- [User's guide](users-guide.md) — Learn to use Gauss and its editing tools.
- [Developer's guide](developers-guide.md) — Build, test, and contribute.
- [Development roadmap](roadmap.md) — Review planned work and delivery status.
- [Feature parity plan](gauss-feature-plan.md) — Explore the Illustrator parity
  goals and feature sequence.
- [Keyboard shortcuts](gauss-keyboard-shortcuts.md) — Find the current key
  bindings.

## Architecture and decisions

- [Architecture design](gauss-architecture-design.md) — Understand system
  boundaries, design principles, and subsystem relationships.
- [AccessKit accessibility proposal](accesskit-based-accessibility-in-gpui.md)
  — Review the accessibility design for GPUI.
- [GPUI and GPUI Component guide](using-gpui-and-gpui-component.md) — Learn how
  Gauss uses the GPUI libraries.
- [GPUI AccessKit primer](gpui-accesskit-primer.md) — Review the AccessKit
  integration concepts used by Gauss.
- [Widget capability audit](widget-capability-audit.md) — Check the Phase 1–2
  widget requirements and current coverage.
- [Widget audit developer guide](widget-audit-developer-guide.md) — Use the
  project workflow for widget capability audits.
- [ADR 0001](adrs/adr-0001-command-docop-relationship.md) — Understand the
  boundary between user intent and document mutations.
- [ADR 0002: Undo history crate](adrs/adr-0002-undo-history-crate-selection.md)
  — Review the decision to use `undo_2` for document history.
- [ADR 0003](adrs/adr-0003-slotmap-shapeid-accesskit-id-mapping.md) — Understand
  the stable identifier mapping for shapes and accessibility.
- [ADR 0004](adrs/adr-0004-resource-style-store-and-paint-model.md) — Review
  the model for shared SVG resources and paint values.
- [ADR 0005: Metadata namespace](adrs/adr-0005-gauss-metadata-namespace.md)
  — Find the namespace policy for Gauss SVG metadata.
- [ADR 0006: Rust build standard](adrs/adr-0006-rust-build-standard.md)
  — Review the development build and linker policy.

## Guides and reference

- [Documentation style guide](documentation-style-guide.md) — Follow the
  project's documentation conventions.
- [Scripting standards](scripting-standards.md) — Follow conventions for
  project scripts.
- [Refactoring guide](complexity-antipatterns-and-refactoring-strategies.md) —
  Recognize complexity patterns and choose suitable refactorings.
- [Injected testing](reliable-testing-in-rust-via-dependency-injection.md) —
  Design tests around injected dependencies and controlled effects.
- [Rust doctest guide](rust-doctest-dry-guide.md) — Write useful, maintainable
  Rust documentation tests.
- [Rust tests with rstest fixtures](rust-testing-with-rstest-fixtures.md) —
  Organize Rust tests around reusable fixtures.
- [Whitaker user's guide](whitaker-users-guide.md) — Configure and use the
  project's lint tooling.
- [rstest-bdd user's guide](rstest-bdd-users-guide.md) — Use the behavioural
  testing framework adopted by Gauss.
- [rstest-bdd v0.5.0 migration guide](rstest-bdd-v0-5-0-migration-guide.md)
  — Review the migration notes for rstest-bdd v0.5.0.
- [rstest-bdd v0.6.0 migration guide](rstest-bdd-v0-6-0-migration-guide.md)
  — Review the migration notes for rstest-bdd v0.6.0.

## Execution plans

### Command dispatch

- [Command dispatch plan](execplans/0-1-2-command-dispatch.md) — Follow the
  implementation plan for command dispatch.
- [Command dispatch overview](execplans/0-1-2-command-dispatch/overview.md) —
  Review the design and scope of the command dispatch work.
- [Dispatch implementation](execplans/0-1-2-command-dispatch/implementation.md)
  — Follow the detailed command-dispatch implementation steps.
- [Command dispatch testing](execplans/0-1-2-command-dispatch/testing.md) —
  Review the test strategy for command dispatch.
- [Architecture update snippet](execplans/0-1-2-command-dispatch/snippets/step11-architecture-update.md)
  — Reuse the planned architecture documentation update.

### Product and editing capabilities

- [Typed action command registry](execplans/0-1-1-action-command-registry.md)
  — Plan the typed action enum and command registry.
- [Key context system](execplans/0-1-3-key-context-system.md) — Plan keyboard
  context handling.
- [Stable ID generation](execplans/0-2-2-stable-id-generation.md) — Plan
  stable shape and accessibility identifiers.
- [Resource stores](execplans/0-2-3-define-resource-stores.md) — Plan typed
  resource and style stores.
- [Undo and redo audit](execplans/0-3-1-audit-undo-redo-implementation.md) —
  Review the existing undo and redo implementation.
- [Undo history spike](execplans/adr-002-undo-2-spike.md) — Record the
  exploratory work for command-layer undo.
- [Command grouping API](execplans/0-3-2-command-grouping-api.md) — Plan
  grouping related document history operations.
- [History ownership](execplans/0-3-4-state-engine-ownership-of-history.md)
  — Plan moving document history into EngineState.
- [History error migration](execplans/0-3-5-evolve-history-error-model-to-enum.md)
  — Complete the error migration across implementation and tests.
- [Metadata namespace](execplans/0-4-1-define-metadata-namespace.md) — Plan
  the Gauss metadata namespace policy.
- [Metadata round-trip](execplans/0-4-2-metadata-round-trip.md) — Plan
  metadata preservation through SVG import and export.
- [Web-ready SVG export](execplans/0-4-3-web-ready-export.md) — Plan
  improvements to exported SVG for web use.
- [Tool trait and command boundary](execplans/0-5-1-tool-trait.md) — Plan the
  shared tool interface and finite-state-machine boundary.
- [Draw refactor](execplans/0-5-2-refactor-draw-mode-to-use-tool-trait.md)
  — Plan the draw mode migration to `PenTool`.
- [Selection tool refactor](execplans/0-5-3-refactor-manipulate-mode-to-tool-trait.md)
  — Plan the manipulate mode migration to `SelectTool`.
- [Shared hit-test service](execplans/0-5-4-shared-hit-test-service.md) — Plan
  deterministic selection and hover hit testing.
- [Accessibility service](execplans/0-6-1-create-a11y-service-structure.md)
  — Plan incremental accessibility updates.
- [Stable node IDs](execplans/0-6-2-wire-existing-stable-node-ids.md) — Plan
  wiring stable identifiers into accessibility semantics.
- [AccessKit action mapping](execplans/0-6-3-map-access-kit-action-requests-to-gauss-actions.md)
  — Plan mapping accessibility requests to Gauss actions.
- [Internationalization module](execplans/0-7-1-i18n-module.md) — Plan the
  internationalization module.
- [i18n test resolution](execplans/0-7-1-i18n-test-resolution.md)
  — Plan reliable test resolution for internationalized strings.
- [Internationalization timeout analysis](execplans/0-7-1-i18n-module-test-timeout-analysis.md)
  — Investigate timeouts in the internationalization module tests.
- [Extract UI strings](execplans/0-7-2-extract-ui-strings.md) — Plan moving UI
  text into translation resources.
- [Required controls list](execplans/0-8-1-list-required-controls-for-phase-1-2.md)
  — Inventory the widget audit's required controls.
- [Issue 18 command variants](execplans/issue-18-add-command-variants.md) —
  Plan migrating Phase 0 edits to commands.
- [Phase 0 proof of concept](execplans/phase-0-poc.md) — Record the initial
  Gauss vector editor proof of concept.
- [Phase 1 basic UI](execplans/phase-1-basic-ui.md) — Plan the initial UI
  shell.

### Test framework adoption and test organization

- [rstest-bdd v0.6.0 adoption](execplans/adopt-rstest-bdd-v0-6-0.md) — Plan
  the workspace migration to rstest-bdd v0.6.0.
- [rstest-bdd beta adoption](execplans/adopt-rstest-bdd-v0-6-0-beta3.md) —
  Record the beta migration and GPUI behavioural test work.
- [Test classification inventory](execplans/test-classification-inventory.md)
  — Track test locations and classifications for test moves.

### Build-time improvements

- [GPUI feature selection audit](execplans/build-time-audit-gpui-feature-selection.md)
  — Identify a narrower set of GPUI build features.
- [Integration test target consolidation](execplans/build-time-consolidate-integration-test-targets.md)
  — Reduce compile churn from integration test targets.
- [Model and SVG test moves](execplans/build-time-move-model-and-svg-tests.md)
  — Plan moving tests behind non-GPUI crate boundaries.
- [Development debuginfo reduction](execplans/build-time-trim-dev-debuginfo-for-dependencies.md)
  — Reduce dependency debuginfo in development builds.
- [Workspace crate split](execplans/build-time-workspace-crate-split.md) —
  Plan separating core, SVG, and application crates.
