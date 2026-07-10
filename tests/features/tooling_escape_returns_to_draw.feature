Feature: Escape tool transitions

  Scenario: Escape in manipulate mode returns to draw mode
    Given a fresh Phase 0 shell window in manipulate mode
    When the canvas is clicked at the test point
    Then no new shape is created
    When Escape is pressed
    And the canvas is clicked at the test point
    Then one new shape is created

  Scenario: Escape cancels a manipulate drag preview without a history commit
    Given a fresh Phase 0 shell window with a two-anchor path in manipulate mode
    When a manipulate drag preview is started
    Then the drag preview is active without a history commit
    When Escape is pressed
    Then the drag preview is cancelled without history or geometry changes
