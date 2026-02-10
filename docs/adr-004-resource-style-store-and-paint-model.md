# Architectural decision record (ADR) 004: Add typed resource/style stores and paint references

## Status

Accepted (2026-02-07): Introduce `ResourceStore` and `StyleStore` in
`EngineState`, and represent stroke/fill as typed `Paint` values that can
reference shared gradient and pattern resources.

## Date

2026-02-07.

## Context and Problem Statement

Roadmap item 0.2.3 requires architecture foundations that allow SVG `<defs>`
resources (gradients, patterns, symbols) to round-trip through open/save while
preserving deterministic IDs. The previous model represented stroke/fill as
`Option<Rgba>`, which could not encode references to shared resources. We
needed a model that supports both simple solid colours and typed resource
references without coupling model code to UI concerns.

## Decision Drivers

- Preserve model-layer independence from the Gauss Platform UI (GPUI) layer.
- Support deterministic `url(#id)` round-trip import/export semantics.
- Keep shape styling ergonomic for solid-colour workflows.
- Provide stable typed IDs for resource/style references.
- Fail fast on dangling references rather than silently degrading output.

## Options Considered

### Option A: Keep `Option<Rgba>` paint model and add side maps in UI

Store resource references outside the model and interpret them during
import/export.

### Option B: Replace paint fields with typed `Paint` enum and add model stores

Model stroke/fill as `Paint` (`None`, `Solid`, `Gradient`, `Pattern`), add
`ResourceStore`/`StyleStore` to `EngineState`, and keep compatibility helper
constructors for existing solid-colour paths.

### Option C: Represent all paints as untyped SVG strings

Store literal SVG paint values and parse lazily at import/export boundaries.

| Topic                        | Option A | Option B | Option C |
| ---------------------------- | -------- | -------- | -------- |
| Typed safety in model        | Low      | High     | Low      |
| SVG round-trip determinism   | Medium   | High     | Medium   |
| Solid colour ergonomics      | High     | High     | Medium   |
| GPUI independence            | Medium   | High     | High     |
| Dangling-reference detection | Low      | High     | Low      |

_Table 1: Trade-offs for paint/resource modelling approaches._

## Decision Outcome / Proposed Direction

Adopt Option B.

- Add `ResourceStore` with typed IDs (`GradientId`, `PatternId`, `SymbolId`)
  and SVG-ID lookup maps.
- Add `StyleStore` with typed `StyleId`, unique naming behaviour, and optional
  default-style tracking.
- Extend `EngineState` with `resources` and `styles`.
- Model stroke/fill as `Paint` so shapes can carry solid colours or typed
  resource references.
- Export with validation for missing resource references and import with
  explicit errors for unresolved `url(#...)` references.

## Known Risks and Limitations

- Resource import now uses `roxmltree` for tag and attribute parsing, which is
  substantially more robust than manual string scanning, but the supported SVG
  subset is intentionally narrow and still omits broader SVG grammar support.
- Compatibility constructors require ongoing maintenance until all call sites
  fully adopt typed paint APIs.

## Architectural Rationale

This decision operationalizes architecture guidance in
`docs/gauss-architecture-design.md` §5.5 by establishing a typed model spine
for shared resources and future colour/effect features, while preserving
incremental migration paths for existing workflows.
