Feature: Draw Bezier auto paths

  Scenario: Tab switches to Bezier auto and synthesises handles
    Given a fresh Phase 0 shell window
    When the first of four drawing anchors is placed
    And the draw edge mode is switched to Bezier auto
    And the remaining three drawing anchors are placed
    Then the path has 4 anchors and 3 cubic segments
    And the middle cubic segment handles match the Catmull-Rom controls
