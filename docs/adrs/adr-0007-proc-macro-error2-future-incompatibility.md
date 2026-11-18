# Architectural decision record (ADR) 007: Resolve the inherited proc-macro-error2 future incompatibility

## Status

Proposed (2026-11-18) — awaiting the owner's dependency disposition before any
manifest or lockfile change.

## Date

2026-11-18.

## Context and problem statement

Gauss inherits a Rust future-incompatibility warning from an unmaintained
transitive crate. The locked path is:

```plaintext
gauss -> gpui 0.2.2 -> stacksafe 0.1.4 -> stacksafe-macro 0.1.4
      -> proc-macro-error2 2.0.1 -> proc-macro-error-attr2 2.0.0
```

The compiler reports:

```plaintext
warning[E0365]: extern crate `proc_macro` is private and cannot be re-exported
   --> .../proc-macro-error2-2.0.1/src/lib.rs:494:13
    |
494 |     pub use proc_macro;
    |             ^^^^^^^^^^
    |
    = warning: this was previously accepted by the compiler but is being phased
      out; it will become a hard error in a future release!
    = note: for more information, see issue #127909
```

The repository first disclosed the warning during the `rstest-bdd` 0.6.0
migration. It is a warning on the pinned Rust 1.99.1 toolchain, not an observed
hard failure, and the report does not name the first compiler release that will
reject it.[^1] [`GnomedDev/proc-macro-error-2#13`][upstream-issue] tracks the
defect; the proposed fix in [pull request #14][upstream-pr] was never merged,
and the repository was archived on 7 June 2026.

`proc-macro-error2` is also unmaintained. RUSTSEC-2026-0173 records that the
author recommends migration and lists no patched version.[^2] The Rust 1.x
crate has already addressed both points: `stacksafe-macro 1.0.2` removed
`proc-macro-error2` while staying on Syn 2, and `1.0.3` moved to Syn 3 and
added further diagnostics.

`fast/stacksafe` itself is active: its last push was 22 August 2026, and it
publishes `stacksafe` and `stacksafe-macro` 1.0.0 through 1.0.3. It has no 0.1
maintenance branch and no 0.1.5 release, so the 0.1 line stopped at
`stacksafe-macro 0.1.4` (26 June 2025). A request for a 0.1.5 backport is
tracked as [`fast/stacksafe#18`][stacksafe-issue].

The consumer constraint is the reason this is not a lockfile-only update.
`gpui 0.2.2` requires `stacksafe = "0.1"` (`^0.1`), so the resolver cannot
select any 1.x release. The latest published `gpui` on crates.io remains 0.2.2,
and `gpui-component 0.5.1` requires `gpui = "0.2.2"`. Zed has merged the
`stacksafe` bump in its own repository, but has not published a newer crate;
its maintainers recommend a pinned Git revision until automated releases
land.[^3]

### Baseline evidence

Recorded on 18 November 2026 before any manifest or lockfile change:

- Revision `740f4145fce689b1b1cf1daa00bba3c067403189`; `Cargo.lock` SHA-256
  `a40d4cc832b309299146fe05648a9d30906738771aaf9015e0ba4ab5e4de3c04`.
- Host `x86_64-unknown-linux-gnu`; `rustc 1.99.1 (58cf8f9c3 2026-10-13)` and
  `cargo 1.99.1 (5f94df478 2026-08-27)`. Builds pass through the shared compile
  admission wrapper (`RUSTC_WRAPPER=build-limits-rustc`,
  `RUSTDOC=build-limits-rustdoc`) with `RUSTFLAGS` unset.
- `bash scripts/check-build-tools.sh` exits 0 with `mold` 2.41.0, a usable
  `clang`, and the 1.92.0, 1.99.1 and `nightly-2026-08-07` toolchains present.
- The locked inverse tree resolves `proc-macro-error2 v2.0.1` through
  `stacksafe-macro v0.1.4` (`proc-macro`), `stacksafe v0.1.4`, and
  `gpui v0.2.2`, which is reached by `gauss`, `gpui-component v0.5.1`, and
  `rstest-bdd-harness-gpui v0.6.0`.
- `proc-macro-error2 2.0.1` requires `proc-macro-error-attr2 2.0.0`,
  `proc-macro2`, `quote`, and `syn 2.0.111`. The unrelated
  `proc-macro-error3 3.1.1` and `proc-macro-error-attr3 3.1.1` pair stays in
  the graph for `rstest-bdd-macros`.
- The baseline
  `cargo check --locked --workspace --all-targets --all-features --future-incompat-report`
  run finishes in 24m27s in a fresh target directory, exits 0, and warns that
  `proc-macro-error2 v2.0.1` contains code the compiler will reject. Cargo
  assigns report id 1, and `cargo report future-incompatibilities --id 1`
  prints E0365 at `proc-macro-error2-2.0.1/src/lib.rs:494:13`, naming
  `rust-lang/rust#127909` as the tracking issue.

## Decision drivers

- Remove a warning that will become a hard error without changing compiler
  versions or suppressing diagnostics.
- Avoid carrying an unmaintained crate with no patched release.
- Keep `gpui 0.2.2` and `gpui-component 0.5.1`, the versions the application is
  built and tested against.
- Keep `#[stacksafe]` output for valid functions unchanged.
- Prefer a lockfile-only consumer change when a supported release exists.
- Keep any interim mechanism reviewed, narrowly scoped, and easy to remove.

## Requirements

### Functional requirements

- A normal workspace build must emit no E0365 warning attributed to
  `proc-macro-error2`.
- Effective stack growth for valid `#[stacksafe]` functions must not change.
- The messages `"#[stacksafe] can only be applied to functions"` (at the macro
  call site) and `"#[stacksafe] does not support async functions"` (at the
  async token span) must be preserved.
- `gpui 0.2.2` must remain the resolved GPUI node.

### Technical requirements

- No lint suppression, `--cap-lints`, `RUSTFLAGS` masking, or toolchain change.
- No unreviewed Git patch.
- The workspace must keep passing `make check-fmt`, `make lint`, and
  `make test` on Linux with all targets and features.
- The preserved messages and spans are demonstrated by `trybuild` compile-fail
  fixtures, added with the approved change, that cover a non-function item and
  an `async fn`.
- Any dependency change requires the owner's explicit approval.

## Options considered

### Option A: upstream `stacksafe` 0.1.5 backport

Ask `fast/stacksafe` to publish 0.1.5 releases of `stacksafe` and
`stacksafe-macro` that backport the `stacksafe-macro 1.0.2` diagnostic change
to the 0.1 line. The change removes `proc-macro-error2`, keeps Syn 2, and
preserves generated code. Because `gpui 0.2.2` already admits `^0.1`, the
consumer fix would then be a lockfile-only update. This is the smallest
supported end state and requires no source changes in Gauss.

The option depends on an upstream release that does not exist today. It also
needs a maintenance branch: upstream publishes only `main`, and the 0.1.4
release predates the 1.0 line. [`fast/stacksafe#18`][stacksafe-issue] requests
the backport and offers to prepare the 0.1 pull request.

### Option B: reviewed vendored `stacksafe-macro` 0.1.4 patch

Add the published `stacksafe-macro 0.1.4` source under
`vendor/stacksafe-macro`, a path-based `[patch.crates-io]` entry, and a root
`workspace.exclude` entry. Limit the source change to the diagnostic path,
mirroring `stacksafe-macro 1.0.2`: replace `#[proc_macro_error]`,
`abort_call_site!`, and `abort!` with `syn::Error` and `to_compile_error()`,
and keep the rest of the macro unchanged.

This removes `proc-macro-error2` immediately and remains compatible with
`gpui 0.2.2`. It is an interim consumer change: the repository owns a copy of
third-party source until upstream publishes 0.1.5. Another consumer carries the
same remedy for this dependency path and has an open issue tracking its
removal.[^4]

### Option C: upgrade `gpui` and `gpui-component`

Upgrade to a GPUI release whose `stacksafe` requirement admits 1.x. No such
release is published: `gpui 0.2.2` is the latest crate on crates.io, and
`gpui-component 0.5.1` pins it. Later `gpui-component` releases do not help
either: 0.10.0 (16 November 2026) depends on `gpui-base` and `gpui-pre` from
[`longbridge/gpui-kit`](https://github.com/longbridge/gpui-kit), not on `gpui`,
so adopting it would replace the application's GPUI stack rather than update a
dependency. A Git dependency on Zed would be a large, unreviewed source change
outside this issue's authority.

### Option D: patch `proc-macro-error2` itself

Vendor a fixed `proc-macro-error2` that makes the `extern crate` public. This
retains an unmaintained crate under RUSTSEC-2026-0173, keeps the dependency in
the graph, and patches code that upstream has abandoned. It treats a symptom
rather than removing the dependency.

### Option E: no change

Disclose the warning, leave the issue open, and wait for upstream. This is
truthful but does not meet the issue's acceptance criterion that the warning is
gone.

| Option | Removes the crate | Needs upstream | Source change | Fits `gpui 0.2.2` |
| ------ | ----------------- | -------------- | ------------- | ----------------- |
| A      | Yes, at 0.1.5     | Yes            | None          | Yes               |
| B      | Yes, immediately  | No             | Vendored      | Yes               |
| C      | Yes               | Yes            | Large         | No release        |
| D      | No                | No             | Vendored      | Yes               |
| E      | No                | No             | None          | Yes               |

_Table 1: Dependency disposition options._

## Decision outcome / proposed direction

The preferred end state is Option A: an upstream 0.1.5 maintenance release,
followed by a lockfile-only consumer update. Because no such release exists,
the owner is asked to approve Option B as the interim consumer change, with
Option A as the removal trigger. If the owner does not approve B, the recorded
disposition is Option E and the issue remains open with the warning disclosed.

Option B is limited to the diagnostic path:

- the crate name and version remain `stacksafe-macro 0.1.4`, so the
  `stacksafe 0.1.4` companion pin `=0.1.4` still resolves;
- the `proc-macro-error2` dependency is removed;
- `syn` stays on 2.x with the `full` feature;
- valid-function generated code is unchanged;
- both diagnostics keep their previous messages and spans.

Removal trigger: delete the vendored crate, the `[patch.crates-io]` entry, and
the `workspace.exclude` entry once `stacksafe` and `stacksafe-macro` 0.1.5 are
published and the lockfile resolves to them without the patch.

## Consequences

- Option B makes the workspace depend on a reviewed local copy of a
  third-party macro crate until upstream releases.
- `Cargo.lock` loses `proc-macro-error2` and `proc-macro-error-attr2`; no other
  package should change.
- The `stacksafe` and `stacker` runtime behaviour is untouched.
- GPUI remains pinned at 0.2.2, so the upstream GPUI release constraint
  remains open.

## Known risks and limitations

- Validation is Linux-only. The macro is platform-independent, but macOS and
  Windows builds are not exercised here.
- The repository has no first-party stack-depth stress test. GPUI is the only
  consumer of `#[stacksafe]`, so runtime equivalence rests on unchanged
  generated code and the existing GPUI behavioural tests.
- The change affects only the error path. Valid functions are expanded by the
  same `quote!` block as before.
- `stacker` and `psm` are unchanged.
- Until upstream publishes 0.1.5, the vendored copy must be reviewed whenever
  the dependency graph changes.
- Any future `gpui` release that admits `stacksafe 1.x` is a preferred removal
  route; track Zed issue #66275 and discussion #55271.[^3]

## References

- Issue: [`leynos/gauss#246`](https://github.com/leynos/gauss/issues/246).
- Backport request: [`fast/stacksafe#18`][stacksafe-issue].
- Upstream report: [`GnomedDev/proc-macro-error-2#13`][upstream-issue].
- Proposed upstream fix: [`GnomedDev/proc-macro-error-2#14`][upstream-pr].
- Advisory:
  [RUSTSEC-2026-0173](https://rustsec.org/advisories/RUSTSEC-2026-0173.html).
- Compiler context:
  [`rust-lang/rust#127909`](https://github.com/rust-lang/rust/issues/127909).
- Prior consumer remedy:
  [`IanTeda/Personal-Ledger#517`](https://github.com/IanTeda/Personal-Ledger/issues/517).
- 0.1.4 source commit:
  `a5c029b7d3a5fe6771ecd79266b6e21313cfa174` in `fast/stacksafe`.

[^1]: The warning was disclosed in
    `docs/execplans/adopt-rstest-bdd-v0-6-0.md` as an out-of-scope transitive
    warning emitted during `make lint`. Issue #246 reports it on Rust 1.92.0
    validation; the repository now pins `1.99.1` in `rust-toolchain.toml`. No
    compiler change is part of this decision.

[^2]: RustSec Advisory Database, "RUSTSEC-2026-0173: proc-macro-error2 is
    unmaintained", <https://rustsec.org/advisories/RUSTSEC-2026-0173.html>.

[^3]: Zed merged the `stacksafe` bump in
    [`zed-industries/zed#62468`](https://github.com/zed-industries/zed/pull/62468)
    but has not published a newer `gpui` crate. See
    [`zed-industries/zed#66275`](https://github.com/zed-industries/zed/issues/66275)
    and [discussion #55271](https://github.com/zed-industries/zed/discussions/55271).

[^4]: [`IanTeda/Personal-Ledger#517`](https://github.com/IanTeda/Personal-Ledger/issues/517),
    opened 11 November 2026, tracks removing exactly such a
    `vendor/stacksafe-macro` patch. It was added for the same
    `gpui -> stacksafe 0.1 -> stacksafe-macro` path to clear E0365.

[upstream-issue]: https://github.com/GnomedDev/proc-macro-error-2/issues/13
[upstream-pr]: https://github.com/GnomedDev/proc-macro-error-2/pull/14
[stacksafe-issue]: https://github.com/fast/stacksafe/issues/18

## Addenda

- **2026-11-18:** Decision record created with status Proposed. The baseline
  evidence, owner decision, candidate evidence, and gate results are added
  after review.
- **2026-11-18:** Opened [`fast/stacksafe#18`][stacksafe-issue] requesting the
  Syn 2-compatible 0.1.5 backport. The request offers the 0.1 maintenance pull
  request, records the `gpui 0.2.2` `^0.1` constraint, and links
  [`GnomedDev/proc-macro-error-2#13`][upstream-issue] and RUSTSEC-2026-0173.
- **2026-11-18:** No new Zed issue was opened. `zed-industries/zed#66275`
  already covers the missing `gpui` publication. Its maintainer reply on
  30 October 2026 states that automated GPUI releases are being prepared, that
  a pinned Git revision is required until then, and that follow-up belongs in
  discussion #55271 rather than a new issue. That thread is the recorded
  follow-up trigger for revisiting Option C.
- **2026-11-18:** The baseline evidence was captured and the repository gates
  were run. `make check-fmt` passes after `make fmt`. `make lint` reaches
  Whitaker and reports seven pre-existing `bumpy-road-function` findings in
  `crates/gauss-core/src/model/`, none of which this branch touches.
  `make test` reached 1449 passing tests with no failures before its two-hour
  budget expired, with three trybuild tests unreported and the doctests not
  started; the remainder was re-run separately. The decision gate is still
  outstanding, so this branch applies no manifest or lockfile change and the
  status remains Proposed.
