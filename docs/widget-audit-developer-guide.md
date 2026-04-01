# Widget capability audit developer guide

This guide provides concrete examples for working with the
[`ControlInventory`](https://github.com/leynos/gauss/blob/main/src/ui/widget_audit/mod.rs)
API when adding new controls, writing tests, or querying the inventory
programmatically.

## Overview

The widget capability audit is a typed inventory of all UI controls
required for Phase 1 and Phase 2 of the Gauss roadmap. It serves as the
canonical source of truth for:

- Which controls are required and when (phase)
- Where controls appear in the UI (surface)
- What user jobs they support
- Accessibility and keyboard requirements
- Implementation status and evidence

## Querying the inventory

### Creating an inventory instance

```rust
use gauss::ui::widget_audit::ControlInventory;

let inventory = ControlInventory::new();
```

### Listing all controls

```rust
let all_controls = inventory.all();
println!("Total controls: {}", all_controls.len());
```

### Querying by phase

```rust
use gauss::ui::widget_audit::Phase;

let phase1_controls = inventory.by_phase(Phase::Phase1);
let phase2_controls = inventory.by_phase(Phase::Phase2);
```

### Querying by surface

```rust
use gauss::ui::widget_audit::ControlSurface;

let toolbar_tools = inventory.by_surface(ControlSurface::Toolbar);
let style_controls = inventory.by_surface(ControlSurface::StylePanel);
```

Available surfaces:

- `ControlSurface::Toolbar` — Tool selection rail
- `ControlSurface::PropertiesPanel` — Transform properties (x, y, width, height,
  rotation)
- `ControlSurface::StylePanel` — Stroke and fill styling
- `ControlSurface::LayersPanel` — Layer management
- `ControlSurface::HistoryPanel` — Undo/redo history
- `ControlSurface::AlignmentPanel` — Alignment and distribution
- `ControlSurface::CharacterPanel` — Text character formatting (Phase 2)
- `ControlSurface::ParagraphPanel` — Text paragraph formatting (Phase 2)
- `ControlSurface::CanvasTextEditor` — On-canvas text editing (Phase 2)
- `ControlSurface::Popover` — Contextual popovers

### Filtering by evidence status

```rust
// Controls with current shell evidence
let implemented = inventory.with_evidence();

// Controls without evidence (planned but not implemented)
let planned = inventory.without_evidence();
```

## Accessing control metadata

Each
[`RequiredControl`](https://github.com/leynos/gauss/blob/main/src/ui/widget_audit/types.rs)
provides comprehensive metadata:

```rust
use gauss::ui::widget_audit::RequiredControl;

fn analyse_control(control: &RequiredControl) {
    // Basic info
    println!("Name: {}", control.name());
    println!("Phase: {:?}", control.phase());
    println!("Surface: {}", control.surface());

    // User job
    println!("Purpose: {}", control.user_job.description);

    // Supported states
    println!("States: {:?}", control.states.states);

    // Keyboard requirements
    println!("Keyboard only: {}", control.keyboard.keyboard_only_operation);
    if let Some(ref shortcut) = control.keyboard.shortcut {
        println!("Shortcut: {}", shortcut);
    }

    // Accessibility
    println!("ARIA role: {}", control.accessibility.role);
    println!("Label: {}", control.accessibility.label);
    println!("States: {:?}", control.accessibility.states);

    // Action linkage
    if control.action_linkage.requires_action {
        println!("Action: {:?}", control.action_linkage.action_name);
        println!("Notes: {}", control.action_linkage.notes);
    }

    // Requirement sources
    for source in &control.sources {
        println!("Source: {}", source);
    }

    // Current evidence
    println!("Implemented: {}", control.current_evidence.exists);
    if let Some(ref path) = control.current_evidence.file_path {
        println!("Evidence path: {}", path);
    }
    println!("Evidence notes: {}", control.current_evidence.notes);
}
```

## Adding new controls

To add a new control to the inventory:

1. Identify the appropriate module in `src/ui/widget_audit/`
2. Add a new function that returns `RequiredControl`
3. Include it in the surface's `controls()` function
4. Update tests to verify the new control

### Example: adding a new toolbar tool

```rust
// In src/ui/widget_audit/toolbar.rs

use gauss_core::model::Action;

fn my_new_tool() -> RequiredControl {
    RequiredControl {
        name: "My New Tool",
        phase: Phase::Phase1,
        surface: ControlSurface::Toolbar,
        user_job: UserJob {
            description: "Description of what this tool does",
        },
        states: ControlStates {
            states: vec!["enabled", "disabled", "active"],
        },
        keyboard: KeyboardRequirements {
            shortcut: Some("cmd-shift-n"),
            keyboard_only_operation: true,
            notes: "Must be accessible via keyboard",
        },
        accessibility: AccessibilityRequirements {
            role: "Button",
            label: "My New Tool",
            states: vec!["focusable", "pressed"],
            notes: "Must announce active state",
        },
        action_linkage: ActionCommandLinkage {
            requires_action: true,
            action_name: Some(Action::ActivateMyNewTool.identifier()),
            notes: "Activates the new tool mode",
        },
        sources: vec![
            RequirementSource::Roadmap("1.5.1"),
            RequirementSource::FeaturePlan("Phase 1: New Features"),
        ],
        current_evidence: CurrentShellEvidence {
            exists: false,
            file_path: None,
            notes: "Planned for next sprint",
        },
    }
}
```

Then add it to the `controls()` function:

```rust
pub(super) fn controls() -> Vec<RequiredControl> {
    vec![
        selection_tool(),
        // ... existing tools
        my_new_tool(), // Add here
    ]
}
```

## Writing tests

### Basic test pattern

```rust
use gauss::ui::widget_audit::{ControlInventory, ControlSurface, Phase};

#[test]
fn test_my_surface_has_controls() {
    let inventory = ControlInventory::new();
    let controls = inventory.by_surface(ControlSurface::MySurface);

    assert!(!controls.is_empty(), "MySurface must have controls");
}
```

### Testing specific controls

```rust
#[test]
fn test_toolbar_has_specific_tools() {
    let inventory = ControlInventory::new();
    let toolbar = inventory.by_surface(ControlSurface::Toolbar);

    let has_pen = toolbar.iter().any(|c| c.name() == "Pen Tool");
    assert!(has_pen, "Toolbar must have Pen Tool");
}
```

### Testing phase categorization

```rust
#[rstest]
#[case(ControlSurface::CharacterPanel)]
#[case(ControlSurface::ParagraphPanel)]
#[case(ControlSurface::CanvasTextEditor)]
fn test_phase2_surfaces(#[case] surface: ControlSurface) {
    let inventory = ControlInventory::new();
    let controls = inventory.by_surface(surface);

    let all_phase2 = controls.iter().all(|c| c.phase() == Phase::Phase2);
    assert!(all_phase2, "All {surface} controls should be Phase 2");
}
```

## Common patterns

### Checking implementation status

```rust
fn check_implementation_status(inventory: &ControlInventory) {
    let with_evidence = inventory.with_evidence();
    let without_evidence = inventory.without_evidence();

    println!("Implemented: {}", with_evidence.len());
    println!("Planned: {}", without_evidence.len());

    for control in without_evidence {
        println!("  - {}: {}", control.name(), control.current_evidence.notes);
    }
}
```

### Validating requirements

```rust
fn validate_requirements(inventory: &ControlInventory) -> Vec<String> {
    let mut errors = Vec::new();

    for control in inventory.all() {
        // Check for required fields
        if control.name().is_empty() {
            errors.push(format!("Control has no name"));
        }
        if control.user_job.description.is_empty() {
            errors.push(format!("'{}' has no description", control.name()));
        }
        if control.sources.is_empty() {
            errors.push(format!("'{}' has no requirement sources", control.name()));
        }

        // Check keyboard accessibility
        if !control.keyboard.keyboard_only_operation {
            errors.push(format!("'{}' must support keyboard operation", control.name()));
        }

        // Check accessibility requirements
        if control.accessibility.role.is_empty() {
            errors.push(format!("'{}' has no ARIA role", control.name()));
        }
    }

    errors
}
```

## Best practices

1. **Always use the inventory for control metadata** — Do not hardcode control
   names or requirements in tests; query the inventory instead.

2. **Update evidence when implementing controls** — When a control is
   implemented, set `current_evidence.exists = true` and provide the file path.

3. **Document requirement sources** — Every control must cite at least one
   roadmap item, feature plan, or architecture decision.

4. **Test both positive and negative cases** — Verify that expected controls
   exist and that unexpected controls do not.

5. **Use parameterized tests** — When testing multiple similar surfaces or
   phases, use `#[rstest]` with `#[case]` parameters.

## Related documentation

- [`widget-capability-audit.md`](widget-capability-audit.md) — High-level
  inventory documentation
- [`gauss-architecture-design.md`](gauss-architecture-design.md) — System
  architecture and design decisions
- [`roadmap.md`](roadmap.md) — Roadmap items referenced by requirement sources
