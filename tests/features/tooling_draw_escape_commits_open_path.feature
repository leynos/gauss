Feature: Commit open paths with Escape

  Scenario: Escape commits an open path and enters manipulate mode
    Given a fresh Phase 0 shell window
    When two distinct drawing anchors are placed
    Then the active path is open with two distinct anchors
    When Escape is pressed
    And the canvas is clicked at the second anchor
    Then the same open path remains with unchanged anchor and segment counts
