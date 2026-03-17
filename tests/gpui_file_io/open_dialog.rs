//! GPUI headless integration tests for Gauss “Open…” wiring.
//!
//! Note: GPUI 0.2.2's test platform does not implement `prompt_for_paths`.
//! Phase 0 therefore routes “Open…” through `prompt_for_new_path` when the
//! `Phase0Shell` is constructed via `Phase0Shell::new_for_tests`.

use std::path::Path;

use camino::Utf8PathBuf;
use cap_std::{ambient_authority, fs_utf8::Dir};
use crate::crate::common::{TempFileGuard, ensure_initial_draw, init_test_app};
use gauss::model::Paint;
use gauss::svg::metadata::{GAUSS_METADATA_NAMESPACE, GAUSS_METADATA_PREFIX};
use gauss::ui::{OpenSvg, Phase0Shell};
use gpui::TestAppContext;
use uuid::Uuid;

#[gpui::test]
fn open_action_loads_selected_svg(cx: &mut TestAppContext) {
    init_test_app(cx);

    let temp_dir =
        Utf8PathBuf::from_path_buf(std::env::temp_dir()).expect("temp dir should be valid UTF-8");
    let file_name = Utf8PathBuf::from(format!("gauss-test-open-{}.svg", Uuid::new_v4()));
    let svg_path = temp_dir.join(&file_name);
    let dir =
        Dir::open_ambient_dir(&temp_dir, ambient_authority()).expect("temp dir should be readable");
    let svg = r##"
        <svg xmlns="http://www.w3.org/2000/svg">
          <path d="M 1 2 L 3 4" stroke="#000000" stroke-width="1" fill="none" />
        </svg>
    "##;
    dir.write(file_name.as_path(), svg.as_bytes())
        .expect("test SVG file should be writable");
    let cleanup = TempFileGuard::new_with_path(dir, file_name, svg_path);
    let svg_path_ref = cleanup.path().expect("temp file path should be set");

    assert!(
        !cx.did_prompt_for_new_path(),
        "No open prompt should be visible before triggering Open"
    );

    let view: gpui::Entity<Phase0Shell> = {
        let (view, visual_cx) =
            cx.add_window_view(|_window, view_cx| Phase0Shell::new_for_tests(view_cx));

        ensure_initial_draw(visual_cx);
        visual_cx.dispatch_action(OpenSvg);
        visual_cx.run_until_parked();

        view
    };
    cx.run_until_parked();

    assert!(
        cx.did_prompt_for_new_path(),
        "Open action should prompt for a path (test backend uses new-path prompt)"
    );

    cx.simulate_new_path_selection(|_directory: &Path| {
        Some(svg_path_ref.as_std_path().to_path_buf())
    });
    cx.run_until_parked();

    let opened = cx.read(|app| view.read(app).last_opened_path().map(Path::to_path_buf));
    assert_eq!(opened.as_deref(), Some(svg_path_ref.as_std_path()));

    let shape_count = cx.read(|app| view.read(app).document().len());
    assert_eq!(shape_count, 1);

    let gradient_count = cx.read(|app| view.read(app).resources().gradient_count());
    let pattern_count = cx.read(|app| view.read(app).resources().pattern_count());
    assert_eq!(gradient_count, 0);
    assert_eq!(pattern_count, 0);
}

#[gpui::test]
fn open_action_loads_resource_defs_and_paint_references(cx: &mut TestAppContext) {
    init_test_app(cx);

    let temp_dir =
        Utf8PathBuf::from_path_buf(std::env::temp_dir()).expect("temp dir should be valid UTF-8");
    let file_name = Utf8PathBuf::from(format!("gauss-test-open-resources-{}.svg", Uuid::new_v4()));
    let svg_path = temp_dir.join(&file_name);
    let dir =
        Dir::open_ambient_dir(&temp_dir, ambient_authority()).expect("temp dir should be readable");
    let svg = r##"
        <svg xmlns="http://www.w3.org/2000/svg">
          <defs>
            <linearGradient id="sunset" x1="0" y1="0" x2="1" y2="0">
              <stop offset="0" stop-color="#ff0000" />
              <stop offset="1" stop-color="#ffff00" />
            </linearGradient>
            <pattern id="dots"><circle cx="1" cy="1" r="1" /></pattern>
          </defs>
          <path d="M 1 2 L 3 4 Z" stroke="url(#sunset)" stroke-width="1" fill="url(#dots)" />
        </svg>
    "##;
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

    let (gradient_count, pattern_count, shape_stroke, shape_fill, expected_stroke, expected_fill) =
        cx.read(|app| {
            let shell = view.read(app);
            let gradient_id = shell
                .resources()
                .gradient_id_for_svg_id("sunset")
                .expect("gradient id should exist");
            let pattern_id = shell
                .resources()
                .pattern_id_for_svg_id("dots")
                .expect("pattern id should exist");
            let shape = shell
                .document()
                .shape_at(0)
                .expect("shape should be imported");
            (
                shell.resources().gradient_count(),
                shell.resources().pattern_count(),
                shape.style.stroke,
                shape.style.fill,
                Paint::gradient(gradient_id),
                Paint::pattern(pattern_id),
            )
        });

    assert_eq!(gradient_count, 1);
    assert_eq!(pattern_count, 1);
    assert_eq!(shape_stroke, expected_stroke);
    assert_eq!(shape_fill, expected_fill);
}

#[gpui::test]
fn open_action_reports_missing_resource_reference(cx: &mut TestAppContext) {
    init_test_app(cx);

    let temp_dir =
        Utf8PathBuf::from_path_buf(std::env::temp_dir()).expect("temp dir should be valid UTF-8");
    let file_name = Utf8PathBuf::from(format!("gauss-test-open-invalid-{}.svg", Uuid::new_v4()));
    let svg_path = temp_dir.join(&file_name);
    let dir =
        Dir::open_ambient_dir(&temp_dir, ambient_authority()).expect("temp dir should be readable");
    let svg = r##"
        <svg xmlns="http://www.w3.org/2000/svg">
          <path d="M 1 2 L 3 4" stroke="#000000" stroke-width="1" fill="url(#missing)" />
        </svg>
    "##;
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

    let (initial_gradient_count, initial_pattern_count, initial_symbol_count) = cx.read(|app| {
        let shell = view.read(app);
        (
            shell.resources().gradient_count(),
            shell.resources().pattern_count(),
            shell.resources().symbol_count(),
        )
    });

    cx.simulate_new_path_selection(|_directory: &Path| {
        Some(svg_path_ref.as_std_path().to_path_buf())
    });
    cx.run_until_parked();

    let (shape_count, gradient_count, pattern_count, symbol_count, open_error) = cx.read(|app| {
        let shell = view.read(app);
        (
            shell.document().len(),
            shell.resources().gradient_count(),
            shell.resources().pattern_count(),
            shell.resources().symbol_count(),
            shell.last_open_error().map(str::to_owned),
        )
    });

    assert_eq!(
        shape_count, 1,
        "failed open should keep the pre-existing demo document"
    );
    assert_eq!(gradient_count, initial_gradient_count);
    assert_eq!(pattern_count, initial_pattern_count);
    assert_eq!(symbol_count, initial_symbol_count);
    let error_message = open_error.expect("open error should be populated");
    assert!(
        error_message.contains("missing resource"),
        "error should describe missing referenced resources, got: {error_message}"
    );
}

#[gpui::test]
fn open_action_accepts_canonical_gauss_metadata_namespace(cx: &mut TestAppContext) {
    init_test_app(cx);

    let temp_dir =
        Utf8PathBuf::from_path_buf(std::env::temp_dir()).expect("temp dir should be valid UTF-8");
    let file_name = Utf8PathBuf::from(format!(
        "gauss-test-open-gauss-metadata-{}.svg",
        Uuid::new_v4()
    ));
    let svg_path = temp_dir.join(&file_name);
    let dir =
        Dir::open_ambient_dir(&temp_dir, ambient_authority()).expect("temp dir should be readable");
    let svg = format!(
        r##"
        <svg xmlns="http://www.w3.org/2000/svg" xmlns:gauss="{GAUSS_METADATA_NAMESPACE}">
          <metadata>
            <gauss:editor version="1" />
          </metadata>
          <path d="M 1 2 L 3 4" stroke="#000000" stroke-width="1" fill="none" />
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

    let (shape_count, open_error) = cx.read(|app| {
        let shell = view.read(app);
        (
            shell.document().len(),
            shell.last_open_error().map(str::to_owned),
        )
    });
    assert_eq!(shape_count, 1);
    assert!(
        open_error.is_none(),
        "open should succeed for canonical metadata"
    );
}

#[gpui::test]
fn open_action_rejects_gauss_namespace_without_canonical_prefix(cx: &mut TestAppContext) {
    init_test_app(cx);

    let temp_dir =
        Utf8PathBuf::from_path_buf(std::env::temp_dir()).expect("temp dir should be valid UTF-8");
    let file_name = Utf8PathBuf::from(format!(
        "gauss-test-open-invalid-gauss-{}.svg",
        Uuid::new_v4()
    ));
    let svg_path = temp_dir.join(&file_name);
    let dir =
        Dir::open_ambient_dir(&temp_dir, ambient_authority()).expect("temp dir should be readable");
    let svg = format!(
        r##"
        <svg xmlns="http://www.w3.org/2000/svg" xmlns:g="{GAUSS_METADATA_NAMESPACE}">
          <metadata><g:editor version="1" /></metadata>
          <path d="M 1 2 L 3 4" stroke="#000000" stroke-width="1" fill="none" />
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

    let open_error = cx.read(|app| view.read(app).last_open_error().map(str::to_owned));
    let Some(error_message) = open_error else {
        panic!("open should fail when gauss namespace policy is violated");
    };
    let expected_namespace = format!("xmlns:{GAUSS_METADATA_PREFIX}");
    assert!(
        error_message.contains(expected_namespace.as_str()),
        "error should mention canonical gauss namespace declaration, got: {error_message}"
    );
}
