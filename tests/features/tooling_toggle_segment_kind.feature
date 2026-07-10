Feature: Toggle selected segment kinds

  Scenario: Tab toggles the selected segment kind and undo restores it
    Given a fresh Phase 0 shell window with a selected line segment
    When Tab is pressed
    Then the selected segment is cubic with initial handles
    And one segment-toggle history entry is added
    When the last document change is undone
    Then the selected segment is a line without handles
