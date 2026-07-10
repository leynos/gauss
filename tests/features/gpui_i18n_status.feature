Feature: Localized Phase 0 shell status

  Scenario: Default English catalogue localizes the mode status
    Given a fresh Phase 0 shell using the default English catalogue
    Then the mode status shows the tool label "Draw"
    And the mode status shows the edge label "Line"

  Scenario: Injected French catalogue localizes the mode status
    Given a fresh Phase 0 shell using the test French catalogue
    Then the mode status shows the tool label "Dessiner"
    And the mode status shows the edge label "Ligne"

  Scenario: Switching locale updates the mode status
    Given a fresh Phase 0 shell with English and French catalogues
    Then the mode status shows the tool label "Draw"
    When the shell locale is switched to French
    Then the mode status shows the tool label "Dessiner"
    And the mode status shows the edge label "Ligne"

  Scenario: Missing French message falls back to the default locale
    Given a fresh Phase 0 shell with a partial French catalogue
    Then the mode status shows the tool label "Dessiner"
    And the mode status shows the fallback edge label

  Scenario: Manipulate mode omits the edge mode from the status
    Given a fresh Phase 0 shell using the test French catalogue
    When the shell tool mode is changed to manipulate
    Then the mode status shows the tool label "Manipuler"
    And the mode status omits the edge mode
