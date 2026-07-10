Feature: Shell window controls

  Scenario: Maximized state changes trigger a rerender
    Given a fresh non-maximized Phase 0 shell window
    When the window is changed to maximized
    Then the shell observes the maximized state

  Scenario: Resize interaction is prevented while maximized
    Given a fresh maximized Phase 0 shell window
    When the window resize zone is dragged
    Then the window bounds are unchanged
