//! Controlled child-process fixtures for the pinned build-tool scripts.
//!
//! Candidate inputs are copied below target; cases inspect exact fake calls,
//! stdin, statuses, and capability-owned filesystem observations.

/// Exercises pinned tool installation and isolation contracts.
mod install;
/// Holds the five detailed Bash isolation procedures for install cases.
mod install_isolation;
/// Supplies independent build-tool expectations and fallible observation checks.
mod install_oracles;
mod model;
/// Owns exact expected observations and complete evidence comparisons.
mod observation_oracles;
/// Exercises preflight and toolchain input contracts.
mod preflight;
/// Holds preflight trace helpers and missing-tool scenario setup.
mod preflight_oracles;
mod process;
/// Validates controlled environment inputs before child launch.
mod process_inputs;
mod protocol;
mod support;

use model::{
    Controls, Entry, EnvironmentChange, Failure, Host, InjectedFunction, InjectionMode, Invocation,
    Observation, PayloadMode, PrefixState, Probe, ProbeRequest, Program, Request, RustupFixture,
};

const CANDIDATE_FILES: &[&str] = &[
    "scripts/build-tools-common.sh",
    "scripts/check-build-tools.sh",
    "scripts/install-build-tools.sh",
    "tools/mold/VERSION",
    "tools/mold/SHA256SUMS",
    "tools/rust-stable/VERSION",
    "rust-toolchain.toml",
];

const ALLOWED_UTILITIES: &[&str] = &["awk", "cat", "mkdir", "mktemp", "rm", "cp"];

const FAKE_COMMON: &str = r#"
record() {
  local tool=$1 argument
  shift
  builtin printf 'CALL\t%s\t%d' "$tool" "$#" >> "$GAUSS_CALL_LOG"
  for argument in "$@"; do
    case $argument in
      *$'\t'*|*$'\n'*) return 98 ;;
    esac
    builtin printf '\t%s' "$argument" >> "$GAUSS_CALL_LOG"
  done
  builtin printf '\n' >> "$GAUSS_CALL_LOG"
}
finish() {
  builtin printf 'STATUS\t%s\t%s\n' "$1" "$2" >> "$GAUSS_CALL_LOG"
  exit "$2"
}
next_ordinal() {
  local counter=$1 value=0
  if [[ -f $counter ]]; then
    IFS= read -r value < "$counter"
  fi
  [[ $value =~ ^[0-9]+$ ]] || return 98
  value=$((value + 1))
  builtin printf '%s\n' "$value" > "$counter"
  builtin printf '%s' "$value"
}
"#;

// The first fake action records argv, including rejected calls.
const FAKE_DIRNAME: &str = r#"#!/bin/bash -p
builtin printf 'CALL\tdirname\t%d' "$#" >> "$GAUSS_CALL_LOG"
for argument in "$@"; do
  case $argument in
    *$'\t'*|*$'\n'*) exit 98 ;;
  esac
  builtin printf '\t%s' "$argument" >> "$GAUSS_CALL_LOG"
done
builtin printf '\n' >> "$GAUSS_CALL_LOG"
source "$GAUSS_FAKE_COMMON"
if (($# != 2)) || [[ $1 != -- || $2 != /* ]]; then
  finish dirname 98
fi
builtin printf '%s\n' "${2%/*}"
finish dirname 0
"#;

const FAKE_UNAME: &str = r#"#!/bin/bash -p
source "$GAUSS_FAKE_COMMON"
record uname "$@" || finish uname 98
if [[ ${1:-} == -s && ${GAUSS_FAILURE_STAGE:-} == uname-system ]]; then
  finish uname 97
fi
if [[ ${1:-} == -m && ${GAUSS_FAILURE_STAGE:-} == uname-machine ]]; then
  finish uname 97
fi
case ${1:-} in
  -s) builtin printf '%s\n' "$GAUSS_HOST_SYSTEM" ;;
  -m) builtin printf '%s\n' "$GAUSS_HOST_MACHINE" ;;
  *) finish uname 98 ;;
esac
finish uname 0
"#;

const FAKE_CURL: &str = r#"#!/bin/bash -p
source "$GAUSS_FAKE_COMMON"
record curl "$@" || finish curl 98
[[ ${GAUSS_FAILURE_STAGE:-} != curl ]] || finish curl 97
output=
while (($#)); do
  if [[ $1 == --output && $# -ge 2 ]]; then
    output=$2
    shift 2
  else
    shift
  fi
done
[[ -n $output ]] || finish curl 98
builtin printf '%s\n' 'inert archive fixture' > "$output"
finish curl 0
"#;

const FAKE_SHA256SUM: &str = r#"#!/bin/bash -p
source "$GAUSS_FAKE_COMMON"
record sha256sum "$@" || finish sha256sum 98
cat > "$GAUSS_SHA_STDIN"
[[ ${GAUSS_FAILURE_STAGE:-} != checksum ]] || finish sha256sum 97
finish sha256sum 0
"#;

const FAKE_TAR: &str = r#"#!/bin/bash -p
source "$GAUSS_FAKE_COMMON"
record tar "$@" || finish tar 98
[[ ${GAUSS_FAILURE_STAGE:-} != tar ]] || finish tar 97
directory=
archive=
while (($#)); do
  case $1 in
    --directory) directory=$2; shift 2 ;;
    --file) archive=$2; shift 2 ;;
    *) shift ;;
  esac
done
[[ -n $directory && -n $archive ]] || finish tar 98
mkdir -p -- "$directory/bin" || finish tar 98
if [[ ${GAUSS_MOLD_PAYLOAD_MODE:-installed} == missing ]]; then
  finish tar 0
fi
cp -p -- "$GAUSS_MOLD_PAYLOAD" "$directory/bin/mold"
finish tar $?
"#;

const FAKE_RUSTUP: &str = r#"#!/bin/bash -p
source "$GAUSS_FAKE_COMMON"
record rustup "$@" || finish rustup 98
if [[ ${1:-} == toolchain && ${2:-} == install ]]; then
  ordinal=$(next_ordinal "$GAUSS_RUSTUP_INSTALL_COUNT") || finish rustup 98
  status_name=GAUSS_RUSTUP_INSTALL_STATUS_$ordinal
  status=${!status_name:-0}
elif [[ ${1:-} == toolchain && ${2:-} == list ]]; then
  ordinal=$(next_ordinal "$GAUSS_RUSTUP_LIST_COUNT") || finish rustup 98
  status_name=GAUSS_RUSTUP_LIST_STATUS_$ordinal
  output_name=GAUSS_RUSTUP_LIST_OUTPUT_$ordinal
  status=${!status_name:-0}
  output=${!output_name-}
  builtin printf '%s' "$output"
elif [[ ${1:-} == component && ${2:-} == list ]]; then
  ordinal=$(next_ordinal "$GAUSS_RUSTUP_COMPONENT_COUNT") || finish rustup 98
  status_name=GAUSS_RUSTUP_COMPONENT_STATUS_$ordinal
  output_name=GAUSS_RUSTUP_COMPONENT_OUTPUT_$ordinal
  status=${!status_name:-0}
  output=${!output_name-}
  builtin printf '%s' "$output"
else
  finish rustup 98
fi
finish rustup "$status"
"#;

pub(super) const FAKE_MOLD: &str = r#"#!/bin/bash -p
source "$GAUSS_FAKE_COMMON"
tool=mold
[[ $0 != "$GAUSS_PREFIX/bin/mold" ]] || tool=prefix-mold
record "$tool" "$@" || finish "$tool" 98
[[ ${1:-} == --version ]] || finish "$tool" 98
if [[ $tool == prefix-mold && ${GAUSS_FAILURE_STAGE:-} == installed-mold-version ]] ||
   [[ $tool == mold && ${GAUSS_FAILURE_STAGE:-} == mold-version ]]; then
  finish "$tool" 97
fi
builtin printf '%s\n' "${GAUSS_MOLD_VERSION_OUTPUT:-mold 2.41.0}"
finish "$tool" 0
"#;

const FAKE_CLANG: &str = r#"#!/bin/bash -p
source "$GAUSS_FAKE_COMMON"
record clang "$@" || finish clang 98
if [[ $# == 1 && $1 == --version ]]; then
  [[ ${GAUSS_FAILURE_STAGE:-} != clang-version ]] || finish clang 97
  builtin printf '%s\n' "${GAUSS_CLANG_VERSION_OUTPUT:-clang fixture}"
  finish clang 0
fi
cat > "$GAUSS_CLANG_STDIN"
[[ ${GAUSS_FAILURE_STAGE:-} != clang-syntax ]] || finish clang 97
finish clang 0
"#;

const FAKE_SENTINEL: &str = r#"#!/bin/bash -p
source "$GAUSS_FAKE_COMMON"
tool=${0##*/}
record "$tool" "$@" || finish "$tool" 98
finish "$tool" 97
"#;

pub(super) const FAKE_FILES: &[(&str, &str)] = &[
    ("dirname", FAKE_DIRNAME),
    ("uname", FAKE_UNAME),
    ("curl", FAKE_CURL),
    ("sha256sum", FAKE_SHA256SUM),
    ("tar", FAKE_TAR),
    ("rustup", FAKE_RUSTUP),
    ("mold", FAKE_MOLD),
    ("clang", FAKE_CLANG),
    ("cargo", FAKE_SENTINEL),
    ("rustc", FAKE_SENTINEL),
    ("cc", FAKE_SENTINEL),
    ("gcc", FAKE_SENTINEL),
    ("c++", FAKE_SENTINEL),
    ("clang++", FAKE_SENTINEL),
    ("make", FAKE_SENTINEL),
    ("cmake", FAKE_SENTINEL),
    ("ninja", FAKE_SENTINEL),
    ("git", FAKE_SENTINEL),
    ("wget", FAKE_SENTINEL),
    ("whitaker-installer", FAKE_SENTINEL),
];

pub(super) const FAKE_COMMON_FILE: &str = "fake/fake-common.sh";
pub(super) const FAKE_MOLD_PAYLOAD_FILE: &str = "fake/mold-payload";
pub(super) const STARTUP_FILE: &str = "fake/startup.sh";
const ENTRY_FUNCTIONS: &[InjectedFunction] = &[
    InjectedFunction::Curl,
    InjectedFunction::Uname,
    InjectedFunction::Printf,
    InjectedFunction::Command,
];
