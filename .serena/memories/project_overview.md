# Gauss: project overview

## Purpose

`gauss` is currently a minimal Rust crate scaffold (generated via Copier) that
will evolve into a proof-of-concept (PoC) vector editor.

## Tech stack

- Language: Rust (edition 2024)
- Toolchain: pinned via `rust-toolchain.toml` (currently `nightly-2025-12-15`)
- Build system: Cargo, with a thin `Makefile` wrapper
- CI: GitHub Actions runs format, markdown lint, clippy, and tests/coverage

## Repository layout

- `src/lib.rs`: crate entrypoint (currently only `greet()`)
- `docs/`: project documentation (style, testing, scripting standards)
- `AGENTS.md`: contribution rules and quality gates
- `Makefile`: canonical developer commands

## Notes

- The codebase enforces strict lints via `Cargo.toml` and `clippy.toml`.
- New modules should start with a module-level `//!` comment.
