Feature: Selection history

  Scenario: Selection undo and redo do not change the document
    Given a fresh Phase 0 shell window with a drawn path in manipulate mode
    When the first anchor is selected
    Then the selection is not empty
    When the selection is cleared
    Then the selection is empty
    When the last selection change is undone
    Then the previous selection is restored
    When the last selection change is redone
    Then the selection is empty
    And the document shape count is unchanged
