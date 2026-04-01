# Widget capability audit for Phase 1-2

This document presents the canonical inventory of user interface (UI) controls
required by Gauss Phase 1 and Phase 2, as defined by roadmap item 0.8.1. The
inventory serves as the authoritative source for control requirements before
implementation begins.

## Purpose

Before broad UI development accelerates, it is necessary to establish which
controls the application needs, what each control must do, and which
accessibility, keyboard, and action-routing requirements each one carries. This
audit distinguishes between stock `gpui-component` coverage and future
custom-control pressure, as mandated by architecture section 14.1.

## Source of truth

The typed inventory is maintained in `src/ui/widget_audit/` as a Rust module
directory. This ensures the audit can be validated through automated tests and
kept consistent with the roadmap. Each control entry records:

- **Name**: User-facing control identifier
- **Phase**: Phase 1 or Phase 2 roadmap requirement
- **Surface**: Where the control appears (toolbar, panel, canvas, etc.)
- **User Job**: What the user accomplishes with this control
- **States**: Required control states (selected, disabled, focused, etc.)
- **Keyboard Requirements**: Shortcuts and keyboard-only operation support
- **Accessibility Requirements**: Accessible Rich Internet Applications (ARIA)/
  AccessKit roles, labels, and states
- **Action/Command Linkage**: Integration with the Action → Command pipeline
- **Requirement Sources**: Roadmap items, feature plan sections, or architecture
  references that justify this control
- **Current Shell Evidence**: Whether the control exists in the Phase 0 shell

## Control inventory summary

### Phase 1 controls

Phase 1 delivers the essential drawing and editing experience. Required
controls cover:

#### Toolbar (6 tools)

- **Selection Tool** (`V`) — Select and move entire objects
- **Direct Selection Tool** (`A`) — Edit individual anchor points and segments
- **Pen Tool** (`P`) — Create freeform Bezier paths *(existing)*
- **Rectangle Tool** (`R`) — Create rectangles and squares
- **Ellipse Tool** (`O`) — Create ellipses and circles
- **Line Tool** (`\`) — Create straight line segments

Current shell evidence exists for Selection, Direct Selection, and Pen tools
via `src/ui/phase0_shell/tool_rail.rs`.

#### Properties panel (5 fields)

- **X Position Field** — View and edit object X coordinate
- **Y Position Field** — View and edit object Y coordinate
- **Width Field** — View and edit object width
- **Height Field** — View and edit object height
- **Rotation Field** — View and edit rotation angle in degrees

All properties panel fields must support keyboard input, arrow-key nudging, and
emit undoable commands on value change.

#### Alignment panel (8 controls)

- **Align Left** — Align selected objects to leftmost edge
- **Align Centre Horizontal** — Align to horizontal centre
- **Align Right** — Align to rightmost edge
- **Align Top** — Align to topmost edge
- **Align Centre Vertical** — Align to vertical centre
- **Align Bottom** — Align to bottommost edge
- **Distribute Horizontal** — Distribute objects evenly along horizontal axis
- **Distribute Vertical** — Distribute objects evenly along vertical axis

All alignment and distribution operations must be undoable.

#### Style panel (6 controls)

**Stroke Controls:**

- **Stroke Colour Picker** — Select stroke colour *(existing)*
- **Stroke Width Field** — Set stroke width in pixels/points *(existing)*
- **Stroke Opacity Slider** — Adjust stroke transparency 0–100% *(existing)*

**Fill Controls:**

- **Fill Colour Picker** — Select fill colour or no-fill state *(existing)*
- **Fill Opacity Slider** — Adjust fill transparency 0–100%
- **No Fill Toggle** — Remove fill from shapes

Current shell evidence exists for stroke colour, stroke width, stroke opacity,
and fill colour via `src/ui/phase0_shell/style_controls.rs`.

#### Layers panel (5 controls)

- **Layer Row** — Represent and select a layer in document hierarchy
- **Layer Visibility Toggle** — Show or hide a layer
- **Layer Lock Toggle** — Lock or unlock a layer to prevent edits
- **Layer Rename Field** — Edit layer name inline
- **Layer Reorder Handle** — Drag to reorder layers in stacking order

All layer operations must support keyboard-only operation and be undoable.

#### History panel (1 control, optional for Phase 1)

- **History Entry Row** — Display and select a history state for undo/redo

### Phase 2 controls

Phase 2 adds text editing and advanced styling capabilities.

#### Character panel (6 controls)

- **Font Family Selector** — Select font family for text
- **Font Size Field** — Set font size in points
- **Bold Toggle** (`Cmd+B`/`Ctrl+B`) — Apply or remove bold formatting
- **Italic Toggle** (`Cmd+I`/`Ctrl+I`) — Apply or remove italic formatting
- **Text Alignment Buttons** — Set alignment (left, centre, right, justify)
- **Text Colour Picker** — Set colour for text characters (reuses fill/stroke
  colour infrastructure per roadmap 2.2)

#### Paragraph panel (3 controls)

- **Paragraph Spacing Field** — Set spacing before/after paragraphs
- **Line Spacing Field** — Set leading (line height) for text
- **Indentation Controls** — Set left indent, right indent, first-line indent

#### Canvas text editor (1 control)

- **Inline Text Cursor** — Position cursor within text for editing on canvas

#### Type tool (1 control)

- **Type Tool** (`T`) — Phase 2 only, listed here for completeness

All text controls must support keyboard-only operation and route through the
Action → Command pipeline for undo/redo.

## Cross-cutting requirements

All controls in this inventory must satisfy the following architectural
invariants:

### Action/command integration

Every control that modifies document state must route through the Action →
Command pipeline. UI, scripting, and LLM control all invoke the same actions.
This ensures pervasive scriptability and consistent undo/redo behaviour.

### Keyboard-only operation

All controls must support keyboard-only operation. Toolbar tools must be
activatable via keyboard shortcuts. Panel controls must be navigable and
operable using only the keyboard. This satisfies the accessibility requirement
that keyboard-only users can accomplish all editing tasks.

### Accessibility

All controls must expose:

- **Role**: ARIA or AccessKit role (Button, TextInput, Slider, etc.).
- **Label**: Accessible label announcing the control's purpose.
- **States**: Accessible states (focusable, checked, expanded, etc.).
- **Announcements**: State changes must be announced to screen readers.

The accessibility tree uses stable node IDs per architecture section 16.

### Undo/redo

All controls that modify document state must emit Commands that support inverse
generation. This enables full undo/redo coverage as mandated by architecture
section 7.

## Implementation status

As of roadmap item 0.8.1, the following controls have current shell evidence
(Phase 0 implementation):

Table: Implementation status for UI controls

| Control               | File Path                               |
| --------------------- | --------------------------------------- |
| Selection Tool        | `src/ui/phase0_shell/tool_rail.rs`      |
| Direct Selection Tool | `src/ui/phase0_shell/tool_rail.rs`      |
| Pen Tool              | `src/ui/phase0_shell/tool_rail.rs`      |
| Stroke Colour Picker  | `src/ui/phase0_shell/style_controls.rs` |
| Stroke Width Field    | `src/ui/phase0_shell/style_controls.rs` |
| Stroke Opacity Slider | `src/ui/phase0_shell/style_controls.rs` |
| Fill Colour Picker    | `src/ui/phase0_shell/style_controls.rs` |

All other controls are pending implementation in future roadmap items.

## Validation

The control inventory is validated by:

- **Unit tests** (`rstest`) that assert completeness and consistency of control
  entries
- **Behaviour-Driven Development (BDD) tests** (`rstest-bdd`) that verify
  roadmap requirements map to typed inventory entries
- **GPUI (Zed's GPU-accelerated UI framework) tests** that prove shell-seam
  consistency for controls with current evidence

See `tests/widget_audit_test.rs` and
`tests/widget_capability_audit_bdd/main.rs` for test implementation.

## Next steps

This audit satisfies roadmap item 0.8.1. Subsequent work includes:

- **0.8.2**: Map controls to `gpui-component` widgets and identify which
  controls require custom implementation
- **0.8.3**: Plan and implement custom widget development for controls not
  covered by stock `gpui-component` widgets

## References

- Roadmap[^1]
- Feature Plan[^2]
- Architecture[^3]
- Typed Inventory[^4]

[^1]: `docs/roadmap.md` section 0.8
[^2]: `docs/gauss-feature-plan.md` Phase 1 and Phase 2
[^3]: `docs/gauss-architecture-design.md` section 14.1
[^4]: `src/ui/widget_audit/`
