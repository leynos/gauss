//! Phase 0 native file dialog wiring.
//!
//! Phase 0 uses GPUI’s platform prompts for selecting paths, because they are
//! "ready-made" and can be exercised in headless `#[gpui::test]` tests.
//!
//! Note: GPUI 0.2.2’s test platform does not implement `prompt_for_paths`. For
//! the “Open…” test we route selection through `prompt_for_new_path` instead,
//! which allows us to test the rest of the pipeline (action dispatch, async
//! wiring, file read, parse, and state update).

use std::path::{Path, PathBuf};

use futures::channel::oneshot;
use gpui::{AsyncWindowContext, Context, PathPromptOptions, WeakEntity, Window};
use gpui_component::history::History;

use crate::model::{SelectToolState, Selection};
use crate::svg::export::{
    CanvasSize, ExportOptions, export_svg_with_metadata_checked,
    export_svg_with_resources_web_ready_checked,
};

use super::Phase0Shell;

/// Receives the result of a prompt for one save or export path.
type SavePathPromptReceiver = oneshot::Receiver<gpui::Result<Option<PathBuf>>>;
/// Receives the result of a prompt that may return several existing paths.
type OpenPathsPromptReceiver = oneshot::Receiver<gpui::Result<Option<Vec<PathBuf>>>>;
/// Reuses the save-path result shape for the test platform's new-path prompt.
#[cfg(any(test, feature = "test-support", coverage, coverage_nightly))]
type OpenPathPromptReceiver = SavePathPromptReceiver;

/// Holds the result from either the native picker or the test-only path prompt.
enum OpenPromptReceiver {
    /// Result from the native picker for existing paths.
    Paths(OpenPathsPromptReceiver),
    /// Result from the test platform's prompt for one path.
    #[cfg(any(test, feature = "test-support", coverage, coverage_nightly))]
    NewPath(OpenPathPromptReceiver),
}

/// Selects which SVG export behaviour a save prompt will invoke.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SaveIntent {
    /// Save SVG while retaining the document's Gauss metadata.
    PreserveMetadata,
    /// Export SVG without Gauss metadata for web use.
    WebReady,
}

impl SaveIntent {
    /// Return the filename suggested by the matching export action.
    ///
    /// For example, `WebReady` suggests `gauss.web.svg`.
    const fn suggested_name(self) -> &'static str {
        match self {
            Self::PreserveMetadata => "gauss.svg",
            Self::WebReady => "gauss.web.svg",
        }
    }
}

/// Selects the native path picker or the test platform's single-path prompt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum OpenPromptMode {
    /// Use the platform file picker for selecting existing paths.
    ///
    /// This is the desired behaviour for real runs of the application.
    Native,
    /// In tests, prompt for a single path using the “new path” prompt.
    ///
    /// GPUI 0.2.2's test platform does not implement `prompt_for_paths`, so
    /// this mode lets us cover the rest of the open pipeline (action dispatch,
    /// async wiring, file read, parse, and state update).
    #[cfg(any(test, feature = "test-support", coverage, coverage_nightly))]
    TestNewPath,
}

impl OpenPromptMode {
    /// Start the platform prompt selected for this execution mode.
    ///
    /// For example, `Native` opens the picker for an existing SVG path.
    fn prompt(self, cx: &mut Context<Phase0Shell>) -> OpenPromptReceiver {
        match self {
            Self::Native => OpenPromptReceiver::Paths(cx.prompt_for_paths(PathPromptOptions {
                files: true,
                directories: false,
                multiple: false,
                prompt: Some("Open SVG".into()),
            })),
            #[cfg(any(test, feature = "test-support", coverage, coverage_nightly))]
            Self::TestNewPath => {
                OpenPromptReceiver::NewPath(cx.prompt_for_new_path(Path::new("."), None))
            }
        }
    }
}

/// Prompt for an SVG path, then load the first selected file asynchronously.
///
/// For example, selecting one SVG in the native picker loads that path.
pub(super) fn request_open(
    mode: OpenPromptMode,
    window: &mut Window,
    cx: &mut Context<Phase0Shell>,
) {
    let rx = mode.prompt(cx);

    cx.spawn_in(
        window,
        move |this: WeakEntity<Phase0Shell>, async_window_cx: &mut AsyncWindowContext| {
            let async_cx = async_window_cx.clone();
            apply_open_prompt(this, async_cx, rx)
        },
    )
    .detach();
}

/// Prompt for a destination and save the document with its Gauss metadata.
///
/// For example, the suggested `gauss.svg` destination retains Gauss metadata.
pub(super) fn request_save(window: &mut Window, cx: &mut Context<Phase0Shell>) {
    request_save_for_intent(window, cx, SaveIntent::PreserveMetadata);
}

/// Prompt for a destination and export SVG without Gauss metadata.
///
/// For example, the suggested `gauss.web.svg` destination uses web-ready export.
pub(super) fn request_web_ready_export(window: &mut Window, cx: &mut Context<Phase0Shell>) {
    request_save_for_intent(window, cx, SaveIntent::WebReady);
}

/// Prompt for the intent's suggested filename and start its asynchronous export.
///
/// For example, `WebReady` prompts with `gauss.web.svg` before exporting.
fn request_save_for_intent(window: &mut Window, cx: &mut Context<Phase0Shell>, intent: SaveIntent) {
    // Use a trivial initial directory for Phase 0. Once we have document state,
    // this can be "current document directory" or similar.
    let rx = cx.prompt_for_new_path(Path::new("."), Some(intent.suggested_name()));

    cx.spawn_in(
        window,
        move |this: WeakEntity<Phase0Shell>, async_window_cx: &mut AsyncWindowContext| {
            let async_cx = async_window_cx.clone();
            apply_save_path(this, async_cx, rx, intent)
        },
    )
    .detach();
}

/// Resolve a save prompt, treating cancellation and prompt errors as no path.
///
/// For example, closing the save dialog produces `None`.
async fn receive_save_path(rx: SavePathPromptReceiver) -> Option<PathBuf> {
    let prompt_result = rx.await.ok()?;
    prompt_result.ok()?
}

/// Resolve an existing-path prompt, treating cancellation and errors as no paths.
///
/// For example, a successful picker result preserves its selected paths.
async fn receive_open_paths(rx: OpenPathsPromptReceiver) -> Option<Vec<PathBuf>> {
    let prompt_result = rx.await.ok()?;
    prompt_result.ok()?
}

/// Return the first selected path, or the single path from the test prompt.
///
/// For example, selections `[a.svg, b.svg]` resolve to `a.svg`.
async fn receive_open_path(receiver: OpenPromptReceiver) -> Option<PathBuf> {
    match receiver {
        OpenPromptReceiver::Paths(paths_rx) => {
            receive_open_paths(paths_rx).await?.into_iter().next()
        }
        #[cfg(any(test, feature = "test-support", coverage, coverage_nightly))]
        OpenPromptReceiver::NewPath(path_rx) => receive_save_path(path_rx).await,
    }
}

/// Export the current document to the chosen path and update its save status.
///
/// For example, a successful write records the destination as `last_saved_path`.
async fn apply_save_path(
    this: WeakEntity<Phase0Shell>,
    mut cx: AsyncWindowContext,
    rx: SavePathPromptReceiver,
    intent: SaveIntent,
) {
    let Some(path) = receive_save_path(rx).await else {
        return;
    };

    let Ok(save_result) = this.update(&mut cx, |view, _view_cx| {
        // FIXME: derive canvas size from document bounds or viewport state - TRACKER-WEB-CANVAS-SIZE.
        let canvas_size = CanvasSize::new(100.0, 100.0);
        match intent {
            SaveIntent::PreserveMetadata => {
                let mut options =
                    ExportOptions::new(&view.state.document, &view.state.resources, canvas_size);
                if let Some(metadata_block) = view.state.gauss_metadata_block.as_deref() {
                    options = options.with_metadata_block(metadata_block);
                }
                export_svg_with_metadata_checked(options)
            }
            SaveIntent::WebReady => export_svg_with_resources_web_ready_checked(
                &view.state.document,
                &view.state.resources,
                canvas_size,
            ),
        }
        .map_err(|err| err.to_string())
        .and_then(|svg| super::super::phase0_support::write_svg_to_path(&path, &svg))
    }) else {
        return;
    };
    let error = save_result.err();

    let _update_result = this.update(&mut cx, move |view, view_cx| {
        if error.is_none() {
            view.last_saved_path = Some(path);
        }
        view.last_save_error = error;
        view_cx.notify();
    });
}

/// Load the chosen document and replace the shell's document state on success.
///
/// For example, a parse error leaves the current document in place and records
/// the error as `last_open_error`.
async fn apply_open_prompt(
    this: WeakEntity<Phase0Shell>,
    mut cx: AsyncWindowContext,
    rx: OpenPromptReceiver,
) {
    let Some(first_path) = receive_open_path(rx).await else {
        return;
    };

    let load_result = super::super::phase0_support::load_document_from_path(&first_path);
    let (loaded_state, error) = match load_result {
        Ok(imported) => (Some(imported), None),
        Err(err) => (None, Some(err)),
    };

    let _update_result = this.update(&mut cx, move |view, view_cx| {
        if let Some(imported) = loaded_state {
            view.state.document = imported.document;
            view.state.resources = imported.resources;
            view.state.gauss_metadata_block = imported.gauss_metadata_block;
            view.state.clear_document_history();
            view.selection_history = History::new();
            view.state.selection = Selection::empty();
            view.select_tool_state = SelectToolState::Idle;
            view.state.active_path = None;
            view.last_history_error = None;
            view.last_history_error_typed = None;
            view.last_opened_path = Some(first_path);
            view.last_open_error = None;
        } else {
            view.last_open_error = error;
        }
        view_cx.notify();
    });
}
