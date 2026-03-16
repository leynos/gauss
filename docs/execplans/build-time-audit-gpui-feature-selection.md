# Audit GPUI (GPU-accelerated UI) feature selection for narrower build surfaces

This Execution Plan (ExecPlan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: DRAFT (2026-03-14)

No `PLANS.md` exists in this repository.

## Purpose / big picture

Gauss currently depends on `gpui = "0.2.2"` and `gpui-component = "0.5.1"`
using their default feature sets. The observed timing data shows both X11 and
Wayland-related Linux crates in the build graph, and the upstream `gpui`
manifest enables both backends in its default feature set. This plan exists to
determine whether Gauss can narrow those features safely for supported
platforms, or whether the defaults are still required.

Success is observable when:

- the repository has a documented decision on whether to keep or narrow GPUI
  features;
- any adopted feature narrowing preserves the platforms Gauss supports;
- developer, continuous integration (CI), and release commands still build the
  application correctly; and
- the decision is backed by measurement rather than intuition.

## Constraints

- This is an audit-first plan. It is valid for the final outcome to be
  “no feature change” if the evidence shows that narrowing is unsafe or not
  worth the risk.
- Preserve Gauss’s supported platforms and display-server expectations as
  documented in `docs/using-gpui-and-gpui-component.md` and any relevant CI
  configuration.
- Do not guess at GPUI’s feature semantics. Verify them against the pinned
  upstream manifest and by compiling the resulting targets.
- Keep feature selection explicit in `Cargo.toml` once a decision is made.
- Avoid adding custom feature flags to Gauss unless they are necessary to
  express target-specific backend choices cleanly.
- Do not start implementation until a maintainer approves the pull request.

## Tolerances (exception triggers)

- Platform tolerance: if narrowing features would drop support for a currently
  supported platform or display backend without explicit approval, stop and
  keep the default feature set.
- Evidence tolerance: if the team cannot verify the runtime or build impact on
  each supported target, stop at the audit phase and document the uncertainty
  instead of shipping an under-verified feature change.
- Complexity tolerance: if the feature selection needs a large matrix of
  target-specific dependency declarations or custom wrapper features, stop and
  decide whether the maintenance burden outweighs the compile-time win.
- Documentation tolerance: if the feature decision would invalidate current
  docs, CI assumptions, or local setup guidance in multiple places, stop and
  scope the required doc updates before making a partial manifest change.

## Risks

- Risk: removing an apparently unused GPUI feature may break runtime behaviour
  on a platform not represented in the main developer environment. Mitigation:
  verify supported target expectations before changing features and treat the
  audit as allowed to end with no change.

- Risk: compile-time wins may be small if the removed features are not the
  dominant cost centre after resolution. Mitigation: measure before and after
  rather than assuming dependency names map directly to meaningful savings.

- Risk: target-specific Cargo dependency tables may become harder to maintain.
  Mitigation: prefer the simplest explicit declaration that still preserves the
  supported platform set.

## Progress

- [x] (2026-03-14) Verified root dependencies currently use default
  `gpui` / `gpui-component` feature selection in `Cargo.toml`.
- [x] (2026-03-14) Verified the pinned upstream `gpui` manifest exposes default
  features including `font-kit`, `wayland`, `x11`, and `windows-manifest`.
- [x] (2026-03-14) Observed timing evidence showing expensive Linux GUI units
  such as `x11rb-protocol` in the current build graph.
- [x] (2026-03-14) Drafted this ExecPlan.
- [ ] Await user approval before implementation.

## Surprises & Discoveries

- The audit target is not just `gpui`; `gpui-component` also deserves review
  because it may bring optional feature surfaces that Gauss does not use.
- The repository already documents GPUI usage heavily, which means any feature
  decision must keep docs and practical build guidance synchronized.
- Because this work is feature-audit-first, a carefully documented “no change”
  outcome is a valid success case.

## Decision Log

- 2026-03-14: Treat this as an audit with an implementation branch point, not
  as a guaranteed manifest change. Rationale: platform support is more
  important than forcing a speculative compile-time optimization.

- 2026-03-14: Require both manifest inspection and actual Cargo build
  verification before narrowing features. Rationale: upstream features often
  have non-obvious transitive and platform-specific effects.

## Context and orientation

Relevant files and sources:

- `Cargo.toml`
- `.cargo/config.toml`
- `Makefile`
- `docs/using-gpui-and-gpui-component.md`
- the pinned upstream manifests for `gpui 0.2.2` and `gpui-component 0.5.1`
  under the local Cargo registry

Questions the audit must answer:

1. Which platforms are actually supported by Gauss today?
2. Which GPUI backends are required for those platforms?
3. Can Gauss replace the current default dependency declaration with explicit
   features without breaking those platforms?
4. Does the before/after build evidence justify keeping the narrower feature
   declaration?

## Plan of work

### Stage A: verify the supported platform matrix

Read current docs and CI configuration to determine which operating systems and
display backends Gauss intends to support today. Record the answer in this plan.

Validation gate:

- This document lists the supported platform matrix and the evidence used to
  justify it.

### Stage B: inspect the pinned upstream feature surfaces

Read the local Cargo-registry manifests for the pinned `gpui` and
`gpui-component` versions and record the default and optional features relevant
to Gauss.

Validation gate:

- This plan records the relevant upstream features and which ones Gauss appears
  to use today.

### Stage C: prototype explicit feature declarations

On an implementation branch, replace the current default dependency
declarations with explicit target-aware feature declarations, then run focused
builds for the supported platform set.

Validation gate:

- `cargo build`
- `cargo test --no-run`
- any supported-target cross-checks agreed by the team

### Stage D: decide keep, narrow, or revert

If the narrower feature set preserves supported platforms and produces a useful
compile-time improvement, keep it and document the policy. Otherwise, restore
the current default dependency declarations and document why the audit ended
with no change.

Validation gate:

- The final state is explicitly justified in this plan and in any updated docs.

### Stage E: rerun full repository gates

Validation gate:

- `make fmt`
- `make markdownlint`
- `make nixie`
- `make check-fmt`
- `make lint`
- `make test`
- `git diff --check`

## Outcomes & Retrospective

Pending. Record the verified platform matrix, the final dependency declaration,
the before/after evidence, and whether the audit ended with a narrowed feature
set or an intentional no-change decision.
