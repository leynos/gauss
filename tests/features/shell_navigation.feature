Feature: Shell navigation buttons

  Scenario: Navigation buttons undo and redo document history
    Given a selected anchor with a red stroke
    When navigation Back is clicked
    Then the original stroke is restored
    When navigation Forward is clicked
    Then the stroke is red

  Scenario: Shift navigation buttons undo and redo selection history
    Given a selected anchor with a red stroke
    When the selection is cleared
    And Shift-navigation Back is clicked
    Then the previous selection is restored
    And the stroke remains red
    When Shift-navigation Forward is clicked
    Then the selection is cleared
