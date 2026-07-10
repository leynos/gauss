Feature: Shell accessibility service

  Scenario: Initial accessibility tree is emitted
    Given a fresh Phase 0 shell window
    Then one initial accessibility tree update is emitted

  Scenario: Shape insertion emits an incremental accessibility update
    Given a fresh Phase 0 shell window
    When a shape is inserted
    Then one incremental accessibility update includes the inserted shape

  Scenario: Idle rendering emits no accessibility update
    Given a fresh Phase 0 shell window
    When accessibility updates are cleared
    And the shell is rendered without state changes
    Then no accessibility update is emitted

  Scenario: Stale shape selection emits no accessibility update
    Given a fresh Phase 0 shell window
    When a stale shape is selected
    Then no accessibility update is emitted

  Scenario: Close-window action requests quit
    Given a fresh Phase 0 shell window
    When the close-window action is dispatched
    Then the shell requests quit

  Scenario: Accessibility close click requests quit
    Given a fresh Phase 0 shell window
    When the close button is clicked through accessibility
    Then the accessibility request routes to close the window
    And the shell requests quit

  Scenario: Unsupported accessibility action is rejected
    Given a fresh Phase 0 shell window
    When the close button receives an unsupported accessibility action
    Then the accessibility request is rejected as unsupported
    And the shell does not request quit

  Scenario: Unknown accessibility node is rejected
    Given a fresh Phase 0 shell window
    When an unknown accessibility node is clicked
    Then the accessibility request is rejected as an unknown node
    And the shell does not request quit
