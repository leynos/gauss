//! Compile-fail fixture for the fixture lint attribute.
//!
//! This is the pass fixture with the attribute removed. The crate denies
//! `unused_braces`, so the compacted `rstest` expansion must be reported here;
//! that failure is the negative control the retained test pins.

#![deny(unused_braces)]

use rstest::fixture;

/// Fixture value returned by the compacted fixture.
pub struct World {
    /// Proves the compacted body ran.
    pub ready: bool,
}

impl Default for World {
    fn default() -> Self { Self { ready: true } }
}

#[fixture]
fn world() -> World { World::default() }

fn main() {
    assert!(world().ready);
}