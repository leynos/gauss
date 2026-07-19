# Gauss

[![Ask DeepWiki](https://deepwiki.com/badge.svg)](
https://deepwiki.com/leynos/gauss)

## Spelling policy

Run `make spelling` to enforce en-GB-oxendict prose spelling. The generated
`typos.toml` starts from the shared estate dictionary, refreshes its untracked
local cache only when the authority is newer, and then applies the narrow
repository policy in `typos.local.toml`.

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

If you need full debuginfo for dependencies during debugging, use one of these
ephemeral approaches to avoid long-lived changes to the committed profile:

#### Option 1: Per-user config override (recommended)

Create or edit `.cargo/config.toml` in the repository root (gitignored):

    [profile.dev.package."*"]
    debug = 2

    [profile.test.package."*"]
    debug = 2

Then run `cargo clean` and rebuild. Remove `.cargo/config.toml` when done.

#### Option 2: Git stash workflow

1. Comment out the `[profile.dev.package."*"]` and
   `[profile.test.package."*"]` sections in `Cargo.toml`.
2. Run `cargo clean` and rebuild.
3. Stash the change with `git stash` when done (or restore manually).

#### Option 3: Throwaway commit

1. Make the profile change, rebuild, and debug.
2. Use `git reset --soft HEAD~1` to undo the commit without losing other work.

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
