"""Regression tests for the repository's spelling exceptions.

`typos.local.toml` must exempt `mold`, `Mold`, `LOD` and `inventario` only
through exact patterns, never as bare accepted words. These tests hold that
contract: every current occurrence is covered by a pattern, and an unrelated
use of any of the four terms is not.
"""

from __future__ import annotations

import re
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


def ignore_patterns() -> list[re.Pattern[str]]:
    """Return the compiled `[patterns] ignore` entries of the local overlay."""
    document = tomllib.loads(LOCAL_OVERLAY.read_text(encoding="utf-8"))
    return [re.compile(pattern) for pattern in document["patterns"]["ignore"]]


def is_covered(line: str, start: int, end: int) -> bool:
    """Return whether an ignore pattern matches a span containing the term."""
    return any(
        match.start() <= start and end <= match.end()
        for pattern in ignore_patterns()
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


def uncovered_in_line(line: str) -> bool:
    """Return whether the line holds a term that no ignore pattern covers."""
    return any(
        not is_covered(line, found.start(), found.end())
        for found in TERM_PATTERN.finditer(line)
    )


def uncovered_lines(path: Path) -> list[str]:
    """Return the `path:line: text` entries of uncovered uses in one file."""
    try:
        text = path.read_text(encoding="utf-8")
    except (UnicodeDecodeError, OSError):
        return []
    name = path.relative_to(REPOSITORY_ROOT)
    return [
        f"{name}:{number}: {line.strip()}"
        for number, line in enumerate(text.splitlines(), start=1)
        if uncovered_in_line(line)
    ]


def test_every_current_occurrence_is_covered_by_a_pattern() -> None:
    """Each use of a term in a tracked file sits inside an ignore pattern."""
    uncovered = [
        entry for path in tracked_text_files() for entry in uncovered_lines(path)
    ]

    assert not uncovered, "occurrences outside any ignore pattern:\n" + "\n".join(
        uncovered
    )


@pytest.mark.parametrize("term", TERMS)
def test_an_unrelated_use_of_each_term_is_not_covered(term: str) -> None:
    """A term in prose that no approved phrase contains is still checked."""
    line = f"Prose that merely mentions {term} in passing."
    start = line.index(term)

    assert not is_covered(line, start, start + len(term)), (
        f"an ignore pattern hides an unrelated use of {term!r}"
    )
