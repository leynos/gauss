Feature: Close drawn paths

  Scenario: Clicking near the first anchor closes the path and enters manipulate mode
    Given a fresh Phase 0 shell window
    When three triangle anchors are placed
    Then the triangle path is open
    When the first triangle anchor is clicked again
    Then the triangle path is closed
    And the closing operation preserves the drawn shape
    When the canvas is clicked away from the closed path
    Then no point is added to the closed path

  Scenario: Closing in Bezier mode uses a cubic closing segment
    Given a fresh Phase 0 shell window
    When the first triangle anchor is placed
    And the draw edge mode is switched to Bezier auto
    And the remaining triangle anchors are placed
    And the first triangle anchor is clicked again
    Then the triangle path is closed with a cubic closing segment
    And the closing operation preserves the drawn shape
    And the first and last anchors have closing handles

  Scenario: Clicking the first anchor before a third point keeps the path open
    Given a fresh Phase 0 shell window
    When two triangle anchors are placed
    And the first triangle anchor is clicked again
    Then the path remains open with 3 anchors and 2 segments
    And the open path has no fill
