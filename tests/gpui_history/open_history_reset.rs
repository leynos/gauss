//! GPUI tests for history state reset when opening a document.

use std::path::Path;


use camino::Utf8PathBuf;
use cap_std::{ambient_authority, fs_utf8::Dir};
use crate::common::{TempFileGuard, ensure_initial_draw, init_test_app, read_history_len, read_selection};
use gauss::model::{Command, SelItem, Selection, ShapeMovement, Vec2};
use gauss::ui::{OpenSvg, Phase0Shell};
use gpui::{Entity, TestAppContext, VisualTestContext};
use test_support::{TestSupportError, TestSupportResult};
use uuid::Uuid;

fn create_open_fixture() -> TestSupportResult<TempFileGuard> {
    let temp_dir = Utf8PathBuf::from_path_buf(std::env::temp_dir())
        .map_err(|_| TestSupportError::expectation("temp dir should be valid UTF-8"))?;
    let file_name = Utf8PathBuf::from(format!(
        "gauss-test-open-history-clear-{}.svg",
        Uuid::new_v4()
    ));
    let svg_path = temp_dir.join(&file_name);
    let dir = Dir::open_ambient_dir(&temp_dir, ambient_authority()).map_err(|error| {
        TestSupportError::expectation(format!("temp dir should be readable: {error}"))
    })?;
    let svg = r##"
        <svg xmlns="http://www.w3.org/2000/svg">
          <path d="M 8 8 L 24 24" stroke="#000000" stroke-width="1" fill="none" />
        </svg>
    "##;
    dir.write(file_name.as_path(), svg.as_bytes())
        .map_err(|error| {
            TestSupportError::expectation(format!("test SVG file should be writable: {error}"))
        })?;
    Ok(TempFileGuard::new_with_path(dir, file_name, svg_path))
}

fn seed_history_and_selection(
    visual_cx: &mut VisualTestContext,
    view: &Entity<Phase0Shell>,
) -> TestSupportResult<()> {
    let view_for_seed = view.clone();
    let seed_result = visual_cx.update(move |_window, app| -> TestSupportResult<()> {
        view_for_seed.update(app, |shell, _view_cx| {
            let shape_id = shell
                .document()
                .shape_at(0)
                .ok_or_else(|| {
                    TestSupportError::expectation("demo document should contain one shape")
                })?
                .id;
            let mut selection = Selection::default();
            selection.toggle(SelItem::Shape(shape_id));
            shell.replace_selection_for_tests(selection);
            shell
                .apply_command_for_tests(Command::MoveShapes {
                    movements: vec![ShapeMovement {
                        shape_id,
                        delta: Vec2::new(5.0, 0.0),
                    }],
                })
                .map_err(|error| {
                    TestSupportError::expectation(format!(
                        "seeding history command should apply: {error}"
                    ))
                })?;
            Ok(())
        })
    });
    seed_result?;
    visual_cx.run_until_parked();
    Ok(())
}

fn assert_seeded_state(visual_cx: &VisualTestContext, view: &Entity<Phase0Shell>) {
    assert_eq!(
        read_history_len(visual_cx, view),
        1,
        "expected one seeded history entry before open",
    );
    assert!(
        !read_selection(visual_cx, view).is_empty(),
        "expected seeded selection before open",
    );
}

#[gpui::test]
fn open_action_clears_document_history_and_selection_state(cx: &mut TestAppContext) {
    init_test_app(cx);
    let cleanup = create_open_fixture().expect("open fixture should be created");
    let svg_path_ref = cleanup.path().expect("temp file path should be set");

    let view: gpui::Entity<Phase0Shell> = {
        let (view, visual_cx) =
            cx.add_window_view(|_window, view_cx| Phase0Shell::new_for_tests(view_cx));
        ensure_initial_draw(visual_cx);
        seed_history_and_selection(visual_cx, &view).expect("seed should succeed");
        assert_seeded_state(visual_cx, &view);

        visual_cx.dispatch_action(OpenSvg);
        visual_cx.run_until_parked();
        view
    };
    cx.run_until_parked();

    cx.simulate_new_path_selection(|_directory: &Path| {
        Some(svg_path_ref.as_std_path().to_path_buf())
    });
    cx.run_until_parked();

    let (history_len, selection_is_empty) = cx.read(|app| {
        let shell = view.read(app);
        (
            shell.document_history_len_for_tests(),
            shell.selection().is_empty(),
        )
    });
    assert_eq!(
        history_len, 0,
        "open should clear document history for the newly loaded document",
    );
    assert!(
        selection_is_empty,
        "open should clear selection when loading a new document",
    );
}
