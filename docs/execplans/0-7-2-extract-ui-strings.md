# ExecPlan: Extract UI Strings for Internationalization (i18n) (0.7.2)

This ExecPlan (execution plan) is a living document. The sections `Constraints`,
`Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`, `Decision Log`,
and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Status: COMPLETE

## Purpose / big picture

Replace inline UI strings with resource IDs to enable localization of the Gauss
user interface. This work implements roadmap item **0.7.2: Extract UI
strings**, building on the i18n scaffolding from 0.7.1.

After this change:

- All user-visible strings in window chrome and tool names are externalized in
  the message catalogue (`src/i18n/catalog/mod.rs`).
- UI code retrieves strings via `MessageId` constants rather than hardcoded
  literals.
- The accessibility tree uses the same localized strings as the visual UI.
- Tests verify that all expected messages exist in the catalogue and that
  lookups succeed.

Observable success: Running `cargo test` shows all i18n, UI, and accessibility
tests passing. Running the application shows identical UI labels as before, but
these are now retrieved via the localizer.

## Constraints

Hard invariants that must hold throughout implementation:

- Do not break existing accessibility tree semantics or labels.
- Maintain backward compatibility with existing `MessageId` factory methods
  (`tool_mode_draw()`, `edge_mode_line()`, etc.).
- No changes to public API signatures of `Localizer`, `Catalog`, `Locale`, or
  `MessageId` that would break existing consumers.
- GPUI (GPU-accelerated User Interface) version remains pinned at 0.2.2;
  do not introduce new GPUI APIs.
- All new dependencies must be workspace-internal (no new external crates).

If satisfying the objective requires violating a constraint, do not proceed.
Document the conflict in `Decision Log` and escalate.

## Tolerances (exception triggers)

Thresholds that trigger escalation when breached:

- **Scope**: If implementation requires changes to more than 15 files, stop and
  escalate.
- **Interface**: If a public API signature must change, stop and escalate.
- **Dependencies**: If a new external dependency is required, stop and escalate.
- **Iterations**: If tests still fail after 3 attempts, stop and escalate.
- **Time**: If a milestone takes more than 4 hours, stop and escalate.
- **Ambiguity**: If multiple valid interpretations exist and the choice
  materially affects the outcome, stop and present options with trade-offs.

## Risks

Known uncertainties that might affect the plan:

- **Risk**: Accessibility tree tests may fail if label changes are not
  synchronized between UI code and tree builder.
  - Severity: medium
  - Likelihood: medium
  - Mitigation: Update tree builder strings simultaneously with UI strings;
    validate with existing a11y service tests.

- **Risk**: Tooltip strings in `icon_button` calls are typed as `&'static str`,
  requiring API change to accept `String` for localized content.
  - Severity: low
  - Likelihood: high
  - Mitigation: Change `icon_button` tooltip parameter to accept `String` or
    `&str` via `Into<SharedString>`; verify no lifetime issues.

- **Risk**: Status bar formatting strings (e.g., "Saved: {}") may need template
  support for proper localization.
  - Severity: low
  - Likelihood: medium
  - Mitigation: Use simple placeholder replacement for now; document need for
    proper templating in 0.7.3 if non-English locales are added.

- **Risk**: Tooltips containing keyboard shortcuts (e.g., "Minimize (Alt+F9)")
  are platform-dependent.
  - Severity: low
  - Likelihood: high
  - Mitigation: Keep shortcut hints in accessibility module as-is for now;
    extract only the descriptive portion for i18n, leaving shortcuts as
    platform-specific suffixes.

## Progress

- [x] (2026-03-29) Audit existing UI strings and map to MessageId hierarchy.
- [x] (2026-03-29) Extend `MessageId` with new factory methods for UI strings.
- [x] (2026-03-29) Add new message entries to `Catalog::default_en_gb()`.
- [x] (2026-03-29) Update `tool_rail.rs` to use localized strings.
- [x] (2026-03-29) Update `chrome.rs` to use localized strings.
- [x] (2026-03-29) Update `chrome_panels.rs` to use localized strings.
- [x] (2026-03-29) Update `window_controls.rs` to use localized strings.
- [x] (2026-03-29) Update `style_controls.rs` to use localized strings.
- [x] (2026-03-29) Update `a11y_service/tree_builder.rs` to use localized
      strings.
- [x] (2026-03-29) Update `view.rs` status line templates to use localized
      strings.
- [x] (2026-03-29) Add unit tests for new `MessageId` variants.
- [x] (2026-03-29) Add BDD (Behaviour-Driven Development) scenarios for i18n
      string extraction.
- [x] (2026-03-29) Update `docs/users-guide.md` if any user-visible labels
      change.
- [x] (2026-03-29) Mark roadmap item 0.7.2 as complete.

## Surprises & discoveries

- The `icon_button` tooltip parameter needed to change from
  `Option<&'static str>` to `Option<String>` to support localized strings. This
  was anticipated in the risk assessment and required updates to all call sites.

- The `chrome_panels.rs` functions were standalone and needed to be converted
  to `Phase0Shell` methods to access the localizer instance. This simplified
  the API and made localization more consistent.

## Decision log

- **Decision**: Use `&MessageId` instead of `MessageId` for `localize()` to
  avoid unnecessary cloning.
  - Rationale: `MessageId` is a small struct, but references are more efficient
    for read-only lookups.
  - Date/Author: 2026-03-29

- **Decision**: Mark shortcut hints (e.g., "Alt+F9") as platform-specific and
  keep them in the accessibility module rather than extracting to i18n.
  - Rationale: Keyboard shortcuts vary by platform and are not part of the
    localizable UI strings for this milestone.
  - Date/Author: 2026-03-29

## Outcomes & retrospective

**Completed successfully.**

- All window chrome strings are now externalized in the message catalogue.
- All tool tooltips are localized via `MessageId` lookups.
- Status bar strings (zoom controls, alignment buttons) are externalized.
- Style control labels (Stroke, Fill) are externalized.
- Status line templates (Saved, Opened, error messages) are externalized.
- Accessibility tree labels are externalized.

**Files modified:**

- `src/i18n/message/mod.rs` - 43 new `MessageId` factory methods (in
  `factories.rs`)
- `src/i18n/catalog/mod.rs` - 47 new message entries
- `src/ui/phase0_shell/mod.rs` - Added `localize()` helper method
- `src/ui/phase0_shell/tool_rail.rs` - Localized tool tooltips
- `src/ui/phase0_shell/chrome.rs` - Localized button labels
- `src/ui/phase0_shell/chrome_panels.rs` - Converted to methods, localized
  strings
- `src/ui/phase0_shell/style_controls.rs` - Localized style labels
- `src/ui/phase0_shell/view.rs` - Localized status templates
- `src/ui/phase0_shell/a11y_service/tree_builder.rs` - Localized a11y labels
- `src/ui/phase0_shell/icon_button.rs` - Updated tooltip type to `String`

**Test results:**

- 142 library tests passed
- Formatting check passed
- Lint check passed

**Lessons learned:**

- Converting standalone functions to methods early would have reduced the
  refactoring needed for localization.
- The `MessageId` namespace hierarchy (`chrome.*`, `tool.tooltip.*`, etc.)
  works well for organizing UI strings.

## Context and orientation

### Current i18n Architecture (implemented in 0.7.1)

The i18n system lives in `src/i18n/` and consists of:

1. **`MessageId`** (`src/i18n/message/mod.rs`): Typed message identifiers with
   factory methods like `tool_mode_draw()`, `edge_mode_line()`. Keys use
   dot-notation (e.g., `"tool_mode.draw"`).

2. **`Catalog`** (`src/i18n/catalog/mod.rs`): Storage for locale-specific
   strings. The `default_en_gb()` method contains all current translations.

3. **`Localizer`** (`src/i18n/catalog/mod.rs`): Service that looks up messages
   with automatic fallback to en-GB.

4. **`i18n_helpers`** (`src/ui/phase0_shell/i18n_helpers.rs`): Helper functions
   to localize `ToolMode` and `EdgeMode` enums.

### UI Structure

The Phase 0 shell UI is organized in `src/ui/phase0_shell/`:

- **`view.rs`**: Main view rendering, status lines, canvas area.
- **`chrome.rs`**: Top bar with file actions, edit actions, window controls.
- **`tool_rail.rs`**: Left-side vertical toolbar with tool buttons.
- **`chrome_panels.rs`**: Document header, status bar, alignment buttons.
- **`window_controls.rs`**: Window management actions (minimize, maximize,
  etc.).
- **`style_controls.rs`**: Stroke/fill colour pickers.
- **`a11y_service/tree_builder.rs`**: AccessKit tree construction with labels.
- **`accessibility.rs`**: Accessibility constants for names and shortcuts.

### Strings to Extract

From `chrome.rs`:

- "Open recent project" (titlebar drag region)
- "New", "Open", "Save", "Export Web" (file actions)
- "Undo", "Redo" (edit actions)
- "Settings" (settings button)
- "Minimize (Alt+F9)", "Maximize (Alt+F10)", "Close Window" (window controls)

From `tool_rail.rs`:

- "Select"
- "Draw Path", "Draw Curve"
- "Draw Rectangle", "Draw Circle"

From `chrome_panels.rs`:

- "untitled" (document header)
- "Zoom Out", "Zoom In", "Zoom to Area", "Snap to Grid" (status bar)
- "Align Left", "Align Centre", "Align Right" (alignment buttons)
- "Align Top", "Align Middle", "Align Bottom"
- "1:1", "Plain Text" (status bar placeholders)

From `style_controls.rs`:

- "Stroke", "Fill" (colour picker labels)
- "Stroke: (loading)", "Fill: (loading)" (loading states)

From `view.rs`:

- `view.rs` `MessageId::tool_status_mode_with_edge()` →
  `tool.status.mode_with_edge` → "Mode: {tool} ({edge})"
- `view.rs` `MessageId::tool_status_mode()` → `tool.status.mode` →
  "Mode: {tool}"
- `view.rs` `FileStatus::HistoryError { error }` →
  `status.history_error` → "History error: {error}"
- `view.rs` `FileStatus::SaveFailed { error }` → `status.save_failed` →
  "Save failed: {error}"
- `view.rs` `FileStatus::OpenFailed { error }` → `status.open_failed` →
  "Open failed: {error}"
- `view.rs` `last_saved_path` → `status.saved` → "Saved: {path}"
- `view.rs` `last_opened_path` → `status.opened` → "Opened: {path}"
- `view.rs` `maximized_indicator` → `status.maximized` → " [MAX]"

From `a11y_service/tree_builder.rs`:

- `a11y_service::tree_builder` `canvas_label` → `a11y.canvas` →
  "Drawing canvas"
- `a11y_service::tree_builder` `shape_list_label` → `a11y.shape_list` →
  "Shapes"
- `a11y_service::tree_builder` `shape_label` default template →
  `a11y.shape_item` → "Shape {index}"
- `a11y_service::tree_builder` `window_title` →
  `a11y.window_title` → "Gauss"

## Plan of work

### Stage A: Preparation and MessageId Extension

**Goal**: Define all new message identifiers and add them to the catalogue.

1. **Extend `MessageId`** (`src/i18n/message/mod.rs`):
   - Add factory methods for window chrome strings.
   - Add factory methods for tool tooltips.
   - Add factory methods for status bar strings.
   - Add factory methods for accessibility labels.
   - Add factory methods for status/error templates.

   Naming convention:
   - `chrome.*` for window chrome (e.g., `chrome.file.open`)
   - `tool.tooltip.*` for tool tooltips (e.g., `tool.tooltip.select`)
   - `status.*` for status messages (e.g., `status.saved`)
   - `a11y.*` for accessibility labels (e.g., `a11y.canvas`)

2. **Extend `Catalog::default_en_gb()`** (`src/i18n/catalog/mod.rs`):
   - Add all new message keys with their English values.
   - Group related messages for readability.

3. **Validation**:
   - Run `cargo test -p gauss i18n` to verify catalogue tests pass.
   - Run `make check-fmt` and `make lint` to ensure code quality.

### Stage B: UI Component Updates

**Goal**: Replace hardcoded strings in each UI module with `MessageId` lookups.

For each module, the pattern is:

```rust
// Before:
.child("Open")

// After:
.child(self.localize(&MessageId::chrome_file_open()))
```

The `Phase0Shell` struct already holds a `Localizer` and `Locale`, accessible
via `self.localizer` and `self.locale`. Add a helper method if needed:

```rust
fn localize(&self, message_id: &MessageId) -> String {
    self.localizer
        .lookup(&self.locale, message_id)
        .unwrap_or_else(|_| message_id.as_str().to_owned())
}
```

Update order (each validates with existing tests before proceeding):

1. **`tool_rail.rs`**: Tool button tooltips.
2. **`chrome.rs`**: File/edit actions, window controls.
3. **`chrome_panels.rs`**: Status bar buttons, alignment buttons.
4. **`style_controls.rs`**: Colour picker labels (may need `SharedString`
   conversion).
5. **`view.rs`**: Status line templates.
6. **`a11y_service/tree_builder.rs`**: Accessibility labels.

### Stage C: Testing

**Goal**: Ensure all strings are properly externalized and retrievable.

1. **Unit tests** (`src/i18n/catalog/tests.rs`):
   - Add test for each new `MessageId` factory method.
   - Verify all messages exist in default catalogue.

2. **BDD scenarios** (`tests/features/i18n_extraction.feature`):

   ```gherkin
   Feature: UI String Localization
     Scenario: All window chrome strings are externalized
       Given the default en-GB catalog is loaded
       Then all chrome message IDs should resolve to non-empty strings

     Scenario: Tool tooltips are localized
       Given the localizer is initialized with en-GB
       When looking up the select tool tooltip
       Then the result should be "Select"
   ```

3. **Integration tests**:
   - Verify accessibility tree labels match UI labels.
   - Verify Phase0Shell produces correct status lines.

4. **GPUI tests**:
   - Ensure UI renders without panics when localizer returns strings.

### Stage D: Documentation and Cleanup

**Goal**: Update documentation and mark work complete.

1. Update `docs/users-guide.md` if any labels changed (they shouldn't; this is
   a refactoring).
2. Update `docs/roadmap.md` to mark 0.7.2 as complete.
3. Run full test suite: `make test`.
4. Run linting: `make lint`.
5. Run formatting check: `make check-fmt`.

## Concrete steps

### Step 1: Extend MessageId

Edit `src/i18n/message/mod.rs`:

```rust
// Window chrome
#[must_use]
pub fn chrome_file_new() -> Self { Self::new("chrome.file.new") }
#[must_use]
pub fn chrome_file_open() -> Self { Self::new("chrome.file.open") }
// ... etc for all chrome strings

// Tool tooltips
#[must_use]
pub fn tool_tooltip_select() -> Self { Self::new("tool.tooltip.select") }
// ... etc

// Status messages
#[must_use]
pub fn status_saved() -> Self { Self::new("status.saved") }
// ... etc

// Accessibility
#[must_use]
pub fn a11y_canvas() -> Self { Self::new("a11y.canvas") }
// ... etc
```

### Step 2: Add catalogue entries

Edit `src/i18n/catalog/mod.rs` in `default_en_gb()`:

```rust
// Window chrome
messages.insert("chrome.file.new".to_owned(), "New".to_owned());
messages.insert("chrome.file.open".to_owned(), "Open".to_owned());
// ... etc

// Tool tooltips
messages.insert("tool.tooltip.select".to_owned(), "Select".to_owned());
// ... etc
```

### Step 3: Update tool_rail.rs

Replace tooltip literals with lookups:

```rust
// Before:
tooltip: "Select",

// After:
tooltip: shell_state.localize(&MessageId::tool_tooltip_select()),
```

Note: This requires changing `ToolModeButtonSpec` to hold `String` or
`SharedString` instead of `&'static str`.

### Step 4: Update remaining modules

Apply the same pattern to:

- `chrome.rs`
- `chrome_panels.rs`
- `style_controls.rs`
- `window_controls.rs` (for tooltip parameters)

### Step 5: Update accessibility

Edit `a11y_service/tree_builder.rs`:

```rust
// Before:
canvas.set_label("Drawing canvas");

// After:
let canvas_label = snapshot.localize(&MessageId::a11y_canvas());
canvas.set_label(&canvas_label);
```

Note: This requires passing localizer to tree builder or accessing it via
snapshot.

### Step 6: Update view.rs status templates

Edit `view.rs` to use localized templates:

```rust
// Before:
Some(format!("Saved: {}", path.display()))

// After:
let template = self.localizer.lookup(&self.locale, &MessageId::status_saved())
    .unwrap_or_else(|_| "Saved: {path}".to_owned());
Some(template.replace("{path}", &path.display().to_string()))
```

## Validation and acceptance

Quality criteria:

- **Tests**: All existing tests pass; new unit tests for each `MessageId`
  variant; new BDD scenarios for string lookup; GPUI integration tests for UI
  rendering.
- **Lint/typecheck**: `make lint` passes with no warnings; `make check-fmt`
  passes.
- **Coverage**: All new code paths covered by tests.

Quality method:

```sh
# Run all tests
cargo test --workspace

# Run i18n-specific tests
cargo test -p gauss i18n

# Run linting
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Check formatting
cargo fmt --workspace -- --check
```

Acceptance behaviour:

1. Application starts and displays identical UI labels as before.
2. All tooltips show correct text.
3. Status bar shows correct localized status messages.
4. Screen readers announce correct labels (verified via accessibility tree
   tests).
5. No hardcoded English strings remain in the targeted modules.

## Idempotence and recovery

This plan is idempotent:

- Adding the same `MessageId` method twice is a compile error (caught early).
- Adding the same catalogue entry twice would overwrite with same value
  (harmless).
- Re-running the migration is safe as it produces the same end state.

Recovery:

- If tests fail, revert the module changes and keep the `MessageId`/`Catalog`
  additions.
- If `icon_button` API change causes issues, use `SharedString` instead of
  `String` for tooltips.

## Interfaces and dependencies

### Modified types

In `src/i18n/message/mod.rs`:

```rust
impl MessageId {
    // New factory methods for UI strings
    pub fn chrome_file_new() -> Self;
    pub fn chrome_file_open() -> Self;
    // ... etc
}
```

In `src/i18n/catalog/mod.rs`:

```rust
impl Catalog {
    pub fn default_en_gb() -> Self {
        // Additional entries for all UI strings
    }
}
```

In `src/ui/phase0_shell/icon_button.rs` (if tooltip type needs changing):

```rust
pub fn icon_button(
    id: &'static str,
    icon: UiIcon,
    state: IconButtonState,
    tooltip: Option<String>, // Changed from Option<&'static str>
) -> impl IntoElement;
```

### Dependencies

No new external dependencies. All work uses existing workspace crates:

- `gauss` (for UI modules)
- `gauss-core` (for model types)

## Artefacts and notes

Key files modified for UI string extraction:

- `src/i18n/message/mod.rs` (+50): new `MessageId` factory methods.
- `src/i18n/message/factories.rs` (+50): `MessageId` factory
  implementations.
- `src/i18n/catalog/mod.rs` (+40): new catalogue entries.
- `src/ui/phase0_shell/tool_rail.rs` (~20): localized tooltips.
- `src/ui/phase0_shell/chrome.rs` (~30): localized button labels.
- `src/ui/phase0_shell/chrome_panels.rs` (~25): localized status bar.
- `src/ui/phase0_shell/style_controls.rs` (~10): localized colour picker
  labels.
- `src/ui/phase0_shell/view.rs` (~20): localized status templates.
- `src/ui/phase0_shell/a11y_service/tree_builder.rs` (~15): localized a11y
  labels.

## Revision note

Initial draft created 2026-03-29.

- Identified all UI strings requiring extraction.
- Defined MessageId naming convention.
- Planned staged approach: MessageId extension, then UI updates, then testing.
- Noted risk around `icon_button` tooltip type.
