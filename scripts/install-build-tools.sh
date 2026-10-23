#!/bin/bash
# Install the pinned published linker and repository Rust components.

if (($# != 0)); then
  builtin printf 'build-tools: unexpected arguments\n' >&2
  exit 1
fi

script_path=${BASH_SOURCE[0]}
[[ $script_path == /* ]] || script_path=$PWD/$script_path
script_dir=$(cd -- "$(dirname -- "$script_path")" && pwd)
# shellcheck source=scripts/build-tools-common.sh
. "$script_dir/build-tools-common.sh"

BUILD_TOOLS_WORKDIR=
BUILD_TOOLS_MOLD_PIN=
BUILD_TOOLS_STABLE_PIN=

cleanup_workdir() {
  if [[ -n $BUILD_TOOLS_WORKDIR ]]; then
    rm -rf -- "$BUILD_TOOLS_WORKDIR"
    BUILD_TOOLS_WORKDIR=
  fi
}

trap cleanup_workdir EXIT

require_command() {
  command -v "$1" >/dev/null 2>&1 ||
    fail "$1 not found on PATH; install it before running this script"
}

create_workdir() {
  if ! BUILD_TOOLS_WORKDIR=$(mktemp -d "${TMPDIR:-/tmp}/gauss-build-tools.XXXXXX"); then
    fail 'could not create a unique temporary directory for the mold download'
  fi
  [[ -n $BUILD_TOOLS_WORKDIR ]] ||
    fail 'could not create a unique temporary directory for the mold download'
  if [[ $BUILD_TOOLS_WORKDIR != /* ]]; then
    BUILD_TOOLS_WORKDIR=$PWD/$BUILD_TOOLS_WORKDIR
  fi
}

read_mold_checksum() {
  local name=$1 record count invalid digest
  [[ -f $MOLD_SHA256SUMS_FILE ]] ||
    fail "expected exactly one checksum for $name; checksum file is missing: $MOLD_SHA256SUMS_FILE"
  record=$(awk -v archive_name="$name" '
    $2 == archive_name {
      count++
      if (NF != 2 || length($1) != 64 || $1 !~ /^[[:xdigit:]]+$/) {
        invalid = 1
      } else {
        digest = $1
      }
    }
    END { printf "%d|%d|%s\n", count, invalid, digest }
  ' "$MOLD_SHA256SUMS_FILE") ||
    fail "invalid SHA-256 checksum file: $MOLD_SHA256SUMS_FILE"
  IFS='|' read -r count invalid digest <<< "$record"
  [[ $count == 1 ]] ||
    fail "expected exactly one checksum for $name in $MOLD_SHA256SUMS_FILE"
  [[ $invalid == 0 && -n $digest ]] ||
    fail "invalid SHA-256 checksum recorded for $name in $MOLD_SHA256SUMS_FILE"
  builtin printf '%s' "$digest"
}

download_and_verify_mold() {
  local name=$1 url=$2 digest=$3 archive=$4
  note "downloading the published mold archive $url"
  if ! curl --fail --silent --show-error --location \
    --connect-timeout 15 --speed-limit 1024 --speed-time 60 \
    --output "$archive" "$url"; then
    fail "failed to download $name"
  fi
  if ! builtin printf '%s  %s\n' "$digest" "$archive" |
    sha256sum --check --status; then
    fail "checksum mismatch for $name; refusing to install"
  fi
  note "verified $name against $MOLD_SHA256SUMS_FILE"
}

verify_installed_mold() {
  local version=$1 mold_path=$2 output first_line
  [[ -x $mold_path ]] || fail "installed mold is unusable: $mold_path is missing or not executable"
  if ! output=$("$mold_path" --version 2>&1); then
    fail "installed mold is unusable at $mold_path: $output"
  fi
  first_line=${output%%$'\n'*}
  if [[ $first_line != "mold $version" && $first_line != "mold $version "* ]]; then
    fail "installed mold version does not match $version at $mold_path: $first_line"
  fi
}

extract_mold_archive() {
  local version=$1 name=$2 archive=$3 mold_path
  mkdir -p -- "$BUILD_TOOLS_PREFIX" || fail "could not create prefix: $BUILD_TOOLS_PREFIX"
  if ! tar --extract --gzip --strip-components=1 --directory "$BUILD_TOOLS_PREFIX" \
    --file "$archive"; then
    fail "failed to unpack $name into $BUILD_TOOLS_PREFIX"
  fi
  mold_path=$BUILD_TOOLS_PREFIX/bin/mold
  verify_installed_mold "$version" "$mold_path"
}

install_mold() {
  local version=$1 name url digest archive
  if ! is_linux; then
    note "mold is Linux-only; skipping on $BUILD_TOOLS_HOST_OS"
    return 0
  fi
  name="mold-$version-$BUILD_TOOLS_MOLD_ARCH-linux.tar.gz"
  url="https://github.com/rui314/mold/releases/download/v$version/$name"
  digest=$(read_mold_checksum "$name")
  require_command curl
  require_command sha256sum
  require_command tar
  create_workdir
  archive=$BUILD_TOOLS_WORKDIR/$name
  download_and_verify_mold "$name" "$url" "$digest" "$archive"
  extract_mold_archive "$version" "$name" "$archive"
  note "installed mold $version into $BUILD_TOOLS_PREFIX"
  note "put $BUILD_TOOLS_PREFIX/bin first on PATH when invoking Cargo directly"
}

install_toolchain() {
  local toolchain=$1 component
  local -a component_args=()
  require_command rustup
  for component in "${BUILD_TOOLS_COMPONENTS[@]}"; do
    component_args+=(--component "$component")
  done
  note "installing toolchain $toolchain with: ${BUILD_TOOLS_COMPONENTS[*]}"
  if ! rustup toolchain install "$toolchain" --profile minimal "${component_args[@]}"; then
    fail "failed to install toolchain $toolchain and its components"
  fi
}

install_stable_toolchain() {
  local toolchain=$1
  require_command rustup
  note "installing stable verification and formatter toolchain $toolchain with rustfmt"
  if ! rustup toolchain install "$toolchain" --profile minimal --component rustfmt; then
    fail "failed to install stable toolchain $toolchain with rustfmt"
  fi
}

prepare_install_inputs() {
  BUILD_TOOLS_MOLD_PIN=$(mold_version) || return 1
  BUILD_TOOLS_STABLE_PIN=$(stable_toolchain_version) || return 1
  load_toolchain_configuration || return 1
  detect_host || return 1
  if is_linux && [[ -z $BUILD_TOOLS_PREFIX ]]; then
    fail 'BUILD_TOOLS_PREFIX must not be empty'
  fi
}

main() {
  prepare_install_inputs || return 1
  install_mold "$BUILD_TOOLS_MOLD_PIN"
  install_toolchain "$BUILD_TOOLS_CHANNEL"
  install_stable_toolchain "$BUILD_TOOLS_STABLE_PIN"
  note 'ready; verify with: bash scripts/check-build-tools.sh'
}

main
