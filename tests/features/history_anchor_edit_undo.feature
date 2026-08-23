Feature: Anchor edit history

  Scenario: Anchor insertion and deletion round-trip through undo and redo
    Given a fresh Phase 0 shell window with a two-anchor path
    When an anchor is inserted at the path midpoint
    Then the path has 3 anchors and 2 segments
    And the document history has gained 1 entry
    When the inserted anchor is deleted
    Then the path has 2 anchors and 1 segment
    And the document history has gained 2 entries
    When the last document change is undone
    Then the path has 3 anchors and 2 segments
    When the last document change is undone
    Then the path has 2 anchors and 1 segment
    When the last document change is redone
    Then the path has 3 anchors and 2 segments
    When the last document change is redone
    Then the path has 2 anchors and 1 segment
