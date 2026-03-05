//! GPUI integration coverage for the accessibility tree service wiring.

mod common;

use accesskit::{Action, Role, TreeUpdate};
use common::{ensure_initial_draw, init_test_app};
use gauss::model::{Selection, ShapeId, Vec2};
use gauss::ui::Phase0Shell;
use gauss::ui::phase0_shell::{A11yUpdateKind, accessibility};
use gpui::TestAppContext;

fn setup_window(
    cx: &mut TestAppContext,
) -> (&mut gpui::VisualTestContext, gpui::Entity<Phase0Shell>) {
    init_test_app(cx);
    let (view, visual_cx) =
        cx.add_window_view(|_window, view_cx| Phase0Shell::new_for_tests(view_cx));
    ensure_initial_draw(visual_cx);
    (visual_cx, view)
}

fn assert_chrome_button_semantics(
    update: &TreeUpdate,
    expected: &accessibility::ChromeButtonSemantics,
) {
    let Some(node) = accessibility::chrome_node_from_update(update, expected.node_id) else {
        panic!(
            "expected node {:#x} in drained accessibility update",
            expected.node_id
        );
    };
    assert_eq!(node.role(), Role::Button);
    assert_eq!(node.label(), Some(expected.label));
    assert_eq!(node.description(), Some(expected.shortcut_hint));
    assert_eq!(node.keyboard_shortcut(), Some(expected.shortcut_hint));
    assert!(node.supports_action(Action::Click));
}

fn assert_initial_serialised_update(initial_update: &TreeUpdate) {
    assert!(
        initial_update.tree.is_some(),
        "expected initial update to include tree metadata"
    );
    let Some(titlebar) =
        accessibility::chrome_node_from_update(initial_update, accessibility::node_ids::TITLEBAR)
    else {
        panic!(
            "expected node {:#x} in drained accessibility update",
            accessibility::node_ids::TITLEBAR
        );
    };
    assert_eq!(titlebar.role(), Role::TitleBar);
    assert_eq!(
        titlebar.label(),
        Some(accessibility::accessible_names::TITLEBAR)
    );
    for expected in accessibility::chrome_button_semantics(false) {
        assert_chrome_button_semantics(initial_update, &expected);
    }
}

#[gpui::test]
fn a11y_initial_tree_update_is_emitted_on_first_draw(cx: &mut TestAppContext) {
    let (visual_cx, view) = setup_window(cx);
    let records = visual_cx.read(|app| view.read(app).a11y_update_records_for_tests().to_vec());
    let drained_updates = visual_cx.update(|_window, app| {
        view.update(app, |shell, _cx| shell.drain_a11y_tree_updates_for_tests())
    });
    assert_eq!(
        records.len(),
        1,
        "expected one initial accessibility update"
    );
    let first = records
        .first()
        .expect("expected one initial accessibility update");
    assert_eq!(first.kind, A11yUpdateKind::InitialTree);
    assert!(
        first
            .inserted_node_ids
            .contains(&accessibility::node_ids::TITLEBAR),
        "expected titlebar node to be included in inserted IDs"
    );

    assert_eq!(
        drained_updates.len(),
        1,
        "expected one initial serialized tree update"
    );
    let initial_update = drained_updates
        .first()
        .expect("expected one initial serialized tree update");
    assert_initial_serialised_update(initial_update);
}

#[gpui::test]
fn a11y_shape_insert_emits_incremental_update(cx: &mut TestAppContext) {
    let (visual_cx, view) = setup_window(cx);

    visual_cx.update(|_window, app| {
        view.update(app, |shell, _cx| {
            shell.clear_a11y_updates_for_tests();
        });
    });

    let inserted_shape_id = visual_cx.update(|_window, app| {
        view.update(app, |shell, view_cx| {
            let shape_id = common::add_square(
                shell.document_mut_for_tests(),
                Vec2::new(10.0, 10.0),
                Vec2::new(40.0, 40.0),
            )
            .expect("adding test shape should succeed");
            view_cx.notify();
            shape_id.to_accesskit_node_id()
        })
    });
    ensure_initial_draw(visual_cx);

    let records = visual_cx.read(|app| view.read(app).a11y_update_records_for_tests().to_vec());
    assert_eq!(records.len(), 1, "expected one incremental update");
    let record = records
        .first()
        .expect("expected one incremental accessibility update");
    assert_eq!(record.kind, A11yUpdateKind::Incremental);
    assert!(
        record.inserted_node_ids.contains(&inserted_shape_id),
        "expected inserted node IDs {:?} to contain inserted shape id {inserted_shape_id:#x}",
        record.inserted_node_ids
    );
}

#[gpui::test]
fn a11y_idle_render_without_state_change_emits_no_update(cx: &mut TestAppContext) {
    let (visual_cx, view) = setup_window(cx);
    visual_cx.update(|_window, app| {
        view.update(app, |shell, _cx| {
            shell.clear_a11y_updates_for_tests();
        });
    });
    ensure_initial_draw(visual_cx);
    let update_count = visual_cx.read(|app| view.read(app).a11y_pending_update_count_for_tests());
    assert_eq!(update_count, 0, "expected no update for unchanged state");
}

#[gpui::test]
fn a11y_stale_shape_selection_is_ignored_without_crashing(cx: &mut TestAppContext) {
    let (visual_cx, view) = setup_window(cx);
    visual_cx.update(|_window, app| {
        view.update(app, |shell, view_cx| {
            shell.clear_a11y_updates_for_tests();
            let stale_id = ShapeId::from_accesskit_node_id(0x2_ffff_ffff);
            shell.replace_selection_for_tests(Selection {
                items: vec![gauss::model::SelItem::Shape(stale_id)],
            });
            view_cx.notify();
        });
    });
    ensure_initial_draw(visual_cx);
    let update_count = visual_cx.read(|app| view.read(app).a11y_pending_update_count_for_tests());
    assert_eq!(
        update_count, 0,
        "expected stale selection to be ignored and emit no update"
    );
}
