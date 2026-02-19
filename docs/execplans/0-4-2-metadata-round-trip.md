# Implement metadata round-trip (0.4.2)

Status: COMPLETE

## Progress

- [x] Reviewed roadmap and architecture requirements for 0.4.2.
- [x] Drafted this ExecPlan.
- [x] Stage A: Model changes — add shape metadata fields.
- [x] Stage B: Export changes — emit metadata on save.
- [x] Stage C: Import changes — parse metadata on load.
- [x] Stage D: Wire metadata through save/load pipeline.
- [x] Stage E: Golden test infrastructure.
- [x] Stage F: Unit tests.
- [x] Stage G: Behaviour-driven development (BDD) tests.
- [x] Stage H: Graphics processing unit interface (GPUI) integration tests.
- [x] Stage I: Documentation.

## Surprises and discoveries

- The fragment-based re-parser in `resource_tags.rs` wraps SVG fragments in a
  `<gauss-import-wrapper>` element without namespace declarations. After Stage
  B added `gauss:*` attributes to `<path>` elements, the fragment parser failed
  because `roxmltree` could not resolve the `gauss:` prefix. Fixed by adding
  `gauss_namespace_declaration()` to the wrapper element.
- The `<metadata>` block export needed conditional trailing-newline logic to
  achieve idempotent round-trip. If the content already ends with `\n`, the
  exporter must not add another; otherwise the second round-trip would gain an
  extra newline.
- Whitaker custom lint rules forbid `expect_used`, `unwrap_or_else` with panic
  closures, and `std::fs` operations. All test code was written using
  `match`/`panic!` patterns, `TestSupportResult<()>` return types, and
  `cap_std::fs_utf8::Dir` for filesystem access.

## Decision log

- Decision: persist `gauss:id`, `gauss:name`, `gauss:locked`,
  `gauss:hidden` as namespaced attributes on `<path>` elements. Rationale:
  local to the element, simple to parse, follows SVG namespace conventions.
  Date/Author: 2026-02-12 (assistant, draft)
- Decision: encode `ShapeId` as 16-character zero-padded lowercase
  hex of the `KeyData::as_ffi()` u64 value. Rationale: compact, unambiguous,
  leverages existing `as_ffi()`/`from_ffi()` round-trip proven in AccessKit
  code. Date/Author: 2026-02-12 (assistant, draft)
- Decision: preserve entire `<metadata>` block content verbatim, not
  just Gauss-namespaced parts. Rationale: respects third-party metadata (Dublin
  Core, Inkscape), simplest implementation, maximum fidelity. Date/Author:
  2026-02-12 (user decision)
- Decision: store unknown `gauss:*` attributes as
  `Vec<GaussAttribute>` for forward-compatible round-trip. Rationale: future
  Gauss versions may add new attributes; current version should not drop them.
  Date/Author: 2026-02-12 (assistant, draft)
