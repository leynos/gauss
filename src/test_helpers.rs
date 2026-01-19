//! Test helper utilities for Gauss fixtures.

use crate::model::ShapeId;

const TEST_ID_VERSION: u64 = 0xffff_fffe;

/// Create a deterministic `ShapeId` from a seed value.
///
/// # Examples
///
/// ```
/// # use gauss::test_helpers::shape_id_from_seed;
/// let id = shape_id_from_seed(42);
/// assert_ne!(id, shape_id_from_seed(43));
/// ```
#[must_use]
pub fn shape_id_from_seed(seed: u128) -> ShapeId {
    let idx = u32::try_from(seed).unwrap_or(u32::MAX);
    let raw = (TEST_ID_VERSION << 32) | u64::from(idx);
    ShapeId::from_accesskit_node_id(raw)
}
