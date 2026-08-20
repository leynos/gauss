Feature: Gauss metadata round trips through SVG files

  Scenario: Save preserves the Gauss shape identifier
    Given a shell whose demo shape has a known Gauss identifier
    When the document is saved to a temporary SVG
    Then the saved SVG contains the known Gauss identifier
    And the temporary SVG is cleaned up

  Scenario: Save preserves the Gauss metadata block
    Given a shell with a Gauss project metadata block
    When the document is saved to a temporary SVG
    Then the saved SVG contains a metadata element
    And the saved SVG contains the Gauss project metadata
    And the temporary SVG is cleaned up

  Scenario: Open restores the Gauss shape identifier
    Given a temporary SVG with a known Gauss shape identifier
    When the SVG is opened through the file dialog
    Then the document shape has the known Gauss identifier
    And the temporary SVG is cleaned up

  Scenario: Open restores the Gauss metadata block
    Given a temporary SVG with a Gauss project metadata block
    When the SVG is opened through the file dialog
    Then the shell metadata contains the Gauss project element
    And the temporary SVG is cleaned up
