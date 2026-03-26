//! i18n-specific tests for accessibility tree status labels.

use std::collections::{BTreeSet, HashMap};

use crate::i18n::{Catalog, Locale, Localizer};
use crate::model::{EdgeMode, ToolMode};
use crate::ui::phase0_shell::a11y_service::{
    A11yShapeSnapshot, A11ySnapshot,
    tree_builder::{STATUS_NODE_ID, build_node_map},
};

// Import shape_id from parent module
use super::shape_id;

fn fr_locale_snapshot(localizer: Localizer) -> A11ySnapshot {
    A11ySnapshot {
        tool_mode: ToolMode::Draw,
        edge_mode: EdgeMode::Line,
        can_undo: false,
        can_redo: false,
        is_maximized: false,
        selected_shape_ids: BTreeSet::new(),
        shapes: vec![],
        localizer,
        locale: Locale::fr_fr(),
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Test helper - flexibility needed for test state construction"
)]
fn snapshot_with_full_state(
    shape_ids: &[u64],
    selected_ids: &[u64],
    is_maximized: bool,
    tool_mode: ToolMode,
    edge_mode: EdgeMode,
    localizer: Localizer,
    locale: Locale,
) -> A11ySnapshot {
    let shapes = shape_ids
        .iter()
        .enumerate()
        .map(|(index, id)| A11yShapeSnapshot {
            id: shape_id(*id),
            name: Some(format!("Shape {}", index + 1)),
            locked: false,
            hidden: false,
        })
        .collect::<Vec<_>>();
    let selected_shape_ids = selected_ids.iter().map(|id| shape_id(*id)).collect();
    A11ySnapshot {
        tool_mode,
        edge_mode,
        can_undo: false,
        can_redo: false,
        is_maximized,
        selected_shape_ids,
        shapes,
        localizer,
        locale,
    }
}

fn get_status_label(snapshot: &A11ySnapshot) -> String {
    let (nodes, _) = build_node_map(snapshot).expect("node map build should succeed");
    let status_node = nodes.get(&STATUS_NODE_ID).unwrap_or_else(|| {
        panic!(
            "expected node map to contain status node id {:#x}",
            STATUS_NODE_ID.0
        )
    });
    status_node
        .label()
        .expect("status node should have a label")
        .to_owned()
}

#[rstest::rstest]
fn status_node_uses_localized_tool_mode_label() {
    let mut messages = HashMap::new();
    messages.insert("tool_mode.draw".to_owned(), "Dessiner".to_owned());
    let mut catalogs = HashMap::new();
    catalogs.insert(Locale::fr_fr(), Catalog::from_messages(messages));
    let localizer = Localizer::with_catalogs(catalogs, Locale::en_gb());

    let label = get_status_label(&fr_locale_snapshot(localizer));
    assert!(
        label.contains("Dessiner"),
        "Expected localized French 'Dessiner', got: {label}"
    );
}

#[rstest::rstest]
fn status_node_uses_localized_edge_mode_label() {
    let mut messages = HashMap::new();
    messages.insert("edge_mode.line".to_owned(), "Ligne".to_owned());
    let mut catalogs = HashMap::new();
    catalogs.insert(Locale::fr_fr(), Catalog::from_messages(messages));
    let localizer = Localizer::with_catalogs(catalogs, Locale::en_gb());

    // Use snapshot_with_full_state to exercise localized_edge_mode_label path
    let snapshot = snapshot_with_full_state(
        &[],
        &[],
        false,
        ToolMode::Draw,
        EdgeMode::Line,
        localizer,
        Locale::fr_fr(),
    );
    let label = get_status_label(&snapshot);
    assert!(
        label.contains("Ligne"),
        "Expected localized French 'Ligne' for edge mode, got: {label}"
    );
}

#[rstest::rstest]
fn status_node_falls_back_to_default_locale_when_message_missing() {
    // Create partial French catalog (missing tool_mode.draw)
    let mut fr_messages = HashMap::new();
    fr_messages.insert("edge_mode.line".to_owned(), "Ligne".to_owned());
    let partial_fr_catalog = Catalog::from_messages(fr_messages);

    // Create default English catalog for fallback
    let en_catalog = Catalog::default_en_gb();

    // Build localizer with both catalogs
    let mut catalogs = HashMap::new();
    catalogs.insert(Locale::fr_fr(), partial_fr_catalog);
    catalogs.insert(Locale::en_gb(), en_catalog);
    let localizer = Localizer::with_catalogs(catalogs, Locale::en_gb());

    let label = get_status_label(&fr_locale_snapshot(localizer));
    assert!(
        label.contains("Draw"),
        "Expected fallback English 'Draw' for missing message, got: {label}"
    );
    assert!(
        label.contains("Ligne"),
        "Expected French 'Ligne' from partial catalog, got: {label}"
    );
}

#[rstest::rstest]
fn status_node_omits_edge_mode_for_manipulate_tool() {
    let mut messages = HashMap::new();
    messages.insert("tool_mode.manipulate".to_owned(), "Manipuler".to_owned());
    let mut catalogs = HashMap::new();
    catalogs.insert(Locale::fr_fr(), Catalog::from_messages(messages));
    let localizer = Localizer::with_catalogs(catalogs, Locale::en_gb());

    // Use snapshot_with_full_state to test Manipulate mode (no edge mode fragment)
    let snapshot = snapshot_with_full_state(
        &[],
        &[],
        false,
        ToolMode::Manipulate,
        EdgeMode::Line,
        localizer,
        Locale::fr_fr(),
    );
    let label = get_status_label(&snapshot);
    assert!(
        label.contains("Manipuler"),
        "Expected localized French 'Manipuler' for Manipulate mode, got: {label}"
    );
    assert!(
        !label.contains("Ligne") && !label.contains("Line"),
        "Manipulate mode should omit edge mode fragment from status, got: {label}"
    );
}
