Feature: Manipulate-mode selection

  Scenario: Dragging an unselected shape selects without moving it
    Given a fresh Phase 0 shell window
    Given an unselected square is arranged
    When the unselected square is dragged by its bounding box
    Then the square is selected
    Then no drag starts before the square is preselected
    Then the square remains unchanged

  Scenario: Clicking empty space clears selection
    Given a fresh Phase 0 shell window
    Given a selected square is arranged
    When empty canvas space is clicked
    Then the selection is empty

  Scenario: Shift-click toggles multi-selection without dragging
    Given a fresh Phase 0 shell window
    Given a two-anchor shape is arranged in manipulate mode
    When the first anchor is clicked
    Then only the first anchor is selected
    When the second anchor is shift-clicked
    Then both anchors are selected
    Then no drag is active
    When the first anchor is shift-clicked
    Then only the second anchor is selected

  Scenario: Dragging one selected shape moves the full selection
    Given a fresh Phase 0 shell window
    Given two selected squares are arranged
    When the first selected square is pressed
    Then both squares remain selected
    When the first selected square is dragged
    Then both squares move by the drag delta
    Then both squares remain selected
    Then no drag is active

  Scenario: Clicking inside a shape bounding box selects it
    Given a fresh Phase 0 shell window
    Given an unselected square is arranged for bounding-box selection
    When the centre of the square is clicked
    Then only the square is selected

  Scenario: Right-clicking in manipulate mode is a no-op
    Given a fresh Phase 0 shell window
    Given manipulate mode is active
    When the canvas is right-clicked
    Then the selection is unchanged
    Then no drag is active

  Scenario: A zero-delta drag is a no-op
    Given a fresh Phase 0 shell window
    Given a drawn shape is selected in manipulate mode
    When the selected point is dragged by zero distance
    Then the document history length is unchanged
    Then the selection is unchanged
    Then no drag is active
