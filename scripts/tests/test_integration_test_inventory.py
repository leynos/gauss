"""Tests for the root-Gauss integration-test inventory script."""

from __future__ import annotations

import importlib
from pathlib import Path

import pytest

SCRIPT_DIRECTORY = Path(__file__).resolve().parents[1]


@pytest.fixture(name="inventory_module")
def inventory_module_fixture(monkeypatch: pytest.MonkeyPatch) -> object:
    """Import the inventory script through its command-line module path."""
    monkeypatch.syspath_prepend(str(SCRIPT_DIRECTORY))
    return importlib.import_module("check_integration_test_inventory")


def root_package(module: object, targets: object) -> dict[str, object]:
    """Build metadata for the root package selected by the inventory script."""
    root_manifest = str(module.REPOSITORY_ROOT / "Cargo.toml")
    return {"name": "gauss", "manifest_path": root_manifest, "targets": targets}


def test_root_gauss_package_selects_the_root_manifest(inventory_module: object) -> None:
    """Select only the package that matches the root ``gauss`` manifest."""
    selected = root_package(inventory_module, [])
    metadata = {
        "packages": [
            {"name": "gauss", "manifest_path": "crates/gauss-core/Cargo.toml"},
            selected,
        ]
    }

    assert inventory_module.root_gauss_package(metadata) is selected


def test_root_gauss_package_rejects_non_list_packages(inventory_module: object) -> None:
    """Reject malformed Cargo metadata before package selection."""
    with pytest.raises(ValueError, match="cargo metadata packages must be a list"):
        inventory_module.root_gauss_package({"packages": {}})


def test_root_gauss_package_rejects_missing_root_package(
    inventory_module: object,
) -> None:
    """Reject metadata that does not describe the root package."""
    metadata = {"packages": [{"name": "other", "manifest_path": "Cargo.toml"}]}

    with pytest.raises(
        ValueError,
        match="could not find the root gauss package in cargo metadata",
    ):
        inventory_module.root_gauss_package(metadata)


def test_root_gauss_targets_rejects_non_list_targets(inventory_module: object) -> None:
    """Reject a root package whose targets field is malformed."""
    metadata = {"packages": [root_package(inventory_module, {})]}

    with pytest.raises(ValueError, match="root gauss package targets must be a list"):
        list(inventory_module.root_gauss_targets(metadata))


def test_root_gauss_targets_filters_non_test_and_malformed_targets(
    inventory_module: object,
) -> None:
    """Yield only dictionary targets whose kind contains ``test``."""
    test_target = {"name": "integration", "kind": ["test"]}
    metadata = {
        "packages": [
            root_package(
                inventory_module,
                [test_target, {"name": "library", "kind": ["lib"]}, "malformed"],
            )
        ]
    }

    assert list(inventory_module.root_gauss_targets(metadata)) == [test_target]
