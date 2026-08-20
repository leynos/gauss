//! Draw-shape lookup for GPUI integration tests.

use gauss::model::{Document, Shape};
use test_support::{TestSupportError, TestSupportResult};

/// Returns the first drawn shape other than the document's initial demo shape.
///
/// The returned shape borrows `doc`; `context` is included in the diagnostic
/// when no drawn shape exists.
///
/// # Errors
///
/// Returns a missing-shape error when the document contains no non-demo shape.
pub fn require_draw_shape<'a>(doc: &'a Document, context: &str) -> TestSupportResult<&'a Shape> {
    let demo_id = doc.shape_id_at(0);
    let shape = doc
        .iter_in_draw_order()
        .find(|shape| Some(shape.id) != demo_id);
    shape.ok_or_else(|| TestSupportError::missing("shape", format!("draw shape: {context}")))
}
