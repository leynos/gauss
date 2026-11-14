//! Compile-pass fixture for the fixture lint attribute.
//!
//! The crate denies `unused_braces`, so this file fails to compile unless the
//! attribute scopes the allowance around the compacted fixture expansion.

#![deny(unused_braces)]

use gauss_test_macros::allow_fixture_expansion_lints;
use rstest::fixture;

/// Fixture value returned by the compacted fixture.
pub struct World {
    /// Proves the compacted body ran.
    pub ready: bool,
}

impl Default for World {
    fn default() -> Self { Self { ready: true } }
}

#[allow_fixture_expansion_lints]
#[fixture]
fn world() -> World { World::default() }

fn main() {
    assert!(world().ready);
}