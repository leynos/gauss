//! i18n-specific tests for accessibility tree status labels.

use std::collections::{BTreeSet, HashMap};

use rstest::{fixture, rstest};

use crate::i18n::{Catalog, Locale, Localizer};
use crate::model::{EdgeMode, ToolMode};
use crate::ui::phase0_shell::a11y_service::{
    A11yShapeSnapshot, A11ySnapshot,
    tree_builder::{STATUS_NODE_ID, build_node_map},
};

// Import shape_id from parent module
use super::shape_id;

/// Creates a French tool mode catalog.
#[fixture]
fn fr_tool_mode_catalog() -> Catalog {
    let mut messages = HashMap::new();
    messages.insert("tool_mode.draw".to_owned(), "Dessiner".to_owned());
    Catalog::from_messages(messages)
}

/// Creates a French edge mode catalog.
#[fixture]
fn fr_edge_mode_catalog() -> Catalog {
    let mut messages = HashMap::new();
    messages.insert("edge_mode.line".to_owned(), "Ligne".to_owned());
    Catalog::from_messages(messages)
}

/// Creates a French manipulate tool mode catalog.
#[fixture]
fn fr_manipulate_catalog() -> Catalog {
    let mut messages = HashMap::new();
    messages.insert("tool_mode.manipulate".to_owned(), "Manipuler".to_owned());
    Catalog::from_messages(messages)
}

/// Creates a localizer with a French catalog and en-GB default.
#[fixture]
fn fr_localizer(fr_tool_mode_catalog: Catalog) -> Localizer {
    let mut catalogs = HashMap::new();
    catalogs.insert(Locale::fr_fr(), fr_tool_mode_catalog);
    catalogs.insert(Locale::en_gb(), Catalog::default_en_gb());
    Localizer::with_catalogs(catalogs, Locale::en_gb())
}

/// Creates a localizer with partial French catalog and full en-GB fallback.
#[fixture]
fn fr_localizer_with_fallback() -> Localizer {
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
    Localizer::with_catalogs(catalogs, Locale::en_gb())
}

/// Parameter object for localised mode state in test snapshots.
struct LocalisedModeState {
    tool_mode: ToolMode,
    edge_mode: EdgeMode,
    localizer: Localizer,
    locale: Locale,
}

fn snapshot_with_full_state(
    shape_ids: &[u64],
    selected_ids: &[u64],
    is_maximized: bool,
    mode: LocalisedModeState,
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
        tool_mode: mode.tool_mode,
        edge_mode: mode.edge_mode,
        can_undo: false,
        can_redo: false,
        is_maximized,
        selected_shape_ids,
        shapes,
        localizer: mode.localizer,
        locale: mode.locale,
    }
}

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

fn get_status_label(snapshot: &A11ySnapshot) -> String {
    let (nodes, _) = build_node_map(snapshot).expect("node map build should succeed");
    let status_node = nodes
        .get(&STATUS_NODE_ID)
        .expect("expected node map to contain status node");
    status_node
        .label()
        .expect("status node should have a label")
        .to_owned()
}

#[rstest]
#[case::tool_mode_draw("Dessiner")]
fn status_node_uses_localized_tool_mode_label(fr_localizer: Localizer, #[case] expected: &str) {
    let label = get_status_label(&fr_locale_snapshot(fr_localizer));
    assert!(
        label.contains(expected),
        "Expected localized French '{expected}', got: {label}"
    );
}

#[rstest]
fn status_node_uses_localized_edge_mode_label(fr_edge_mode_catalog: Catalog) {
    let mut catalogs = HashMap::new();
    catalogs.insert(Locale::fr_fr(), fr_edge_mode_catalog);
    catalogs.insert(Locale::en_gb(), Catalog::default_en_gb());
    let localizer = Localizer::with_catalogs(catalogs, Locale::en_gb());

    // Use snapshot_with_full_state to exercise localized_edge_mode_label path
    let snapshot = snapshot_with_full_state(
        &[],
        &[],
        false,
        LocalisedModeState {
            tool_mode: ToolMode::Draw,
            edge_mode: EdgeMode::Line,
            localizer,
            locale: Locale::fr_fr(),
        },
    );
    let label = get_status_label(&snapshot);
    assert!(
        label.contains("Ligne"),
        "Expected localized French 'Ligne' for edge mode, got: {label}"
    );
}

#[rstest]
fn status_node_falls_back_to_default_locale_when_message_missing(
    fr_localizer_with_fallback: Localizer,
) {
    let label = get_status_label(&fr_locale_snapshot(fr_localizer_with_fallback));
    assert!(
        label.contains("Draw"),
        "Expected fallback English 'Draw' for missing message, got: {label}"
    );
    assert!(
        label.contains("Ligne"),
        "Expected French 'Ligne' from partial catalog, got: {label}"
    );
}

#[rstest]
fn status_node_omits_edge_mode_for_manipulate_tool(fr_manipulate_catalog: Catalog) {
    let mut catalogs = HashMap::new();
    catalogs.insert(Locale::fr_fr(), fr_manipulate_catalog);
    catalogs.insert(Locale::en_gb(), Catalog::default_en_gb());
    let localizer = Localizer::with_catalogs(catalogs, Locale::en_gb());

    // Use snapshot_with_full_state to test Manipulate mode (no edge mode fragment)
    let snapshot = snapshot_with_full_state(
        &[],
        &[],
        false,
        LocalisedModeState {
            tool_mode: ToolMode::Manipulate,
            edge_mode: EdgeMode::Line,
            localizer,
            locale: Locale::fr_fr(),
        },
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
