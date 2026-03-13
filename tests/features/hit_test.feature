Feature: Shared hit-test service

  The shared hit-test service must provide deterministic pointer and hover
  results so selection and hover flows stay consistent.

  Scenario: Handle hits have highest priority
    Given a document with one square shape and an out handle
    And the hit-test cursor is at world position 2 0
    And the hit-test tolerance is 0.5
    When I run a pointer hit-test query
    Then the hit-test result is Handle

  Scenario: Segment hits win over shape-body fallback
    Given a document with one square shape and an out handle
    And the hit-test cursor is at world position 5 0
    And the hit-test tolerance is 0.5
    When I run a pointer hit-test query
    Then the hit-test result is Segment

  Scenario: Hover and pointer queries match
    Given a document with one square shape and an out handle
    And the hit-test cursor is at world position 5 0
    And the hit-test tolerance is 0.5
    When I run both pointer and hover hit-test queries
    Then the pointer and hover hit-test results match

  Scenario: Overlapping shapes resolve to the topmost shape
    Given a document with two overlapping square shapes
    And the hit-test cursor is at world position 5 5
    And the hit-test tolerance is 0.5
    When I run a pointer hit-test query
    Then the hit-test result is TopmostShape

  Scenario: Non-finite tolerance produces no hit
    Given a document with one square shape and an out handle
    And the hit-test cursor is at world position 5 5
    And the hit-test tolerance is NaN
    When I run a pointer hit-test query
    Then the hit-test result is None
