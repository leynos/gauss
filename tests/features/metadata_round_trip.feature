Feature: Metadata round-trip fidelity

  Scenario: Shape IDs survive save and reload
    Given a document with a shape
    When I export and re-import the document
    Then the imported shape retains its gauss:id

  Scenario: Shape names survive round-trip
    Given a document with a named shape
    When I export and re-import the document
    Then the imported shape retains its name

  Scenario: Locked and hidden states survive round-trip
    Given a document with a locked and hidden shape
    When I export and re-import the document
    Then the imported shape retains its locked and hidden states

  Scenario: Unknown Gauss attributes survive round-trip
    Given an SVG with unknown gauss attributes on a shape
    When I import and re-export the SVG
    Then the unknown gauss attributes are preserved

  Scenario: Metadata block survives round-trip
    Given an SVG with a metadata block
    When I import and re-export the SVG
    Then the metadata block content is preserved

  Scenario: Plain SVG without metadata imports cleanly
    Given a plain SVG without Gauss metadata
    When I import the SVG
    Then all shapes have default metadata
