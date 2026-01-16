//! GPUI-specific keystroke formatting.
//!
//! This module keeps GPUI serialisation out of the model layer by providing a
//! formatter trait implemented for model keystrokes.

use crate::model::{Keystroke, Modifier};

/// Format a [`Keystroke`] for GPUI keybindings.
///
/// # Examples
///
/// ```rust,no_run
/// use gauss::model::Keystroke;
/// use gauss::ui::action_bridge::KeystrokeFormatter;
///
/// let undo = Keystroke::secondary("z");
/// assert_eq!(undo.to_gpui_string(), "secondary-z");
/// ```
pub trait KeystrokeFormatter {
    /// Convert the keystroke into GPUI's string format.
    fn to_gpui_string(&self) -> String;
}

impl KeystrokeFormatter for Keystroke {
    fn to_gpui_string(&self) -> String {
        let mut parts: Vec<String> = self
            .modifiers
            .active_in_order()
            .map(gpui_modifier_token)
            .map(str::to_owned)
            .collect();

        // Normalise key to lowercase to match GPUI's expected format.
        parts.push(self.key.to_lowercase());
        parts.join("-")
    }
}

const fn gpui_modifier_token(modifier: Modifier) -> &'static str {
    match modifier {
        Modifier::Control => "ctrl",
        Modifier::Alt => "alt",
        Modifier::Shift => "shift",
        Modifier::Secondary => "secondary",
    }
}
