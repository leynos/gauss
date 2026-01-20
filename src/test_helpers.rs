//! Test helper utilities for Gauss fixtures.

use crate::model::ShapeId;

/// The AccessKit test ID version used for deterministic fixtures.
pub const TEST_ID_VERSION: u64 = 0xffff_fffe;

/// Create a deterministic `ShapeId` from a seed value.
///
/// The low 32 bits of the seed are used to avoid collisions when values exceed
/// `u32::MAX`.
///
///
#[must_use]
pub fn shape_id_from_seed(seed: u128) -> ShapeId {
    let masked = seed & u128::from(u32::MAX);
    // Masking constrains the value to the `u32` range; clippy cannot infer it.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "masking limits the value to the u32 range"
    )]
    let idx = masked as u32;
    let raw = (TEST_ID_VERSION << 32) | u64::from(idx);
    ShapeId::from_accesskit_node_id(raw)
}
