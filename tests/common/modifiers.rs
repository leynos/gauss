//! Modifier construction for GPUI integration tests.

use gpui::Modifiers;

pub const fn shift_secondary(modifiers: Modifiers) -> Modifiers {
    let mut next = modifiers;
    next.shift = true;
    next
}
