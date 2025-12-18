//! Phase 0 support helpers.
//!
//! Phase 0 deliberately keeps the UI surface area small, but it still needs a
//! few helpers for:
//!
//! - writing SVG files selected via platform dialogs, and
//! - creating a deterministic demo document to render and export.
//!
//! These helpers are split out to keep `src/ui/phase0_shell.rs` under the
//! repository’s file-size limit.

use std::path::Path;

use camino::{Utf8Path, Utf8PathBuf};
use cap_std::{ambient_authority, fs_utf8::Dir};
use uuid::Uuid;

use crate::{
    model::{Anchor, Document, PaintStyle, PathGeom, Rgba, SegmentKind, Shape, ShapeId, Vec2},
    svg::import::import_svg,
};

/// Write an SVG string to the provided filesystem path.
///
/// Phase 0 treats non-UTF-8 paths as unsupported and returns a string error.
pub(super) fn write_svg_to_path(path: &Path, svg: &str) -> Result<(), String> {
    let utf8_path = Utf8PathBuf::from_path_buf(path.to_path_buf())
        .map_err(|_| "path is not valid UTF-8".to_owned())?;
    let directory = utf8_path.parent().unwrap_or_else(|| Utf8Path::new("."));
    let file_name = utf8_path
        .file_name()
        .ok_or_else(|| "path does not include a file name".to_owned())?;

    Dir::create_ambient_dir_all(directory, ambient_authority()).map_err(|err| err.to_string())?;
    let dir =
        Dir::open_ambient_dir(directory, ambient_authority()).map_err(|err| err.to_string())?;

    dir.write(Utf8Path::new(file_name), svg.as_bytes())
        .map_err(|err| err.to_string())
}

/// Load a document from the provided SVG file path.
///
/// Phase 0 supports only a limited SVG subset (see `crate::svg::import`).
pub(super) fn load_document_from_path(path: &Path) -> Result<Document, String> {
    let utf8_path = Utf8PathBuf::from_path_buf(path.to_path_buf())
        .map_err(|_| "path is not valid UTF-8".to_owned())?;
    let directory = utf8_path.parent().unwrap_or_else(|| Utf8Path::new("."));
    let file_name = utf8_path
        .file_name()
        .ok_or_else(|| "path does not include a file name".to_owned())?;

    let dir =
        Dir::open_ambient_dir(directory, ambient_authority()).map_err(|err| err.to_string())?;
    let svg = dir
        .read_to_string(Utf8Path::new(file_name))
        .map_err(|err| err.to_string())?;

    import_svg(&svg).map_err(|err| err.to_string())
}

/// Build a deterministic document for Phase 0 rendering and export.
pub(super) fn demo_document() -> Document {
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
            closing_segment: SegmentKind::Line,
        },
    };

    Document {
        shapes: vec![shape],
    }
}
