//! Shared palette helpers for the Phase 1 chrome layout.

use gpui::{Hsla, opaque_grey};

pub(super) fn chrome_background() -> Hsla {
    // Base canvas chrome background to keep the shell calm and neutral.
    opaque_grey(0.94, 1.0)
}

pub(super) fn chrome_panel() -> Hsla {
    // Slightly lighter panel fill to lift toolbars above the background.
    opaque_grey(0.97, 1.0)
}

pub(super) fn chrome_border() -> Hsla {
    // Soft border tone for dividers and panel outlines.
    opaque_grey(0.78, 1.0)
}

pub(super) fn chrome_text() -> Hsla {
    // Primary text colour for high-contrast labels.
    opaque_grey(0.2, 1.0)
}

pub(super) fn chrome_muted_text() -> Hsla {
    // Muted text for secondary labels and disabled controls.
    opaque_grey(0.45, 1.0)
}

pub(super) fn chrome_active() -> Hsla {
    // Highlight for active controls without overpowering the chrome.
    opaque_grey(0.86, 1.0)
}
