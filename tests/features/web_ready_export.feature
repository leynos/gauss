Feature: Web-ready SVG export

  Scenario: Web-ready export strips Gauss metadata artefacts
    Given a document with Gauss metadata on a shape
    When I export the document as web-ready SVG
    Then the web-ready SVG strips Gauss metadata artefacts

  Scenario: Checked web-ready export reports missing gradient references
    Given a document with a dangling gradient paint reference
    When I export the document as web-ready SVG with reference validation
    Then the web-ready export fails with a missing gradient reference error

  Scenario: Web-ready output imports as plain SVG metadata defaults
    Given a document with Gauss metadata on a shape
    When I export and re-import the document as web-ready SVG
    Then the imported web-ready shape has default metadata values
    And the imported web-ready shape keeps renderable geometry and paint style
