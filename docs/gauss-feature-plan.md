# Gauss Feature Parity Roadmap (Illustrator 10 Parity)

This roadmap outlines a multi-phase plan to evolve **Gauss** (a Rust-based SVG
illustration tool built on GPUI/GPUI Component with AccessKit) toward full
feature parity with Adobe Illustrator 10. Features are prioritized by impact
and foundational value, and each phase builds upon the last. Cross-cutting
concerns – **accessibility** (AccessKit integration), **localizability**,
**performance optimizations**, and **pervasive scripting** (via RustPython and
potential LLM-driven control) – are addressed **from the start and in every
phase** rather than retrofitted later. Gauss targets **Linux (Fedora/Ubuntu)**,
**FreeBSD**, **Windows**, and **macOS**, so all decisions ensure cross-platform
compatibility.

**Note:** We avoid investing in flashy demos or novelty features until core
professional needs are met. The focus is on robust, reusable architecture and
high-value capabilities. All features are exposed through scripting APIs as
they are implemented, enabling automation and future AI assistant control from
day one.

## Phase 1: MVP Core Editing Tools and Foundation

**Goals:** Deliver the basic drawing and editing experience – the
highest-impact, most frequently used vector tools – on top of a solid
application framework. This phase establishes the **foundation** for Gauss’s
UI, document model, and cross-cutting systems. It prioritizes features that
every illustrator uses daily, and builds reusable patterns to support later
phases.

**Key Features and Tasks:**

- **Basic Shape Drawing:** Implement tools to create fundamental shapes:
  rectangles, ellipses, and straight lines. These correspond to Illustrator’s
  core shape tools (Rectangle Tool, Ellipse Tool, Line Segment Tool). Users
  should be able to draw constrained shapes (e.g. hold Shift for
  squares/circles) and simple polylines. A **Pen Tool** is introduced for
  freeform path drawing (straight and Bezier curves), enabling creation of
  arbitrary shapes – a cornerstone of any vector editor.

- **Selection Tools:** Provide both the standard Selection tool (to select and
  move entire objects) and the Direct Selection tool (to select and edit
  individual anchor points or path segments) as in Illustrator. These allow
  users to manipulate whole shapes or tweak the nodes of paths. Basic **group
  selection** and multi-select via marquee drag are included for efficiency.
  The **Lasso tool** (freeform selection of points) can be scheduled for a
  later polish sub-phase if not done here.

- **Transformation & Alignment:** Enable basic object transformations: move,
  scale, rotate (with optional numeric input or on-canvas handles). A
  bounding-box transform UI or separate Rotate/Scale tools can be used for
  intuitive manipulation. Implement object alignment and distribution controls
  (e.g. align left, center vertically) – these are high-impact utilities for
  layouts. Also include **arrange** operations like send-to-back/front and
  grouping/ungrouping for layer ordering control.

- **Fill and Stroke (Solid Colours):** Introduce a simple styling panel to set
  an object’s fill colour, stroke colour, and stroke weight. Use GPUI Component’s
  built-in colour picker for colour selection. At this stage, support solid fills
  and basic strokes (solid line, adjustable width) only – complex paint styles
  like gradients or patterns will come later. Ensure the UI reflects current
  fill/stroke and can apply changes to selected objects.

- **Layers Panel Basics:** Implement a minimal **Layers** or Objects panel to
  list objects and their grouping. Users should be able to toggle visibility
  and lock/unlock layers, as these are foundational for complex artwork
  management. Advanced layer operations (e.g. “Release to Layers (Sequence)”
  for animations) are not yet needed, but the underlying document model should
  support a hierarchy of groups/layers for future expansion.

- **Undo/Redo and File I/O:** Integrate a History stack (leveraging GPUI
  Component's `History` if available) to support multiple undo/redo steps. This
  is critical for any editing workflow. Gauss maintains **separate history
  stacks** for document edits (Ctrl+Z/Y) and selection changes
  (Ctrl+Shift+Z/Y), enabling independent traversal of edit and selection
  states. Furthermore, implement opening and saving in SVG format (as the
  native document format). Using SVG ensures immediate compatibility with other
  tools; Illustrator 10 itself emphasized improved SVG
  support([1](https://www.macworld.com/article/164061/illustrator-6.html#:~:text=Illustrator%20now%20offers%20better%20support,and%20improvements%20abound%20as%20well)).
  In this phase, SVG is sufficient to represent the basic shapes, groups, and
  current style properties. (In later phases, the plan will evaluate whether
  SVG can still capture all advanced features or if an alternative open format
  is needed.)

- **Cross-Platform UI Framework:** Stand up the application shell using **GPUI
  and GPUI Component**. Create the main window with a toolbox (toolbar), canvas
  area, menus, and status bar. Use GPUI Component for standard controls
  (buttons, sliders, menus, colour picker) to get consistent theming and avoid
  custom widget work upfront. Verify that GPUI Component can handle high-DPI
  rendering, input methods, and theming on each target OS. As part of this,
  **assess GPUI Component’s suitability** for Gauss’s more specialized UI
  needs: identify gaps where we may need custom widgets (e.g. a future gradient
  editor or on-canvas Bézier handles for path editing). If gaps are found,
  scope out the work for custom components in later phases (for example, a
  custom gradient slider control or editable anchor-point handles on the
  canvas).

- **Accessibility from Day One:** Integrate **AccessKit** at the outset to
  provide a basic accessibility tree for the UI. All interactive UI elements
  (buttons, menu items, tool options) should expose proper roles and labels so
  that screen readers on Windows, Mac, and Linux can identify and activate
  them. Ensure keyboard-only operation is possible (every command can be
  triggered via shortcuts or focusable UI controls). For now, the canvas itself
  can be a single accessible node (e.g. role=“canvas”) with a simple label,
  since implementing per-object accessibility is complex; we will refine that
  in later phases. By treating accessibility as a first-class concern, we avoid
  retrofitting it later (Illustrator never had to consider this in 2001, but
  Gauss will). We will use AccessKit’s cross-platform adapters to cover Windows
  UI Automation, macOS NSAccessibility, and AT-SPI on Unix, and test with
  screen readers in this phase to catch issues early.

- **Performance Baseline:** Even with this minimal feature set, establish
  performance budgets – the canvas should handle basic drawings at interactive
  framerates via GPU rendering. Use simple scenes to measure frame rates and
  input latency on all platforms. This phase likely won’t hit performance
  bottlenecks, but we set up profiling tools now. Ensure the architecture (e.g.
  use of GPU for rendering shapes, minimal recomputation on modest edits) is
  sound to scale up.

- **Scripting Interface Init:** Embed a **RustPython** interpreter and design a
  scripting API that exposes the Phase 1 features. For example, provide
  programmatic ways to create shapes, select items, transform objects, and
  adjust colours through Python scripts. Each user-facing command in the UI
  should correspond to a callable function in the scripting engine. This
  establishes a **reusable pattern**: as new features are added in each phase,
  we expose them to scripts immediately. By doing this early, we validate that
  our internal architecture cleanly separates model operations from UI
  (facilitating both scripting and potential LLM-driven control). In later
  phases we will expand this API and possibly add a natural-language command
  layer (where an LLM can translate user requests into sequences of script
  calls).

**Phase 1 Outcome:** A **minimal yet complete editor core** – users can draw
shapes and paths, style them with solid colours, select and transform objects,
manage basic layers, undo mistakes, and save their artwork to SVG. The UI runs
on all target OSs with a consistent look. Crucially, the app’s foundation is
laid with accessibility, scripting, and performance-aware design in place. This
foundation will support all subsequent enhancements without significant rework.

## Phase 2: Typography and Text Tools

**Goals:** Introduce text capabilities – a critical high-impact feature set for
any design application. Text support is somewhat self-contained and can be
developed in parallel after Phase 1, but it relies on a stable rendering core
from Phase 1. This phase covers creating and editing text, a typography panel,
and text styling. Text is both **frequently used (e.g. for logos, layouts)**
and a **foundational domain** on its own, so it gets a dedicated phase. We also
continue to refine cross-cutting concerns like accessibility (text is a
challenge there) and scripting for text manipulation.

**Key Features:**

- **Basic Type Tool:** Implement a **Type tool** to allow users to add text to
  the canvas, as either **point text** (a single line that grows as you type)
  or **area text** (text inside a resizable textbox frame). Illustrator’s Type
  Tool creates both point type and area type depending on drag vs click. We
  will support clicking on the canvas to start a point text object, or
  click-and-dragging to create a fixed text area. Text objects can be moved and
  transformed like other objects.

- **Text Rendering & Fonts:** Integrate a text shaping and rendering library
  (e.g. using Rust’s font due or skribo/Swash, or platform text APIs if
  available) to draw text with proper font metrics. Ensure we support standard
  font formats installed on the system. The text engine should handle at least
  basic Latin text in this phase; full international text and complex script
  shaping might be expanded later (but the design should not preclude it). Use
  vector text (outlines) for final output to SVG (e.g. embed text as SVG
  `<text>` elements for now). Provide a way to **convert text to outlines**
  (create vector shapes from text) since that is often needed for
  interoperability – though this can be a later phase feature if complex.

- **Typography Controls:** Add UI for common text properties: font family
  selection (list system fonts), font size, basic styles (bold, italic,
  underline), text alignment (left/center/right for paragraphs), and colour
  (reusing the colour picker for fill colour of text). These can be in a
  dedicated **Text/Character panel** or options bar when text is selected. The
  goal is to cover what designers use daily for text styling. Advanced
  typography (kerning, tracking, vertical text, text on path) can be deferred,
  but we note them for future phases.

- **Text Object Editing:** Support editing text directly on canvas: users can
  double-click a text object to enter edit mode, type to change content, and
  see changes live. Implement basic text selection, cursor movement, and
  clipboard copy-paste in text objects. This requires intercepting keyboard
  events when a text object is focused. It’s important for usability that
  editing text is WYSIWYG on the canvas.

- **Accessibility for Text:** Expose text objects to screen readers via
  AccessKit. This is a known challenge – AccessKit supports plain text controls
  but **rich text is not fully supported yet**. To stay accessible, we might
  restrict text editing to uniform styling for now (every text object is a
  single style run) which fits AccessKit’s current capabilities. Each text box
  can register as an `AccessibilityNode` with role “text field” and the content
  as its value, enabling screen-reader reading and editing. If formatting
  within a text object becomes needed (rich text), we will coordinate with
  AccessKit upstream or find a workaround, but for MVP we keep text objects
  simple to ensure compatibility. All text controls in the UI (font menus,
  etc.) also get proper labels for accessibility.

- **Internationalization & Locales:** Since text is being added, this is a good
  point to ensure Gauss can handle different languages. Verify that entering
  Unicode characters (accented letters, non-Latin scripts) is possible. We also
  take the opportunity to audit that all UI strings (tool names, menu labels,
  tooltips) are separated for localization. Setting up a localization framework
  (e.g. using `gettext` or Fluent) in this phase ensures that as features grow,
  we can easily translate Gauss’s interface for non-English users. This
  addresses the **localisability** concern early. We can demonstrate this by
  providing one alternative locale (maybe a test with a language like Spanish
  or an RTL language) to validate the design.

- **Scripting for Text:** Extend the RustPython scripting API to cover text
  operations. For example, allow scripts to create a text object, set its
  content and font, and adjust text properties. This could enable use cases
  like bulk-generating labels from data (which ties into the data-driven
  features in a later phase) or simply letting power users automate repetitive
  text styling tasks. By exposing text capabilities to scripting now, we also
  set the stage for the future data-driven graphics (which will likely
  manipulate text via scripts or APIs).

- **Performance:** Text rendering can be performance-intensive, especially with
  many characters or large blocks of text. We ensure that our text rendering
  uses GPU where possible (e.g. caching glyph atlas textures or using wgpu text
  rendering techniques). In this phase, we test scenarios with multiple text
  objects for any slowdowns. Also, memory usage of loaded font glyphs is
  monitored. The text editing experience should remain smooth (no lag on
  typing).

**Phase 2 Outcome:** Gauss gains robust **typography capabilities**, covering
the majority of common text needs in Illustrator 10. Users can add and style
text in their illustrations, making Gauss viable for layouts, logos, and
diagrams that mix text with graphics. The application remains cross-platform
(needing consideration especially on font handling differences on each OS) and
accessible (with some limitations on rich text announced to users). All text
features are scriptable. By the end of Phase 2, Gauss can handle most basic
artwork tasks (shapes + text), which are the bread-and-butter of vector design
work.

## Phase 3: Path Effects and Advanced Shape Operations

**Goals:** Now that the core drawing and text are in place, Phase 3 expands
Gauss’s capabilities to include more advanced path manipulations and creative
effects. This phase targets features that are **foundational for complex
illustration** (like combining shapes or distorting them) and also includes
some of Illustrator 10’s signature creative tools (e.g. liquify tools). We also
incorporate **reusable code patterns** here – for example, a generic framework
for applying effects or modifications to shapes – so future effects can plug in
easily. These features are highly valuable to professional users who need fine
geometric control or artistic effects.

**Key Features:**

- **Pathfinder Boolean Operations:** Implement shape combination functions
  equivalent to Illustrator’s **Pathfinder**: **Union/Add**, **Subtract**,
  **Intersect**, **Exclude**, etc. These allow users to take multiple
  overlapping shapes and compute a new shape via boolean geometry operations.
  Boolean ops are foundational (enabling broad creation of complex shapes from
  primitives) and frequently used by vector artists. Under the hood, we can
  integrate a geometry library (for Rust, e.g. `lyon` or `boolean-operation`
  crates) to reliably perform these operations. Provide a Pathfinder palette UI
  or menu commands for these actions. This feature’s code will be reused for
  other tools (e.g. we might reuse the boolean engine for cutting paths or
  compound shapes in later features).

- **“Liquify” Distortion Tools:** Add the suite of interactive distortion
  brushes that Illustrator 10 introduced (under the Liquify tools). These
  include the **Warp**, **Twirl**, **Pucker**, **Bloat**, **Scallop**,
  **Crystallize**, and **Wrinkle** tools. Each is a brush that, when dragged
  over vector artwork, locally warps the shape in a particular way (e.g. Warp
  tool drags points in the brush’s radius, Twirl spins them around a centre,
  etc.) [1][illustrator-6]. These tools were novel in Illustrator 10 and enable
  very organic, freeform modifications that would be tedious to do
  point-by-point. Implementing them in Gauss involves:

- Creating a generic **brush engine** for shape manipulation: track the brush
  radius, strength, and continuously apply a geometric transform to points
  under the brush as the user drags.

- For each specific tool, define the transform: e.g. Twirl rotates points
  around the
  cursor([1](https://www.macworld.com/article/164061/illustrator-6.html#:~:text=What%20really%20caught%20my%20eye,be%20made%20to%20an%20illustration)),
  Bloat moves points outward from centre (inflating the shape), etc.

- Use the GPU if possible for performance, but as these modify vector data,
  likely this is done on the CPU and then the result rendered. Efficiency is
  key – applying these to complex paths should remain responsive (we may limit
  the brush effect to a certain number of points per frame, or decimate the
  geometry under extreme cases).

- Add tool UI controls for brush size and intensity (similar to Illustrator’s
  options for these tools).

- These distortion tools are *mostly unique to power users*, but they address a
  big **pain point** in manual drawing (as one reviewer noted, they saved
  “hours… tweaking points” by automating
  distortion([1](https://www.macworld.com/article/164061/illustrator-6.html#:~:text=to%20the%20wonderful%20,be%20made%20to%20an%20illustration))).
  They are prioritized here to position Gauss as a serious creative tool.

- **Envelope and Warp Effects:** Introduce the ability to **warp shapes by
  envelopes or preset warps**. Illustrator 10 had menu commands (or effects)
  like *Envelope Distort* and *Warp* (e.g. arc, bulge, flag shapes that you
  could warp an object into). For Gauss, we implement an **Envelope
  Distortion** feature: allow a shape or text object to be placed in a
  deformable envelope (perhaps defined by a Bezier mesh or another path).
  Alternatively, provide a set of **parametric warp effects** (like arches,
  waves, fisheye, etc.) that the user can apply to an object with adjustable
  parameters. These can be implemented initially as non-destructive effects
  (the original shape is preserved and a transformed version is rendered). From
  an architecture standpoint, we build a general **effects framework**: objects
  can have an “effect stack” altering their appearance (similar to how
  Illustrator’s live effects work). Envelope/warp would be one such effect
  type. This framework is foundational for later adding other effects like drop
  shadows or gaussian blurs (in Phase 4). If full live-effect stacking is too
  complex now, we can implement warp/envelope as one-off tools (apply and bake
  the distortion into the path). Ensure that the **scripting API** can apply
  and parameterize these warps as well.

- **Blend Tool:** Add the **Blend Tool** and blending features, which allow
  smoothly interpolating shapes. In Illustrator, the Blend Tool creates a
  series of intermediate objects between two or more selected objects,
  interpolating their shapes and styles. This is a high-value feature for
  advanced illustrations (for instance, creating colour transitions or object
  morphing effects). For Gauss, implement the ability to blend two shapes or
  colours with a specified number of steps. This likely involves computing
  intermediate geometry (for paths, maybe morphing via matching path points or
  using simpler linear interpolation if shapes are similar). Focus on blending
  position, size, and colour for now. The UI can allow setting the number of
  steps and whether the blend is smooth colour. This feature tests the
  reusability of our code: it might reuse the boolean/path code (for
  interpolating shapes, a naive approach is fine to start) and will integrate
  with grouping (the blend result could be treated as a special group object).
  **Foundational benefit:** the concepts developed here (morphing shapes,
  generating objects programmatically) will be useful when we tackle
  data-driven variations in Phase 5.

- **Advanced Selection & Editing Aids:** This is a good point to introduce the
  **Magic Wand tool** and other selection aids. The Magic Wand in Illustrator
  allows selecting all objects with similar fill/stroke or other
  attributes([1](https://www.macworld.com/article/164061/illustrator-6.html#:~:text=Moving%20along%20the%20toolbar%20Photoshop,%E2%80%94%20variations%20to%20be%20selected)).
  It’s not used by beginners often, but it’s an **essential overlooked
  utility** for complex illustrations (e.g. quickly select all text of a
  certain colour). Implement a Magic Wand that by default selects by fill colour
  (and later allow criteria like stroke or opacity). Also, consider adding
  **measurement tools** (e.g. a Measure tool to measure distances/angles) and
  improved snapping guides (smart guides) in this phase, as they assist in
  precision which becomes more relevant as complexity grows. These do not need
  heavy R&D, but add polish for power users.

- **Continued Cross-Platform & Performance Work:** With many new geometric
  operations, test on each OS for any platform-specific issues (e.g.
  differences in floating-point or memory). Optimize critical paths: for
  instance, ensure boolean ops and blends are using efficient algorithms. If
  some operations are slow on large inputs, consider multithreading those
  computations (Rust makes this feasible). Also leverage the GPU for
  parallelism when possible, e.g., for envelope warping perhaps compute new
  positions via a shader if the math can be done per-vertex. Ensure no
  significant lag when applying a liquify brush on a reasonably complex vector
  graphic – if needed, implement progressive rendering (apply effect
  incrementally) to keep UI responsive.

- **Accessibility & Scripting:** As new tool panels or options are added (for
  brushes, blends, etc.), continue to wire them into AccessKit. For example,
  the brush size slider should be keyboard accessible and labelled. Where
  possible, allow **keyboard alternatives** for these advanced tools (not all
  will have easy keyboard analogs, but e.g. one could nudge selected anchor
  points with arrow keys as a coarse alternative to Warp tool). Expand the
  scripting API to cover these new features: allow scripts to perform boolean
  combines on shapes, initiate a blend between objects, or perhaps simulate a
  distortion (for instance, a script could programmatically warp a path by a
  given function). This ensures even complex operations can be automated or
  controlled by external logic (including potential AI-driven algorithms
  generating effects).

**Phase 3 Outcome:** Gauss significantly closes the gap to Illustrator 10 by
adding **creative vector manipulation tools**. Users can now do much more than
basic shapes: they can combine shapes into new ones, create smooth blends, and
apply funky distortions for artistic effect. Many of these features (liquify
tools, blends) were distinguishing features of Illustrator 10.[1][
illustrator-6] Thus Gauss now stands on par in offering advanced creative
freedom. The underlying implementation of effects and operations emphasizes
reusability (e.g., a unified way to apply “effects” to objects) that will make
adding future effects easier. Gauss remains stable and reasonably performant
under the more complex workflows introduced here, and all operations continue
to be scriptable and as accessible as possible.

## Phase 4: Colour, Appearance, and Visual Effects

**Goals:** This phase focuses on the **appearance** side of vector artwork –
high-fidelity colouring tools, transparency, and visual effects that define the
polish of professional illustrations. These features are often **high-value for
power users** because they allow creating sophisticated visuals (gradients,
meshes, complex styling) and were a big part of Illustrator’s appeal. We treat
many of these as foundational in the sense that a robust colour and paint system
will unlock a broad set of design possibilities. We will also address the
transparency and compositing model in this phase, ensuring Gauss can handle
modern graphics rendering needs.

**Key Features:**

- **Gradient Fills and Gradient Tool:** Expand fill options beyond solid colour
  by introducing **gradients**. Support at least linear and radial gradients
  (Illustrator 10 had both). Users can apply a gradient fill to any shape.
  Implement a new **Gradient Editor widget** to manage gradient colour stops –
  likely a custom GPUI component since this is a specialized UI. The editor can
  appear as a popover or in a panel, showing a line or bar with multiple colour
  stops that the user can add, remove, and adjust. Additionally, provide the
  **Gradient Tool** on the canvas: users can click and drag on a
  gradient-filled object to adjust the vector (start/end points and angle) of
  the gradient. The combination of the off-canvas editor (for precise colour
  stop control) and on-canvas Gradient Tool (for positioning) gives a complete
  gradient experience. Ensure that gradient parameters are part of the object’s
  style and saved to SVG (map to SVG `<linearGradient>` or `<radialGradient>`
  definitions). This feature is **high-impact**: gradients are extremely common
  in design (backgrounds, shading) and were one of the top requested
  capabilities.

- **Gradient Mesh Tool:** For ultimate colour control, implement the **Gradient
  Mesh** feature (Mesh Tool) to create complex multi-coloured meshes within an
  object. This is a power-user feature that defined Illustrator’s capabilities
  in high-end illustration. The Mesh Tool allows a single shape to have a grid
  of mesh points with different colours, blending smoothly across the shape. We
  will let users convert a shape to a gradient mesh, specify rows and columns,
  then use a Mesh Tool to move mesh points and assign colours. This is one of
  the more complex vector painting features: from an implementation standpoint,
  we must interpolate colours across a grid on a path, likely requiring a
  triangulation or splitting the shape into many small patches for rendering
  (since SVG does not natively support gradient meshes). We treat gradient
  meshes as an **advanced** feature in this phase; it might be acceptable to
  release this later within Phase 4 as an update. Scripting wise, exposing
  gradient mesh might be limited (it’s complex to manipulate via code), but at
  least allow script conversion of a simple gradient to a mesh if possible.

- **Advanced Stroke Styles:** Enhance strokes with features like **dashed
  lines**, **arrowheads**, and variable width profiles. Illustrator 10 allowed
  dashed stroke patterns and adding arrowhead markers to lines. We can
  implement dash patterns easily via SVG stroke attributes. Arrowheads might
  require providing a set of marker shapes or letting users specify a symbol to
  use at ends. Variable width profiles (custom stroke thickness along a path)
  were introduced later in Illustrator (CS5), so not required for AI10 parity,
  but if feasible we might include a basic version as a nice-to-have for power
  users (or leave it as a future extension). At minimum, include a **Stroke
  panel** where users can set dash pattern (with numeric fields for dash/gap)
  and choose arrowhead styles for start/end of lines. All these should reflect
  in the SVG output (using `<marker>` for arrowheads etc., which are part of
  SVG spec).

- **Pattern Fills:** Allow objects to be filled with **pattern swatches**
  (tiled vector patterns). Illustrator has long supported defining a selection
  of artwork as a pattern and then filling shapes with that repeating pattern.
  In Gauss, implement a basic ability to apply preset patterns (perhaps include
  a few simple ones, like stripes or dots, as SVG patterns). If time permits,
  also implement a **Pattern Editor**: e.g. a way to select some objects and
  define them as a new pattern swatch, then that swatch can be applied as a
  fill to other shapes. Patterns may not be as commonly used as gradients, but
  they are important in certain designs and were present in AI10. From a
  technical perspective, patterns can be represented with SVG `<pattern>`
  elements for output. The UI to manage swatches (colours, gradients, patterns)
  can be unified in a **Swatches palette**, where users see all saved colours
  and patterns and can apply or edit them.

- **Transparency & Blending:** Introduce global **transparency** settings and
  **blend modes** for objects. Illustrator 10 (and 9) brought in transparency,
  which was a big shift from earlier versions. Each object in Gauss should have
  an opacity value (0–100%) and we support compositing modes like *Multiply,
  Screen, Overlay, etc.*, matching common Adobe blending modes. Implement a
  **Transparency panel** where users can adjust opacity and set a blend mode
  for the selected object or group. This requires our rendering engine (GPUI)
  to support compositing operations – since we’re using GPU, we can leverage
  blending functions in shaders for these modes. Verify that GPUI or our canvas
  abstraction can handle layer blend modes; if not, consider rendering certain
  layers to an offscreen and blending manually. Also include **group
  isolation** (in Illustrator, groups can isolate blending so that blend modes
  only affect within the group). This might be advanced, but mention it for
  completeness of parity if needed. Transparency and blending are
  **foundational for visual richness**, so we ensure performance (blended
  layers can slow rendering; we might use caching of flattened layers when
  static).

- **Visual Effects (Filters):** Add a set of commonly used **live effects**
  (aka filters). By Illustrator 10, popular ones included **Drop Shadow**,
  **Outer Glow/Feather**, **Gaussian Blur**, etc., usually applied via the
  Stylize or Effects menu. For Gauss, implement a subset of these as
  **non-destructive effects** that can be toggled or adjusted. A
  straightforward way is to use shaders or graphical effects: e.g., Drop Shadow
  can be achieved by rendering a copy of the shape with blur and offset behind
  the original. Gaussian Blur can be done with a shader filter on the object.
  Since we already planned an effects framework in Phase 3, we can slot these
  in as additional effect types on objects. Provide a UI to add/remove effects
  on an object (like an Appearance panel or an Effects dialog). Initially, even
  just Drop Shadow and Blur would cover many use cases. These are **high-value
  for professionals** because they eliminate having to switch to raster tools
  for these finishing touches. We must ensure that stacking multiple effects
  doesn’t kill performance – but modern GPUs can handle a couple of shader
  passes. We will test with combinations (e.g. a translucent shape with a blur)
  for speed. All effects parameters should be script-accessible too (so a
  script could, say, apply a drop shadow of given X/Y blur to all selected
  objects).

- **Clipping and Masking:** Implement **Clipping Masks** (ability to use one
  shape to mask the visibility of others) and **Opacity Masks** (using a
  grayscale mask for transparency). Illustrator allows any object to serve as a
  mask for a group (clipping mask), which is important for cropping artwork or
  making complex photo masks. In Gauss, allow users to designate an
  object/group as a clipping parent – in the UI, maybe a “Make Clipping Mask”
  command when two objects are selected (one being on top as the mask). This is
  more of an object grouping behaviour than a tool, but it’s essential. Opacity
  masks (layer masks) could be more advanced; we can map it to SVG by grouping
  and using an object as a mask with fill = black-to-white for transparency.
  These features ensure Gauss can do sophisticated compositing like Illustrator.

- **UI and Accessibility:** Expand the UI to include the new panels (Gradient,
  Swatches, Stroke, Transparency, Appearance/Effects). Use GPUI Component to
  build list views or sliders as needed, ensuring consistent look. Many of
  these panels will involve new custom controls (gradient editors, lists of
  effects, etc.), so careful attention to **AccessKit**: label everything,
  ensure keyboard navigation (e.g. tab through gradient stops perhaps by
  focusing an index). Some complex widgets like the gradient slider may not be
  fully accessible initially (it’s tricky to operate via keyboard alone), but
  we should at least expose alternative numeric input for stop positions/colors
  as a fallback. Continue to offer scriptability as an alternative way to
  adjust these for those who might use assistive tech (e.g. a blind user could
  script a gradient by code if UI is too visual – a stretch goal but good to
  consider).

- **Performance Optimization:** By now the rendering engine has to handle
  gradients, semi-transparent layers, and effects – all potential performance
  hits. Profile rendering of a document with many overlapping transparent
  gradients and a couple of effects. Optimize via GPU wherever possible: e.g.,
  use render-to-texture caching for complex effect stacks (only recompute when
  object changes). Utilize multithreading for non-GPU parts (pattern fill
  generation, etc.). We might introduce level-of-detail reductions (for
  example, while dragging an object, temporarily simplify the rendering of its
  gradient or hide effects, then restore on drop). The aim is to keep the
  editor feeling responsive even as visual complexity increases.

**Phase 4 Outcome:** Gauss achieves full parity in the **colour and appearance
domain** with Illustrator 10. Users can paint with any colour, apply beautiful
gradients (even advanced mesh gradients), use pattern fills, and tweak
transparency and blend modes for sophisticated compositions. Illustrations can
be brought to a professional visual finish completely within Gauss, thanks to
effects like shadows and blurs. The application by this stage covers
essentially all the fundamental and high-impact features of Illustrator 10’s
toolkit. What remains are mostly workflow enhancements and some specialized
tools. Gauss at Phase 4 is a competent alternative to Illustrator 10 for most
use cases, with the added benefits of modern foundations (GPU speed, scripting,
cross-platform consistency, built-in accessibility).

## Phase 5: Symbols, Reuse, and Power-User Tools

**Goals:** Phase 5 tackles the features that enhance reuse, productivity, and
automation – many of which were *differentiators* for Illustrator 10 and cater
to advanced workflows. This includes the **Symbols** system, more powerful
brush capabilities, and making good on Gauss’s scripting/automation promises
(including data-driven graphics). We also consolidate any remaining high-value
tools not yet implemented (e.g. advanced graphing or special shape tools) if
they are deemed useful and not just novelties. Throughout this phase, the
emphasis is on features that **professionals appreciate for efficiency** and on
ensuring Gauss can handle large, complex projects gracefully.

**Key Features:**

- **Symbols and Symbol Libraries:** Introduce a **Symbol** mechanism analogous
  to Illustrator’s Symbols. A Symbol is a master artwork that can be
  instantiated multiple times. Users can create a symbol from any selection of
  objects, which adds it to a Symbols library/panel. They can then drag out
  instances of that symbol onto the canvas. Each **symbol instance** references
  the master; editing the master updates all instances – this massively
  streamlines repetitive
  graphics([1](https://www.macworld.com/article/164061/illustrator-6.html#:~:text=Those%20familiar%20with%20the%20program,copying%20and%20pasting%20as%20necessary)
  )(
  [1](https://www.macworld.com/article/164061/illustrator-6.html#:~:text=Finally%2C%20Symbols%20are%20a%20space,space%20as%20a%20single%20copy)).
  Implement a **Symbols panel** listing all symbols in the document (with
  thumbnails). Provide commands to redefine a symbol, break the link (expand to
  regular objects), and replace symbols (swap all instances of A with B). The
  benefit is both performance (one stored definition drawn many times, saving
  memory) and workflow efficiency. For output, since we remain SVG-native,
  symbol instances could be translated to SVG `<use>` elements referencing a
  single `<symbol>` or `<defs>` entry (this aligns well with SVG). Ensure the
  scripting API can create and swap symbols too. This feature is **foundational
  for reuse** and was a marquee addition in Illustrator 10 (the Macworld review
  noted entire future textbook chapters for it
  ([1](https://www.macworld.com/article/164061/illustrator-6.html#:~:text=Suffice%20it%20to%20say%20that,with%20a%20discussion%20of%20Symbols))),
  so including it solidifies Gauss’s professional toolkit.

- **Symbolism Tools:** Along with basic symbols, implement the associated
  **Symbol Sprayer** and its companion tools (Shifter, Scruncher, Sizer,
  Spinner, Stainer, Screener, Styler). These are niche, but were very
  emblematic of Illustrator 10’s power. The Symbol Sprayer allows quickly
  placing multiple symbol instances by “spraying” them on the canvas([1][
  symbol-sprayer]) – great for backgrounds (leaves, stars, etc.). The other
  tools manipulate the set: e.g. Symbol Shifter moves them around as a group,
  Sizer scales instances, Spinner rotates them, Stainer recolors, Screener
  adjusts transparency, Styler applies graphic styles. Implementing all of
  these is a bit of an undertaking, but we can prioritize a subset if needed:

- The **Symbol Sprayer** itself is the main one (spray copies of a symbol with
  a brush).

- A couple of key modifiers like Symbol Sizer and Spinner could be next (to
  vary the instances).

- If time permits, implement the rest for full parity.

Each tool is essentially iterating over a set of symbol instances in a region
and changing an attribute. This can reuse the brush framework from Phase 3’s
liquify tools (treat symbol instances like particles that can be moved or
transformed by a brush). These tools, while not everyday needs, demonstrate
Gauss’s completeness and provide a fun, powerful way to create complex
illustrations quickly. They cater to **power users** who want to save time on
repetitive elements.

- **Graphic Styles and Appearance Presets:** Enable saving and reusing
  **appearance styles**. Illustrator’s Graphic Styles palette lets users
  capture the entire appearance (fills, strokes, effects) of an object and
  apply it to others. In Gauss, implement a **Styles panel** where the user can
  save the style of the selected object and later apply that style to another.
  This is essentially a macro for copy-pasting appearance attributes. It’s
  particularly handy when working with symbols or repeated elements that need
  consistent styling. Styles can be saved in the document (and possibly
  exported/imported as needed). This feature encourages **reusable components**
  and speeds up the workflow for large documents. It’s not as high-profile as
  symbols, but very useful in pro workflows.

- **Brush Enhancements:** Build on the basic brush tool from earlier phases by
  introducing **custom brush types**:

- **Calligraphic Brushes:** (varying stroke based on angle/pressure) – could
  tie into tablet support if available.

- **Art Brushes:** which stretch a single shape along a path.

- **Pattern Brushes:** which tile a pattern or sequence of shapes along a path.

Illustrator had these brush types to allow creative stroke styles. Implementing
these fully can be complex (especially pattern brushes require handling
corners, etc.). We might start with Art Brush (map an object to the path’s
shape) and simple Pattern Brush (repeat an object along a path with fixed
spacing). These brushes again leverage the symbol concept (often brushes embed
art that could be symbol definitions). They are **power-user tools**, but
including at least some advanced brush capabilities will appeal to illustrators
who do more artistic drawing in Gauss. It also shows off reusability of code:
e.g., reusing the path manipulation code to orient objects along a curve. All
brushes should be saved to SVG as expanded paths for now (since SVG has limited
direct support for these concepts; although one could approximate with markers
or patterns).

- **Charts/Graphs Tool (Optional):** Illustrator 10 included a set of graph
  tools (pie charts, line graphs, etc.). These are arguably less used by pure
  illustrators, but were part of the feature set. If aiming for strict parity
  and if our target users might benefit, we could incorporate a **Graph tool**.
  This would allow users to input data (or paste from CSV) and generate simple
  charts (bar, line, pie). Given that Gauss is scriptable and data-oriented,
  this could actually tie nicely with data-driven goals. However, this is a
  fairly standalone module; we might decide it’s out of scope or implement a
  simplified version (e.g. support one type of graph to start). We mention it
  here as a **nice-to-have addition** that some professionals might expect, but
  it should not distract from more central features unless resources allow.

- **Data-Driven Graphics (Variables Panel):** One of Illustrator 10’s most
  innovative features was **Variables** for data-driven
  graphics([2](https://atpm.com/8.04/illustrator.shtml#:~:text=The%20ability%20to%20separate%20the,innovative%20features%20introduced%20in)
  )(
  [1](https://www.macworld.com/article/164061/illustrator-6.html#:~:text=Truly%20hardcore%20web%20designers%20will,The%20possibilities%20are)).
  Gauss, having a built-in scripting engine, is well-positioned to support
  this. In this phase, we evaluate and prototype data-driven workflows:

- Provide a UI (similar to Illustrator’s Variables panel) to designate certain
  object properties as **variables** (e.g. a text object’s content, or a
  shape’s fill colour can be marked variable).

- Allow importing a data source (CSV or JSON) with fields corresponding to
  those variables.

- Then generate **multiple outputs** by populating the template with each
  record from the data source. For example, a user could design a name tag as a
  template, link the name text to a “Name” variable, load a CSV of names, and
  Gauss would generate one tag per name automatically.

This effectively automates graphic generation, a powerful feature for things
like batch creation of graphics or personalization. Implementation could
utilize the scripting engine under the hood: e.g. the UI triggers a RustPython
script that iterates over data and produces outputs. We might not fully
integrate external databases as Illustrator 10 could (ODBC
linking([1](https://www.macworld.com/article/164061/illustrator-6.html#:~:text=…%20new%20dynamic%20data,ODBC%20compliant%20data%20source))),
but supporting CSV/XML covers most needs in a modern sense. This feature is
**high-value for certain professional scenarios** (especially printing, web
banners, etc.), but not commonly used by every user – thus it comes in this
later phase. It demonstrates Gauss going beyond static design into programmatic
design. Testing and refining this will involve ensuring that performance holds
up when generating possibly hundreds of variants and that memory is managed
(maybe generate to files rather than keep everything in memory).

- **Scripting & LLM Control:** By now, virtually every Gauss feature is
  accessible via the scripting API. In this phase, we finalize the scripting
  documentation and possibly build a simple **scripting IDE** or console within
  Gauss for users to write and run scripts easily. We also explore **LLM
  integration**: for example, a “command palette” where a user can type a
  natural language command (or ask an assistant) and an LLM (connected to
  Gauss’s API) will execute the corresponding actions. Since our scripting API
  is consistent, we could adapt an open-source LLM or a prompt-based system to
  translate instructions (e.g. “draw a red star and duplicate it 5 times around
  a circle”) into script calls. This remains experimental and optional, but
  given the cross-cutting plan, we anticipated it. Even if not shipped as a
  user feature, internally we can test Gauss by controlling it through language
  (which also serves as a robust test of our scripting coverage and
  consistency). In summary, by the end of Phase 5, Gauss should appeal strongly
  to developers and automation-focused designers as well, not just manual
  artists.

- **Cross-Platform & Quality:** With all major features implemented, ensure
  that Gauss runs reliably on all supported OSes. Set up a rigorous test matrix
  (different OS, different hardware, including maybe a lower-end machine to
  test performance limits). Address any platform-specific bugs (e.g. FreeBSD
  might have unique issues with GUI libs or font handling). Ensure that file
  saving is consistent and that files created on one OS open fine on others
  (SVG being the medium, this should hold, but line endings or font naming
  might differ). Also, by now the codebase is large – invest some time in
  **refactoring and using reusable patterns** (e.g. if we notice duplicate code
  for various tool state machines, unify them). This addresses the priority of
  maintaining **reusable components** in the code, making future maintenance
  easier.

**Phase 5 Outcome:** Gauss now matches or exceeds Illustrator 10 in nearly
every feature. The addition of Symbols and advanced brushes means users can
efficiently create and reuse complex elements across their
design([1](https://www.macworld.com/article/164061/illustrator-6.html#:~:text=Symbols%20to%20the%20rescue,a%20library%20of%20its%20own)
)(
[1](https://www.macworld.com/article/164061/illustrator-6.html#:~:text=All%20Symbols%20are%20grouped%20as,stylized%20through%20the%20Styles%20palette)).
The ability to automate via data-driven graphics and scripting integration
sets Gauss apart as a modern tool for large-scale design tasks (something
Illustrator 10 pioneered but Gauss can do with even more ease using Python).
For power users, virtually every Illustrator 10 feature of note (from envelope
distort to symbol spray) is available, meaning they can execute all their
typical workflows. The application has grown complex, but thanks to phasing and
good architecture, it remains stable and user-friendly. We have also avoided
chasing any gimmicks – every added feature serves a clear user purpose in
professional illustration work.

## Phase 6: Final Polish, Parity Checks, and Open Format Evaluation

**Goals:** In this final phase, we concentrate on **polishing the product,
closing any parity gaps**, and evaluating long-term decisions like file format
and future extensibility. We ensure that all **essential but perhaps overlooked
features** of Illustrator 10 (small tools, options) are either in place or
consciously decided against. We also prepare for Gauss’s 1.0 release across all
platforms, with full documentation and quality checks. Importantly, we re-check
whether continuing to use pure SVG as the document format will serve us moving
forward or if we need to adopt/define an expanded open format to cover features
that SVG cannot handle well.

**Key Tasks:**

- **Complete Feature Parity Audit:** Using the Illustrator 10 feature audit
  (the attached spreadsheets) as a checklist, verify if any feature is still
  missing. This might include minor tools or commands such as:

- The **Artboard/Canvas size tool** (if not implemented yet, ensure users can
  adjust canvas size and units).

- The **Slice Tool** and slicing workflow for web images (Illustrator 10
  introduced object-based
  slicing([1](https://www.macworld.com/article/164061/illustrator-6.html#:~:text=Internet%20design%20firms%20will%20jump,supports%20manual%20slicing%20%E2%80%94%20and)).
  While web slicing is less relevant today, if we want parity we could include
  a basic slicing feature for exporting image assets. Alternatively, we might
  skip this as a conscious decision given modern web design changes – but
  document the decision).

- The **Graph Tool set** (if we skipped in Phase 5 and decide it’s not crucial,
  note it; or if we partially implemented, ensure it’s documented).

- Any **UI improvements** that were in AI10, e.g. Shift to constrain
  proportions (likely we have), or alternate colour models (maybe allow entering
  colours in CMYK or Lab if needed for print parity, though SVG is RGB-based).

- The **Measure Tool** (for measuring distances/angles on the canvas – if not
  yet, add it as a small utility).

- **Plugins or SDK**: Illustrator had a plug-in architecture. We might not
  create a full plugin API, but since Gauss has scripting and is open-source,
  extensibility is inherently there. We can declare scripting as the official
  extension mechanism.

For each minor item, either implement it quickly (if easy and useful) or
explicitly decide to omit if it’s outdated (and perhaps mention alternatives).
The goal is no significant feature of Illustrator 10 goes unaddressed either by
implementation or intentional exclusion.

- **User Experience Polish:** Refine the UI/UX: check that icons (from the
  Zed/Carbon icon set) are all replaced for implemented features (remove
  “greyed out” placeholders now). Ensure tooltips and cursor hints are
  correctly showing for all tools. Balance the layout of panels and toolbars so
  the interface isn’t cluttered – perhaps introduce an **arrangement for
  panels** (dock/undock, collapse) similar to Adobe’s, if needed, or a simpler
  approach like tabbed panels, depending on GPUI Component’s capabilities.
  Also, make sure the **keyboard shortcuts** are comprehensive and match
  familiar defaults (we have been adding them along, but now do a final pass).
  This is an overlooked yet essential aspect – power users expect to operate
  quickly via shortcuts (the Top 100 list we referenced includes shortcuts for
  every tool, which we have largely followed).

- **Accessibility & Localization Finalization:** Run a thorough **accessibility
  audit** now that the app is feature-complete. Test with screen readers (NVDA,
  JAWS, VoiceOver, Orca) on various flows: navigating menus, using tools via
  keyboard, reading properties in panels. Fix any missing labels or focus order
  issues. Make sure any custom UI (e.g. colour picker, gradient editor) has at
  least basic accessibility (e.g. the gradient stops list could be represented
  in the accessibility tree, even if not fully manipulable via keyboard). Aim
  for compliance with relevant standards (WCAG2.1, EN 301 549) as noted in our
  AccessKit design – this could be a selling point for Gauss in certain
  sectors. Likewise, ensure the application can be fully used in different
  languages: by now, we should have a system to load translations, so test with
  a couple of localized UI strings (maybe community-provided). Also verify that
  text input and rendering works for a variety of scripts (Chinese, Arabic,
  etc.) – any major issues (like missing shaping for Arabic) should be
  documented and possibly scheduled for future improvement if beyond scope now.
  The aim is that Gauss 1.0 is usable and welcoming to people with disabilities
  and non-English users, which sets it apart from many developer-focused tools.

- **Performance Tuning:** Do final performance tuning. At this stage, we can
  test Gauss on large real-world SVG files or complex illustrations and profile
  memory and CPU/GPU usage. Identify any slow spots – e.g. maybe the gradient
  mesh rendering is slow, or too many symbol instances cause frame drops.
  Optimize where possible (caching, algorithmic improvements, etc.). Also
  consider memory usage: ensure that large documents don’t leak memory or
  degrade over time. If needed, implement a simple **LOD (Level of Detail)**
  system: e.g. hide very fine details or reduce mesh resolution while
  interacting, to keep interactions smooth on less powerful machines. This
  phase is about **quality**: making sure Gauss is not only feature-rich but
  also dependable and efficient.

- **File Format and Open Standards Evaluation:** Now that all features are
  implemented, critically assess if **SVG as the sole native format** is
  sustainable. SVG covers a lot (shapes, groups, symbols via `<use>`,
  gradients, patterns, text, etc.), but some features might be awkward:

- **Gradient Meshes** are not part of standard SVG 1.1 (and only in SVG 2 as an
  experimental feature). We might be encoding them in a non-standard way or
  losing fidelity on save.

- **Live Effects** (blurs, shadows) can be represented with SVG filter effects
  to some extent, but complex appearance stacks might not round-trip perfectly.

- **Variable data** and unpublished intermediate data (like which object is a
  symbol instance vs expanded, or the internal structure of blend objects)
  might not serialize cleanly to SVG without metadata.

If we find SVG cannot express some Gauss features without hacks or data loss,
plan a transition:

- Consider using **PDF or an AI-like format** as an alternative for saving. PDF
  is an open specification and can handle transparency, meshes (as gradients in
  PDF), etc. However, editing a PDF is not straightforward for external tools.

- Another option is to extend SVG in a custom way or use SVG plus a sidecar
  XML/JSON for Gauss-specific data. Since the prompt suggests transitioning to
  a suitable open standard, we could explore existing open formats: e.g.
  **Figma’s .fig isn’t open,** but perhaps the **Open Design Format (if one
  exists)** or **CSS Paint Worklets**, etc. It might even mean working with the
  W3C SVG group to incorporate needed features.

- As a concrete plan: in this phase we start an **evaluation project** to
  compare potential formats. If one stands out (say, a future SVG 2.0 spec or
  PDF 1.7), we prototype exporting to it and see if it preserves everything. If
  not, we define **“Gauss SVG+”**: basically SVG for all standard stuff, plus
  embedded `<metadata>` blocks for Gauss-specific info (e.g. variable
  definitions, non-exportable guide lines, etc.). This way, a Gauss file is
  still essentially SVG and viewable anywhere, but only Gauss will fully
  interpret the extra info.

- We document the plan for format transition in the roadmap. Perhaps this
  results in Gauss supporting **two save modes**: a pure SVG (for
  interoperability) and a “Gauss document” (which is SVG plus extras or an
  alternate extension like `.gauss` that is basically an archive or JSON).
  Emphasize that any such format will be openly documented – avoiding
  proprietary lock-in.

The outcome is that we won’t blindly stick to SVG if it hampers functionality.
But we also won’t abruptly drop SVG: rather, ensure anything extra we do stays
aligned with open standards philosophy.

- **Avoiding Novelty & Future Ideas:** We explicitly review any features that
  were left out either due to novelty or low value:

- For example, the **Flare Tool** in Illustrator (which just creates a lens
  flare vector graphic) is arguably a novelty feature. If not already done, we
  might decide to *skip it entirely*, or implement it last as a fun add-on. It
  doesn’t drive much real usage, so leaving it out doesn’t harm parity in
  spirit.

- Any other “Easter egg” or hidden features (Illustrator had some, like the Tao
  easter egg or a pixel grid tool) can be safely ignored or deferred.

- We ensure our development focus has consistently avoided chasing things that
  look cool in demos but aren’t truly useful per our priority criteria. This
  final phase is about confirming that: our added features each have clear
  purpose.

If stakeholders request some whimsical feature (say an animated demo mode), we
can always add it after 1.0 as optional, but it’s not part of the core roadmap.

- **Documentation and Release Prep:** Finalize **documentation**: user guides
  (especially for new or complex features like data merge or symbol tools),
  in-app help tooltips, and scripting API references. Possibly create example
  files that show off each major feature (good for testing and as templates for
  new users). Set up the website or README to clearly list Gauss’s capabilities
  relative to Illustrator 10 (by now we can proudly state parity on major
  features). Also, perform a round of **beta testing** with users if possible,
  to catch any usability issues we as developers might have missed.

**Phase 6 Outcome:** Gauss 1.0 is polished and ready. All major and minor
features from Illustrator 10 have been accounted for, and the app is stable,
optimized, and well-documented. We have taken care of cross-cutting concerns to
the extent that Gauss is not just feature-rich but also accessible,
localizable, and automatable out of the box – something few creative tools can
claim. We have a strategy for file format going forward, ensuring that as we
possibly exceed SVG’s limits, we remain grounded in open standards and
portability. At this point Gauss truly realizes its goal: a modern,
cross-platform Illustrator 10 equivalent (and more), built on a cutting-edge
Rust+GPU foundation.

**Sources:**

- Adobe Illustrator 10 new feature highlights (Symbols, Liquify tools,
  data-driven graphics, etc.). [1][illustrator-6]

- Description of key Illustrator 10 tools (selection, pen, type, liquify,
  gradient, mesh, symbol tools)

- GPUI/GPUI Component usage and AccessKit integration guidance

[symbol-sprayer]:
<https://www.macworld.com/article/164061/illustrator-6.html#:~:text=Symbols%20to%20the%20rescue,a%20library%20of%20its%20own>

[illustrator-6]: <https://www.macworld.com/article/164061/illustrator-6.html>
