//! Shared durable GPUI state for selection behavioural scenarios.
#![expect(
    dead_code,
    reason = "each selection integration test uses a subset of this shared support module"
)]

use std::cell::RefCell;

use gauss::model::{Document, SelItem, Selection, Shape, ShapeId, Vec2, Viewport};
use gauss::ui::Phase0Shell;
use gpui::{
    AnyWindowHandle, Entity, Pixels, Point, TestAppContext, VisualContext, VisualTestContext, px,
};
use rstest::fixture;
use rstest_bdd_macros::given;
use test_support::math;
use test_support::{TestSupportError, TestSupportResult};

use crate::common::{ensure_initial_draw, init_test_app};

#[derive(Default)]
pub struct ScenarioState {
    pub entity: Option<Entity<Phase0Shell>>,
    pub window: Option<AnyWindowHandle>,
    pub points: Vec<Point<Pixels>>,
    pub shape_ids: Vec<ShapeId>,
    pub shapes_before: Vec<Shape>,
    pub selection_before: Option<Selection>,
    pub history_before: Option<usize>,
    pub delta: Option<Vec2>,
    pub drag_started_after_press: Option<bool>,
}

thread_local! {
    static SCENARIO_STATE: RefCell<ScenarioState> = RefCell::new(ScenarioState::default());
}

pub fn with_state<R>(f: impl FnOnce(&mut ScenarioState) -> R) -> R {
    SCENARIO_STATE.with(|cell| f(&mut cell.borrow_mut()))
}

fn reset_state_after_scenario() {
    SCENARIO_STATE.with(|cell| *cell.borrow_mut() = ScenarioState::default());
}

fn reset_state_before_assignment() {
    reset_state_after_scenario();
}

pub struct ScenarioStateCleanup;

impl Drop for ScenarioStateCleanup {
    fn drop(&mut self) {
        reset_state_after_scenario();
    }
}

#[fixture]
pub fn scenario_state_cleanup() -> ScenarioStateCleanup {
    reset_state_before_assignment();
    ScenarioStateCleanup
}

pub fn with_visual_cx<R>(
    cx: &mut TestAppContext,
    f: impl FnOnce(&mut VisualTestContext, &Entity<Phase0Shell>) -> TestSupportResult<R>,
) -> TestSupportResult<R> {
    let handles = with_state(|state| (state.entity.clone(), state.window));
    let (Some(entity), Some(window)) = handles else {
        return Err(TestSupportError::missing(
            "scenario handles",
            "set by the fresh-window step",
        ));
    };
    let mut visual_cx = VisualTestContext::from_window(window, cx);
    f(&mut visual_cx, &entity)
}

pub fn require_point(index: usize, context: &str) -> TestSupportResult<Point<Pixels>> {
    with_state(|state| state.points.get(index).copied()).ok_or_else(|| {
        TestSupportError::missing(format!("scenario point {index}"), context.to_owned())
    })
}

pub fn require_shape_id(index: usize, context: &str) -> TestSupportResult<ShapeId> {
    with_state(|state| state.shape_ids.get(index).copied()).ok_or_else(|| {
        TestSupportError::missing(format!("scenario shape {index}"), context.to_owned())
    })
}

pub fn require_shape<'a>(
    document: &'a Document,
    id: ShapeId,
    context: &str,
) -> TestSupportResult<&'a Shape> {
    document
        .shape(id)
        .ok_or_else(|| TestSupportError::missing("shape", format!("shape {id:?}: {context}")))
}

pub fn shape_bbox_centre(shape: &Shape) -> TestSupportResult<Vec2> {
    let Some(first) = shape.path.anchors.first() else {
        return Err(TestSupportError::missing(
            "shape anchor",
            "computing bounding-box centre",
        ));
    };
    let (mut min_x, mut min_y) = (first.pos.x, first.pos.y);
    let (mut max_x, mut max_y) = (first.pos.x, first.pos.y);
    for anchor in shape.path.anchors.iter().skip(1) {
        min_x = min_x.min(anchor.pos.x);
        min_y = min_y.min(anchor.pos.y);
        max_x = max_x.max(anchor.pos.x);
        max_y = max_y.max(anchor.pos.y);
    }
    Ok(Vec2::new(
        math::midpoint(min_x, max_x),
        math::midpoint(min_y, max_y),
    ))
}

pub const fn viewport_to_screen_point(viewport: Viewport, world: Vec2) -> Point<Pixels> {
    let screen = viewport.world_to_screen(world);
    gpui::point(px(screen.x), px(screen.y))
}

pub fn require_selection_contains_shapes(
    selection: &Selection,
    expected: &[ShapeId],
    context: &str,
) -> TestSupportResult<()> {
    let has_expected_items = selection.items.len() == expected.len()
        && expected
            .iter()
            .all(|id| selection.contains(&SelItem::Shape(*id)));
    if !has_expected_items {
        return Err(TestSupportError::expectation(format!(
            "expected selected shapes {expected:?} ({context}); selection={selection:?}"
        )));
    }
    Ok(())
}

#[given("a fresh Phase 0 shell window")]
fn fresh_phase0_shell_window(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    reset_state_before_assignment();
    init_test_app(cx);
    let (entity, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);
    let window = visual_cx.window_handle();
    with_state(|state| {
        state.entity = Some(entity);
        state.window = Some(window);
    });
    with_visual_cx(cx, |_visual_cx, _entity| Ok(()))?;
    Ok(())
}
