//! Shared palette helpers for the Phase 1 chrome layout.

use gpui::{Hsla, opaque_grey};

pub(super) fn chrome_background() -> Hsla {
    opaque_grey(0.94, 1.0)
}

pub(super) fn chrome_panel() -> Hsla {
    opaque_grey(0.97, 1.0)
}

pub(super) fn chrome_border() -> Hsla {
    opaque_grey(0.78, 1.0)
}

pub(super) fn chrome_text() -> Hsla {
    opaque_grey(0.2, 1.0)
}

pub(super) fn chrome_muted_text() -> Hsla {
    opaque_grey(0.45, 1.0)
}

pub(super) fn chrome_active() -> Hsla {
    opaque_grey(0.86, 1.0)
}
