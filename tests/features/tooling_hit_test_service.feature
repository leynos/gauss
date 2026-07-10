Feature: Tool hit testing

  Scenario: Hovering a handle prefers the handle hit
    Given a fresh Phase 0 shell window in manipulate mode with a square handle
    When the square's outgoing handle is hovered
    Then the hover hit identifies the square's first handle

  Scenario: The hover hit clears when the cursor moves to empty space
    Given a fresh Phase 0 shell window in manipulate mode with a square handle
    When the square's first segment is hovered
    Then the hover hit identifies a segment
    When the cursor moves to empty space
    Then the hover hit is clear

  Scenario: The hover hit clears when leaving manipulate mode
    Given a fresh Phase 0 shell window in manipulate mode with a square handle
    When the square's outgoing handle is hovered
    Then the hover hit is present
    When the pen tool is activated
    Then the hover hit is clear

  Scenario: Clicking overlapping shapes selects the topmost shape
    Given a fresh Phase 0 shell window in manipulate mode with overlapping squares
    When the overlapping squares are clicked
    Then only the topmost square is selected
