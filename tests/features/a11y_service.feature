Feature: A11yService incremental tree updates
  Scenario: Initial accessibility snapshot includes window chrome nodes with stable IDs
    Given a fresh accessibility service snapshot
    When I publish the initial accessibility snapshot
    Then one initial accessibility update is queued
    And the update includes titlebar and window control node IDs
    And the chrome nodes expose expected roles, labels, and shortcut hints

  Scenario: Maximized window exposes restore semantics on the maximize node
    Given a fresh accessibility service snapshot
    And the snapshot marks the window as maximized
    When I publish the initial accessibility snapshot
    Then one initial accessibility update is queued
    And the maximize node uses the restore label and maximize shortcut hint

  Scenario: Adding a shape emits one inserted accessibility node update
    Given an initialized accessibility service baseline
    When I append one shape and publish an incremental snapshot
    Then one incremental accessibility update is queued
    And the inserted node list contains the appended shape node ID

  Scenario: Unchanged state emits no accessibility updates
    Given an initialized accessibility service baseline
    When I publish the same snapshot again
    Then no new accessibility update is queued

  Scenario: Duplicate node ID is reported and update is aborted
    Given an accessibility snapshot with duplicate shape node IDs
    When I publish the duplicate-node accessibility snapshot
    Then publishing fails with a duplicate shape node ID error

  Scenario: Close button click request routes to the existing close action
    Given a fresh accessibility service snapshot
    When I route a click request for the close accessibility node
    Then accessibility request routing succeeds
    And the request routes to the close window action

  Scenario: Unsupported close button action request is rejected
    Given a fresh accessibility service snapshot
    When I route a focus request for the close accessibility node
    Then routing fails with an unsupported accessibility action error

  Scenario: Unknown accessibility node request is rejected
    Given a fresh accessibility service snapshot
    When I route a click request for an unknown accessibility node
    Then routing fails with an unknown accessibility node error
