Feature: Tooling keybinding integration

  Scenario: UI initialization registers action bindings
    Given the test application initializes its UI action bindings
    When V is pressed in the opened Phase 0 shell
    Then manipulate mode is active

  Scenario: Select all selects every shape
    Given a fresh Phase 0 shell window with two unselected shapes
    When the select-all action is dispatched
    Then both shapes are selected

  Scenario: Deselect all clears the selection
    Given a fresh Phase 0 shell window with one selected shape
    When the deselect-all action is dispatched
    Then the selection is empty

  Scenario: Activating the pen tool switches to draw mode
    Given a fresh Phase 0 shell window in manipulate mode
    When the pen tool is activated
    Then draw mode is active

  Scenario: Activating the select tool switches to manipulate mode
    Given a fresh Phase 0 shell window in draw mode
    When the select tool is activated
    Then manipulate mode is active

  Scenario: Activating the select tool clears the active draw shape
    Given a fresh Phase 0 shell window with an active draw shape
    When the select tool is activated
    Then the active draw shape is clear

  Scenario: Tab toggles the edge mode in draw mode
    Given a fresh Phase 0 shell window in draw mode with line edge mode
    When Tab is pressed
    Then Bezier auto edge mode is active

  Scenario: Tab does not toggle the draw edge mode in manipulate mode
    Given a fresh Phase 0 shell window in manipulate mode with line edge mode
    When Tab is pressed
    Then manipulate mode with line edge mode remains active
