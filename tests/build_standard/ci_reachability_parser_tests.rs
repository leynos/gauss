//! Specifies the strict YAML subset and preserves literal workflow text exactly.

use super::super::{Value, parser};

/// Reads a scalar field through fallible mapping and scalar checks.
fn parser_scalar_field<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value.required(key, "parser fixture field")?.text(key)
}

/// Requires a parser result to fail with the diagnostic for its unsupported form.
fn parser_error_contains<T>(result: Result<T, String>, expected: &str) -> Result<(), String> {
    match result {
        Ok(_) => Err(format!("expected an error containing `{expected}`")),
        Err(error) if error.contains(expected) => Ok(()),
        Err(error) => Err(format!("expected `{expected}`, received `{error}`")),
    }
}

/// Parses an explicit document boundary while preserving the workflow mapping.
#[test]
fn parser_document_marker_fixture() {
    let document = parser::parse(include_str!(
        "fixtures/ci_reachability/linux-document-marker.yml"
    ))
    .expect("the document-marker fixture must parse");
    assert!(
        document.field("jobs").is_some(),
        "document markers retain jobs"
    );
}

/// Preserves hashes after doubled apostrophes in YAML single-quoted scalars.
#[test]
fn parser_preserves_doubled_single_quote_hash() {
    let document = parser::parse("on: push\nrelease: 'it''s #42'\n")
        .expect("the doubled-quote scalar must parse");
    assert_eq!(
        parser_scalar_field(&document, "release").expect("release must be scalar"),
        "it's #42",
        "doubled quote decodes"
    );
}

/// Preserves root boolean-like keys so workflow semantics can reject ambiguous trigger spellings.
#[test]
fn parser_preserves_boolean_like_root_keys() {
    let document = parser::parse("on: push\ntrue: false\n")
        .expect("boolean-like scalar spellings must remain parseable");
    assert_eq!(
        parser_scalar_field(&document, "on").expect("on must stay a scalar"),
        "push"
    );
    assert_eq!(
        parser_scalar_field(&document, "true").expect("true key must stay text"),
        "false"
    );
}

/// Rejects duplicate mapping keys after decoding quoted and plain spellings.
#[test]
fn parser_rejects_duplicate_decoded_keys() {
    parser_error_contains(
        parser::parse("on: push\njobs: {}\njobs: {}\n"),
        "duplicate decoded",
    )
    .expect("plain duplicate keys must be refused");
    parser_error_contains(
        parser::parse("on: {push: {}, \"push\": {}}\n"),
        "duplicate decoded",
    )
    .expect("quoted and plain flow keys must compare after decoding");
    parser_error_contains(
        parser::parse("on: push\njobs:\n  build: {}\n  \"build\": {}\n"),
        "duplicate decoded",
    )
    .expect("quoted and plain block keys must compare after decoding");
}

/// Rejects aliases, anchors, tags and merge keys instead of resolving hidden structure.
#[test]
fn parser_rejects_yaml_indirection() {
    for source in [
        "on: &events push\n",
        "on: *events\n",
        "on: !custom push\n",
        "<<: {on: push}\n",
    ] {
        parser_error_contains(parser::parse(source), "unsupported")
            .expect("YAML indirection must fail closed");
    }
}

/// Rejects empty documents, inconsistent nesting, unterminated quotes and incomplete flow lists.
#[test]
fn parser_rejects_empty_and_malformed_documents() {
    parser_error_contains(parser::parse("  # empty\n"), "empty")
        .expect("an empty document must be refused");
    parser_error_contains(
        parser::parse(concat!(
            "on: push\njobs:\n",
            "  build:\n    first: one\n",
            "     second: two\n"
        )),
        "indentation",
    )
    .expect("inconsistent sibling indentation must be refused");
    parser_error_contains(
        parser::parse("on: \"unterminated\n"),
        "unterminated quoted scalar",
    )
    .expect("an unterminated quote must be refused");
    parser_error_contains(
        parser::parse("on: [push\n"),
        "expected `,` or `]` in flow sequence",
    )
    .expect("an incomplete flow list must be refused for its syntax error");
}

/// Treats unmatched shell quotes inside a literal block as data, not YAML syntax.
#[test]
fn parser_preserves_literal_shell_quotes() {
    let document = parser::parse(concat!(
        "on: push\njobs:\n  build:\n    run: |\n",
        "      echo \"unterminated # shell text\n"
    ))
    .expect("literal shell text must not be parsed as a YAML quoted scalar");
    let job = document
        .required("jobs", "workflow")
        .expect("jobs must exist")
        .mapping("jobs")
        .expect("jobs must be a mapping")
        .get("build")
        .expect("build job must exist");
    assert_eq!(
        job.required("run", "job")
            .expect("run must exist")
            .text("run")
            .expect("run block must be a scalar"),
        "echo \"unterminated # shell text\n",
        "literal shell text is preserved verbatim"
    );
}

/// Keeps the first key and value of an inline sequence mapping and its nested mapping intact.
#[test]
fn parser_sequence_step_preserves_first_mapping_entry() {
    let document = parser::parse(concat!(
        "on: push\njobs:\n  build:\n    steps:\n",
        "      - name: Example\n        run: echo \"$GITHUB_REF\"\n",
        "        with:\n          field: value\n"
    ))
    .expect("a sequence mapping step must parse");
    let step = document
        .required("jobs", "workflow")
        .expect("jobs must exist")
        .mapping("jobs")
        .expect("jobs must be a mapping")
        .get("build")
        .expect("build job must exist")
        .required("steps", "job")
        .expect("steps must exist")
        .sequence("steps")
        .expect("steps must be a sequence")
        .first()
        .expect("the step must be present");
    assert_eq!(
        parser_scalar_field(step, "name").expect("first key must be retained"),
        "Example"
    );
    assert_eq!(
        parser_scalar_field(step, "run").expect("run key must be retained"),
        "echo \"$GITHUB_REF\""
    );
    let fields = step
        .required("with", "step")
        .expect("nested input mapping must exist")
        .mapping("with")
        .expect("step inputs must be a mapping");
    assert_eq!(
        fields
            .get("field")
            .expect("nested input must be retained")
            .text("field")
            .expect("nested input must be scalar"),
        "value"
    );
}

/// Preserves YAML folded-scalar line breaks around more-indented content.
#[test]
fn parser_folded_more_indented_lines_keep_line_breaks() {
    let document = parser::parse(concat!(
        "on: push\n",
        "message: >\n  first\n",
        "   indented\n  second\n"
    ))
    .expect("folded scalar with an indented line must parse");
    assert_eq!(
        parser_scalar_field(&document, "message").expect("message must be scalar"),
        "first\n indented\nsecond\n",
        "more-indented folded lines retain YAML line breaks"
    );
}

/// Rejects a leading blank line instead of discarding its shell-text boundary.
#[test]
fn parser_rejects_folded_leading_blank_line() {
    let source = "on: push\nmessage: >\n\n  first\n";
    let error = parser::parse(source).expect_err("leading folded blank must be rejected");
    assert_eq!(error, "line 2: folded blank lines are unsupported");
}

/// Rejects an interior blank line instead of changing its exact shell text.
#[test]
fn parser_rejects_folded_interior_blank_line() {
    let source = "on: push\nmessage: >\n  first\n\n  second\n";
    let error = parser::parse(source).expect_err("interior folded blank must be rejected");
    assert_eq!(error, "line 2: folded blank lines are unsupported");
}

/// Rejects a blank-only folded body while preserving genuinely absent bodies.
#[test]
fn parser_rejects_folded_blank_only_body() {
    let source = "on: push\nmessage: >\n\nnext: value\n";
    let error = parser::parse(source).expect_err("blank-only folded body must be rejected");
    assert_eq!(error, "line 2: folded blank lines are unsupported");
}

/// Rejects keep-chomped trailing blank lines whose folded value is not proven.
#[test]
fn parser_rejects_folded_trailing_keep_blank_line() {
    let source = "on: push\nmessage: >+\n  first\n\n";
    let error = parser::parse(source).expect_err("keep-chomped folded blank must be rejected");
    assert_eq!(error, "line 2: folded blank lines are unsupported");
}

/// Keeps root markers distinct from marker-looking text inside a literal block.
#[test]
fn parser_keeps_document_markers_inside_literal_text() {
    let document = parser::parse(concat!(
        "---\non: push\njobs:\n  build:\n    run: |\n",
        "      ---\n      ...\n",
        "... # end of document\n# trailing comment\n"
    ))
    .expect("one marked document with a trailing comment must parse");
    let run = document
        .required("jobs", "workflow")
        .expect("jobs must exist")
        .mapping("jobs")
        .expect("jobs must be a mapping")
        .get("build")
        .expect("build job must exist")
        .required("run", "job")
        .expect("run must exist")
        .text("run")
        .expect("run must be scalar");
    assert_eq!(
        parser_scalar_field(&document, "on").expect("trigger must be scalar"),
        "push"
    );
    assert_eq!(run, "---\n...\n", "literal marker text is preserved");
}

/// Rejects a second document even when the first has an explicit end marker.
#[test]
fn parser_rejects_an_additional_yaml_document() {
    let source = "---\non: push\n...\n---\non: pull_request\n";
    parser_error_contains(
        parser::parse(source),
        "multiple YAML documents or trailing content are unsupported",
    )
    .expect("a second YAML document must be refused");
}

/// Rejects repeated start and end markers instead of accepting empty documents.
#[test]
fn parser_rejects_repeated_document_markers() {
    for (source, expected) in [
        ("---\n---\non: push\n", "empty block mapping is unsupported"),
        (
            "on: push\n...\n...\n",
            "multiple YAML documents or trailing content are unsupported",
        ),
    ] {
        parser_error_contains(parser::parse(source), expected)
            .expect("a repeated document marker must be refused");
    }
}

/// Rejects mapping and scalar content after an explicit document end.
#[test]
fn parser_rejects_trailing_content_after_document_end() {
    for source in ["on: push\n...\njobs: {}\n", "on: push\n...\ntrailing\n"] {
        parser_error_contains(
            parser::parse(source),
            "multiple YAML documents or trailing content are unsupported",
        )
        .expect("content after the document end must be refused");
    }
}

/// Rejects indented start and end markers instead of treating them as root boundaries.
#[test]
fn parser_rejects_indented_document_markers() {
    parser_error_contains(
        parser::parse("  ---\non: push\n"),
        "expected a mapping entry",
    )
    .expect("an indented start marker must be refused");
    parser_error_contains(parser::parse("on: push\n  ...\n"), "unexpected indentation")
        .expect("an indented end marker must be refused");
}
