//! Message catalog storage and lookup.

use std::collections::HashMap;

use super::{I18nError, Locale, MessageId};

/// A message catalog for a specific locale.
///
/// Catalogs map message identifiers to localized strings. Each catalog
/// corresponds to a single locale (e.g., en-GB, fr-FR).
///
/// # Examples
///
/// ```rust
/// use std::collections::HashMap;
/// use gauss::i18n::{Catalog, MessageId};
///
/// let mut messages = HashMap::new();
/// messages.insert("greeting".to_owned(), "Hello".to_owned());
/// let catalog = Catalog::from_messages(messages);
///
/// let msg = catalog.get(&MessageId::from("greeting"));
/// assert_eq!(msg, Some("Hello"));
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Catalog {
    messages: HashMap<String, String>,
}

impl Catalog {
    /// Create a new catalog from a map of message identifiers to strings.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use gauss::i18n::Catalog;
    ///
    /// let mut messages = HashMap::new();
    /// messages.insert("key".to_owned(), "value".to_owned());
    /// let catalog = Catalog::from_messages(messages);
    /// ```
    #[must_use]
    pub const fn from_messages(messages: HashMap<String, String>) -> Self {
        Self { messages }
    }

    /// Look up a message by its identifier.
    ///
    /// Returns `Some(&str)` if the message exists, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use gauss::i18n::{Catalog, MessageId};
    ///
    /// let mut messages = HashMap::new();
    /// messages.insert("found".to_owned(), "Found!".to_owned());
    /// let catalog = Catalog::from_messages(messages);
    ///
    /// assert_eq!(catalog.get(&MessageId::from("found")), Some("Found!"));
    /// assert_eq!(catalog.get(&MessageId::from("missing")), None);
    /// ```
    #[must_use]
    pub fn get(&self, message_id: &MessageId) -> Option<&str> {
        self.messages.get(message_id.as_str()).map(String::as_str)
    }

    /// Create the default English (en-GB) catalog.
    ///
    /// This catalog contains the baseline translations for Gauss.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gauss::i18n::{Catalog, MessageId};
    ///
    /// let catalog = Catalog::default_en_gb();
    /// let draw = catalog.get(&MessageId::tool_mode_draw());
    /// assert_eq!(draw, Some("Draw"));
    /// ```
    #[must_use]
    pub fn default_en_gb() -> Self {
        let mut messages = HashMap::new();
        messages.insert("tool_mode.draw".to_owned(), "Draw".to_owned());
        messages.insert("tool_mode.manipulate".to_owned(), "Manipulate".to_owned());
        messages.insert("edge_mode.line".to_owned(), "Line".to_owned());
        messages.insert(
            "edge_mode.bezier_auto".to_owned(),
            "Bezier (auto)".to_owned(),
        );
        // Status templates for accessibility tree labels
        messages.insert(
            "tool.status.mode_with_edge".to_owned(),
            "Mode: {tool} ({edge})".to_owned(),
        );
        messages.insert("tool.status.mode".to_owned(), "Mode: {tool}".to_owned());
        Self::from_messages(messages)
    }
}

/// Service for looking up localized messages with fallback behaviour.
///
/// The `Localizer` holds multiple locale-specific catalogs and provides
/// lookup with automatic fallback to the default locale (en-GB) when a
/// requested locale is unavailable.
///
/// # Examples
///
/// ```rust
/// use std::collections::HashMap;
/// use gauss::i18n::{Catalog, Locale, Localizer, MessageId};
///
/// let mut catalogs = HashMap::new();
/// catalogs.insert(Locale::en_gb(), Catalog::default_en_gb());
///
/// let localizer = Localizer::with_catalogs(catalogs, Locale::en_gb());
/// let msg = localizer.lookup(&Locale::en_gb(), &MessageId::tool_mode_draw());
/// assert!(msg.is_ok());
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Localizer {
    catalogs: HashMap<Locale, Catalog>,
    default_locale: Locale,
}

impl Localizer {
    /// Create a new localizer with the given catalogs and default locale.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use gauss::i18n::{Catalog, Locale, Localizer};
    ///
    /// let mut catalogs = HashMap::new();
    /// catalogs.insert(Locale::en_gb(), Catalog::default_en_gb());
    ///
    /// let localizer = Localizer::with_catalogs(catalogs, Locale::en_gb());
    /// ```
    #[must_use]
    pub const fn with_catalogs(catalogs: HashMap<Locale, Catalog>, default_locale: Locale) -> Self {
        Self {
            catalogs,
            default_locale,
        }
    }

    /// Create a default localizer with only the en-GB catalog.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gauss::i18n::{Locale, Localizer, MessageId};
    ///
    /// let localizer = Localizer::default();
    /// let msg = localizer.lookup(&Locale::en_gb(), &MessageId::tool_mode_draw());
    /// assert!(msg.is_ok());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        let mut catalogs = HashMap::new();
        catalogs.insert(Locale::en_gb(), Catalog::default_en_gb());
        Self::with_catalogs(catalogs, Locale::en_gb())
    }

    /// Add or replace a catalog for a specific locale.
    ///
    /// This method is primarily used for testing.
    pub fn add_catalog(&mut self, locale: Locale, catalog: Catalog) {
        self.catalogs.insert(locale, catalog);
    }

    /// Check if a specific locale's catalog contains a message.
    ///
    /// Returns `true` if the locale has a catalog and that catalog contains
    /// the message, `false` otherwise (including if the locale doesn't exist).
    #[must_use]
    pub fn catalog_contains(&self, locale: &Locale, message_id: &MessageId) -> bool {
        self.catalogs
            .get(locale)
            .is_some_and(|catalog| catalog.get(message_id).is_some())
    }

    /// Look up a message for the given locale and message identifier.
    ///
    /// If the requested locale is not available, falls back to the default
    /// locale. Returns an error if the message is not found in either catalog.
    ///
    /// # Errors
    ///
    /// Returns [`I18nError::UnsupportedLocale`] if neither the requested locale
    /// nor the default locale are available. Returns
    /// [`I18nError::MessageNotFound`] if the message identifier is not found in
    /// the catalog.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gauss::i18n::{Locale, Localizer, MessageId};
    ///
    /// let localizer = Localizer::default();
    /// let msg = localizer.lookup(&Locale::en_gb(), &MessageId::tool_mode_draw());
    /// assert_eq!(msg.unwrap(), "Draw");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the message is not found in the requested locale or
    /// the fallback locale, or if the locale is not supported.
    ///
    /// When falling back to the default locale, errors report the effective
    /// catalog's locale (i.e. the locale that was actually queried).
    pub fn lookup(&self, locale: &Locale, message_id: &MessageId) -> Result<String, I18nError> {
        // First, try the requested locale's catalog if it exists
        if let Some(catalog) = self.catalogs.get(locale)
            && let Some(message) = catalog.get(message_id)
        {
            return Ok(message.to_owned());
        }

        // If not found and requested locale differs from default, try default catalog
        if locale != &self.default_locale
            && let Some(default_catalog) = self.catalogs.get(&self.default_locale)
        {
            return default_catalog.get(message_id).map_or_else(
                || {
                    // Message not found in default catalog either
                    Err(I18nError::MessageNotFound {
                        message_id: message_id.as_str().to_owned(),
                        locale: self.default_locale.language_tag().to_owned(),
                    })
                },
                |message| Ok(message.to_owned()),
            );
        }

        // Either:
        // 1. Requested locale == default locale, but message not found, or
        // 2. Requested locale != default, but neither catalog has the message, or
        // 3. No catalog exists for either locale
        let effective_locale = if self.catalogs.contains_key(locale) {
            locale
        } else if self.catalogs.contains_key(&self.default_locale) {
            &self.default_locale
        } else {
            return Err(I18nError::UnsupportedLocale {
                requested: locale.language_tag().to_owned(),
            });
        };

        Err(I18nError::MessageNotFound {
            message_id: message_id.as_str().to_owned(),
            locale: effective_locale.language_tag().to_owned(),
        })
    }
}

impl Default for Localizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use rstest::{fixture, rstest};

    use super::*;

    #[test]
    fn catalog_from_messages_creates_correctly() {
        let mut messages = HashMap::new();
        messages.insert("test".to_owned(), "value".to_owned());
        let catalog = Catalog::from_messages(messages);
        assert_eq!(catalog.get(&MessageId::from("test")), Some("value"));
    }

    #[test]
    fn catalog_get_returns_none_for_missing_key() {
        let catalog = Catalog::from_messages(HashMap::new());
        assert_eq!(catalog.get(&MessageId::from("missing")), None);
    }

    #[test]
    fn catalog_default_en_gb_contains_tool_modes() {
        let catalog = Catalog::default_en_gb();
        assert_eq!(catalog.get(&MessageId::tool_mode_draw()), Some("Draw"));
        assert_eq!(
            catalog.get(&MessageId::tool_mode_manipulate()),
            Some("Manipulate")
        );
    }

    #[test]
    fn catalog_default_en_gb_contains_edge_modes() {
        let catalog = Catalog::default_en_gb();
        assert_eq!(catalog.get(&MessageId::edge_mode_line()), Some("Line"));
        assert_eq!(
            catalog.get(&MessageId::edge_mode_bezier_auto()),
            Some("Bezier (auto)")
        );
    }

    #[test]
    fn localizer_new_creates_with_default_catalog() {
        let localizer = Localizer::new();
        let result = localizer.lookup(&Locale::en_gb(), &MessageId::tool_mode_draw());
        assert!(result.is_ok());
        assert_eq!(result.expect("Should have found message"), "Draw");
    }

    #[fixture]
    fn fr_test_catalog() -> Catalog {
        let mut messages = HashMap::new();
        messages.insert("test".to_owned(), "test_fr".to_owned());
        Catalog::from_messages(messages)
    }

    #[fixture]
    fn fr_test_localizer(fr_test_catalog: Catalog) -> Localizer {
        let mut catalogs = HashMap::new();
        catalogs.insert(Locale::fr_fr(), fr_test_catalog);
        Localizer::with_catalogs(catalogs, Locale::en_gb())
    }

    #[rstest]
    fn localizer_lookup_succeeds_for_available_locale(fr_test_localizer: Localizer) {
        let result = fr_test_localizer.lookup(&Locale::fr_fr(), &MessageId::from("test"));
        assert!(result.is_ok());
        assert_eq!(result.expect("Should have found message"), "test_fr");
    }

    #[test]
    fn localizer_lookup_falls_back_to_default_locale() {
        let mut catalogs = HashMap::new();
        catalogs.insert(Locale::en_gb(), Catalog::default_en_gb());

        let localizer = Localizer::with_catalogs(catalogs, Locale::en_gb());
        let result = localizer.lookup(&Locale::fr_fr(), &MessageId::tool_mode_draw());
        assert!(result.is_ok());
        assert_eq!(result.expect("Should have found message"), "Draw");
    }

    #[test]
    fn localizer_lookup_returns_error_for_missing_message() {
        let localizer = Localizer::new();
        let result = localizer.lookup(&Locale::en_gb(), &MessageId::from("nonexistent"));
        assert!(result.is_err());
        match result.expect_err("Should have returned error") {
            I18nError::MessageNotFound { message_id, .. } => {
                assert_eq!(message_id, "nonexistent");
            }
            I18nError::UnsupportedLocale { .. } => {
                panic!("Expected MessageNotFound error")
            }
        }
    }

    #[rstest]
    fn localizer_add_catalog_works(fr_test_catalog: Catalog) {
        let mut localizer = Localizer::new();
        localizer.add_catalog(Locale::fr_fr(), fr_test_catalog);
        let result = localizer.lookup(&Locale::fr_fr(), &MessageId::from("test"));
        assert!(result.is_ok());
        assert_eq!(result.expect("Should have found message"), "test_fr");
    }
}
