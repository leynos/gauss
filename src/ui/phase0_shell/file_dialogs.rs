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

use super::Phase0Shell;
use crate::{
    model::{SelectToolState, Selection},
    svg::export::{
        CanvasSize,
        ExportOptions,
        export_svg_with_metadata_checked,
        export_svg_with_resources_web_ready_checked,
    },
};

/// Channel that yields the path chosen by a save prompt.
type SavePathPromptReceiver = oneshot::Receiver<gpui::Result<Option<PathBuf>>>;
/// Channel that yields the paths chosen by an open prompt.
type OpenPathsPromptReceiver = oneshot::Receiver<gpui::Result<Option<Vec<PathBuf>>>>;
/// Channel that yields a single path from the test-only new-path prompt.
#[cfg(any(test, feature = "test-support", coverage, coverage_nightly))]
type OpenPathPromptReceiver = SavePathPromptReceiver;

/// Pending open prompt, whose shape depends on the active prompt mode.
enum OpenPromptReceiver {
    /// Pending multi-path picker used by the native prompt.
    Paths(OpenPathsPromptReceiver),
    /// Pending new-path prompt used by the test-only mode.
    #[cfg(any(test, feature = "test-support", coverage, coverage_nightly))]
    NewPath(OpenPathPromptReceiver),
}

/// Which SVG export variant a save request should produce.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SaveIntent {
    /// Export preserving Gauss metadata for later reload.
    PreserveMetadata,
    /// Export a web-ready SVG with resources inlined.
    WebReady,
}

impl SaveIntent {
    /// Default file name offered for this export variant.
    const fn suggested_name(self) -> &'static str {
        match self {
            Self::PreserveMetadata => "gauss.svg",
            Self::WebReady => "gauss.web.svg",
        }
    }
}

/// Strategy used to prompt for a document to open.
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
    /// Issue the prompt for this mode and return its pending receiver.
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

/// Show an open prompt and load the chosen document once the user responds.
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

/// Show a save prompt for the metadata-preserving SVG export.
pub(super) fn request_save(window: &mut Window, cx: &mut Context<Phase0Shell>) {
    request_save_for_intent(window, cx, SaveIntent::PreserveMetadata);
}

/// Show a save prompt for the web-ready SVG export.
pub(super) fn request_web_ready_export(window: &mut Window, cx: &mut Context<Phase0Shell>) {
    request_save_for_intent(window, cx, SaveIntent::WebReady);
}

/// Prompt for a destination path and start the export for `intent`.
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

/// Await a save prompt, returning `None` on cancellation or prompt failure.
async fn receive_save_path(rx: SavePathPromptReceiver) -> Option<PathBuf> {
    let prompt_result = rx.await.ok()?;
    prompt_result.ok()?
}

/// Await an open prompt, returning `None` on cancellation or prompt failure.
async fn receive_open_paths(rx: OpenPathsPromptReceiver) -> Option<Vec<PathBuf>> {
    let prompt_result = rx.await.ok()?;
    prompt_result.ok()?
}

/// Await an open prompt and return the first chosen path, if any.
async fn receive_open_path(receiver: OpenPromptReceiver) -> Option<PathBuf> {
    match receiver {
        OpenPromptReceiver::Paths(paths_rx) => {
            receive_open_paths(paths_rx).await?.into_iter().next()
        }
        #[cfg(any(test, feature = "test-support", coverage, coverage_nightly))]
        OpenPromptReceiver::NewPath(path_rx) => receive_save_path(path_rx).await,
    }
}

/// Export the document to the chosen path and record the outcome on the shell.
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
        // FIXME: derive canvas size from document bounds or viewport state -
        // TRACKER-WEB-CANVAS-SIZE.
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

/// Load the chosen document and reset history, selection, and transient state.
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
