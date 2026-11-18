# Resolve the inherited proc-macro-error2 future incompatibility (#246)

This ExecPlan (execution plan) is a living document. The sections `Constraints`,
`Tolerances`, `Risks`, `Progress`, `Surprises & discoveries`, `Decision log`,
`Outcomes & retrospective`, `Conformance basis`, and `Verification plan` must
be kept up to date as work proceeds.

Status: BLOCKED — the dependency disposition awaits the owner's explicit
approval. The documentation change is complete and delivered as draft pull
request [#317](https://github.com/leynos/gauss/pull/317).

Branch:
`issue-246-resolve-the-inherited-proc-macro-error2-future-incompatibility`

Roadmap item: none. The governing issue is
[leynos/gauss#246](https://github.com/leynos/gauss/issues/246).

## Purpose / big picture

Gauss inherits a Rust future-incompatibility warning from an unmaintained
transitive crate. The path is:

```plaintext
gauss -> gpui 0.2.2 -> stacksafe 0.1.4 -> stacksafe-macro 0.1.4
      -> proc-macro-error2 2.0.1 -> proc-macro-error-attr2 2.0.0
```

`proc-macro-error2 2.0.1` re-exports the private `proc_macro` crate from its
`src/lib.rs:494`:

```plaintext
warning[E0365]: extern crate `proc_macro` is private and cannot be re-exported
   --> .../proc-macro-error2-2.0.1/src/lib.rs:494:13
    |
494 |     pub use proc_macro;
    |             ^^^^^^^^^^
```

Rust currently accepts this with a warning but states that it will become a
hard error in a future release. The repository does not suppress the warning;
it is visible in ordinary builds.

The outcome sought is a reviewed dependency disposition that removes
`proc-macro-error2` from the workspace graph without changing what
`#[stacksafe]` does to valid functions, followed by fresh normal gates and a
Cargo future-incompatibility report showing the warning is gone. If no
dependency change is approved, the truthful outcome is a blocked disposition
with the warning still disclosed and the issue left open.

## Constraints

- Do not suppress the warning through lint attributes, `--cap-lints`, Cargo
  configuration, or compiler flags.
- Do not change the pinned Rust toolchain (`rust-toolchain.toml` pins
  `1.99.1`) to mask the warning.
- Do not introduce a Git-sourced patch or an unreviewed vendored change.
- Do not edit `gpui`, `gpui-component`, `stacksafe`, or `stacksafe-macro`
  outside a reviewed change. `gpui 0.2.2` requires `stacksafe = "0.1"`
  (`^0.1`), so a plain lockfile update cannot select the 1.x line.
- A dependency change requires explicit approval from the issue owner
  (`@leynos`) before it is applied.
- Preserve the `#[stacksafe]` diagnostic messages and their spans:
  `"#[stacksafe] can only be applied to functions"` at the macro call site and
  `"#[stacksafe] does not support async functions"` at the async token.
- Keep generated code for valid functions unchanged.
- Do not retroactively rewrite completed ExecPlans or accepted ADRs. Add a
  cross-reference instead.
- Documentation must use en-GB-oxendict spelling and the repository style
  guide. Markdown prose is wrapped at 80 columns; code at 120 columns.
- No code file may exceed 400 lines.
- This ExecPlan is excluded from every change-size budget.

## Tolerances (exception triggers)

Budgets are measured against revision `740f414`, the baseline recorded in
`Progress`. The ExecPlan file itself is excluded from file and line counts.
Counts are net lines (additions minus deletions) unless stated otherwise.

- Files: more than 8 non-ExecPlan files changed.
- Code: more than 250 net lines of code, including the vendored macro copy.
- Configuration: more than 30 net lines across `Cargo.toml`,
  `Cargo.lock`, and any workspace-exclusion entry.
- Documentation: more than 350 net lines excluding ExecPlan files.
- Dependencies: any dependency change beyond `stacksafe-macro 0.1.4`, a
  published `stacksafe`/`stacksafe-macro` 0.1.5, or the `stacksafe` fixture
  dev-dependency in `crates/test_support` requires escalation.
- Interface: any public Gauss API change, or any change to `#[stacksafe]`
  output for valid functions.
- Iterations: three failed attempts at the same gate without a new,
  evidence-based hypothesis.
- Time: a single milestone exceeding four hours of active work.
- Ambiguity: more than one materially different disposition remains plausible
  and the owner has not chosen.

No comparable historical ExecPlan in this repository raised these budgets, so
the values are initial estimates from the surveyed diff (one ~100-line vendored
macro, one manifest patch, one ADR, and small guide and changelog edits).

## Risks

- Risk: the upstream `fast/stacksafe` project does not publish a 0.1.5
  maintenance release. Severity: high. Likelihood: high in the short term.
  Mitigation: request the backport, and offer a reviewed vendored
  `stacksafe-macro` 0.1.4 patch as an explicit fallback with a removal trigger.
- Risk: a vendored patch drifts from upstream and silently changes macro
  behaviour. Severity: medium. Likelihood: low. Mitigation: copy the published
  0.1.4 source, restrict changes to the error-reporting path, compare generated
  tokens, and record the upstream 1.0.2 diff as evidence.
- Risk: the patch leaks into unrelated dependency resolution or duplicates a
  crate. Severity: medium. Likelihood: low. Mitigation: `[patch.crates-io]`
  targets only `stacksafe-macro`; verify with `cargo tree -d` and a single
  `gpui 0.2.2` node.
- Risk: lockfile churn from unrelated updates.
  Severity: medium. Likelihood: medium. Mitigation: update with `--locked`
  where possible, accept only the targeted entries, and review the diff.
- Risk: the workspace cannot link GPUI on this host or the full gates are
  long-running under shared compile admission. Severity: low. Likelihood:
  medium. Mitigation: use fresh target directories, retain logs, and allow for
  queueing.
- Risk: `stacksafe` is used only by GPUI here, so the repository has no
  first-party stack-depth stress test to prove runtime equivalence. Severity:
  low. Likelihood: high. Mitigation: record the gap; rely on the unchanged
  generated code and existing GPUI behavioural tests.
- Risk: `make lint` cannot pass anywhere in this repository yet. Seven
  `bumpy-road-function` findings in `crates/gauss-core/src/model/` fail the
  Whitaker stage at the baseline, and the `CI` workflow at `main` revision
  `740f414` fails the same job. Severity: medium. Likelihood: certain.
  Mitigation: hold the candidate to "no finding beyond the seven recorded
  baseline findings" rather than to a zero exit, record the result without
  waiving it, and treat clearing the findings as a prerequisite owned by a
  separate change.

## Progress

- [x] (2026-11-18) Renamed the branch and set the Lody session title.
- [x] (2026-11-18) Recorded baseline revision, lock hash, host triple, tool
  versions, wrapper variables, and the locked dependency graph. The record is
  `/tmp/gauss-246-baseline-record.txt`.
- [x] (2026-11-18) Baseline future-incompatibility report captured.
      `cargo check --locked --workspace --all-targets --all-features --future-incompat-report`
      in `~/.cache/gauss-target/issue-246-baseline`
      exited 0 after 24m27s and recorded report id 1:
      `proc-macro-error2 v2.0.1`, E0365 at `src/lib.rs:494:13`, tracking
      `rust-lang/rust#127909`. The build log is
      `/tmp/check-baseline-gauss-issue-246.out` and the expanded report is
      `/tmp/future-incompat-baseline-gauss-issue-246.out`.
- [x] `EP-M1` Normal baseline gates retained (`make check-fmt`, `make lint`,
  `make test`), run sequentially through `scrutineer` at revision `8f652cf`:
  - `make check-fmt` failed initially on `mdtablefix --check` for the two new
    Markdown files. Applying `make fmt` and re-running made it pass at
    revision `a8e674e`.
  - `make lint` passed `cargo doc` and `cargo clippy` and failed in the
    Whitaker stage with seven `bumpy-road-function` findings in
    `crates/gauss-core/src/model/`. None of those paths is touched by this
    branch; the failure is pre-existing and is recorded, not waived or fixed
    here.
  - `make test` reached 1449 passing tests with no failures before its
    two-hour budget expired with three trybuild tests unreported and the
    doctest half not started, under a host-wide compile queue of roughly 270
    waiters. The remainder was re-run on the same tree with the gate's exact
    `RUSTFLAGS` and `RUSTDOCFLAGS`: the three trybuild tests passed after
    2927 s, 3062 s and 3112 s, and every doctest invocation passed (70 passed,
    0 failed, 3 ignored). Across both runs 1452 tests passed, 1 was skipped and
    none failed. The `make test` command itself was interrupted, so it is
    recorded as interrupted rather than passing, and the two runs are reported
    separately. The remainder log is
    `/tmp/test-remainder-gauss-issue-246-resolve-the-inherited-proc-macro-error2-future-incompatibility.out`.
- [x] (2026-11-18) `EP-M2` Upstream `fast/stacksafe#18` opened; ADR 007 drafted
  with the five dispositions, indexed in `docs/contents.md`, and submitted for
  the owner's decision.
- [ ] `EP-M3` Owner decision recorded; approved dependency change applied. The
  decision gate is outstanding: `Cargo.toml` and `Cargo.lock` remain untouched
  until the owner approves a disposition.
- [x] (2026-11-18) Branch pushed over SSH and tracked against `origin`; draft
  pull request [#317](https://github.com/leynos/gauss/pull/317) opened. First
  CodeRabbit pass returned two minor findings: index the ExecPlan in
  `docs/contents.md`, and make the `V2` non-vacuity check exercise both
  diagnostic branches. Both were adopted; see `D4`.
- [x] (2026-11-18) CI observed: pull request `#317` fails `build-test` in the
  `Lint` job on the same seven Whitaker findings. `main` at `740f414` fails the
  same job, so the failure is pre-existing and not caused by this branch, whose
  diff is three Markdown files.
- [x] (2026-11-18) Documentation gates run: `make markdownlint` reports 0
  issues in 78 files, and `make nixie` validates every Mermaid diagram and
  exits 0. The nixie log is
  `/tmp/nixie-gauss-issue-246-resolve-the-inherited-proc-macro-error2-future-incompatibility.out`.
- [x] (2026-11-18) Pull request body records the `build-test` result and the
  CodeRabbit passes with the corrections each one produced.
- [x] (2026-11-18) GPUI target names confirmed with `cargo metadata --no-deps`.
  Verifying the `-E` filter with `cargo nextest list` was attempted and stopped
  at its background budget under a saturated compile queue without producing
  output, so that check is recorded as unverified rather than passed.
- [ ] `EP-M4` Candidate validated with fresh report and gates; documentation
  and changelog updated; draft pull request opened.

## Surprises & discoveries

- Observation: `stacksafe-macro 1.0.2` already removed `proc-macro-error2`
  while remaining on Syn 2. It is therefore the exact template for a 0.1.5
  backport; 1.0.3 moved to Syn 3 and added further diagnostics that the issue
  excludes. Evidence: crates.io index dependency records and the published
  crate sources compared in `/tmp/gauss-246-crates`. Impact: a 0.1.5 patch can
  be a small diagnostic-only change rather than a port of the 1.0.3 diagnostics.
- Observation: no published `gpui` release above `0.2.2` exists on crates.io,
  and `gpui-component 0.5.1` requires `gpui = "0.2.2"`. Evidence: crates.io
  index for `gpui` and the `gpui-component 0.5.1` manifest. Impact: upgrading
  GPUI is not a viable way to reach `stacksafe 1.x` in this workspace today.
- Observation: other consumers already use the same vendored-macro remedy.
  Evidence: `IanTeda/Personal-Ledger#517` describes a `vendor/stacksafe-macro`
  patch for exactly this dependency path; `PORTALSURFER/gpui-embedded#3` moved
  its GPUI fork to `stacksafe 1.x`. Impact: the fallback is established prior
  art, not a novel or speculative mechanism.
- Observation: `fast/stacksafe` is actively maintained and dropped
  `proc-macro-error2` in pull request #12, released as `stacksafe-macro` 1.0.2
  on 8 June 2026. Its last push is 22 August 2026. It publishes only `main`;
  there is no 0.1 maintenance branch and no 0.1.5 release. Evidence: repository
  branches, tags, pulls and crate versions queried on 18 November 2026. Impact:
  a 0.1.5 backport is plausible rather than hypothetical, and the removal
  request has a concrete, recently released template.
- Observation: `gpui-component` 0.10.0, published 16 November 2026, no longer
  depends on `gpui`. It uses `gpui-base` and `gpui-pre` from
  `longbridge/gpui-kit`. Evidence: crates.io dependency records for
  `gpui-component 0.10.0`. Impact: upgrading `gpui-component` is not a route to
  `stacksafe 1.x`; it would replace the application's GPUI stack.

## Decision log

- D1. Decision: treat the upstream `stacksafe` 0.1.x backport as the preferred
  end state and the reviewed vendored `stacksafe-macro` 0.1.4 patch as the
  fallback candidate. Rationale: a 0.1.5 release would make the consumer fix
  lockfile-only and would require no repository source changes. Until it
  exists, only a path patch can remove the unmaintained crate while preserving
  `gpui 0.2.2`. Rejected option: patching `proc-macro-error2` itself. That
  crate is unmaintained under RUSTSEC-2026-0173, and the upstream fix is
  unmerged in an archived repository. Date/Author: 2026-11-18, implementation
  agent, pending owner approval.
- D2. Decision: do not upgrade `gpui` or `gpui-component`.
  Rationale: no published `gpui` release admits `stacksafe 1.x`, and
  `gpui-component 0.5.1` pins `gpui = "0.2.2"`. A fork or Git dependency would
  be an unreviewed source change and outside the issue's authority.
  Date/Author: 2026-11-18, implementation agent.
- D3. Decision: record the seven Whitaker findings and the truncated test run
  without fixing or waiving either. Rationale: repository guidance forbids
  waiving unrelated failures and forbids folding unrelated refactors into this
  change. `crates/gauss-core` is untouched by this branch, so the findings
  cannot be attributed to it, and the missing tests are a host-capacity
  artefact rather than a code defect. The remainder is re-run separately and
  reported with its own status. Date/Author: 2026-11-18, implementation agent.
- D4. Decision: adopt CodeRabbit's correction to `V2`. Rationale: the original
  method inspected sources and compiled the macro crate, neither of which
  applies `#[stacksafe]` to invalid input, so neither error branch ran and the
  non-vacuity claim was unsupported. The `trybuild` compile-fail fixtures need
  a test target that depends on `stacksafe`, so they belong to the approved
  change rather than to this documentation change, and must pass before a
  vendored macro change merges. Date/Author: 2026-11-18, implementation agent.
- D5. Decision: correct the ADR statement of expected lockfile churn. A path
  patch rewrites the `stacksafe-macro 0.1.4` entry as well as removing
  `proc-macro-error2` and `proc-macro-error-attr2`, because a path package has
  no checksum and lists fewer dependencies. Requiring "no other package change"
  would have made the accepted diff fail its own criterion. Date/Author:
  2026-11-18, implementation agent.
- D6. Decision: adopt three further review corrections. The `V2` fixtures have
  no host, because no workspace crate depends on `stacksafe`;
  `crates/test_support` becomes the host and gains a `stacksafe`
  dev-dependency, which adds a fourth `Cargo.lock` entry and a third manifest
  change to Option B's scope. `V6` stops treating `make lint` as a passing
  gate, because the repository cannot pass it yet; the acceptance set is
  `make check-fmt`, `make markdownlint` and `make test`. Rationale: each
  correction removes a way the plan could have claimed evidence it did not
  have. Date/Author: 2026-11-18, implementation agent.
- D7. Decision: keep `make lint` inside the candidate acceptance set and give
  it a comparative criterion, superseding the part of `D6` that dropped it.
  Rationale: removing the gate let a candidate that introduced new lint
  findings pass acceptance, which the ADR's technical requirement forbids. The
  gate cannot exit zero until the baseline findings are cleared by a separate
  change, so acceptance is "no finding beyond the seven recorded baseline
  findings" and the ADR records that its lint requirement is not yet met. The
  ADR requirement wording and `V6` were amended together so the two documents
  cannot disagree. Date/Author: 2026-11-18, implementation agent.
- D8. Decision: adopt four further review corrections. `V6` requires
  `cargo doc` and Clippy to pass and Whitaker to have run before the finding
  comparison applies, because the lint stages are a fail-fast chain and absent
  Whitaker output would otherwise read as no new findings. `EP-M3`'s
  registry-release path now carries the `V2` fixtures, since the diagnostics
  asserted are a property of the released macro rather than of the vendoring.
  The ADR removal trigger moves the fixture dev-dependency with the patch:
  leaving it on `^0.1.4` after a 1.x `stacksafe` arrives would pull the 0.1
  macro back in and either duplicate `stacksafe` or fail to resolve. Option A
  is consequently no longer described as lockfile-only, and the ADR states the
  lockfile consequence separately for each option. Date/Author: 2026-11-18,
  implementation agent.
- D9. Decision: adopt three review corrections about scope and verification
  boundaries. Option A now states that only the dependency update is
  lockfile-only while the approved change still touches the test manifest and
  adds fixtures. `EP-M3` gives separate acceptance for an approved dependency
  change and for the blocked or no-change path, because a blocked disposition
  selects no `stacksafe` release to fixture against. The ADR removal trigger
  requires the resolved graph, the diagnostics and a fresh
  future-incompatibility report to pass before the patch is deleted: a 0.1.5
  publication alone does not prove `proc-macro-error2` was removed, so the old
  wording could have restored the warning. Date/Author: 2026-11-18,
  implementation agent.

## Outcomes & retrospective

The dependency disposition is blocked, not decided. Option B is prepared
outside the repository, but a dependency change requires the owner's explicit
approval and none was given, so the recorded outcome is Option E: the E0365
warning stays disclosed, `Cargo.toml` and `Cargo.lock` are unchanged, and
`leynos/gauss#246` stays open.

Observed results:

- The baseline report still names `proc-macro-error2 v2.0.1` with E0365 at
  `src/lib.rs:494:13`. Nothing was suppressed, masked, or moved to a different
  toolchain.
- `make check-fmt`, `make markdownlint` and `make nixie` pass. `make lint`
  passes `cargo doc` and `cargo clippy` and fails in Whitaker on seven
  pre-existing findings in `crates/gauss-core/src/model/`, which this branch
  does not touch, so the ADR's lint requirement is recorded as not yet met.
  `make test` was interrupted by its two-hour budget; the remaining three
  trybuild tests and the doctest half then passed when run separately, so
  across both runs 1452 tests passed, 1 was skipped and none failed, but the
  command never reported a final status of its own.
- No Rust source, manifest, lockfile, or dependency changed. The diff is three
  Markdown files, one of which is this plan.

Lessons:

- The 0.1 line is abandoned in practice. `fast/stacksafe` publishes 1.0.x only,
  and `gpui 0.2.2` pins its consumers to `^0.1`, so no consumer-side registry
  update exists until one side moves.
- A 0.1.5 backport is the smallest unblocking change. It is requested as
  `fast/stacksafe#18`, and the `stacksafe-macro 1.0.2` change is its exact
  template, so the request asks for a known patch rather than a design.
- A plan's verification claims deserve the same review as code. The first `V2`
  wording asserted that both error branches were reachable without exercising
  either, and review caught it; the corrected obligation now names the fixtures
  and a negative control.

## Context and orientation

The workspace root is `Cargo.toml`, with members `crates/gauss-core`,
`crates/gauss-svg`, `crates/gauss-test-macros`, and `crates/test_support`. The
root package owns the GPUI application and its integration tests.

`gpui 0.2.2` is declared in the root manifest and is also required by
`gpui-component 0.5.1` and `rstest-bdd-harness-gpui 0.6.0`. Its manifest
requires `stacksafe = "0.1"`. `stacksafe 0.1.4` requires
`stacksafe-macro = "=0.1.4"`, and that macro crate requires
`proc-macro-error2 = "2"`.

The repository gate commands are defined in `Makefile`:

```sh
bash scripts/check-build-tools.sh
make check-fmt
make lint
make test
```

The relevant documentation is `docs/developers-guide.md` (build standard and
lint baseline), `docs/using-gpui-and-gpui-component.md` (GPUI version
constraints), `docs/adrs/` (decision records), and `docs/contents.md` (index).

The warning was first disclosed during the `rstest-bdd` 0.6.0 migration, whose
completed ExecPlan is `docs/execplans/adopt-rstest-bdd-v0-6-0.md`; that
historical document is not rewritten.

## Conformance basis

- Issue `leynos/gauss#246`.
- Repository instructions `AGENTS.md` and `docs/documentation-style-guide.md`.
- ADR 002 and ADR 003 for decision-record shape, and ADR 006 for dependency
  and build constraints.
- Upstream references: `GnomedDev/proc-macro-error-2#13`,
  `GnomedDev/proc-macro-error-2#14`, RUSTSEC-2026-0173, and
  `rust-lang/rust#127909`.

No Terms of Reference or technical-design revision exists for this change; the
issue and the repository guidance above are the governing artefacts.

## Verification plan

The change touches dependency resolution and procedural-macro diagnostics. It
introduces no runtime state, protocol, or arithmetic invariant. The obligations
are:

1. `V1` Valid-function expansion is unchanged. The macro must emit the same
   tokens for a valid function before and after the change. Method: compare the
   published 0.1.4 source with the candidate source, and exercise GPUI builds
   and tests that use `#[stacksafe]`. Non-vacuity: at least one valid function
   must be compiled through the patched macro; the GPUI dependency itself does
   this.
2. `V2` The call-site and async diagnostics are preserved.
   Method: `trybuild` compile-fail fixtures that apply `#[stacksafe]` to a
   non-function item and to an `async fn`, each asserting the expected message
   and the reported span, together with source inspection against the published
   1.0.2 diff. The fixtures belong to the approved change, because they need a
   test target that depends on `stacksafe`; both must pass before a vendored
   macro change is merged. Non-vacuity: the fixtures must fail against an
   intentionally altered message, and the baseline macro must satisfy them
   before the change is applied.
3. `V3` `proc-macro-error2` is absent from the resolved graph and lockfile
   after an approved change. Method: `cargo tree -i proc-macro-error2`,
   lockfile inspection, and `cargo metadata`. Non-vacuity: the same commands
   detect the crate on the baseline.
4. `V4` `gpui` remains a single node and the duplicate-crate set does not grow.
   Method: `cargo tree -i gpui` and `cargo tree -d`, compared with the baseline.
5. `V5` The future-incompatibility report no longer names
   `proc-macro-error2`. Method: a fresh
   `cargo check --locked --workspace --all-targets
   --all-features --future-incompat-report`
   in a new target directory, retaining literal output. Non-vacuity: the
   baseline run names the crate and the E0365 warning.
6. `V6` Workspace gates pass: `make check-fmt`, `make markdownlint`,
   `make nixie`, and `make test`, plus the EP-M4 GPUI tests with
   `--features test-support`. `make lint` is part of the acceptance set with a
   different criterion: the candidate's findings must not exceed the seven
   pre-existing `bumpy-road-function` findings in
   `crates/gauss-core/src/model/` recorded at the baseline. That gate cannot
   exit zero here. The `CI` workflow at `main` revision `740f414` fails the
   same Whitaker stage for the same reason, and clearing those findings is a
   prerequisite owned by a separate change that this plan neither performs nor
   waives. `make lint` runs `cargo doc`, then Clippy, then Whitaker as a
   fail-fast chain, so the comparison applies only once `cargo doc` and Clippy
   pass and Whitaker has actually run. A failure in either earlier stage fails
   the gate outright; absent Whitaker output is not "no new findings". The
   candidate's `make lint` output is therefore compared finding-by-finding
   against the baseline list, and the ADR's lint requirement is recorded as not
   yet met until that separate change lands. Non-vacuity: the recorded baseline
   failure is the witness that the gate is not silently skipped, a candidate
   that adds an eighth finding is rejected, and a candidate whose earlier stage
   fails cannot pass by reporting no findings.

Axioms: Cargo's `[patch.crates-io]` path replacement semantics, the published
crate contents, the Rust compiler's future-incompatibility reporting, and the
GPUI dependency requirement are treated as external interfaces.

Residual gap: there is no first-party stack-depth stress test, and validation
is Linux-only. These are recorded rather than claimed as covered.

## Milestones

### EP-M1: retained baseline

Record the revision, clean state, lock hash, host triple, tool and wrapper
versions, and the baseline report. Run the three repository gates. End state:
evidence retained under `/tmp`, no repository change beyond this plan.
Acceptance: the report names `proc-macro-error2 2.0.1`, E0365, and
`src/lib.rs:494`; the inverse tree matches the path above; gate logs exist.

### EP-M2: upstream coordination and disposition

Open a `fast/stacksafe` issue requesting a Syn 2-compatible 0.1.5 backport.
Record the GPUI `^0.1` constraint and link the upstream report and advisory.
Draft ADR 007 with the five options, recommendation, vendored-patch scope and
removal trigger, and remaining constraint. Submit it to the owner for a
decision. End state: an explicit approved, blocked, or no-change disposition.

### EP-M3: approved change

If a registry release is approved, update the lockfile and make the same
fixture change: add `stacksafe` at the approved release to the
`crates/test_support` dev-dependencies and add the `V2` fixtures, then
regenerate the lockfile accepting only the targeted entries. No Gauss source
change is needed on this path, but the fixtures are still required, because the
diagnostics they assert are a property of the released macro rather than of the
vendoring. If a vendored patch is approved, add `vendor/stacksafe-macro`, a path
`[patch.crates-io]` entry, and a `workspace.exclude` entry, and add
`stacksafe = "0.1.4"` to the `crates/test_support` dev-dependencies so the `V2`
fixtures can apply `#[stacksafe]`. If the disposition is blocked or no-change,
do not touch the manifest or lockfile and add no fixtures: there is no selected
`stacksafe` release for them to apply, and the planned outcome for that path is
the disclosed warning with the issue left open. End state: a candidate that
matches the approved disposition. Acceptance, for either approved path only: the
`V2` compile-fail fixtures exist in `crates/test_support` and pass for both
diagnostic branches against the `stacksafe` release the approved disposition
selects, with `cargo tree` showing no `proc-macro-error2` node. Acceptance, for
the blocked or no-change path: `Cargo.toml` and `Cargo.lock` are unchanged,
this plan records the disposition, and the issue stays open with the warning
disclosed.

### EP-M4: validation and delivery

Repeat the baseline report in a new target directory, run all gates, update the
ADR and GPUI guide and changelog if a dependency change landed, run the
Markdown and Mermaid gates, and open the draft pull request. The GPUI tests
named by the issue are the `gpui_draw_undo_bdd`, `gpui_shell_chrome_bdd` and
`gpui_shell_canvas_layout` targets of the root `gauss` package. Run them with:

```sh
RUSTFLAGS="-D warnings -Clink-arg=-fuse-ld=mold" \
  cargo nextest run --profile default --features test-support \
  -E 'binary(gpui_draw_undo_bdd) | binary(gpui_shell_chrome_bdd) | binary(gpui_shell_canvas_layout)'
```

Acceptance: every listed gate reports the result `V6` requires, `make lint`
adds no finding to the seven recorded in the baseline list, and the fresh
future-incompatibility report satisfies `V5`.

The three target names are confirmed from `cargo metadata --no-deps`, which
lists them as `test` targets of the root `gauss` package with no required
features. The `-E` filter expression itself has not been exercised: an attempt
to confirm the selection with `cargo nextest list` was stopped at its
background budget while the host compile queue was saturated, and produced no
output. Confirm the selection when this milestone runs, before treating a
passing run as covering the named tests.
