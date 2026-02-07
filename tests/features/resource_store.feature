Feature: Resource stores and paint references

  Scenario: Gradient and pattern references round-trip through SVG
    Given a document with a gradient stroke and pattern fill
    When I export and re-import the document with resources
    Then the imported document should keep gradient and pattern paint references

  Scenario: Missing resource reference is rejected
    Given an SVG path that references an unknown gradient
    When I import the SVG with resources
    Then the import should fail with a missing referenced resource error
