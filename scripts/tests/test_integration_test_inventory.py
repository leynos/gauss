"""Tests for the root-Gauss integration-test inventory script."""

from __future__ import annotations

import importlib.util
import subprocess
from collections.abc import Callable, Iterable
from pathlib import Path
from typing import Protocol, cast

import pytest
from hypothesis import given, settings, strategies as st

SCRIPT_DIRECTORY = Path(__file__).resolve().parents[1]
CATEGORY_ORDER = (
    "harness_gpui_bdd",
    "raw_structural_gpui",
    "non_gpui_bdd",
    "other_integration",
)


class InventoryModule(Protocol):
    """Typed surface exercised from the dynamically imported checker module."""

    REPOSITORY_ROOT: Path

    def cargo_metadata(
        self,
        runner: Callable[..., subprocess.CompletedProcess[str]] = subprocess.run,
    ) -> dict[str, object]: ...

    def root_gauss_package(self, metadata: dict[str, object]) -> dict[str, object]: ...

    def root_gauss_targets(
        self, metadata: dict[str, object]
    ) -> Iterable[dict[str, object]]: ...

    def category_for(self, source: str) -> str: ...

    def inventory(
        self,
        metadata_reader: Callable[[], dict[str, object]],
        source_reader: Callable[[Path], str],
    ) -> dict[str, list[str]]: ...

    def counts(self, targets_by_category: dict[str, list[str]]) -> dict[str, int]: ...

    def documented_counts(self, source: str, document: Path) -> dict[str, int]: ...

    def documented_targets(
        self, source: str, document: Path
    ) -> dict[str, list[str]]: ...

    def validate_documentation(
        self,
        targets_by_category: dict[str, list[str]],
        documents: tuple[Path, ...],
        target_list_document: Path,
        document_reader: Callable[[Path], str],
    ) -> None: ...

    def main(self) -> None: ...


@pytest.fixture(name="inventory_module", scope="module")
def inventory_module_fixture() -> InventoryModule:
    module_path = SCRIPT_DIRECTORY / "check_integration_test_inventory.py"
    specification = importlib.util.spec_from_file_location(
        "check_integration_test_inventory", module_path
    )
    if specification is None or specification.loader is None:
        raise RuntimeError("could not load integration-test inventory checker")
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return cast(InventoryModule, module)


def root_package(module: InventoryModule, targets: object) -> dict[str, object]:
    root_manifest = str(module.REPOSITORY_ROOT / "Cargo.toml")
    return {"name": "gauss", "manifest_path": root_manifest, "targets": targets}


def target_inventory() -> dict[str, list[str]]:
    return dict(
        zip(CATEGORY_ORDER, (["gpui_bdd"], ["gpui_raw"], ["non_gpui_bdd"], ["other"]))
    )


def count_marker() -> str:
    return (
        "<!-- integration-test-inventory: total=4 "
        + " ".join(f"{category}=1" for category in CATEGORY_ORDER)
        + " gpui_target=2 -->"
    )


def consolidation_map() -> str:
    return "\n\n".join(
        f"### {heading} (1)\n\n- `{target}`"
        for heading, target in (
            ("Harness-backed GPUI BDD targets", "gpui_bdd"),
            ("Raw structural GPUI targets", "gpui_raw"),
            ("Non-GPUI BDD targets", "non_gpui_bdd"),
            ("Other integration targets", "other"),
        )
    )


def inventory_docs(tmp_path: Path) -> tuple[tuple[Path, ...], Path, dict[Path, str]]:
    documents = tuple(tmp_path / f"inventory-{index}.md" for index in range(4))
    target_list_document = tmp_path / "CONSOLIDATION_MAP.md"
    sources = {document: count_marker() for document in documents}
    sources[target_list_document] = consolidation_map()
    return documents, target_list_document, sources


def test_root_gauss_package_selects_the_root_manifest(
    inventory_module: InventoryModule,
) -> None:
    selected = root_package(inventory_module, [])
    metadata = {
        "packages": [
            {"name": "gauss", "manifest_path": "crates/gauss-core/Cargo.toml"},
            selected,
        ]
    }
    assert inventory_module.root_gauss_package(metadata) is selected, (
        "root package selection must use the repository manifest"
    )


def test_root_gauss_package_rejects_non_list_packages(
    inventory_module: InventoryModule,
) -> None:
    with pytest.raises(ValueError, match="cargo metadata packages must be a list"):
        inventory_module.root_gauss_package({"packages": {}})


def test_root_gauss_package_rejects_missing_root_package(
    inventory_module: InventoryModule,
) -> None:
    metadata = {"packages": [{"name": "other", "manifest_path": "Cargo.toml"}]}
    with pytest.raises(
        ValueError,
        match="could not find the root gauss package in cargo metadata",
    ):
        inventory_module.root_gauss_package(metadata)


def test_root_gauss_targets_rejects_non_list_targets(
    inventory_module: InventoryModule,
) -> None:
    metadata = {"packages": [root_package(inventory_module, {})]}
    with pytest.raises(ValueError, match="root gauss package targets must be a list"):
        list(inventory_module.root_gauss_targets(metadata))


def test_root_gauss_targets_filters_non_test_and_malformed_targets(
    inventory_module: InventoryModule,
) -> None:
    test_target = {"name": "integration", "kind": ["test"]}
    metadata = {
        "packages": [
            root_package(
                inventory_module,
                [test_target, {"name": "library", "kind": ["lib"]}, "malformed"],
            )
        ]
    }
    assert list(inventory_module.root_gauss_targets(metadata)) == [test_target], (
        "only dictionary test targets should remain"
    )


@pytest.mark.parametrize(
    ("source", "expected"),
    [
        (
            '#[scenario(path = "test.feature", harness = '
            "rstest_bdd_harness_gpui::GpuiHarness)]",
            "harness_gpui_bdd",
        ),
        ("#[gpui::test]\nfn raw() {}", "raw_structural_gpui"),
        ('#[scenario(path = "test.feature")]', "non_gpui_bdd"),
        ("#[test]\nfn ordinary() {}", "other_integration"),
    ],
)
def test_category_for_uses_registration_markers(
    inventory_module: InventoryModule,
    source: str,
    expected: str,
) -> None:
    assert inventory_module.category_for(source) == expected, (
        "registration markers must select the documented category"
    )


@settings(database=None)
@given(
    harness=st.booleans(),
    raw=st.booleans(),
    bdd=st.booleans(),
)
def test_category_for_uses_documented_precedence(
    inventory_module: InventoryModule,
    harness: bool,
    raw: bool,
    bdd: bool,
) -> None:
    source = "\n".join(
        marker
        for enabled, marker in (
            (
                harness,
                '#[scenario(path = "test.feature", harness = '
                "rstest_bdd_harness_gpui::GpuiHarness)]",
            ),
            (raw, "#[gpui::test]"),
            (bdd, '#[scenario(path = "test.feature")]'),
        )
        if enabled
    )
    expected = (
        "harness_gpui_bdd"
        if harness
        else "raw_structural_gpui"
        if raw
        else "non_gpui_bdd"
        if bdd
        else "other_integration"
    )
    assert inventory_module.category_for(source) == expected, (
        "harness, raw, and BDD markers must retain their precedence"
    )


def test_inventory_reads_metadata_and_sources_through_injected_seams(
    inventory_module: InventoryModule,
) -> None:
    metadata = {
        "packages": [
            root_package(
                inventory_module,
                [
                    {"name": "z_raw", "kind": ["test"], "src_path": "raw.rs"},
                    {"name": "a_bdd", "kind": ["test"], "src_path": "bdd.rs"},
                ],
            )
        ]
    }
    sources = {
        Path("raw.rs"): "#[gpui::test]",
        Path("bdd.rs"): '#[scenario(path = "test.feature", harness = '
        "rstest_bdd_harness_gpui::GpuiHarness)]",
    }
    assert inventory_module.inventory(lambda: metadata, sources.__getitem__) == {
        "harness_gpui_bdd": ["a_bdd"],
        "raw_structural_gpui": ["z_raw"],
        "non_gpui_bdd": [],
        "other_integration": [],
    }, "injected readers must produce a sorted classified inventory"


@settings(database=None)
@given(
    st.dictionaries(
        st.sampled_from(CATEGORY_ORDER),
        st.lists(st.text(alphabet="abc", max_size=5), max_size=6),
        min_size=len(CATEGORY_ORDER),
        max_size=len(CATEGORY_ORDER),
    )
)
def test_counts_conserve_all_category_targets(
    inventory_module: InventoryModule,
    targets_by_category: dict[str, list[str]],
) -> None:
    actual = inventory_module.counts(targets_by_category)
    assert actual["total"] == sum(
        len(targets_by_category[category]) for category in CATEGORY_ORDER
    ), "total must conserve targets across all categories"
    assert actual["gpui_target"] == (
        len(targets_by_category["harness_gpui_bdd"])
        + len(targets_by_category["raw_structural_gpui"])
    ), "GPUI total must include only harness-backed and raw targets"


def test_documented_counts_parses_a_complete_marker(
    inventory_module: InventoryModule,
    tmp_path: Path,
) -> None:
    document = tmp_path / "inventory.md"
    source = f"{count_marker()}\n"
    assert inventory_module.documented_counts(source, document) == {
        "total": 4,
        "harness_gpui_bdd": 1,
        "raw_structural_gpui": 1,
        "non_gpui_bdd": 1,
        "other_integration": 1,
        "gpui_target": 2,
    }, "a complete marker must preserve every derived count"


@pytest.mark.parametrize(
    ("marker", "message"),
    (
        ("<!-- integration-test-inventory: total=4 -->", "fields do not match"),
        (count_marker().replace("total=4", "total=4 total=4"), "duplicate fields"),
    ),
)
def test_documented_counts_rejects_invalid_fields(
    inventory_module: InventoryModule,
    tmp_path: Path,
    marker: str,
    message: str,
) -> None:
    document = tmp_path / "inventory.md"
    source = f"{marker}\n"
    with pytest.raises(ValueError, match=message):
        inventory_module.documented_counts(source, document)


def test_validate_documentation_rejects_a_target_list_mismatch(
    inventory_module: InventoryModule,
    tmp_path: Path,
) -> None:
    documents, target_document, sources = inventory_docs(tmp_path)
    sources[target_document] = sources[target_document].replace(
        "gpui_raw", "different_raw"
    )
    with pytest.raises(ValueError, match=r"actual \['gpui_raw'\]"):
        inventory_module.validate_documentation(
            target_inventory(),
            documents,
            target_document,
            sources.__getitem__,
        )


def test_documented_targets_delimit_first_and_final_category_sections(
    inventory_module: InventoryModule,
    tmp_path: Path,
) -> None:
    document = tmp_path / "CONSOLIDATION_MAP.md"
    targets = inventory_module.documented_targets(consolidation_map(), document)
    assert targets["harness_gpui_bdd"] == ["gpui_bdd"]
    assert targets["other_integration"] == ["other"]


@pytest.mark.parametrize(
    ("contents", "message"),
    (
        (consolidation_map().replace("targets (1)", "targets (2)", 1), "heading has 2"),
        (
            f"{consolidation_map()}\n### Harness-backed GPUI BDD targets (0)\n",
            "exactly one target list",
        ),
        (
            consolidation_map().replace(
                "### Other integration targets (1)", "### Missing (1)"
            ),
            "missing target list for other_integration",
        ),
    ),
)
def test_documented_targets_rejects_invalid_headings(
    inventory_module: InventoryModule,
    tmp_path: Path,
    contents: str,
    message: str,
) -> None:
    document = tmp_path / "CONSOLIDATION_MAP.md"
    with pytest.raises(ValueError, match=message):
        inventory_module.documented_targets(contents, document)


def test_main_prints_the_validated_inventory(
    inventory_module: InventoryModule,
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    monkeypatch.setattr(inventory_module, "inventory", target_inventory)
    monkeypatch.setattr(inventory_module, "validate_documentation", lambda _: None)
    inventory_module.main()
    assert capsys.readouterr().out == (
        "harness_gpui_bdd: 1 (gpui_bdd)\n"
        "raw_structural_gpui: 1 (gpui_raw)\n"
        "non_gpui_bdd: 1 (non_gpui_bdd)\n"
        "other_integration: 1 (other)\n"
        "gpui_target: 2\n"
        "total: 4\n"
    ), "main must print each validated category and derived total"
