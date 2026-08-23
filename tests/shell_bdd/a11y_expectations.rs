//! Rendered accessibility-tree expectations for shell BDD scenarios.

use accesskit::{Action, Role, TreeUpdate};
use gauss::ui::phase0_shell::accessibility;
use test_support::{TestSupportError, TestSupportResult};

use crate::{expect_equal_support::expect_equal, expect_true_support::expect_true};

fn expect_chrome_button(
    update: &TreeUpdate,
    expected: &accessibility::ChromeButtonSemantics,
) -> TestSupportResult<()> {
    let node =
        accessibility::chrome_node_from_update(update, expected.node_id).ok_or_else(|| {
            TestSupportError::missing(
                "accessibility node",
                format!("node {:#x}", expected.node_id),
            )
        })?;
    expect_equal(&node.role(), &Role::Button, "chrome button role")?;
    expect_equal(&node.label(), &Some(expected.label), "chrome button label")?;
    expect_equal(
        &node.description(),
        &Some(expected.shortcut_hint),
        "chrome button description",
    )?;
    expect_equal(
        &node.keyboard_shortcut(),
        &Some(expected.shortcut_hint),
        "chrome button shortcut",
    )?;
    expect_true(node.supports_action(Action::Click), "button supports click")
}

/// Checks the metadata and chrome controls in an initial accessibility update.
pub(super) fn expect_initial_tree_metadata_and_buttons(
    update: &TreeUpdate,
) -> TestSupportResult<()> {
    expect_true(
        update.tree.is_some(),
        "initial update includes tree metadata",
    )?;
    let titlebar =
        accessibility::chrome_node_from_update(update, accessibility::node_ids::TITLEBAR)
            .ok_or_else(|| TestSupportError::missing("titlebar node", "initial update"))?;
    expect_equal(&titlebar.role(), &Role::TitleBar, "titlebar role")?;
    expect_equal(
        &titlebar.label(),
        &Some(accessibility::accessible_names::TITLEBAR),
        "titlebar label",
    )?;
    for expected in accessibility::chrome_button_semantics(false) {
        expect_chrome_button(update, &expected)?;
    }
    Ok(())
}
