//! Canvas layout and input event callbacks for the Phase 0 shell.

use gpui::{Window, div, prelude::*, white};

use super::{Phase0Shell, chrome_palette::chrome_border};

/// Selects whether a changed canvas event stops propagating after notification.
#[derive(Clone, Copy)]
enum CanvasEventPropagation {
    /// Keep propagation active after a changed event.
    Continue,
    /// Stop propagation after notifying for a changed event.
    Stop,
}

/// Finish a canvas event according to its handler's change indication.
///
/// For example, a changed mouse-move event notifies and continues bubbling;
/// a changed navigation event notifies before stopping propagation.
/// A `false` result adds neither notification nor propagation stop.
fn complete_canvas_event(
    did_change: bool,
    propagation: CanvasEventPropagation,
    cx: &mut Context<Phase0Shell>,
) {
    if !did_change {
        return;
    }

    cx.notify();
    match propagation {
        CanvasEventPropagation::Continue => {}
        CanvasEventPropagation::Stop => cx.stop_propagation(),
    }
}

impl Phase0Shell {
    /// Build the interactive canvas area for document rendering and input.
    ///
    /// The returned element owns the canvas event handlers for pointer,
    /// navigation, click, and scroll interactions.
    pub(super) fn canvas_area(&self, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        div()
            .id("phase0-canvas")
            .debug_selector(|| "#phase0-canvas".to_owned())
            .flex()
            .flex_1()
            .border_1()
            .border_color(chrome_border())
            .bg(white())
            .rounded_md()
            .overflow_hidden()
            .occlude()
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(Self::canvas_mouse_down),
            )
            .on_mouse_down(
                gpui::MouseButton::Navigate(gpui::NavigationDirection::Back),
                cx.listener(Self::canvas_navigate_mouse_down),
            )
            .on_mouse_down(
                gpui::MouseButton::Navigate(gpui::NavigationDirection::Forward),
                cx.listener(Self::canvas_navigate_mouse_down),
            )
            .on_mouse_move(cx.listener(Self::canvas_mouse_move))
            .on_mouse_up(gpui::MouseButton::Left, cx.listener(Self::canvas_mouse_up))
            .on_mouse_up_out(gpui::MouseButton::Left, cx.listener(Self::canvas_mouse_up))
            .on_click(cx.listener(Self::canvas_click))
            .on_scroll_wheel(cx.listener(Self::canvas_scroll_wheel))
            .child(super::super::canvas_paint::canvas_for_document(
                &self.state.document,
                &self.state.selection,
                self.state.viewport,
            ))
    }

    /// Pass left-button presses to the manipulation handler, which accepts
    /// them only in Manipulate mode.
    ///
    /// For example, a hit-tested press can produce a select-tool transition;
    /// a reported change notifies and continues propagation.
    fn canvas_mouse_down(
        shell: &mut Self,
        event: &gpui::MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let did_change = shell.handle_canvas_mouse_down(event);
        complete_canvas_event(did_change, CanvasEventPropagation::Continue, cx);
    }

    /// Delegate Back and Forward navigation presses to history handling.
    ///
    /// For example, Shift selects selection history; a handled navigation
    /// notifies and stops propagation.
    fn canvas_navigate_mouse_down(
        shell: &mut Self,
        event: &gpui::MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let did_change = shell.handle_navigation_mouse_down(event);
        complete_canvas_event(did_change, CanvasEventPropagation::Stop, cx);
    }

    /// Dispatch pointer motion to hover and manipulation drag handling.
    ///
    /// For example, motion can refresh hover when left is not held or advance
    /// an active drag when it is; a reported change notifies and continues
    /// propagation.
    fn canvas_mouse_move(
        shell: &mut Self,
        event: &gpui::MouseMoveEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let did_change = shell.handle_canvas_mouse_move(event);
        complete_canvas_event(did_change, CanvasEventPropagation::Continue, cx);
    }

    /// Dispatch a left-button release to select-tool pointer-up handling.
    ///
    /// For example, the release can change select-tool pointer state; a
    /// reported change notifies and continues propagation.
    fn canvas_mouse_up(
        shell: &mut Self,
        event: &gpui::MouseUpEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let did_change = shell.handle_canvas_mouse_up(event);
        complete_canvas_event(did_change, CanvasEventPropagation::Continue, cx);
    }

    /// Record the screen click before delegating mode-specific draw handling.
    ///
    /// For example, a non-Draw click still records `last_canvas_click_screen`
    /// when the handler returns false; that alone adds no notify or stop. A
    /// reported Draw-mode change notifies and stops propagation.
    fn canvas_click(
        shell: &mut Self,
        event: &gpui::ClickEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let did_change = shell.handle_canvas_click(event.position());
        complete_canvas_event(did_change, CanvasEventPropagation::Stop, cx);
    }

    /// Convert wheel input using window line height and apply viewport changes.
    ///
    /// For example, Ctrl/Cmd-wheel zooms around the cursor; Shift-vertical
    /// input can pan horizontally. A changed viewport notifies and stops
    /// propagation.
    fn canvas_scroll_wheel(
        shell: &mut Self,
        event: &gpui::ScrollWheelEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let line_height = window.line_height();
        let did_change = super::super::viewport_input::apply_scroll_wheel_event(
            &mut shell.state.viewport,
            event,
            line_height,
        );

        complete_canvas_event(did_change, CanvasEventPropagation::Stop, cx);
    }
}
