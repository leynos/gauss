//! Given steps that prepare shell state for Save and web-ready export scenarios.

use gauss::{
    model::{
        Gradient, GradientKind, GradientStop, LinearGradient, Paint, PatternResource, Rgba, Vec2,
    },
    ui::Phase0Shell,
};
use gpui::TestAppContext;
use rstest_bdd_macros::given;
use test_support::TestSupportError;

use super::{ExportAction, prepare_shell};

fn setup_dangling_gradient(shell: &mut Phase0Shell) {
    let id = shell
        .resources_mut_for_tests()
        .insert_gradient(Gradient::new(
            "dangling",
            GradientKind::Linear(LinearGradient::new(
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                vec![
                    GradientStop::new(0.0, Rgba::new(255, 0, 0, 255)),
                    GradientStop::new(1.0, Rgba::new(255, 255, 0, 255)),
                ],
            )),
        ));
    let _removed = shell.resources_mut_for_tests().remove_gradient(id);
    if let Some(shape) = shell.document_mut_for_tests().shape_at_mut(0) {
        shape.style.fill = Paint::gradient(id);
    }
}

fn setup_dangling_pattern(shell: &mut Phase0Shell) {
    let id = shell
        .resources_mut_for_tests()
        .insert_pattern(PatternResource::new("dangling-pattern", "<circle />"));
    let _removed = shell.resources_mut_for_tests().remove_pattern(id);
    if let Some(shape) = shell.document_mut_for_tests().shape_at_mut(0) {
        shape.style.fill = Paint::pattern(id);
    }
}

#[given("a fresh shell prepared for a standard save")]
fn standard_save_shell(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    prepare_shell(cx, ExportAction::Save, |_shell| {})
}

#[given("a shell with Gauss metadata prepared for a web-ready export")]
fn web_ready_shell_with_metadata(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    prepare_shell(cx, ExportAction::WebReady, |shell| {
        shell.set_gauss_metadata_block_for_tests(Some(
            "<gauss:test gauss:purpose=\"round-trip\" />".to_owned(),
        ));
    })
}

macro_rules! dangling_given {
    ($function:ident, $text:literal, $action:expr, $configure:expr) => {
        #[given($text)]
        fn $function(
            #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
        ) -> Result<(), TestSupportError> {
            prepare_shell(cx, $action, $configure)
        }
    };
}

dangling_given!(
    save_dangling_gradient,
    "a shell with a dangling gradient reference prepared for a standard save",
    ExportAction::Save,
    setup_dangling_gradient
);
dangling_given!(
    save_dangling_pattern,
    "a shell with a dangling pattern reference prepared for a standard save",
    ExportAction::Save,
    setup_dangling_pattern
);
dangling_given!(
    web_ready_dangling_gradient,
    "a shell with a dangling gradient reference prepared for a web-ready export",
    ExportAction::WebReady,
    setup_dangling_gradient
);
dangling_given!(
    web_ready_dangling_pattern,
    "a shell with a dangling pattern reference prepared for a web-ready export",
    ExportAction::WebReady,
    setup_dangling_pattern
);
