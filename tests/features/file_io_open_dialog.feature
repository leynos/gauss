Feature: Open SVG file dialog

  Scenario: Open loads the selected SVG
    Given a temporary plain SVG
    Then no file path prompt is visible
    When Open is requested
    Then a file path prompt is visible
    When the temporary SVG is selected
    Then the selected SVG path is recorded
    And the document contains one shape
    And the document has no gradient or pattern resources

  Scenario: Open loads resource definitions and paint references
    Given a temporary SVG with gradient and pattern resources
    When Open is requested
    And the temporary SVG is selected
    Then the document contains one gradient and one pattern
    And the imported shape references the gradient and pattern

  Scenario: Open reports a missing resource reference
    Given a temporary SVG with a missing resource reference
    When Open is requested
    And the temporary SVG is selected
    Then the original document and resources are preserved
    And the open error reports a missing resource

  Scenario: Open accepts the canonical Gauss metadata namespace
    Given a temporary SVG with the canonical Gauss metadata namespace
    When Open is requested
    And the temporary SVG is selected
    Then the document contains one shape
    And no open error is reported

  Scenario: Open rejects a non-canonical Gauss metadata prefix
    Given a temporary SVG with a non-canonical Gauss metadata prefix
    When Open is requested
    And the temporary SVG is selected
    Then the open error reports the canonical Gauss namespace declaration
