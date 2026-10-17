#!/bin/bash
# Check the pinned build tools before Cargo starts.

if (($# != 0)); then
  builtin printf 'build-tools: unexpected arguments\n' >&2
  exit 1
fi

script_path=${BASH_SOURCE[0]}
[[ $script_path == /* ]] || script_path=$PWD/$script_path
script_dir=$(cd -- "$(dirname -- "$script_path")" && pwd)
# shellcheck source=scripts/build-tools-common.sh
. "$script_dir/build-tools-common.sh"

BUILD_TOOLS_MOLD_PIN=
BUILD_TOOLS_STABLE_PIN=

check_supported_host_target() {
  local target=${BUILD_TOOLS_TARGET:-${CARGO_BUILD_TARGET:-}}
  if ! is_linux && [[ -n $target ]]; then
    case "-$target-" in
      *-linux-*)
        note "Linux target requires a Linux host: $target"
        return 1
        ;;
    esac
  fi
}

check_encoded_rustflags() {
  if [[ ${CARGO_ENCODED_RUSTFLAGS+x} ]]; then
    note 'CARGO_ENCODED_RUSTFLAGS is set and can override the development RUSTFLAGS passed by Make'
    note 'unset CARGO_ENCODED_RUSTFLAGS and supply development flags through RUSTFLAGS'
    return 1
  fi
}

check_mold() {
  local pinned=$1 resolved output first_line
  if ! is_linux; then
    note "mold is Linux-only; using the default $BUILD_TOOLS_HOST_OS linker"
    return 0
  fi
  if ! resolved=$(command -v mold 2>/dev/null); then
    note "mold not found on PATH (required pin: $pinned)"
    note 'install it with: bash scripts/install-build-tools.sh'
    return 1
  fi
  if ! output=$("$resolved" --version 2>/dev/null); then
    note "mold at $resolved is unusable or cannot report its version"
    note 'reinstall it with: bash scripts/install-build-tools.sh'
    return 1
  fi
  first_line=${output%%$'\n'*}
  if [[ $first_line != "mold $pinned" && $first_line != "mold $pinned "* ]]; then
    note "mold at $resolved does not match the pin $pinned: $first_line"
    note 'install the pinned release with: bash scripts/install-build-tools.sh'
    return 1
  fi
  note "mold $pinned at $resolved"
}

check_clang() {
  local resolved
  if ! is_linux; then
    return 0
  fi
  if ! resolved=$(command -v clang 2>/dev/null); then
    note 'clang not found on PATH; install clang with the system package manager'
    return 1
  fi
  if ! clang --version >/dev/null 2>&1; then
    note "clang at $resolved cannot report its version"
    return 1
  fi
  if ! clang -x c -fsyntax-only - <<'C_SOURCE'
int main(void) { return 0; }
C_SOURCE
  then
    note "clang at $resolved cannot compile a basic C translation unit"
    return 1
  fi
  note "clang is usable at $resolved"
}

check_toolchain() {
  local toolchain=$1 installed_toolchains installed_components installed found
  local component installed_name
  if ! command -v rustup >/dev/null 2>&1; then
    note 'rustup not found on PATH; install it from https://rustup.rs'
    return 1
  fi
  if ! installed_toolchains=$(rustup toolchain list 2>&1); then
    note "could not list installed Rust toolchains: $installed_toolchains"
    return 1
  fi
  found=0
  while IFS= read -r installed; do
    case $installed in
      "$toolchain"|"$toolchain"-*|"$toolchain "*) found=1 ;;
    esac
  done <<< "$installed_toolchains"
  if [[ $found != 1 ]]; then
    note "toolchain $toolchain is not installed"
    note 'install it with: bash scripts/install-build-tools.sh'
    return 1
  fi
  if ! installed_components=$(rustup component list --toolchain "$toolchain" --installed 2>&1); then
    note "could not list components for toolchain $toolchain: $installed_components"
    return 1
  fi
  for component in "${BUILD_TOOLS_COMPONENTS[@]}"; do
    installed_name=$(installed_component_name "$component")
    if ! toolchain_has_component "$installed_name" "$installed_components"; then
      note "toolchain $toolchain is missing required component $component"
      note 'install the pinned toolchain components with: bash scripts/install-build-tools.sh'
      return 1
    fi
  done
  note "toolchain $toolchain and its required components are available"
}

check_stable_toolchain() {
  local toolchain=$1
  local -a BUILD_TOOLS_COMPONENTS=(rustfmt)
  check_toolchain "$toolchain"
}

prepare_check_inputs() {
  BUILD_TOOLS_MOLD_PIN=$(mold_version) || return 1
  BUILD_TOOLS_STABLE_PIN=$(stable_toolchain_version) || return 1
  load_toolchain_configuration || return 1
  detect_host || return 1
  check_supported_host_target || return 1
  check_encoded_rustflags || return 1
}

main() {
  local status=0
  prepare_check_inputs || return 1
  check_mold "$BUILD_TOOLS_MOLD_PIN" || status=1
  check_clang || status=1
  check_toolchain "$BUILD_TOOLS_CHANNEL" || status=1
  check_stable_toolchain "$BUILD_TOOLS_STABLE_PIN" || status=1
  if [[ $status != 0 ]]; then
    note 'build-tool check failed; resolve the messages above before running Cargo'
  fi
  return "$status"
}

main
