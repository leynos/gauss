//! Behaviour tests for `A11yService` action-request routing.

#[path = "a11y_service_bdd/support.rs"]
mod a11y_service_bdd_support;

use accesskit::{Action, ActionRequest, NodeId, TreeId};
use gauss::ui::phase0_shell::{
    A11yActionRequestError, A11yRequestedAction, A11yWindowAction, accessibility,
};
use rstest_bdd_macros::{scenario, then, when};

use a11y_service_bdd_support::*;

const UNKNOWN_NODE_ID: u64 = 0xbeef;

fn route_request(world: &mut A11yWorld, request: &ActionRequest) -> TestSupportResult<()> {
    if world.service.update_records().is_empty() {
        world.last_publish_result = Some(world.service.sync_snapshot(world.snapshot.clone()));
        if let Some(Err(error)) = world.last_publish_result.as_ref() {
            return Err(TestSupportError::expectation(format!(
                "routing bootstrap sync failed: {error}"
            )));
        }
        capture_last_emitted_nodes(world);
    }
    match world.service.route_action_request(request) {
        Ok(action) => {
            world.last_routed_action = Some(action);
            world.last_route_error = None;
        }
        Err(error) => {
            world.last_routed_action = None;
            world.last_route_error = Some(error);
        }
    }
    Ok(())
}

fn route_action_for_close_node(world: &mut A11yWorld, action: Action) -> TestSupportResult<()> {
    route_request(
        world,
        &ActionRequest {
            action,
            target_tree: TreeId::ROOT,
            target_node: NodeId(accessibility::node_ids::CLOSE_BUTTON),
            data: None,
        },
    )
}

#[when("I route a click request for the close accessibility node")]
fn route_click_request_for_close_node(world: &mut A11yWorld) -> TestSupportResult<()> {
    route_action_for_close_node(world, Action::Click)
}

#[then("the request routes to the close window action")]
fn request_routes_to_close_window_action(world: &A11yWorld) -> TestSupportResult<()> {
    if world.last_routed_action != Some(A11yRequestedAction::Window(A11yWindowAction::CloseWindow))
    {
        return Err(TestSupportError::expectation(format!(
            "expected routed action {:?}, got {:?}",
            A11yRequestedAction::Window(A11yWindowAction::CloseWindow),
            world.last_routed_action
        )));
    }
    Ok(())
}

#[then("accessibility request routing succeeds")]
fn accessibility_request_routing_succeeds(world: &A11yWorld) -> TestSupportResult<()> {
    if let Some(error) = &world.last_route_error {
        return Err(TestSupportError::expectation(format!(
            "expected routing success, got error {error}"
        )));
    }
    if world.last_routed_action.is_none() {
        return Err(TestSupportError::missing(
            "routed accessibility action",
            "successful route assertion",
        ));
    }
    Ok(())
}

#[when("I route a focus request for the close accessibility node")]
fn route_focus_request_for_close_node(world: &mut A11yWorld) -> TestSupportResult<()> {
    route_action_for_close_node(world, Action::Focus)
}

#[when("I route a click request for an unknown accessibility node")]
fn route_click_request_for_an_unknown_accessibility_node(
    world: &mut A11yWorld,
) -> TestSupportResult<()> {
    route_request(
        world,
        &ActionRequest {
            action: Action::Click,
            target_tree: TreeId::ROOT,
            target_node: NodeId(UNKNOWN_NODE_ID),
            data: None,
        },
    )
}

fn assert_last_route_error(
    world: &A11yWorld,
    expected: &A11yActionRequestError,
    context: &str,
) -> TestSupportResult<()> {
    let Some(error) = &world.last_route_error else {
        return Err(TestSupportError::missing("routing error", context));
    };
    if error != expected {
        return Err(TestSupportError::expectation(format!(
            "expected {expected:?}, got {error:?}"
        )));
    }
    Ok(())
}

#[then("routing fails with an unsupported accessibility action error")]
fn routing_fails_with_unsupported_accessibility_action_error(
    world: &A11yWorld,
) -> TestSupportResult<()> {
    assert_last_route_error(
        world,
        &A11yActionRequestError::UnsupportedAction {
            target_node: accessibility::node_ids::CLOSE_BUTTON,
            action: Action::Focus,
        },
        "unsupported routing assertion",
    )
}

#[then("routing fails with an unknown accessibility node error")]
fn routing_fails_with_an_unknown_accessibility_node_error(
    world: &A11yWorld,
) -> TestSupportResult<()> {
    assert_last_route_error(
        world,
        &A11yActionRequestError::UnknownNode {
            target_node: UNKNOWN_NODE_ID,
        },
        "unknown-node routing assertion",
    )
}

#[scenario(
    path = "tests/features/a11y_service.feature",
    name = "Close button click request routes to the existing close action"
)]
fn close_button_click_request_routes_to_the_existing_close_action(world: A11yWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/a11y_service.feature",
    name = "Unsupported close button action request is rejected"
)]
fn unsupported_close_button_action_request_is_rejected(world: A11yWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/a11y_service.feature",
    name = "Unknown accessibility node request is rejected"
)]
fn unknown_accessibility_node_request_is_rejected(world: A11yWorld) {
    let _ = world;
}
