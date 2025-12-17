//! Phase 0 UI shell.
//!
//! This module intentionally contains "just enough UI" to validate that GPUI is
//! wired up correctly and to allow incremental integration tests using GPUI's
//! `TestAppContext`.

use std::path::{Path, PathBuf};

use camino::{Utf8Path, Utf8PathBuf};
use cap_std::{ambient_authority, fs_utf8::Dir};
use futures::channel::oneshot;
use gpui::{AsyncWindowContext, WeakEntity, Window, div, prelude::*};
use uuid::Uuid;

use crate::{
    model::{Anchor, Document, PaintStyle, PathGeom, Rgba, SegmentKind, Shape, ShapeId, Vec2},
    svg::export::export_svg,
};

/// Trigger a “Save…” workflow for the current document.
///
/// Phase 0 uses this action purely to validate that:
///
/// - action dispatch is wired end-to-end, and
/// - platform file prompts can be exercised in headless GPUI tests.
#[derive(Clone, Debug, Default, PartialEq, gpui::Action)]
#[action(no_json)]
pub struct SaveSvg;

/// Minimal root view for Phase 0.
///
/// This view exists to keep a stable "entrypoint view" for the `PoC` while the
/// real UI is built out.
pub struct Phase0Shell {
    focus_handle: gpui::FocusHandle,
    did_focus: bool,
    document: Document,
    last_saved_path: Option<PathBuf>,
    last_save_error: Option<String>,
}

type SavePathPromptReceiver = oneshot::Receiver<gpui::Result<Option<PathBuf>>>;

impl Phase0Shell {
    /// Construct a new shell.
    #[must_use]
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            did_focus: false,
            document: demo_document(),
            last_saved_path: None,
            last_save_error: None,
        }
    }

    /// Return the last path selected by the platform save prompt, if any.
    #[must_use]
    pub fn last_saved_path(&self) -> Option<&Path> {
        self.last_saved_path.as_deref()
    }

    async fn receive_save_path(rx: SavePathPromptReceiver) -> Option<PathBuf> {
        let prompt_result = rx.await.ok()?;
        prompt_result.ok()?
    }

    async fn apply_save_path(
        this: WeakEntity<Self>,
        mut cx: AsyncWindowContext,
        rx: SavePathPromptReceiver,
    ) {
        let Some(path) = Self::receive_save_path(rx).await else {
            return;
        };

        let Ok(doc) = this.update(&mut cx, |view, _view_cx| view.document.clone()) else {
            return;
        };

        let svg = export_svg(&doc, 100.0, 100.0);
        let save_result = write_svg_to_path(&path, &svg);
        let error = save_result.err().map(|err| err.to_string());

        drop(this.update(&mut cx, move |view, view_cx| {
            if error.is_none() {
                view.last_saved_path = Some(path);
            }
            view.last_save_error = error;
            view_cx.notify();
        }));
    }

    fn request_save(window: &mut Window, cx: &mut Context<Self>) {
        // Use a trivial initial directory for Phase 0. Once we have document
        // state, this can be "current document directory" or similar.
        let rx = cx.prompt_for_new_path(Path::new("."), Some("gauss.svg"));

        cx.spawn_in(
            window,
            move |this: WeakEntity<Self>, async_window_cx: &mut AsyncWindowContext| {
                let async_cx = async_window_cx.clone();
                Self::apply_save_path(this, async_cx, rx)
            },
        )
        .detach();
    }
}

impl Render for Phase0Shell {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        if !self.did_focus {
            self.did_focus = true;
            window.focus(&self.focus_handle);
        }

        div()
            .p_4()
            .track_focus(&self.focus_handle)
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child("Gauss PoC: Phase 0 shell")
                    .child(
                        div()
                            .id("save-button")
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .border_1()
                            .on_click(cx.listener(
                                |_shell: &mut Self,
                                 _event: &gpui::ClickEvent,
                                 click_window,
                                 click_cx| {
                                    Self::request_save(click_window, click_cx);
                                },
                            ))
                            .child("Save…"),
                    ),
            )
            .on_action(
                cx.listener(|_shell: &mut Self, _: &SaveSvg, action_window, action_cx| {
                    Self::request_save(action_window, action_cx);
                }),
            )
            .child(
                "This view validates action wiring and platform save dialogs, and provides a \
                 minimal manual Save… affordance.",
            )
            .child(match (&self.last_saved_path, &self.last_save_error) {
                (_, Some(err)) => format!("Save failed: {err}"),
                (Some(path), None) => format!("Last saved path: {}", path.display()),
                (None, None) => "Last saved path: (none)".to_owned(),
            })
    }
}

#[derive(Debug)]
enum SaveSvgError {
    NonUtf8Path,
    MissingFileName,
    Io(std::io::Error),
}

impl std::fmt::Display for SaveSvgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonUtf8Path => write!(f, "path is not valid UTF-8"),
            Self::MissingFileName => write!(f, "path does not include a file name"),
            Self::Io(err) => write!(f, "io error: {err}"),
        }
    }
}

impl std::error::Error for SaveSvgError {}

fn write_svg_to_path(path: &Path, svg: &str) -> Result<(), SaveSvgError> {
    let utf8_path =
        Utf8PathBuf::from_path_buf(path.to_path_buf()).map_err(|_| SaveSvgError::NonUtf8Path)?;
    let directory = utf8_path.parent().unwrap_or_else(|| Utf8Path::new("."));
    let file_name = utf8_path.file_name().ok_or(SaveSvgError::MissingFileName)?;

    Dir::create_ambient_dir_all(directory, ambient_authority()).map_err(SaveSvgError::Io)?;
    let dir = Dir::open_ambient_dir(directory, ambient_authority()).map_err(SaveSvgError::Io)?;

    dir.write(Utf8Path::new(file_name), svg.as_bytes())
        .map_err(SaveSvgError::Io)
}

fn demo_document() -> Document {
    let shape = Shape {
        id: ShapeId::from(Uuid::from_u128(0x6d3c_0fb4_43a8_48f1_9f14_623a_70d5_2e1a)),
        z: 0,
        style: PaintStyle::new(
            Some(Rgba::new(0, 0, 0, 255)),
            2.0,
            Some(Rgba::new(0, 0, 255, 96)),
        ),
        path: PathGeom {
            anchors: vec![
                Anchor::new(Vec2::new(10.0, 10.0)),
                Anchor::new(Vec2::new(90.0, 10.0)),
                Anchor::new(Vec2::new(90.0, 90.0)),
                Anchor::new(Vec2::new(10.0, 90.0)),
            ],
            segments: vec![SegmentKind::Line, SegmentKind::Line, SegmentKind::Line],
            closed: true,
        },
    };

    Document {
        shapes: vec![shape],
    }
}
