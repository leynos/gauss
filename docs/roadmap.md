# Gauss development roadmap

This roadmap defines the path from the current Phase 0 proof-of-concept to
Illustrator 10 feature parity. It synthesises the strategic vision from the
[feature plan](gauss-feature-plan.md) with the architectural foundations
described in the [architecture document](gauss-architecture-design.md), and
reflects the current state of the codebase.

Each phase builds upon the last. Cross-cutting concerns—accessibility,
localizability, performance, and pervasive scripting—are addressed from the
start rather than retrofitted.

## Guiding principles

These principles from the [architecture document](gauss-architecture-design.md)
are non-negotiable invariants. If a feature violates them, the feature is
redesigned.

1. **Everything is an Action (and therefore scriptable)**: All user-visible
   behaviour must be representable as an Action → Command pipeline. UI,
   scripting, and LLM control all invoke the same actions. If there is no
   command, the feature does not exist.

2. **Single source of truth: document state in the engine**: The document and
   editor state (selection, tool mode, viewport) live in engine state, not in
   the view layer. The view is a projection of state plus a dispatcher of
   actions.

3. **Deterministic geometry and rendering**: Given the same document and
   viewport, Gauss produces the same scene every time, across platforms and
   runs. Stable ordering of nodes, stable IDs, deterministic floating-point
   strategy.

4. **Accessibility is a first-class API surface**: Keyboard-only operation is
   always supported. UI controls expose roles, labels, states, and actions.
   Canvas interactions have keyboard equivalents where feasible. Accessibility
   tree uses stable node IDs.

5. **SVG first with reversible transforms**: SVG is the native format.
   Round-tripping keeps SVG semantics intact. Non-SVG editor metadata is stored
   in a safe, namespaced way. Export can strip metadata for web-ready SVG.

6. **Platform abstraction at the edges**: All platform-specific behaviour
   (dialogs, clipboard, filesystem, accessibility adapters) is isolated behind
   a small platform boundary.

## Current state summary

Phase 0 (proof-of-concept) is complete. The following capabilities are
implemented and tested:

- **Model/view separation**: Pure data model in `src/model/` with no GPUI
  dependency; UI logic in `src/ui/`.
- **Draw mode**: Click-to-place anchors with line or auto-smooth cubic
  segments; snap-to-first closes paths.
- **Manipulate mode**: Selection of shapes, anchors, handles, and segments;
  drag-to-move; multi-select via Shift+click.
- **Invertible operations**: Full undo/redo via `DocOp` with dual history
  stacks (document edits and selection changes).
- **SVG round-trip**: Import and export of `<path>` elements with absolute
  commands (M, L, C, Z) and basic styling.
- **Viewport control**: Pan and zoom with cursor-centred scaling.
- **Basic styling**: Colour picker for stroke and fill; stroke width and
  opacity.
- **Window chrome**: Minimize, maximize, fullscreen, close; keyboard
  shortcuts; resize handles that respect maximised state.
- **Accessibility framework**: Stable node IDs defined; AccessKit not yet
  wired.

**Not yet implemented**: Shape tools, transform handles, layers panel, advanced
styling, gradients, text, effects, symbols, scripting integration.

______________________________________________________________________

## 0. Architecture foundations

**Goal**: Establish the architectural spine before broad feature work
accelerates. These work items create the infrastructure that all subsequent
phases depend upon. See architecture document §20.

### 0.1. Action/Command registry

- [x] 0.1.1. Define typed Action enum.
  - [x] Actions represent user intent (e.g., "Delete Selection").
  - [x] Actions are dispatchable from UI, scripts, and tests.
- [x] 0.1.2. Implement Command dispatch.
  - [x] Commands are concrete, undoable state changes.
  - [x] Commands are serializable for macro recording (optional initially).
- [x] 0.1.3. Create key context system.
  - [x] Actions bind to keyboard shortcuts via key contexts.
  - [x] Extend existing `KEY_CONTEXT` pattern from Phase 0.

### 0.2. Core EngineState

- [x] 0.2.1. Consolidate engine state.
  - [x] Unify document, selection, viewport, and resources into `EngineState`.
  - [x] Ensure single source of truth per guiding principle §2.
- [x] 0.2.2. Implement stable ID generation.
  - [x] Use generational IDs (`slotmap` or similar). See architecture §5.1.
  - [x] Ensure IDs are stable across frames for AccessKit.
- [x] 0.2.3. Define resource stores.
  - [x] `StyleStore`, `ResourceStore` for gradients, patterns, symbols.
  - [x] Prepare for Phase 4 colour and effects features.

### 0.3. History and grouping

- [x] 0.3.1. Audit existing undo/redo implementation. (Complete — the
  `undo_2` spike migrated document history to a model-layer adapter
  `DocumentUndoHistory`, validated undo/redo round-trips, and confirmed
  history-clear-on-open semantics. Single-entry-per-gesture invariant verified
  by unit tests, BDD scenarios, and GPUI integration tests.) In scope:
  undo/redo round-trip validation, history-clear semantics, adapter
  integration, single-entry-per-gesture audit. Out of scope: command grouping
  (see 0.3.2).
  - [x] Verify multistep interactions create single undo entries.
        Completion: all drag and compound tool interactions produce
        exactly one undo entry each.
  - [x] Test history clear on document open. See architecture §7.2.
- [x] 0.3.2. Implement command grouping API. Requires 0.3.1. (Complete —
  `DocumentUndoHistory` now exposes `begin_group()` / `end_group()` with
  deterministic boundary errors. Grouped command sequences collapse to one undo
  step, validated by model unit tests, behavioural BDD scenarios, and GPUI
  integration tests.)
  - [x] Begin/end transaction for compound operations. Completion:
        `DocumentUndoHistory` exposes grouping API and groups
        collapse to a single undo step in tests.
  - [x] Integrate with model-layer `DocumentUndoHistory`.
- [x] 0.3.3. Add inverse command generation.
  - [x] All commands produce `CommandInverse` for undo.
        See architecture §7.1.
- [x] 0.3.4. Move document history ownership to EngineState. (Complete —
  `EngineState` now owns `DocumentUndoHistory`; `Phase0Shell` delegates
  document history operations in draw/chrome/open call paths; validated by unit
  tests, behavioural BDD scenarios, and GPUI integration tests.)
- [ ] 0.3.5. Evolve history error model from `String` to enum.
  The adapter returns `Result<(), String>`; converting to a `HistoryError` type
  provides structured error handling. Deferred: changes public API and all call
  sites.

### 0.4. SVG load/save and metadata policy

- [x] 0.4.1. Define Gauss metadata namespace.
  - [x] Use `gauss:` prefixed attributes or `<metadata>` block.
  - [x] Document namespace in ADR. See architecture §10.1.
- [x] 0.4.2. Implement metadata round-trip.
  - [x] Editor-only data (`gauss:id`, `gauss:name`, `gauss:locked`,
        `gauss:hidden`, opaque `gauss:*` attrs, `<metadata>` block) survives
        load/save cycle.
  - [x] 6 golden reference SVGs with idempotent round-trip assertions.
  - [x] 30 unit tests, 6 BDD scenarios, 3 GPUI integration tests.
- [ ] 0.4.3. Implement web-ready export.
  - [ ] Strip all Gauss metadata on export.
  - [ ] Produce valid, minimal SVG.

### 0.5. Tool framework

- [ ] 0.5.1. Define Tool trait.
  - [ ] Tools are FSMs driven by input events. See architecture §6.1.
  - [ ] Tools emit Commands, not direct state mutations.
- [ ] 0.5.2. Refactor existing draw mode to Tool trait.
  - [ ] Extract `PenTool` FSM from current implementation.
  - [ ] Maintain existing functionality.
- [ ] 0.5.3. Refactor existing manipulate mode to Tool trait.
  - [ ] Extract `SelectTool` FSM.
  - [ ] Handle selection, drag, and transform states.
- [ ] 0.5.4. Create shared hit-test service.
  - [ ] Deterministic hit testing for selection and hover.
  - [ ] Prepare for R-tree/BVH optimisation. See architecture §6.2.

### 0.6. A11yService skeleton

- [ ] 0.6.1. Create `A11yService` structure.
  - [ ] Build AccessKit tree from UI and document state.
  - [ ] Push incremental updates. See architecture §11.1.
- [ ] 0.6.2. Wire existing stable node IDs.
  - [ ] Connect pre-defined IDs in `accessibility.rs` to AccessKit.
  - [ ] Expose roles and labels for window chrome.
- [ ] 0.6.3. Map AccessKit action requests to Gauss Actions.
  - [ ] Accessibility actions trigger the same command pipeline as UI.
  - [ ] Ensure keyboard-only operation parity.

### 0.7. i18n scaffolding

- [ ] 0.7.1. Create i18n module.
  - [ ] Define message catalog structure.
  - [ ] Evaluate Fluent vs simpler keyed system. See architecture §12.
- [ ] 0.7.2. Extract UI strings.
  - [ ] Replace inline strings with resource IDs.
  - [ ] Start with window chrome and tool names.
- [ ] 0.7.3. Localise command names.
  - [ ] Command names are user-visible (undo descriptions, scripting).
  - [ ] Ensure locale-aware formatting for numbers.

### 0.8. Widget capability audit

- [ ] 0.8.1. List required controls for Phase 1–2.
  - [ ] Toolbars, panels, layers, properties, colour pickers.
  - [ ] Document each control's requirements.
- [ ] 0.8.2. Map controls to gpui-component widgets.
  - [ ] Identify which widgets exist and are sufficient.
  - [ ] Flag controls needing custom implementation.
- [ ] 0.8.3. Plan custom widget development.
  - [ ] Bezier handle overlays (canvas-adjacent).
  - [ ] Gradient editor (Phase 4, may defer).
  - [ ] Ensure consistent focus, keyboard, and a11y behaviour.

______________________________________________________________________

## 1. MVP core editing tools and foundation

**Goal**: Deliver the essential drawing and editing experience on top of a
solid application framework. This phase completes the Phase 1 vision from the
feature plan while integrating the architectural foundations.

### 1.1. Shape drawing tools

- [ ] 1.1.1. Implement Rectangle tool.
  - [ ] Create `RectangleTool` FSM with drag-to-create interaction.
  - [ ] Support Shift constraint for squares.
  - [ ] Emit `InsertNode` command on completion. See architecture §7.1.
  - [ ] Add keyboard shortcut (`R`).
- [ ] 1.1.2. Implement Ellipse tool.
  - [ ] Create `EllipseTool` FSM with drag-to-create interaction.
  - [ ] Support Shift constraint for circles.
  - [ ] Add keyboard shortcut (`O`).
- [ ] 1.1.3. Implement Line Segment tool.
  - [ ] Create `LineTool` FSM for two-point line creation.
  - [ ] Add keyboard shortcut (`\`).
- [ ] 1.1.4. Extend document model for primitive shapes.
  - [ ] Add `NodeKind::Rect`, `NodeKind::Ellipse` variants. See
        architecture §5.3.
  - [ ] Ensure SVG export maps to `<rect>`, `<ellipse>` elements.
  - [ ] Ensure SVG import parses these elements.
- [ ] 1.1.5. Upgrade Pen tool.
  - [ ] Extend existing draw mode to support click-drag for Bezier handles.
  - [ ] Add explicit handle manipulation during path creation.

### 1.2. Selection tools

- [ ] 1.2.1. Implement marquee (rectangular) selection.
  - [ ] Add drag-to-select rectangle in manipulate mode.
  - [ ] Compute intersection with shape bounding boxes.
  - [ ] Support Shift modifier for additive selection.
- [ ] 1.2.2. Implement Direct Selection tool refinements.
  - [ ] Distinguish Selection tool (whole objects) from Direct Selection
        (anchors/handles).
  - [ ] Add keyboard shortcuts (`V` for Selection, `A` for Direct Selection).
- [ ] 1.2.3. Group selection support.
  - [ ] Implement grouping command (`Cmd+G` / `Ctrl+G`).
  - [ ] Implement ungrouping command (`Cmd+Shift+G` / `Ctrl+Shift+G`).
  - [ ] Extend document model for `NodeKind::Group`. See architecture §5.3.

### 1.3. Transformation and alignment

- [ ] 1.3.1. Implement bounding-box transform handles.
  - [ ] Render corner and edge handles on selected objects.
  - [ ] Handle drag for scale transformation.
  - [ ] Support Shift for proportional scaling.
  - [ ] Support Alt/Option for centre-origin scaling.
- [ ] 1.3.2. Implement rotation.
  - [ ] Add rotation handles outside bounding box corners.
  - [ ] Support Shift for 15-degree increments.
- [ ] 1.3.3. Implement numeric transform input.
  - [ ] Add Properties panel fields for X, Y, Width, Height, Rotation.
  - [ ] Apply changes as `SetTransform` commands. See architecture §7.1.
- [ ] 1.3.4. Implement alignment commands.
  - [ ] Align left, centre, right (horizontal).
  - [ ] Align top, middle, bottom (vertical).
  - [ ] Add menu entries and keyboard shortcuts.
- [ ] 1.3.5. Implement distribution commands.
  - [ ] Distribute horizontally and vertically.
- [ ] 1.3.6. Implement arrange commands. (Partially complete.)
  - [ ] Verify bring-to-front, send-to-back work correctly.
  - [ ] Add bring-forward, send-backward commands.

### 1.4. Fill and stroke (solid colours)

- [ ] 1.4.1. Create dedicated Stroke panel.
  - [ ] Display stroke colour, width, opacity.
  - [ ] Wire changes to `SetStyle` commands.
- [ ] 1.4.2. Create dedicated Fill panel.
  - [ ] Display fill colour and opacity.
  - [ ] Support "no fill" state.
- [ ] 1.4.3. Implement eyedropper tool.
  - [ ] Sample colour from canvas.
  - [ ] Apply to selected object's stroke or fill.
  - [ ] Add keyboard shortcut (`I`).

### 1.5. Layers panel

- [ ] 1.5.1. Create Layers panel UI.
  - [ ] List all shapes and groups hierarchically.
  - [ ] Display thumbnails or icons per layer.
- [ ] 1.5.2. Implement visibility toggle.
  - [ ] Add eye icon per layer.
  - [ ] Store visibility state in document model.
- [ ] 1.5.3. Implement lock toggle.
  - [ ] Add lock icon per layer.
  - [ ] Prevent selection and editing of locked layers.
- [ ] 1.5.4. Implement drag-to-reorder in panel.
  - [ ] Emit `ReorderChildren` commands. See architecture §7.1.
- [ ] 1.5.5. Implement layer naming.
  - [ ] Double-click to rename.
  - [ ] Store names in document model.

### 1.6. Undo/redo and history

(Partially complete. The dual history stack is implemented. Document history
has been migrated to the model-layer `DocumentUndoHistory` adapter backed by
`undo_2` — see [ADR-002](adr-002-undo-history-crate-selection.md). History
depth is bounded by `keep_last()` with a configurable limit, and A/B testing
infrastructure exists via the `undo_2` API — see ADR-002 for details. Selection
history remains on `gpui_component::History`.) In scope: grouping audit,
optional History panel. Out of scope: preview operation interaction with
history.

- [ ] 1.6.1. Audit history grouping. Requires 0.3.2.
  - [ ] Ensure multistep interactions create single undo entries.
        Completion: every drag and compound tool interaction produces
        exactly one undo entry; verified by parameterized tests.
  - [x] Verify history clears correctly on document open.
        See architecture §7.2.
- [ ] 1.6.2. Add History panel (optional).
  - [ ] Display list of recent operations.
  - [ ] Allow clicking to revert to a specific state.

### 1.7. File I/O and SVG

(Partially complete. Path-only round-trip works.)

- [ ] 1.7.1. Extend SVG import for new shape elements.
  - [ ] Parse `<rect>`, `<ellipse>`, `<circle>`, `<line>`, `<polyline>`,
        `<polygon>`.
  - [ ] Parse `<g>` groups with transforms.
- [ ] 1.7.2. Extend SVG export for new shape elements.
  - [ ] Emit native SVG elements where applicable.
  - [ ] Maintain deterministic output. See architecture §2.3.
- [ ] 1.7.3. Define Gauss metadata namespace.
  - [ ] Store editor-only data in `gauss:` namespaced attributes or
        `<metadata>` block. See architecture §10.1.
  - [ ] Implement "Export web-ready SVG" command that strips metadata.
- [ ] 1.7.4. Fix canvas size derivation.
  - [ ] Derive viewBox from document bounds. (TODO in `file_dialogs.rs`.)
  - [ ] Support explicit artboard sizing.

### 1.8. Cross-platform UI framework

(Partially complete. GPUI + gpui-component shell exists.)

- [ ] 1.8.1. Conduct widget capability audit.
  - [ ] List all required controls for Phase 1–2.
  - [ ] Map each to gpui-component widget or "needs custom".
        See architecture §14.1.
- [ ] 1.8.2. Implement missing custom widgets.
  - [ ] Gradient editor (for Phase 4). May defer.
  - [ ] Bezier handle overlays (partial—canvas handles exist).
- [ ] 1.8.3. Verify high-DPI rendering on all platforms.
  - [ ] Test on Linux (X11, Wayland), FreeBSD, Windows, macOS.

### 1.9. Accessibility integration

(Framework ready; AccessKit not wired.)

- [ ] 1.9.1. Wire AccessKit tree for UI chrome.
  - [ ] Connect pre-defined node IDs to AccessKit. See architecture §11.1.
  - [ ] Expose roles and labels for all buttons, menus, panels.
- [ ] 1.9.2. Ensure keyboard-only operation.
  - [ ] Audit all features for keyboard equivalents.
  - [ ] Add focus indicators for all interactive elements.
- [ ] 1.9.3. Make canvas focusable and described.
  - [ ] Expose canvas as single accessible node with descriptive label.
  - [ ] Plan for per-object navigation in later phases.
- [ ] 1.9.4. Test with screen readers.
  - [ ] NVDA and JAWS on Windows.
  - [ ] VoiceOver on macOS.
  - [ ] Orca on Linux.

### 1.10. Performance baseline

- [ ] 1.10.1. Establish performance profiling infrastructure.
  - [ ] Add frame-time measurement.
  - [ ] Set up benchmarks for common operations.
- [ ] 1.10.2. Define performance budgets.
  - [ ] Target 60fps for basic drawing operations.
  - [ ] Target sub-100ms latency for input response.
- [ ] 1.10.3. Profile and optimise critical paths.
  - [ ] Identify bottlenecks in canvas rendering.
  - [ ] Optimise hit testing (consider R-tree or BVH). See architecture §6.2.

### 1.11. Scripting interface initialisation

- [ ] 1.11.1. Embed RustPython interpreter.
  - [ ] Add `rustpython` dependency.
  - [ ] Initialize interpreter on application start.
- [ ] 1.11.2. Design scripting API surface.
  - [ ] Define `gauss.app`, `gauss.doc`, `gauss.commands`, `gauss.selection`
        modules. See architecture §13.2.
  - [ ] Ensure all UI actions are callable from scripts.
- [ ] 1.11.3. Implement basic script execution.
  - [ ] Add script console or command palette.
  - [ ] Execute Python code that creates shapes.
- [ ] 1.11.4. Document scripting API.
  - [ ] Write API reference.
  - [ ] Provide example scripts.

______________________________________________________________________

## 2. Typography and text tools

**Goal**: Introduce text capabilities for layouts, logos, and diagrams.

### 2.1. Basic type tool

- [ ] 2.1.1. Implement Type tool.
  - [ ] Click to create point text.
  - [ ] Click-drag to create area text (bounded text box).
  - [ ] Add keyboard shortcut (`T`).
- [ ] 2.1.2. Integrate text rendering engine.
  - [ ] Evaluate: cosmic-text, fontdue, swash, or platform text APIs.
  - [ ] Handle font discovery and loading.
  - [ ] Record decision in ADR. See architecture §18.
- [ ] 2.1.3. Implement on-canvas text editing.
  - [ ] Double-click text to enter edit mode.
  - [ ] Cursor, selection, clipboard support.

### 2.2. Typography controls

- [ ] 2.2.1. Create Character panel.
  - [ ] Font family selector (list system fonts).
  - [ ] Font size control.
  - [ ] Basic styles: bold, italic, underline.
- [ ] 2.2.2. Create Paragraph panel.
  - [ ] Text alignment: left, centre, right, justify.
  - [ ] Line spacing control.
- [ ] 2.2.3. Apply text colour via existing colour picker.

### 2.3. Text in document model

- [ ] 2.3.1. Add `NodeKind::Text` variant. See architecture §5.3.
- [ ] 2.3.2. Implement SVG export for text.
  - [ ] Emit `<text>` elements with proper attributes.
- [ ] 2.3.3. Implement SVG import for text.
  - [ ] Parse `<text>` and `<tspan>` elements.
- [ ] 2.3.4. Implement "Convert to outlines" command.
  - [ ] Convert text to vector paths for interoperability.

### 2.4. Text accessibility

- [ ] 2.4.1. Expose text objects to AccessKit.
  - [ ] Register text as accessible nodes with content.
  - [ ] Note AccessKit rich text limitations. See architecture §11.3.
- [ ] 2.4.2. Ensure keyboard navigation in text editing mode.

### 2.5. Internationalisation scaffolding

- [ ] 2.5.1. Set up localisation framework.
  - [ ] Evaluate: Fluent, gettext, or custom solution.
  - [ ] Extract all UI strings to resource files. See architecture §12.
- [ ] 2.5.2. Verify Unicode input.
  - [ ] Test accented characters, non-Latin scripts.
- [ ] 2.5.3. Provide one test locale.
  - [ ] Translate UI to Spanish or another language for validation.

### 2.6. Scripting for text

- [ ] 2.6.1. Extend scripting API for text operations.
  - [ ] Create text objects from scripts.
  - [ ] Set content, font, size, colour programmatically.

______________________________________________________________________

## 3. Path effects and advanced shape operations

**Goal**: Expand capabilities with boolean operations, distortion tools, and
creative effects that define professional illustration workflows.

### 3.1. Pathfinder boolean operations

- [ ] 3.1.1. Evaluate geometry library.
  - [ ] Consider: lyon, boolean-operation crates.
  - [ ] Record decision in ADR.
- [ ] 3.1.2. Implement Union (Add) operation.
- [ ] 3.1.3. Implement Subtract (Minus Front) operation.
- [ ] 3.1.4. Implement Intersect operation.
- [ ] 3.1.5. Implement Exclude operation.
- [ ] 3.1.6. Create Pathfinder panel or menu.
  - [ ] Add keyboard shortcuts for common operations.

### 3.2. Liquify distortion tools

- [ ] 3.2.1. Create brush engine framework.
  - [ ] Track brush radius, strength, position.
  - [ ] Apply geometric transforms to points within radius.
- [ ] 3.2.2. Implement Warp tool.
  - [ ] Drag points in brush direction.
- [ ] 3.2.3. Implement Twirl tool.
  - [ ] Rotate points around cursor.
- [ ] 3.2.4. Implement Pucker tool.
  - [ ] Move points toward centre.
- [ ] 3.2.5. Implement Bloat tool.
  - [ ] Move points away from centre.
- [ ] 3.2.6. Implement remaining liquify tools.
  - [ ] Scallop, Crystallize, Wrinkle. (Lower priority.)
- [ ] 3.2.7. Add tool options UI.
  - [ ] Brush size and intensity controls.

### 3.3. Envelope and warp effects

- [ ] 3.3.1. Design effects framework.
  - [ ] Objects can have effect stacks. See feature plan Phase 3.
  - [ ] Effects are non-destructive and undoable.
- [ ] 3.3.2. Implement parametric warp effects.
  - [ ] Arc, bulge, flag, wave presets.
  - [ ] Adjustable parameters.
- [ ] 3.3.3. Implement envelope distortion.
  - [ ] Define envelope shape.
  - [ ] Map object geometry to envelope.

### 3.4. Blend tool

- [ ] 3.4.1. Implement basic blend between two shapes.
  - [ ] Interpolate position, size, colour.
  - [ ] User-specified number of steps.
- [ ] 3.4.2. Implement blend along spine.
  - [ ] Distribute blend steps along a path.
- [ ] 3.4.3. Store blends in document model.
  - [ ] Blends as special group objects.
  - [ ] Regenerate on source edit.

### 3.5. Advanced selection aids

- [ ] 3.5.1. Implement Magic Wand tool.
  - [ ] Select objects with similar fill colour.
  - [ ] Configurable tolerance.
  - [ ] Add keyboard shortcut (`Y`).
- [ ] 3.5.2. Implement Measure tool.
  - [ ] Display distance and angle between two points.
- [ ] 3.5.3. Implement smart guides.
  - [ ] Show alignment hints during drag.
  - [ ] Snap to guides, edges, centres.

### 3.6. Scripting for path effects

- [ ] 3.6.1. Extend scripting API for boolean operations.
- [ ] 3.6.2. Extend scripting API for distortions.
- [ ] 3.6.3. Extend scripting API for blends.

______________________________________________________________________

## 4. Colour, appearance, and visual effects

**Goal**: Implement high-fidelity colouring tools, transparency, and visual
effects for professional illustration polish.

### 4.1. Gradient fills

- [ ] 4.1.1. Extend style model for gradients.
  - [ ] Linear and radial gradient types.
  - [ ] Gradient stops with position and colour.
- [ ] 4.1.2. Create Gradient Editor widget.
  - [ ] Custom GPUI component for stop manipulation.
  - [ ] Add, remove, reposition colour stops.
- [ ] 4.1.3. Implement Gradient tool.
  - [ ] Click-drag on canvas to set gradient vector.
  - [ ] Add keyboard shortcut (`G`).
- [ ] 4.1.4. Implement SVG gradient support.
  - [ ] Export to `<linearGradient>`, `<radialGradient>`.
  - [ ] Import gradient definitions.

### 4.2. Gradient mesh

- [ ] 4.2.1. Evaluate implementation approach.
  - [ ] Mesh grid representation.
  - [ ] Rendering via triangulation or patches.
  - [ ] Record decision in ADR.
- [ ] 4.2.2. Implement Mesh tool.
  - [ ] Convert shape to mesh.
  - [ ] Add mesh rows and columns.
- [ ] 4.2.3. Implement mesh point manipulation.
  - [ ] Move mesh points.
  - [ ] Assign colours to mesh points.
- [ ] 4.2.4. Address SVG mesh export.
  - [ ] SVG 2 mesh gradient (experimental).
  - [ ] Fallback: rasterize or approximate.

### 4.3. Advanced stroke styles

- [ ] 4.3.1. Implement dashed lines.
  - [ ] Dash and gap pattern input.
  - [ ] SVG `stroke-dasharray` export.
- [ ] 4.3.2. Implement arrowheads.
  - [ ] Start and end marker selection.
  - [ ] Built-in arrow shapes.
  - [ ] SVG `<marker>` export.
- [ ] 4.3.3. Implement stroke profiles (optional).
  - [ ] Variable width along path.

### 4.4. Pattern fills

- [ ] 4.4.1. Implement pattern swatch application.
  - [ ] Built-in pattern library (stripes, dots, etc.).
  - [ ] Apply pattern as fill.
- [ ] 4.4.2. Implement pattern definition.
  - [ ] Select objects and define as pattern.
  - [ ] Store in Swatches panel.
- [ ] 4.4.3. Implement SVG pattern export.
  - [ ] Emit `<pattern>` elements.

### 4.5. Transparency and blending

- [ ] 4.5.1. Implement object opacity.
  - [ ] 0–100% opacity slider.
  - [ ] Apply to selected objects.
- [ ] 4.5.2. Implement blend modes.
  - [ ] Multiply, Screen, Overlay, etc.
  - [ ] Blend mode selector in Transparency panel.
- [ ] 4.5.3. Create Transparency panel.
  - [ ] Opacity and blend mode controls.
- [ ] 4.5.4. Implement group isolation mode.
  - [ ] Blend modes apply within group only.

### 4.6. Visual effects (filters)

- [ ] 4.6.1. Implement Drop Shadow effect.
  - [ ] Offset, blur, colour parameters.
  - [ ] GPU shader implementation.
- [ ] 4.6.2. Implement Gaussian Blur effect.
  - [ ] Blur radius parameter.
- [ ] 4.6.3. Implement Outer Glow / Feather effect.
- [ ] 4.6.4. Create Appearance panel.
  - [ ] Add/remove effects on objects.
  - [ ] Reorder effect stack.
- [ ] 4.6.5. Implement SVG filter export.
  - [ ] Map effects to SVG filter primitives.

### 4.7. Clipping and masking

- [ ] 4.7.1. Implement clipping masks.
  - [ ] "Make Clipping Mask" command.
  - [ ] Top object clips objects below.
- [ ] 4.7.2. Implement opacity masks.
  - [ ] Grayscale mask controls transparency.
- [ ] 4.7.3. Implement SVG mask export.
  - [ ] Emit `<clipPath>`, `<mask>` elements.

### 4.8. Swatches panel

- [ ] 4.8.1. Create unified Swatches panel.
  - [ ] Solid colours, gradients, patterns.
  - [ ] Save and apply swatches.
- [ ] 4.8.2. Implement swatch import/export.
  - [ ] ASE, ACO format support (optional).

### 4.9. Performance optimisation

- [ ] 4.9.1. Profile gradient and effect rendering.
  - [ ] Identify GPU bottlenecks.
- [ ] 4.9.2. Implement render caching.
  - [ ] Cache effect results per object.
  - [ ] Invalidate on change. See architecture §9.2.
- [ ] 4.9.3. Implement level-of-detail reduction.
  - [ ] Simplify effects during interaction.

______________________________________________________________________

## 5. Symbols, reuse, and power-user tools

**Goal**: Implement features that enhance reuse, productivity, and
automation—differentiators for professional workflows.

### 5.1. Symbols and symbol libraries

- [ ] 5.1.1. Implement Symbol creation.
  - [ ] Select objects, create symbol from selection.
  - [ ] Store symbol definition in document. See architecture §5.3.
- [ ] 5.1.2. Implement Symbol instances.
  - [ ] Place instances that reference master symbol.
  - [ ] Edit master updates all instances.
- [ ] 5.1.3. Create Symbols panel.
  - [ ] List symbols with thumbnails.
  - [ ] Drag to place instances.
- [ ] 5.1.4. Implement symbol commands.
  - [ ] Redefine symbol.
  - [ ] Break link (expand to objects).
  - [ ] Replace symbol.
- [ ] 5.1.5. Implement SVG symbol export.
  - [ ] Emit `<symbol>` and `<use>` elements.

### 5.2. Symbolism tools

- [ ] 5.2.1. Implement Symbol Sprayer.
  - [ ] Spray symbol instances with brush.
  - [ ] Density and spacing controls.
- [ ] 5.2.2. Implement Symbol Shifter.
  - [ ] Move instances with brush.
- [ ] 5.2.3. Implement Symbol Sizer.
  - [ ] Scale instances with brush.
- [ ] 5.2.4. Implement Symbol Spinner.
  - [ ] Rotate instances with brush.
- [ ] 5.2.5. Implement remaining symbolism tools. (Lower priority.)
  - [ ] Stainer, Screener, Styler.

### 5.3. Graphic styles

- [ ] 5.3.1. Implement style capture.
  - [ ] Save appearance (fill, stroke, effects) as style.
- [ ] 5.3.2. Create Styles panel.
  - [ ] List saved styles.
  - [ ] Apply style to selected objects.
- [ ] 5.3.3. Implement style update propagation.
  - [ ] Edit style, apply to all linked objects.

### 5.4. Brush enhancements

- [ ] 5.4.1. Implement calligraphic brushes.
  - [ ] Angle and roundness parameters.
  - [ ] Pressure sensitivity (if tablet available).
- [ ] 5.4.2. Implement art brushes.
  - [ ] Stretch single shape along path.
- [ ] 5.4.3. Implement pattern brushes.
  - [ ] Tile pattern along path.
  - [ ] Handle corners.
- [ ] 5.4.4. Create Brushes panel.
  - [ ] Select and manage brushes.

### 5.5. Charts and graphs (optional)

- [ ] 5.5.1. Evaluate scope.
  - [ ] Determine if graph tools are in scope for parity.
- [ ] 5.5.2. Implement basic chart types.
  - [ ] Bar, line, pie charts.
  - [ ] Data input UI.

### 5.6. Data-driven graphics

- [ ] 5.6.1. Implement Variables panel.
  - [ ] Mark object properties as variables (text content, fill colour, etc.).
- [ ] 5.6.2. Implement data source import.
  - [ ] CSV and JSON support.
- [ ] 5.6.3. Implement batch generation.
  - [ ] Generate outputs for each data record.
  - [ ] Export multiple files.
- [ ] 5.6.4. Integrate with scripting.
  - [ ] Script-driven data merge.

### 5.7. Scripting and LLM control

- [ ] 5.7.1. Finalize scripting API.
  - [ ] Document all modules and functions.
  - [ ] Ensure coverage of all features.
- [ ] 5.7.2. Create scripting IDE / console.
  - [ ] In-app script editor.
  - [ ] Run and debug scripts.
- [ ] 5.7.3. Explore LLM integration. (Experimental.)
  - [ ] Natural language command palette.
  - [ ] LLM translates requests to script calls.
  - [ ] See architecture §13.3.

### 5.8. Cross-platform quality

- [ ] 5.8.1. Set up cross-platform test matrix.
  - [ ] Linux (Fedora, Ubuntu), FreeBSD, Windows, macOS.
  - [ ] Include low-end hardware configurations.
- [ ] 5.8.2. Address platform-specific issues.
  - [ ] Font handling differences.
  - [ ] Filesystem path conventions.
- [ ] 5.8.3. Refactor for code reuse.
  - [ ] Identify and unify duplicated patterns.

______________________________________________________________________

## 6. Final polish, parity checks, and format evaluation

**Goal**: Complete feature parity, polish the product, and evaluate long-term
format strategy.

### 6.1. Feature parity audit

- [ ] 6.1.1. Audit against Illustrator 10 feature list.
  - [ ] Verify all major features implemented or consciously excluded.
- [ ] 6.1.2. Implement missing minor tools.
  - [ ] Artboard / canvas size tool.
  - [ ] Measure tool (if not done in Phase 3).
  - [ ] Any remaining overlooked utilities.
- [ ] 6.1.3. Document excluded features.
  - [ ] Flare tool (low value).
  - [ ] Slice tool (less relevant today).
  - [ ] Graph tools (if excluded).

### 6.2. User experience polish

- [ ] 6.2.1. Audit all icons and tooltips.
  - [ ] Replace placeholder icons.
  - [ ] Ensure tooltips are present and accurate.
- [ ] 6.2.2. Finalise keyboard shortcuts.
  - [ ] Match familiar defaults where possible.
  - [ ] Document complete shortcut reference.
- [ ] 6.2.3. Implement panel arrangement.
  - [ ] Dock, undock, collapse panels.
  - [ ] Save and restore layout.
- [ ] 6.2.4. Implement preferences dialog.
  - [ ] Keyboard shortcuts customisation.
  - [ ] Appearance settings.

### 6.3. Accessibility finalisation

- [ ] 6.3.1. Conduct accessibility audit.
  - [ ] All controls have labels and roles.
  - [ ] Focus order is logical.
  - [ ] Colour contrast meets WCAG 2.1 AA.
- [ ] 6.3.2. Test with screen readers.
  - [ ] Document known limitations.
  - [ ] File upstream issues for AccessKit gaps.
- [ ] 6.3.3. Document accessibility features.
  - [ ] User guide for assistive technology users.

### 6.4. Localisation finalisation

- [ ] 6.4.1. Complete translation infrastructure.
  - [ ] All strings externalized.
  - [ ] Plural and gender support where needed.
- [ ] 6.4.2. Provide initial translations.
  - [ ] At least one non-English locale.
- [ ] 6.4.3. Test RTL layout.
  - [ ] Arabic or Hebrew UI.
  - [ ] Verify panel and text layout.

### 6.5. Performance tuning

- [ ] 6.5.1. Profile with large real-world files.
  - [ ] Complex illustrations with many objects.
  - [ ] Heavy gradient and effect usage.
- [ ] 6.5.2. Optimise identified bottlenecks.
  - [ ] Rendering, hit testing, undo.
- [ ] 6.5.3. Monitor memory usage.
  - [ ] Detect and fix leaks.
  - [ ] Optimise large document handling.

### 6.6. File format evaluation

- [ ] 6.6.1. Assess SVG limitations.
  - [ ] Gradient meshes: SVG 2 support status.
  - [ ] Live effects: round-trip fidelity.
  - [ ] Variables and metadata: storage approach.
- [ ] 6.6.2. Evaluate alternative formats.
  - [ ] PDF 1.7 for advanced features.
  - [ ] Custom GaussDoc format (SVG + sidecar).
  - [ ] Record decision in ADR. See architecture §10.2.
- [ ] 6.6.3. Implement format strategy.
  - [ ] If needed: dual save modes (pure SVG, Gauss document).
  - [ ] Ensure open documentation of any custom format.

### 6.7. Documentation and release preparation

- [ ] 6.7.1. Write user documentation.
  - [ ] Getting started guide.
  - [ ] Feature reference.
  - [ ] Scripting API reference.
- [ ] 6.7.2. Create example files.
  - [ ] Showcase each major feature.
  - [ ] Provide templates for common use cases.
- [ ] 6.7.3. Prepare release notes.
  - [ ] List capabilities relative to Illustrator 10.
  - [ ] Document known limitations.
- [ ] 6.7.4. Beta testing.
  - [ ] Recruit testers.
  - [ ] Collect and address feedback.

______________________________________________________________________

## Migration notes

The following changes or migrations are required to progress from the current
codebase:

### Document model evolution

The current `Document` uses a flat `Vec<Shape>` structure. The architecture
document (§5.3) describes a hierarchical `Node` model with `NodeKind` variants
for different object types. Migration path:

1. Introduce `Node` and `NodeKind` types alongside existing `Shape`.
2. Migrate `Shape` to `NodeKind::Path`.
3. Add `NodeKind::Group`, `NodeKind::Rect`, `NodeKind::Ellipse`, etc.
4. Replace `Vec<Shape>` with `NodeStore` using generational IDs.
5. Update operations, selection, and rendering accordingly.

### SVG I/O expansion

The current SVG parser handles only `<path>` elements with absolute commands.
Expansion required:

1. Add parsing for `<rect>`, `<ellipse>`, `<circle>`, `<line>`, `<polyline>`,
   `<polygon>`.
2. Add parsing for `<g>` groups with transform inheritance.
3. Add parsing for `<text>` and `<tspan>` (Phase 2).
4. Add parsing for gradient definitions (Phase 4).
5. Define Gauss namespace for metadata and implement stripping for web export.

### Accessibility wiring

The accessibility framework has stable node IDs defined but is not connected to
AccessKit. Steps:

1. Implement `A11yService` as described in architecture §11.1.
2. Wire existing node IDs to AccessKit tree.
3. Expose roles and labels for UI chrome.
4. Add incremental update mechanism.

### Scripting integration

RustPython is not yet integrated. Steps:

1. Add `rustpython` crate dependency.
2. Create `gauss-script` module per architecture §17.
3. Define Python API surface per architecture §13.2.
4. Ensure all actions are callable from scripts.

### Crate extraction (optional)

The current codebase is a single crate with `crates/test_support/`. The
architecture document (§17) recommends extraction into workspace crates:

- `gauss-core`: document, selection, viewport, commands, tools.
- `gauss-geometry`: bezier math, hit testing, booleans.
- `gauss-svg`: SVG parse/serialize.
- `gauss-render`: scene extraction, caching, draw adapters.
- `gauss-a11y`: AccessKit integration.
- `gauss-script`: RustPython host.

This extraction can be performed incrementally as the codebase grows.
