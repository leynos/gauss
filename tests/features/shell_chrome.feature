Feature: Shell chrome actions

  Scenario: Canvas input remains active beneath chrome
    Given a fresh Phase 0 shell window
    When the canvas is clicked
    Then the shell records the canvas click

  Scenario: Open button requests a path
    Given a fresh testable Phase 0 shell window
    When the Open button is clicked
    Then a new-path prompt is requested
    When the new-path prompt is cancelled

  Scenario: Save button requests a path
    Given a fresh Phase 0 shell window
    When the Save button is clicked
    Then a new-path prompt is requested
    When the new-path prompt is cancelled

  Scenario: Undo and redo buttons update the document
    Given a fresh Phase 0 shell window
    When two anchors are placed
    Then the draw shape anchor count is 2
    When the Undo button is clicked
    Then the draw shape anchor count is 1
    When the Redo button is clicked
    Then the draw shape anchor count is 2

  Scenario: Quit button requests quit
    Given a fresh Phase 0 shell window
    When the Quit button is clicked
    Then the shell requests quit
