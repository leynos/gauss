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
from hypothesis import given, strategies as st
from spelling_migration_cases import (
    MIGRATION_CONTEXTS,
    NEAR_MISS_CATEGORIES,
    STATEMENT_PARTS,
    SpellingMigrationContext,
    make_near_miss,
    render_migration_context,
    separator_strategy,
    token_bounds,
)

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


def _approved_context_spans(
    category: str, context: SpellingMigrationContext, approved: str
) -> list[tuple[str, int, int]]:
    """Select original migration spans retained by the adjacent case.

    Example: the type row yields ``[("Color", 3, 8)]`` for its approved token.
    """
    if category != "unrelated old name beside an approved context":
        return []
    return [(token, *token_bounds(approved, token)) for token in context.legacy_tokens]


def _diagnostic_colour_spans(
    category: str, line: str, token: str, token_start: int
) -> list[tuple[int, int]]:
    """Select the Color diagnostic spans to check in a near-miss line.

    Example: adjacent cases select the added token's ``Color`` span; others
    select every literal ``Color`` occurrence.
    """
    if category == "unrelated old name beside an approved context":
        start = token_start + token.rfind("Color")
        return [(start, start + len("Color"))]
    return [(match.start(), match.end()) for match in re.finditer("Color", line)]


def _assert_approved_context_coverage(
    case_id: str,
    pattern: re.Pattern[str],
    line: str,
    labelled_spans: list[tuple[str, int, int]],
) -> None:
    """Assert each labelled approved token stays covered by its local rule."""
    for token, start, end in labelled_spans:
        assert is_covered([pattern], line, start, end), (
            f"{case_id}: approved context lost coverage for {token!r}"
        )


def _assert_diagnostic_spans_uncovered(
    case_category_label: str,
    patterns: list[re.Pattern[str]],
    line: str,
    spans: list[tuple[int, int]],
) -> None:
    """Assert every selected Color diagnostic span remains visible."""
    assert spans and all(start >= 0 for start, _ in spans), (
        f"{case_category_label} has no Color diagnostic span in {line!r}"
    )
    for start, end in spans:
        assert not is_covered(patterns, line, start, end), (
            f"{case_category_label} hides the Color diagnostic span "
            f"at {start} in {line!r}"
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


@pytest.fixture(scope="module")
def migration_patterns(
    patterns: list[re.Pattern[str]],
) -> dict[str, re.Pattern[str]]:
    """Bind each canonical context to its unique rule from the local overlay."""
    selected: dict[str, re.Pattern[str]] = {}
    for context in MIGRATION_CONTEXTS:
        separator_count = 4 if context.layout == "table" else len(STATEMENT_PARTS) - 1
        separators = [" "] * separator_count
        assert render_migration_context(context, separators) == context.canonical, (
            f"{context.case_id}: generated example differs from its documented text"
        )
        matches = [
            pattern for pattern in patterns if pattern.fullmatch(context.canonical)
        ]
        assert len(matches) == 1, (
            f"{context.case_id}: expected exactly one local overlay pattern, "
            f"found {len(matches)}"
        )
        selected[context.case_id] = matches[0]

    assert len({id(pattern) for pattern in selected.values()}) == len(
        MIGRATION_CONTEXTS
    ), "migration contexts must use four distinct local overlay patterns"
    return selected


@pytest.mark.parametrize(
    "context", MIGRATION_CONTEXTS, ids=lambda context: context.case_id
)
@given(data=st.data())
def test_migration_exceptions_cover_generated_whitespace(
    context: SpellingMigrationContext,
    migration_patterns: dict[str, re.Pattern[str]],
    data,
) -> None:
    """Each approved migration token stays inside its actual local rule."""
    separators = data.draw(
        separator_strategy(context), label="approved-context-whitespace"
    )
    line = render_migration_context(context, separators)
    pattern = migration_patterns[context.case_id]

    for token in context.legacy_tokens:
        start = line.index(token)
        end = start + len(token)
        assert is_covered([pattern], line, start, end), (
            f"{context.case_id}: local rule did not cover {token!r} in {line!r}"
        )


@pytest.mark.parametrize(
    "context", MIGRATION_CONTEXTS, ids=lambda context: context.case_id
)
@given(data=st.data())
def test_migration_exceptions_leave_generated_near_misses_uncovered(
    context: SpellingMigrationContext,
    migration_patterns: dict[str, re.Pattern[str]],
    patterns: list[re.Pattern[str]],
    data,
) -> None:
    """Every near-miss keeps its complete forbidden token span visible."""
    pattern = migration_patterns[context.case_id]
    for category in NEAR_MISS_CATEGORIES:
        separators = data.draw(
            separator_strategy(context),
            label=f"{context.case_id}-{category} whitespace",
        )
        approved = render_migration_context(context, separators)
        line, token, last_occurrence = make_near_miss(
            context, approved, separators, category
        )
        approved_spans = _approved_context_spans(category, context, approved)
        _assert_approved_context_coverage(
            context.case_id, pattern, line, approved_spans
        )

        start, end = token_bounds(line, token, last_occurrence)
        assert not is_covered(patterns, line, start, end), (
            f"{context.case_id} {category} hides {token!r} in {line!r}"
        )
        colour_spans = _diagnostic_colour_spans(category, line, token, start)
        _assert_diagnostic_spans_uncovered(
            f"{context.case_id} {category}", patterns, line, colour_spans
        )


#: Prose that the approved exact patterns exempt, one phrase per term.
APPROVED_PROSE = (
    "The `mold` linker is configured for Linux.",
    "Pass -fuse-ld=mold to the linker.",
    "Implement a simple **LOD (Level of Detail)** scheme.",
    "Dado un inventario vac\u00edo",
    *(context.canonical for context in MIGRATION_CONTEXTS),
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
    assert {context.case_id for context in MIGRATION_CONTEXTS} == {
        "colour-type",
        "stroke-action",
        "fill-action",
        "recorded-identifiers",
    }
    result = run_gate(tmp_path, "\n\n".join(APPROVED_PROSE))

    assert result.returncode == 0, result.stdout + result.stderr


def test_the_gate_reports_an_unrelated_colour_type_name(tmp_path: Path) -> None:
    """The real gate diagnoses Color outside every approved migration phrase."""
    result = run_gate(tmp_path, "The legacy `Color` type is named here.")
    output = result.stdout + result.stderr

    assert result.returncode == 2, output
    assert "guide.md:1" in output, output
    assert re.search(r"`Color` should be `Colour`", output), output


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
