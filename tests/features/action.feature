Feature: Action categorisation

  Actions are categorised for correct dispatch routing. Document actions
  produce undoable commands; editor actions update selection, viewport,
  or tool state.

  Scenario: Document action kind
    Given the action DeleteSelection
    Then its kind should be Document

  Scenario: Editor action kind for selection
    Given the action SelectAll
    Then its kind should be Editor

  Scenario: Editor action kind for tools
    Given the action ActivatePenTool
    Then its kind should be Editor

  Scenario: Editor action kind for history
    Given the action Undo
    Then its kind should be Editor

  Scenario: Action requires selection
    Given the action DeleteSelection
    Then it should require a selection

  Scenario: Action does not require selection
    Given the action SelectAll
    Then it should not require a selection
