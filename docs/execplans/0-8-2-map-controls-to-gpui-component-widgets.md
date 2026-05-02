# Map controls to gpui-component widgets (0.8.2)

This ExecPlan (execution plan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: DRAFT (2026-05-01)

## Purpose / big picture

Roadmap item `0.8.2` in [docs/roadmap.md](../roadmap.md) asks the project to
take the typed Phase 1-2 control inventory shipped by `0.8.1` and decide, for
each control, whether the stock `gpui-component` v0.5.1 catalogue covers it,
covers it only partially, or leaves Gauss to build a custom widget. The roadmap
phrases the deliverables as two bullet points:

- identify which widgets exist and are sufficient;
- flag controls needing custom implementation.

This matters because architecture section `14.1` in
[docs/gauss-architecture-design.md](../gauss-architecture-design.md) treats the
mapping as the second of three audit steps and explicitly expects it to feed a
"tiny internal widget library for missing pieces" in `0.8.3`. Without `0.8.2`,
Phase 1 UI work will keep guessing whether a stock control is good enough,
which contradicts the architectural directive that Gauss's chrome stays on
stock components wherever feasible while custom canvas-adjacent widgets are
planned deliberately. Roadmap items `1.4.3` (eyedropper), `1.5` (layers panel),
`1.8.1` (widget audit re-statement at the Phase 1 boundary), and `4.1.2`
(gradient editor) all depend on this mapping being explicit and testable.

Success is observable when:

- every `RequiredControl` in `src/ui/widget_audit/` carries a typed
  `WidgetMapping` that is one of "stock widget covers this control",
  "stock widget partially covers this control", or "no stock widget; custom
  required", with concrete `gpui-component` 0.5.1 widget names where they
  apply and a free-form rationale explaining gaps;
- the mapping is validated by `rstest` unit tests, by `rstest-bdd` v0.5.0
  scenarios that read the roadmap requirement back from the inventory, and by
  at least one `#[gpui::test]` that pins shipped Phase 0 widgets
  (`gpui_component::ColorPicker` for stroke and fill) to their `WidgetMapping`
  entries so the audit cannot silently drift away from the running shell;
- `docs/widget-capability-audit.md` is extended with a mapping section, a
  custom-widget watch list, and references that explicitly hand the watch list
  to `0.8.3`;
- `docs/gauss-architecture-design.md` section `14.1` records a new design
  decision noting that the typed inventory now carries the widget mapping as a
  first-class field;
- `docs/users-guide.md` and `docs/developers-guide.md` are updated only where
  the change actually affects a user-visible behaviour or developer-facing
  API. If neither applies, the plan must record explicitly why no delta is
  needed;
- `docs/roadmap.md` marks `0.8.2` done only after the mapping artefact,
  documentation updates, tests, and full repository gates pass;
- implementation does not begin until the user explicitly approves this draft.

## Context and orientation

The audit infrastructure already shipped by `0.8.1` is the starting point. A
novice should expect to find:

- the typed inventory at `src/ui/widget_audit/`, with `types.rs` defining
  `RequiredControl`, `ControlSurface`, `Phase`, `KeyboardRequirements`,
  `AccessibilityRequirements`, `ActionCommandLinkage`, `RequirementSource`,
  and `CurrentShellEvidence`. `RequiredControl` has no widget-mapping field
  today; this plan adds one;
- per-surface populators at `toolbar.rs`, `style.rs`, `properties.rs`,
  `alignment.rs`, `layers.rs`, `history.rs`, `canvas.rs`, `character.rs`, and
  `paragraph.rs`;
- `mod.rs` exposing `ControlInventory` with `all`, `by_phase`, `by_surface`,
  `with_evidence`, and `without_evidence`;
- `action.rs` declaring `AuditAction`, the payload-free mirror of
  `gauss_core::model::Action`.

The current shell seams that the mapping must reconcile against are:

- `src/ui/phase0_shell/style_controls.rs` already uses
  `gpui_component::color_picker::{ColorPicker, ColorPickerState,
  ColorPickerEvent}` for stroke colour and fill colour. Those two audit rows
  have `current_evidence.exists = true` and must therefore be tagged
  `WidgetMapping::Stock` against `gpui_component::color_picker::ColorPicker`;
- `src/ui/phase0_shell/tool_rail.rs` and
  `src/ui/phase0_shell/chrome_panels.rs` use bespoke styled `Div` builders
  rather than `gpui_component::Button`. The mapping must record that the audit
  rows for Selection, Direct Selection, Pen, and the alignment buttons can be
  satisfied by `gpui_component::button::Button` even though the current shell
  has not yet adopted it. The mapping is about `gpui-component` 0.5.1
  capabilities, not about what the Phase 0 shell happens to have wired today;
- `src/ui/phase0_shell/accessibility.rs` and
  `src/ui/phase0_shell/a11y_service/` define the accessibility expectations
  that any chosen widget must continue to honour;
- `src/ui/action_bridge/mod.rs` and `src/model/action.rs` define the
  Action-to-Command routing that the mapping must not break.

The available `gpui-component = "0.5.1"` widget catalogue, surveyed against
the registry source at `~/.cargo/registry/src/.../gpui-component-0.5.1/`,
covers most Phase 1-2 chrome. In summary:

- `button::{Button, ToggleButton, DropdownButton}`, with size, primary/ghost,
  outline, and toggle variants, covers tools, alignment buttons, fill toggle,
  arrange buttons, history-clear, and bold/italic toggles;
- `input::{Input, InputState, NumberInput}` covers properties-panel numeric
  fields and layer rename;
- `slider::Slider` covers stroke and fill opacity, font-size where a slider is
  desired, and any rotation slider;
- `color_picker::ColorPicker` covers stroke and fill colour;
- `select::Select` covers font-family combobox, alignment justify selectors,
  and dashed-line presets;
- `checkbox::Checkbox`, `switch::Switch`, and `radio::Radio` cover layer
  visibility and lock toggles, although a custom layer row is still likely to
  be more ergonomic;
- `tree::Tree` and `list::List` cover the layers list at a structural level
  but lack drag-reorder out of the box;
- `history::History` is already used by the existing selection history;
- `tooltip::Tooltip`, `popover::Popover`, `dialog::Dialog`, `tab::Tab`,
  `dock::Dock`, and `resizable::ResizablePanelGroup` cover surrounding chrome;
- `icon::{Icon, IconName}` covers iconography, subject to font and asset
  decisions outside this plan.

Concrete gaps that the mapping must call out and hand to `0.8.3`:

- no built-in eyedropper, marquee selection rectangle, increment spinner pair
  for numeric input, indeterminate ("mixed") checkbox, exclusive button group,
  drag-reorder for tree/list, font-preview combobox, or no-fill affordance on
  the colour picker. These map to roadmap items `1.4.3`, `1.2.1`, `1.3.3`,
  `1.5.4`, `1.5`, `2.2.1`, and `1.4.2` respectively, plus the gradient editor
  in `4.1.2`.

## Wyvern agent team

This planning draft uses a Wyvern agent team so the implementation can proceed
with explicit ownership rather than an undifferentiated to-do list. The same
team structure should be used during implementation, even if one human or one
coding agent performs the edits sequentially.

- **Wyrm-Lead** (architecture lead) owns the inventory shape and the boundary
  between requirement metadata and widget-mapping metadata. Wyrm-Lead is also
  responsible for ensuring the mapping artefact stays an internal UI-planning
  concern and does not leak into a public API.
- **Talon** (contracts lead) owns the typed schema for the new
  `WidgetMapping` enum and its supporting fields. Invalid combinations such
  as `Stock` with no widget reference, or `Custom` with no rationale, must be
  hard to express.
- **Scale** (mapping fidelity lead) owns the per-control mapping decisions.
  Scale ensures every `RequiredControl` receives a defensible mapping grounded
  in the `gpui-component` 0.5.1 catalogue and the existing shell evidence,
  and that "partial" rows record both what works and what is missing.
- **Wing** (validation lead) owns the rstest, BDD, and GPUI layers. Wing also
  owns the test fixture used to compile-time-or-runtime check that referenced
  widget paths actually exist in `gpui-component` 0.5.1.
- **Ember** (accessibility and keyboard lead) owns the rule that each chosen
  widget must continue to satisfy the existing accessibility and keyboard
  fields in `RequiredControl`. Ember flags any mapping where the stock widget
  cannot meet the audit's a11y or keyboard contract.
- **Echo** (documentation and signposting lead) owns the human-readable audit
  document, the architecture decision record update, and the cross-links that
  let `0.8.3` planning find the custom-widget watch list without re-deriving
  it from source.

## Constraints

- Scope is limited to roadmap item `0.8.2`: identify which widgets exist and
  are sufficient, and flag controls needing custom implementation. Do not
  implement, prototype, or design any custom widget here. Custom-widget work
  belongs to roadmap item `0.8.3`.
- Do not change the existing `RequiredControl` semantics for fields that
  `0.8.1` already shipped. The new `WidgetMapping` field must be additive and
  the existing query API on `ControlInventory` must remain stable.
- Do not introduce a public crate-level API surface for `WidgetMapping`. The
  artefact remains internal to the `gauss` UI crate, mirroring the typed
  audit catalogue decision recorded in architecture section `14.1`.
- Do not refactor `Phase0Shell` or its modules to adopt new widgets as part
  of this roadmap item. The mapping records *capability*, not adoption. Adoption
  of `gpui_component::button::Button` for the tool rail or alignment buttons is
  Phase 1 UI work, not `0.8.2`.
- Preserve the Action-to-Command architecture: the mapping must not relax the
  rule that every state-mutating control routes through actions and commands.
  If a candidate stock widget cannot route through the action pipeline without
  contortion, the mapping records that as a "partial" with a flagged risk.
- Preserve all existing accessibility and keyboard fields. The mapping is not
  permitted to weaken them in pursuit of a cleaner stock-widget choice.
- Any Rust additions must obey repository rules: module-level `//!` comments,
  rustdoc on public items, no file above 400 lines, en-GB-Oxford spelling, and
  no lint suppressions except as a last resort.
- Required validation for the final implementation remains: `rstest` unit
  tests, `rstest-bdd` v0.5.0 behaviour tests, `#[gpui::test]` coverage,
  `make check-fmt`, `make lint`, and `make test`.
- Update `docs/widget-capability-audit.md` to publish the mapping in a
  human-readable form, and update `docs/gauss-architecture-design.md` section
  `14.1` to record the new design decision.
- Update `docs/users-guide.md` only if the implementation introduces a new
  user-visible surface or behaviour. The mapping itself is internal, so the
  default outcome is "no user's-guide delta required" with a recorded
  rationale. Update `docs/developers-guide.md` to describe the `WidgetMapping`
  schema and how to extend it when adding controls in `0.8.3` or later.
- Do not mark `docs/roadmap.md` item `0.8.2` done until the implementation is
  complete, approved, and fully validated.
- Do not begin implementation until the user explicitly approves this
  ExecPlan.
- Operate from a task-named branch. Do not touch `main`. The current branch
  `session/6e985842` is acceptable for drafting; for implementation, prefer a
  branch named `roadmap/0-8-2-map-controls-to-gpui-widgets` or equivalent.

## Tolerances (exception triggers)

- Scope tolerance: if completing `0.8.2` requires shipping any custom widget,
  prototype, or runtime mapping viewer, stop and split the work into `0.8.3`
  rather than rolling roadmap items together.
- File tolerance: if the implementation grows beyond 14 files or 700 net lines
  of changes, pause and re-scope before proceeding.
- Interface tolerance: if a typed `WidgetMapping` would need to become part of
  a public library API (for example, exposed from `gauss-core`) rather than
  an internal `gauss` UI artefact, stop and re-evaluate the shape.
- Mapping tolerance: if more than five Phase 1-2 controls would require a
  newly-discovered `gpui-component` upgrade or a contribution upstream, stop
  and confirm whether the upgrade is acceptable.
- Documentation tolerance: if keeping the inventory and the prose audit page
  consistent without generation tooling becomes unsustainable, stop and ask
  whether generated documentation is acceptable.
- Test tolerance: if GPUI coverage would require introducing a synthetic
  surface beyond the existing `style_controls` colour pickers to satisfy the
  GPUI layer, stop and confirm whether a shell-seam validation test is
  sufficient.
- Dependency tolerance: if a new external crate appears necessary to satisfy
  the mapping, stop and escalate.
- Tooling tolerance: if `grepai` remains unavailable, proceed with `leta` plus
  `rg` as 0.8.1 did, but record any evidence that cannot be verified locally.
- Iteration tolerance: if rstest, BDD, or GPUI coverage still fails after
  three structural attempts, stop and escalate rather than tightening the
  schema until something passes.
- Gate tolerance: if `make check-fmt`, `make lint`, or `make test` fail for
  unrelated pre-existing reasons, capture tee-log evidence and leave roadmap
  closure undone.

## Risks

- Risk: the mapping could become superficial labels without a rationale,
  undermining its usefulness for `0.8.3`. Severity: high. Likelihood: medium.
  Mitigation: Talon enforces a typed schema where `Partial` and `Custom`
  variants must carry a non-empty rationale string; Wing's tests assert it.
- Risk: stock-widget choices could silently weaken accessibility or keyboard
  behaviour. Severity: high. Likelihood: medium. Mitigation: Ember reviews
  every mapping against the existing `accessibility` and `keyboard` fields,
  and adds an rstest invariant that `keyboard.keyboard_only_operation` cannot
  be downgraded by a mapping decision.
- Risk: the mapping drifts from the actual `gpui-component` 0.5.1 surface as
  the dependency upgrades. Severity: medium. Likelihood: medium. Mitigation:
  encode widget references as compile-time-checked module paths in the test
  layer where feasible, otherwise as `&'static str` paths whose existence is
  asserted by a small GPUI or build-time test.
- Risk: `gpui-component` 0.5.1 lacks a dedicated "no-fill" or "indeterminate"
  affordance, leading to tempting work-arounds. Severity: medium. Likelihood:
  high. Mitigation: explicitly enumerate these gaps in the custom-widget watch
  list and label the relevant Style and Layers controls `Partial`.
- Risk: the mapping could implicitly promise a layers panel implementation by
  marking it `Stock` against `tree::Tree`. Severity: high. Likelihood: medium.
  Mitigation: layer-row rename, visibility, lock, and reorder controls must be
  evaluated individually; reorder must remain `Custom` because `tree::Tree`
  has no drag-reorder out of the box, and rename should remain `Partial`
  pending a chosen interaction model.
- Risk: tests become busywork that snapshot strings rather than validate
  behaviour. Severity: medium. Likelihood: high. Mitigation: Wing's plan
  layers tests at three levels (data invariants, roadmap-language scenarios,
  shell-seam GPUI consistency) so each layer answers a distinct question.
- Risk: `grepai` is unavailable in this environment. Severity: low.
  Likelihood: high. Mitigation: use `leta` for symbol-aware discovery and
  `rg` for cross-document coverage, exactly as `0.8.1` did.

## Progress

- [x] (2026-05-01) Loaded `AGENTS.md`, the `execplans` skill, and the auto
  memory index before drafting.
- [x] (2026-05-01) Reviewed `docs/roadmap.md` items `0.8.1`, `0.8.2`, `0.8.3`,
  `1.3`, `1.4`, `1.5`, `1.6.2`, `1.8.1`, `1.9`, `2.2`, and `4.1.2`.
- [x] (2026-05-01) Reviewed `docs/widget-capability-audit.md`,
  `docs/widget-audit-developer-guide.md`,
  `docs/using-gpui-and-gpui-component.md`,
  `docs/accesskit-based-accessibility-in-gpui.md`,
  `docs/gauss-architecture-design.md` section `14.1`,
  `docs/rust-testing-with-rstest-fixtures.md`,
  `docs/rust-doctest-dry-guide.md`,
  `docs/reliable-testing-in-rust-via-dependency-injection.md`, and
  `docs/rstest-bdd-users-guide.md`.
- [x] (2026-05-01) Inspected the typed audit module at
  `src/ui/widget_audit/`, especially `types.rs::RequiredControl`.
- [x] (2026-05-01) Inspected current shell seams in
  `src/ui/phase0_shell/tool_rail.rs`,
  `src/ui/phase0_shell/style_controls.rs`, and
  `src/ui/phase0_shell/chrome_panels.rs` and confirmed which already use
  `gpui_component` widgets.
- [x] (2026-05-01) Catalogued the `gpui-component = "0.5.1"` widget surface
  from the registry source under
  `~/.cargo/registry/src/.../gpui-component-0.5.1/` to anchor the mapping.
- [x] (2026-05-01) Used a Wyvern agent team to ground this draft in concrete
  evidence rather than free-form recall.
- [x] (2026-05-01) Drafted this ExecPlan at
  `docs/execplans/0-8-2-map-controls-to-gpui-component-widgets.md`.
- [ ] Await user approval of this ExecPlan.
- [ ] Implement the typed `WidgetMapping`, populate per-control mappings,
  publish documentation updates, ship validation tests, and close the roadmap
  item per the milestones below.

## Surprises & Discoveries

- The Phase 0 shell's tool rail and alignment buttons do not use
  `gpui_component::button::Button` today; they are bespoke styled `Div`s in
  `tool_rail.rs` and `chrome_panels.rs`. This means the audit's
  `current_evidence` field for those rows must stay independent of the
  `WidgetMapping` field. The mapping should record that
  `gpui_component::button::Button` *can* satisfy them, while
  `current_evidence` continues to record what is actually wired.
- The shipped colour pickers in `src/ui/phase0_shell/style_controls.rs`
  already use `gpui_component::color_picker::ColorPicker`. Those two rows are
  the natural anchor for a single `#[gpui::test]` that pins the mapping to a
  real running widget rather than only to the static catalogue.
- `gpui-component` 0.5.1 ships a `History` type (`gpui_component::history`)
  that the existing selection-history code already uses. This makes the
  optional History panel row a reasonable `Stock` candidate, although the
  current evidence note must continue to say that no panel yet uses it.
- `gpui-component` 0.5.1 has no dedicated "no-fill" affordance on its colour
  picker and no built-in eyedropper, marquee, indeterminate checkbox, font
  preview combobox, or drag-reorder tree. These are precisely the items the
  custom-widget watch list must call out, and they align with the architecture
  document's expectation that custom canvas-adjacent controls will be needed
  early.
- `grepai` remains unavailable. `leta` is available and works as the
  symbol-aware discovery tool. `rg` remains necessary for multi-file roadmap
  and documentation coverage checks.

## Decision Log

- 2026-05-01: Add a typed `WidgetMapping` enum to the audit module rather
  than encoding mapping information as a free-text annotation on
  `current_evidence`. Rationale: the audit must distinguish "stock widget
  exists" from "shell has wired it" and from "no stock widget exists" cleanly,
  and the schema must reject incomplete entries.
- 2026-05-01: Express stock widgets as compile-time references where
  practical, falling back to `&'static str` paths when type erasure or
  generics make a direct path unwieldy. Rationale: a compile error is the
  most reliable signal that a referenced widget no longer exists; static
  strings keep the schema flexible for module-level entries.
- 2026-05-01: Keep `WidgetMapping` internal to the `gauss` UI crate.
  Rationale: the artefact is a planning input for `0.8.3`, not a product
  contract. Architecture section `14.1` already established this principle for
  the typed audit catalogue.
- 2026-05-01: Do not change `current_evidence` semantics. Rationale: `0.8.1`
  shipped that field with a specific meaning (the Phase 0 shell wiring), and
  conflating it with the mapping would lose information.
- 2026-05-01: Wing's GPUI layer covers only the two existing
  `gpui_component::ColorPicker` instances in `style_controls.rs`. Rationale:
  inventing additional UI surfaces purely to exercise more widgets would
  violate the test-tolerance threshold and the rule that GPUI tests must
  exercise real shell seams.
- 2026-05-01: Treat `grepai` as unavailable for this planning turn. Rationale:
  no installed binary or MCP resource is present, and the draft must stay
  grounded in verifiable local evidence.
- 2026-05-01: No implementation work begins until the user approves this
  draft. Rationale: the `execplans` approval gate is mandatory for this task.

## Plan of work

Implementation should proceed in four milestones. Each ends with validation,
and no milestone may begin until the prior one is green.

### Milestone 1. Extend the audit schema for widget mapping

Add a new module `src/ui/widget_audit/widget_mapping.rs` that defines a
`WidgetMapping` enum and supporting types. Talon owns the schema; the goal is
that an invalid entry will not compile and an incomplete entry will not pass
the rstest invariant suite.

The proposed shape:

```rust
/// Reference to a widget shipped by `gpui-component` 0.5.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StockWidget {
    /// Fully qualified module path, e.g. "gpui_component::button::Button".
    pub path: &'static str,
    /// Short note, e.g. "with .ghost() variant".
    pub variant: &'static str,
}

/// Reason a stock widget only partially covers a required control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PartialCoverage {
    /// Stock widget that covers the bulk of the control.
    pub stock: StockWidget,
    /// Concrete capability that is missing or weak.
    pub gap: &'static str,
    /// Recommended follow-up: extend, wrap, or defer to 0.8.3.
    pub follow_up: &'static str,
}

/// Why a control cannot be satisfied by a stock widget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CustomRequirement {
    /// Short rationale, e.g. "no built-in eyedropper".
    pub rationale: &'static str,
    /// Cross-link to the future custom-widget plan, e.g. "0.8.3".
    pub planned_in: &'static str,
}

/// Mapping of a `RequiredControl` onto the gpui-component 0.5.1 catalogue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetMapping {
    Stock(StockWidget),
    Partial(PartialCoverage),
    Custom(CustomRequirement),
}
```

Add a `widget_mapping: WidgetMapping` field to `RequiredControl` in
`src/ui/widget_audit/types.rs`. Re-export `WidgetMapping`, `StockWidget`,
`PartialCoverage`, and `CustomRequirement` from
`src/ui/widget_audit/mod.rs`. Add accessor methods on `RequiredControl`:
`widget_mapping()`, `is_stock()`, `is_partial()`, `is_custom()`.

Extend `ControlInventory` in `src/ui/widget_audit/mod.rs` with three filters:
`stock_mapped()`, `partially_mapped()`, and `custom_required()`. Each returns
a `Vec<&RequiredControl>` for use in tests and documentation.

Update the developer guide at `docs/widget-audit-developer-guide.md` with a
new "Adding widget mapping" section that walks through populating
`WidgetMapping` for a new control, mirroring the existing
"Adding new controls" walkthrough.

Validate Milestone 1 with rstest unit tests in
`tests/widget_audit_test.rs` that:

- assert every `RequiredControl` has a mapping;
- assert every `Partial` mapping has a non-empty `gap` and `follow_up`;
- assert every `Custom` mapping has a non-empty `rationale` and `planned_in`;
- assert that stock widget paths begin with `gpui_component::`.

### Milestone 2. Populate the Phase 1-2 mapping

Scale and Ember work through every populator file under
`src/ui/widget_audit/` and assign a mapping per control. The expected
distribution at the end of this milestone is roughly:

- Toolbar (`toolbar.rs`): all six Phase 1 tools and the Phase 2 Type tool map
  to `Stock(gpui_component::button::Button)` with the toggle-button variant
  for tool selection. The eyedropper from `1.4.3` maps to
  `Custom(rationale = "no built-in eyedropper", planned_in = "0.8.3")`.
- Properties (`properties.rs`): X, Y, Width, Height, Rotation map to
  `Stock(gpui_component::input::NumberInput)`. If arrow-key nudging requires
  spinner buttons that the stock widget lacks, mark `Partial` with the spinner
  gap and the recommendation to extend in `0.8.3`.
- Alignment (`alignment.rs`): all six alignment buttons and both distribute
  buttons map to `Stock(gpui_component::button::Button)`.
- Style (`style.rs`): stroke colour and fill colour map to
  `Stock(gpui_component::color_picker::ColorPicker)`. Stroke width maps to
  `Stock(gpui_component::input::NumberInput)`. Stroke and fill opacity map to
  `Stock(gpui_component::slider::Slider)`. The "no fill" toggle maps to
  `Partial(stock = ColorPicker, gap = "no built-in no-fill state", follow_up
  = "wrap with toggle in 0.8.3")`.
- Layers (`layers.rs`): the layer row maps to
  `Stock(gpui_component::list::ListItem)`; visibility and lock toggles map to
  `Stock(gpui_component::checkbox::Checkbox)` or
  `Stock(gpui_component::switch::Switch)` depending on Ember's accessibility
  review; rename maps to
  `Partial(stock = Input, gap = "double-click-to-edit affordance must be
  added", follow_up = "wrap as part of layer row")`; reorder maps to
  `Custom(rationale = "no drag-reorder for tree/list in gpui-component 0.5.1",
  planned_in = "0.8.3")`.
- History (`history.rs`): the optional history-entry row maps to
  `Stock(gpui_component::list::ListItem)` and the underlying state to
  `Stock(gpui_component::history::History)`.
- Character (`character.rs`): font family maps to
  `Partial(stock = Select, gap = "no font-preview rendering", follow_up =
  "wrap with custom item renderer in 0.8.3")`. Font size maps to
  `Stock(NumberInput)`. Bold and italic toggles map to
  `Stock(gpui_component::button::ToggleButton)`. Text alignment maps to
  `Stock(button::ToggleButton)` group plus a manual exclusivity wrapper, with
  exclusivity flagged as a `Partial` gap. Text colour maps to
  `Stock(ColorPicker)`.
- Paragraph (`paragraph.rs`): paragraph spacing, line spacing, and indents map
  to `Stock(NumberInput)`.
- Canvas text editor (`canvas.rs`): the inline cursor maps to
  `Custom(rationale = "no on-canvas text caret", planned_in = "0.8.3")`.

The exact assignments are part of implementation, not part of this draft. The
above is a budget that Scale must defend during implementation. Where Ember
flags an accessibility or keyboard shortfall, the mapping must move from
`Stock` to `Partial`.

Validate Milestone 2 with parameterised rstest cases that:

- assert that controls with `current_evidence.exists = true` have a `Stock`
  mapping that names the widget the shell currently uses;
- assert that any control whose `keyboard.shortcut` is set has a mapping
  whose stock widget is documented to honour custom keybindings;
- assert that the toolbar surface is fully `Stock` mapped against
  `gpui_component::button::Button` family widgets.

### Milestone 3. Behavioural and GPUI validation

Wing extends `tests/widget_capability_audit_bdd/main.rs` and the feature file
`tests/features/widget_capability_audit.feature` with new scenarios that
read the roadmap requirement for `0.8.2` back from the inventory:

- a scenario "Every Phase 1-2 control declares a widget mapping" verifies
  that no control has been overlooked;
- a scenario "Stock-mapped controls reference a `gpui_component` widget"
  verifies the path prefix invariant;
- a scenario "Controls without stock coverage carry a custom-widget rationale"
  verifies the `Custom` schema and links to `0.8.3`;
- a scenario "Controls with current shell evidence are marked stock" pins the
  Phase 0 colour pickers' mapping.

Then add or extend a `#[gpui::test]` in
`tests/phase0_shell_widget_audit_gpui_test.rs` (new file or extension of the
existing GPUI audit test) that:

- builds the `Phase0Shell` for tests via the existing `new_for_tests` path;
- looks up the stroke and fill colour-picker rows in the inventory;
- asserts that their `WidgetMapping::Stock.path` resolves to the
  `gpui_component::color_picker::ColorPicker` type used by the shell.

The GPUI test must not add a new UI surface. It exercises only the two real
shell seams that already use `ColorPicker`.

### Milestone 4. Documentation and roadmap closure

Echo extends `docs/widget-capability-audit.md` with a new "Mapping to
`gpui-component` 0.5.1" section that:

- restates the three mapping outcomes (`Stock`, `Partial`, `Custom`);
- presents a compact table of the Phase 1-2 mapping, derived from the typed
  inventory;
- lists the custom-widget watch list that hands off to `0.8.3`, with explicit
  cross-references to roadmap items `1.4.3` (eyedropper), `1.2.1` (marquee),
  `1.5.4` (drag-reorder), `2.2.1` (font preview), and `4.1.2` (gradient
  editor).

Update `docs/gauss-architecture-design.md` section `14.1` with a new "Design
Decision: Widget Mapping (2026-05-XX)" block that records the typed
`WidgetMapping` decision, mirroring the existing "Typed Audit Catalogue"
block from `0.8.1`.

Update `docs/developers-guide.md` to document the new `WidgetMapping` schema,
how to query it from `ControlInventory`, and how to extend it when adding new
controls. Update `docs/users-guide.md` only if a user-visible behaviour
changes; the default outcome is no delta, recorded in the Decision Log.

Mark roadmap item `0.8.2` as done in `docs/roadmap.md` only after every prior
milestone is green and the gates pass.

## Concrete steps

Run the following commands from the repository root after each milestone.
Capture output to `tee` files so truncated terminal output remains
recoverable.

```plaintext
set -o pipefail
make check-fmt 2>&1 | tee /tmp/0-8-2-check-fmt.log
```

```plaintext
set -o pipefail
make lint 2>&1 | tee /tmp/0-8-2-lint.log
```

```plaintext
set -o pipefail
make test 2>&1 | tee /tmp/0-8-2-test.log
```

If documentation outside the Rust tree changes, also run:

```plaintext
set -o pipefail
make fmt 2>&1 | tee /tmp/0-8-2-fmt.log
```

```plaintext
set -o pipefail
make markdownlint 2>&1 | tee /tmp/0-8-2-markdownlint.log
```

```plaintext
set -o pipefail
make nixie 2>&1 | tee /tmp/0-8-2-nixie.log
```

The implementation is complete only when the inventory carries widget
mappings, docs are updated, tests are green, and the roadmap entry is marked
done.

## Validation and acceptance

A novice on a fresh checkout must be able to verify the result as follows.

1. Run `cargo doc --no-deps -p gauss` and confirm
   `gauss::ui::widget_audit::WidgetMapping`, `StockWidget`, `PartialCoverage`,
   and `CustomRequirement` are present.
2. Run `make test 2>&1 | tee /tmp/0-8-2-test.log`. Expect the new rstest
   invariants, the new `rstest-bdd` scenarios in
   `tests/widget_capability_audit_bdd/main.rs`, and the GPUI shell-seam
   consistency test to pass. Each new test must fail before the
   `WidgetMapping` field is populated and pass after.
3. Open `docs/widget-capability-audit.md` and confirm the new mapping table
   and custom-widget watch list are present, and that they cross-link to
   `0.8.3` and the relevant Phase 1 roadmap items.
4. Open `docs/gauss-architecture-design.md` and confirm the new design
   decision block under section `14.1` records the typed mapping.
5. Open `docs/roadmap.md` and confirm `0.8.2` is checked off only once the
   above hold.

Quality criteria (what "done" means):

- Tests: every layer above passes; `make test` is green.
- Lint: `make lint` is green with no new `#[allow]` or `#[expect]` entries
  introduced solely to satisfy the new code.
- Format: `make check-fmt` is green.
- Documentation: `make markdownlint` and `make nixie` are green if any
  Markdown or Mermaid diagrams change.

Quality method (how we check): the `make` targets above, run from the
repository root, with `tee` capturing logs to `/tmp/0-8-2-*.log` for review.

## Idempotence and recovery

- The schema extension is additive: rerunning the implementation steps on a
  partially-applied tree must not corrupt the inventory. If a previous run
  left some controls without a mapping, the rstest invariants will fail
  loudly until those rows are filled in.
- If a populator file has been updated but tests have not yet been re-run,
  rerun `make test 2>&1 | tee /tmp/0-8-2-test.log` to re-establish the
  signal. No destructive cleanup is required.
- If the `gpui-component` dependency is upgraded mid-implementation, rerun
  the agent that surveyed the registry source and confirm any widget paths
  that changed before continuing.

## Artifacts and notes

- `src/ui/widget_audit/types.rs` — extend `RequiredControl`.
- `src/ui/widget_audit/widget_mapping.rs` — new module with the typed
  mapping schema.
- `src/ui/widget_audit/mod.rs` — re-export the new types and add inventory
  filters.
- `src/ui/widget_audit/{toolbar, style, properties, alignment, layers,
  history, canvas, character, paragraph}.rs` — populate the mapping for each
  control.
- `tests/widget_audit_test.rs` — extend with mapping invariants.
- `tests/features/widget_capability_audit.feature` and
  `tests/widget_capability_audit_bdd/main.rs` — extend with `0.8.2`
  scenarios.
- `tests/phase0_shell_widget_audit_gpui_test.rs` — new or extended GPUI
  consistency test for stroke and fill `ColorPicker` rows.
- `docs/widget-capability-audit.md`,
  `docs/widget-audit-developer-guide.md`,
  `docs/gauss-architecture-design.md`, `docs/developers-guide.md`,
  `docs/users-guide.md`, and `docs/roadmap.md` — documentation updates per
  Milestone 4.

## Interfaces and dependencies

In `src/ui/widget_audit/widget_mapping.rs`, define exactly the types listed
in Milestone 1. In `src/ui/widget_audit/mod.rs`, re-export them and add:

```rust
impl ControlInventory {
    pub fn stock_mapped(&self) -> Vec<&RequiredControl> { /* ... */ }
    pub fn partially_mapped(&self) -> Vec<&RequiredControl> { /* ... */ }
    pub fn custom_required(&self) -> Vec<&RequiredControl> { /* ... */ }
}
```

In `src/ui/widget_audit/types.rs`, add `widget_mapping: WidgetMapping` to
`RequiredControl` and the corresponding accessor methods. The new module
declaration in `mod.rs` must include `pub mod widget_mapping;` and a
re-export.

External dependencies do not change. The plan must not introduce new crates.

## References

This plan signposts the following documents and skills. Re-read them before
implementation.

- [docs/roadmap.md](../roadmap.md) — items `0.8.1`, `0.8.2`, `0.8.3`, `1.3`,
  `1.4`, `1.5`, `1.8.1`, `1.9`, `2.2`, `4.1.2`.
- [docs/widget-capability-audit.md](../widget-capability-audit.md) — the
  inventory's prose form, to be extended in Milestone 4.
- [docs/widget-audit-developer-guide.md](../widget-audit-developer-guide.md)
  — the existing developer guide to extend with mapping guidance.
- [docs/gauss-architecture-design.md](../gauss-architecture-design.md) —
  section `14`, especially `14.1`.
- [docs/using-gpui-and-gpui-component.md](../using-gpui-and-gpui-component.md)
  — pinned versions, component patterns, headless test guidance.
- [docs/accesskit-based-accessibility-in-gpui.md](../accesskit-based-accessibility-in-gpui.md)
  — accessibility expectations Ember enforces.
- [docs/rust-testing-with-rstest-fixtures.md](../rust-testing-with-rstest-fixtures.md)
  — rstest fixture patterns Wing relies on.
- [docs/rust-doctest-dry-guide.md](../rust-doctest-dry-guide.md) — doctest
  guidance for the new public types' rustdoc.
- [docs/reliable-testing-in-rust-via-dependency-injection.md](../reliable-testing-in-rust-via-dependency-injection.md)
  — dependency injection for testable wiring.
- [docs/rstest-bdd-users-guide.md](../rstest-bdd-users-guide.md) — BDD
  scenario authoring at the v0.5.0 surface.
- Skills to load before implementation: `execplans`, `rust-router`,
  `rust-types-and-apis`, `arch-crate-design`, `nextest`, `commit-message`,
  `pr-creation`, `en-gb-oxendict`. Load `logisphere-design-review` if a
  pre-implementation review of this draft is requested.

## Outcomes & Retrospective

Pending. This section must remain in place and be completed after the user
approves the plan and the implementation finishes. Capture, at minimum: how
many controls fell into each mapping category, which custom-widget gaps were
new versus already known, and any `gpui-component` upstream contributions or
issues that emerged during implementation.
