Feature: Anchor-drag history

  Scenario: Dragging an anchor creates one undo entry and undo restores it
    Given a fresh Phase 0 shell window with a two-anchor line selected for editing
    When the first anchor is dragged
    Then only the first anchor moves by the drag delta
    And the document history has gained 1 entry
    When the last document change is undone
    Then both anchors return to their positions before the drag
