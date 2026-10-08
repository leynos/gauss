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
GENERATED_OR_OVERLAY = {"typos.toml", "typos.local.toml"}


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
    """Return tracked UTF-8 text files, excluding the spelling configuration."""
    listing = subprocess.run(
        ["git", "ls-files", "-z"],
        cwd=REPOSITORY_ROOT,
        check=True,
        capture_output=True,
    ).stdout.decode()
    return [
        REPOSITORY_ROOT / name
        for name in listing.split("\0")
        if name and name not in GENERATED_OR_OVERLAY
    ]


def test_no_term_is_accepted_as_a_bare_word() -> None:
    """The overlay accepts none of the four terms outright."""
    document = tomllib.loads(LOCAL_OVERLAY.read_text(encoding="utf-8"))
    accepted = set(document["words"]["accepted"])

    assert not accepted & set(TERMS), f"bare accepted words: {sorted(accepted)}"


def test_every_current_occurrence_is_covered_by_a_pattern() -> None:
    """Each use of a term in a tracked file sits inside an ignore pattern."""
    uncovered: list[str] = []
    for path in tracked_text_files():
        try:
            text = path.read_text(encoding="utf-8")
        except (UnicodeDecodeError, OSError):
            continue
        for number, line in enumerate(text.splitlines(), start=1):
            for found in TERM_PATTERN.finditer(line):
                if not is_covered(line, found.start(), found.end()):
                    name = path.relative_to(REPOSITORY_ROOT)
                    uncovered.append(f"{name}:{number}: {line.strip()}")

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
