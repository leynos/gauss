Feature: Draw-mode undo and redo

  Scenario: Draw clicks add anchors and undo removes them
    Given a fresh Phase 0 shell window
    When the first anchor is placed
    Then the draw shape anchor count is 1
    When the second anchor is placed
    Then the draw shape anchor count is 2
    When the last change is undone
    Then the draw shape anchor count is 1
    When the last change is undone
    Then the draw shape is absent
    When the last change is redone
    Then the draw shape anchor count is 1
    When the last change is redone
    Then the draw shape anchor count is 2
