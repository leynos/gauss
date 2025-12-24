# Design Proposal: AccessKit-Based Accessibility in GPUI

**Keeping GPUI on desktop** while delivering **full screen‑reader support
and limited‑mobility accessibility** (keyboard‑only, switch/voice control
friendly) is feasible – but it requires deliberate engineering. This plan
outlines how to integrate **AccessKit** into GPUI, implement proper semantics
(especially for text), and verify the result on NVDA/JAWS/Narrator (Windows),
VoiceOver (macOS), and Orca (Linux). The immediate focus is enabling
cross-platform **keyboard-only control** in **Gauss** (the Gauss vector
illustration application) via GPUI, with later phases adding visual aids and
enhanced screen-reader
support (e.g. for the upcoming Marrakesh Express client).

## Executive summary

- **Feasibility:** High, with caveats. AccessKit already provides adapters for
  **Windows (UI Automation)**, **macOS (NSAccessibility)**, and **Unix
  (AT‑SPI)**. It's designed so that immediate-mode toolkits (like GPUI) can
  expose an accessibility tree with **stable node IDs**
  ([1](https://accesskit.dev/how-it-works/#:~:text=One%20notable%20consequence%20of%20this,ID%20for%20each%20UI%20element))
  ([1](https://accesskit.dev/how-it-works/#:~:text=The%20current%20released%20platform%20adapters,support%20rich%20text%20or%20hypertext)).
  This approach allows GPUI to push updates to an AccessKit tree, with the
  adapters bridging to native Accessibility APIs on each OS.

- **Biggest technical risk:** **Text support.** AccessKit's adapters handle
  single-line and multi-line text controls today, but **rich text or
  hypertext** content is *not yet supported*
  ([1](https://accesskit.dev/how-it-works/#:~:text=The%20current%20released%20platform%20adapters,support%20rich%20text%20or%20hypertext)).
  If Gauss or Marrakesh Express requires full rich-text semantics (formatted
  text spans, embedded objects, code editor features), either limiting those
  features in v1 or contributing upstream to extend AccessKit will be required.

- **Why now:** Currently, GPUI-based apps lack practical screen-reader support
  on some platforms. For example, Zed (which uses GPUI) on Windows was reported
  as "absolutely inaccessible" by blind users
  ([2](https://github.com/zed-industries/zed/issues/41138#:~:text=Summary))
  ([2](https://github.com/zed-industries/zed/issues/41138#:~:text=Expected%20Behavior%3A%20The%20editor%20should,the%20folder%2C%20again%20nothing%20spoke)).
  Shipping Gauss without accessibility is not an option. This project brings
  GPUI to parity with competitor tools (e.g. VS Code's strong accessibility)
  and meets user needs from day one.

- **Tooling reality:** Accessibility won't come "for free" – the GPUI ↔ AccessKit
  bridge must be built, maintained, and thoroughly tested. OS-level accessibility
  inspection tools validate the implementation:
  **Accessibility Insights/Inspect** on Windows
  ([2](https://github.com/zed-industries/zed/issues/41138#:~:text=2,the%20editor%20to%20learn%20it))
  ([2](https://github.com/zed-industries/zed/issues/41138#:~:text=Expected%20Behavior%3A%20The%20editor%20should,the%20folder%2C%20again%20nothing%20spoke)),
  **Accessibility Inspector** on macOS, and **Accerciser** on Linux
  ([1](https://accesskit.dev/how-it-works/#:~:text=accessible%2C%20including%20support%20for%20both,support%20rich%20text%20or%20hypertext))
  ([1](https://accesskit.dev/how-it-works/#:~:text=,the%20current%20Windows%20accessibility%20API)).
  Continuous testing with screen readers (NVDA, JAWS, VoiceOver, Orca) is
  essential to catch issues early.

**Bottom line:** With focused integration, disciplined text handling, and a
rigorous test matrix, **screen reader + keyboard-only operation** on desktop
is achievable while preserving GPUI's performance advantages. Accessibility
becomes a first-class feature of the applications, not a post-hoc addition –
ensuring Gauss can be fully used by keyboard or alternative input, and laying
groundwork for future products.

## What "full" accessibility means (Acceptance Criteria)

### Screen-reader support

- **Windows (UIA):** All interactive widgets expose correct roles, names,
  states, and actions via UI Automation. Text controls implement the UIA
  **TextPattern** (and **TextEditPattern** for editors) – including caret
  movement, selection, word/line navigation, and text-change events
  ([2](https://github.com/zed-industries/zed/issues/41138#:~:text=Summary))
  ([2](https://github.com/zed-industries/zed/issues/41138#:~:text=3,the%20editor%20to%20learn%20it)).
  Events like `TextSelectionChanged` must be fired, and patterns like
  **Invoke**, **Value**, **Selection**, etc., as appropriate. Success is
  verified with **NVDA**, **JAWS**, and **Narrator**, using Microsoft's Inspect
  or Accessibility Insights tool to confirm the presence of the expected UIA
  properties and patterns
  ([2](https://github.com/zed-industries/zed/issues/41138#:~:text=Expected%20Behavior%3A%20The%20editor%20should,the%20folder%2C%20again%20nothing%20spoke))
  ([2](https://github.com/zed-industries/zed/issues/41138#:~:text=intuition%20and%20Windows%2FVS%20Code%20habits,the%20folder%2C%20again%20nothing%20spoke)).
  (Reference: Microsoft's UIA TextPattern documentation
  ([1](https://accesskit.dev/how-it-works/#:~:text=The%20current%20released%20platform%20adapters,support%20rich%20text%20or%20hypertext))
  ([1](https://accesskit.dev/how-it-works/#:~:text=accessible%2C%20including%20support%20for%20both,support%20rich%20text%20or%20hypertext))
  and implementation guidance
  ([3](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-about-text-and-textrange-patterns#:~:text=About%20the%20Text%20and%20TextRange,Services%20Framework%20%C2%B7%20Control%20Types)).)

- **macOS (NSAccessibility):** Expose proper **NSAccessibility roles, labels,
  and traits** for all controls. Ensure text fields and text views provide the
  attributes and notifications that VoiceOver expects (e.g.
  `NSAccessibilitySelectedTextRangesAttribute`,
  `NSAccessibilityValueChangedNotification`). Verify with **VoiceOver**
  (navigating via VO keys) and **Accessibility Inspector** on macOS
  ([1](https://accesskit.dev/how-it-works/#:~:text=accessible%2C%20including%20support%20for%20both,support%20rich%20text%20or%20hypertext))
  ([1](https://accesskit.dev/how-it-works/#:~:text=,the%20current%20Windows%20accessibility%20API))
  that elements are reachable and announced correctly. (See Apple's VoiceOver
  developer guide
  ([1](https://accesskit.dev/how-it-works/#:~:text=accessible%2C%20including%20support%20for%20both,support%20rich%20text%20or%20hypertext))
  for expected behavior.)

- **Linux (AT-SPI2):** All widgets expose correct **AT-SPI roles, states, and
  actions** on the accessibility bus. Text components implement the
  **AtspiText** interface (for caret and selection operations, text content
  retrieval, etc.)
  ([1](https://accesskit.dev/how-it-works/#:~:text=accessible%2C%20including%20support%20for%20both,support%20rich%20text%20or%20hypertext)).
  Test with **Orca** screen reader to ensure it can read focus, navigate text,
  and activate controls, and use **Accerciser** to inspect the accessibility
  tree.

**Note:** For text editing, the implementation should allow screen reader users
to review text by character, word, and line, and get feedback on cursor
position and selection changes. This is crucial for code or text editing
scenarios. Implementation will follow platform-specific guidance (UIA
TextPattern on Windows
([1](https://accesskit.dev/how-it-works/#:~:text=The%20current%20released%20platform%20adapters,support%20rich%20text%20or%20hypertext))
([3](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-about-text-and-textrange-patterns#:~:text=About%20the%20Text%20and%20TextRange,Services%20Framework%20%C2%B7%20Control%20Types)),
ATSPI Text on Linux
([1](https://accesskit.dev/how-it-works/#:~:text=accessible%2C%20including%20support%20for%20both,support%20rich%20text%20or%20hypertext)),
and the macOS accessibility text protocol) to get this right.

### Limited mobility / Keyboard and alternative input

- **Keyboard-only operation:** Every function of the UI can be accessed via
  keyboard alone. This means a **logical, cyclical focus order** through all
  interactive elements (matching visual navigation order) and no keyboard
  traps. Users can perform all commands using key combinations (or macro of key
  combos). There should be a visible focus indicator on the focused element at
  all times (meeting Web Content Accessibility Guidelines (WCAG) 2.1 Focus
  Visible criteria
  ([2](https://github.com/zed-industries/zed/issues/41138#:~:text=2,the%20editor%20to%20learn%20it))).
  Complex canvas interactions in Gauss will be mapped to keyboard controls (e.g.
  arrow keys or WASD to nudge objects, keyboard shortcuts for drawing tools,
  etc.), so that a designer can create and edit illustrations without a mouse.
  This aligns with WCAG 2.1 Success Criterion **2.1.1 Keyboard** and **2.4.3
  Focus Order**
  ([2](https://github.com/zed-industries/zed/issues/41138#:~:text=3,the%20editor%20to%20learn%20it))
  ([4](https://zed.dev/blog/gpui-ownership#:~:text=Ownership%20and%20data%20flow%20in,an%20old%20version%20of%20GPUI)),
  ensuring no functionality is mouse-exclusive.

- **Switch and voice control friendly:** All actionable UI elements expose
  **programmatic actions** that can be triggered by external assistive tools.
  For example, buttons should support the **Invoke** action (so a switch
  control or voice command can activate them), sliders support **SetValue**,
  and list items support **Selection**. On Windows, this corresponds to
  implementing the appropriate UIA Control Patterns (InvokePattern,
  ValuePattern, SelectionItem, etc.)
  ([2](https://github.com/zed-industries/zed/issues/41138#:~:text=Summary)).
  On macOS, it means implementing the `accessibilityPerformPress` or similar
  methods, and on AT-SPI, providing atspi `Action` interfaces for clickable
  items. The goal is compatibility with voice control software and hardware
  switches that emulate keystrokes or direct API calls. (For reference,
  Microsoft's guide on UI Automation control patterns for text and value
  provides insight
  ([5](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-implementingtextandtextrange#:~:text=Learn%20learn,1)).)

- **Low-vision accommodations:** Provide a high-contrast UI theme and scalable
  UI elements. Contrast-enhanced colour tokens should be defined (ensuring text
  meets at least WCAG AA contrast ratio against its background) and honor
  system high-contrast settings when possible. Users should be able to increase
  UI scale (text and control size) without breaking layout. Also, ensure no
  critical information is conveyed by colour alone (provide shapes or text
  labels as needed). While this is more of a UI design task, it ties into
  accessibility standards and will be documented as part of the compliance
  effort (relates to WCAG 2.1 **1.4.3 Contrast** and others).

**Note:** If formal compliance is needed (e.g. for EU government adoption),
alignment with **EN 301 549** (the European accessibility standard for ICT
products, which extends WCAG to non-web software) is recommended
([2](https://github.com/zed-industries/zed/issues/41138#:~:text=Expected%20Behavior%3A%20The%20editor%20should,the%20folder%2C%20again%20nothing%20spoke)).
This essentially means meeting WCAG 2.x success criteria in a desktop context.
The acceptance criteria above are designed with those guidelines in mind.

## Proposed Implementation Steps and Impact

Below is a phased plan focusing on scope, potential risks, and expected
outcomes of each step.

### 1) Adopt AccessKit early – GPUI bridge to native accessibility

**What changes:** AccessKit will be integrated at the core of GPUI's rendering
loop. In GPUI's platform-specific code (for Windows, macOS, Linux), an
**AccessKit adapter** is instantiated for the active window
(`accesskit_windows::Adapter`, `accesskit_macos::Adapter`,
`accesskit_unix::Adapter`). GPUI will maintain an **accessibility tree**
parallel to its UI hierarchy, where each UI element corresponds to an
**AccessKit `Node`** with a stable ID and properties (role, name/label, value,
state, relationships, etc.). Every time the UI changes (layout updates, focus
moves, elements added/removed), an incremental **TreeUpdate** is sent to the
adapter. An **action handler** callback is also registered with the adapter, so
when an assistive technology requests an action (e.g. "press button X" or
"focus text field Y"), AccessKit calls into the handler to perform that
action in GPUI.

**Why it's tractable:** AccessKit is explicitly designed to make immediate-mode
UIs accessible, as long as stable IDs can be provided for elements
([1](https://accesskit.dev/how-it-works/#:~:text=One%20notable%20consequence%20of%20this,ID%20for%20each%20UI%20element))
([1](https://accesskit.dev/how-it-works/#:~:text=The%20current%20released%20platform%20adapters,support%20rich%20text%20or%20hypertext)).
GPUI's architecture (all UI entities owned by a central `App` context
([4](https://zed.dev/blog/gpui-ownership#:~:text=After%20initial%20attempts%20to%20use,and%20interact%20with%20other%20entities))
([4](https://zed.dev/blog/gpui-ownership#:~:text=UI%20framework%2C%20GPUI,and%20interact%20with%20other%20entities)))
means stable `NodeId`s can be generated for each component (for example, using a
combination of entity ID and type). AccessKit already handles the hard part:
translating the `Node` tree and events to each platform's API. There is no need
to implement UIA, NSAccessibility, or AT-SPI from scratch – data is fed to
AccessKit, and it updates the OS accordingly. The AccessKit adapters for
Windows, Mac, and Linux are at feature parity and actively maintained
([1](https://accesskit.dev/how-it-works/#:~:text=The%20current%20released%20platform%20adapters,support%20rich%20text%20or%20hypertext)).

**Notable risks:**

- **Custom windowing:** GPUI doesn't use the typical `winit` loop (it has its
  own windowing). This means the `accesskit_winit` crate cannot be used
  out-of-the-box
  ([1](https://accesskit.dev/how-it-works/#:~:text=,the%20current%20Windows%20accessibility%20API));
  instead, the lower-level adapter APIs must be called directly. On Windows, for
  example, the `WM_GETOBJECT` message must be handled to provide the
  `IAccessible` interface pointer to assistive tech. The AccessKit Windows
  adapter provides a helper for this (`Adapter::handle_wm_getobject`) which
  must be called when that message is received
  ([1](https://accesskit.dev/how-it-works/#:~:text=Platform%20adapters))
  ([1](https://accesskit.dev/how-it-works/#:~:text=,the%20current%20Windows%20accessibility%20API)).
  Additionally, `Adapter::update_window_focus_state` needs to be called when the
  window gains or loses focus, so the accessibility framework knows which window
  is active. These details require careful integration with GPUI's Win32 message
  loop, Cocoa event loop, and X11/Wayland events respectively.

- **Threading and init order:** On Windows, UI Automation expects the provider
  (the app) to be ready to supply the accessibility tree **before returning
  from `WM_GETOBJECT`**. That means the adapter initialization has to happen at
  the right time. The AccessKit adapter will likely be initialized when the
  window is created (or first shown) with a cached tree ready by the time a
  screen reader first queries it. The adapter provides `update_if_active` and
  `update` methods which only take effect when a screen reader is actually
  listening (avoiding overhead if no AT is in use)
  ([1](https://accesskit.dev/how-it-works/#:~:text=Platform%20adapters))
  ([1](https://accesskit.dev/how-it-works/#:~:text=,the%20current%20Windows%20accessibility%20API)).
  These must be used correctly to avoid missing events.

**Proof of concept:** A minimal window in GPUI is created, a few controls are
added (e.g. a button and a text field), and the following is verified:

- On **Windows**, running a screen reader (NVDA) and using Inspect.exe shows
  the controls in the UIA tree with correct names/roles. Pressing the button
  via the screen reader's command (or via Accessibility Insights' "invoke"
  action) activates the GPUI button. `WM_GETOBJECT` is handled by returning the
  AccessKit-provided object. Calling `adapter.update_if_active(tree_update)` on
  any UI change results in events being raised (like `UIA_Event_FocusChanged`).

- On **macOS**, enabling VoiceOver should allow navigation to the GPUI window's
  controls (e.g. VO+Right moves focus to the button, and it announces properly).
  The **Accessibility Inspector** on Mac should list the custom window's
  elements in the hierarchy with proper roles. Subclassing `NSView` or using
  `NSAccessibilityUnignoredDescendant` ties the view to AccessKit's adapter (the
  adapter provides a simple way to hook into NSAccessibility).

- On **Linux**, running Orca and tabbing through the UI should have Orca speak
  the control labels. **Accerciser** should show the app's accessibility tree
  with the expected structure.

**Sizing:** *Medium-to-Large.* This is foundational work across three
platforms. The bulk is in setting up the translation layer (GPUI elements →
AccessKit Nodes) and handling platform events. Once the core is in place,
adding more controls is incremental.

### 2) Prioritize text input and editing support (caret, selection, range ops)

**What changes:** Text boxes and code editor fields are often the most complex
accessibility elements. The **text interfaces** must be implemented in the
accessibility tree:

- Represent the text content of each text field or editor in the `Node` (likely
  via the `Node::value` for plain text, and perhaps a separate role like
  `EditableText`). For multiline editors, the text content might be designated
  as child nodes or use the appropriate role (like `Paragraph` children, or a
  generic `Document` role containing text).

- Track the **caret (cursor) position** and any **selected text range**.
  AccessKit allows specifying a `focus` on a particular node (the active text
  field) and can include the selection offsets in the update.

- Implement **text navigation actions**: screen readers will request operations
  like "move cursor by character/word/line" or "select text range". The
  AccessKit action handler needs to interpret generic
  `ActionRequest::TextNavigation` or similar (if provided) and translate that
  into GPUI operations (like moving the caret in the underlying text model). On
  Windows, this corresponds to supporting the UIA **TextPattern** methods for
  moving the text caret and selecting text
  ([1](https://accesskit.dev/how-it-works/#:~:text=The%20current%20released%20platform%20adapters,support%20rich%20text%20or%20hypertext)).
  On Linux, the AT-SPI `Text` interface has methods like `SetCaretOffset`,
  `SetSelection`, etc., which the Unix adapter will invoke on the action
  handler.

- Fire **text events**: e.g., when text is inserted or deleted in an editor,
  events should be sent, so the screen reader announces changes (UIA
  `Text_TextChangedEvent`, AT-SPI `TextChanged` signal, etc.). AccessKit might
  handle some of this if the node's text value and selection are updated
  appropriately, but the right update functions must be called, so the adapters
  emit incremental events (rather than reading the whole text each time).

**Risk to call out:** As noted, **rich text support is limited** in AccessKit
currently
([1](https://accesskit.dev/how-it-works/#:~:text=The%20current%20released%20platform%20adapters,support%20rich%20text%20or%20hypertext))
([1](https://accesskit.dev/how-it-works/#:~:text=accessible%2C%20including%20support%20for%20both,support%20rich%20text%20or%20hypertext)).
This means if Gauss or the code editor has formatted text (different styles, or
embedded controls within text), the adapters might not fully expose that
structure. For phase 1, restricting to plain text or simple formatting in
accessible content is likely. If the use-case absolutely needs rich text (say,
styled text or inline images in a future Marrakesh Express chat client), either
implementing a simplified accessibility presentation (e.g., expose just plain
text to the screen reader) or engaging with the AccessKit project to extend
support should be planned. This is a known gap; AccessKit's updates on this
front should be tracked. Another risk is performance: long documents (like a
large code file open in Gauss's editor, if any) should not produce overly large
events. Testing with large text content is needed to ensure updates are
efficient (AccessKit's diff mechanism should help here).

**Proof points:**

- Use **Accessibility Insights (Windows)** in **Inspect mode** to verify that a
  text box in the app supports the **Text Pattern**. Insights will list
  supported patterns for a focused element; *Value* and *Text* are expected
  (and *TextEdit* if editable) for a multiline editor
  ([2](https://github.com/zed-industries/zed/issues/41138#:~:text=Expected%20Behavior%3A%20The%20editor%20should,the%20folder%2C%20again%20nothing%20spoke)).
  Inspect.exe can also be used to retrieve the text via UIA and ensure it
  matches.

- With a screen reader, test typical text interactions: type in a text field
  and have the screen reader echo characters or words; navigate
  character-by-character (arrow keys) and word-by-word (Ctrl+Arrow on Windows,
  Option+Arrow on Mac) and ensure the screen reader reads the letters/words.
  Try selecting text with Shift+Arrow and confirm the screen reader announces
  selection. Delete text and ensure it says "deleted" etc., if the screen
  reader normally does (some of this is reader-specific). These interactions
  should behave similarly to standard text controls.

- For a multiline text editor (if Gauss has a code editor or text area), test
  that moving up/down lines is announced correctly (the screen reader should
  read the new line or say "blank" for empty lines). Also test that when focus
  moves away and back, the screen reader can read the current line or the
  entire content upon request.

- Verify on macOS with VoiceOver: VoiceOver has specific rotor settings for
  text (like moving by words/lines). The text fields should support those.
  Similarly on Orca, ensure the Orca "SayAll" (read from cursor to end) works
  in the text control.

**Sizing:** *Large* for full-featured text editing (especially if code editor
or rich text). For basic single-line inputs and simple multiline areas,
*Medium*. Baseline text support will be implemented in phase 1, and if
rich-text or advanced editing is needed, that may be a separate subproject.

### 3) Audit and improve colour contrast & theming for visibility

**What changes:** This step is about **visual accessibility** (particularly for
low-vision users or those with contrast sensitivity):

- Define a set of **high-contrast theme tokens**. A High Contrast mode should be
  added to the theme system (in addition to the default Light/Dark). In High
  Contrast, text and important UI elements use colours that maximize contrast
  with the background (e.g., white on black or black on yellow, depending on
  user preference or system settings). All text should meet at least a 4.5:1
  contrast ratio against its background (WCAG 2.1 AA level).

- Ensure **focus indicators** are very visible. This might mean using a thick
  outline or an obvious highlight on the focused element, not just a subtle
  shadow. For high contrast mode, the focus ring might be a distinct colour
  (like yellow) that stands out on black.

- Support **UI scaling**: Users who have trouble seeing small text should be
  able to scale up the interface. GPUI should already handle different
  DPI/resolutions; a user preference slider for UI scale can be added or the
  OS text scaling setting respected if available. All custom drawing (in
  Gauss's canvas for example) should either scale with it or be at least
  unaffected by larger system font settings.

- If possible, detect OS-level high-contrast or dark mode settings and map them
  to the application themes. On Windows, for instance, a query can determine if
  a high-contrast theme is active and automatically switch or adjust colours.
  On macOS, the "Increase Contrast" and "Differentiate Without Colour" settings
  could inform some adjustments (like adding shapes or labels in addition to
  colour coding).

This step is relatively self-contained in terms of code (mostly styling), but
it's crucial for compliance and user comfort.

**Proof points:**

- Use a colour contrast analyser (there's one in Accessibility Insights and
  browser dev tools) to verify that all text-colour vs background-colour
  combinations in the UI are >= 4.5:1 (for normal text) or 3:1 for large
  text/icons. These contrast ratios should be documented for key UI components.

- Manually enable high-contrast mode on each OS and observe the app: On
  Windows, turn on High Contrast (via Ease of Access settings) and confirm that
  either the app switches to its high-contrast theme or at least remains usable
  (Windows might override system colours if allowed, but since everything is
  custom-drawn, manually applying a theme is likely needed). On macOS,
  enable "Increase contrast" and "Reduce transparency" and check that the UI
  still looks correct (no reliance on transparency or low-contrast elements).
  On Linux, test with a high-contrast GTK theme if applicable (though the app
  might not automatically adopt it, an in-app toggle can still be provided).

- Ensure that when users increase system font scaling (or with an application
  zoom preference), text and controls scale appropriately without clipping or
  overlapping. All interactive targets should remain accessible (e.g., if
  someone sets system DPI to 150%, hit regions should also scale up).

**Sizing:** *Small to Medium.* Mostly adjusting styles and providing a new
theme variant. The main workload is verifying each component under those
conditions and possibly tweaking a few hard-coded visuals (for example,
replacing a faint gray border with a thicker one in high contrast mode).

### 4) Test with real assistive technologies (not just inspectors)

**What changes:** A **test matrix and schedule** will be created for iterative
accessibility testing:

- Identify key user journeys in Gauss (and later Marrakesh Express) that need
  verification: e.g., **launch app, open a document, draw an object, edit its
  properties, save/export** – all using keyboard + screen reader. Also cover
  **navigating menus**, using **toolbars/palettes**, and any **dialogs** or
  notifications.

- For each platform (Win/Mac/Linux), script a sequence of keyboard actions to
  accomplish these tasks. Step-by-step test cases may be written down (like
  "Press Alt+F to open File menu, arrow down to 'Open...', press Enter, ...").
  These will be used by QA or developers with screen readers turned on.

- Use multiple screen readers where available: On Windows, test with NVDA
  (free, popular), JAWS (commercial, widely used by power users), and Narrator
  (built-in, to catch any UIA discrepancies). On macOS, test with VoiceOver
  (built-in). On Linux, test with Orca.

- Also test alternative input: try Windows Speech Recognition or Dragon if
  available to ensure voice commands can activate UI controls (this often works
  if UIA is implemented correctly). If possible, test with a Switch interface
  (or at least the concept, e.g., Windows has a built-in "Scan Mode" that
  tab-steps through UI).

These tests will be incorporated into the development cycle – e.g., running
through a basic smoke test on every major UI change, and planning a dedicated
accessibility testing session for each release.

**Proof points:**

- Maintain **transcripts or reports** from screen reader usage. For example,
  when testing NVDA, turn on key echo and speech viewer to capture what NVDA
  announces at each step. Ensure that the announcements make sense (e.g., when
  focusing a button, NVDA says "Open (button)" if the button's label is "Open";
  when toggling a toolbar toggle, it says "Bold, pressed, toggle button",
  etc.). These outputs will be collected and used to verify nothing broke
  later.

- Use **Accessibility Insights for Windows (AIWin)** in its **Assessment** mode
  or the FastPass automated checks
  ([2](https://github.com/zed-industries/zed/issues/41138#:~:text=2,the%20editor%20to%20learn%20it))
  regularly. This can catch missing labels, contrast issues, and missing
  keyboard support. For example, AIWin's **FastPass** should report no
  **keyboard trap** issues, no missing names for controls, and no inappropriate
  roles. AIWin's CLI can be integrated into the CI pipeline for Windows builds
  to catch regressions automatically.

- On macOS, use the **Accessibility Inspector** to examine the UI hierarchy at
  runtime. Ensure that every focusable item has an `AXDescription` (label) and
  the roles look correct. Xcode's Accessibility Audit can also be used. On
  Linux, use **Accerciser** to introspect the app's accessibility tree,
  ensuring all expected interfaces are present (the Accerciser plugin can
  highlight if an object doesn't implement the text interface when it should,
  etc.).

- As a final verification, have an **external accessibility expert or an actual
  screen reader user** exercise Gauss and give feedback. They might find
  usability issues that technical checks miss (like confusing focus order or
  verbosity of announcements). Their feedback should be treated as high
  priority to fix.

**Sizing:** *Ongoing.* Initial setup of test scenarios is Medium effort, but
testing is an ongoing commitment. Time may be allocated each sprint for
accessibility regression testing.

### 5) Document an accessibility support matrix (for components and features)

**What changes:** To ensure clarity and track coverage, an **accessibility
support matrix** for GPUI components and Gauss features will be created:

- List each UI component (Button, Toggle, Slider, TextField, List,
  Canvas/Artboard, etc.) and detail how it's made accessible: **Role**,
  **Accessible Name** (and whether it comes from label, tooltip, etc.),
  **State/Value** (e.g., toggle state or slider value range), **Keyboard
  Interaction** (e.g., "Tab focuses, Space/Enter activates button, arrow keys
  change slider", etc.), **Screen Reader Behavior** (what is announced when
  focused or when its value changes), and any **Live Region** usage (for
  dynamic announcements like notifications).

- Also list overall UI flows (like the main toolbar, dialogs, the canvas area)
  and note any specific accessibility notes (for example, "Canvas: treated as a
  graphics document, objects can be navigated via Tab or arrow keys when focus
  is in canvas; provides descriptive names for objects like 'Rectangle element'
  etc., and supports keyboard moving/resizing via arrow keys or other
  shortcuts").

- Highlight any **known gaps** or exceptions. For instance, if in v1 the canvas
  drawing area is not fully accessible (perhaps complex freehand drawing has no
  keyboard equivalent), be transparent about it and perhaps suggest workarounds
  or plans.

This document will live in the repo (e.g., as `ACCESSIBILITY.md`) and be
updated as things improve. It's both for internal use (engineers and QA refer
to it) and external (it shows the project's commitment to accessibility to
users or auditors).

**Proof points:**

- The existence of this document itself is a proof: it means each component has
  been thought through. Enterprise customers or compliance officers can be
  pointed to this matrix to answer questions like "Can a user do X without a
  mouse?" or "Will a screen reader announce Y for this widget?".

- Automated checks for the items in the matrix should also be incorporated where
  possible. For example, if the matrix says "All buttons have accessible
  names," a unit test could run through the UI widget registry and ensure every
  button has a non-empty label property in the accessibility tree (this could
  be a development-time assert). Similarly, if keyboard shortcuts are listed
  for all actions, ensure the documentation of shortcuts is up-to-date and
  perhaps accessible via an in-app cheat sheet.

**Sizing:** *Small* for initial documentation (one engineer can draft it from
the implementation). *Medium* to keep it updated as features evolve (this
becomes part of the definition-of-done for new UI components: update the a11y
matrix).

## Architecture notes and gotchas

- **Stable Node IDs:** Because GPUI is an immediate-mode framework (the UI is
  redrawn each frame), each accessibility node must have a **persistent
  identifier** across frames
  ([1](https://accesskit.dev/how-it-works/#:~:text=One%20notable%20consequence%20of%20this,ID%20for%20each%20UI%20element))
  ([1](https://accesskit.dev/how-it-works/#:~:text=The%20current%20released%20platform%20adapters,support%20rich%20text%20or%20hypertext)).
  The `NodeId` will likely be tied to the underlying GPUI `Entity` (since an
  `Entity<Counter>` or similar persists in the App's entity map
  ([4](https://zed.dev/blog/gpui-ownership#:~:text=After%20initial%20attempts%20to%20use,and%20interact%20with%20other%20entities))
  ([4](https://zed.dev/blog/gpui-ownership#:~:text=UI%20framework%2C%20GPUI,and%20interact%20with%20other%20entities))).
  AccessKit doesn't keep the whole UI tree; it relies on updates being sent. If
  an ID is accidentally changed when nothing else changed, the adapters might
  treat it as a removal/addition, which can confuse screen readers (e.g., focus
  might get lost). A consistent ID scheme should be implemented (perhaps using
  the pointer or index of the element in the GPUI tree, or a hash of its stable
  path).

- **Focus management and window activation:** GPUI's internal focus system
  needs to be hooked to AccessKit. When the user presses Tab or otherwise moves
  focus in GPUI, `adapter.update_focus(focused_node_id)` should be called (or
  include the focused node in TreeUpdate). On Windows, call
  `update_window_focus_state(true/false)` when the app window gains/loses focus
  (so the screen reader knows to switch context). On macOS, it might be
  necessary to override `NSWindow.makeFirstResponder` or similar to notify the
  adapter of focus shifts. These ensure that, for example, when the window is
  activated, the screen reader knows where to resume, and when it's inactive,
  unnecessary events aren't sent.

- **Performance considerations:** AccessKit is designed to handle frequent
  updates efficiently by sending only diffs. Accessibility updates should be
  batched to coincide with frame updates or state changes, rather than calling
  on every minor event individually. For instance, if 10 buttons enable/disable
  at once, push one update with all their state changes. The platform APIs can
  be chatty, but coalescing events helps. Computing accessibility info can also
  be avoided if no assistive tech is active – AccessKit adapters have an
  `is_active()` to check, so that if (for example) no screen reader is running,
  building the tree can be skipped to save cycles. However, `WM_GETOBJECT`
  etc., must always be handled in case one starts mid-run. In practice, this
  overhead is low (tree updates are essentially serialization of some node
  structs), and testing will ensure minimal impact on frame rates.

- **Custom drawing and hit testing:** Gauss's canvas is highly custom (for
  vector graphics). Deciding how to expose the canvas content to accessibility
  is needed. Initially, the canvas might be treated as a single component
  (role=Graphic or Canvas) with an accessible name (e.g., "Drawing area").
  Eventually, if keyboard navigation of objects on the canvas is allowed, each
  shape could be exposed as an accessible object in a hierarchy (like objects
  as children of the canvas node). This may require dynamic generation of nodes
  for drawing objects and possibly describing them (e.g., "Blue circle at (x,y)
  radius r"). This is advanced and could be a later phase; for now, the focus
  is on the UI chrome and ensuring at least the canvas can be focused, and its
  purpose conveyed (so a blind user isn't left wondering what that region is).

- **No free lunch with `accesskit_winit`:** Since GPUI doesn't use Winit's
  event loop (it has its own windowing and rendering), AccessKit will be
  integrated **manually**. The `accesskit_winit` adapter is a convenience for
  Winit apps; instead, the lower-level adapter APIs must be used directly. On
  Windows, handle the `WM_GETOBJECT` message, etc. The good news is AccessKit's
  documentation and examples include using the adapters directly (for example,
  they have a pure Win32 example and an SDL example in C). These can be
  referred to for correct implementation. (See also the discussion in the Winit
  issue about GPUI: GPUI has its own windowing system
  ([6](https://www.reddit.com/r/rust/comments/19fle6w/gpui_ui_framework_from_the_makers_of_zed/#:~:text=GPUI%3A%20UI%20Framework%20from%20the,to%20be%20depending%20on%20Metal)),
  which reinforces that the integration must be handled directly, not expecting
  Winit to do it.)

## Example: Windows adapter integration snippet

To illustrate, here's a simplified approach for integrating the Windows adapter
in the GPUI window procedure (pseudo-Rust code):

```rust
use accesskit::{ActionRequest, NodeId, TreeUpdate};
use accesskit_windows::Adapter;
use windows::Win32::UI::WindowsAndMessaging::{WM_GETOBJECT, GetFocus};

struct AccessibilityState {
    adapter: Option<Adapter>,
}

// Called when creating a window, or on first enable of a11y
fn initialize_accesskit(hwnd: isize, initial_tree: TreeUpdate) -> AccessibilityState {
    let adapter = Adapter::new(
        accesskit_windows::HWND(hwnd),
        // specify whether this window is focused initially:
        hwnd == unsafe { GetFocus() } as isize,
        // Action handler: closure called on events like "invoke" or "set value"
        Box::new(move |request: ActionRequest| {
            // TODO: translate ActionRequest (e.g., focus, invoke) into GPUI actions
            handle_action_request(request);
        }),
    );
    // Provide the initial full tree to the adapter
    adapter.initialize(initial_tree);
    AccessibilityState { adapter: Some(adapter) }
}

// In the Win32 window proc:
match msg {
    WM_GETOBJECT => {
        if let Some(acc_state) = &mut self.accessibility {
            // Let AccessKit handle the accessibility object request:
            if let Some(result) = acc_state.adapter
                   .as_mut()
                   .and_then(|ad| ad.handle_wm_getobject(wparam, lparam))
            {
                return result;
            }
        }
        // ... otherwise, default processing
    }
    // ... other messages ...
}
```

In practice, an update function would be called whenever the UI changes, for
example on UI state change:

```rust
fn update_accessibility(&mut self, update: TreeUpdate) {
    if let Some(adapter) = self.accessibility.adapter.as_mut() {
        if let Some(events) = adapter.update_if_active(update) {
            events.raise(); // dispatch queued events to OS (must be on UI thread)
        }
    }
}
```

This code is a sketch; the actual implementation will follow AccessKit's API
closely (see `Adapter::new`, `Adapter::initialize`, `handle_wm_getobject`,
`update_if_active`, etc. in AccessKit's docs
([1](https://accesskit.dev/how-it-works/#:~:text=Platform%20adapters))
([1](https://accesskit.dev/how-it-works/#:~:text=,the%20current%20Windows%20accessibility%20API))).
The macOS adapter (`accesskit_macos::Adapter`) and Linux adapter
(`accesskit_unix::Adapter`) have analogous methods for hooking into their
respective event loops. For instance, on macOS an `NSView` subclass might be
created for the GPUI content view and `accesskit_macos::Adapter::new` used with
that view; on Linux, the adapter is initialized with the window handle, and it
registers on D-Bus to communicate with AT-SPI
([1](https://accesskit.dev/how-it-works/#:~:text=,the%20current%20Windows%20accessibility%20API)).

*Takeaway:* The integration involves initializing the adapter with the window,
providing an initial accessibility tree (with all nodes and their properties),
handling OS requests (like `WM_GETOBJECT` or the Cocoa accessibility
callbacks), and sending incremental updates whenever the UI changes state.

## Risks and Challenges to Mitigate

- **Rich text editor semantics:** As mentioned, if Gauss's text editing or
  Marrakesh Express's messaging require rich text (multiple styles, inline
  graphics), **AccessKit's current limitations** mean the content might not be
  presented perfectly to ATs
  ([1](https://accesskit.dev/how-it-works/#:~:text=The%20current%20released%20platform%20adapters,support%20rich%20text%20or%20hypertext)).
  Either simplifying the exposed structure (e.g., expose plain text alternative
  for screen readers) or contributing to AccessKit's development should be
  planned. This might involve working on features like an HTML-like
  accessibility tree for formatted text or ensuring code editors expose line
  numbers and syntax as needed (similar to how VS Code's accessibility works).
  It's a non-trivial extension, so best addressed early if needed.

- **Windows-first bias:** Windows screen reader users are often the largest
  segment and very vocal (see the Zed issue
  ([2](https://github.com/zed-industries/zed/issues/41138#:~:text=Summary))
  ([2](https://github.com/zed-industries/zed/issues/41138#:~:text=3,the%20editor%20to%20learn%20it))).
  A lot of feedback from them is anticipated. It's critical that Windows
  support is as complete as possible – that means all UIA patterns implemented
  where appropriate, no missing keyboard focuses, and thorough testing with
  NVDA and JAWS. If something works in NVDA but not JAWS (which can happen due
  to JAWS-specific quirks), investigation is needed since enterprise users
  often use JAWS. Essentially, treat **UIA compliance** as a gate for release.
  The **UIA Verify** tool or other automated UIA tests might even be run on the
  app to ensure no required patterns are missing.

- **Cross-platform quirks:** Each OS has its unique behaviours. For example, on
  macOS, VoiceOver might automatically group certain items or require an
  `AXGroup` for composite controls – the tree structure might need adjustment
  to accommodate that. On Linux, the AT-SPI might require certain events (like
  `Object:StateChanged:focused`) to be emitted in a specific order relative to
  `Object:ActiveDescendantChanged` for things like list selections. The
  AccessKit adapters abstract a lot of this, but not all. Time should be
  budgeted for debugging platform-specific issues. Having a diverse test set
  (as per step 4) will help catch these. Upstream AccessKit issues should also
  be monitored for any platform bugs and the AccessKit version updated as fixes
  come out.

- **Maintenance and performance:** Once integrated, accessibility code must be
  maintained alongside feature changes. Developers adding a new widget or
  feature need to update the accessibility tree accordingly. The team should be
  trained on how to do this (perhaps via the support matrix and code examples).
  Also, although AccessKit is efficient, if someone inadvertently generates
  tons of accessibility events (e.g., updating a node property every frame), it
  could slow things down or flood assistive tech. Code review should catch
  misuse (like not marking purely decorative elements as such, or not spamming
  events).

## "Done means done": Acceptance Tests

The accessibility work will be considered "done" (for phase 1) only when it
passes the following tests:

- **Automated smoke tests (CI-level checks):**

- **Windows:** Run Accessibility Insights for Windows **FastPass** on the main
  windows of Gauss. The FastPass automated checks should report **no errors**
  in the categories of: Name, Role, Value for controls; keyboard traps; colour
  contrast (if the contrast rules are included); focus order (AI can flag if
  focus moves in an illogical order). **Inspect** should also be used manually
  to ensure key controls expose the correct UIA control patterns (e.g., a
  slider control should show a RangeValue pattern, a text box shows a Text
  pattern, etc.).

- **macOS:** Use Apple's Accessibility Inspector to verify that all interactive
  views have an appropriate `AXRole` and `AXLabel`. Run the built-in macOS
  **Accessibility Audit** (part of Xcode's Developer Tools) to catch obvious
  issues. Also test basic VoiceOver navigation: launch VoiceOver and use the
  keyboard (VO+Arrow keys) to move through the app's UI – ensure every
  focusable item is reachable and announced.

- **Linux:** Use **Accerciser** to navigate the accessibility hierarchy of the
  app. Check that each focusable widget has the expected role and states. Use
  Orca in a basic way (tab through, use Orca's reading commands) to see that
  nothing crashes and announcements are made. Continuous integration will also
  be relied upon to run a headless test of the accessibility tree if possible
  (for instance, using AccessKit's ability to dump the tree, the output could
  be compared against expected output for a known UI state).

- **Full scenario manual tests:**

End-to-end tasks are scripted and performed *purely via keyboard and screen
reader*. Example scenario for Gauss: **"Create and save a simple drawing"** –
The tester will: launch Gauss, create a new canvas (via menu or shortcut), draw
a shape (using keyboard to select shape tool and place a shape of default size,
or using an accessible shape insertion command), change the shape's colour or
properties (navigating the properties panel), save the file (via keyboard).
While doing this, the screen reader should announce each action (e.g., when
selecting the rectangle tool, it might say "Rectangle tool selected, press
Enter to place" if such feedback is implemented; when a shape is inserted, a
live region might announce "Shape added"). The success criterion is that a
blind or motor-impaired user *can accomplish the task without sighted
assistance*. Similar scenarios for other key workflows (opening an existing
drawing, exporting, using advanced features) will be performed and blockers
addressed. Any time a step cannot be done via keyboard or isn't spoken, it will
be addressed before considering the feature complete.

- **Contrast and scaling checks:**

Verify that in high-contrast mode, or with a custom high-contrast theme, all UI
elements are still distinguishable. Use a colour contrast tool on all UI text.
Increase the system font scale (or the app's zoom) to, say, 150% and ensure the
UI is still usable (text isn't cut off, icons scale or have appropriate
alternatives like text labels instead of relying on tiny icons). Focus
indicator visibility is tested under various conditions (e.g., window not
focused vs focused, high contrast on/off).

- **Documentation and knowledge transfer:**

The accessibility support matrix document is reviewed and accepted by the team.
It is considered a "test" that an engineer can read the matrix and understand
how to make any new component accessible, following the patterns established.
Additionally, a brief training or demo for the whole team is planned (so
design, QA, and dev know how to use the screen reader to test their features).
When everyone is comfortable that accessibility can be maintained going
forward, that's a sign the implementation is truly integrated into the process
(not just a one-time fix).

Passing all the above means GPUI (and thus Gauss) can confidently be said to
support **screen reader users, keyboard-only users, and low-vision users** in
its core functionality. It sets the stage for future improvements (like more
advanced ARIA roles for canvas content, or support in other platforms like a
possible web version via AccessKit's upcoming web adapter, etc.).

## Dependencies & References

This plan builds on existing technologies and guidelines:

- **AccessKit library:** The core of the solution. See the AccessKit project's
  README and documentation
  ([1](https://accesskit.dev/how-it-works/#:~:text=Platform%20adapters))
  ([1](https://accesskit.dev/how-it-works/#:~:text=The%20current%20released%20platform%20adapters,support%20rich%20text%20or%20hypertext)),
  which explain the immediate-mode friendly design, stable ID requirement, and
  currently supported features (including text, with the noted omission of rich
  text/hypertext). AccessKit provides the multi-platform adapters needed (UIA,
  NSAccessibility, AT-SPI). It's open source and contributions can be made if
  needed
  ([1](https://accesskit.dev/how-it-works/#:~:text=,the%20current%20Windows%20accessibility%20API)).

- **GPUI architecture:** Understanding GPUI's entity/component system is key to
  integrating AccessKit. Refer to Zed's blog post "Ownership and data flow in
  GPUI"
  ([4](https://zed.dev/blog/gpui-ownership#:~:text=After%20initial%20attempts%20to%20use,and%20interact%20with%20other%20entities))
  ([4](https://zed.dev/blog/gpui-ownership#:~:text=UI%20framework%2C%20GPUI,and%20interact%20with%20other%20entities))
  for an overview of how all UI state in GPUI is owned by a central `App`. This
  informs how stable IDs are generated and where the accessibility updates hook
  in (likely at the App or Window level).

- **Platform-specific accessibility APIs:** AccessKit is relied upon to abstract
  these, but it's useful to know them. Microsoft's documentation on UI
  Automation (especially the **TextPattern** and control patterns) provides
  insight into what screen readers expect
  ([1](https://accesskit.dev/how-it-works/#:~:text=The%20current%20released%20platform%20adapters,support%20rich%20text%20or%20hypertext))
  ([5](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-implementingtextandtextrange#:~:text=Learn%20learn,1)).
  Apple's developer docs on making apps accessible (NSAccessibility programming
  guide and VoiceOver guide) will help for macOS specifics (like how to label
  custom controls properly)
  ([1](https://accesskit.dev/how-it-works/#:~:text=accessible%2C%20including%20support%20for%20both,support%20rich%20text%20or%20hypertext)).
  The AT-SPI documentation (e.g., the GNOME accessibility guide) is a reference
  for Linux
  ([1](https://accesskit.dev/how-it-works/#:~:text=,the%20current%20Windows%20accessibility%20API)).

- **Assistive Technology tools:** Tools like **Accessibility Insights for
  Windows** (which includes "Inspect" and automation scanners) will be used
  ([2](https://github.com/zed-industries/zed/issues/41138#:~:text=2,the%20editor%20to%20learn%20it)),
  Apple's **Accessibility Inspector**
  ([1](https://accesskit.dev/how-it-works/#:~:text=accessible%2C%20including%20support%20for%20both,support%20rich%20text%20or%20hypertext)),
  and the **Orca screen reader** (with its learning resources
  ([2](https://github.com/zed-industries/zed/issues/41138#:~:text=2,the%20editor%20to%20learn%20it)))
  to develop and test. These are essential to debug what the app is exposing.
  Familiarization with these tools early on is recommended.

- **Standards and guidelines:** The **Web Content Accessibility Guidelines
  (WCAG) 2.1/2.2** are the basis for many accessibility requirements even in
  native apps
  ([4](https://zed.dev/blog/gpui-ownership#:~:text=Ownership%20and%20data%20flow%20in,an%20old%20version%20of%20GPUI)).
  Relevant ones (like keyboard access, focus order, contrast, semantics) are
  mapped to the project context. Additionally, **EN 301 549** in the EU and
  **Section 508** in the US are standards/laws requiring accessible software –
  aligning with these will be important for enterprise adoption
  ([2](https://github.com/zed-industries/zed/issues/41138#:~:text=Expected%20Behavior%3A%20The%20editor%20should,the%20folder%2C%20again%20nothing%20spoke)).
  While the primary motive is usability, adhering to standards ensures nothing
  is missed (like offering captions for any audio feedback, etc., though Gauss
  is mostly visual).

By leveraging the above and treating accessibility as a core part of the
development (with design and QA involvement), the chance of major issues down
the line is minimized.

## What this investment yields

- **Compliance and market access:** Upon implementing this plan, **screen-reader
  support on Windows, macOS, and Linux** as well as keyboard-only operation can
  be credibly claimed. This opens doors to government and education markets
  that require software to meet accessibility standards (e.g., European public
  sector tenders often ask for EN 301 549 compliance
  ([2](https://github.com/zed-industries/zed/issues/41138#:~:text=Expected%20Behavior%3A%20The%20editor%20should,the%20folder%2C%20again%20nothing%20spoke))).
  A VPAT (Voluntary Product Accessibility Template) based on the support matrix
  can be prepared to formally document compliance.

- **Better software design:** Building with accessibility in mind often
  improves overall UX. Clear focus handling and keyboard shortcuts benefit
  power users; well-structured UI content (for screen readers) often translates
  to cleaner state management in code. By "learning the hard lessons early,"
  good practices are instilled in the team – making accessibility a default
  consideration for any new feature (rather than a retrofit).

- **User trust and goodwill:** Users with disabilities (and advocacy
  communities) will appreciate that Gauss is usable from day one, without
  needing to ask for special support. This fosters a positive reputation. Even
  among users without disabilities, features like keyboard shortcuts and
  high-contrast modes are often welcomed (think of situational impairments or
  simply personal preference). This delivers a robust product for everyone, not
  a niche add-on.

- **Future-proofing:** By integrating with AccessKit, the project is positioned
  to easily gain new platform accessibility support. For instance, if a mobile
  app using a similar paradigm is later built, AccessKit's planned Android/iOS
  adapters could be leveraged. If a WebAssembly/web-canvas version of GPUI is
  considered, the planned web adapter could allow screen readers to work there
  too ([1](https://accesskit.dev/how-it-works/#:~:text=Planned%20adapters)).
  Also, as AccessKit extends to rich text or new patterns, those improvements
  can be adopted with minimal changes to the app logic (since data is primarily
  populated for AccessKit to consume). In short, building once yields repeated
  benefits.

Finally, doing this work reinforces that **accessibility is a core feature** of
the platform. There will be no need to scramble later to "add accessibility" –
it will be baked into GPUI, benefiting Gauss and any other app built on this
framework.

## Suggested Next Steps

- **Prototype the AccessKit integration on one platform (Windows) and a simple
  UI:** Start with a minimal GPUI window on Windows and get the AccessKit
  `Adapter` working. Build a tiny test app (one window, a couple of controls)
  and iterate until NVDA can read them. This will flush out integration issues
  (like event timing or focus handling) in a simpler setting. Once Windows is
  roughly working, do the same on macOS and Linux. The aim is to have a basic
  cross-platform "Hello, Accessibility" demo (a window with a label, button,
  and text field that screen readers can read and interact with) within a short
  time. This serves as a proof-of-concept and confidence boost.

- **Implement full keyboard navigation and TextPattern for text inputs:**
  Before diving into every widget, ensure the fundamentals are solid. That
  means: the **focus system** in GPUI is tied to accessibility (Tab order,
  arrow key nav in menus, etc., all generate the right focus events), and
  **text boxes support editing via screen reader** (caret moves, selection
  announced). Achieve parity with a standard platform text box. This will
  likely involve refining the action handling for text (using UIA TextPattern
  methods as a reference) and testing heavily on Windows (since it's the
  strictest). Once a text box works well on Windows, check VoiceOver and Orca
  for any additional tweaks.

- **Iteratively enhance component by component:** Go through the UI component
  list (buttons, checkboxes, menus, dropdowns, list views, tree views, dialogs,
  etc.) and implement their accessibility roles/properties. Core ones used in
  Gauss's main workflow can be prioritized first. As this progresses, update
  the accessibility support matrix document. Also, implement the
  **high-contrast theme toggle** and get feedback from someone with low-vision
  on the colour choices. Doing them in parallel ensures visual accessibility is
  not forgotten while focusing on screen readers.

- **Automate where possible and prevent regressions:** Integrate accessibility
  checks into CI. For example, on Windows the Accessibility Insights CLI can be
  used to run tests on the app UI (though it might require the app to be
  running; an automated test mode that opens key windows and then runs the tool
  could be used). Unit tests for accessibility tree generation can also be
  written (e.g., given a certain UI state, the code produces a certain
  `TreeUpdate` – this can be asserted in a test). A policy that any UI change
  must be tested for accessibility should be set up (just as is done for
  performance or memory). This cultural shift will keep the project accessible
  long-term.

Remember, **treat accessibility work as foundational infrastructure**, not a
one-off feature. Investing effort up front will pay off by reducing future
retrofits and by expanding the user base. As these steps progress, users (if
possible) and experts should be continuously involved, adjusting the plan if
something isn't working well. By phase's end, Gauss will be one of the few (if
not only) vector graphic tools that is fully accessible – a significant
achievement technically and socially.
