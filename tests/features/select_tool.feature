Feature: SelectTool command emission

  SelectTool transitions are deterministic and emit commands for selection and
  drag lifecycles in manipulate mode.

  Scenario: Pointer down on shape selects it and starts drag state
    Given the select tool mode is Manipulate
    And the select tool event is pointer down on shape without shift
    When the select tool transition is evaluated
    Then it emits a selection change record
    And it emits SetSelection for the hit shape
    And it emits SetSelectToolState Dragging

  Scenario: Shift pointer down toggles selection and stays idle
    Given the select tool mode is Manipulate
    And the select tool event is pointer down on shape with shift
    When the select tool transition is evaluated
    Then it emits SetSelectToolState Idle

  Scenario: Pointer move with dragging state emits preview command
    Given the select tool mode is Manipulate
    And the select tool has an active shape dragging state
    And the select tool event is pointer move with Dragging state and has_primary_button true
    When the select tool transition is evaluated
    Then it emits PreviewSelectDrag at world position 8 9

  Scenario: Pointer up with no movement restores preview and returns idle
    Given the select tool mode is Manipulate
    And the select tool has an active shape dragging state
    And the select tool event is pointer up at origin with Dragging state and is_primary_button true
    When the select tool transition is evaluated
    Then it emits RestoreSelectDragPreview
    And it emits exactly 2 select tool commands
    And it emits SetSelectToolState Idle

  Scenario: Pointer up with movement emits move command and returns idle
    Given the select tool mode is Manipulate
    And the select tool has an active shape dragging state
    And the select tool event is pointer up at moved with Dragging state and is_primary_button true
    When the select tool transition is evaluated
    Then it emits RestoreSelectDragPreview
    And it emits ApplyDocumentCommand MoveShapes
    And it emits SetSelectToolState Idle

  Scenario: Pointer move in draw mode emits nothing
    Given the select tool mode is Draw
    And the select tool event is pointer move with Idle state and has_primary_button true
    When the select tool transition is evaluated
    Then it emits no select tool commands

  Scenario: Pointer move in Marquee state emits nothing
    Given the select tool mode is Manipulate
    And the select tool event is pointer move with Marquee state and has_primary_button true
    When the select tool transition is evaluated
    Then it emits no select tool commands

  Scenario: Pointer move in Transforming state emits nothing
    Given the select tool mode is Manipulate
    And the select tool event is pointer move with Transforming state and has_primary_button true
    When the select tool transition is evaluated
    Then it emits no select tool commands

  Scenario: Pointer move with dragging state and released primary button emits nothing
    Given the select tool mode is Manipulate
    And the select tool has an active shape dragging state
    And the select tool event is pointer move with Dragging state and has_primary_button false
    When the select tool transition is evaluated
    Then it emits no select tool commands

  Scenario: Pointer up in Marquee state emits nothing
    Given the select tool mode is Manipulate
    And the select tool event is pointer up at moved with Marquee state and is_primary_button true
    When the select tool transition is evaluated
    Then it emits no select tool commands

  Scenario: Pointer up in Transforming state emits nothing
    Given the select tool mode is Manipulate
    And the select tool event is pointer up at moved with Transforming state and is_primary_button true
    When the select tool transition is evaluated
    Then it emits no select tool commands

  Scenario: Pointer up with dragging state and non-primary button emits nothing
    Given the select tool mode is Manipulate
    And the select tool has an active shape dragging state
    And the select tool event is pointer up at moved with Dragging state and is_primary_button false
    When the select tool transition is evaluated
    Then it emits no select tool commands

  Scenario: Pointer up with anchor drag emits move anchor command and returns idle
    Given the select tool mode is Manipulate
    And the select tool has an active anchor dragging state
    And the select tool event is pointer up at moved with Dragging state and is_primary_button true
    When the select tool transition is evaluated
    Then it emits RestoreSelectDragPreview
    And it emits ApplyDocumentCommand MoveAnchor
    And it emits SetSelectToolState Idle

  Scenario: Pointer up with handle drag emits move handle command and returns idle
    Given the select tool mode is Manipulate
    And the select tool has an active handle dragging state
    And the select tool event is pointer up at moved with Dragging state and is_primary_button true
    When the select tool transition is evaluated
    Then it emits RestoreSelectDragPreview
    And it emits ApplyDocumentCommand MoveHandle
    And it emits SetSelectToolState Idle
