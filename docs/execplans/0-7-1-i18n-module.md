# Create the i18n module (0.7.1)

This ExecPlan (execution plan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: DRAFT (2026-03-14)

No `PLANS.md` exists in this repository.

## Purpose / big picture

Roadmap item `0.7.1` in [docs/roadmap.md](../roadmap.md) is the first
localization milestone. The architecture in
[docs/gauss-architecture-design.md](../gauss-architecture-design.md) §12 says
Gauss must treat localizability as a first-class concern, but the current code
still hard-codes English strings in model helpers such as `Action::name()`,
`Command::name()`, `KeyContext::name()`, `ToolMode::label()`,
`EdgeMode::label()`, and in Phase 0 shell UI and accessibility text.

This milestone should establish the module boundary and catalog shape that
later milestones can build on without forcing a large rewrite. Because Gauss
intends to ship multilingual support from the beginning, `0.7.1` must choose
infrastructure that is suitable for translator-authored resources and for the
initial locale set: `en-GB`, `de`, `fr`, `es`, `hi`, `zh-Hans`, `ar`, and `ja`.

After this work lands, Gauss should have a GPUI-independent `i18n` module, a
Fluent-backed message catalog, locale selection plus fallback behaviour, typed
translation errors, direction metadata needed for future right-to-left (RTL)
work, and one narrow end-to-end integration slice that proves real UI and
accessibility code can render through the new service. Broad string extraction
still belongs to roadmap item `0.7.2`, and localized command names still belong
to `0.7.3`, but the underlying localization infrastructure should be
production-grade from the start.

Success is observable when:

- `src/i18n/` exists and exposes a small, testable public API for message
  lookup, fallback, formatting, and locale metadata.
- The implementation records a concrete Fluent-versus-keyed decision in
  [docs/gauss-architecture-design.md](../gauss-architecture-design.md) §12,
  including why Fluent fits `0.7.1`.
- Fluent resource bundles exist for the initial locale set: `en-GB`, `de`,
  `fr`, `es`, `hi`, `zh-Hans`, `ar`, and `ja`, at least for the proving slice
  implemented in this milestone.
- At least one real Phase 0 shell surface renders through the `i18n` module so
  GPUI and AccessKit tests can observe the behaviour instead of only testing an
  unused helper.
- Unit tests (`rstest`), behaviour tests (`rstest-bdd` v0.5.0), and GPUI tests
  cover happy paths, unhappy paths, and edge cases.
- Documentation and roadmap state are synchronized, and required gates pass:
  `make fmt`, `make markdownlint`, `make nixie`, `make check-fmt`, `make lint`,
  and `make test`.

## Agent team

Implementation should be coordinated as one small agent team with clear
ownership and short hand-offs:

- Architecture agent: confirm the module boundary, document why Fluent is the
  correct foundation for Gauss, and update the architecture document with the
  final rationale and follow-up triggers.
- Core i18n agent: implement `src/i18n/`, including locale handling, catalog
  lookup, fallback, and typed errors.
- UI and accessibility agent: wire one proving integration slice into
  `Phase0Shell` and `A11yService` without broad extraction that belongs to
  `0.7.2`.
- Test and docs agent: drive red-green coverage across unit, BDD, and GPUI
  layers; update `docs/users-guide.md`; replay all gates with tee logs; and
  mark `0.7.1` done in the roadmap only after green results.

The team should work milestone-by-milestone, not by parallel speculative
rewrites. Parallel exploration is appropriate; parallel editing is not unless
file ownership is disjoint.

## Constraints

- Scope is limited to roadmap item `0.7.1`:
  - create the `i18n` module,
  - define the message catalog structure,
  - evaluate Fluent versus a simpler keyed system,
  - prove the module on a narrow real call path.
- Do not treat `0.7.1` as blanket UI-string extraction. Most inline Phase 0 UI
  labels stay for `0.7.2`, and localized command names stay for `0.7.3`.
- Keep the `i18n` module independent of GPUI. Views may consume localized
  strings, but catalog lookup and locale logic must not depend on GPUI types or
  window context.
- The chosen catalog structure must be capable of supporting the initial
  locale set immediately: `en-GB`, `de`, `fr`, `es`, `hi`, `zh-Hans`, `ar`, and
  `ja`.
- The foundation must expose locale direction metadata or an equivalent seam so
  later RTL layout work for Arabic does not require redesigning the i18n API.
- Do not use process-global locale state or environment mutation for tests.
  Locale selection must be injected so tests remain deterministic and parallel.
- Prefer typed errors and explicit fallbacks over silent English-only
  behaviour.
- Respect the repository file-size policy. No module may exceed 400 lines.
- Update:
  - [docs/gauss-architecture-design.md](../gauss-architecture-design.md),
  - [docs/users-guide.md](../users-guide.md),
  - [docs/roadmap.md](../roadmap.md).
- Do not begin implementation until the user explicitly approves this plan.

## Tolerances (exception triggers)

- Scope tolerance: if making the scaffold observable requires converting more
  than two existing UI or accessibility surfaces, stop and split the extra
  extraction into `0.7.2`.
- Dependency tolerance: if Fluent integration requires broad invasive
  restructuring outside a localized module and one proving call path, stop and
  decompose the adapter work rather than retreating to a less capable backend.
- Blast-radius tolerance: if correct delivery grows beyond 14 files or 700 net
  lines of code and docs, stop and decompose the work before continuing.
- API tolerance: if the best design requires changing existing public action,
  command, or keybinding APIs rather than adding adjacent i18n seams, stop and
  review that change explicitly.
- Test-harness tolerance: if GPUI verification requires replacing the existing
  shell harness instead of extending it with a locale or localizer seam, stop
  and split harness work from feature work.
- Gate tolerance: if any required gate fails for unrelated pre-existing
  reasons, capture tee-log evidence and do not mark the roadmap item done.

## Risks

- Risk: `0.7.1` expands into `0.7.2` and `0.7.3`, turning a scaffold into a
  large extraction. Mitigation: keep the end-to-end proof to one narrow Phase 0
  shell surface plus one accessibility surface.
- Risk: Fluent integration adds parser, loader, and bundle-management
  complexity in a repo that does not yet use localization directly. Mitigation:
  keep the Fluent dependency and loading logic inside `src/i18n/` and expose a
  small Gauss-owned facade to the rest of the codebase.
- Risk: the initial locale promise could turn `0.7.1` into a broad translation
  exercise. Mitigation: require locale bundles for the proving slice now, and
  let `0.7.2` expand message coverage across the application surface.
- Risk: global locale state would create flaky tests. Mitigation: pass locale
  or localizer values through constructors or test-only setters.
- Risk: accessibility strings drift from visible UI strings. Mitigation: use
  the same message identifiers for the proving slice in both the shell and
  `A11yService`.

## Implementation outline

Implementation should proceed in five milestones.

1. Establish a failing test baseline. Add unit tests for locale selection,
   fallback, message lookup, and formatting failures. Add `rstest-bdd` feature
   scenarios that describe catalog lookup and fallback behaviour in business
   terms. Add one GPUI test that proves a Phase 0 shell surface changes when a
   shipped non-English locale is selected. The first pass should fail because
   the i18n layer does not exist yet.

2. Add the core module under `src/i18n/`. Keep files small and purpose-driven,
   likely split as `mod.rs`, `locale.rs`, `message.rs`, `catalog.rs`, and
   `error.rs` if needed. Export the module from `src/lib.rs`. The public API
   should cover:
   - a locale type or newtype,
   - stable message identifiers,
   - a localizer or catalog lookup service,
   - typed lookup and formatting errors,
   - explicit fallback to the default catalog,
   - locale metadata needed for later layout decisions, including text
     direction.

3. Implement the catalog backend for `0.7.1`. The recommended choice for this
   milestone is Fluent, not a simpler keyed system. Store translator-authored
   resources in locale-specific Fluent files and load them through a small
   Gauss-owned adapter so the rest of the application is not coupled directly
   to Fluent internals. Provide locale bundles for `en-GB`, `de`, `fr`, `es`,
   `hi`, `zh-Hans`, `ar`, and `ja` for the messages introduced in this
   milestone. Use a fallback chain rooted at `en-GB`.

4. Wire one real integration slice. Keep it intentionally narrow. The best
   proving seam in the current codebase is the mode/status text shared between
   `src/ui/phase0_shell/view.rs` and
   `src/ui/phase0_shell/a11y_service/tree_builder.rs`, because both currently
   hard-code English phrases built from `ToolMode::label()` and
   `EdgeMode::label()`. Introduce an injected localizer seam so:
   - the Phase 0 shell status line uses the new message IDs,
   - the accessibility status node uses the same localized message IDs,
   - tests can override the catalog without mutating global state.
   Leave other chrome labels, tooltips, and command names for the later roadmap
   items.

5. Record the decision and close the milestone. Update the architecture
   document to explain why `0.7.1` chose Fluent now, how the initial locale set
   is represented, and which future roadmap items own broader string extraction
   and command-name localization. Update the user guide with any user-visible
   behaviour change, including the initial locale support story if it becomes
   observable through system-locale selection or an equivalent app-level seam.
   Then update the roadmap checkbox for `0.7.1` only after all gates pass.

## Recommended design

The recommended `0.7.1` design is:

- `src/i18n/` as a top-level library module, not nested under `model` or `ui`.
- A Fluent-backed catalog behind a Gauss-owned facade with stable message
  identifiers.
- Shipped locale bundles for `en-GB`, `de`, `fr`, `es`, `hi`, `zh-Hans`, `ar`,
  and `ja` for the initial message set.
- Explicit locale fallback to `en-GB`.
- Locale metadata that can distinguish left-to-right (LTR) and RTL locales.
- No requirement to build a full locale-preference UI in this milestone.
- No attempt to localize every existing string in this milestone.

This choice fits the actual architectural goal better than a keyed system.
Gauss intends to provide multilingual support from the beginning, including
German, French, Spanish, Hindi, Simplified Chinese, Arabic, and Japanese as
soon as the infrastructure exists. That implies translator-authored resources,
placeholder formatting across multiple grammatical systems, and an early path
for Arabic and later RTL UI work. Fluent is a better fit for those constraints
than inventing a custom keyed format now and migrating later.

The Gauss-facing API should still stay small so future changes remain local to
`src/i18n/`, but the backend decision for `0.7.1` should be Fluent.

## Concrete file plan

Expected files to add or update:

- `src/i18n/mod.rs`
- `src/i18n/locale.rs`
- `src/i18n/message.rs`
- `src/i18n/catalog.rs`
- `src/i18n/error.rs`
- `i18n/en-GB/gauss.ftl`
- `i18n/de/gauss.ftl`
- `i18n/fr/gauss.ftl`
- `i18n/es/gauss.ftl`
- `i18n/hi/gauss.ftl`
- `i18n/zh-Hans/gauss.ftl`
- `i18n/ar/gauss.ftl`
- `i18n/ja/gauss.ftl`
- `src/lib.rs`
- `src/ui/phase0_shell/mod.rs` or `src/ui/phase0_shell/test_helpers.rs` for an
  injected localizer seam
- `src/ui/phase0_shell/view.rs`
- `src/ui/phase0_shell/a11y_service/tree_builder.rs`
- `tests/features/i18n.feature`
- `tests/i18n_bdd.rs`
- `tests/gpui_i18n_module.rs`
- `docs/gauss-architecture-design.md`
- `docs/users-guide.md`
- `docs/roadmap.md`

During implementation, keep an eye on `src/model/tool.rs`,
`src/model/action.rs`, `src/model/command/command_def.rs`, and
`src/model/key_context.rs`. They already document future i18n work in comments.
For `0.7.1`, prefer adding adjacent message-identifier helpers only when needed
by the proving slice, not broad API churn.

## Validation

Validation must follow a red-green-refactor flow.

Unit tests should use `rstest` and cover:

- successful lookup in every shipped locale for the initial message set,
- locale fallback when a requested locale is unsupported,
- typed failure for an unknown message identifier,
- typed failure for missing formatting arguments,
- stability of the shipped message registry,
- locale metadata such as text direction for Arabic versus the LTR locales.

Behaviour tests should use `rstest-bdd` v0.5.0 and cover:

- a requested locale using its own Fluent catalog,
- fallback to English when a locale is unsupported,
- failure reporting when a message key is unavailable,
- selection of one LTR locale and one RTL locale from the shipped set.

GPUI tests should cover:

- the Phase 0 shell status text rendering localized output through a shipped
  non-English locale,
- the accessibility status node using the same localized output,
- unchanged behaviour when the default English catalog is used,
- exposure of locale direction metadata through the proving seam if the UI seam
  surfaces it.

Use `set -o pipefail` and `tee` for every long-running gate. Expected commands:

```sh
set -o pipefail && make fmt 2>&1 | tee /tmp/fmt-gauss-0-7-1-i18n-module.out
set -o pipefail && make markdownlint 2>&1 | tee /tmp/markdownlint-gauss-0-7-1-i18n-module.out
set -o pipefail && make nixie 2>&1 | tee /tmp/nixie-gauss-0-7-1-i18n-module.out
set -o pipefail && make check-fmt 2>&1 | tee /tmp/check-fmt-gauss-0-7-1-i18n-module.out
set -o pipefail && make lint 2>&1 | tee /tmp/lint-gauss-0-7-1-i18n-module.out
set -o pipefail && make test 2>&1 | tee /tmp/test-gauss-0-7-1-i18n-module.out
```

Completion evidence is:

- all commands exit successfully,
- the new unit, BDD, and GPUI tests fail before implementation and pass after,
- documentation reflects the final decision,
- `docs/roadmap.md` marks `0.7.1` done only after the gate replay succeeds.

## Progress

- [x] (2026-03-14) Loaded repository instructions, roadmap context, architecture
  references, testing guidance, and existing execplans.
- [x] (2026-03-14) Queried project notes for architecture, tooling, and known
  documentation gate expectations.
- [x] (2026-03-14) Used `leta` plus direct code inspection to map existing
  string call sites and current action, command, tool, and accessibility seams.
- [x] (2026-03-14) Attempted to use `grepai` as requested, confirmed it is not
  installed in this environment, and fell back to `rg` for plain-text search.
- [x] (2026-03-14) Drafted this ExecPlan with an explicit approval gate before
  implementation.
- [x] (2026-03-14) Revised the draft to align with the requirement that Gauss
  should support multilingual operation from the start, including the initial
  locale set `en-GB`, `de`, `fr`, `es`, `hi`, `zh-Hans`, `ar`, and `ja`.
- [ ] Await user approval.
- [ ] Implement the i18n module and proving slice.
- [ ] Run all required gates with tee logs.
- [ ] Update the roadmap entry to done.

## Surprises & Discoveries

- `grepai` is not installed in this environment. `leta` is available and works
  well for symbol discovery; `rg` is still needed for plain-text string audits.
- The architecture asks for localization in §12, but the codebase still keeps
  English labels in model helpers and Phase 0 shell rendering.
- The cleanest real proving seam is the status text path:
  `src/ui/phase0_shell/view.rs` and
  `src/ui/phase0_shell/a11y_service/tree_builder.rs` both build English status
  strings from `ToolMode::label()` and `EdgeMode::label()`.
- `A11yService` currently hard-codes additional English strings such as
  `"Drawing canvas"`, `"Shapes"`, `"Shape {n}"`, and `"Gauss"`, which confirms
  there is plenty of follow-on work for `0.7.2`.
- Gauss has no direct localization dependency in `Cargo.toml` today. Fluent and
  related crates only appear transitively in `Cargo.lock`, so `0.7.1` will need
  to choose and add a direct localization stack intentionally.
- Existing test seams are already strong enough for this work: model-layer
  `rstest` tests, `rstest-bdd` scenario tests, and GPUI shell tests all exist.
- The architectural requirement is stronger than the current proof-of-concept
  UI surface. The i18n choice therefore needs to optimize for the future
  multilingual application, not for today's small set of English strings.

## Decision Log

- 2026-03-14: Plan `0.7.1` as a narrow scaffold milestone, not the full string
  extraction. Rationale: the roadmap explicitly splits module creation,
  extraction, and command-name localization into separate items.
- 2026-03-14: Revise the recommendation from a keyed catalog to Fluent.
  Rationale: Gauss intends to support multiple locales from the start, with
  translator-authored resources and early RTL-aware foundations.
- 2026-03-14: Require one real UI plus accessibility proving slice instead of a
  dead internal helper. Rationale: the repository requires GPUI tests and the
  ExecPlan must lead to observable behaviour.
- 2026-03-14: Keep locale selection injected and testable rather than global.
  Rationale: the repository forbids flaky test patterns and prefers dependency
  injection for non-deterministic state.
- 2026-03-14: Do not begin implementation until the user approves this
  document. Rationale: follow the mandatory execplans approval gate.

## Outcomes & Retrospective

Current outcome: draft plan only.

If executed as written, this milestone should leave Gauss with a small,
coherent localization spine:

- a top-level, GPUI-independent `i18n` module,
- a documented decision for Fluent now,
- initial locale bundles for `en-GB`, `de`, `fr`, `es`, `hi`, `zh-Hans`, `ar`,
  and `ja` for the proving slice,
- one real localized shell plus accessibility path,
- tests at unit, behaviour, and GPUI levels,
- synchronized architecture, user, and roadmap documentation.

The main lesson already visible from planning is that the codebase has enough
English call sites to tempt an over-scoped rewrite. Delivery quality here will
come from resisting that temptation and proving the architecture with one real
slice first.
