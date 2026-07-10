Feature: Multi-shape drag history

  Scenario: Dragging multiple shapes creates one undo entry and undo restores them
    Given a fresh Phase 0 shell window with two selected shapes
    When the selected shapes are dragged together
    Then both selected shapes move by the drag delta
    And the document history has gained 1 entry
    When the last document change is undone
    Then both selected shapes return to their positions before the drag
