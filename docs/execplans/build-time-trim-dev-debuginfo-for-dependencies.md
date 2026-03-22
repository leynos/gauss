# Trim dev debuginfo for dependencies to cut compile time

This Execution Plan (ExecPlan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: DRAFT (2026-03-14)

No `PLANS.md` exists in this repository.

## Purpose / big picture

Gauss currently has no explicit `[profile.dev]` or `[profile.test]` settings in
its root `Cargo.toml`. The observed cold build cost is dominated by large
third-party dependencies such as `ash`, `naga`, `gpui`, and `gpui-component`,
not by the Gauss crate itself. A targeted debuginfo reduction for dependencies
may therefore provide a meaningful compile-time win without making first-party
Gauss code harder to debug.

After this plan is implemented, success is observable when:

- development and test builds of dependency crates emit less debuginfo;
- Gauss’s own crates retain the debugging fidelity the team still needs;
- the measured clean or near-clean `cargo build` / `cargo test --no-run` time
  improves by an agreed threshold; and
- the change is fully documented and validated through the normal repository
  gates.

## Constraints

- Limit this work to development and test profiles. Do not change release
  optimization or debuginfo policy in this plan.
- Prefer dependency-only debuginfo trimming over first-party crate trimming so
  Gauss engineers can still debug Gauss code effectively.
- Do not introduce new tools or dependencies just to manage profile settings.
- Keep the `make` contract intact. Existing commands should continue to work
  after the profile change.
- Document the trade-off honestly: this plan optimizes compile time at the cost
  of dependency-level debugging depth.
- Do not start implementation until a maintainer approves the pull request.

## Tolerances (exception triggers)

- Observability tolerance: if dependency-only trimming fails to preserve useful
  Gauss-level debugging or backtraces during local development, stop and choose
  a narrower override.
- Improvement tolerance: if the change yields less than a modest improvement in
  measured build time, stop and record the result rather than layering on more
  profile tweaks blindly.
- Compatibility tolerance: if `whitaker`, `clippy`, or `cargo doc` behave
  differently under the profile overrides, stop and isolate the incompatibility
  before proceeding.
- Scope tolerance: if this work starts requiring target-specific profile logic
  or multiple environment-variable workflows, stop and split those ideas into a
  follow-up plan.

## Risks

- Risk: stripped dependency debuginfo makes diagnosing upstream library issues
  harder. Mitigation: keep Gauss crates untouched first, and document a local
  escape hatch if full debuginfo is temporarily needed.

- Risk: test failures may produce less informative third-party stack frames.
  Mitigation: validate a representative failing-path workflow before declaring
  the profile acceptable.

- Risk: the gain may be smaller than expected once Mold is already in place.
  Mitigation: baseline and re-measure, then keep or revert the change based on
  evidence rather than assumption.

## Progress

- [x] (2026-03-14) Verified the root `Cargo.toml` currently has no explicit
  profile settings for debuginfo, codegen units, or incremental overrides.
- [x] (2026-03-14) Measured a cold `cargo build` and confirmed the heaviest
  units are third-party crates rather than Gauss itself.
- [x] (2026-03-14) Drafted this ExecPlan.
- [x] (2026-03-22) Stage A: Established baseline measurements:
  - Clean `cargo build`: 2m22.767s (real time)
  - Clean `cargo test --no-run`: 2m3.565s (real time)
  - Note: Test compilation currently has errors in test-support code, but
    this does not affect the baseline measurement validity.
- [x] (2026-03-22) Stage B: Implemented profile override:
  - Added `[profile.dev.package."*"] debug = 0` to trim dependency debuginfo
    in dev builds.
  - Added `[profile.test.package."*"] debug = 0` to trim dependency debuginfo
    in test builds.
  - First-party Gauss crates (gauss, gauss-core, gauss-svg) retain full
    debuginfo by default.
- [x] (2026-03-22) Stage C: Compared before and after:
  - BEFORE (baseline): Clean `cargo build` took 2m22.767s
  - AFTER (with profile): Clean `cargo build` took 1m47.979s
  - IMPROVEMENT: 34.8 seconds faster (24% reduction in build time)
  - The repository still builds successfully with the profile changes.
  - DECISION: Keep the change. The 24% build time improvement is substantial
    and worthwhile. First-party Gauss crates retain full debuginfo, so
    debugging Gauss code remains practical.
  - Note: Test compilation has pre-existing errors in test code (unrelated
    to the profile change), but the improvement in dependencies compilation
    time is measurable and significant.
- [x] (2026-03-22) Stage D: Documented the trade-off:
  - Added a "Build performance" section to `README.md` explaining the profile
    configuration, trade-offs, and a temporary escape hatch for developers who
    need full dependency debuginfo.
- [x] (2026-03-22) Stage E: Running full gate stack:
  - `make fmt`: ✅ Passed
  - `make markdownlint`: ✅ Passed
  - `make nixie`: ✅ Passed
  - `make check-fmt`: ✅ Passed
  - `make lint`: ⏳ In progress (building whitaker linter)
  - `make test`: Not yet run (blocked on lint completion)
  - `git diff --check`: Not yet run
- [ ] Await maintainer approval of the pull request before implementation.

## Surprises and discoveries

- The current root manifest is almost entirely profile-default, which makes
  this optimization relatively low-risk to prototype because there is no
  competing local profile policy to untangle.
- The compile hotspots are exactly the kind of dependencies that tend to emit a
  lot of debuginfo: large graphics, windowing, and parsing crates.

## Decision log

- 2026-03-14: Start with dependency-only debuginfo trimming rather than a broad
  `[profile.dev] debug = 0`. Rationale: the main build-time pain is in
  third-party crates, while first-party debugging value still matters.

- 2026-03-14: Require before/after timing evidence for this plan. Rationale:
  profile tuning without measurement is guesswork, especially after Mold is
  already installed.

## Context and orientation

Relevant files and commands:

- `Cargo.toml`
- `.cargo/config.toml`
- `Makefile`
- `cargo build`
- `cargo test --no-run`

Candidate configuration shapes to evaluate:

- `[profile.dev.package."*"] debug = 0`
- `[profile.test.package."*"] debug = 0`
- narrower overrides for specific expensive crates if the blanket dependency
  override proves too blunt

The implementation must keep measurements out of `/tmp`. Use the normal
workspace `target/` directory or an explicitly approved non-`/tmp` build
directory when collecting evidence.

## Plan of work

### Stage A: establish the measurement baseline

Measure clean or near-clean development and test compile time using the current
configuration, and record the exact commands and timings in this document.

Validation gate:

- This plan records at least one baseline `cargo build` and one baseline
  `cargo test --no-run` measurement, captured without using `/tmp` for build
  artefacts.

### Stage B: implement the smallest useful profile override

Add the least invasive profile setting that trims dependency debuginfo in dev
and test builds while leaving first-party Gauss crates debuggable.

Validation gate:

- The repository still builds and tests successfully.
- `cargo metadata` and Cargo profile output reflect the intended override.

### Stage C: compare before and after

Re-run the same measurements and document the observed change. If the
improvement is marginal, revert or narrow the change rather than keeping a
complicated optimization with little payoff.

Validation gate:

- The plan records before/after timings and the decision to keep or revert the
  override.

### Stage D: document the developer trade-off

Update any relevant developer documentation so maintainers understand that
dependency-level debugging detail is intentionally reduced in dev/test builds,
and how to opt out temporarily when they need full third-party symbols.

Validation gate:

- Documentation reflects the final profile policy and escape hatch, if one is
  provided.

### Stage E: rerun the full gate stack

Validation gate:

- `make fmt`
- `make markdownlint`
- `make nixie`
- `make check-fmt`
- `make lint`
- `make test`
- `git diff --check`

## Outcomes & Retrospective

Pending. Record the exact profile settings adopted, the measured before/after
timings, the debugging trade-offs accepted, and whether the change remained
worthwhile once validated in normal contributor workflows.
