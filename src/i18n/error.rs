//! Error types for i18n operations.

use std::fmt;

/// Errors that can occur during i18n operations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// The requested message identifier was not found in the catalog.
    MessageNotFound {
        /// The message identifier that was not found.
        message_id: String,
        /// The locale that was queried.
        locale: String,
    },
    /// The requested locale is not supported.
    UnsupportedLocale {
        /// The locale that was requested.
        requested: String,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MessageNotFound { message_id, locale } => write!(
                f,
                "Message '{}' not found in catalog for locale '{}'",
                message_id, locale
            ),
            Self::UnsupportedLocale { requested } => {
                write!(f, "Unsupported locale: '{}'", requested)
            }
        }
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_not_found_displays_correctly() {
        let error = Error::MessageNotFound {
            message_id: "test.key".to_owned(),
            locale: "en-GB".to_owned(),
        };
        let display = format!("{error}");
        assert!(display.contains("test.key"));
        assert!(display.contains("en-GB"));
    }

    #[test]
    fn unsupported_locale_displays_correctly() {
        let error = Error::UnsupportedLocale {
            requested: "invalid".to_owned(),
        };
        let display = format!("{error}");
        assert!(display.contains("invalid"));
    }
}
