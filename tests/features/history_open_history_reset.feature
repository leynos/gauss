Feature: Open-document history reset

  Scenario: Opening a document clears history and selection
    Given a fresh Phase 0 shell test window with document history and selection
    When another document is opened
    Then the document history is empty
    And the selection is empty
