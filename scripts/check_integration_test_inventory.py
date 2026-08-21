#!/usr/bin/env -S uv run python
# /// script
# requires-python = ">=3.13"
# dependencies = []
# ///
"""Verify the documented root-Gauss integration-test inventory.

Cargo metadata identifies integration-test targets independently of the test
directory layout. Source markers then classify each target by its registered
test harness, preserving a reproducible distinction between GPUI BDD and raw
structural coverage.
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
from collections import defaultdict
from collections.abc import Iterable
from pathlib import Path

REPOSITORY_ROOT = Path(__file__).resolve().parent.parent
INVENTORY_DOCUMENTS = (
    REPOSITORY_ROOT
    / "docs/execplans/build-time-consolidate-integration-test-targets.md",
    REPOSITORY_ROOT / "docs/execplans/adopt-rstest-bdd-v0-6-0-beta3.md",
    REPOSITORY_ROOT / "docs/execplans/test-classification-inventory.md",
    REPOSITORY_ROOT / "tests/CONSOLIDATION_MAP.md",
)
MARKER_PATTERN = re.compile(r"<!-- integration-test-inventory: (?P<fields>[^>]+) -->")
FIELD_PATTERN = re.compile(r"(?P<name>[a-z_]+)=(?P<value>\d+)")
HARNESS_PATTERN = re.compile(
    r"#\[scenario\([\s\S]*?harness\s*=\s*"
    r"rstest_bdd_harness_gpui::GpuiHarness",
    re.MULTILINE,
)
RAW_GPUI_PATTERN = re.compile(r"^\s*#\[gpui::test\]", re.MULTILINE)
BDD_PATTERN = re.compile(r"^\s*#\[scenario\(", re.MULTILINE)
CATEGORY_ORDER = (
    "harness_gpui_bdd",
    "raw_structural_gpui",
    "non_gpui_bdd",
    "other_integration",
)


def cargo_metadata() -> dict[str, object]:
    """Return Cargo metadata for the repository's current workspace."""
    result = subprocess.run(
        ("cargo", "metadata", "--no-deps", "--format-version", "1"),
        cwd=REPOSITORY_ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return json.loads(result.stdout)


def root_gauss_targets(metadata: dict[str, object]) -> Iterable[dict[str, object]]:
    """Yield integration-test targets declared by the root ``gauss`` package."""
    root_manifest = str(REPOSITORY_ROOT / "Cargo.toml")
    packages = metadata["packages"]
    if not isinstance(packages, list):
        raise ValueError("cargo metadata packages must be a list")

    for package in packages:
        if not isinstance(package, dict):
            continue
        if (
            package.get("name") != "gauss"
            or package.get("manifest_path") != root_manifest
        ):
            continue
        targets = package.get("targets")
        if not isinstance(targets, list):
            raise ValueError("root gauss package targets must be a list")
        for target in targets:
            if isinstance(target, dict) and "test" in target.get("kind", []):
                yield target
        return

    raise ValueError("could not find the root gauss package in cargo metadata")


def category_for(source: str) -> str:
    """Classify one integration-test target using its explicit source markers."""
    if HARNESS_PATTERN.search(source):
        return "harness_gpui_bdd"
    if RAW_GPUI_PATTERN.search(source):
        return "raw_structural_gpui"
    if BDD_PATTERN.search(source):
        return "non_gpui_bdd"
    return "other_integration"


def inventory() -> dict[str, list[str]]:
    """Return current target names grouped by their mutually exclusive category."""
    targets_by_category: dict[str, list[str]] = defaultdict(list)
    for target in root_gauss_targets(cargo_metadata()):
        source_path = target.get("src_path")
        target_name = target.get("name")
        if not isinstance(source_path, str) or not isinstance(target_name, str):
            raise ValueError(
                "integration-test target requires string name and source path"
            )
        targets_by_category[
            category_for(Path(source_path).read_text(encoding="utf-8"))
        ].append(target_name)
    return {
        category: sorted(targets_by_category[category]) for category in CATEGORY_ORDER
    }


def counts(targets_by_category: dict[str, list[str]]) -> dict[str, int]:
    """Calculate documented totals from the classified integration targets."""
    result = {
        category: len(targets_by_category[category]) for category in CATEGORY_ORDER
    }
    result["gpui_target"] = result["harness_gpui_bdd"] + result["raw_structural_gpui"]
    result["total"] = sum(result[category] for category in CATEGORY_ORDER)
    return result


def documented_counts(document: Path) -> dict[str, int]:
    """Read the single machine-checkable current-inventory marker from a document."""
    matches = MARKER_PATTERN.findall(document.read_text(encoding="utf-8"))
    if len(matches) != 1:
        raise ValueError(f"{document}: expected exactly one inventory marker")
    values = {
        field["name"]: int(field["value"])
        for field in FIELD_PATTERN.finditer(matches[0])
    }
    required = {*CATEGORY_ORDER, "gpui_target", "total"}
    if values.keys() != required:
        raise ValueError(
            f"{document}: inventory marker fields do not match {sorted(required)}"
        )
    return values


def validate_documentation(actual: dict[str, int]) -> None:
    """Fail when any current-inventory document differs from Cargo metadata."""
    failures = [
        f"{document}: documented {documented_counts(document)}, actual {actual}"
        for document in INVENTORY_DOCUMENTS
        if documented_counts(document) != actual
    ]
    if failures:
        raise ValueError("\n".join(failures))


def main() -> None:
    """Print the target inventory and validate every documented current count."""
    targets_by_category = inventory()
    actual = counts(targets_by_category)
    validate_documentation(actual)
    for category in CATEGORY_ORDER:
        print(
            f"{category}: {actual[category]} ({', '.join(targets_by_category[category])})"
        )
    print(f"gpui_target: {actual['gpui_target']}")
    print(f"total: {actual['total']}")


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"integration-test inventory check failed: {error}", file=sys.stderr)
        raise SystemExit(1) from error
