Feature: Shape reorder history

  Scenario: Lowering and raising shapes are undoable
    Given a fresh Phase 0 shell window with two overlapping drawn shapes
    When the overlap is clicked
    Then the topmost shape is selected
    When the selected shape is lowered
    Then the shape identifiers are unchanged
    And the selected shape is below the other drawn shape
    And the document history has gained 1 entry
    When the selected shape is raised
    Then the shape identifiers are unchanged
    And the selected shape is above the other drawn shape
    And the document history has gained 2 entries
    When the last document change is undone
    Then the selected shape is below the other drawn shape
    When the last document change is undone
    Then the selected shape is above the other drawn shape
