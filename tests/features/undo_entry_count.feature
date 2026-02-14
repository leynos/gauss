Feature: Undo entry count

  Each command type produces exactly one undo entry when applied
  through DocumentUndoHistory, verifying the single-entry-per-gesture
  invariant.

  Scenario: MoveShapes command produces single undo entry
    Given an empty history and a document with one shape
    When I apply a MoveShapes command
    Then the history length should be 1

  Scenario: MoveAnchor command produces single undo entry
    Given an empty history and a document with one shape
    When I apply a MoveAnchor command
    Then the history length should be 1

  Scenario: MoveHandle command produces single undo entry
    Given an empty history and a document with one cubic shape
    When I apply a MoveHandle command
    Then the history length should be 1

  Scenario: InsertShape command produces single undo entry
    Given an empty history and an empty document
    When I apply an InsertShape command
    Then the history length should be 1

  Scenario: DeleteShapes command produces single undo entry
    Given an empty history and a document with one shape
    When I apply a DeleteShapes command
    Then the history length should be 1

  Scenario: ClosePath command produces single undo entry
    Given an empty history and a document with an open triangle
    When I apply a ClosePath command
    Then the history length should be 1

  Scenario: InsertAnchor command produces single undo entry
    Given an empty history and a document with one shape
    When I apply an InsertAnchor command
    Then the history length should be 1

  Scenario: SetSegmentKind command produces single undo entry
    Given an empty history and a document with one shape
    When I apply a SetSegmentKind command
    Then the history length should be 1

  Scenario: Reorder command produces single undo entry
    Given an empty history and a document with two shapes
    When I apply a Reorder command
    Then the history length should be 1

  Scenario: SetStyle command produces single undo entry
    Given an empty history and a document with one shape
    When I apply a SetStyle command
    Then the history length should be 1

  Scenario: Multiple sequential commands produce matching undo count
    Given an empty history and a document with one shape
    When I apply a MoveShapes command
    And I apply another MoveShapes command
    And I apply a SetStyle command
    Then the history length should be 3
