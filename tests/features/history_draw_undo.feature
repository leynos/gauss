Feature: Drawing-mode recovery

  Scenario: Activating the pen tool from manipulate mode allows drawing
    Given a fresh Phase 0 shell window in manipulate mode
    When the canvas is clicked in manipulate mode
    Then the document shape count is unchanged
    When the pen tool is activated
    And the canvas is clicked in draw mode
    Then the document shape count has gained 1 shape

  Scenario: Drawing recovers from a stale active path
    Given a fresh Phase 0 shell window with a stale active draw path
    When the canvas is clicked in draw mode
    Then a new open shape has 1 anchor and 0 segments
    And the active draw path tracks the new shape
    And no history error is present
