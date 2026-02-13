# Implement metadata round-trip (0.4.2)

Status: IN PROGRESS

## Progress

- [x] Reviewed roadmap and architecture requirements for 0.4.2.
- [x] Drafted this ExecPlan.
- [ ] Stage A: Model changes — add shape metadata fields.
- [ ] Stage B: Export changes — emit metadata on save.
- [ ] Stage C: Import changes — parse metadata on load.
- [ ] Stage D: Wire metadata through save/load pipeline.
- [ ] Stage E: Golden test infrastructure.
- [ ] Stage F: Unit tests.
- [ ] Stage G: BDD tests.
- [ ] Stage H: GPUI integration tests.
- [ ] Stage I: Documentation.

## Surprises & Discoveries

(To be updated during implementation.)

## Decision Log

- Decision: persist `gauss:id`, `gauss:name`, `gauss:locked`,
  `gauss:hidden` as namespaced attributes on `<path>` elements.
  Rationale: local to the element, simple to parse, follows SVG
  namespace conventions.
  Date/Author: 2026-02-12 (assistant, draft)
- Decision: encode `ShapeId` as 16-character zero-padded lowercase
  hex of the `KeyData::as_ffi()` u64 value. Rationale: compact,
  unambiguous, leverages existing `as_ffi()`/`from_ffi()` round-trip
  proven in AccessKit code.
  Date/Author: 2026-02-12 (assistant, draft)
- Decision: preserve entire `<metadata>` block content verbatim, not
  just Gauss-namespaced parts. Rationale: respects third-party
  metadata (Dublin Core, Inkscape), simplest implementation, maximum
  fidelity.
  Date/Author: 2026-02-12 (user decision)
- Decision: store unknown `gauss:*` attributes as
  `Vec<(String, String)>` for forward-compatible round-trip.
  Rationale: future Gauss versions may add new attributes; current
  version should not drop them.
  Date/Author: 2026-02-12 (assistant, draft)
