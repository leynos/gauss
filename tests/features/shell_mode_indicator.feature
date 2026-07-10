Feature: Shell mode indicator

  Scenario: Mode indicator follows tool and edge mode
    Given a fresh Phase 0 shell window
    Then the mode indicator reads "Mode: Draw (Line)"
    When the edge mode is cycled with Tab
    Then the mode indicator reads "Mode: Draw (Bezier (auto))"
    When manipulate mode is entered
    Then the mode indicator reads "Mode: Manipulate"
