Feature: Shape-drag history

  Scenario: Dragging a shape creates one undo entry and undo restores it
    Given a fresh Phase 0 shell window with a drawn shape in manipulate mode
    When the drawn shape is dragged
    Then the drawn shape moves by the drag delta
    And the drawn shape remains selected
    And the document history has gained 1 entry
    When the last document change is undone
    Then the drawn shape returns to its position before the drag
