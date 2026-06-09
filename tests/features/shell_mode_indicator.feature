Feature: Phase 0 shell mode indicator

  The Phase 0 UI shows a mode indicator line (for example, "Mode: Draw (Line)").
  It reflects the active tool mode and, while drawing, the active edge mode.

  Scenario: Mode indicator reflects tool and edge mode
    Given a Phase 0 shell window is open
    Then the mode indicator reads "Mode: Draw (Line)"
    When the user presses "tab"
    Then the mode indicator reads "Mode: Draw (Bezier (auto))"
    When the shell switches to manipulate mode
    Then the mode indicator reads "Mode: Manipulate"
