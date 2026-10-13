Feature: Step returns spelled through a Result alias

  Steps whose declared return type is a local alias of `Result` must still be
  classified as fallible, so the `Ok` payload is injected as the step value and
  `Err` propagates out of the scenario.

  Scenario: A step returning Ok through the result alias succeeds
    Given an empty document for alias classification
    When a step returns a sentinel through the result alias
    Then the returned sentinel is injected as the step value
    And the failing step is not reached

  Scenario: A step returning Err through the result alias fails the scenario
    Given an empty document for alias classification
    When a step returns Err through the result alias
    Then the returned sentinel is injected as the step value
