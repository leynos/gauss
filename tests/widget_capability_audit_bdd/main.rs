//! BDD tests for widget capability audit inventory.
//!
//! This test module uses rstest-bdd v0.5.0 to validate that the Phase 1-2
//! control inventory satisfies roadmap requirements expressed in Gherkin
//! scenarios.

mod steps;

use gauss::ui::widget_audit::{ControlInventory, ControlSurface, Phase, RequiredControl};
use rstest::fixture;
use rstest_bdd_macros::{scenario, when};
use test_support::{TestSupportError, TestSupportResult};

/// World state for widget audit BDD tests.
#[derive(Default)]
pub(crate) struct AuditWorld {
    pub(crate) inventory: Option<ControlInventory>,
    pub(crate) controls: Vec<String>,
    pub(crate) count: usize,
}

#[fixture]
fn world() -> AuditWorld {
    AuditWorld::default()
}

#[scenario("tests/features/widget_capability_audit.feature")]
fn widget_capability_audit(world: AuditWorld) {
    let _ = world;
}

// === Helper functions ===

pub(crate) fn query_by_phase_and_surface(
    world: &mut AuditWorld,
    phase: Phase,
    surface: ControlSurface,
) {
    if let Some(ref inventory) = world.inventory {
        world.controls = inventory
            .by_surface(surface)
            .iter()
            .filter(|c| c.phase() == phase)
            .map(|c| c.name().to_owned())
            .collect();
    }
}

pub(crate) fn assert_includes(world: &AuditWorld, expected: &str) -> TestSupportResult<()> {
    if !world.controls.iter().any(|n| n == expected) {
        return Err(TestSupportError::expectation(format!(
            "Inventory must include {expected}"
        )));
    }
    Ok(())
}

pub(crate) fn assert_includes_substrings(
    world: &AuditWorld,
    checks: &[(&str, &str)],
) -> TestSupportResult<()> {
    for (sub, msg) in checks {
        if !world.controls.iter().any(|n| n.contains(sub)) {
            return Err(TestSupportError::expectation(*msg));
        }
    }
    Ok(())
}

pub(crate) fn for_each_control<F>(world: &AuditWorld, check: F) -> TestSupportResult<()>
where
    F: Fn(&RequiredControl) -> TestSupportResult<()>,
{
    let inventory = world.inventory.as_ref().ok_or_else(|| {
        TestSupportError::expectation("inventory not loaded in AuditWorld".to_owned())
    })?;
    for control in inventory.all() {
        check(control)?;
    }
    Ok(())
}

pub(crate) fn assert_each_control<P, M>(
    world: &AuditWorld,
    predicate: P,
    message: M,
) -> TestSupportResult<()>
where
    P: Fn(&RequiredControl) -> bool,
    M: Fn(&RequiredControl) -> String,
{
    for_each_control(world, |c| {
        if predicate(c) {
            Ok(())
        } else {
            Err(TestSupportError::expectation(message(c)))
        }
    })
}

// === Given steps ===

#[rstest_bdd_macros::given("the widget capability audit inventory is loaded")]
pub(crate) fn given_inventory_loaded(world: &mut AuditWorld) {
    world.inventory = Some(ControlInventory::new());
}

// === When steps ===

#[when("I query controls for Phase 1 toolbar")]
pub(crate) fn when_query_phase1_toolbar(world: &mut AuditWorld) {
    query_by_phase_and_surface(world, Phase::Phase1, ControlSurface::Toolbar);
}

#[when("I query controls for Phase 1 properties panel")]
pub(crate) fn when_query_phase1_properties(world: &mut AuditWorld) {
    query_by_phase_and_surface(world, Phase::Phase1, ControlSurface::PropertiesPanel);
}

#[when("I query controls for Phase 1 alignment panel")]
pub(crate) fn when_query_phase1_alignment(world: &mut AuditWorld) {
    query_by_phase_and_surface(world, Phase::Phase1, ControlSurface::AlignmentPanel);
}

#[when("I query controls for Phase 1 style panel")]
pub(crate) fn when_query_phase1_style(world: &mut AuditWorld) {
    query_by_phase_and_surface(world, Phase::Phase1, ControlSurface::StylePanel);
}

#[when("I query controls for Phase 1 layers panel")]
pub(crate) fn when_query_phase1_layers(world: &mut AuditWorld) {
    query_by_phase_and_surface(world, Phase::Phase1, ControlSurface::LayersPanel);
}

#[when("I query controls for Phase 2")]
pub(crate) fn when_query_phase2(world: &mut AuditWorld) {
    if let Some(ref inventory) = world.inventory {
        world.controls = inventory
            .by_phase(Phase::Phase2)
            .iter()
            .map(|c| c.name().to_owned())
            .collect();
    }
}

#[when("I examine all controls in the inventory")]
pub(crate) fn when_examine_all_controls(world: &mut AuditWorld) {
    if let Some(ref inventory) = world.inventory {
        world.count = inventory.all().len();
    }
}

#[when("I query controls with current shell evidence")]
pub(crate) fn when_query_with_evidence(world: &mut AuditWorld) {
    if let Some(ref inventory) = world.inventory {
        world.controls = inventory
            .with_evidence()
            .iter()
            .map(|c| c.name().to_owned())
            .collect();
    }
}

#[when("I query Phase 1 toolbar controls")]
pub(crate) fn when_query_toolbar_phase1(world: &mut AuditWorld) {
    when_query_phase1_toolbar(world);
}
