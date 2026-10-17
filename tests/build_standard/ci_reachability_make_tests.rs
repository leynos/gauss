//! Specifies Make reachability, command classification, and source-backed tool identity.

use rstest::rstest;

use self::route_tests::{expected_cargo_rows, require_ordered_cargo_rows};
use super::{
    Host, Invocation, Route, make_output, make_routes, make_source, make_syntax, make_tools, tests,
};

/// The production Make source used as the suite-evaluation baseline.
const CURRENT_MAKE: &str = include_str!("../../Makefile");
/// A conditional that can swallow the Nextest fallback route.
const CONDITION_SWALLOW: &str =
    include_str!("fixtures/ci_reachability/make_condition_swallow/Makefile");
/// Explicit goal, pattern, alias, and selector routes for Make evaluation.
const ROUTES: &str = include_str!("fixtures/ci_reachability/make_routes/Makefile");
/// A sequential recipe whose successful prefix can mask a later failure.
const SEQUENTIAL_SWALLOW: &str =
    include_str!("fixtures/ci_reachability/make_sequential_swallow/Makefile");
/// The spelling goal before and after acquiring a test prerequisite.
const SPELLING_SUITE: &str = include_str!("fixtures/ci_reachability/make_spelling_suite/Makefile");
/// An unreviewed redirection that can hide command output.
const OUTPUT_REDIRECT: &str =
    include_str!("fixtures/ci_reachability/make_unreviewed_output_redirect/Makefile");
/// An unreviewed input redirection that can alter the executed route.
const INPUT_REDIRECT: &str =
    include_str!("fixtures/ci_reachability/make_unreviewed_redirect/Makefile");
/// A parse-time shell function outside either canonical lookup assignment.
const INLINE_EXECUTION: &str =
    include_str!("fixtures/ci_reachability/make_inline_execution/Makefile");
/// The exact supported Nextest availability branch and Cargo fallback.
const SAFE_NEXTTEST: &str = concat!(
    "if cargo nextest --version >/dev/null 2>&1; then ",
    "cargo nextest run --profile default --workspace --all-targets --all-features; ",
    "else echo \"cargo-nextest not installed, falling back to cargo test\"; ",
    "cargo test --workspace --all-targets --all-features; fi",
);
/// The exact protected-output command used to record `CodeScene` token presence.
const TOKEN_PRESENCE: &str =
    r#"echo "available=${{ secrets.CS_ACCESS_TOKEN != '' }}" >> "$GITHUB_OUTPUT""#;
/// A canonical token check prefixed by the one reviewed environment assignment.
const ASSIGNED_TOKEN_PRESENCE: &str = concat!(
    "PYTHONPATH=scripts ",
    r#"echo "available=${{ secrets.CS_ACCESS_TOKEN != '' }}" >> "$GITHUB_OUTPUT""#,
);

/// Fully explicit inputs for one future Make evaluation.
struct MakeCase<'a> {
    /// Complete source Makefile placed inside the temporary capability root.
    source: &'a str,
    /// One literal Make goal.
    goal: &'a str,
    /// Optional non-tool selector supplied explicitly on Make's command line.
    mode: Option<&'a str>,
    /// Host resolved by the workflow reader.
    host: Host,
}

/// Appends one named source mutation without altering the production fixture.
fn append_fixture(addition: &str) -> String {
    format!("{CURRENT_MAKE}\n{addition}")
}

/// Extracts one fixture recipe and substitutes only its reviewed Make tool variable.
fn fixture_recipe(source: &str) -> Result<String, String> {
    let line = source
        .lines()
        .find(|line| line.starts_with('\t'))
        .ok_or_else(|| "Make fixture has no recipe line".to_owned())?;
    let command = line
        .trim_start_matches('\t')
        .trim_start_matches(['@', '-', '+']);
    Ok(command.replace("$(CARGO)", "cargo").replace("$(UV)", "uv"))
}

/// Reports an output-classifier gap after its canonical suite baseline succeeds.
fn output_gap(source: &str) -> Result<String, String> {
    if make_output::classify(SAFE_NEXTTEST)? != Route::Suite {
        return Err("canonical Nextest/fallback baseline did not reach a suite".to_owned());
    }
    let command = fixture_recipe(source)?;
    make_output::classify(&command)
        .err()
        .ok_or_else(|| "Make shell mutation did not produce its expected gap".to_owned())
}

/// Retains only Cargo rows so every command printed by one Make goal has an oracle.
fn cargo_rows(evaluation: &make_output::Evaluation) -> Vec<&Invocation> {
    evaluation
        .invocations
        .iter()
        .filter(|invocation| {
            invocation
                .words
                .first()
                .is_some_and(|word| word == make_tools::PROBE_CARGO)
        })
        .collect()
}

/// Finds an ordered argv fragment without discarding additional Cargo arguments.
fn has_words(actual: &[String], expected: &[&str]) -> bool {
    actual.windows(expected.len()).any(|window| {
        window
            .iter()
            .zip(expected)
            .all(|(word, expected_word)| word == expected_word)
    })
}

/// Evaluates a requested goal in an exclusive fixture after staging the reviewed script bodies.
fn evaluate_case(case: &MakeCase<'_>) -> Result<make_output::Evaluation, String> {
    let goals = vec![case.goal.to_owned()];
    let assignments = Vec::new();
    tests::with_fixture(&[], |workspace| {
        workspace
            .directory
            .write("Makefile", case.source)
            .map_err(|error| format!("writing Make evaluation source: {error}"))?;
        make_tools::stage_python_bodies(workspace, make_tools::LineEnding::Lf)?;
        make_routes::evaluate_make(
            workspace,
            &make_routes::MakeRequest {
                goals: &goals,
                mode: case.mode,
                assignments: &assignments,
                host: case.host,
            },
        )
    })
}

/// Evaluates one production Make goal with its supported workflow host.
fn evaluate_current(goal: &str, host: Host) -> Result<make_output::Evaluation, String> {
    evaluate_case(&MakeCase {
        source: CURRENT_MAKE,
        goal,
        mode: None,
        host,
    })
}

/// Classifies a workflow command inside the same capability boundary used by consumer tests.
/// For example, the token echo resolves as `NonSuite`, but appending
/// `&& cargo test --workspace` resolves as `Suite`.
fn classify_workflow_command(command: &str) -> Result<Route, String> {
    tests::with_fixture(&[], |workspace| {
        make_routes::classify(command, workspace, Host::Linux)
    })
}

/// The exact token check resolves narrowly and keeps following commands visible to the reader.
#[test]
fn workflow_codescene_token_presence_echo_is_exact_and_preserves_following_routes() {
    assert_eq!(
        classify_workflow_command(TOKEN_PRESENCE)
            .expect("the canonical token-presence command must resolve"),
        Route::NonSuite
    );

    let suite = format!("{TOKEN_PRESENCE} && cargo test --workspace");
    assert_eq!(
        classify_workflow_command(&suite).expect("the following Cargo suite must remain visible"),
        Route::Suite
    );

    let masked = format!("{TOKEN_PRESENCE}; cargo test --workspace");
    let masked_error = classify_workflow_command(&masked)
        .expect_err("a successful token check must not mask the following suite");
    assert_eq!(masked_error, "shell sequence can hide a command failure");

    let unknown = format!("{TOKEN_PRESENCE} && unreviewed-tool command");
    let unknown_error = classify_workflow_command(&unknown)
        .expect_err("the token check must not hide an unknown following command");
    assert_eq!(
        unknown_error,
        "unresolved executable route: unreviewed-tool"
    );
}

/// Near-miss echoes cannot use the exact protected-output exception or its assignment route.
#[rstest]
#[case::make_echo_arbitrary("echo arbitrary", "unresolved executable route: echo")]
#[case::make_echo_changed_word(
    r#"echo "available=false" >> "$GITHUB_OUTPUT""#,
    "unreviewed shell redirection"
)]
#[case::make_echo_changed_target(
    r#"echo "available=${{ secrets.CS_ACCESS_TOKEN != '' }}" >> "$OTHER_OUTPUT""#,
    "unreviewed shell redirection"
)]
#[case::make_echo_substitution(
    r#"echo $(printf available) >> "$GITHUB_OUTPUT""#,
    "unreviewed shell expansion in command word"
)]
#[case::make_echo_extra_argument(
    r#"echo "available=${{ secrets.CS_ACCESS_TOKEN != '' }}" extra >> "$GITHUB_OUTPUT""#,
    "unreviewed shell redirection"
)]
#[case::make_echo_assignment(ASSIGNED_TOKEN_PRESENCE, "unreviewed shell redirection")]
#[case::make_echo_unknown_assignment(
    r#"TOKEN=1 echo "available=${{ secrets.CS_ACCESS_TOKEN != '' }}" >> "$GITHUB_OUTPUT""#,
    "unreviewed shell environment assignment: TOKEN"
)]
fn workflow_token_presence_near_misses_have_specific_gaps(
    #[case] command: &str,
    #[case] expected: &str,
) {
    let error = classify_workflow_command(command)
        .expect_err("only the canonical token-presence command may bypass tool classification");
    assert_eq!(error, expected);
}

/// The shell classifier preserves conditional delimiters and rejects noncanonical suite control flow.
#[rstest]
#[case::make_condition_swallow(CONDITION_SWALLOW, "shell conditional has no else")]
#[case::make_sequential_swallow(SEQUENTIAL_SWALLOW, "shell sequence can hide a command failure")]
#[case::make_unreviewed_output_redirect(OUTPUT_REDIRECT, "unreviewed shell redirection")]
#[case::make_unreviewed_redirect(INPUT_REDIRECT, "unreviewed shell redirection")]
fn shell_mutations_have_specific_gaps(#[case] source: &str, #[case] expected: &str) {
    assert_eq!(
        output_gap(source).expect("shell failure mutation must be reported"),
        expected
    );
}

/// Trace locations outside a literal rule or any of its recipe lines remain explicit gaps.
#[test]
fn make_trace_unbound_location_is_rejected() {
    let block = make_output::TraceBlock {
        file: "Makefile".to_owned(),
        line: 0,
        target: "test".to_owned(),
        command: "cargo test --workspace".to_owned(),
    };
    let error = make_routes::verify_trace(CURRENT_MAKE, &[block])
        .expect_err("an unbound trace location must not pass");
    assert_eq!(error, "GNU Make trace has a zero source line");
}

/// Canonical shell lookups stay location-bound and inline recipes fail before Make runs.
#[test]
fn make_inline_execution_and_shell_exemptions_are_location_bound() {
    make_source::validate(CURRENT_MAKE).expect("canonical source assignments must pass");
    let renamed_lookup = append_fixture(&INLINE_EXECUTION.replace(
        "UNREVIEWED_INPUT := $(shell printf unsafe)",
        "UNREVIEWED_INPUT := $(shell uname -s)",
    ));
    let lookup_error = make_source::validate(&renamed_lookup)
        .expect_err("a canonical lookup at an unreviewed location must fail");
    assert_eq!(lookup_error, "unreviewed Make execution function: shell");

    let safe_fixture = append_fixture(&INLINE_EXECUTION.replace(
        "UNREVIEWED_INPUT := $(shell printf unsafe)",
        "UNREVIEWED_INPUT := safe",
    ));
    make_source::validate(&safe_fixture).expect("fixture without parse-time execution must pass");
    let inline_recipe = safe_fixture.replace(
        "inline-route:\n\t$(CARGO) test --workspace",
        "inline-route: ; +RUSTFLAGS=x cargo test --workspace",
    );
    let recipe_error = make_source::validate(&inline_recipe)
        .expect_err("inline assignment and + recipe must fail before dry-run evaluation");
    assert_eq!(
        recipe_error,
        "inline Make recipe cannot be proved safe under dry-run"
    );

    let spaced_prefix = safe_fixture.replace(
        "\t$(CARGO) test --workspace",
        "\t +$(CARGO) test --workspace",
    );
    let whitespace_error = make_source::validate(&spaced_prefix)
        .expect_err("ambiguous recipe-prefix whitespace must fail closed");
    assert_eq!(
        whitespace_error,
        "Make recipe has ambiguous leading whitespace"
    );
}

/// Quoted syntax, shell substitution, and OR fallback cannot manufacture suite evidence.
#[test]
fn shell_quotes_and_short_circuit_expansions_are_gaps() {
    assert_eq!(make_output::classify(SAFE_NEXTTEST), Ok(Route::Suite));
    assert_eq!(
        make_output::classify("cargo test --workspace"),
        Ok(Route::Suite)
    );

    let substitution = make_output::classify("cargo test $(echo --no-run)")
        .expect_err("command substitution must not hide a nonexecuting Cargo switch");
    assert_eq!(substitution, "unreviewed shell expansion in command word");

    let quoted_control = make_output::classify("echo 'if'")
        .expect_err("quoted control text must not become shell syntax");
    assert_eq!(quoted_control, "quoted shell control word is unsupported");

    let empty = make_syntax::tokenize("echo \"\"")
        .expect_err("an empty quoted word must not disappear from argv");
    assert_eq!(empty, "empty quoted shell word is unsupported");

    let short_circuit = SAFE_NEXTTEST.replace(
        "echo \"cargo-nextest not installed, falling back to cargo test\";",
        "echo \"cargo-nextest not installed, falling back to cargo test\" ||",
    );
    let error = make_output::classify(&short_circuit)
        .expect_err("successful echo must not skip the Cargo fallback");
    assert_eq!(
        error,
        "Cargo fallback branch contains a failure-masking command"
    );
}

/// A goal available only through Make builtin implicit rules remains a failed reader route.
#[test]
fn implicit_only_goal_is_a_gap_while_repository_pattern_rule_remains_available() {
    let explicit = append_fixture(ROUTES);
    let positive = evaluate_case(&MakeCase {
        source: &explicit,
        goal: "suite-smoke",
        mode: Some("coverage"),
        host: Host::Linux,
    })
    .expect("repository-defined Cargo pattern prerequisite must remain available under -r");
    assert_eq!(positive.route, Route::Suite);
    let expected = expected_cargo_rows("default").expect("coverage rows must be specified");
    require_ordered_cargo_rows(&cargo_rows(&positive), &expected)
        .expect("the repository pattern prerequisite must preserve every Cargo row");

    let error = evaluate_case(&MakeCase {
        source: CURRENT_MAKE,
        goal: "route.o",
        mode: Some("coverage"),
        host: Host::Linux,
    })
    .expect_err("an implicit-only goal must not be reported as non-suite success");
    assert!(
        error.contains("No rule to make target"),
        "unexpected implicit route result: {error}"
    );
}

/// Direct Cargo suite classification refuses nonexecuting switches and unknown subcommands.
#[rstest]
#[case::make_test_help(&["cargo", "test", "--help"], "--help")]
#[case::make_test_no_run(&["cargo", "test", "--no-run"], "--no-run")]
#[case::make_test_list(&["cargo", "test", "--", "--list"], "--list")]
#[case::make_nextest_list(&["cargo", "nextest", "list"], "unresolved Cargo subcommand: nextest")]
fn cargo_nonexecution_modifiers_are_gaps(#[case] words: &[&str], #[case] diagnostic: &str) {
    let baseline = ["cargo", "test", "--workspace"].map(str::to_owned);
    assert_eq!(
        make_tools::classify_words(&baseline, false),
        Ok(Route::Suite)
    );
    let mutated = words
        .iter()
        .map(|word| (*word).to_owned())
        .collect::<Vec<_>>();
    let error = make_tools::classify_words(&mutated, false)
        .expect_err("nonexecuting or unknown Cargo route must not count as a suite");
    assert!(
        error.contains(diagnostic),
        "wrong Cargo route diagnostic: {error}"
    );
}

/// Keeps evaluated Make route cases beside the shared fixture helpers.
#[cfg(test)]
#[path = "ci_reachability_make_route_tests.rs"]
mod route_tests;
