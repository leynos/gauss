Feature: Shell style controls

  Scenario: Style changes apply to selected shapes and are undoable
    Given a drawn shape with its first anchor selected
    When the stroke is changed to red
    And the fill is changed to blue
    Then the shape has a red stroke and blue fill
    And two document history entries are added
    When both style changes are undone
    Then the shape has its original style
