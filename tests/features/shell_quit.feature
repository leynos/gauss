Feature: Shell quit button

  Scenario: Quit button requests quit
    Given a fresh Phase 0 shell window
    When the Quit button is clicked
    Then the shell requests quit
