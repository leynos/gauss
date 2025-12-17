Feature: Model operations

  Scenario: Insert and undo a shape
    Given an empty document
    When I insert a shape
    Then the document contains 1 shapes
    When I undo the insertion
    Then the document contains 0 shapes
