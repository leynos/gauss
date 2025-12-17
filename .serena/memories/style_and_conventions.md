# Style and conventions (Gauss)

## Source of truth

- `AGENTS.md` defines coding, documentation, and testing rules.
- `docs/documentation-style-guide.md` defines documentation grammar/spelling
  (en-GB-oxendict) and wrapping rules.

## Rust style

- Prefer small, single-responsibility functions.
- Avoid unnecessary `mut`.
- No `unsafe` unless unavoidable; document with a `SAFETY` comment.
- Public APIs should have `///` Rustdoc.
- Every module should begin with a module-level `//!` comment.

## Lints

- Clippy warnings are denied in CI (`make lint`).
- `unwrap()` / `expect()` are denied in production code by lint policy; in
  tests, `expect(...)` is preferred over `unwrap()`.

## Documentation

- Use en-GB-oxendict spelling in comments/docs unless matching an external API.
- Wrap Markdown paragraphs at 80 columns; code blocks at 120 columns.
- Use `-` for Markdown bullets.
