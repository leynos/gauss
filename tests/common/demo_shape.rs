//! Demo-shape lookup for GPUI integration tests.

use gauss::model::{Document, ShapeId};

/// Returns the first shape's ID, or `None` when the document is empty.
pub fn demo_shape_id(doc: &Document) -> Option<ShapeId> {
    doc.shape_id_at(0)
}
