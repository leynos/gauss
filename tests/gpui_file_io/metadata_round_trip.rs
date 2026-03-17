//! GPUI headless integration tests for metadata round-trip through save/open.

use std::path::Path;

use camino::Utf8PathBuf;
use cap_std::{ambient_authority, fs_utf8::Dir};
use crate::crate::common::{TempFileGuard, ensure_initial_draw, init_test_app};
use gauss::svg::metadata::GAUSS_METADATA_NAMESPACE;
use gauss::ui::{OpenSvg, Phase0Shell, SaveSvg};
use gauss_core::test_helpers::shape_id_from_seed;
use gpui::TestAppContext;
use uuid::Uuid;

fn create_temp_file(
    prefix: &str,
) -> Result<(Utf8PathBuf, Utf8PathBuf, Dir, TempFileGuard), String> {
    let temp_dir = Utf8PathBuf::from_path_buf(std::env::temp_dir())
        .map_err(|_| "temp dir should be valid UTF-8".to_owned())?;
    let file_name = Utf8PathBuf::from(format!("{prefix}-{}.svg", Uuid::new_v4()));
    let full_path = temp_dir.join(&file_name);
    let dir = Dir::open_ambient_dir(&temp_dir, ambient_authority())
        .map_err(|err| format!("temp dir should be readable: {err}"))?;
    let cleanup = TempFileGuard::new(dir, file_name.clone());
    let dir2 = Dir::open_ambient_dir(&temp_dir, ambient_authority())
        .map_err(|err| format!("temp dir should be readable: {err}"))?;
    Ok((full_path, file_name, dir2, cleanup))
}

fn setup_save_and_get_contents(
    cx: &mut TestAppContext,
    configure_shell: impl FnOnce(&mut Phase0Shell),
    prefix: &str,
) -> (String, TempFileGuard) {
    let view = {
        let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
        ensure_initial_draw(visual_cx);
        view.update(visual_cx, |shell, _cx| {
            configure_shell(shell);
        });
        visual_cx.dispatch_action(SaveSvg);
        visual_cx.run_until_parked();
        view
    };
    cx.run_until_parked();

    let (expected, file_name, dir, cleanup) = match create_temp_file(prefix) {
        Ok(target) => target,
        Err(err) => panic!("{err}"),
    };

    cx.simulate_new_path_selection(|_directory: &Path| Some(expected.as_std_path().to_path_buf()));
    cx.run_until_parked();

    let saved = cx.read(|app| view.read(app).last_saved_path().map(Path::to_path_buf));
    assert_eq!(saved.as_deref(), Some(expected.as_std_path()));

    let contents = match dir.read_to_string(file_name.as_path()) {
        Ok(s) => s,
        Err(err) => panic!("Saved SVG file should be readable: {err}"),
    };
    (contents, cleanup)
}

#[gpui::test]
fn save_preserves_gauss_id(cx: &mut TestAppContext) {
    init_test_app(cx);

    let seed = 42_u128;
    let expected_id = shape_id_from_seed(seed);

    let (contents, _cleanup) = setup_save_and_get_contents(
        cx,
        |shell| {
            if let Some(shape) = shell.document_mut_for_tests().shape_at_mut(0) {
                shape.id = expected_id;
            }
        },
        "gauss-test-metadata-id",
    );

    let hex = gauss::svg::metadata::shape_id_to_hex(expected_id);
    assert!(
        contents.contains(&format!(r#"gauss:id="{hex}""#)),
        "Saved SVG should contain the shape's gauss:id, got: {contents}"
    );
}

#[gpui::test]
fn save_preserves_metadata_block(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (contents, _cleanup) = setup_save_and_get_contents(
        cx,
        |shell| {
            shell.set_gauss_metadata_block_for_tests(Some(
                "\n<gauss:project>test-project</gauss:project>".to_owned(),
            ));
        },
        "gauss-test-metadata-block",
    );

    assert!(
        contents.contains("<metadata>"),
        "Saved SVG should contain a <metadata> element"
    );
    assert!(
        contents.contains("<gauss:project>test-project</gauss:project>"),
        "Saved SVG should preserve the metadata block content"
    );
}

#[gpui::test]
fn open_restores_gauss_id(cx: &mut TestAppContext) {
    init_test_app(cx);

    let seed = 99_u128;
    let expected_id = shape_id_from_seed(seed);
    let hex = gauss::svg::metadata::shape_id_to_hex(expected_id);

    let temp_dir =
        Utf8PathBuf::from_path_buf(std::env::temp_dir()).expect("temp dir should be valid UTF-8");
    let file_name = Utf8PathBuf::from(format!(
        "gauss-test-open-metadata-id-{}.svg",
        Uuid::new_v4()
    ));
    let svg_path = temp_dir.join(&file_name);
    let dir =
        Dir::open_ambient_dir(&temp_dir, ambient_authority()).expect("temp dir should be readable");
    let svg = format!(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" xmlns:gauss="{GAUSS_METADATA_NAMESPACE}" width="100" height="100" viewBox="0 0 100 100">
<path d="M 0 0 L 10 10" stroke="#000000" stroke-width="1" fill="none" gauss:id="{hex}" />
</svg>
"##
    );
    dir.write(file_name.as_path(), svg.as_bytes())
        .expect("test SVG file should be writable");
    let cleanup = TempFileGuard::new_with_path(dir, file_name, svg_path);
    let svg_path_ref = cleanup.path().expect("temp file path should be set");

    let view: gpui::Entity<Phase0Shell> = {
        let (view, visual_cx) =
            cx.add_window_view(|_window, view_cx| Phase0Shell::new_for_tests(view_cx));
        ensure_initial_draw(visual_cx);
        visual_cx.dispatch_action(OpenSvg);
        visual_cx.run_until_parked();
        view
    };
    cx.run_until_parked();

    cx.simulate_new_path_selection(|_directory: &Path| {
        Some(svg_path_ref.as_std_path().to_path_buf())
    });
    cx.run_until_parked();

    let shape_id = cx.read(|app| {
        let shell = view.read(app);
        shell
            .document()
            .shape_at(0)
            .expect("shape should be imported")
            .id
    });
    assert_eq!(shape_id, expected_id);
}

#[gpui::test]
fn open_restores_metadata_block(cx: &mut TestAppContext) {
    init_test_app(cx);

    let temp_dir =
        Utf8PathBuf::from_path_buf(std::env::temp_dir()).expect("temp dir should be valid UTF-8");
    let file_name = Utf8PathBuf::from(format!(
        "gauss-test-open-metadata-block-{}.svg",
        Uuid::new_v4()
    ));
    let svg_path = temp_dir.join(&file_name);
    let dir =
        Dir::open_ambient_dir(&temp_dir, ambient_authority()).expect("temp dir should be readable");
    let svg = format!(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" xmlns:gauss="{GAUSS_METADATA_NAMESPACE}" width="100" height="100" viewBox="0 0 100 100">
<metadata>
<gauss:project>test-project</gauss:project>
</metadata>
<path d="M 0 0 L 10 10" stroke="#000000" stroke-width="1" fill="none" gauss:id="ffffffff00000001" />
</svg>
"##
    );
    dir.write(file_name.as_path(), svg.as_bytes())
        .expect("test SVG file should be writable");
    let cleanup = TempFileGuard::new_with_path(dir, file_name, svg_path);
    let svg_path_ref = cleanup.path().expect("temp file path should be set");

    let view: gpui::Entity<Phase0Shell> = {
        let (view, visual_cx) =
            cx.add_window_view(|_window, view_cx| Phase0Shell::new_for_tests(view_cx));
        ensure_initial_draw(visual_cx);
        visual_cx.dispatch_action(OpenSvg);
        visual_cx.run_until_parked();
        view
    };
    cx.run_until_parked();

    cx.simulate_new_path_selection(|_directory: &Path| {
        Some(svg_path_ref.as_std_path().to_path_buf())
    });
    cx.run_until_parked();

    let metadata_block = cx.read(|app| {
        let shell = view.read(app);
        shell.gauss_metadata_block().map(str::to_owned)
    });
    let Some(block) = metadata_block else {
        panic!("gauss_metadata_block should be populated after opening SVG with metadata");
    };
    assert!(
        block.contains("<gauss:project>test-project</gauss:project>"),
        "metadata block should contain the project element, got: {block}"
    );
}
