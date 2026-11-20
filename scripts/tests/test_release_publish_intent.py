"""Regression tests for the release publish-intent resolver.

The pinned ``determine-release-modes`` action returns ``should-publish: true``
for any ``push`` event before it reads its own ``publish`` input. A reusable
caller inherits the caller's event context, so a caller that asks for
``publish: false`` from a tag push is indistinguishable from a direct tag push
by event name alone. `release_publish_intent` narrows the action's verdict
using the workflow's `inputs` context, which is populated only for a reusable
call.

These tests pin the properties that make that safe: the resolver may only
downgrade the action's verdict, it never raises one, and a direct tag push
without a caller keeps publishing.
"""

from __future__ import annotations

from pathlib import Path

import pytest
from hypothesis import given, strategies as st
from release_publish_intent import (
    coerce_bool,
    main,
    resolve_publish_intent,
)


@pytest.mark.parametrize(
    ("pinned", "publish_input", "expected"),
    [
        # A direct tag push supplies no caller input, so the event stands.
        pytest.param(True, "", True, id="direct-tag-push-publishes"),
        # A reusable caller may decline even though the inherited event is a
        # tag push. This is the defect the resolver exists to correct.
        pytest.param(True, "false", False, id="caller-declines-on-tag-push"),
        pytest.param(True, "true", True, id="caller-opts-in"),
        # The declared `publish` default arrives as `false` for a called
        # workflow, so a caller that omits the input stays unpublished.
        pytest.param(True, "false", False, id="caller-omits-publish"),
        # A verdict the action already declined is never raised, whatever the
        # caller asks for.
        pytest.param(False, "true", False, id="dry-run-cannot-publish"),
        pytest.param(False, "", False, id="declined-verdict-stands"),
    ],
)
def test_resolve_publish_intent_narrows_the_action_verdict(
    *, pinned: bool, publish_input: str, expected: bool
) -> None:
    """The resolver may turn publishing off, never on."""
    actual = resolve_publish_intent(
        pinned_should_publish=pinned,
        publish_input=publish_input,
        dry_run_input="",
    )

    assert actual is expected, (
        "publish intent should narrow the pinned verdict; "
        f"pinned={pinned}, publish_input={publish_input!r}, got {actual}"
    )


@pytest.mark.parametrize(
    ("dry_run_input", "expected"),
    [
        ("true", False),
        ("1", False),
        ("yes", False),
        ("", True),
        ("false", True),
    ],
)
def test_dry_run_input_outranks_publish(*, dry_run_input: str, expected: bool) -> None:
    """A dry run never publishes, even when the caller opts in."""
    actual = resolve_publish_intent(
        pinned_should_publish=True,
        publish_input="true",
        dry_run_input=dry_run_input,
    )

    assert actual is expected, (
        "dry-run input must outrank publish; "
        f"dry_run_input={dry_run_input!r}, got {actual}"
    )


@given(
    pinned=st.booleans(),
    publish_input=st.sampled_from(["", "true", "false", "TRUE", " False "]),
    dry_run_input=st.sampled_from(["", "true", "false"]),
)
def test_resolver_never_enables_a_declined_release(
    *, pinned: bool, publish_input: str, dry_run_input: str
) -> None:
    """No input combination can publish a run the action declined.

    This is the property the workflow's release guard relies on: the resolver
    sits between the pinned action and the `release` job's `if:`, so a bug
    that raised a declined verdict would upload assets from a dry run.
    """
    actual = resolve_publish_intent(
        pinned_should_publish=pinned,
        publish_input=publish_input,
        dry_run_input=dry_run_input,
    )

    assert actual is False or pinned is True, (
        "resolver enabled a release the action declined; "
        f"pinned={pinned}, publish_input={publish_input!r}, "
        f"dry_run_input={dry_run_input!r}"
    )


@pytest.mark.parametrize(
    ("raw", "default", "expected"),
    [
        ("true", False, True),
        ("TRUE", False, True),
        (" False ", True, False),
        ("", True, True),
        ("", False, False),
        # An unrecognized spelling keeps the caller's default rather than
        # guessing, which matches the pinned action's own coercion.
        ("maybe", True, True),
    ],
)
def test_coerce_bool(*, raw: str, default: bool, expected: bool) -> None:
    """Input coercion tolerates GitHub's spellings and keeps a default."""
    actual = coerce_bool(raw, default=default)

    assert actual is expected, (
        f"coerce_bool({raw!r}, default={default}) returned {actual}"
    )


def test_main_writes_the_value_github_output_consumes(
    *, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """The step output is what the release job's `if:` reads.

    The workflow consumes `steps.publish_intent.outputs.value`, so the key and
    its rendering are part of the contract, not an implementation detail.
    """
    output_path = tmp_path / "github_output"
    monkeypatch.setenv("GITHUB_OUTPUT", str(output_path))

    status = main(
        [
            "--pinned-should-publish",
            "true",
            "--publish-input",
            "false",
            "--dry-run-input",
            "",
        ]
    )

    written = output_path.read_text(encoding="utf-8")
    assert status == 0, f"main should succeed, returned {status}"
    assert written == "value=false\n", f"unexpected GITHUB_OUTPUT contents: {written!r}"


def test_main_requires_every_input() -> None:
    """A missing input fails the step instead of silently not publishing.

    A release that quietly does nothing is far harder to notice than a job
    that fails, so the resolver refuses to guess.
    """
    with pytest.raises(SystemExit) as raised:
        main(["--pinned-should-publish", "true"])

    assert raised.value.code == 2, (
        "argparse should reject the incomplete invocation, "
        f"got exit {raised.value.code}"
    )


def test_module_does_not_read_ambient_event_state() -> None:
    """The resolver depends on declared inputs, not the inherited event.

    `GITHUB_EVENT_NAME` describes the caller's trigger and is the very thing
    that made the pinned action's verdict wrong for a reusable call. Reading
    it here would reintroduce the defect.
    """
    module_source = (
        Path(__file__).resolve().parents[1] / "release_publish_intent.py"
    ).read_text(encoding="utf-8")

    assert "GITHUB_EVENT_NAME" not in module_source, (
        "the resolver must not consult the inherited event name"
    )
