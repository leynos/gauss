//! Message lookup for the Phase 0 shell.
//!
//! The shell exposes two lookups: a typed lookup for callers that can handle
//! errors, and a render-boundary helper for element builders that cannot.

use super::Phase0Shell;
use crate::i18n::{I18nError, MessageId};

impl Phase0Shell {
    /// Localize a message identifier to a string for the current locale.
    ///
    /// Lookup falls back to the default-locale catalogue when the requested
    /// locale lacks the message; it never falls back to the message key.
    ///
    /// # Errors
    ///
    /// Returns [`I18nError::MessageNotFound`] when neither the requested nor
    /// the default-locale catalogue contains `message_id`, and
    /// [`I18nError::UnsupportedLocale`] when no catalogue exists for either
    /// locale.
    pub(super) fn localize(&self, message_id: &MessageId) -> Result<String, I18nError> {
        self.localizer.lookup(&self.locale, message_id)
    }

    /// Localize a message identifier for render paths that cannot propagate
    /// errors, falling back to the raw message identifier.
    ///
    /// Logs a warning at warn level when the lookup fails. Callers that can
    /// handle the error themselves should call [`Self::localize`] directly.
    pub(super) fn localize_or_key(&self, message_id: &MessageId) -> String {
        self.localize(message_id).unwrap_or_else(|err| {
            log::warn!(
                "i18n lookup failed for {:?} in locale {:?}: {err}",
                message_id.as_str(),
                self.locale,
            );
            message_id.as_str().to_owned()
        })
    }
}
