Feature: PenTool canvas click transitions

  PenTool emits deterministic command sequences for draw-mode canvas clicks.

  Scenario: Click with no active path starts a new open shape
    Given the pen tool mode is Draw
    And the pen edge mode is Line
    And no pen active path exists
    And the pen click is at (20, 30)
    When the pen transition is evaluated
    Then the pen transition should emit InsertShape
    And the pen transition should emit SetActivePath Some
    And the pen transition should emit exactly 2 commands

  Scenario: Click with active open path appends anchor
    Given the pen tool mode is Draw
    And the pen edge mode is Line
    And a pen active open path with 2 anchors exists
    And the pen click is at (60, 24)
    When the pen transition is evaluated
    Then the pen transition should emit InsertAnchor
    And the pen transition should emit exactly 1 command

  Scenario: Click near first anchor closes path and exits draw mode
    Given the pen tool mode is Draw
    And the pen edge mode is BezierAuto
    And a pen active open path with 3 anchors exists
    And the pen click is at (4, 3)
    When the pen transition is evaluated
    Then the pen transition should emit ClosePath
    And the pen transition should emit SetToolMode Manipulate
    And the pen transition should emit SetActivePath None
    And the pen transition should emit exactly 3 commands

  Scenario: Stale active path is cleared then new shape starts
    Given the pen tool mode is Draw
    And the pen edge mode is Line
    And a stale pen active path exists
    And the pen click is at (8, 9)
    When the pen transition is evaluated
    Then the pen transition should emit SetActivePath None
    And the pen transition should emit InsertShape
    And the pen transition should emit SetActivePath Some
    And the pen transition should emit exactly 3 commands

  Scenario: Click while in manipulate mode is ignored
    Given the pen tool mode is Manipulate
    And the pen edge mode is Line
    And no pen active path exists
    And the pen click is at (10, 10)
    When the pen transition is evaluated
    Then the pen transition should emit no commands

  Scenario: Close attempt with two anchors appends instead of closing
    Given the pen tool mode is Draw
    And the pen edge mode is Line
    And a pen active open path with 2 anchors exists
    And the pen click is at (1, 1)
    When the pen transition is evaluated
    Then the pen transition should emit InsertAnchor
    And the pen transition should emit exactly 1 command
