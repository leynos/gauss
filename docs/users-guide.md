# Gauss User Guide

Gauss is a Scalable Vector Graphics (SVG) illustration tool built on GPUI (the
GPU-accelerated UI framework from Zed). This guide covers keyboard shortcuts
and interaction patterns.

## Keyboard Shortcuts

Gauss uses a context-aware keyboard shortcut system. Shortcuts are organized
into contexts based on the current editor mode.

### Global Shortcuts

These shortcuts work in any editor mode:

| Action           | macOS       | Linux/Windows |
| ---------------- | ----------- | ------------- |
| Undo             | Cmd+Z       | Ctrl+Z        |
| Redo             | Cmd+Y       | Ctrl+Y        |
| Selection Undo   | Cmd+Shift+Z | Ctrl+Shift+Z  |
| Selection Redo   | Cmd+Shift+Y | Ctrl+Shift+Y  |
| Select All       | Cmd+A       | Ctrl+A        |
| Deselect All     | Cmd+Shift+A | Ctrl+Shift+A  |
| Pen Tool         | P           | P             |
| Select Tool      | V           | V             |
| Toggle Edge Mode | Tab         | Tab           |

**Note:** Gauss uses Cmd+Shift+Z / Ctrl+Shift+Z for Selection Undo rather than
the standard macOS Redo shortcut. This supports the dual history stack design
where document and selection changes can be undone independently.

### Mode-Specific Shortcuts

Some shortcuts only work in specific modes:

#### Manipulate Mode (Select Tool Active)

| Action              | Shortcut            |
| ------------------- | ------------------- |
| Delete Selection    | Backspace or Delete |
| Insert Anchor       | I                   |
| Raise Selection     | Cmd/Ctrl+]          |
| Lower Selection     | Cmd/Ctrl+[          |
| Toggle Segment Kind | Tab                 |

### Window Controls

Gauss provides keyboard-accessible window controls following platform
conventions:

| Action            | macOS      | Linux/Windows |
| ----------------- | ---------- | ------------- |
| Close Window      | Cmd+Q      | Alt+F4        |
| Minimize          | Cmd+M      | Alt+F9        |
| Toggle Maximize   | -          | Alt+F10       |
| Toggle Fullscreen | Ctrl+Cmd+F | Alt+F11       |
| Window Menu       | -          | Alt+Space     |
| Move Window       | -          | Alt+F7        |
| Resize Window     | -          | Alt+F8        |

## Tool Modes

Gauss has two primary tool modes:

### Draw Mode (Pen Tool)

In draw mode, new shapes are created by clicking to place anchor points:

1. Click to place the first anchor
2. Continue clicking to add more anchors
3. Press Tab to toggle between Line and Bezier edge modes
4. Click near the first anchor to close the path
5. Press Escape to finish an open path and switch to Manipulate mode

### Manipulate Mode (Select Tool)

In manipulate mode, existing shapes can be selected and edited:

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

Selection can be undone and redone independently of document changes.

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
- **Selection changes** (Cmd+Shift+Z / Ctrl+Shift+Z, Cmd+Shift+Y /
  Ctrl+Shift+Y): What is currently selected

This allows selection changes to be undone without affecting the document, and
vice versa. For example, after selecting several shapes and then undoing the
selection, the shapes themselves remain unchanged.

## SVG Resource Definitions

Gauss now preserves shared SVG resource definitions in `<defs>` when opening
and saving files.

- **Supported resources**: linear gradients, radial gradients, patterns, and
  symbols.
- **Paint references**: paths that use `stroke="url(#...)"` or
  `fill="url(#...)"` are loaded and saved with those references intact.
- **Resource attribute fidelity**: pattern and symbol attributes (for example,
  `patternUnits`, `patternTransform`, and `preserveAspectRatio`) are preserved
  when opening and saving.
- **Paint-server opacity fidelity**: `stroke-opacity` and `fill-opacity` are
  preserved for `url(#...)` paints.
- **Validation**: if an SVG path references a missing resource ID, Gauss
  reports an open error and keeps the currently loaded document unchanged.
- **Save invariants**: if the current document contains dangling gradient or
  pattern references, save fails with an explicit error instead of silently
  exporting `none`.

In this phase, Gauss supports resource round-tripping and rendering, but does
not yet provide dedicated editing controls for gradients, patterns, or symbols.

## Platform Differences

Gauss adapts to platform conventions:

- **macOS**: Uses Cmd as the primary modifier key
- **Linux/Windows**: Uses Ctrl as the primary modifier key

All shortcuts using the "secondary" modifier (shown as Cmd on macOS) will
automatically use Ctrl on other platforms.
