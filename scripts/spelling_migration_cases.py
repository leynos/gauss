"""Pure inputs and strategies for the local spelling migration properties.

These examples describe only the approved grammar. The tests load the actual
patterns from ``typos.local.toml`` and check these generated strings against
those patterns.
"""

from __future__ import annotations

from typing import NamedTuple

from hypothesis import strategies as st


class SpellingMigrationContext(NamedTuple):
    case_id: str
    layout: str
    canonical: str
    previous_cell: str
    current_cell: str
    legacy_tokens: tuple[str, ...]
    mismatched_pair_from: str
    mismatched_pair_to: str


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
    if category == "mismatched current-name pair":
        return (
            approved.replace(
                context.mismatched_pair_from, context.mismatched_pair_to, 1
            ),
            legacy,
            True,
        )
    if category == "altered legacy identifier":
        altered = legacy + "X"
        return approved.replace(f"`{legacy}`", f"`{altered}`", 1), altered, False
    if category == "malformed backtick boundary":
        return approved.replace(f"`{legacy}`", f"{legacy}`", 1), legacy, False
    if category == "nonapproved line break":
        if context.layout == "table":
            malformed = f"`{legacy}\n`"
            return approved.replace(f"`{legacy}`", malformed, 1), legacy, False
        broken_identifier = "SetStroke\nColor"
        return (
            approved.replace(f"`{legacy}`", f"`{broken_identifier}`", 1),
            broken_identifier,
            False,
        )
    if category == "ordinary prose or source use":
        name = (
            context.previous_cell if context.layout == "table" else f"Action::{legacy}"
        )
        before, after = separators[:2]
        return (
            f"Source code mentions{before}`{name}`"
            f"{after}outside the migration context.",
            name,
            False,
        )
    if category == "unrelated old name beside an approved context":
        name = (
            context.previous_cell if context.layout == "table" else f"Action::{legacy}"
        )
        before, after = separators[:2]
        return (
            f"{approved}\nSource code mentions{before}`{name}`{after}outside it.",
            name,
            True,
        )
    raise ValueError(f"unknown migration near-miss category: {category}")


def token_bounds(
    line: str, token: str, last_occurrence: bool = False
) -> tuple[int, int]:
    """Return the full token span, selecting its last occurrence when asked.

    Example:
        ``token_bounds("Color then Color", "Color", True)`` returns ``(11, 16)``.
    """
    start = line.rindex(token) if last_occurrence else line.index(token)
    return start, start + len(token)
