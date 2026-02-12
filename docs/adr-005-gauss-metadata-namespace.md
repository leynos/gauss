# Architectural decision record (ADR) 005: Define Gauss metadata namespace policy

## Status

Accepted (2026-02-10): Define a canonical Gauss metadata namespace using
`xmlns:gauss="https://gauss.dev/ns/metadata/1"` and enforce namespace policy in
Scalable Vector Graphics (SVG) import/export.

## Date

2026-02-10.

## Context and Problem Statement

Roadmap item 0.4.1 requires Gauss to define a metadata namespace before broader
SVG metadata work in 0.4.2 and 0.4.3. Architecture section 10.1 states that
Gauss metadata must be stored in namespaced attributes or `<metadata>` while
preserving valid standard SVG rendering in other tools.

Without a canonical namespace contract, future metadata payload work risks
fragmented prefixes, inconsistent import behaviour, and non-deterministic
round-trips.

## Decision Drivers

- Keep visible artwork standard SVG and viewer-compatible.
- Provide one canonical namespace Uniform Resource Identifier (URI) and prefix
  for deterministic tooling.
- Keep import validation strict enough to detect malformed Gauss namespace
  usage.
- Keep 0.4.1 scoped to namespace policy, not payload schema definition.

## Requirements

### Functional requirements

- Exported SVG must include a canonical Gauss metadata namespace declaration.
- Canonical namespace declaration must be accepted on import.
- Files with invalid `gauss` prefix bindings must be rejected.
- Gauss namespace usage must be rejected when canonical `xmlns:gauss`
  declaration is missing.
- Behaviour must be covered by unit tests, behavioural tests, and Gauss UI
  framework tests (GPUI tests).

### Technical requirements

- Namespace constants must live in one reusable module.
- Import/export seams must be ready for 0.4.2 metadata payload round-trip work.
- No new external dependencies are required.

## Options considered

### Option A: Prefix-only convention without URI enforcement

Use `gauss:*` by convention and accept any `xmlns:gauss` URI.

### Option B: Canonical prefix and URI with import validation

Define one canonical URI and prefix, emit it on export, and validate it on
import.

### Option C: Metadata-only policy without namespaced attributes

Allow only `<metadata>` content and disallow namespaced attributes.

| Topic                              | Option A | Option B | Option C |
| ---------------------------------- | -------- | -------- | -------- |
| Deterministic namespace contract   | Low      | High     | Medium   |
| Compatibility with roadmap wording | High     | High     | Medium   |
| Import-time error detection        | Low      | High     | Medium   |
| Future extensibility for payloads  | Medium   | High     | Medium   |

_Table 1: Trade-offs for Gauss metadata namespace policy options._

## Decision Outcome / Proposed Direction

Adopt Option B.

- Canonical prefix: `gauss`.
- Canonical URI: `https://gauss.dev/ns/metadata/1`.
- Export always includes `xmlns:gauss="https://gauss.dev/ns/metadata/1"` on
  the SVG root.
- Import validates that:
  - if `gauss` prefix is declared, it points to the canonical URI,
  - if Gauss namespace URI is used by elements or attributes, canonical
    `xmlns:gauss` declaration exists.

This policy supports both roadmap representations:

- namespaced attributes (`gauss:*`), and
- namespaced metadata payload under `<metadata>`.

## Goals and Non-Goals

### Goals

- Define and enforce namespace identity for Gauss metadata.
- Provide deterministic import/export behaviour.
- Establish seams for later metadata payload round-trip work.

### Non-Goals

- Defining full metadata payload schema.
- Implementing metadata payload round-trip persistence (roadmap 0.4.2).
- Implementing metadata stripping for web-ready export (roadmap 0.4.3).

## Known Risks and Limitations

- Existing SVG files using the Gauss URI under non-`gauss` prefixes are now
  rejected to preserve canonical prefix semantics.
- This ADR defines namespace identity only; payload keys and value semantics
  remain future work.

## Architectural Rationale

This decision concretizes architecture section 10.1 by making metadata
namespacing explicit, testable, and deterministic while preserving standard SVG
compatibility for rendered artwork. It creates the architectural spine required
to complete roadmap items 0.4.2 and 0.4.3 safely.
