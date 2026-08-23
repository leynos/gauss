Feature: Close-path history

  Scenario: Closing a path creates one undo entry and undo reopens it
    Given a fresh Phase 0 shell window with an open three-anchor path
    When the path is closed by clicking its first anchor
    Then the path is closed with 3 anchors
    And the document history has gained 1 entry
    When the last document change is undone
    Then the path is open with 3 anchors
