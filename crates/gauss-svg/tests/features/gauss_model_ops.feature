Feature: Model operations

  Scenario: Insert and undo a shape
    Given an empty document
    When I insert a shape
    Then the document contains 1 shapes
    When I undo the insertion
    Then the document contains 0 shapes

  Scenario: Export a closed triangle path
    Given an empty document
    When I add a closed triangle path
    When I export the document as an SVG
    Then the exported SVG contains d="M 1 1 L 2 1 L 2 2 Z"
