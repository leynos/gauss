//! Stroke/fill style controls for the Phase 0 shell.
//!
//! Phase 0 uses `gpui-component`’s `ColorPicker` to provide a ready-made UI for
//! editing colours. When a shape is selected, changes apply to the selected
//! shapes via `Command::SetStyle` so they participate in document undo/redo.

use gpui::{AppContext as _, Context, Hsla, ParentElement as _, Styled as _, Window, div};
use gpui_component::color_picker::{ColorPicker, ColorPickerEvent, ColorPickerState};

use crate::model::{Command, PaintStyle, Rgba, SelItem, ShapeId, StyleChange};

use super::Phase0Shell;

impl Phase0Shell {
    pub(super) fn ensure_style_pickers(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.stroke_picker.is_some() && self.fill_picker.is_some() {
            return;
        }

        let stroke_default = self.state.current_style.stroke.map(model_rgba_to_hsla);
        let fill_default = self.state.current_style.fill.map(model_rgba_to_hsla);

        let stroke_state = cx.new(|picker_cx| {
            let mut state = ColorPickerState::new(window, picker_cx);
            if let Some(default_value) = stroke_default {
                state = state.default_value(default_value);
            }
            state
        });

        let fill_state = cx.new(|picker_cx| {
            let mut state = ColorPickerState::new(window, picker_cx);
            if let Some(default_value) = fill_default {
                state = state.default_value(default_value);
            }
            state
        });

        let stroke_sub = cx.subscribe_in(
            &stroke_state,
            window,
            |shell, _emitter, event: &ColorPickerEvent, _window, view_cx| {
                if shell.handle_stroke_picker_event(event) {
                    view_cx.notify();
                }
            },
        );
        let fill_sub = cx.subscribe_in(
            &fill_state,
            window,
            |shell, _emitter, event: &ColorPickerEvent, _window, view_cx| {
                if shell.handle_fill_picker_event(event) {
                    view_cx.notify();
                }
            },
        );

        self.stroke_picker = Some(stroke_state);
        self.fill_picker = Some(fill_state);
        self.style_picker_subscriptions.push(stroke_sub);
        self.style_picker_subscriptions.push(fill_sub);
    }

    pub(super) fn style_picker_row(&self) -> impl gpui::IntoElement {
        let stroke_picker = self
            .stroke_picker
            .as_ref()
            .map(|state| ColorPicker::new(state).label("Stroke"));
        let fill_picker = self
            .fill_picker
            .as_ref()
            .map(|state| ColorPicker::new(state).label("Fill"));

        let mut row = div().flex().items_center().gap_2();
        if let Some(picker) = stroke_picker {
            row = row.child(picker);
        } else {
            row = row.child("Stroke: (loading)");
        }

        if let Some(picker) = fill_picker {
            row = row.child(picker);
        } else {
            row = row.child("Fill: (loading)");
        }

        row
    }

    fn apply_stroke_colour_inner(&mut self, colour: Option<Hsla>) -> bool {
        let stroke = colour.map(hsla_to_model_rgba);
        self.apply_style_update(StyleUpdate::Stroke(stroke))
    }

    fn apply_fill_colour_inner(&mut self, colour: Option<Hsla>) -> bool {
        let fill = colour.map(hsla_to_model_rgba);
        self.apply_style_update(StyleUpdate::Fill(fill))
    }

    /// Apply a new stroke colour to the current selection.
    ///
    /// This is a small public API so headless `#[gpui::test]` integration tests
    /// can drive style changes without relying on brittle palette UI
    /// hit-testing.
    #[cfg(any(test, feature = "test-support", coverage, coverage_nightly))]
    pub fn apply_stroke_colour(&mut self, colour: Option<Hsla>) -> bool {
        self.apply_stroke_colour_inner(colour)
    }

    /// Apply a new fill colour to the current selection.
    ///
    /// See [`Self::apply_stroke_colour`] for why this method is public.
    #[cfg(any(test, feature = "test-support", coverage, coverage_nightly))]
    pub fn apply_fill_colour(&mut self, colour: Option<Hsla>) -> bool {
        self.apply_fill_colour_inner(colour)
    }

    fn handle_stroke_picker_event(&mut self, event: &ColorPickerEvent) -> bool {
        match event {
            ColorPickerEvent::Change(value) => self.apply_stroke_colour_inner(*value),
        }
    }

    fn handle_fill_picker_event(&mut self, event: &ColorPickerEvent) -> bool {
        match event {
            ColorPickerEvent::Change(value) => self.apply_fill_colour_inner(*value),
        }
    }

    fn apply_style_update(&mut self, update: StyleUpdate) -> bool {
        let selected_shapes = self.selected_shape_ids_for_style();
        if selected_shapes.is_empty() {
            update.apply_to_style(&mut self.state.current_style);
            return true;
        }

        let mut changes = Vec::new();
        for shape_id in selected_shapes {
            let Some(shape) = self.state.document.shape(shape_id) else {
                continue;
            };

            let from = shape.style.clone();
            let mut to = from.clone();
            update.apply_to_style(&mut to);

            if from == to {
                continue;
            }

            changes.push(StyleChange { shape_id, from, to });
        }

        if changes.is_empty() {
            return false;
        }

        self.apply_command(Command::SetStyle { changes }).is_ok()
    }

    fn selected_shape_ids_for_style(&self) -> Vec<ShapeId> {
        use std::collections::HashSet;

        let mut seen = HashSet::new();
        let mut ids = Vec::new();
        for item in &self.state.selection.items {
            let shape_id = shape_id_for_selection_item(item);
            if seen.insert(shape_id) {
                ids.push(shape_id);
            }
        }
        ids
    }
}

#[derive(Clone, Copy, Debug)]
enum StyleUpdate {
    Stroke(Option<Rgba>),
    Fill(Option<Rgba>),
}

impl StyleUpdate {
    const fn apply_to_style(self, style: &mut PaintStyle) {
        match self {
            Self::Stroke(stroke) => {
                style.stroke = stroke;
            }
            Self::Fill(fill) => {
                style.fill = fill;
            }
        }
    }
}

const fn shape_id_for_selection_item(item: &SelItem) -> ShapeId {
    match item {
        SelItem::Shape(shape)
        | SelItem::Anchor { shape, .. }
        | SelItem::HandleIn { shape, .. }
        | SelItem::HandleOut { shape, .. }
        | SelItem::Segment { shape, .. } => *shape,
    }
}

fn model_rgba_to_hsla(color: Rgba) -> Hsla {
    let packed = (u32::from(color.r) << 24)
        | (u32::from(color.g) << 16)
        | (u32::from(color.b) << 8)
        | u32::from(color.a);
    let rgba = gpui::rgba(packed);
    Hsla::from(rgba)
}

fn hsla_to_model_rgba(color: Hsla) -> Rgba {
    let rgba = gpui::Rgba::from(color);
    let packed: u32 = rgba.into();
    let r = ((packed >> 24) & 0xff) as u8;
    let g = ((packed >> 16) & 0xff) as u8;
    let b = ((packed >> 8) & 0xff) as u8;
    let a = (packed & 0xff) as u8;
    Rgba::new(r, g, b, a)
}
