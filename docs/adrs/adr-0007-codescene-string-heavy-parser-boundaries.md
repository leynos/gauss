# Architectural decision record (ADR) 007: Scope CodeScene string-heavy findings in text-parser contract modules

## Status

Accepted on 18 November 2026. `String Heavy Function Arguments` is repaired at
genuine parse boundaries by validating newtypes, and is exempted by exact path
for workflow-contract modules whose parameters are the text they decode. No
inline directive, threshold change, global profile edit, or cosmetic wrapper is
authorized.

## Date

2026-11-18.

## Context and problem statement

CodeScene reports `String Heavy Function Arguments` when too many of a module's
function parameters are strings. The factor is the module-level expression of
`Primitive Obsession`: the recommended remedy is a domain type that carries the
validation and meaning of the value rather than a raw primitive.[^1]

Issue [#257][issue-257] recorded ten findings in `tests/build_standard/` at the
39% threshold. The modules are text decoders and their contract fixtures: they
read YAML keys, Makefile lines, shell fragments, executable argv and GNU Make
trace rows. Those parameters must keep arbitrary spellings so the suite can
reject malformed input and prove evaluated Make behaviour. Wrapping every slice
or line would add indirection without an invariant and would hide the raw
fixtures that make rejection tests meaningful.

The Rust baseline train also forbids the convenient escape hatches: inline
suppression, relaxed thresholds or profiles, cosmetic one-field wrappers,
argument bundles, and moving source only to change the measured population. The
issue therefore asked for either a source correction with a new invariant, or
an authoritative interpretation of the rule that stays inside those limits.

This decision is recorded because the disposition is a policy choice about a
CodeScene factor, not an implementation detail that the code states by itself.

## Decision drivers

- Preserve arbitrary spellings for rejection tests, near misses and evaluated
  Make behaviour.
- Keep the factor where a domain value is genuinely missing, and close those
  boundaries with a constructor that cannot be bypassed.
- Keep every deviation visible, version-controlled, path-scoped and reviewable.
- Add no inline directive, no threshold override, and no global profile change.
- Let a new module, or a renamed module, report the factor again rather than
  inherit an exemption silently.

## Options considered

1. Wrap every string parameter, or introduce argument bundles, to move each
   ratio below 39%. Rejected: this is metric-only refactoring. It adds no
   invariant, changes the decoder boundary that the tests exist to exercise,
   and removes raw rejection fixtures from direct use.
2. Relax the threshold or change the profile for all test code. Rejected: it
   hides future genuine findings in every module and applies beyond the paths
   this issue assessed. The issue authorizes no profile change.
3. Add `@codescene(disable:"String Heavy Function Arguments")` directives.
   Rejected: directives apply only to function-level smells, so they cannot
   address this module-level factor. They would also be an inline suppression,
   which the train forbids.
4. Repair genuine parse boundaries with parse-only newtypes, and exempt the
   remaining text-decoder modules by exact path in the repository rule file.
   This is the option taken.

## Decision outcome

Genuine parse boundaries now carry validating newtypes:

- `MakeGoal` and `ModeSelector` in `ci_reachability_make.rs` are constructed
  only by their parsers, so `MakeRequest` cannot carry an unreviewed goal or an
  unresolved `MODE` value.
- `RunnerLabel` in `ci_reachability_runner.rs` is constructed only from a
  literal label, so host classification never re-validates a raw scalar.

The seven remaining modules are exempted in
`.codescene/code-health-rules.json`. Each rule set:

- names exactly one repository-relative path, with no glob metacharacter;
- carries a `matching_content_path_doc` that states why the string is the
  domain for that module and cites issue #257;
- overrides only `String Heavy Function Arguments`, at weight `0.0`;
- declares no threshold override.

Exact paths are used deliberately. CodeScene documents `matching_content_path`
as a glob relative to the repository root and presents every override in the
analysis Scope view.[^2] An exact path keeps the override bound to the module
the issue assessed: a new module still reports the factor, and a renamed module
stops matching and reports again. Widening the set requires a source review
that changes both the rule file and the contract test described below.

### What this decision is not

- It is not a waiver of the ten findings. The rules file records which paths are
  exempt and why, and CodeScene continues to show the overrides in Scope.
- It is not a function-level suppression. No `@codescene` directive is added,
  and the factor is module-level so a directive could not apply.
- It is not a threshold or profile change. No threshold is overridden and no
  global rules repository is modified.
- It does not change the source or the tests of the exempted modules. Their
  rejection, ordering, strict-parser and failure-propagation coverage remains
  the evidence of behaviour.

### Relation to the no-suppression requirement

The train forbids hiding a finding. CodeScene offers two customization
mechanisms: function-level directives and JSON rule sets. This factor is
module-level, so a directive cannot target it. The JSON rule set is therefore
the only mechanism that can scope the factor to the paths where the string is
the domain, and it keeps the override visible: the file is version-controlled,
each path carries its reason, CodeScene lists every override in Scope, and the
contract test refuses a widened set. The owner authorizes this exact-path
disposition as the issue's option (b); it is not a blanket waiver, and any
widening needs a new decision.

## Provenance and approval

- **Decision owner:** the repository owner, [@leynos][leynos], on behalf of
  issue [#257][issue-257]. This ADR records that decision; the pull request
  that introduces it is the review gate. Any widening still needs separate
  authorization.
- **Rule semantics:** CodeScene `String Heavy Function Arguments`, a
  module-level factor measured as the share of string parameters, with a
  reported threshold of 39%. The override names only this factor, so no
  critical rule is disabled.[^3]
- **Effective configuration:** the repository-local
  `.codescene/code-health-rules.json` over CodeScene's built-in defaults. No
  repository source records a global rules repository for this project, so the
  local file is the only repository-controlled override.
- **Policy owner:** the hosted CodeScene project and its profile belong to the
  repository owner. This decision changes only the repository-local rule file,
  so it needs no service-side action and no escalation; changing the hosted
  profile or a global rule set would be a separate authorization.
- **Local pre-flight:** `cs` 1.0.56, build
  `8b49281a77374d072db8470c5c53d348bcad3ef3`, built 2026-11-02.
  `cs rules-config validate` checks the file and `cs check <path>` measures one
  module. The local tool is a pre-flight; pull-request CI does not call
  CodeScene, and the hosted service owns the merge gate.
- **Hosted gate:** the CodeScene project's pull-request delta check. The
  published evidence is delta result `7995575` for PR #256 and the earlier
  candidate result `8016140`. The hosted engine version is service-controlled
  and is not pinned in the repository.
- **Out of scope:** Concordat [#153][concordat-153] owns rule-file schema and
  path validation, and Concordat [#157][concordat-157] owns canon publication.
  Neither decides this repository's disposition. No policy or service
  configuration change is requested here.

## Evidence

Three evidence sets stay distinct. The ratios are the local findings on the
17 October 2026 candidate described in issue #257. The `Without exemption` and
`With exemption` scores are a fresh local re-measurement of the current tree
with the installed `cs` 1.0.56: the first removes the seven rule sets
temporarily, the second applies the committed file. The published-head findings
are the hosted pull-request deltas recorded under Provenance and approval; this
record claims no new hosted run.

| Module                                    | Ratio | Disposition     | Without | With  |
| ----------------------------------------- | ----- | --------------- | ------- | ----- |
| `ci_reachability_make_tests.rs`           | 81.3% | exact-path rule | 9.68    | 10.00 |
| `ci_reachability_workflows_tests.rs`      | 85.7% | exact-path rule | 9.68    | 10.00 |
| `ci_reachability_make_source.rs`          | 64.3% | exact-path rule | 9.68    | 10.00 |
| `ci_reachability.rs`                      | 48.1% | exact-path rule | 9.68    | 10.00 |
| `ci_reachability_make.rs`                 | 40.7% | newtypes        | 10.00   | 10.00 |
| `ci_reachability_make_source_controls.rs` | 72.2% | exact-path rule | 9.68    | 10.00 |
| `ci_reachability_make_python_routes.rs`   | 76.9% | below threshold | 10.00   | 10.00 |
| `ci_reachability_make_trace.rs`           | 73.9% | exact-path rule | 9.68    | 10.00 |
| `ci_reachability_make_tools.rs`           | 76.5% | exact-path rule | 9.68    | 10.00 |
| `ci_reachability_runner.rs`               | 58.3% | newtypes        | 10.00   | 10.00 |

_Table 1: Measured Code Health scores per module with and without the
exact-path rule sets._

The three modules without an exemption are below the threshold on the current
source. Only the seven that still report the factor are exempted, so the rule
file is the smallest set that clears the measured gate.

## Acceptance criteria

- `cs rules-config validate` accepts the rule file.
- `cs check` reports 10.00 for all ten paths with the committed file.
- Removing the seven rule sets makes the seven modules report the factor again,
  proving the exemptions are load-bearing rather than stale.
- `tests/build_standard/codescene_rules.rs` holds the exact path set, the single
  rule override, the per-path rationale, and the absence of inline directives.
- `make check-fmt`, `make lint`, `make test` and `make markdownlint` pass.
- The hosted CodeScene pull-request check no longer reports the ten findings.

## Consequences

- A new text-parser module reports `String Heavy Function Arguments` until it
  either gains a type that models its text or is added, with a recorded reason,
  to both the rule file and the contract test.
- A renamed exempt module stops matching its rule set and reports the factor;
  the failure is closed rather than widening to a similar path.
- Reassess an exemption when its module gains a declared concept that would
  carry the parsed text as a value.
- Removing an exemption because the source no longer trips the factor is
  expected: the rule file shrinks with the source.
- The functional suite, not the health factor, remains the evidence that the
  decoders accept, reject and normalize text correctly.

## References

- [Issue #257: Clarify string-argument requirements for contract parsers][issue-257]
- [Developers' guide: CodeScene string-heavy arguments][developers-guide]
- [CodeScene: Code Health, custom rule sets and disabled rules][codescene-docs]

[^1]: CodeScene, "Code Health — How easy is your code to maintain and evolve?",
    section "Adapt Code Health to your Coding Standards",
    <https://docs.enterprise.codescene.io/latest/guides/technical/code-health.html#adapt-code-health-to-your-coding-standards>.

[^2]: CodeScene, "Customize the Code Health Rules via JSON" and "What happens to
    disabled rules?", same page,
    <https://docs.enterprise.codescene.io/latest/guides/technical/code-health.html#customize-the-code-health-rules-via-json>.

[^3]: CodeScene, "Recommendations for customizing code health rules", same page,
    <https://docs.enterprise.codescene.io/latest/guides/technical/code-health.html#recommendations-for-customizing-code-health-rules>.

[issue-257]: https://github.com/leynos/gauss/issues/257
[leynos]: https://github.com/leynos
[developers-guide]: ../developers-guide.md#codescene-string-heavy-arguments
[codescene-docs]: https://docs.enterprise.codescene.io/latest/guides/technical/code-health.html
[concordat-153]: https://github.com/leynos/concordat/issues/153
[concordat-157]: https://github.com/leynos/concordat/issues/157
