Feature: Bezier-handle drag history

  Scenario: Dragging a handle creates one undo entry and undo restores it
    Given a fresh Phase 0 shell window with a selected Bezier handle
    When the selected Bezier handle is dragged
    Then the anchor stays fixed and its outgoing handle moves by the drag delta
    And the document history has gained 1 entry
    When the last document change is undone
    Then the anchor and outgoing handle return to their positions before the drag
