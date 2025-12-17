# Suggested commands (Gauss)

Run commands from the repository root.

## Discover available Make targets

    make help

## Build

    make build

## Format (apply)

    make fmt

## Format (check only)

    make check-fmt

## Lint

    make lint

## Tests

    make test

## Markdown lint

    make markdownlint

## Mermaid validation

    make nixie

## “All gates” (what CI expects)

    make all

## Capturing output (recommended for long-running gates)

To avoid losing output to truncation and to preserve exit codes:

    set -o pipefail
    (make all) 2>&1 | tee /tmp/gauss-make-all.log
    echo "exit=$?"
