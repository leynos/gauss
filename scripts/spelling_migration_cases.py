"""Pure inputs and strategies for the local spelling migration properties.

These examples describe only the approved grammar. The tests load the actual
patterns from ``typos.local.toml`` and check these generated strings against
those patterns.
"""

from __future__ import annotations

from collections.abc import Callable
from typing import NamedTuple

from hypothesis import strategies as st


class SpellingMigrationContext(NamedTuple):
    """Describe an approved spelling-migration phrase for generated tests.

    Attributes
    ----------
    case_id : str
        Identifier used to select the overlay pattern and label test cases.
    layout : str
        Phrase layout: ``"table"`` for a migration row or ``"statement"``.
    canonical : str
        Fixed approved phrase rendered with single-space separators.
    previous_cell : str
        Previous spelling in a table row; empty for a statement.
    current_cell : str
        Current spelling in a table row; empty for a statement.
    legacy_tokens : tuple[str, ...]
        Previous-spelling tokens whose spans the approved rule must cover.
    mismatched_pair_from : str
        Exact fragment replaced once to make a mismatched-name near miss.
    mismatched_pair_to : str
        Replacement fragment inserted for that mismatched-name near miss.
    """

    case_id: str
    layout: str
    canonical: str
    previous_cell: str
    current_cell: str
    legacy_tokens: tuple[str, ...]
    mismatched_pair_from: str
    mismatched_pair_to: str


NearMissResult = tuple[str, str, bool]
NearMissBuilder = Callable[
    [SpellingMigrationContext, str, str, list[str]], NearMissResult
]


MIGRATION_CONTEXTS = (
    SpellingMigrationContext(
        "colour-type",
        "table",
        "| `Color` | `Colour` |",
        "Color",
        "Colour",
        ("Color",),
        "`Colour`",
        "`Color`",
    ),
    SpellingMigrationContext(
        "stroke-action",
        "table",
        "| `Action::SetStrokeColor` | `Action::SetStrokeColour` |",
        "Action::SetStrokeColor",
        "Action::SetStrokeColour",
        ("Action::SetStrokeColor",),
        "`Action::SetStrokeColour`",
        "`Action::SetFillColour`",
    ),
    SpellingMigrationContext(
        "fill-action",
        "table",
        "| `Action::SetFillColor` | `Action::SetFillColour` |",
        "Action::SetFillColor",
        "Action::SetFillColour",
        ("Action::SetFillColor",),
        "`Action::SetFillColour`",
        "`Action::SetStrokeColour`",
    ),
    SpellingMigrationContext(
        "recorded-identifiers",
        "statement",
        "`Action::identifier()` intentionally retains the recorded strings "
        "`SetStrokeColor` and `SetFillColor`; persisted identifiers are unchanged.",
        "",
        "",
        ("SetStrokeColor", "SetFillColor"),
        "`SetFillColor`;",
        "`SetFillColour`;",
    ),
)

STATEMENT_PARTS = (
    "`Action::identifier()`",
    "intentionally",
    "retains",
    "the",
    "recorded",
    "strings",
    "`SetStrokeColor`",
    "and",
    "`SetFillColor`;",
    "persisted",
    "identifiers",
    "are",
    "unchanged.",
)
NEAR_MISS_CATEGORIES = (
    "mismatched current-name pair",
    "altered legacy identifier",
    "malformed backtick boundary",
    "nonapproved line break",
    "ordinary prose or source use",
    "unrelated old name beside an approved context",
)


def render_migration_context(
    context: SpellingMigrationContext, separators: list[str]
) -> str:
    """Render one approved migration context with generated whitespace.

    Example:
        ``render_migration_context(MIGRATION_CONTEXTS[0], [" ", " ", " ", " "])``
        returns ``| `Color` | `Colour` |``.
    """
    if context.layout == "table":
        before_previous, after_previous, before_current, after_current = separators
        return (
            f"|{before_previous}`{context.previous_cell}`{after_previous}|"
            f"{before_current}`{context.current_cell}`{after_current}|"
        )
    return (
        "".join(
            part + separator
            for part, separator in zip(STATEMENT_PARTS[:-1], separators, strict=True)
        )
        + STATEMENT_PARTS[-1]
    )


def separator_strategy(context: SpellingMigrationContext):
    """Return independent bounded whitespace for this context.

    A table draw contains four strings of spaces and tabs, each up to 16
    characters. A statement draw has one nonempty whitespace string between
    each adjacent pair of recorded words and may include CR or LF.
    """
    if context.layout == "table":
        whitespace = st.text(alphabet=" \t", min_size=0, max_size=16)
        return st.lists(whitespace, min_size=4, max_size=4)

    whitespace = st.text(alphabet=" \t\r\n", min_size=1, max_size=16)
    separator_count = len(STATEMENT_PARTS) - 1
    return st.lists(whitespace, min_size=separator_count, max_size=separator_count)


def _mismatched_current_pair(
    context: SpellingMigrationContext,
    legacy: str,
    approved: str,
    _separators: list[str],
) -> NearMissResult:
    """Replace the current-name cell with its specified mismatched name.

    Example: the stroke row uses ``Action::SetFillColour`` as its current name.
    """
    return (
        approved.replace(context.mismatched_pair_from, context.mismatched_pair_to, 1),
        legacy,
        True,
    )


def _altered_legacy_identifier(
    _context: SpellingMigrationContext,
    legacy: str,
    approved: str,
    _separators: list[str],
) -> NearMissResult:
    """Append ``X`` to the legacy token inside its code span.

    Example: ``Action::SetStrokeColor`` becomes ``Action::SetStrokeColorX``.
    """
    altered = legacy + "X"
    return approved.replace(f"`{legacy}`", f"`{altered}`", 1), altered, False


def _malformed_backtick_boundary(
    _context: SpellingMigrationContext,
    legacy: str,
    approved: str,
    _separators: list[str],
) -> NearMissResult:
    """Remove the opening backtick while retaining the closing delimiter.

    Example: `` `Color` `` becomes `` Color` ``.
    """
    return approved.replace(f"`{legacy}`", f"{legacy}`", 1), legacy, False


def _nonapproved_line_break(
    context: SpellingMigrationContext,
    legacy: str,
    approved: str,
    _separators: list[str],
) -> NearMissResult:
    """Insert a line break inside the legacy token's code span.

    Example: a table token becomes `` `Color\\n` ``; a statement splits
    ``SetStrokeColor`` between ``SetStroke`` and ``Color``.
    """
    if context.layout == "table":
        malformed = f"`{legacy}\n`"
        return approved.replace(f"`{legacy}`", malformed, 1), legacy, False
    broken_identifier = "SetStroke\nColor"
    return (
        approved.replace(f"`{legacy}`", f"`{broken_identifier}`", 1),
        broken_identifier,
        False,
    )


def _source_reference(context: SpellingMigrationContext, legacy: str) -> str:
    """Name the old spelling as it appears in a table or statement.

    Example: a table uses ``Color`` and the recorded-identifier statement uses
    ``Action::SetStrokeColor``.
    """
    if context.layout == "table":
        return context.previous_cell
    return f"Action::{legacy}"


def _ordinary_prose_or_source_use(
    context: SpellingMigrationContext,
    legacy: str,
    _approved: str,
    separators: list[str],
) -> NearMissResult:
    """Place the old name in prose outside any approved migration context.

    Example: ``Source code mentions `Color` outside the migration context.``
    """
    name = _source_reference(context, legacy)
    before, after = separators[:2]
    return (
        f"Source code mentions{before}`{name}`{after}outside the migration context.",
        name,
        False,
    )


def _unrelated_old_name_beside_context(
    context: SpellingMigrationContext,
    legacy: str,
    approved: str,
    separators: list[str],
) -> NearMissResult:
    """Append an exposed old name after a still-valid approved context.

    Example: the approved row remains on its line before a separate ``Color`` use.
    """
    name = _source_reference(context, legacy)
    before, after = separators[:2]
    return (
        f"{approved}\nSource code mentions{before}`{name}`{after}outside it.",
        name,
        True,
    )


_NEAR_MISS_BUILDERS: dict[str, NearMissBuilder] = {
    "mismatched current-name pair": _mismatched_current_pair,
    "altered legacy identifier": _altered_legacy_identifier,
    "malformed backtick boundary": _malformed_backtick_boundary,
    "nonapproved line break": _nonapproved_line_break,
    "ordinary prose or source use": _ordinary_prose_or_source_use,
    "unrelated old name beside an approved context": _unrelated_old_name_beside_context,
}


def make_near_miss(
    context: SpellingMigrationContext,
    approved: str,
    separators: list[str],
    category: str,
) -> tuple[str, str, bool]:
    """Create one malformed input and identify its forbidden token.

    For example, an altered stroke identifier changes
    ``Action::SetStrokeColor`` to ``Action::SetStrokeColorX``.
    """
    legacy = context.legacy_tokens[0]
    builder = _NEAR_MISS_BUILDERS.get(category)
    if builder is None:
        raise ValueError(f"unknown migration near-miss category: {category}")
    return builder(context, legacy, approved, separators)


def token_bounds(
    line: str, token: str, last_occurrence: bool = False
) -> tuple[int, int]:
    """Return the full token span, selecting its last occurrence when asked.

    Example:
        ``token_bounds("Color then Color", "Color", True)`` returns ``(11, 16)``.
    """
    start = line.rindex(token) if last_occurrence else line.index(token)
    return start, start + len(token)
