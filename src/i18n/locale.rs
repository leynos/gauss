//! Locale handling for internationalization.

use std::fmt;

/// Represents a locale identifier.
///
/// Locales are represented using BCP 47 language tags (e.g., "en-GB", "fr-FR").
/// This implementation uses simple string-based comparison and does not perform
/// sophisticated locale matching or fallback beyond exact matches.
///
/// # Examples
///
/// ```rust
/// use gauss::i18n::Locale;
///
/// let locale = Locale::en_gb();
/// assert_eq!(locale.language_tag(), "en-GB");
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Locale {
    language_tag: String,
}

impl Locale {
    /// Create a new locale from a BCP 47 language tag.
    ///
    /// The tag is canonicalised to lowercase for consistent `HashMap` lookups.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gauss::i18n::Locale;
    ///
    /// let locale = Locale::from_language_tag("en-GB");
    /// assert_eq!(locale.language_tag(), "en-gb");
    ///
    /// let locale_lower = Locale::from_language_tag("fr-fr");
    /// let locale_upper = Locale::from_language_tag("fr-FR");
    /// assert_eq!(locale_lower, locale_upper);
    /// ```
    #[must_use]
    pub fn from_language_tag(tag: &str) -> Self {
        Self {
            language_tag: tag.to_ascii_lowercase(),
        }
    }

    /// Return the BCP 47 language tag for this locale.
    ///
    /// The returned tag is lowercase (canonicalised).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gauss::i18n::Locale;
    ///
    /// let locale = Locale::fr_fr();
    /// assert_eq!(locale.language_tag(), "fr-fr");
    /// ```
    #[must_use]
    pub fn language_tag(&self) -> &str {
        &self.language_tag
    }

    /// Create a locale for British English (en-GB).
    ///
    /// This is the default locale for Gauss.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gauss::i18n::Locale;
    ///
    /// let locale = Locale::en_gb();
    /// assert_eq!(locale.language_tag(), "en-gb");
    /// ```
    #[must_use]
    pub fn en_gb() -> Self {
        Self::from_language_tag("en-GB")
    }

    /// Create a locale for French (fr-FR).
    ///
    /// This is primarily used for testing.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gauss::i18n::Locale;
    ///
    /// let locale = Locale::fr_fr();
    /// assert_eq!(locale.language_tag(), "fr-fr");
    /// ```
    #[must_use]
    pub fn fr_fr() -> Self {
        Self::from_language_tag("fr-FR")
    }
}

impl Default for Locale {
    fn default() -> Self {
        Self::en_gb()
    }
}

impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.language_tag)
    }
}

impl From<&str> for Locale {
    fn from(tag: &str) -> Self {
        Self::from_language_tag(tag)
    }
}

impl AsRef<str> for Locale {
    fn as_ref(&self) -> &str {
        &self.language_tag
    }
}

#[cfg(test)]
mod tests {
    //! Unit tests for locale parsing and language tag handling.

    use super::*;

    #[test]
    fn locale_from_language_tag_creates_correctly() {
        let locale = Locale::from_language_tag("de-DE");
        assert_eq!(locale.language_tag(), "de-de");
    }

    #[test]
    fn locale_en_gb_creates_correctly() {
        let locale = Locale::en_gb();
        assert_eq!(locale.language_tag(), "en-gb");
    }

    #[test]
    fn locale_fr_fr_creates_correctly() {
        let locale = Locale::fr_fr();
        assert_eq!(locale.language_tag(), "fr-fr");
    }

    #[test]
    fn locale_default_is_en_gb() {
        let locale = Locale::default();
        assert_eq!(locale.language_tag(), "en-gb");
    }

    #[test]
    fn locale_display_shows_language_tag() {
        let locale = Locale::from_language_tag("es-ES");
        assert_eq!(format!("{locale}"), "es-es");
    }

    #[test]
    fn locale_equality_works() {
        let locale1 = Locale::en_gb();
        let locale2 = Locale::from_language_tag("en-GB");
        let locale3 = Locale::from_language_tag("en-gb");
        assert_eq!(locale1, locale2);
        assert_eq!(locale1, locale3);
        assert_eq!(locale2, locale3);
    }
}
