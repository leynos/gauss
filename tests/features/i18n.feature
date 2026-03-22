Feature: Internationalization (i18n) message lookup and fallback

  Background:
    Given a default English catalog exists
    And a test French catalog is available for testing

  Scenario: Successful message lookup in default catalog
    Given the locale is set to "en-GB"
    When I look up the message "tool_mode.draw"
    Then the message should be "Draw"

  Scenario: Successful message lookup in alternate catalog
    Given the locale is set to "fr-FR"
    When I look up the message "tool_mode.draw"
    Then the message should be "Dessiner"

  Scenario: Fallback to English when locale is unsupported
    Given the locale is set to "de-DE"
    When I look up the message "tool_mode.draw"
    Then the message should be "Draw"
    And a fallback should have occurred

  Scenario: Error when message key is unavailable
    Given the locale is set to "en-GB"
    When I look up the message "nonexistent.key"
    Then a lookup error should be returned

  Scenario: Edge mode message lookup
    Given the locale is set to "en-GB"
    When I look up the message "edge_mode.line"
    Then the message should be "Line"
    When I look up the message "edge_mode.bezier_auto"
    Then the message should be "Bezier (auto)"
