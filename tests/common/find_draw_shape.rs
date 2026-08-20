//! Optional draw-shape lookup for GPUI integration tests.

use gauss::model::{Document, Shape};

/// Borrows the first shape other than the document's initial demo shape.
///
/// Returns `None` when the document has no first shape or no subsequent drawn
/// shape.
pub fn find_draw_shape(doc: &Document) -> Option<&Shape> {
    let demo_id = doc.shape_id_at(0)?;
    doc.iter_in_draw_order().find(|shape| shape.id != demo_id)
}
