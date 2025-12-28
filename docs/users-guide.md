# Gauss User Guide

Gauss is an SVG illustration tool built on GPUI. This guide covers keyboard
shortcuts and interaction patterns.

## Keyboard Shortcuts

Gauss uses a context-aware keyboard shortcut system. Shortcuts are organised
into contexts based on the current editor mode.

### Global Shortcuts

These shortcuts work in any editor mode:

| Action | macOS | Linux/Windows |
|--------|-------|---------------|
| Undo | Cmd+Z | Ctrl+Z |
| Redo | Cmd+Y | Ctrl+Y |
| Selection Undo | Cmd+Shift+Z | Ctrl+Shift+Z |
| Selection Redo | Cmd+Shift+Y | Ctrl+Shift+Y |
| Select All | Cmd+A | Ctrl+A |
| Deselect All | Cmd+Shift+A | Ctrl+Shift+A |
| Pen Tool | P | P |
| Select Tool | V | V |
| Toggle Edge Mode | Tab | Tab |

**Note:** Gauss uses Cmd+Shift+Z / Ctrl+Shift+Z for Selection Undo rather than
the standard macOS Redo shortcut. This supports the dual history stack design
where document and selection changes can be undone independently.

### Mode-Specific Shortcuts

Some shortcuts only work in specific modes:

#### Manipulate Mode (Select Tool Active)

| Action | Shortcut |
|--------|----------|
| Delete Selection | Backspace or Delete |

### Window Controls

Gauss provides keyboard-accessible window controls following platform
conventions:

| Action | macOS | Linux/Windows |
|--------|-------|---------------|
| Close Window | Cmd+Q | Alt+F4 |
| Minimise | Cmd+M | Alt+F9 |
| Toggle Maximise | - | Alt+F10 |
| Toggle Fullscreen | Ctrl+Cmd+F | Alt+F11 |
| Window Menu | - | Alt+Space |
| Move Window | - | Alt+F7 |
| Resize Window | - | Alt+F8 |

## Tool Modes

Gauss has two primary tool modes:

### Draw Mode (Pen Tool)

In draw mode, you create new shapes by clicking to place anchor points:

1. Click to place the first anchor
2. Continue clicking to add more anchors
3. Press Tab to toggle between Line and Bezier edge modes
4. Click near the first anchor to close the path
5. Press Escape to finish an open path and switch to Manipulate mode

### Manipulate Mode (Select Tool)

In manipulate mode, you select and edit existing shapes:

1. Click on a shape to select it
2. Shift+click to add or remove from selection
3. Drag selected shapes to move them
4. Click on anchors or handles to select them for editing
5. Press Delete or Backspace to delete selected anchors
6. Press Escape to switch to Draw mode

## Selection

- **Click**: Select a single item (deselects others)
- **Shift+Click**: Toggle item in selection
- **Cmd+A / Ctrl+A**: Select all shapes
- **Cmd+Shift+A / Ctrl+Shift+A**: Deselect all

Selection can be undone and redone independently from document changes.

## Edge Modes

When drawing, the Tab key toggles between:

- **Line**: Creates straight line segments
- **Bezier (auto)**: Creates smooth curves with automatically calculated
  control handles using Catmull-Rom interpolation

## Viewport Navigation

- **Scroll**: Pan the canvas
- **Cmd+Scroll / Ctrl+Scroll**: Zoom around the cursor position

## Undo/Redo

Gauss maintains separate history stacks for:

- **Document changes** (Cmd+Z / Ctrl+Z, Cmd+Y / Ctrl+Y): Shape creation,
  movement, deletion, style changes
- **Selection changes** (Cmd+Shift+Z / Ctrl+Shift+Z, Cmd+Shift+Y / Ctrl+Shift+Y):
  What is currently selected

This allows you to undo selection changes without affecting the document,
and vice versa. For example, if you select several shapes and then undo the
selection, the shapes themselves remain unchanged.

## Platform Differences

Gauss adapts to platform conventions:

- **macOS**: Uses Cmd as the primary modifier key
- **Linux/Windows**: Uses Ctrl as the primary modifier key

All shortcuts using the "secondary" modifier (shown as Cmd on macOS) will
automatically use Ctrl on other platforms.
