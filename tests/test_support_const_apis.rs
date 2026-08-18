//! Compile-time contracts for capability-sized GPUI test-support APIs.

//! These constants ensure that callers can use the declared const APIs during
//! compile-time evaluation, rather than only through runtime test scenarios.

#[path = "common/canvas.rs"]
mod canvas;
#[path = "common/modifiers.rs"]
mod modifiers;
#[path = "common/temp_file.rs"]
mod temp_file;

use camino::{Utf8Path, Utf8PathBuf};
use cap_std::fs_utf8::Dir;
use gpui::Modifiers;
use temp_file::TempFileGuard;

const CANVAS_PADDING_IN_CONST_CONTEXT: f32 = canvas::CANVAS_PADDING_PX;
const SHIFT_SECONDARY_IN_CONST_CONTEXT: Modifiers = modifiers::shift_secondary(Modifiers {
    control: false,
    alt: false,
    shift: false,
    platform: false,
    function: false,
});

const _: () = assert!(
    CANVAS_PADDING_IN_CONST_CONTEXT == 2.0,
    "canvas padding must remain two pixels in const contexts",
);
const _: () = assert!(
    SHIFT_SECONDARY_IN_CONST_CONTEXT.shift,
    "shift_secondary must enable Shift in const contexts",
);

const fn construct_temp_file_guard_in_const_context(
    dir: Dir,
    file_name: Utf8PathBuf,
    path: Utf8PathBuf,
) -> TempFileGuard {
    TempFileGuard::new(dir, file_name, path)
}

fn guarded_path(guard: &TempFileGuard) -> &Utf8Path {
    guard.path()
}

#[test]
fn const_apis_compile_in_const_contexts() {
    let constructor = construct_temp_file_guard_in_const_context;
    let path = guarded_path;
    let _ = std::hint::black_box((constructor, path));
}
