# Resolve the inherited proc-macro-error2 future incompatibility (#246)

This ExecPlan (execution plan) is a living document. The sections `Constraints`,
`Tolerances`, `Risks`, `Progress`, `Surprises & discoveries`, `Decision log`,
`Outcomes & retrospective`, `Conformance basis`, and `Verification plan` must
be kept up to date as work proceeds.

Status: IN PROGRESS

Branch: `issue-246-resolve-the-inherited-proc-macro-error2-future-incompatibility`

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
- Dependencies: any dependency change beyond `stacksafe-macro 0.1.4` (or a
  published `stacksafe`/`stacksafe-macro` 0.1.5) requires escalation.
- Interface: any public Gauss API change, or any change to `#[stacksafe]`
  output for valid functions.
- Iterations: three failed attempts at the same gate without a new,
  evidence-based hypothesis.
- Time: a single milestone exceeding four hours of active work.
- Ambiguity: more than one materially different disposition remains plausible
  and the owner has not chosen.

No comparable historical ExecPlan in this repository raised these budgets, so
the values are initial estimates from the surveyed diff (one ~100-line
vendored macro, one manifest patch, one ADR, and small guide and changelog
edits).

## Risks

- Risk: the upstream `fast/stacksafe` project does not publish a 0.1.5
  maintenance release.
  Severity: high. Likelihood: high in the short term. Mitigation: request the
  backport, and offer a reviewed vendored `stacksafe-macro` 0.1.4 patch as an
  explicit fallback with a removal trigger.
- Risk: a vendored patch drifts from upstream and silently changes macro
  behaviour.
  Severity: medium. Likelihood: low. Mitigation: copy the published 0.1.4
  source, restrict changes to the error-reporting path, compare generated
  tokens, and record the upstream 1.0.2 diff as evidence.
- Risk: the patch leaks into unrelated dependency resolution or duplicates a
  crate.
  Severity: medium. Likelihood: low. Mitigation: `[patch.crates-io]` targets
  only `stacksafe-macro`; verify with `cargo tree -d` and a single
  `gpui 0.2.2` node.
- Risk: lockfile churn from unrelated updates.
  Severity: medium. Likelihood: medium. Mitigation: update with `--locked`
  where possible, accept only the targeted entries, and review the diff.
- Risk: the workspace cannot link GPUI on this host or the full gates are
  long-running under shared compile admission.
  Severity: low. Likelihood: medium. Mitigation: use fresh target directories,
  retain logs, and allow for queueing.
- Risk: `stacksafe` is used only by GPUI here, so the repository has no
  first-party stack-depth stress test to prove runtime equivalence.
  Severity: low. Likelihood: high. Mitigation: record the gap; rely on the
  unchanged generated code and existing GPUI behavioural tests.

## Progress

- [x] (2026-11-18) Renamed the branch and set the Lody session title.
- [x] (2026-11-18) Recorded baseline revision, lock hash, host triple, tool
  versions, wrapper variables, and the locked dependency graph. The record is
  `/tmp/gauss-246-baseline-record.txt`.
- [ ] `EP-M1` Baseline future-incompatibility report and normal gates retained.
  The `cargo check --future-incompat-report` run is in progress in
  `~/.cache/gauss-target/issue-246-baseline`, logging to
  `/tmp/check-baseline-gauss-issue-246.out`. It is queued behind other agents'
  builds under shared compile admission.
- [ ] `EP-M2` Upstream `fast/stacksafe` issue opened; ADR 007 drafted with a
  disposition for owner approval. `fast/stacksafe#18` is open; ADR 007 is
  indexed in `docs/contents.md` and awaits the owner's decision.
- [ ] `EP-M3` Owner decision recorded; approved dependency change applied.
- [ ] `EP-M4` Candidate validated with fresh report and gates; documentation
  and changelog updated; draft pull request opened.

## Surprises & discoveries

- Observation: `stacksafe-macro 1.0.2` already removed `proc-macro-error2`
  while remaining on Syn 2. It is therefore the exact template for a 0.1.5
  backport; 1.0.3 moved to Syn 3 and added further diagnostics that the issue
  excludes.
  Evidence: crates.io index dependency records and the published crate
  sources compared in `/tmp/gauss-246-crates`.
  Impact: a 0.1.5 patch can be a small diagnostic-only change rather than a
  port of the 1.0.3 diagnostics.
- Observation: no published `gpui` release above `0.2.2` exists on crates.io,
  and `gpui-component 0.5.1` requires `gpui = "0.2.2"`.
  Evidence: crates.io index for `gpui` and the `gpui-component 0.5.1`
  manifest.
  Impact: upgrading GPUI is not a viable way to reach `stacksafe 1.x` in this
  workspace today.
- Observation: other consumers already use the same vendored-macro remedy.
  Evidence: `IanTeda/Personal-Ledger#517` describes a `vendor/stacksafe-macro`
  patch for exactly this dependency path; `PORTALSURFER/gpui-embedded#3` moved
  its GPUI fork to `stacksafe 1.x`.
  Impact: the fallback is established prior art, not a novel or speculative
  mechanism.
- Observation: `fast/stacksafe` is actively maintained and dropped
  `proc-macro-error2` in pull request #12, released as `stacksafe-macro` 1.0.2
  on 8 June 2026. Its last push is 22 August 2026. It publishes only `main`;
  there is no 0.1 maintenance branch and no 0.1.5 release.
  Evidence: repository branches, tags, pulls and crate versions queried on
  18 November 2026.
  Impact: a 0.1.5 backport is plausible rather than hypothetical, and the
  removal request has a concrete, recently released template.
- Observation: `gpui-component` 0.10.0, published 16 November 2026, no longer
  depends on `gpui`. It uses `gpui-base` and `gpui-pre` from
  `longbridge/gpui-kit`.
  Evidence: crates.io dependency records for `gpui-component 0.10.0`.
  Impact: upgrading `gpui-component` is not a route to `stacksafe 1.x`; it
  would replace the application's GPUI stack.

## Decision log

- D1. Decision: treat the upstream `stacksafe` 0.1.x backport as the preferred
  end state and the reviewed vendored `stacksafe-macro` 0.1.4 patch as the
  fallback candidate.
  Rationale: a 0.1.5 release would make the consumer fix lockfile-only and
  would require no repository source changes. Until it exists, only a path
  patch can remove the unmaintained crate while preserving `gpui 0.2.2`.
  Rejected option: patching `proc-macro-error2` itself. That crate is
  unmaintained under RUSTSEC-2026-0173, and the upstream fix is unmerged in an
  archived repository.
  Date/Author: 2026-11-18, implementation agent, pending owner approval.
- D2. Decision: do not upgrade `gpui` or `gpui-component`.
  Rationale: no published `gpui` release admits `stacksafe 1.x`, and
  `gpui-component 0.5.1` pins `gpui = "0.2.2"`. A fork or Git dependency
  would be an unreviewed source change and outside the issue's authority.
  Date/Author: 2026-11-18, implementation agent.

## Outcomes & retrospective

To be completed when the disposition is known. The final entry must state
whether `proc-macro-error2` remains, which option was approved, the gate
results, and any remaining upstream constraint.

## Context and orientation

The workspace root is `Cargo.toml`, with members `crates/gauss-core`,
`crates/gauss-svg`, `crates/gauss-test-macros`, and `crates/test_support`.
The root package owns the GPUI application and its integration tests.

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
   tokens for a valid function before and after the change.
   Method: compare the published 0.1.4 source with the candidate source, and
   exercise GPUI builds and tests that use `#[stacksafe]`. Non-vacuity: at
   least one valid function must be compiled through the patched macro; the
   GPUI dependency itself does this.
2. `V2` The call-site and async diagnostics are preserved.
   Method: source inspection against the published 1.0.2 diff, plus compile
   checks of the vendored macro crate. Non-vacuity: the two error branches are
   reachable from a non-function item and an async function.
3. `V3` `proc-macro-error2` is absent from the resolved graph and lockfile
   after an approved change.
   Method: `cargo tree -i proc-macro-error2`, lockfile inspection, and
   `cargo metadata`. Non-vacuity: the same commands detect the crate on the
   baseline.
4. `V4` `gpui` remains a single node and the duplicate-crate set does not grow.
   Method: `cargo tree -i gpui` and `cargo tree -d`, compared with the
   baseline.
5. `V5` The future-incompatibility report no longer names
   `proc-macro-error2`.
   Method: a fresh `cargo check --locked --workspace --all-targets
   --all-features --future-incompat-report` in a new target directory,
   retaining literal output. Non-vacuity: the baseline run names the crate and
   the E0365 warning.
6. `V6` Workspace gates pass: `make check-fmt`, `make lint`, and `make test`,
   plus the named GPUI behavioural tests with `--features test-support`.

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

If a registry release is approved, update the lockfile only. If a vendored
patch is approved, add `vendor/stacksafe-macro`, a path `[patch.crates-io]`
entry, and a `workspace.exclude` entry. If the disposition is blocked or
no-change, do not touch the manifest or lockfile. End state: a candidate that
matches the approved disposition.

### EP-M4: validation and delivery

Repeat the baseline report in a new target directory, run all gates, run the
named GPUI tests, update the ADR and GPUI guide and changelog if a dependency
change landed, run Markdown and Mermaid gates, and open the draft pull request.
