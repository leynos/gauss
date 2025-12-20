//! Document structure and shape ordering.

use crate::model::{Shape, ShapeId};

/// A vector document consisting of an ordered list of shapes.
#[derive(Clone, Debug, PartialEq)]
pub struct Document {
    /// Shapes in draw order (back to front).
    pub shapes: Vec<Shape>,
}

impl Document {
    /// Construct an empty document.
    #[must_use]
    pub const fn new() -> Self {
        Self { shapes: Vec::new() }
    }

    /// Return the index of the shape with the given id.
    #[must_use]
    pub fn find_index(&self, id: ShapeId) -> Option<usize> {
        self.shapes.iter().position(|shape| shape.id == id)
    }

    /// Return a mutable reference to the shape with the given id.
    #[must_use]
    pub fn get_mut(&mut self, id: ShapeId) -> Option<&mut Shape> {
        let index = self.find_index(id)?;
        self.shapes.get_mut(index)
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}
