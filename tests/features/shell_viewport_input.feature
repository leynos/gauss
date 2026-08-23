Feature: Shell viewport input

  Scenario: Scroll wheel pans the viewport
    Given a fresh Phase 0 shell window
    When the scroll wheel moves by 10 pixels right and 20 pixels up
    Then the viewport is panned 10 pixels right and 20 pixels up

  Scenario: Secondary scroll wheel zooms around the cursor
    Given a fresh Phase 0 shell window
    When the secondary-modified scroll wheel zooms at the canvas cursor
    Then the viewport zoom increases
    And the world point beneath the cursor is preserved
