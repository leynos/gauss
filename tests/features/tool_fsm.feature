Feature: Tool mode FSM command emission

  Tool mode transitions are deterministic and emit commands rather than
  mutating state directly.

  Scenario: Escape from draw enters manipulate and clears active path
    Given the current tool mode is Draw
    And the current edge mode is Line
    And the input event is EscapePressed
    When the tool transition is evaluated
    Then it should emit SetToolMode Manipulate
    And it should emit SetActivePath None
    And it should emit exactly 2 commands

  Scenario: Escape from manipulate enters draw
    Given the current tool mode is Manipulate
    And the current edge mode is Line
    And the input event is EscapePressed
    When the tool transition is evaluated
    Then it should emit SetToolMode Draw
    And it should emit exactly 1 command

  Scenario: Toggle edge mode in draw emits edge command
    Given the current tool mode is Draw
    And the current edge mode is Line
    And the input event is ToggleEdgeMode
    When the tool transition is evaluated
    Then it should emit SetEdgeMode BezierAuto
    And it should emit exactly 1 command

  Scenario: Toggle edge mode in manipulate emits nothing
    Given the current tool mode is Manipulate
    And the current edge mode is Line
    And the input event is ToggleEdgeMode
    When the tool transition is evaluated
    Then it should emit no commands

  Scenario: Activate draw from manipulate with explicit edge mode
    Given the current tool mode is Manipulate
    And the current edge mode is Line
    And the input event is ActivateDrawBezier
    When the tool transition is evaluated
    Then it should emit SetToolMode Draw
    And it should emit SetEdgeMode BezierAuto
    And it should emit exactly 2 commands

  Scenario: Activate draw from manipulate without explicit edge mode
    Given the current tool mode is Manipulate
    And the current edge mode is Line
    And the input event is ActivateDraw
    When the tool transition is evaluated
    Then it should emit SetToolMode Draw
    And it should emit exactly 1 command

  Scenario: Activate draw in draw with explicit edge mode only changes edge
    Given the current tool mode is Draw
    And the current edge mode is Line
    And the input event is ActivateDrawBezier
    When the tool transition is evaluated
    Then it should emit SetEdgeMode BezierAuto
    And it should emit exactly 1 command

  Scenario: Close path committed from draw enters manipulate and clears active path
    Given the current tool mode is Draw
    And the current edge mode is BezierAuto
    And the input event is ClosePathCommitted
    When the tool transition is evaluated
    Then it should emit SetToolMode Manipulate
    And it should emit SetActivePath None
    And it should emit exactly 2 commands

  Scenario: Close path committed in manipulate emits nothing
    Given the current tool mode is Manipulate
    And the current edge mode is BezierAuto
    And the input event is ClosePathCommitted
    When the tool transition is evaluated
    Then it should emit no commands
