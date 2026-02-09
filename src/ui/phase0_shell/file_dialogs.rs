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

use crate::model::Selection;
use crate::svg::export::export_svg_with_resources_checked;

use super::Phase0Shell;

type SavePathPromptReceiver = oneshot::Receiver<gpui::Result<Option<PathBuf>>>;
type OpenPathsPromptReceiver = oneshot::Receiver<gpui::Result<Option<Vec<PathBuf>>>>;
#[cfg(any(test, feature = "test-support", coverage, coverage_nightly))]
type OpenPathPromptReceiver = SavePathPromptReceiver;

enum OpenPromptReceiver {
    Paths(OpenPathsPromptReceiver),
    #[cfg(any(test, feature = "test-support", coverage, coverage_nightly))]
    NewPath(OpenPathPromptReceiver),
}

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

pub(super) fn request_save(window: &mut Window, cx: &mut Context<Phase0Shell>) {
    // Use a trivial initial directory for Phase 0. Once we have document state,
    // this can be "current document directory" or similar.
    let rx = cx.prompt_for_new_path(Path::new("."), Some("gauss.svg"));

    cx.spawn_in(
        window,
        move |this: WeakEntity<Phase0Shell>, async_window_cx: &mut AsyncWindowContext| {
            let async_cx = async_window_cx.clone();
            apply_save_path(this, async_cx, rx)
        },
    )
    .detach();
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
            receive_open_paths(paths_rx).await?.into_iter().next()
        }
        #[cfg(any(test, feature = "test-support", coverage, coverage_nightly))]
        OpenPromptReceiver::NewPath(path_rx) => receive_save_path(path_rx).await,
    }
}

async fn apply_save_path(
    this: WeakEntity<Phase0Shell>,
    mut cx: AsyncWindowContext,
    rx: SavePathPromptReceiver,
) {
    let Some(path) = receive_save_path(rx).await else {
        return;
    };

    let Ok((doc, resources)) = this.update(&mut cx, |view, _view_cx| {
        (view.state.document.clone(), view.state.resources.clone())
    }) else {
        return;
    };

    // TODO: derive canvas size from document bounds or viewport state.
    let save_result = export_svg_with_resources_checked(&doc, &resources, 100.0, 100.0)
        .map_err(|err| err.to_string())
        .and_then(|svg| super::super::phase0_support::write_svg_to_path(&path, &svg));
    let error = save_result.err();

    let _update_result = this.update(&mut cx, move |view, view_cx| {
        if error.is_none() {
            view.last_saved_path = Some(path);
        }
        view.last_save_error = error;
        view_cx.notify();
    });
}

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
            view.document_history = History::new();
            view.selection_history = History::new();
            view.state.selection = Selection::empty();
            view.drag_state = None;
            view.state.active_path = None;
            view.last_opened_path = Some(first_path);
            view.last_open_error = None;
        } else {
            view.last_open_error = error;
        }
        view_cx.notify();
    });
}
