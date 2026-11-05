//! Byte-exact shell fixtures used by the controlled Make invocations.

/// Shell command used in place of Cargo and Whitaker by controlled Make runs.
pub(super) const CARGO_RECORDER: &str = r#"{
    printf 'CALL'
    for argument do printf '\t%s' "$argument"; done
    printf '\tENV_RUST_FLAGS=%s\tENV_RUSTFLAGS=%s\tENV_RUSTDOCFLAGS=%s\n' \
        "${RUST_FLAGS+present}" "${RUSTFLAGS-<unset>}" "${RUSTDOCFLAGS-<unset>}"
} >> "$GAUSS_MAKE_CONTRACT_LOG"

if [ "${1-}:${2-}" = "nextest:--version" ]; then
    [ "${GAUSS_NEXTTEST_AVAILABLE-}" = "yes" ]
    exit $?
fi

if [ "${GAUSS_FAIL_COMMAND-}:${1-}:${2-}" = "nextest-run:nextest:run" ]; then
    exit 23
fi

if [ "${1-}" = "test" ]; then
    case " $* " in
        *" --doc "*) [ "${GAUSS_FAIL_COMMAND-}" = "doctest" ] && exit 23 ;;
        *) [ "${GAUSS_FAIL_COMMAND-}" = "cargo-test" ] && exit 23 ;;
    esac
fi

# The lint gate's leaves share this recorder: Cargo runs `doc` and `clippy`,
# and Whitaker is invoked with `--all` before the forwarded Cargo arguments.
if [ "${1-}:${GAUSS_FAIL_COMMAND-}" = "clippy:clippy" ]; then
    exit 23
fi

if [ "${1-}:${GAUSS_FAIL_COMMAND-}" = "--all:whitaker" ]; then
    exit 23
fi

exit 0
"#;

/// Shell command that checks spelling setup cannot inherit `RUST_FLAGS`.
pub(super) const UV_NOOP: &str = "[ \"${RUST_FLAGS+x}\" != \"x\" ]\n";
/// Make variables a surrounding `make test` may forward to this child.
///
/// Removing them isolates the controlled run; recursive Make then forwards the
/// command-line assignments the controlled run generates for itself.
pub(super) const SURROUNDING_MAKE_VARIABLES: &[&str] = &[
    "MAKEFLAGS",
    "MAKEOVERRIDES",
    "MAKEFILES",
    "GNUMAKEFLAGS",
    "MFLAGS",
    "RUST_FLAGS",
    "CARGO",
    "UV",
    "WHITAKER",
    "WHITAKER_PACKAGES",
    "TEST_FLAGS",
    "CARGO_FLAGS",
    "CLIPPY_FLAGS",
    "BUILD_JOBS",
    "RUSTDOC_FLAGS",
    "NEXTEST_FLAGS",
    "MDTABLEFIX",
    "MDLINT",
    "NIXIE",
    "TYPOS_CONFIG_BUILDER",
];
/// Scratch-only goal that recurses through the candidate `test-ci` target.
pub(super) const RECURSIVE_FIXTURE: &str = concat!(
    "\n.PHONY: gauss-input-recursive\n",
    "gauss-input-recursive:\n",
    "\t+$(MAKE) --no-print-directory -f Makefile test-ci\n",
);
