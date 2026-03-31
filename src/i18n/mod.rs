//! Internationalization (i18n) module for Gauss.
//!
//! This module provides message catalog lookup, locale handling, and fallback
//! behaviour for localized strings. The implementation is GPUI-independent and
//! uses a simple keyed catalog system.
//!
//! The module is designed to support future migration to more sophisticated
//! localization systems (such as Fluent) without breaking client code, as all
//! lookup operations go through stable public APIs.
//!
//! # Architecture
//!
//! The i18n module consists of four main components:
//!
//! - [`Locale`]: Represents a locale identifier using BCP 47 language tags.
//! - [`MessageId`]: Stable message identifiers for catalog lookups.
//! - [`Catalog`]: Storage for locale-specific message translations.
//! - [`Localizer`]: Service that provides lookup with automatic fallback.
//!
//! # Example
//!
//! ```rust
//! use gauss::i18n::{Locale, Localizer, MessageId};
//!
//! let localizer = Localizer::default();
//! let locale = Locale::en_gb();
//! let message_id = MessageId::tool_mode_draw();
//!
//! match localizer.lookup(&locale, &message_id) {
//!     Ok(text) => println!("Tool mode: {}", text),
//!     Err(err) => eprintln!("Lookup failed: {}", err),
//! }
//! ```

#[expect(
    unknown_lints,
    reason = "module_max_lines is a custom lint from whitaker tool"
)]
#[expect(
    module_max_lines,
    reason = "Message catalog is a data table; splitting would fragment related entries"
)]
mod catalog;
mod error;
mod locale;
#[expect(
    unknown_lints,
    reason = "module_max_lines is a custom lint from whitaker tool"
)]
#[expect(
    module_max_lines,
    reason = "MessageId factory methods are a flat lookup table; splitting would harm discoverability"
)]
mod message;

pub use catalog::{Catalog, Localizer};
pub use error::I18nError;
pub use locale::Locale;
pub use message::MessageId;
