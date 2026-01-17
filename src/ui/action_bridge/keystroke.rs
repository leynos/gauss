//! GPUI-specific keystroke formatting.
//!
//! This module keeps GPUI serialization out of the model layer by providing
//! a formatting helper for model keystrokes.

use crate::model::{Keystroke, Modifier};

/// Format a [`Keystroke`] for GPUI keybindings.
///
/// # Examples
///
/// ```rust,no_run
/// use gauss::model::Keystroke;
/// use gauss::ui::action_bridge::keystroke_to_gpui_string;
///
/// let undo = Keystroke::secondary("z");
/// assert_eq!(keystroke_to_gpui_string(&undo), "secondary-z");
/// ```
pub fn keystroke_to_gpui_string(keystroke: &Keystroke) -> String {
    let mut parts: Vec<String> = keystroke
        .modifiers
        .active_in_order()
        .map(gpui_modifier_token)
        .map(str::to_owned)
        .collect();

    // Normalise key to lowercase to match GPUI's expected format.
    parts.push(keystroke.key.to_lowercase());
    parts.join("-")
}

const fn gpui_modifier_token(modifier: Modifier) -> &'static str {
    match modifier {
        Modifier::Control => "ctrl",
        Modifier::Alt => "alt",
        Modifier::Shift => "shift",
        Modifier::Secondary => "secondary",
    }
}
