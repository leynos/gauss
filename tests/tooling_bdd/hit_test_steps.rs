//! Step definitions for shared hit-test service scenarios.

use crate::{common, state};
use gauss::model::{Document, SelItem, SelectPointerHit, ShapeId, Vec2};
use gauss::ui::GpuiActivatePenTool;
use gauss_core::test_helpers::square_shape_with_out_handle;
use gpui::{Modifiers, MouseButton, Pixels, Point, TestAppContext, point, px};
use rstest_bdd_macros::{given, then, when};
use test_support::TestSupportError;

enum HitTestState {
    Square {
        shape_id: ShapeId,
        min: Vec2,
        max: Vec2,
    },
    Overlap {
        top: ShapeId,
        centre: Vec2,
    },
}

fn screen_point(
    visual_cx: &gpui::VisualTestContext,
    view: &gpui::Entity<gauss::ui::Phase0Shell>,
    world: Vec2,
) -> Point<Pixels> {
    let viewport = visual_cx.read(|app| view.read(app).viewport());
    let screen = viewport.world_to_screen(world);
    point(px(screen.x), px(screen.y))
}

fn hover(visual_cx: &mut gpui::VisualTestContext, point: Point<Pixels>) {
    visual_cx.simulate_mouse_move(point, None, Modifiers::none());
    visual_cx.run_until_parked();
}

#[given("a fresh Phase 0 shell window in manipulate mode with a square handle")]
fn shell_with_square_handle(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::initialize(cx, |visual_cx, view| {
        let bounds = common::canvas_bounds(visual_cx)?;
        let origin = Vec2::new(f32::from(bounds.origin.x), f32::from(bounds.origin.y));
        let min = origin.add(Vec2::new(20.0, 20.0));
        let max = origin.add(Vec2::new(120.0, 120.0));
        let shape_id = ShapeId::from_accesskit_node_id(301);
        let mut document = Document::new();
        let _inserted = document.append_shape(square_shape_with_out_handle(
            shape_id,
            min,
            max,
            Vec2::new(6.0, 0.0),
        ));
        visual_cx.update(|_window, app| {
            view.update(app, |shell, view_cx| {
                shell.enter_manipulate_mode_for_tests();
                shell.replace_document_for_tests(document);
                view_cx.notify();
            });
        });
        visual_cx.run_until_parked();
        Ok(HitTestState::Square { shape_id, min, max })
    })
}

#[when("the square's outgoing handle is hovered")]
fn hover_outgoing_handle(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, view, data: &mut HitTestState| {
        let HitTestState::Square { min, .. } = data else {
            return Err(TestSupportError::missing(
                "square state",
                "handle-hover scenario",
            ));
        };
        let handle = screen_point(visual_cx, view, min.add(Vec2::new(6.0, 0.0)));
        hover(visual_cx, handle);
        Ok(())
    })
}

#[then("the hover hit identifies the square's first handle")]
fn hover_identifies_first_handle(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, view, data: &mut HitTestState| {
        let HitTestState::Square { shape_id, .. } = data else {
            return Err(TestSupportError::missing(
                "square state",
                "handle-hover scenario",
            ));
        };
        let hit = visual_cx.read(|app| view.read(app).hover_hit_for_tests());
        if !matches!(hit, SelectPointerHit::Handle(handle)
            if handle.shape_id == *shape_id && handle.anchor_index == 0)
        {
            return Err(TestSupportError::expectation(format!(
                "expected the first outgoing handle hit; got {hit:?}"
            )));
        }
        Ok(())
    })
}

#[when("the square's first segment is hovered")]
fn hover_first_segment(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, view, data: &mut HitTestState| {
        let HitTestState::Square { min, .. } = data else {
            return Err(TestSupportError::missing(
                "square state",
                "segment-hover scenario",
            ));
        };
        let segment = screen_point(visual_cx, view, min.add(Vec2::new(40.0, 0.0)));
        hover(visual_cx, segment);
        Ok(())
    })
}

#[then("the hover hit identifies a segment")]
fn hover_identifies_segment(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, view, _data: &mut HitTestState| {
        let hit = visual_cx.read(|app| view.read(app).hover_hit_for_tests());
        if !matches!(hit, SelectPointerHit::Segment(_)) {
            return Err(TestSupportError::expectation(format!(
                "expected a segment hit; got {hit:?}"
            )));
        }
        Ok(())
    })
}

#[when("the cursor moves to empty space")]
fn move_to_empty_space(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, view, data: &mut HitTestState| {
        let HitTestState::Square { max, .. } = data else {
            return Err(TestSupportError::missing(
                "square state",
                "empty-space scenario",
            ));
        };
        let empty = screen_point(visual_cx, view, max.add(Vec2::new(200.0, 200.0)));
        hover(visual_cx, empty);
        Ok(())
    })
}

#[then("the hover hit is clear")]
fn hover_hit_is_clear(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, view, _data: &mut HitTestState| {
        let hit = visual_cx.read(|app| view.read(app).hover_hit_for_tests());
        if hit != SelectPointerHit::None {
            return Err(TestSupportError::expectation(format!(
                "expected cleared hover hit; got {hit:?}"
            )));
        }
        Ok(())
    })
}

#[then("the hover hit is present")]
fn hover_hit_is_present(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, view, _data: &mut HitTestState| {
        let hit = visual_cx.read(|app| view.read(app).hover_hit_for_tests());
        if hit == SelectPointerHit::None {
            return Err(TestSupportError::expectation("expected a hover hit"));
        }
        Ok(())
    })
}

#[when("the pen tool is activated")]
fn activate_pen_tool(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, _view, _data: &mut HitTestState| {
        visual_cx.dispatch_action(GpuiActivatePenTool);
        visual_cx.run_until_parked();
        Ok(())
    })
}

#[given("a fresh Phase 0 shell window in manipulate mode with overlapping squares")]
fn shell_with_overlapping_squares(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::initialize(cx, |visual_cx, view| {
        let bounds = common::canvas_bounds(visual_cx)?;
        let origin = Vec2::new(f32::from(bounds.origin.x), f32::from(bounds.origin.y));
        let min = origin.add(Vec2::new(20.0, 20.0));
        let max = origin.add(Vec2::new(120.0, 120.0));
        let centre = Vec2::new(f32::midpoint(min.x, max.x), f32::midpoint(min.y, max.y));
        let bottom = ShapeId::from_accesskit_node_id(401);
        let top = ShapeId::from_accesskit_node_id(402);
        let mut document = Document::new();
        let _bottom = document.append_shape(square_shape_with_out_handle(
            bottom,
            min,
            max,
            Vec2::new(6.0, 0.0),
        ));
        let _top = document.append_shape(square_shape_with_out_handle(
            top,
            min,
            max,
            Vec2::new(6.0, 0.0),
        ));
        visual_cx.update(|_window, app| {
            view.update(app, |shell, view_cx| {
                shell.enter_manipulate_mode_for_tests();
                shell.replace_document_for_tests(document);
                shell.replace_selection_for_tests(gauss::model::Selection::empty());
                view_cx.notify();
            });
        });
        visual_cx.run_until_parked();
        Ok(HitTestState::Overlap { top, centre })
    })
}

#[when("the overlapping squares are clicked")]
fn click_overlapping_squares(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, view, data: &mut HitTestState| {
        let HitTestState::Overlap { centre, .. } = data else {
            return Err(TestSupportError::missing(
                "overlap state",
                "overlap scenario",
            ));
        };
        let centre_screen = screen_point(visual_cx, view, *centre);
        visual_cx.simulate_mouse_down(centre_screen, MouseButton::Left, Modifiers::none());
        visual_cx.run_until_parked();
        visual_cx.simulate_mouse_up(centre_screen, MouseButton::Left, Modifiers::none());
        visual_cx.run_until_parked();
        Ok(())
    })
}

#[then("only the topmost square is selected")]
fn only_topmost_square_is_selected(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, view, data: &mut HitTestState| {
        let HitTestState::Overlap { top, .. } = data else {
            return Err(TestSupportError::missing(
                "overlap state",
                "overlap scenario",
            ));
        };
        let selection = visual_cx.read(|app| view.read(app).selection().clone());
        if selection.items != vec![SelItem::Shape(*top)] {
            return Err(TestSupportError::expectation(format!(
                "expected only the topmost shape selected; got {selection:?}"
            )));
        }
        Ok(())
    })
}
