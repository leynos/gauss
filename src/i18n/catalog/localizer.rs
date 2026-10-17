//! Locale-specific lookup and fallback for message catalogues.

use std::collections::HashMap;

use super::{Catalog, I18nError, Locale, MessageId};

/// Service for looking up localized messages with fallback behaviour.
///
/// The `Localizer` holds multiple locale-specific catalogues and provides
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
    /// Create a new localizer with the given catalogues and default locale.
    ///
    /// # Panics
    ///
    /// Panics if the `default_locale` is not present in the `catalogs` `HashMap`.
    /// The default locale must have a corresponding catalogue for fallback to work.
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
    pub fn with_catalogs(catalogs: HashMap<Locale, Catalog>, default_locale: Locale) -> Self {
        assert!(
            catalogs.contains_key(&default_locale),
            "default_locale must have a corresponding catalog in catalogs"
        );
        Self {
            catalogs,
            default_locale,
        }
    }

    /// Create a default localizer with only the en-GB catalogue.
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

    /// Add or replace a catalogue for a specific locale.
    ///
    /// This method is primarily used for testing.
    pub fn add_catalog(&mut self, locale: Locale, catalog: Catalog) {
        self.catalogs.insert(locale, catalog);
    }

    /// Check if a specific locale's catalogue contains a message.
    ///
    /// Returns `true` if the locale has a catalogue and that catalogue contains
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
    /// locale. Returns an error if the message is not found in either catalogue.
    ///
    /// # Errors
    ///
    /// Returns [`I18nError::MessageNotFound`] when the requested catalogue and,
    /// if needed, the fallback catalogue do not contain the message identifier.
    /// The error reports the effective locale: the default locale when lookup
    /// falls back.
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
