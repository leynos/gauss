//! Draw-shape lookup for GPUI integration tests.

use gauss::model::{Document, Shape};
use test_support::{TestSupportError, TestSupportResult};

pub fn require_draw_shape<'a>(doc: &'a Document, context: &str) -> TestSupportResult<&'a Shape> {
    let demo_id = doc.shape_id_at(0);
    let shape = doc
        .iter_in_draw_order()
        .find(|shape| Some(shape.id) != demo_id);
    shape.ok_or_else(|| TestSupportError::missing("shape", format!("draw shape: {context}")))
}
