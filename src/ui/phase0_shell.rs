//! Phase 0 UI shell.
//!
//! This module intentionally contains "just enough UI" to validate that GPUI is
//! wired up correctly and to allow incremental integration tests using GPUI's
//! `TestAppContext`.

use std::path::{Path, PathBuf};

use futures::channel::oneshot;
use gpui::{AsyncWindowContext, PathPromptOptions, WeakEntity, Window, div, prelude::*};

use crate::model::{Document, Viewport};
use crate::svg::export::export_svg;

use super::phase0_support::{demo_document, load_document_from_path, write_svg_to_path};

/// Trigger an “Open…” workflow for loading a document from disk.
#[derive(Clone, Debug, Default, PartialEq, gpui::Action)]
#[action(no_json)]
pub struct OpenSvg;

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
    open_prompt_mode: OpenPromptMode,
    document: Document,
    viewport: Viewport,
    last_saved_path: Option<PathBuf>,
    last_save_error: Option<String>,
    last_opened_path: Option<PathBuf>,
    last_open_error: Option<String>,
}

type SavePathPromptReceiver = oneshot::Receiver<gpui::Result<Option<PathBuf>>>;
type OpenPathsPromptReceiver = oneshot::Receiver<gpui::Result<Option<Vec<PathBuf>>>>;
type OpenPathPromptReceiver = SavePathPromptReceiver;

enum OpenPromptReceiver {
    Paths(OpenPathsPromptReceiver),
    NewPath(OpenPathPromptReceiver),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OpenPromptMode {
    /// Use the platform file picker for selecting existing paths.
    ///
    /// This is the desired behaviour for real runs of the application.
    Native,
    /// In tests, prompt for a single path using the “new path” prompt.
    ///
    /// GPUI 0.2.2's test platform does not implement `prompt_for_paths`, so
    /// this mode lets us cover the rest of the open pipeline (action dispatch,
    /// async wiring, file read, parse, and state update).
    TestNewPath,
}

impl Phase0Shell {
    /// Construct a new shell.
    #[must_use]
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            did_focus: false,
            open_prompt_mode: OpenPromptMode::Native,
            document: demo_document(),
            viewport: Viewport::new(),
            last_saved_path: None,
            last_save_error: None,
            last_opened_path: None,
            last_open_error: None,
        }
    }

    /// Construct a new shell configured for headless `#[gpui::test]` tests.
    ///
    /// This differs from [`Self::new`] only in how it triggers the file dialog
    /// for “Open…”.
    #[must_use]
    pub fn new_for_tests(cx: &mut Context<Self>) -> Self {
        Self {
            open_prompt_mode: OpenPromptMode::TestNewPath,
            ..Self::new(cx)
        }
    }

    /// Return the last path selected by the platform save prompt, if any.
    #[must_use]
    pub fn last_saved_path(&self) -> Option<&Path> {
        self.last_saved_path.as_deref()
    }

    /// Return the last path selected by the platform open prompt, if any.
    #[must_use]
    pub fn last_opened_path(&self) -> Option<&Path> {
        self.last_opened_path.as_deref()
    }

    /// Return the current document.
    ///
    /// This is intended for tests and debugging while Phase 0 is still
    /// assembling the real UI.
    #[must_use]
    pub const fn document(&self) -> &Document {
        &self.document
    }

    /// Return the current viewport.
    ///
    /// This is intended for tests and debugging while Phase 0 is still
    /// assembling the real editor UI.
    #[must_use]
    pub const fn viewport(&self) -> Viewport {
        self.viewport
    }

    async fn receive_save_path(rx: SavePathPromptReceiver) -> Option<PathBuf> {
        let prompt_result = rx.await.ok()?;
        prompt_result.ok()?
    }

    async fn receive_open_paths(rx: OpenPathsPromptReceiver) -> Option<Vec<PathBuf>> {
        let prompt_result = rx.await.ok()?;
        prompt_result.ok()?
    }

    async fn receive_open_path(receiver: OpenPromptReceiver) -> Option<PathBuf> {
        match receiver {
            OpenPromptReceiver::Paths(paths_rx) => {
                Self::receive_open_paths(paths_rx).await?.into_iter().next()
            }
            OpenPromptReceiver::NewPath(path_rx) => Self::receive_save_path(path_rx).await,
        }
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
        let error = save_result.err();

        drop(this.update(&mut cx, move |view, view_cx| {
            if error.is_none() {
                view.last_saved_path = Some(path);
            }
            view.last_save_error = error;
            view_cx.notify();
        }));
    }

    async fn apply_open_prompt(
        this: WeakEntity<Self>,
        mut cx: AsyncWindowContext,
        rx: OpenPromptReceiver,
    ) {
        let Some(first_path) = Self::receive_open_path(rx).await else {
            return;
        };

        let load_result = load_document_from_path(&first_path);
        let (loaded_doc, error) = match load_result {
            Ok(doc) => (Some(doc), None),
            Err(err) => (None, Some(err)),
        };

        drop(this.update(&mut cx, move |view, view_cx| {
            if let Some(doc) = loaded_doc {
                view.document = doc;
                view.last_opened_path = Some(first_path);
                view.last_open_error = None;
            } else {
                view.last_open_error = error;
            }
            view_cx.notify();
        }));
    }

    fn request_open(mode: OpenPromptMode, window: &mut Window, cx: &mut Context<Self>) {
        let rx = match mode {
            OpenPromptMode::Native => {
                OpenPromptReceiver::Paths(cx.prompt_for_paths(PathPromptOptions {
                    files: true,
                    directories: false,
                    multiple: false,
                    prompt: Some("Open SVG".into()),
                }))
            }
            OpenPromptMode::TestNewPath => {
                OpenPromptReceiver::NewPath(cx.prompt_for_new_path(Path::new("."), None))
            }
        };

        cx.spawn_in(
            window,
            move |this: WeakEntity<Self>, async_window_cx: &mut AsyncWindowContext| {
                let async_cx = async_window_cx.clone();
                Self::apply_open_prompt(this, async_cx, rx)
            },
        )
        .detach();
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

    fn header_row(cx: &mut Context<Self>) -> impl gpui::IntoElement {
        div()
            .flex()
            .items_center()
            .justify_between()
            .child("Gauss PoC: Phase 0 shell")
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(Self::open_button(cx))
                    .child(Self::save_button(cx)),
            )
    }

    fn open_button(cx: &mut Context<Self>) -> impl gpui::IntoElement {
        div()
            .id("open-button")
            .px_3()
            .py_2()
            .rounded_md()
            .border_1()
            .on_click(cx.listener(
                |shell: &mut Self, _event: &gpui::ClickEvent, click_window, click_cx| {
                    Self::request_open(shell.open_prompt_mode, click_window, click_cx);
                },
            ))
            .child("Open…")
    }

    fn save_button(cx: &mut Context<Self>) -> impl gpui::IntoElement {
        div()
            .id("save-button")
            .px_3()
            .py_2()
            .rounded_md()
            .border_1()
            .on_click(cx.listener(
                |_shell: &mut Self, _event: &gpui::ClickEvent, click_window, click_cx| {
                    Self::request_save(click_window, click_cx);
                },
            ))
            .child("Save…")
    }

    fn save_status_line(&self) -> String {
        match (&self.last_saved_path, &self.last_save_error) {
            (_, Some(err)) => format!("Save failed: {err}"),
            (Some(path), None) => format!("Last saved path: {}", path.display()),
            (None, None) => "Last saved path: (none)".to_owned(),
        }
    }

    fn open_status_line(&self) -> String {
        match (&self.last_opened_path, &self.last_open_error) {
            (_, Some(err)) => format!("Open failed: {err}"),
            (Some(path), None) => format!("Last opened path: {}", path.display()),
            (None, None) => "Last opened path: (none)".to_owned(),
        }
    }

    fn canvas_area(&self, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        div()
            .id("phase0-canvas")
            .debug_selector(|| "#phase0-canvas".to_owned())
            .flex()
            .flex_1()
            .on_scroll_wheel(cx.listener(
                |shell: &mut Self, event: &gpui::ScrollWheelEvent, window, view_cx| {
                    let line_height = window.line_height();
                    let did_change = super::viewport_input::apply_scroll_wheel_event(
                        &mut shell.viewport,
                        event,
                        line_height,
                    );

                    if did_change {
                        view_cx.notify();
                        view_cx.stop_propagation();
                    }
                },
            ))
            .child(super::canvas_paint::canvas_for_document(
                &self.document,
                self.viewport,
            ))
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
            .child(Self::header_row(cx))
            .on_action(
                cx.listener(|shell: &mut Self, _: &OpenSvg, action_window, action_cx| {
                    Self::request_open(shell.open_prompt_mode, action_window, action_cx);
                }),
            )
            .on_action(
                cx.listener(|_shell: &mut Self, _: &SaveSvg, action_window, action_cx| {
                    Self::request_save(action_window, action_cx);
                }),
            )
            .child(
                "This view validates action wiring, native open/save prompts, and canvas \
                 painting, while Phase 0 assembles the real editor UI.",
            )
            .child(self.canvas_area(cx))
            .child(self.save_status_line())
            .child(self.open_status_line())
    }
}
