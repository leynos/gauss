//! Error types for i18n operations.

/// Errors that can occur during i18n operations.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum I18nError {
    /// The requested message identifier was not found in the catalog.
    #[error("Message '{message_id}' not found in catalog for locale '{locale}'")]
    MessageNotFound {
        /// The message identifier that was not found.
        message_id: String,
        /// The locale that was queried.
        locale: String,
    },
    /// The requested locale is not supported.
    #[error("Unsupported locale: '{requested}'")]
    UnsupportedLocale {
        /// The locale that was requested.
        requested: String,
    },
}

#[cfg(test)]
mod tests {
    //! Unit tests for i18n error types and display formatting.

    use super::*;

    #[test]
    fn message_not_found_displays_correctly() {
        let error = I18nError::MessageNotFound {
            message_id: "test.key".to_owned(),
            locale: "en-GB".to_owned(),
        };
        let display = format!("{error}");
        assert!(display.contains("test.key"));
        assert!(display.contains("en-GB"));
    }

    #[test]
    fn unsupported_locale_displays_correctly() {
        let error = I18nError::UnsupportedLocale {
            requested: "invalid".to_owned(),
        };
        let display = format!("{error}");
        assert!(display.contains("invalid"));
    }
}
