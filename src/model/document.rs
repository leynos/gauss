//! Document structure and shape ordering.

use std::collections::HashSet;

use slotmap::{Key, SlotMap};

use crate::model::{Shape, ShapeId};

/// A vector document consisting of an ordered list of shapes.
#[derive(Clone, Debug)]
pub struct Document {
    shapes: Vec<Shape>,
    id_registry: SlotMap<ShapeId, ()>,
    /// IDs currently assigned to shapes in this document.
    active_ids: HashSet<ShapeId>,
    /// IDs reserved for this document to avoid allocator collisions.
    reserved_ids: HashSet<ShapeId>,
}

impl Document {
    /// Construct an empty document.
    #[must_use]
    pub fn new() -> Self {
        Self {
            shapes: Vec::new(),
            id_registry: SlotMap::with_key(),
            active_ids: HashSet::new(),
            reserved_ids: HashSet::new(),
        }
    }

    /// Return the number of shapes in the document.
    #[expect(
        clippy::missing_const_for_fn,
        reason = "Vec::len() is not const in stable Rust"
    )]
    #[must_use]
    pub fn len(&self) -> usize {
        self.shapes.len()
    }

    /// Allocate a new stable shape identifier.
    pub fn allocate_shape_id(&mut self) -> ShapeId {
        loop {
            let id = self.id_registry.insert(());
            if self.reserved_ids.insert(id) {
                return id;
            }
        }
    }

    /// Return whether the document contains no shapes.
    #[expect(
        clippy::missing_const_for_fn,
        reason = "Vec::is_empty() is not const in stable Rust"
    )]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.shapes.is_empty()
    }

    /// Return an iterator over shapes in draw order.
    pub fn iter_in_draw_order(&self) -> std::slice::Iter<'_, Shape> {
        self.shapes.iter()
    }

    /// Return an iterator over shape IDs in draw order.
    pub fn iter_ids_in_draw_order(&self) -> impl Iterator<Item = ShapeId> + '_ {
        self.shapes.iter().map(|shape| shape.id)
    }

    /// Return the index of the shape with the given id.
    #[must_use]
    pub fn find_index(&self, id: ShapeId) -> Option<usize> {
        self.shapes.iter().position(|shape| shape.id == id)
    }

    /// Return the shape with the given id.
    #[must_use]
    pub fn shape(&self, id: ShapeId) -> Option<&Shape> {
        let index = self.find_index(id)?;
        self.shapes.get(index)
    }

    /// Return a mutable reference to the shape with the given id.
    #[must_use]
    pub fn shape_mut(&mut self, id: ShapeId) -> Option<&mut Shape> {
        let index = self.find_index(id)?;
        self.shapes.get_mut(index)
    }

    /// Return the shape at the given draw-order index.
    #[must_use]
    pub fn shape_at(&self, index: usize) -> Option<&Shape> {
        self.shapes.get(index)
    }

    /// Return a mutable reference to the shape at the given draw-order index.
    #[must_use]
    pub fn shape_at_mut(&mut self, index: usize) -> Option<&mut Shape> {
        self.shapes.get_mut(index)
    }

    /// Return the shape id at the given draw-order index.
    #[must_use]
    pub fn shape_id_at(&self, index: usize) -> Option<ShapeId> {
        self.shapes.get(index).map(|shape| shape.id)
    }

    /// Insert a shape at the given draw-order index.
    ///
    /// Returns the assigned [`ShapeId`] when insertion succeeds.
    pub fn insert_shape(&mut self, index: usize, mut shape: Shape) -> Option<ShapeId> {
        if index > self.shapes.len() {
            return None;
        }

        let id = self.ensure_shape_id(&mut shape);
        self.shapes.insert(index, shape);
        Some(id)
    }

    /// Append a shape to the end of the draw order.
    ///
    /// Returns the assigned [`ShapeId`].
    pub fn append_shape(&mut self, mut shape: Shape) -> ShapeId {
        let id = self.ensure_shape_id(&mut shape);
        self.shapes.push(shape);
        id
    }

    /// Remove the shape at the given draw-order index.
    ///
    /// Returns the removed shape when the index is in bounds.
    pub fn remove_shape(&mut self, index: usize) -> Option<Shape> {
        if index >= self.shapes.len() {
            return None;
        }
        let shape = self.shapes.remove(index);
        self.active_ids.remove(&shape.id);
        Some(shape)
    }

    /// Remove the shape with the given id.
    ///
    /// Returns the removed shape when the id is present.
    pub fn remove_shape_by_id(&mut self, id: ShapeId) -> Option<Shape> {
        let index = self.find_index(id)?;
        let shape = self.shapes.remove(index);
        self.active_ids.remove(&shape.id);
        Some(shape)
    }

    /// Clear all shapes from the document and reset the ID registry.
    pub fn clear(&mut self) {
        self.shapes.clear();
        self.id_registry = SlotMap::with_key();
        self.active_ids.clear();
        self.reserved_ids.clear();
    }

    /// Reorder a shape by draw-order indices.
    pub fn reorder(&mut self, from: usize, to: usize) -> bool {
        if from == to {
            return true;
        }

        if from >= self.shapes.len() || to > self.shapes.len() {
            return false;
        }

        let shape = self.shapes.remove(from);
        let insert_at = std::cmp::min(to, self.shapes.len());
        self.shapes.insert(insert_at, shape);
        true
    }

    /// Ensure the shape has a unique ID, reserving external IDs to avoid collisions.
    fn ensure_shape_id(&mut self, shape: &mut Shape) -> ShapeId {
        if shape.id.is_null() {
            let id = self.allocate_shape_id();
            shape.id = id;
            let _active = self.active_ids.insert(id);
            return id;
        }

        if self.active_ids.contains(&shape.id) {
            let id = self.allocate_shape_id();
            shape.id = id;
            let _active = self.active_ids.insert(id);
            return id;
        }

        let _reserved = self.reserved_ids.insert(shape.id);
        let _active = self.active_ids.insert(shape.id);
        shape.id
    }
}

// `PartialEq` compares visual content only; registry state is ignored.
impl PartialEq for Document {
    fn eq(&self, other: &Self) -> bool {
        self.visually_eq(other)
    }
}

impl Document {
    /// Compare documents by visual content (shape order and properties).
    #[must_use]
    pub fn visually_eq(&self, other: &Self) -> bool {
        self.shapes == other.shapes
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    //! Tests for document ID management and ordering.

    use super::Document;
    use crate::model::{PaintStyle, PathGeom, Shape, ShapeId};
    use rstest::rstest;

    fn sample_shape(id: ShapeId) -> Shape {
        Shape {
            id,
            z: 0,
            style: PaintStyle::new(None, 1.0, None),
            path: PathGeom::new(),
            name: None,
            locked: false,
            hidden: false,
            gauss_metadata: Vec::new(),
        }
    }

    #[rstest]
    fn append_shape_preserves_existing_id() {
        let mut doc = Document::new();
        let existing_id = ShapeId::from_accesskit_node_id(0xffff_fffe_0000_0001);
        let returned = doc.append_shape(sample_shape(existing_id));

        assert_eq!(returned, existing_id);
        assert_eq!(doc.shape_id_at(0), Some(existing_id));
    }

    #[rstest]
    fn allocate_shape_id_avoids_existing_external_id() {
        let mut doc = Document::new();
        let existing_id = ShapeId::from_accesskit_node_id(0x0000_0001_0000_0001);
        doc.append_shape(sample_shape(existing_id));

        let new_id = doc.allocate_shape_id();
        assert_ne!(new_id, existing_id);
    }

    #[rstest]
    fn append_shape_assigns_stable_ids_across_reorder() {
        let mut doc = Document::new();
        let first_id = doc.append_shape(sample_shape(ShapeId::default()));
        let second_id = doc.append_shape(sample_shape(ShapeId::default()));

        assert_eq!(doc.shape_id_at(0), Some(first_id));
        assert_eq!(doc.shape_id_at(1), Some(second_id));

        assert!(doc.reorder(0, 1), "reorder should succeed");
        assert_eq!(doc.shape_id_at(0), Some(second_id));
        assert_eq!(doc.shape_id_at(1), Some(first_id));
    }

    #[rstest]
    fn insert_shape_out_of_bounds_returns_none() {
        let mut doc = Document::new();
        let result = doc.insert_shape(1, sample_shape(ShapeId::default()));

        assert!(result.is_none());
        assert!(doc.is_empty());
    }

    #[rstest]
    fn accesskit_node_id_round_trip() {
        let mut doc = Document::new();
        let id = doc.append_shape(sample_shape(ShapeId::default()));

        let round_tripped = ShapeId::from_accesskit_node_id(id.to_accesskit_node_id());
        assert_eq!(round_tripped, id);
    }
}
