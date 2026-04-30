Feature: Widget Capability Audit for Phase 1-2

  As a product owner
  I want a complete inventory of required UI controls for Phase 1 and Phase 2
  So that the team can plan implementation and identify custom widget needs

  Background:
    Given the widget capability audit inventory is loaded

  Scenario: Phase 1 toolbar tools are catalogued
    When I query controls for Phase 1 toolbar
    Then the inventory includes a Selection Tool
    And the inventory includes a Direct Selection Tool
    And the inventory includes a Pen Tool
    And the inventory includes a Rectangle Tool
    And the inventory includes an Ellipse Tool
    And the inventory includes a Line Tool

  Scenario: Phase 1 properties panel controls are catalogued
    When I query controls for Phase 1 properties panel
    Then the inventory includes an X Position Field
    And the inventory includes a Y Position Field
    And the inventory includes a Width Field
    And the inventory includes a Height Field
    And the inventory includes a Rotation Field

  Scenario: Phase 1 alignment controls are catalogued
    When I query controls for Phase 1 alignment panel
    Then the inventory includes alignment controls for left, centre, and right
    And the inventory includes alignment controls for top, centre, and bottom
    And the inventory includes distribution controls for horizontal and vertical

  Scenario: Phase 1 style panel controls are catalogued
    When I query controls for Phase 1 style panel
    Then the inventory includes stroke colour, width, and opacity controls
    And the inventory includes fill colour and opacity controls

  Scenario: Phase 1 layers panel controls are catalogued
    When I query controls for Phase 1 layers panel
    Then the inventory includes layer visibility toggle
    And the inventory includes layer lock toggle
    And the inventory includes layer rename capability
    And the inventory includes layer reorder capability

  Scenario: Phase 2 text controls are catalogued
    When I query controls for Phase 2
    Then the inventory includes character panel controls
    And the inventory includes paragraph panel controls
    And the inventory includes canvas text editing controls

  Scenario: Phase 1 history panel controls are catalogued
    When I query controls for Phase 1 history panel
    Then the inventory includes undo and redo controls
    And the inventory includes history clear capability

  Scenario: Phase 2 canvas text editor controls are catalogued
    When I query controls for Phase 2 canvas text editor
    Then the inventory includes text insertion capability
    And the inventory includes text selection capability

  Scenario: All controls have complete requirement fields
    When I examine all controls in the inventory
    Then each control has a non-empty name
    And each control has a user job description
    And each control has at least one defined state
    And each control has an accessibility role and label
    And each control cites at least one requirement source

  Scenario: Current shell evidence is tracked
    When I query controls with current shell evidence
    Then at least one control has evidence
    And each control with evidence references a source file path

  Scenario: All toolbar tools have keyboard shortcuts
    When I query Phase 1 toolbar controls
    Then each tool has a keyboard shortcut defined

  Scenario: All controls support keyboard-only operation
    When I examine all controls in the inventory
    Then each control supports keyboard-only operation

  Scenario: All controls integrate with Action/Command system
    When I examine all controls in the inventory
    Then all controls require action linkage
    And action linkage includes implementation notes
