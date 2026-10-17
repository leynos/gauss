#!/bin/bash
# Shared pins and checks for the repository's development build tools.
#
# Entry points source this file so direct calls share pin parsing and host
# decisions without provisioning tools as a side effect of loading helpers.

set -euo pipefail

BUILD_TOOLS_HELPER_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
BUILD_TOOLS_REPO_ROOT=$(cd -- "$BUILD_TOOLS_HELPER_DIR/.." && pwd)

MOLD_VERSION_FILE=${MOLD_VERSION_FILE-$BUILD_TOOLS_REPO_ROOT/tools/mold/VERSION}
MOLD_SHA256SUMS_FILE=${MOLD_SHA256SUMS_FILE-$BUILD_TOOLS_REPO_ROOT/tools/mold/SHA256SUMS}
RUST_TOOLCHAIN_FILE=${RUST_TOOLCHAIN_FILE-$BUILD_TOOLS_REPO_ROOT/rust-toolchain.toml}
RUST_STABLE_VERSION_FILE=$BUILD_TOOLS_REPO_ROOT/tools/rust-stable/VERSION
if [[ ${BUILD_TOOLS_PREFIX+x} ]]; then
  : # Preserve an explicit value so installation can reject an empty prefix.
elif [[ -n ${HOME:-} ]]; then
  BUILD_TOOLS_PREFIX=$HOME/.local
else
  BUILD_TOOLS_PREFIX=
fi

BUILD_TOOLS_CHANNEL=
BUILD_TOOLS_COMPONENTS=()
BUILD_TOOLS_HOST_OS=
BUILD_TOOLS_HOST_MACHINE=
BUILD_TOOLS_MOLD_ARCH=

note() { builtin printf 'build-tools: %s\n' "$*" >&2; }

fail() {
  note "$*"
  exit 1
}

# Keep the existing one-line, trimming, and embedded-whitespace contract.
read_pin() {
  local file=$1 value lines
  [[ -f $file ]] || fail "missing version pin: $file"
  lines=$(awk 'END { print NR }' "$file")
  [[ $lines == 1 ]] || fail "expected one line in version pin: $file"
  value=$(cat -- "$file")
  value=${value#"${value%%[![:space:]]*}"}
  value=${value%"${value##*[![:space:]]}"}
  [[ -n $value ]] || fail "empty version pin: $file"
  case $value in
    *[[:space:]]*) fail "version pin contains whitespace: $file" ;;
  esac
  builtin printf '%s' "$value"
}

read_numeric_pin() {
  local file=$1 value
  value=$(read_pin "$file")
  [[ $value =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] ||
    fail "invalid version pin: $file"
  builtin printf '%s' "$value"
}

mold_version() { read_numeric_pin "$MOLD_VERSION_FILE"; }

stable_toolchain_version() { read_numeric_pin "$RUST_STABLE_VERSION_FILE"; }

channel_is_valid() {
  local channel=$1 month day
  [[ $channel =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] && return 0
  [[ $channel =~ ^nightly-([0-9]{4})-([0-9]{2})-([0-9]{2})$ ]] || return 1
  month=$((10#${BASH_REMATCH[2]}))
  day=$((10#${BASH_REMATCH[3]}))
  (( month >= 1 && month <= 12 && day >= 1 && day <= 31 ))
}

# This is a supported TOML subset reader, not a full TOML validator.
# Headers accept bare/dotted keys only; unsupported forms fail closed.
BUILD_TOOLS_TOML_AWK='
  function trim(value) {
    sub(/^[[:space:]]+/, "", value)
    sub(/[[:space:]]+$/, "", value)
    return value
  }
  function strip_comment(value,    i, char, quoted, escaped, result) {
    quoted = 0
    escaped = 0
    result = ""
    for (i = 1; i <= length(value); i++) {
      char = substr(value, i, 1)
      if (char == "#" && !quoted) break
      result = result char
      if (quoted) {
        if (escaped) escaped = 0
        else if (char == "\\") escaped = 1
        else if (char == "\"") quoted = 0
      } else if (char == "\"") {
        quoted = 1
      }
    }
    return result
  }
  function find_close(value,    i, char, quoted, escaped) {
    quoted = 0
    escaped = 0
    for (i = 1; i <= length(value); i++) {
      char = substr(value, i, 1)
      if (!quoted && char == "]") return i
      if (quoted) {
        if (escaped) escaped = 0
        else if (char == "\\") escaped = 1
        else if (char == "\"") quoted = 0
      } else if (char == "\"") {
        quoted = 1
      }
    }
    return 0
  }
  # Reject unsupported key forms so malformed headers cannot hide the table.
  function valid_table_key(key) {
    key = trim(key)
    return key ~ /^[A-Za-z0-9_-]+([[:space:]]*\.[[:space:]]*[A-Za-z0-9_-]+)*$/
  }
  # Return 0 for invalid, 1 for [toolchain], and 2 for a valid other table.
  function table_header_kind(value,    n, key, is_array_table) {
    value = trim(value)
    n = length(value)
    is_array_table = 0
    if (substr(value, 1, 2) == "[[") {
      if (n < 5 || substr(value, n - 1, 2) != "]]") return 0
      is_array_table = 1
      key = substr(value, 3, n - 4)
    } else if (substr(value, 1, 1) == "[") {
      if (n < 3 || substr(value, n, 1) != "]") return 0
      key = substr(value, 2, n - 2)
    } else {
      return 0
    }
    key = trim(key)
    if (!valid_table_key(key)) return 0
    if (key == "toolchain") {
      if (is_array_table) return 0
      return 1
    }
    return 2
  }
  function append_component(token) {
    if (component_values != "") component_values = component_values " "
    component_values = component_values token
  }
  # Consume a whole token and separator; never discard an unparsed suffix.
  function consume_component(value,    token, tail) {
    value = trim(value)
    if (value !~ /^\"[A-Za-z0-9_-]+\"/) return 0
    match(value, /^\"[A-Za-z0-9_-]+\"/)
    token = substr(value, 2, RLENGTH - 2)
    tail = trim(substr(value, RLENGTH + 1))
    if (tail != "") {
      if (substr(tail, 1, 1) != ",") return 0
      tail = trim(substr(tail, 2))
    }
    append_component(token)
    component_tail = tail
    return 1
  }
  function parse_array(value,    n, body) {
    value = trim(value)
    n = length(value)
    if (n < 2 || substr(value, 1, 1) != "[" ||
        substr(value, n, 1) != "]") return 0
    body = trim(substr(value, 2, n - 2))
    if (body == "") return 1
    component_tail = body
    while (component_tail != "") {
      if (!consume_component(component_tail)) return 0
    }
    return 1
  }
  function complete_array(    closing_at, tail, array_text) {
    closing_at = find_close(components_text)
    if (!closing_at) return 0
    array_text = substr(components_text, 1, closing_at)
    tail = trim(substr(components_text, closing_at + 1))
    if (tail != "" || !parse_array(array_text)) bad_components = 1
    components_open = 0
    return 1
  }
  BEGIN { component_values = "" }
  {
    line = trim(strip_comment($0))
    if (components_open) {
      components_text = components_text " " line
      complete_array()
      next
    }
    if (line == "") next
    if (substr(line, 1, 1) == "[") {
      header_kind = table_header_kind(line)
      if (header_kind == 0) {
        bad_channel = 1
        in_toolchain = 0
      } else if (header_kind == 1) {
        table_count++
        if (table_count > 1) bad_channel = 1
        in_toolchain = 1
      } else {
        in_toolchain = 0
      }
      next
    }
    if (!in_toolchain) next
    if (line ~ /^channel([[:space:]]|=|$)/) {
      channel_count++
      if (line !~ /^channel[[:space:]]*=/) {
        bad_channel = 1
        next
      }
      value = trim(substr(line, index(line, "=") + 1))
      if (value !~ /^"[A-Za-z0-9.-]+"$/) bad_channel = 1
      else channel = substr(value, 2, length(value) - 2)
      next
    }
    if (line ~ /^components([[:space:]]|=|$)/) {
      components_count++
      if (components_count > 1 || line !~ /^components[[:space:]]*=/) {
        bad_components = 1
        next
      }
      components_text = trim(substr(line, index(line, "=") + 1))
      components_open = 1
      complete_array()
    }
  }
  END {
    channel_bad = (table_count != 1 || channel_count != 1 || bad_channel)
    components_bad = (components_count > 1 || components_open || bad_components)
    printf "%d|%d|%s|%s\n", channel_bad, components_bad, channel, component_values
  }
'

load_toolchain_configuration() {
  local file=$RUST_TOOLCHAIN_FILE parsed listed channel_bad components_bad component
  local -a declared_components=()
  [[ -f $file ]] || fail "missing toolchain file: $file"
  if ! parsed=$(awk "$BUILD_TOOLS_TOML_AWK" "$file"); then
    fail "could not read components from: $file"
  fi
  IFS='|' read -r channel_bad components_bad BUILD_TOOLS_CHANNEL listed <<< "$parsed"
  [[ $channel_bad == 0 ]] || fail "expected one valid channel in: $file"
  [[ $components_bad == 0 ]] || fail "could not read components from: $file"
  channel_is_valid "$BUILD_TOOLS_CHANNEL" ||
    fail "expected one valid channel in: $file"
  BUILD_TOOLS_COMPONENTS=()
  append_toolchain_component rustfmt
  append_toolchain_component clippy
  append_toolchain_component rust-analyzer
  if [[ -n $listed ]]; then
    read -r -a declared_components <<< "$listed"
    for component in "${declared_components[@]}"; do
      append_toolchain_component "$component"
    done
  fi
}

append_toolchain_component() {
  local candidate=$1 existing
  for existing in "${BUILD_TOOLS_COMPONENTS[@]}"; do
    [[ $existing == "$candidate" ]] && return 0
  done
  BUILD_TOOLS_COMPONENTS+=("$candidate")
}

detect_host() {
  local system machine
  if ! system=$(uname -s 2>/dev/null) || [[ -z $system ]]; then
    fail 'could not read host operating system'
  fi
  case $system in
    *[[:space:]]*) fail 'could not read host operating system' ;;
  esac
  BUILD_TOOLS_HOST_OS=$system
  if [[ ${BUILD_TOOLS_MOLD_REQUIRED+x} ]]; then
    case $BUILD_TOOLS_MOLD_REQUIRED in
      0|1) ;;
      *) fail 'BUILD_TOOLS_MOLD_REQUIRED must be 0 or 1' ;;
    esac
  fi
  [[ $system == Linux ]] || return 0
  if ! machine=$(uname -m 2>/dev/null) || [[ -z $machine ]]; then
    fail 'could not read host architecture'
  fi
  BUILD_TOOLS_HOST_MACHINE=$machine
  case $machine in
    x86_64|amd64) BUILD_TOOLS_MOLD_ARCH=x86_64 ;;
    aarch64|arm64) BUILD_TOOLS_MOLD_ARCH=aarch64 ;;
    armv7l|armv8l) BUILD_TOOLS_MOLD_ARCH=arm ;;
    riscv64|ppc64le|s390x|loongarch64) BUILD_TOOLS_MOLD_ARCH=$machine ;;
    *) fail "unsupported architecture for the pinned mold release: $machine" ;;
  esac
  if [[ ${BUILD_TOOLS_MOLD_REQUIRED:-1} == 0 ]]; then
    fail 'BUILD_TOOLS_MOLD_REQUIRED=0 cannot disable mold on Linux'
  fi
}

is_linux() { [[ $BUILD_TOOLS_HOST_OS == Linux ]]; }

installed_component_name() {
  case $1 in
    llvm-tools-preview) builtin printf 'llvm-tools' ;;
    *) builtin printf '%s' "$1" ;;
  esac
}

toolchain_has_component() {
  local installed_name=$1 installed_components=$2 installed
  while IFS= read -r installed; do
    case $installed in
      "$installed_name"|"$installed_name"-*|"$installed_name (installed)"|"$installed_name"-*" (installed)")
        return 0
        ;;
    esac
  done <<< "$installed_components"
  return 1
}
