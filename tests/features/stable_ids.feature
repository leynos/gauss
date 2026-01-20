Feature: Stable shape identifiers

  Scenario: Shape IDs remain stable when reordered
    Given an empty document
    When I append two shapes
    And I reorder the first shape to the end
    Then the document still contains the same shape IDs

  Scenario: AccessKit node IDs round trip
    Given an empty document
    When I append a shape
    And I convert the shape id to an AccessKit node id
    Then the reconstructed shape id matches the original
