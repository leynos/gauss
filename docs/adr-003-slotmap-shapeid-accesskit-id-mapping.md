# Architectural decision record (ADR) 003: Use slotmap for ShapeId and AccessKit ID mapping

## Status

Accepted (2026-01-19): Use `slotmap` generational keys for `ShapeId` and map
`ShapeId` values to AccessKit node IDs via `KeyData::as_ffi`/`from_ffi`.

## Date

2026-01-19.

## Context and Problem Statement

Gauss needs stable identifiers for document shapes so AccessKit can track
objects across frames. The model must support deletion and reuse of indices
without ID collisions, while still providing a deterministic `u64` mapping for
AccessKit node IDs. We need a strategy that preserves stable identifiers,
avoids bespoke allocation logic, and keeps the conversion path explicit for
testing and debugging.

## Decision Drivers

- Stable, generational identifiers across insert/remove and reorder.
- Simple conversion to and from `u64` AccessKit node IDs.
- Minimal bespoke allocation logic and low maintenance overhead.
- Clear, testable semantics for ID reuse.

## Options Considered

### Option A: `slotmap` generational keys

Use `slotmap` for ID allocation and expose a `ShapeId` newtype backed by a
`slotmap::Key`. Convert to AccessKit node IDs via `KeyData::as_ffi` and back via
`KeyData::from_ffi`.

### Option B: `generational-arena`

Use `generational-arena` for allocation and implement explicit conversion
helpers for AccessKit node IDs.

### Option C: `Uuid`-backed IDs

Continue using UUIDs and maintain a separate mapping table for AccessKit node
IDs.

| Topic                     | Slotmap | Generational-arena | UUID + map |
| ------------------------- | ------- | ------------------ | ---------- |
| Generational semantics    | Yes     | Yes                | No         |
| `u64` mapping ergonomics  | Yes     | Manual             | Manual     |
| Allocation plumbing       | Low     | Low                | High       |
| AccessKit stability tests | Simple  | Medium             | Medium     |

_Table 1: Trade-offs between allocation and mapping options._

## Decision Outcome / Proposed Direction

Adopt `slotmap` for `ShapeId` and implement `ShapeId` as a `slotmap` key
newtype. Use `slotmap::KeyData::as_ffi` to convert IDs into a stable `u64`
representation for AccessKit node IDs, and `KeyData::from_ffi` to reconstruct
`ShapeId` values from AccessKit nodes. This preserves generational stability,
keeps conversion helpers explicit, and avoids bespoke mapping tables.

## Known Risks and Limitations

- AccessKit mapping is tied to the `slotmap` key representation; changing key
  formats would require a migration.
- Conversion helpers must remain centralized to avoid divergent mappings.

## Architectural Rationale

The decision aligns with the architecture guidance in
`docs/gauss-architecture-design.md` §5.1 by ensuring stable, generational IDs
with explicit conversions for accessibility integration.
