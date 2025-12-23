# GPUI AccessKit integration primer

**Keeping GPUI on desktop** while delivering **full screen‑reader support and limited‑mobility accessibility** (keyboard‑only, switch/voice control friendly). The crux: it’s feasible, but it’s engineering, not wishful thinking. The plan hinges on integrating **AccessKit** into GPUI, implementing proper text semantics, and proving the result on NVDA/JAWS/Narrator (Windows), VoiceOver (macOS) and Orca (Linux).

---

## Executive summary

* **Feasibility:** High, with caveats. AccessKit already ships platform adapters for **Windows (UIA)**, **macOS (NSAccessibility)** and **Unix (AT‑SPI)**. It’s explicitly designed so immediate‑mode toolkits (like GPUI) can expose an accessibility tree with **stable node IDs**. ([GitHub][1])
* **Biggest technical risk:** **Text**. AccessKit’s adapters support single/multi‑line text inputs today; **rich/hypertext** are not yet supported upstream. If Marrakesh Express needs full rich‑text semantics (attributes per range, embedded objects) you’ll either constrain features for v1 or fund upstream work. ([GitHub][1])
* **Why now:** Zed/GPUI currently lacks practical Windows screen‑reader support (recent user report: “absolutely inaccessible”), so shipping without this work is not an option. ([GitHub][2])
* **Tooling reality:** You won’t get “free” a11y. You will build and maintain the GPUI↔AccessKit bridge, then verify with OS‑level tools: **Accessibility Insights/Inspect** (Windows), **Accessibility Inspector/VoiceOver** (macOS), **Accerciser/Orca** (Linux). ([Accessibility Insights][3])

Bottom line: with a focused integration, disciplined text support, and a rigorous test matrix, you can meet **screen reader + limited mobility** requirements on desktop while keeping GPUI’s performance virtues.

---

## What “full” accessibility means here (acceptance criteria)

### Screen readers

* **Windows:** All interactive widgets expose correct **UIA roles**, names, states, and actions; text controls implement **TextPattern** (and **TextEditPattern** when editing), including caret movement, selection ranges, word/line navigation and text‑changed events. Verified on **NVDA**, **JAWS**, and **Narrator** with **Inspect**/**Accessibility Insights**. ([Microsoft Learn][4])
* **macOS:** Widgets expose **NSAccessibility** roles/attributes; text implements the macOS text attributes and notifications VoiceOver expects. Verified with **VoiceOver** and **Accessibility Inspector**. ([Apple Developer][5])
* **Linux:** Widgets expose **AT‑SPI** roles/states/actions; text implements **Atspi.Text** interface with selections and text attributes. Verified with **Orca** and **Accerciser**. ([gnome.pages.gitlab.gnome.org][6])

### Limited mobility

* **Keyboard‑only operation:** Logical focus order, consistent and visible focus indication, no keyboard traps, and full command coverage from the keyboard (parallels WCAG 2.x **2.1 Keyboard**, **2.4 Focus Order/Visible**). ([W3C][7])
* **Switch/voice control:** All actionable elements expose programmatic **invoke**/**set value** actions via platform APIs (UIA patterns / NSAccessibility actions / AT‑SPI actions). ([Microsoft Learn][8])
* **Low‑vision:** High‑contrast theme tokens and scalable typography; platform contrast settings respected where possible.

If you also need procurement‑grade compliance in Europe, align with **EN 301 549** for “non‑web software”. It references WCAG but explicitly covers native desktop software too. ([ETSI][9])

---

## Impact of each proposed step

Below I assess **scope**, **risks**, and **proof points** per step you listed.

### 1) Adopt AccessKit early (GPUI → AccessKit tree + actions)

**What changes:**

* In GPUI’s platform layer, instantiate per‑OS adapters: accesskit_windows::Adapter, accesskit_macos::Adapter, and accesskit_unix::Adapter.
* Maintain a GPUI→AccessKit **tree translator** that emits Node{ id, role, name, value, bounds, states, relations } plus **action handlers** (invoke, focus, set_value, toggle, scroll, text ops).
* Dispatch **tree updates** and **focus updates** on every relevant GPUI state change (layout, visibility, selection, enable/disable).

**Why it’s tractable:**

* AccessKit’s design explicitly targets immediate‑mode UIs given **stable node IDs**, and provides per‑OS adapters already. GPUI’s architecture (AppContext owns entities) gives you a central locus to generate stable identities. ([GitHub][1])

**Notable risks:**

* **Windowing:** GPUI does not use winit by default; you’ll integrate AccessKit’s adapters directly (not accesskit_winit). This is normal: the Windows adapter hooks WM_GETOBJECT and focus, macOS adapter subclasses NSView or forwards focus, Unix adapter binds AT‑SPI over D‑Bus. ([GitHub][10])
* **Threading & event order:** UIA demands you initialize before responding to WM_GETOBJECT; the Windows adapter provides a handle_wm_getobject flow and update_if_active/update_window_focus_state to keep you honest. ([Docs.rs][11])

**Proof points:**

* **Windows**: Adapter::new(HWND, focused, action_handler); call handle_wm_getobject on WM_GETOBJECT; call update_if_active on tree changes. ([Docs.rs][11])
* **macOS**: accesskit_macos::Adapter or SubclassingAdapter; forward focus from NSWindow to content view if needed. ([Docs.rs][12])
* **Linux**: accesskit_unix adapter publishing roles/states/actions to AT‑SPI. ([doc.servo.org][13])

**Sizing:** Medium–Large (core platform work across three OSes).

---

### 2) Prioritize text controls (caret, selection, range ops)

**What changes:**

* Implement the **text control** surface in your AccessKit nodes, including:

  * current caret (possibly modelled as a zero‑width selection),
  * **selection ranges**,
  * **move**/**expand** by character/word/line,
  * value change notifications and editable text actions.
* On Windows that maps to **UIA TextPattern/TextEditPattern**; Linux maps to **Atspi.Text**; macOS maps to the corresponding NSAccessibility text attributes/events. ([Microsoft Learn][4])

**Risk to call out:**

* AccessKit currently documents that adapters support single‑/multi‑line text inputs but **not rich/hypertext** yet. For rich text editing (attributes per span, embedded links, code folding), you’ll need to scope features or extend AccessKit’s schema/adapter support. For standard form inputs, you’re fine. ([GitHub][1])

**Proof points:**

* Verify in **Accessibility Insights/Inspect** that a text field exposes **TextPattern**; cursor movement fires **TextSelectionChanged** appropriately; VoiceOver/Orca can read/modify selection by word/line. ([Accessibility Insights][14])

**Sizing:** Large if you need rich text; Medium for standard text inputs.

---

### 3) Audit colour/contrast and theming

**What changes:**

* Introduce **contrast tokens** (foreground/background pairs with ≥ WCAG contrast ratios) and a **High‑Contrast** variant in Longbridge theme presets; expose per‑user scaling for typography and hit‑targets.
* Honour OS settings where practical (e.g., macOS “Increase contrast”; Windows contrast themes) and always provide an app‑level override.

**Proof points:**

* Contrast ratios meet WCAG AA for text and UI components; focused elements are clearly visible and not colour‑only. (Yes, WCAG is “web”, but the contrast maths applies just as well to native surfaces; EN 301 549 references WCAG criteria for non‑web software.) ([ETSI][9])

**Sizing:** Small–Medium; mainly design tokens + a few component tweaks.

---

### 4) Test on real AT, not just inspectors

**What changes:**

* Formalize a **test matrix** with scripted journeys and checkpoints:

  * Windows (NVDA, JAWS, Narrator), macOS (VoiceOver), Linux (Orca).
  * Inspectors: **Accessibility Insights**/**Inspect** (Windows), **Accessibility Inspector** (macOS), **Accerciser** (Linux).
  * Scenarios: focus order; announcing names/roles/states; text entry/editing; list/grid navigation; menus; live updates; error notifications. ([Accessibility Insights][3])

**Proof points:**

* Capture **AT transcripts** for each scenario (what the screen reader announces, keystrokes used) and keep them in CI artefacts.
* Use **Accessibility Insights** to assert required UIA patterns/properties when you build. ([Accessibility Insights][3])

**Sizing:** Ongoing; initial setup Medium.

---

### 5) Publish an a11y support matrix (component + framework)

**What changes:**

* For each Longbridge component (Button, Input, Dialog, Table, Tree, Tabs, Menu, Tooltip, Toast, etc.) and key GPUI features (window management, focus, navigation), publish a row with:
  **Role, Name, State, Keyboard, Screen‑reader narration, Text interactions (if any), Live regions, Announcements** and **Known gaps**.
* Tie this matrix to automated smoke checks (Windows UIA properties via Insights CLI where possible; AT‑SPI/NSAccessibility inspections in test harnesses).

**Proof points:**

* Externally visible documentation; gives buyers/regulators confidence (and reduces “does X read correctly in Y?” emails).

**Sizing:** Small for the doc; Medium to keep green as components evolve.

---

## Architecture notes and gotchas

* **Stable Node IDs:** Immediate‑mode toolkits must keep **stable identities** for elements across frames. Map GPUI entities to **AccessKit NodeId** deterministically (e.g., a u64 derived from a component key + stable path). AccessKit’s design assumes this and avoids keeping the full tree on your side. ([GitHub][1])
* **Focus & window activation:** Drive focus through GPUI's focus manager and forward it into the adapter (update_window_focus_state). On Windows, ensure you initialize UIA **before** returning from WM_GETOBJECT. ([Docs.rs][11])
* **Performance:** Accessibility tree updates are **incremental**; you only push diffs to the adapter. This keeps overhead low, provided you coalesce updates sensibly. ([GitHub][1])
* **GPUI/plumbing reality:** GPUI uses its own windowing; plan to integrate the **platform adapters directly**, not via accesskit_winit. (There is an accesskit_winit crate, but it’s for winit‑based apps.) ([GitHub][10])

---

## Example: Windows adapter wiring (sketch)

This illustrates the control flow you'll embed in GPUI's Windows platform layer:

```rust
// Pseudocode-ish; see accesskit_windows docs for exact signatures
use accesskit::{ActionRequest, Node, NodeId, Tree, TreeUpdate};
use accesskit_windows::{Adapter, QueuedEvents};
use windows::Win32::UI::WindowsAndMessaging::{WM_GETOBJECT, MSG};

struct A11y {
    adapter: Option<Adapter>,
}

impl A11y {
    fn init_for_hwnd(&mut self, hwnd: isize, is_focused: bool) {
        self.adapter = Some(Adapter::new(
            accesskit_windows::HWND(hwnd),
            is_focused,
            move |action: ActionRequest| {
                // map actions (focus, click, set_value, text ops) back into GPUI
            },
        ));
    }

    fn handle_wm_getobject(&mut self, wparam: isize, lparam: isize) -> Option<isize> {
        self.adapter.as_mut()?.handle_wm_getobject(
            accesskit_windows::WPARAM(wparam),
            accesskit_windows::LPARAM(lparam),
            &mut || {
                // Return the initial full tree
                let root = Node::new(/* role, name, ... */);
                TreeUpdate { /* root, nodes, focus, .. */ }
            },
        ).map(|r| r.into())
    }

    fn update(&mut self, produce_update: impl FnOnce() -> TreeUpdate) {
        if let Some(adapter) = &mut self.adapter {
            if let Some(events) = adapter.update_if_active(produce_update) {
                events.raise(); // must be raised on the right thread
            }
        }
    }
}
```

See Adapter::new, handle_wm_getobject, update_if_active, update_window_focus_state in the Windows adapter. macOS (accesskit_macos) and Linux (accesskit_unix) expose similar “adapter + updates” patterns for NSAccessibility and AT‑SPI respectively. ([Docs.rs][11])

---

## Risks you must explicitly budget for

1. **Rich text/editor semantics**
   AccessKit's own docs state rich text/hypertext aren't yet implemented in adapters. If Marrakesh Express requires attribute runs, in‑line widgets, hyperlinks, or code‑editor semantics, expect upstream work and extra testing. Otherwise, constrain to plain text for v1. ([GitHub][1])

2. **Windows parity pressure**
   Windows users are vocal (rightly). Zed’s current Windows a11y issue shows expectations: NVDA/JAWS must work end‑to‑end. Treat **UIA TextPattern** conformance as a gate. ([GitHub][2])

3. **Fragmentation across three OSes**
   You’ll hit platform quirks (e.g., NSAccessibility focus forwarding, AT‑SPI event ordering). The adapters smooth many edges, but your test matrix keeps you honest. ([Docs.rs][12])

---

## “Done means done”: concrete acceptance tests

* **Smoke** (every build):

  * Windows: Accessibility Insights “FastPass” on key windows shows correct **control types**, **names**, **patterns** (especially TextPattern). ([Accessibility Insights][3])
  * macOS: Accessibility Inspector shows expected roles/labels; VoiceOver can navigate the main flows without rotor spelunking. ([Apple Developer][15])
  * Linux: Accerciser tree shows correct roles/states; Orca reads and edits text fields and announces selections. ([help.gnome.org][16])
* **Scenario** (manual scripts):

  * Task flows (create/open/search/send/etc.) are **completely** operable from keyboard and screen reader on all three OSes, with documented transcripts.
* **Contrast & scaling:**

  * All text meets contrast thresholds; focus ring is visible at all times; UI scales meaningfully for large text.
* **Matrix published:**

  * Longbridge components table shows complete mappings and notes any temporary exceptions (with release IDs).

---

## Dependencies & references (why this isn’t pie‑in‑the‑sky)

* **AccessKit** (design, adapters, immediate‑mode suitability, single/multi‑line text support, lack of rich text/hypertext): project site and README; plus adapter crates on docs.rs/crates.io. ([GitHub][1])
* **GPUI architecture** (AppContext, entity ownership): Zed blog—useful to reason about stable node identities and update boundaries. ([Zed][17])
* **GPUI↔winit** (not used): confirms you’ll embed adapters directly, not via accesskit_winit. ([GitHub][10])
* **Zed Windows a11y status:** recent issue describing read‑world lack of screen‑reader support—your baseline to beat. ([GitHub][2])
* **Text patterns & inspectors:** official Microsoft UIA text pattern docs; Apple/Orca docs; Accessibility Insights/Inspect/Accerciser usage. ([Microsoft Learn][4])
* **Standards:** WCAG 2.2 (focus order etc.) and EN 301 549 for non‑web software. ([W3C][18])

---

## What this buys you

* **Compliance runway:** You can credibly claim screen‑reader support across desktop OSes, align with **EN 301 549** expectations, and keep procurement doors open in the UK/EU public sector. ([ETSI][9])
* **User trust:** Screen‑reader users get first‑class support rather than a post‑script apology.
* **Future‑proofing:** When AccessKit gains richer text/hypertext, your GPUI bridge simply **emits more semantics**; you don’t have to rethink your UI.

---

## Suggested next moves (concrete, bounded)

1. **Prototype the bridge on one window** (e.g., your main shell): implement node tree + actions and prove NVDA/VoiceOver/Orca navigation works end‑to‑end.
2. **Ship text inputs** with full caret/selection semantics; lock down TextPattern on Windows. ([Microsoft Learn][4])
3. **Roll out component by component** per your a11y matrix; add contrast tokens and a high‑contrast theme early to avoid re‑painting later.
4. **Automate inspections** (Windows first with Accessibility Insights CLI) to stop regressions. ([Accessibility Insights][3])

Treat this as foundational infrastructure, not “nice to have”. Do it well once; reap the benefits release after release.

[1]: https://github.com/AccessKit/accesskit "GitHub - AccessKit/accesskit: Accessibility infrastructure for UI toolkits"
[2]: https://github.com/zed-industries/zed/issues/41138?utm_source=chatgpt.com "Windows: Screen reader accessibility missing completely"
[3]: https://accessibilityinsights.io/docs/windows/overview/?utm_source=chatgpt.com "Accessibility Insights for Windows"
[4]: https://learn.microsoft.com/en-us/dotnet/framework/ui-automation/ui-automation-textpattern-overview?utm_source=chatgpt.com "UI Automation TextPattern Overview - .NET Framework"
[5]: https://developer.apple.com/documentation/accessibility/voiceover?utm_source=chatgpt.com "VoiceOver | Apple Developer Documentation"
[6]: https://gnome.pages.gitlab.gnome.org/at-spi2-core/libatspi/iface.Text.html?utm_source=chatgpt.com "Atspi.Text"
[7]: https://www.w3.org/WAI/WCAG21/Understanding/focus-order.html?utm_source=chatgpt.com "Understanding Success Criterion 2.4.3: Focus Order | WAI"
[8]: https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-implementingtextandtextrange?utm_source=chatgpt.com "Text and TextRange Control Patterns - Win32 apps"
[9]: https://www.etsi.org/human-factors-accessibility/en-301-549-v3-the-harmonized-european-standard-for-ict-accessibility?utm_source=chatgpt.com "EN 301 549 V3 the harmonized European Standard for ICT ..."
[10]: https://github.com/rust-windowing/winit/issues/3535?utm_source=chatgpt.com "[Research] Can we use GPUI in the winit library? #3535"
[11]: https://docs.rs/accesskit_windows/latest/accesskit_windows/struct.Adapter.html "Adapter in accesskit_windows - Rust"
[12]: https://docs.rs/accesskit_macos "accesskit_macos - Rust"
[13]: https://doc.servo.org/accesskit_unix/atspi/index.html?utm_source=chatgpt.com "accesskit_unix::atspi - Rust"
[14]: https://accessibilityinsights.io/docs/windows/getstarted/inspect/?utm_source=chatgpt.com "Inspect in Accessibility Insights for Windows"
[15]: https://developer.apple.com/documentation/accessibility/accessibility-inspector?utm_source=chatgpt.com "Accessibility Inspector | Apple Developer Documentation"
[16]: https://help.gnome.org/users/orca/?utm_source=chatgpt.com "Orca Screen Reader"
[17]: https://zed.dev/blog/gpui-ownership "Ownership and data flow in GPUI — Zed's Blog"
[18]: https://www.w3.org/WAI/WCAG22/quickref/?utm_source=chatgpt.com "How to Meet WCAG (Quick Reference)"
