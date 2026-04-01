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
        Self::insert_tool_mode_messages(&mut messages);
        Self::insert_chrome_messages(&mut messages);
        Self::insert_tool_tooltip_messages(&mut messages);
        Self::insert_status_messages(&mut messages);
        Self::insert_align_messages(&mut messages);
        Self::insert_style_messages(&mut messages);
        Self::insert_doc_messages(&mut messages);
        Self::insert_a11y_messages(&mut messages);
        Self::from_messages(messages)
    }

    fn insert_tool_mode_messages(messages: &mut HashMap<String, String>) {
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
    }

    fn insert_chrome_messages(messages: &mut HashMap<String, String>) {
        // Window chrome strings
        messages.insert("chrome.file.new".to_owned(), "New".to_owned());
        messages.insert("chrome.file.open".to_owned(), "Open".to_owned());
        messages.insert("chrome.file.save".to_owned(), "Save".to_owned());
        messages.insert("chrome.file.export_web".to_owned(), "Export Web".to_owned());
        messages.insert(
            "chrome.titlebar.recent".to_owned(),
            "Open recent project".to_owned(),
        );
        messages.insert("chrome.settings".to_owned(), "Settings".to_owned());
        messages.insert("chrome.edit.undo".to_owned(), "Undo".to_owned());
        messages.insert("chrome.edit.redo".to_owned(), "Redo".to_owned());

        // Window control strings
        messages.insert("chrome.window.minimize".to_owned(), "Minimize".to_owned());
        messages.insert("chrome.window.maximize".to_owned(), "Maximize".to_owned());
        messages.insert("chrome.window.close".to_owned(), "Close Window".to_owned());
    }

    fn insert_tool_tooltip_messages(messages: &mut HashMap<String, String>) {
        // Tool tooltip strings
        messages.insert("tool.tooltip.select".to_owned(), "Select".to_owned());
        messages.insert("tool.tooltip.draw_path".to_owned(), "Draw Path".to_owned());
        messages.insert(
            "tool.tooltip.draw_curve".to_owned(),
            "Draw Curve".to_owned(),
        );
        messages.insert(
            "tool.tooltip.draw_rectangle".to_owned(),
            "Draw Rectangle".to_owned(),
        );
        messages.insert(
            "tool.tooltip.draw_circle".to_owned(),
            "Draw Circle".to_owned(),
        );
    }

    fn insert_status_messages(messages: &mut HashMap<String, String>) {
        // Status bar strings
        messages.insert("status.zoom_out".to_owned(), "Zoom Out".to_owned());
        messages.insert("status.zoom_in".to_owned(), "Zoom In".to_owned());
        messages.insert("status.zoom_area".to_owned(), "Zoom to Area".to_owned());
        messages.insert("status.snap_grid".to_owned(), "Snap to Grid".to_owned());

        // Status template strings
        messages.insert("status.saved".to_owned(), "Saved: {path}".to_owned());
        messages.insert("status.opened".to_owned(), "Opened: {path}".to_owned());
        messages.insert(
            "status.history_error".to_owned(),
            "History error: {error}".to_owned(),
        );
        messages.insert(
            "status.save_failed".to_owned(),
            "Save failed: {error}".to_owned(),
        );
        messages.insert(
            "status.open_failed".to_owned(),
            "Open failed: {error}".to_owned(),
        );
        messages.insert("status.maximized".to_owned(), " [MAX]".to_owned());
        messages.insert("status.plain_text".to_owned(), "Plain Text".to_owned());
    }

    fn insert_align_messages(messages: &mut HashMap<String, String>) {
        // Alignment button strings
        messages.insert("align.left".to_owned(), "Align Left".to_owned());
        messages.insert("align.centre".to_owned(), "Align Centre".to_owned());
        messages.insert("align.right".to_owned(), "Align Right".to_owned());
        messages.insert("align.top".to_owned(), "Align Top".to_owned());
        messages.insert("align.middle".to_owned(), "Align Middle".to_owned());
        messages.insert("align.bottom".to_owned(), "Align Bottom".to_owned());
    }

    fn insert_style_messages(messages: &mut HashMap<String, String>) {
        // Style control strings
        messages.insert("style.stroke".to_owned(), "Stroke".to_owned());
        messages.insert("style.fill".to_owned(), "Fill".to_owned());
        messages.insert(
            "style.stroke_loading".to_owned(),
            "Stroke: (loading)".to_owned(),
        );
        messages.insert(
            "style.fill_loading".to_owned(),
            "Fill: (loading)".to_owned(),
        );
    }

    fn insert_doc_messages(messages: &mut HashMap<String, String>) {
        // Document header strings
        messages.insert("doc.untitled".to_owned(), "untitled".to_owned());
    }

    fn insert_a11y_messages(messages: &mut HashMap<String, String>) {
        // Accessibility strings
        messages.insert("a11y.canvas".to_owned(), "Drawing canvas".to_owned());
        messages.insert("a11y.shape_list".to_owned(), "Shapes".to_owned());
        messages.insert("a11y.shape_item".to_owned(), "Shape {index}".to_owned());
        messages.insert("a11y.window_title".to_owned(), "Gauss".to_owned());
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
    /// # Panics
    ///
    /// Panics if the `default_locale` is not present in the `catalogs` `HashMap`.
    /// The default locale must have a corresponding catalog for fallback to work.
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
mod tests;
