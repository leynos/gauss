"""End-to-end tests for the integration-test inventory checker command."""

from __future__ import annotations

import shutil
import subprocess
import sys
from pathlib import Path

SCRIPT_DIRECTORY = Path(__file__).resolve().parents[1]


def write_end_to_end_repository(root: Path) -> Path:
    """Create a minimal root package and copied checker for CLI validation."""
    (root / "scripts").mkdir()
    (root / "tests").mkdir()
    (root / "docs/execplans").mkdir(parents=True)
    (root / "Cargo.toml").write_text(
        """[package]
name = "gauss"
version = "0.1.0"
edition = "2024"

[[test]]
name = "gpui_example"
path = "tests/gpui_example.rs"
""",
        encoding="utf-8",
    )
    (root / "tests/gpui_example.rs").write_text(
        """#[scenario(
    path = "example.feature",
    harness = rstest_bdd_harness_gpui::GpuiHarness
)]
fn example() {}
""",
        encoding="utf-8",
    )
    marker = (
        "<!-- integration-test-inventory: total=1 harness_gpui_bdd=1 "
        "raw_structural_gpui=0 non_gpui_bdd=0 other_integration=0 gpui_target=1 -->\n"
    )
    for document in (
        root / "docs/execplans/build-time-consolidate-integration-test-targets.md",
        root / "docs/execplans/adopt-rstest-bdd-v0-6-0-beta3.md",
        root / "docs/execplans/test-classification-inventory.md",
    ):
        document.write_text(marker, encoding="utf-8")
    (root / "tests/CONSOLIDATION_MAP.md").write_text(
        marker
        + """### Harness-backed GPUI BDD targets (1)

- `gpui_example`

### Raw structural GPUI targets (0)

### Non-GPUI BDD targets (0)

### Other integration targets (0)
""",
        encoding="utf-8",
    )
    checker = root / "scripts/check_integration_test_inventory.py"
    shutil.copy2(SCRIPT_DIRECTORY / "check_integration_test_inventory.py", checker)
    return checker


def test_checker_cli_reports_a_valid_inventory(tmp_path: Path) -> None:
    """Run the checker subprocess against a complete minimal repository."""
    checker = write_end_to_end_repository(tmp_path)

    result = subprocess.run(
        (sys.executable, str(checker)),
        cwd=tmp_path,
        capture_output=True,
        text=True,
        check=False,
    )

    assert result.returncode == 0
    assert "harness_gpui_bdd: 1 (gpui_example)" in result.stdout
    assert "total: 1" in result.stdout


def test_checker_cli_rejects_documentation_count_mismatch(tmp_path: Path) -> None:
    """Return a non-zero status when a documented count differs from metadata."""
    checker = write_end_to_end_repository(tmp_path)
    map_path = tmp_path / "tests/CONSOLIDATION_MAP.md"
    map_path.write_text(
        map_path.read_text(encoding="utf-8").replace("total=1", "total=2"),
        encoding="utf-8",
    )

    result = subprocess.run(
        (sys.executable, str(checker)),
        cwd=tmp_path,
        capture_output=True,
        text=True,
        check=False,
    )

    assert result.returncode == 1
    assert "integration-test inventory check failed" in result.stderr
    assert "documented {'total': 2" in result.stderr
