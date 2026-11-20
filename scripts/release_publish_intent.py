#!/usr/bin/env python3
"""Resolve whether the release workflow may publish its artefacts.

The pinned ``determine-release-modes`` action derives ``should-publish`` from
the event name. It returns ``True`` for ``push`` before it consults its own
``publish`` input, so a reusable caller that asks for ``publish: false`` while
the caller's event is a tag push is still told to publish. The action cannot
tell the two situations apart on its own: a called workflow inherits the
caller's event context, so both present ``push``.

A caller can be distinguished from a direct run by this workflow's ``inputs``
context, which is populated only when the workflow was invoked through
``workflow_call``. When it is populated, the caller's ``publish`` value is
authoritative. When it is empty, the event-derived verdict stands, so a direct
tag push keeps publishing exactly as before.

This module therefore narrows the action's verdict rather than replacing it.
It can only ever turn publishing *off*: it never enables a release that the
action declined, so a dry run stays a dry run whatever the caller asks for.

Examples
--------
A direct tag push declares no inputs, so the action's verdict is kept::

    >>> resolve_publish_intent(
    ...     pinned_should_publish=True, publish_input="", dry_run_input=""
    ... )
    True

A reusable caller that sets ``publish: false`` overrides the tag-push verdict::

    >>> resolve_publish_intent(
    ...     pinned_should_publish=True, publish_input="false", dry_run_input=""
    ... )
    False

A verdict the action already declined is never raised, even when the caller
asks to publish::

    >>> resolve_publish_intent(
    ...     pinned_should_publish=False, publish_input="true", dry_run_input=""
    ... )
    False
"""

from __future__ import annotations

import argparse
import os
import sys
import typing as typ
from pathlib import Path

_TRUE_VALUES = {"true", "1", "yes", "on"}
_FALSE_VALUES = {"false", "0", "no", "off"}


def coerce_bool(raw: str, *, default: bool) -> bool:
    """Interpret a workflow input as a boolean.

    GitHub renders a boolean workflow input in the ``inputs`` context as
    ``true`` or ``false``, but an environment override can carry any spelling.
    An empty value means "not supplied", which keeps the caller's default.

    Parameters
    ----------
    raw
        The value to interpret, as it arrived from the workflow.
    default
        The value to use when ``raw`` is empty or unrecognized.

    Returns
    -------
    bool
        The interpreted value.

    Examples
    --------
    >>> coerce_bool("true", default=False)
    True
    >>> coerce_bool("", default=True)
    True
    >>> coerce_bool("not-a-boolean", default=True)
    True
    """
    normalized = raw.strip().lower()
    if normalized in _TRUE_VALUES:
        return True
    if normalized in _FALSE_VALUES:
        return False
    return default


def resolve_publish_intent(
    *, pinned_should_publish: bool, publish_input: str, dry_run_input: str
) -> bool:
    """Narrow the action's publish verdict using the caller's own input.

    Parameters
    ----------
    pinned_should_publish
        The ``should-publish`` output of the pinned action, which reflects the
        inherited event and may not see a caller's override.
    publish_input
        This workflow's ``publish`` input. Empty means the workflow was not
        invoked as a reusable workflow, so the event-derived verdict is
        authoritative.
    dry_run_input
        This workflow's ``dry-run`` input. When it is truthy, publishing stays
        off regardless of any other input.

    Returns
    -------
    bool
        Whether the release job may publish.

    Examples
    --------
    A caller cannot publish out of a dry run::

        >>> resolve_publish_intent(
        ...     pinned_should_publish=False,
        ...     publish_input="true",
        ...     dry_run_input="true",
        ... )
        False
    """
    if not pinned_should_publish:
        return False
    if coerce_bool(dry_run_input, default=False):
        return False
    if not publish_input.strip():
        return True
    return coerce_bool(publish_input, default=True)


def _append_output(output_path: Path, name: str, value: str) -> None:
    """Append one assignment to the step's ``GITHUB_OUTPUT`` file.

    Examples
    --------
    >>> import tempfile
    >>> with tempfile.TemporaryDirectory() as directory:
    ...     target = Path(directory) / "output"
    ...     target.touch()
    ...     _append_output(target, "value", "true")
    ...     target.read_text(encoding="utf-8")
    'value=true\\n'
    """
    with output_path.open("a", encoding="utf-8") as handle:
        handle.write(f"{name}={value}\n")


def _format_bool(value: bool) -> str:
    """Render a boolean the way GitHub compares step outputs.

    Examples
    --------
    >>> _format_bool(True)
    'true'
    >>> _format_bool(False)
    'false'
    """
    return "true" if value else "false"


def _parse_args(argv: typ.Sequence[str]) -> argparse.Namespace:
    """Parse the workflow-supplied inputs from the command line.

    Every input is required on the command line so a missing value fails the
    step rather than silently resolving to "do not publish": a release that
    quietly does nothing is harder to notice than a failed job.

    Examples
    --------
    >>> _parse_args(
    ...     [
    ...         "--pinned-should-publish",
    ...         "true",
    ...         "--publish-input",
    ...         "false",
    ...         "--dry-run-input",
    ...         "",
    ...     ]
    ... ).publish_input
    'false'
    """
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pinned-should-publish", required=True)
    parser.add_argument("--publish-input", required=True)
    parser.add_argument("--dry-run-input", required=True)
    return parser.parse_args(argv)


def main(argv: typ.Sequence[str] | None = None) -> int:
    """Resolve the publish intent and write it to ``GITHUB_OUTPUT``.

    Returns
    -------
    int
        The process exit status. ``0`` means the intent was written.

    Examples
    --------
    The workflow captures the ``value`` output, so a caller asking for
    ``publish: false`` from a tag push yields ``false``::

        >>> import os, tempfile
        >>> with tempfile.TemporaryDirectory() as directory:
        ...     target = Path(directory) / "output"
        ...     target.touch()
        ...     os.environ["GITHUB_OUTPUT"] = str(target)
        ...     main(
        ...         [
        ...             "--pinned-should-publish",
        ...             "true",
        ...             "--publish-input",
        ...             "false",
        ...             "--dry-run-input",
        ...             "",
        ...         ]
        ...     )
        ...     target.read_text(encoding="utf-8")
        Publish intent: false
        0
        'value=false\\n'
    """
    args = _parse_args(sys.argv[1:] if argv is None else argv)
    intent = resolve_publish_intent(
        pinned_should_publish=coerce_bool(args.pinned_should_publish, default=False),
        publish_input=args.publish_input,
        dry_run_input=args.dry_run_input,
    )
    output_path = Path(os.environ["GITHUB_OUTPUT"]).resolve()
    _append_output(output_path, "value", _format_bool(intent))
    print(f"Publish intent: {_format_bool(intent)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
