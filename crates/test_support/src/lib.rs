//! Shared error types for test fixtures and helpers.

use std::num::TryFromIntError;
use std::{io, result};

use thiserror::Error;

pub mod fixtures;
pub mod math;
pub mod selection;
pub mod shapes;
pub use shapes::{line_shape, open_triangle, sample_shape, shape_id, shape_with_handles};

/// A convenient result type for test helper operations.
pub type TestSupportResult<T> = result::Result<T, TestSupportError>;

/// Structured errors for test fixtures that must not panic.
#[derive(Debug, Error)]
pub enum TestSupportError {
    /// Required data was missing when a test helper attempted to read it.
    #[error("missing {kind}: {context}")]
    Missing {
        /// The category of data that was expected.
        kind: String,
        /// Additional context describing where the data was needed.
        context: String,
    },
    /// A test assertion failed inside a fixture helper.
    #[error("expectation failed: {context}")]
    Expectation {
        /// Additional context describing the failed expectation.
        context: String,
    },
    /// Converting a value to an `i32` failed for a test fixture.
    #[error("i32 conversion failed for {context}: {source}")]
    ZOrderOverflow {
        /// Additional context describing the failed conversion.
        context: String,
        /// The underlying integer conversion error.
        #[source]
        source: TryFromIntError,
    },
    /// An I/O operation failed in test support code.
    #[error("io error during {context}: {source}")]
    Io {
        /// Additional context describing the I/O operation.
        context: String,
        /// The underlying I/O error.
        #[source]
        source: io::Error,
    },
}

impl TestSupportError {
    /// Create a missing-data error with useful context.
    pub fn missing(kind: impl Into<String>, context: impl Into<String>) -> Self {
        Self::Missing {
            kind: kind.into(),
            context: context.into(),
        }
    }

    /// Create a failed-expectation error with useful context.
    pub fn expectation(context: impl Into<String>) -> Self {
        Self::Expectation {
            context: context.into(),
        }
    }

    /// Wrap a failed conversion to `i32` for z-ordering.
    pub fn z_order_overflow(context: impl Into<String>, source: TryFromIntError) -> Self {
        Self::ZOrderOverflow {
            context: context.into(),
            source,
        }
    }

    /// Wrap an I/O failure with context.
    pub fn io(context: impl Into<String>, source: io::Error) -> Self {
        Self::Io {
            context: context.into(),
            source,
        }
    }
}
