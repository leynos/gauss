//! GPUI integration coverage for the accessibility tree service wiring.

mod common;

use common::{ensure_initial_draw, init_test_app};
use gauss::model::{Selection, ShapeId, Vec2};
use gauss::ui::Phase0Shell;
use gauss::ui::phase0_shell::A11yUpdateKind;
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

#[gpui::test]
fn a11y_initial_tree_update_is_emitted_on_first_draw(cx: &mut TestAppContext) {
    let (visual_cx, view) = setup_window(cx);
    let records = visual_cx.read(|app| view.read(app).a11y_update_records_for_tests().to_vec());
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
        first.inserted_node_ids.contains(&0x1006),
        "expected titlebar node to be included in inserted IDs"
    );
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
