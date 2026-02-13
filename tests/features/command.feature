Feature: Command dispatch

  Commands are concrete, undoable state changes that bridge user intent
  (Actions) to atomic document mutations (DocOps).

  Background:
    Given a document with two shapes
    And the first shape is selected

  Scenario: Delete selection produces valid command
    When I prepare DeleteSelection action
    Then the command should be DeleteShapes
    And the command should target one shape

  Scenario: Delete selection command removes shape
    When I prepare DeleteSelection action
    And I apply the command
    Then the document should have one shape

  Scenario: Delete selection is undoable
    When I prepare DeleteSelection action
    And I apply the command
    And I apply the inverse
    Then the document should have two shapes

  Scenario: Delete selection requires selection
    Given nothing is selected
    When I prepare DeleteSelection action
    Then the command should fail with EmptySelection

  Scenario: Command has human-readable name
    When I prepare DeleteSelection action
    Then the command name should be "Delete"

  Scenario: Inverse command has matching name
    When I prepare DeleteSelection action
    And I apply the command
    Then the inverse name should be "Delete"

  Scenario: Raise selection produces reorder command
    When I prepare RaiseSelection action
    Then the command should be Reorder
    And the command should include one reorder operation

  Scenario: Empty history undo is safe
    When I undo on an empty history
    Then the document should have two shapes
