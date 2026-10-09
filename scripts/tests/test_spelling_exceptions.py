"""Regression tests for the repository's spelling exceptions.

`typos.local.toml` must exempt `mold`, `Mold`, `LOD` and `inventario` only
through exact patterns, never as bare accepted words. These tests hold that
contract: every current occurrence is covered by a pattern, and an unrelated
use of any of the four terms is not.
"""

from __future__ import annotations

import re
import shutil
import subprocess
import tomllib
from pathlib import Path

import pytest

REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
LOCAL_OVERLAY = REPOSITORY_ROOT / "typos.local.toml"
TERMS = ("mold", "Mold", "LOD", "inventario")
#: Terms as prose words. A term joined to a hyphenated identifier, such as the
#: `setup-mold` action reference, is a name rather than prose.
TERM_PATTERN = re.compile(r"(?<![-\w])(?:mold|Mold|LOD|inventario)\b")
#: Files that legitimately name the terms: the configuration itself and this
#: module, whose own source lists them.
EXCLUDED_FILES = {
    "typos.toml",
    "typos.local.toml",
    "scripts/tests/test_spelling_exceptions.py",
}


@pytest.fixture(scope="module")
def patterns() -> list[re.Pattern[str]]:
    """Load and compile the `[patterns] ignore` entries of the local overlay once.

    File reading and TOML parsing happen here, at the setup boundary, so a
    malformed overlay fails the test run instead of hiding inside a query.
    """
    document = tomllib.loads(LOCAL_OVERLAY.read_text(encoding="utf-8"))
    return [re.compile(pattern) for pattern in document["patterns"]["ignore"]]


def is_covered(
    patterns: list[re.Pattern[str]], line: str, start: int, end: int
) -> bool:
    """Return whether an ignore pattern matches a span containing the term."""
    return any(
        match.start() <= start and end <= match.end()
        for pattern in patterns
        for match in pattern.finditer(line)
    )


def tracked_text_files() -> list[Path]:
    """Return tracked Markdown files, which are the files the gate scans."""
    listing = subprocess.run(
        ["git", "ls-files", "-z"],
        cwd=REPOSITORY_ROOT,
        check=True,
        capture_output=True,
    ).stdout.decode()
    return [
        REPOSITORY_ROOT / name
        for name in listing.split("\0")
        if name.endswith(".md") and name not in EXCLUDED_FILES
    ]


def test_no_term_is_accepted_as_a_bare_word() -> None:
    """The overlay accepts none of the four terms outright."""
    document = tomllib.loads(LOCAL_OVERLAY.read_text(encoding="utf-8"))
    accepted = set(document["words"]["accepted"])

    assert not accepted & set(TERMS), f"bare accepted words: {sorted(accepted)}"


def uncovered_in_line(patterns: list[re.Pattern[str]], line: str) -> bool:
    """Return whether the line holds a term that no ignore pattern covers."""
    return any(
        not is_covered(patterns, line, found.start(), found.end())
        for found in TERM_PATTERN.finditer(line)
    )


def uncovered_lines(patterns: list[re.Pattern[str]], path: Path) -> list[str]:
    """Return the `path:line: text` entries of uncovered uses in one file."""
    text = path.read_text(encoding="utf-8")
    name = path.relative_to(REPOSITORY_ROOT)
    return [
        f"{name}:{number}: {line.strip()}"
        for number, line in enumerate(text.splitlines(), start=1)
        if uncovered_in_line(patterns, line)
    ]


def test_every_current_occurrence_is_covered_by_a_pattern(
    patterns: list[re.Pattern[str]],
) -> None:
    """Each use of a term in a tracked file sits inside an ignore pattern."""
    uncovered = [
        entry
        for path in tracked_text_files()
        for entry in uncovered_lines(patterns, path)
    ]

    assert not uncovered, "occurrences outside any ignore pattern:\n" + "\n".join(
        uncovered
    )


@pytest.mark.parametrize("term", TERMS)
def test_an_unrelated_use_of_each_term_is_not_covered(
    patterns: list[re.Pattern[str]], term: str
) -> None:
    """A term in prose that no approved phrase contains is still checked."""
    line = f"Prose that merely mentions {term} in passing."
    start = line.index(term)

    assert not is_covered(patterns, line, start, start + len(term)), (
        f"an ignore pattern hides an unrelated use of {term!r}"
    )


#: Prose that the approved exact patterns exempt, one phrase per term.
APPROVED_PROSE = (
    "The `mold` linker is configured for Linux.",
    "Pass -fuse-ld=mold to the linker.",
    "Implement a simple **LOD (Level of Detail)** scheme.",
    "Dado un inventario vac\u00edo",
)


def run_gate(repository: Path, prose: str) -> subprocess.CompletedProcess[str]:
    """Run the pinned spelling gate over a scratch repository.

    Parameters
    ----------
    repository
        Empty directory that becomes the scratch repository.
    prose
        Markdown text written to `guide.md`.

    Returns
    -------
    subprocess.CompletedProcess[str]
        The finished gate process, with its exit code and captured output.
    """
    uv = shutil.which("uv")
    git = shutil.which("git")
    if uv is None or git is None:
        pytest.skip("uv and git are needed to run the spelling gate")
    makefile = (REPOSITORY_ROOT / "Makefile").read_text(encoding="utf-8")
    version = re.search(
        r"^TYPOS_CONFIG_BUILDER_VERSION\s*\?=\s*(\S+)$", makefile, re.MULTILINE
    )
    assert version is not None, "the builder version is not pinned"
    (repository / "guide.md").write_text(prose + "\n", encoding="utf-8")
    shutil.copy(LOCAL_OVERLAY, repository / "typos.local.toml")
    (repository / ".gitignore").write_text(
        ".typos-oxendict-base.json\n.typos-oxendict-base.toml\n", encoding="utf-8"
    )
    subprocess.run([git, "init", "-q"], cwd=repository, check=True)
    subprocess.run(
        [git, "add", "guide.md", "typos.local.toml", ".gitignore"],
        cwd=repository,
        check=True,
    )
    builder = "git+https://github.com/leynos/typos-config-builder.git@" + version.group(
        1
    )
    return subprocess.run(
        [
            uv,
            "tool",
            "run",
            "--python",
            "3.14",
            "--from",
            builder,
            "typos-config-builder",
            "gate",
            "--repository",
            str(repository),
        ],
        check=False,
        capture_output=True,
        text=True,
        timeout=300,
    )


def test_the_gate_accepts_the_approved_phrases(tmp_path: Path) -> None:
    """The real gate passes prose that uses only the approved exact phrases."""
    result = run_gate(tmp_path, "\n\n".join(APPROVED_PROSE))

    assert result.returncode == 0, result.stdout + result.stderr


@pytest.mark.parametrize("term", TERMS)
def test_the_gate_reports_an_unrelated_use_of_each_term(
    tmp_path: Path, term: str
) -> None:
    """The real gate reports each term when no approved phrase contains it."""
    result = run_gate(tmp_path, f"Prose that merely mentions {term} in passing.")

    assert result.returncode == 2, result.stdout + result.stderr
    assert term in result.stdout + result.stderr, (
        f"the gate failed without naming {term!r}"
    )
