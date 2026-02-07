//! Resource store for shared document resources.
//!
//! This module provides storage for reusable resources like gradients, patterns,
//! symbols, and markers. Resources are referenced by ID from style definitions.
//!
//! 0.2.3 introduces concrete storage for:
//!
//! - gradients (`linearGradient` and `radialGradient`)
//! - patterns (`pattern`)
//! - symbols (`symbol`)
//!
//! SVG IDs are tracked alongside typed keys so the engine can keep strong
//! references internally while still round-tripping to SVG URLs (`url(#id)`).

use std::collections::HashMap;

use slotmap::{SlotMap, new_key_type};

use crate::model::{Rgba, Vec2};

new_key_type! {
    /// Identifier for a gradient definition in [`ResourceStore`].
    pub struct GradientId;
    /// Identifier for a pattern definition in [`ResourceStore`].
    pub struct PatternId;
    /// Identifier for a symbol definition in [`ResourceStore`].
    pub struct SymbolId;
}

/// A single gradient stop.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GradientStop {
    /// Offset within the gradient (`0.0..=1.0`).
    pub offset: f32,
    /// Stop colour (including alpha).
    pub colour: Rgba,
}

impl GradientStop {
    /// Construct a gradient stop.
    #[must_use]
    pub const fn new(offset: f32, colour: Rgba) -> Self {
        Self { offset, colour }
    }
}

/// Parameters for a linear gradient.
#[derive(Clone, Debug, PartialEq)]
pub struct LinearGradient {
    /// Start point in normalised object space.
    pub start: Vec2,
    /// End point in normalised object space.
    pub end: Vec2,
    /// Ordered colour stops.
    pub stops: Vec<GradientStop>,
}

impl LinearGradient {
    /// Construct a linear gradient.
    #[must_use]
    pub const fn new(start: Vec2, end: Vec2, stops: Vec<GradientStop>) -> Self {
        Self { start, end, stops }
    }
}

/// Parameters for a radial gradient.
#[derive(Clone, Debug, PartialEq)]
pub struct RadialGradient {
    /// Circle centre in normalised object space.
    pub centre: Vec2,
    /// Radius in normalised object space.
    pub radius: f32,
    /// Optional focal point for non-uniform radial gradients.
    pub focal: Option<Vec2>,
    /// Ordered colour stops.
    pub stops: Vec<GradientStop>,
}

impl RadialGradient {
    /// Construct a radial gradient.
    #[must_use]
    pub const fn new(
        centre: Vec2,
        radius: f32,
        focal: Option<Vec2>,
        stops: Vec<GradientStop>,
    ) -> Self {
        Self {
            centre,
            radius,
            focal,
            stops,
        }
    }
}

/// Gradient payload.
#[derive(Clone, Debug, PartialEq)]
pub enum GradientKind {
    /// A linear gradient.
    Linear(LinearGradient),
    /// A radial gradient.
    Radial(RadialGradient),
}

/// A reusable gradient resource.
#[derive(Clone, Debug, PartialEq)]
pub struct Gradient {
    /// SVG identifier used in `url(#...)` references.
    pub svg_id: String,
    /// Gradient geometry and stop data.
    pub kind: GradientKind,
}

impl Gradient {
    /// Construct a gradient resource.
    #[must_use]
    pub fn new(svg_id: impl Into<String>, kind: GradientKind) -> Self {
        Self {
            svg_id: svg_id.into(),
            kind,
        }
    }
}

/// A reusable pattern resource.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatternResource {
    /// SVG identifier used in `url(#...)` references.
    pub svg_id: String,
    /// Raw child SVG payload inside `<pattern>...</pattern>`.
    pub body: String,
}

impl PatternResource {
    /// Construct a pattern resource.
    #[must_use]
    pub fn new(svg_id: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            svg_id: svg_id.into(),
            body: body.into(),
        }
    }
}

/// A reusable symbol resource.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolResource {
    /// SVG identifier used by `<use href="#...">`.
    pub svg_id: String,
    /// Optional SVG `viewBox`.
    pub view_box: Option<String>,
    /// Raw child SVG payload inside `<symbol>...</symbol>`.
    pub body: String,
}

impl SymbolResource {
    /// Construct a symbol resource.
    #[must_use]
    pub fn new(
        svg_id: impl Into<String>,
        view_box: Option<String>,
        body: impl Into<String>,
    ) -> Self {
        Self {
            svg_id: svg_id.into(),
            view_box,
            body: body.into(),
        }
    }
}

/// Storage for shared document resources.
#[derive(Clone, Debug, Default)]
pub struct ResourceStore {
    gradients: SlotMap<GradientId, Gradient>,
    patterns: SlotMap<PatternId, PatternResource>,
    symbols: SlotMap<SymbolId, SymbolResource>,
    gradient_svg_ids: HashMap<String, GradientId>,
    pattern_svg_ids: HashMap<String, PatternId>,
    symbol_svg_ids: HashMap<String, SymbolId>,
}

impl PartialEq for ResourceStore {
    fn eq(&self, other: &Self) -> bool {
        if self.gradients.len() != other.gradients.len() {
            return false;
        }

        if self.patterns.len() != other.patterns.len() || self.symbols.len() != other.symbols.len()
        {
            return false;
        }

        for (id, gradient) in &self.gradients {
            if other.gradients.get(id) != Some(gradient) {
                return false;
            }
        }

        for (id, pattern) in &self.patterns {
            if other.patterns.get(id) != Some(pattern) {
                return false;
            }
        }

        for (id, symbol) in &self.symbols {
            if other.symbols.get(id) != Some(symbol) {
                return false;
            }
        }

        self.gradient_svg_ids == other.gradient_svg_ids
            && self.pattern_svg_ids == other.pattern_svg_ids
            && self.symbol_svg_ids == other.symbol_svg_ids
    }
}

impl ResourceStore {
    /// Construct an empty resource store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Return whether all resource collections are empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.gradients.is_empty() && self.patterns.is_empty() && self.symbols.is_empty()
    }

    /// Return the number of gradients.
    #[must_use]
    pub fn gradient_count(&self) -> usize {
        self.gradients.len()
    }

    /// Return the number of patterns.
    #[must_use]
    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }

    /// Return the number of symbols.
    #[must_use]
    pub fn symbol_count(&self) -> usize {
        self.symbols.len()
    }

    /// Insert a gradient, normalising duplicate SVG IDs.
    pub fn insert_gradient(&mut self, mut gradient: Gradient) -> GradientId {
        gradient.svg_id = make_unique_id(&gradient.svg_id, "gradient", &self.gradient_svg_ids);
        let svg_id = gradient.svg_id.clone();
        let id = self.gradients.insert(gradient);
        self.gradient_svg_ids.insert(svg_id, id);
        id
    }

    /// Return a gradient by typed identifier.
    #[must_use]
    pub fn gradient(&self, id: GradientId) -> Option<&Gradient> {
        self.gradients.get(id)
    }

    /// Return a gradient identifier by SVG ID.
    #[must_use]
    pub fn gradient_id_for_svg_id(&self, svg_id: &str) -> Option<GradientId> {
        self.gradient_svg_ids.get(svg_id).copied()
    }

    /// Remove a gradient by typed identifier.
    pub fn remove_gradient(&mut self, id: GradientId) -> Option<Gradient> {
        let removed = self.gradients.remove(id)?;
        self.gradient_svg_ids.remove(removed.svg_id.as_str());
        Some(removed)
    }

    /// Iterate all gradients as `(id, gradient)` tuples.
    pub fn gradients(&self) -> impl Iterator<Item = (GradientId, &Gradient)> + '_ {
        self.gradients.iter()
    }

    /// Insert a pattern, normalising duplicate SVG IDs.
    pub fn insert_pattern(&mut self, mut pattern: PatternResource) -> PatternId {
        pattern.svg_id = make_unique_id(&pattern.svg_id, "pattern", &self.pattern_svg_ids);
        let svg_id = pattern.svg_id.clone();
        let id = self.patterns.insert(pattern);
        self.pattern_svg_ids.insert(svg_id, id);
        id
    }

    /// Return a pattern by typed identifier.
    #[must_use]
    pub fn pattern(&self, id: PatternId) -> Option<&PatternResource> {
        self.patterns.get(id)
    }

    /// Return a pattern identifier by SVG ID.
    #[must_use]
    pub fn pattern_id_for_svg_id(&self, svg_id: &str) -> Option<PatternId> {
        self.pattern_svg_ids.get(svg_id).copied()
    }

    /// Remove a pattern by typed identifier.
    pub fn remove_pattern(&mut self, id: PatternId) -> Option<PatternResource> {
        let removed = self.patterns.remove(id)?;
        self.pattern_svg_ids.remove(removed.svg_id.as_str());
        Some(removed)
    }

    /// Iterate all patterns as `(id, pattern)` tuples.
    pub fn patterns(&self) -> impl Iterator<Item = (PatternId, &PatternResource)> + '_ {
        self.patterns.iter()
    }

    /// Insert a symbol, normalising duplicate SVG IDs.
    pub fn insert_symbol(&mut self, mut symbol: SymbolResource) -> SymbolId {
        symbol.svg_id = make_unique_id(&symbol.svg_id, "symbol", &self.symbol_svg_ids);
        let svg_id = symbol.svg_id.clone();
        let id = self.symbols.insert(symbol);
        self.symbol_svg_ids.insert(svg_id, id);
        id
    }

    /// Return a symbol by typed identifier.
    #[must_use]
    pub fn symbol(&self, id: SymbolId) -> Option<&SymbolResource> {
        self.symbols.get(id)
    }

    /// Return a symbol identifier by SVG ID.
    #[must_use]
    pub fn symbol_id_for_svg_id(&self, svg_id: &str) -> Option<SymbolId> {
        self.symbol_svg_ids.get(svg_id).copied()
    }

    /// Remove a symbol by typed identifier.
    pub fn remove_symbol(&mut self, id: SymbolId) -> Option<SymbolResource> {
        let removed = self.symbols.remove(id)?;
        self.symbol_svg_ids.remove(removed.svg_id.as_str());
        Some(removed)
    }

    /// Iterate all symbols as `(id, symbol)` tuples.
    pub fn symbols(&self) -> impl Iterator<Item = (SymbolId, &SymbolResource)> + '_ {
        self.symbols.iter()
    }
}

fn make_unique_id<T>(requested: &str, prefix: &str, existing: &HashMap<String, T>) -> String {
    let trimmed = requested.trim();
    let base = if trimmed.is_empty() {
        prefix.to_owned()
    } else {
        trimmed.to_owned()
    };

    if !existing.contains_key(base.as_str()) {
        return base;
    }

    let mut suffix = 1_u32;
    loop {
        let candidate = format!("{base}-{suffix}");
        if !existing.contains_key(candidate.as_str()) {
            return candidate;
        }
        suffix += 1;
    }
}
