Feature: Save button file dialog

  Scenario: Clicking Save opens the save prompt
    Given a fresh Phase 0 shell window for file I/O
    Then no file path prompt is visible
    When the Save button is clicked
    Then a file path prompt is visible
