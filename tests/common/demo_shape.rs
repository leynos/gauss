//! Demo-shape lookup for GPUI integration tests.

use gauss::model::{Document, ShapeId};

pub fn demo_shape_id(doc: &Document) -> Option<ShapeId> {
    doc.shape_id_at(0)
}
