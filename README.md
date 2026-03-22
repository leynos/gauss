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

## Build performance

Gauss uses a custom Cargo profile configuration to reduce dependency
compilation time. The `dev` and `test` profiles disable debuginfo for all
dependencies (`debug = 0` for `package."*"`), while first-party Gauss crates
retain full debuginfo.

### Trade-offs

- **Faster builds**: Dependency compilation is ~24% faster (measured on a clean
  build).
- **Limited third-party debugging**: Stack traces and debugger symbols for
  dependencies (GPUI, ash, naga, etc.) are reduced.
- **Gauss code remains debuggable**: First-party crates (gauss, gauss-core,
  gauss-svg) retain full debuginfo.

### Temporary escape hatch

If you need full debuginfo for dependencies during debugging:

1. Comment out the `[profile.dev.package."*"]` and
   `[profile.test.package."*"]` sections in `Cargo.toml`.
2. Run `cargo clean` and rebuild.
3. Restore the profile settings and clean build again when done.

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
