#!/usr/bin/env -S uv run python
# /// script
# requires-python = ">=3.13"
# dependencies = []
# ///
"""Verify documented root-Gauss integration-test inventory from Cargo metadata.

Source markers classify registered harnesses as GPUI BDD or raw structural coverage.
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
from collections import defaultdict
from collections.abc import Callable, Iterable
from pathlib import Path

REPOSITORY_ROOT = Path(__file__).resolve().parent.parent
CONSOLIDATION_MAP = REPOSITORY_ROOT / "tests/CONSOLIDATION_MAP.md"
INVENTORY_DOCUMENTS = (
    REPOSITORY_ROOT
    / "docs/execplans/build-time-consolidate-integration-test-targets.md",
    REPOSITORY_ROOT / "docs/execplans/adopt-rstest-bdd-v0-6-0-beta3.md",
    REPOSITORY_ROOT / "docs/execplans/test-classification-inventory.md",
    CONSOLIDATION_MAP,
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
TARGET_LIST_HEADINGS = {
    "harness_gpui_bdd": "Harness-backed GPUI BDD targets",
    "raw_structural_gpui": "Raw structural GPUI targets",
    "non_gpui_bdd": "Non-GPUI BDD targets",
    "other_integration": "Other integration targets",
}


def _read_target_source(source_path: Path) -> str:
    try:
        return source_path.read_text(encoding="utf-8")
    except OSError as error:
        raise ValueError(
            f"could not read integration-test source {source_path}"
        ) from error


def cargo_metadata(
    runner: Callable[..., subprocess.CompletedProcess[str]] = subprocess.run,
) -> dict[str, object]:
    """Read Cargo metadata for the repository's current workspace.

    Parameters
    ----------
    runner : Callable[..., subprocess.CompletedProcess[str]], optional
        Process runner. Defaults to :func:`subprocess.run` with the metadata command.

    Returns
    -------
    dict[str, object]
        Decoded JSON object emitted by Cargo metadata.

    Raises
    ------
    ValueError
        If Cargo metadata cannot be read or is malformed.
    """
    try:
        result = runner(
            ("cargo", "metadata", "--no-deps", "--format-version", "1"),
            cwd=REPOSITORY_ROOT,
            check=True,
            capture_output=True,
            text=True,
        )
    except (OSError, subprocess.CalledProcessError) as error:
        raise ValueError("could not read Cargo metadata") from error
    try:
        metadata = json.loads(result.stdout)
    except json.JSONDecodeError as error:
        raise ValueError("Cargo metadata is not valid JSON") from error
    if not isinstance(metadata, dict):
        raise ValueError("Cargo metadata must be a JSON object")
    return metadata


def root_gauss_package(metadata: dict[str, object]) -> dict[str, object]:
    """Select the root ``gauss`` package from Cargo metadata.

    Parameters
    ----------
    metadata : dict[str, object]
        Decoded Cargo metadata for the current workspace.

    Returns
    -------
    dict[str, object]
        The root-manifest package named ``gauss``.

    Raises
    ------
    ValueError
        If packages are malformed or omit the root ``gauss`` package.
    """
    root_manifest = str(REPOSITORY_ROOT / "Cargo.toml")
    packages = metadata["packages"]
    if not isinstance(packages, list):
        raise ValueError("cargo metadata packages must be a list")
    package = next(
        (
            package
            for package in packages
            if isinstance(package, dict)
            and package.get("name") == "gauss"
            and package.get("manifest_path") == root_manifest
        ),
        None,
    )
    if package is None:
        raise ValueError("could not find the root gauss package in cargo metadata")
    return package


def root_gauss_targets(metadata: dict[str, object]) -> Iterable[dict[str, object]]:
    """Yield integration-test targets declared by the root ``gauss`` package.

    Parameters
    ----------
    metadata : dict[str, object]
        Decoded Cargo metadata for the current workspace.

    Returns
    -------
    Iterable[dict[str, object]]
        Dictionary targets whose kind includes ``test``.

    Raises
    ------
    ValueError
        If the selected root package has a malformed targets field.
    """
    targets = root_gauss_package(metadata).get("targets")
    if not isinstance(targets, list):
        raise ValueError("root gauss package targets must be a list")
    return (
        target
        for target in targets
        if isinstance(target, dict) and "test" in target.get("kind", [])
    )


def category_for(source: str) -> str:
    """Classify one integration-test target using explicit source markers.

    Parameters
    ----------
    source : str
        UTF-8 source text for one Cargo integration-test target.

    Returns
    -------
    str
        One mutually exclusive category in :data:`CATEGORY_ORDER`.
    """
    if HARNESS_PATTERN.search(source):
        return "harness_gpui_bdd"
    if RAW_GPUI_PATTERN.search(source):
        return "raw_structural_gpui"
    if BDD_PATTERN.search(source):
        return "non_gpui_bdd"
    return "other_integration"


def inventory(
    metadata_reader: Callable[[], dict[str, object]] = cargo_metadata,
    source_reader: Callable[[Path], str] = _read_target_source,
) -> dict[str, list[str]]:
    """Group current root targets by their mutually exclusive category.

    Parameters
    ----------
    metadata_reader : Callable[[], dict[str, object]], optional
        Reader for the authoritative Cargo metadata object.
    source_reader : Callable[[Path], str], optional
        Reader for each target source path.

    Returns
    -------
    dict[str, list[str]]
        Sorted target names for every category in :data:`CATEGORY_ORDER`.

    Raises
    ------
    ValueError
        If target metadata or a reader cannot provide required input.
    """
    targets_by_category: dict[str, list[str]] = defaultdict(list)
    for target in root_gauss_targets(metadata_reader()):
        source_path = target.get("src_path")
        target_name = target.get("name")
        if not isinstance(source_path, str) or not isinstance(target_name, str):
            raise ValueError(
                "integration-test target requires string name and source path"
            )
        targets_by_category[category_for(source_reader(Path(source_path)))].append(
            target_name
        )
    return {
        category: sorted(targets_by_category[category]) for category in CATEGORY_ORDER
    }


def counts(targets_by_category: dict[str, list[str]]) -> dict[str, int]:
    """Calculate documented totals from classified integration targets.

    Parameters
    ----------
    targets_by_category : dict[str, list[str]]
        Target names grouped under every category in :data:`CATEGORY_ORDER`.

    Returns
    -------
    dict[str, int]
        Per-category counts plus ``gpui_target`` and ``total`` derived totals.
    """
    result = {
        category: len(targets_by_category[category]) for category in CATEGORY_ORDER
    }
    result["gpui_target"] = result["harness_gpui_bdd"] + result["raw_structural_gpui"]
    result["total"] = sum(result[category] for category in CATEGORY_ORDER)
    return result


def documented_counts(document: Path) -> dict[str, int]:
    """Read the single machine-checkable current-inventory marker.

    Parameters
    ----------
    document : Path
        Inventory document containing exactly one current-inventory marker.

    Returns
    -------
    dict[str, int]
        Parsed category and derived-total counts from the marker.

    Raises
    ------
    ValueError
        If the document has the wrong marker count or marker fields.
    """
    matches = MARKER_PATTERN.findall(document.read_text(encoding="utf-8"))
    if len(matches) != 1:
        raise ValueError(f"{document}: expected exactly one inventory marker")
    fields = list(FIELD_PATTERN.finditer(matches[0]))
    if len({field["name"] for field in fields}) != len(fields):
        raise ValueError(f"{document}: inventory marker has duplicate fields")
    values = {field["name"]: int(field["value"]) for field in fields}
    required = {*CATEGORY_ORDER, "gpui_target", "total"}
    if values.keys() != required:
        raise ValueError(
            f"{document}: inventory marker fields do not match {sorted(required)}"
        )
    return values


def _target_list_section(
    source: str,
    document: Path,
    category: str,
    heading: str,
) -> str:
    heading_pattern = rf"^### {re.escape(heading)} \((?P<count>\d+)\)$"
    heading_matches = list(re.finditer(heading_pattern, source, re.MULTILINE))
    if not heading_matches:
        raise ValueError(f"{document}: missing target list for {category}")
    if len(heading_matches) != 1:
        raise ValueError(f"{document}: expected exactly one target list for {category}")
    heading_match = heading_matches[0]
    next_heading = re.search(r"^### ", source[heading_match.end() :], re.MULTILINE)
    section_end = (
        heading_match.end() + next_heading.start()
        if next_heading is not None
        else len(source)
    )
    section = source[heading_match.end() : section_end]
    target_count = len(re.findall(r"^- `(?P<target>[^`]+)`$", section, re.MULTILINE))
    if int(heading_match["count"]) != target_count:
        raise ValueError(
            f"{document}: {category} heading has {heading_match['count']} targets, "
            f"but lists {target_count}"
        )
    return section


def documented_targets(document: Path) -> dict[str, list[str]]:
    """Read the category target lists documented in the consolidation map.

    Parameters
    ----------
    document : Path
        Consolidation map containing one headed target list per category.

    Returns
    -------
    dict[str, list[str]]
        Sorted target names for every category in :data:`CATEGORY_ORDER`.

    Raises
    ------
    ValueError
        If a category heading is missing or its count is incorrect.
    """
    source = document.read_text(encoding="utf-8")
    targets_by_category: dict[str, list[str]] = {}
    for category, heading in TARGET_LIST_HEADINGS.items():
        section = _target_list_section(source, document, category, heading)
        targets_by_category[category] = sorted(
            re.findall(r"^- `(?P<target>[^`]+)`$", section, re.MULTILINE)
        )
    return targets_by_category


def validate_documentation(
    targets_by_category: dict[str, list[str]],
    documents: tuple[Path, ...] = INVENTORY_DOCUMENTS,
    target_list_document: Path = CONSOLIDATION_MAP,
) -> None:
    """Validate documented counts and exact target lists against Cargo metadata.

    Parameters
    ----------
    targets_by_category : dict[str, list[str]]
        Sorted target inventory derived from the root Cargo package.
    documents : tuple[Path, ...], optional
        Documents that must carry a current-inventory count marker.
    target_list_document : Path, optional
        Consolidation map whose category lists must match the inventory.

    Raises
    ------
    ValueError
        If a documented marker or target list differs from the inventory.
    """
    actual = counts(targets_by_category)
    failures = [
        f"{document}: documented {documented_counts(document)}, actual {actual}"
        for document in documents
        if documented_counts(document) != actual
    ]
    documented = documented_targets(target_list_document)
    failures.extend(
        f"{target_list_document}: documented {documented[category]}, "
        f"actual {targets_by_category[category]} for {category}"
        for category in CATEGORY_ORDER
        if documented[category] != targets_by_category[category]
    )
    if failures:
        raise ValueError("\n".join(failures))


def main() -> None:
    """Print and validate the current integration-test inventory.

    Raises
    ------
    ValueError
        If source metadata or documented inventory data does not match.
    """
    targets_by_category = inventory()
    actual = counts(targets_by_category)
    validate_documentation(targets_by_category)
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
