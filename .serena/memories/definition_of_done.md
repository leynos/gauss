# Definition of done (Gauss)

A change is considered done when:

- It has tests appropriate to the behaviour (unit and/or behavioural).
- `make check-fmt` passes.
- `make lint` passes (no clippy warnings; warnings are treated as errors).
- `make test` passes.
- Documentation in `docs/` is updated when requirements/architecture changes.

When running gates locally, prefer capturing output with `tee` and preserving
exit codes:

    set -o pipefail
    (make check-fmt) 2>&1 | tee /tmp/gauss-check-fmt.log
    echo "exit=$?"

    set -o pipefail
    (make lint) 2>&1 | tee /tmp/gauss-lint.log
    echo "exit=$?"

    set -o pipefail
    (make test) 2>&1 | tee /tmp/gauss-test.log
    echo "exit=$?"
