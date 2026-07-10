Feature: Shell tool rail

  Scenario: Select tool enters manipulate mode and clears active shape
    Given a fresh Phase 0 shell window with an active draw shape
    When the Select tool is clicked
    Then manipulate mode is active
    And no draw shape is active

  Scenario: Draw tools switch edge modes
    Given a fresh Phase 0 shell window
    When the Curve tool is clicked
    Then Bezier auto draw mode is active
    When the Line tool is clicked
    Then line draw mode is active
