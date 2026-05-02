Feature: Shell mode indicator

  The Phase 0 shell mode indicator displays the current tool mode
  and, for draw tools, the active edge mode.

  Scenario: Initial draw mode
    Given the Phase 0 shell is open
    Then the mode indicator reads "Mode: Draw (Line)"

  Scenario: Pressing Tab toggles the draw edge mode
    Given the Phase 0 shell is open
    When I press the "tab" key
    Then the mode indicator reads "Mode: Draw (Bezier (auto))"

  Scenario: Switching to manipulate mode hides the edge suffix
    Given the Phase 0 shell is open
    When I enter manipulate mode
    Then the mode indicator reads "Mode: Manipulate"
