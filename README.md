# Gauss

Gauss is a Phase 0 proof-of-concept vector editor built with GPUI. It provides
Draw and Manipulate modes, SVG import/export, and undo/redo with a separate
selection history.

## Requirements

- Rust toolchain: `nightly-2025-10-23` (temporary, see
  `docs/execplans/phase-0-poc.md` for rationale).
- macOS or Linux (GPUI currently targets these platforms).

## Run

    cargo run

## Quality gates

Run the required quality gates before committing changes:

    make fmt
    make check-fmt
    make lint
    make test

Or run them all at once:

    make all

## Documentation

- Phase 0 plan: `docs/execplans/phase-0-poc.md`
- GPUI rustdoc: `docs/rustdoc-gpui-0.2.2/`
