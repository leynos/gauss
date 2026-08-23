Feature: Grouped document-command history

  Scenario: Grouped document commands collapse to one undo step
    Given a fresh Phase 0 shell test window with one shape
    When two shape moves are committed in one document command group
    Then the document history has gained 1 entry
    And the shape reflects both grouped moves
    When the last document change is undone
    Then the shape returns to its position before the grouped moves

  Scenario: Ending a group without beginning one preserves history
    Given a fresh Phase 0 shell test window
    When a document command group is ended without being begun
    Then the history error is no active group
    And the document history is unchanged

  Scenario: Beginning a nested group preserves history
    Given a fresh Phase 0 shell test window with an active document command group
    When another document command group is begun
    Then the history error is group already active
    And the document history is unchanged
    And the active document command group remains closable

  Scenario: Undo while a command group is active preserves state
    Given a fresh Phase 0 shell test window prepared for undo during an active group
    When document undo is attempted while the group is active
    Then the history error is undo while group active
    And the document and history are unchanged
    And closing the group preserves the document and history

  Scenario: Redo while a command group is active preserves state
    Given a fresh Phase 0 shell test window prepared for redo during an active group
    When document redo is attempted while the group is active
    Then the history error is redo while group active
    And the document and history are unchanged
    And closing the group preserves the document and history
