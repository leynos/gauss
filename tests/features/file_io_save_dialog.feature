Feature: Save and export SVG file dialogs

  Scenario: Save prompts for a path and writes SVG
    Given a fresh shell prepared for a standard save
    Then no file path prompt is visible
    When the configured export action is requested
    Then a file path prompt is visible
    When a temporary save path is selected
    Then the selected save path is recorded
    And the saved SVG contains the demo shape
    And the saved SVG contains the canonical Gauss metadata namespace

  Scenario: Save reports a dangling gradient reference
    Given a shell with a dangling gradient reference prepared for a standard save
    When the configured export action is requested
    When a temporary save path is selected
    Then no save path is recorded
    And the save error reports the missing gradient resource
    And no SVG file is written

  Scenario: Save reports a dangling pattern reference
    Given a shell with a dangling pattern reference prepared for a standard save
    When the configured export action is requested
    When a temporary save path is selected
    Then no save path is recorded
    And the save error reports the missing pattern resource
    And no SVG file is written

  Scenario: Web-ready export strips Gauss metadata
    Given a shell with Gauss metadata prepared for a web-ready export
    Then no file path prompt is visible
    When the configured export action is requested
    Then a file path prompt is visible
    When a temporary save path is selected
    Then the selected save path is recorded
    And no save error is reported
    And the saved SVG is web-ready and contains no Gauss metadata

  Scenario: Web-ready export reports a dangling gradient reference
    Given a shell with a dangling gradient reference prepared for a web-ready export
    When the configured export action is requested
    When a temporary save path is selected
    Then no save path is recorded
    And the save error reports the missing gradient resource
    And no SVG file is written

  Scenario: Web-ready export reports a dangling pattern reference
    Given a shell with a dangling pattern reference prepared for a web-ready export
    When the configured export action is requested
    When a temporary save path is selected
    Then no save path is recorded
    And the save error reports the missing pattern resource
    And no SVG file is written
