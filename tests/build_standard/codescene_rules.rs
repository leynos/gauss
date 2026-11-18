//! Contract for the `CodeScene` string-heavy exemptions in this repository.
//!
//! ADR 0007 records why the workflow-contract modules that decode YAML, Make,
//! shell and argv text are exempted from `String Heavy Function Arguments`, and
//! why a genuine parse boundary must instead gain a validating newtype. These
//! tests keep the exemption set exact, documented and free of inline
//! directives, so widening it needs an explicit source change rather than a
//! quiet edit to the rule file.

use std::collections::BTreeSet;

use camino::Utf8Path;
use cap_std::{ambient_authority, fs_utf8::Dir};
use serde_json::{Value, json};

/// The tracked `CodeScene` rule file.
const RULES: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/.codescene/code-health-rules.json"
));

/// The single rule this repository overrides.
const RULE: &str = "String Heavy Function Arguments";

/// The exact paths authorized by ADR 0007.
///
/// Adding or removing a path means changing the decision record and this list
/// together; the rule file alone is not sufficient.
const EXEMPT_PATHS: [&str; 7] = [
    "tests/build_standard/ci_reachability.rs",
    "tests/build_standard/ci_reachability_make_source.rs",
    "tests/build_standard/ci_reachability_make_source_controls.rs",
    "tests/build_standard/ci_reachability_make_tests.rs",
    "tests/build_standard/ci_reachability_make_tools.rs",
    "tests/build_standard/ci_reachability_make_trace.rs",
    "tests/build_standard/ci_reachability_workflows_tests.rs",
];

/// Opens the repository root through the capability boundary used by tests.
fn project_directory() -> Result<Dir, String> {
    Dir::open_ambient_dir(env!("CARGO_MANIFEST_DIR"), ambient_authority())
        .map_err(|error| format!("opening the project capability: {error}"))
}

/// Parses the rule file, failing with the parser error when it is malformed.
fn rule_document() -> Result<Value, String> {
    serde_json::from_str(RULES)
        .map_err(|error| format!("the CodeScene rule file is not valid JSON: {error}"))
}

/// Returns one required string field of a JSON object.
fn string_field<'value>(object: &'value Value, name: &str) -> Result<&'value str, String> {
    object
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("rule set is missing the string field `{name}`: {object}"))
}

/// Returns the rule sets as an array, rejecting any other shape.
fn rule_sets(document: &Value) -> Result<&[Value], String> {
    document
        .get("rule_sets")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| "the CodeScene rule file has no `rule_sets` array".to_owned())
}

/// Requires one exact repository-relative path with no glob metacharacter.
fn exact_path(rule_set: &Value) -> Result<&str, String> {
    let path = string_field(rule_set, "matching_content_path")?;
    if path.is_empty() || path.chars().any(|character| "*?[]{}".contains(character)) {
        return Err(format!(
            "`matching_content_path` must be one exact path, not a glob: {path}"
        ));
    }
    let is_rust_file = Utf8Path::new(path)
        .extension()
        .is_some_and(|file_extension| file_extension.eq_ignore_ascii_case("rs"));
    if !path.starts_with("tests/build_standard/") || !is_rust_file {
        return Err(format!(
            "`matching_content_path` must name a workflow-contract Rust file: {path}"
        ));
    }
    Ok(path)
}

/// Requires a non-empty rationale that cites the governing issue.
fn rationale(rule_set: &Value, path: &str) -> Result<(), String> {
    let document = string_field(rule_set, "matching_content_path_doc")?;
    if document.trim().is_empty() || !document.contains("#257") {
        return Err(format!(
            "the rationale for {path} must be non-empty and cite issue #257"
        ));
    }
    Ok(())
}

/// Requires exactly one override of [`RULE`] at zero weight and no thresholds.
fn only_rule_override(rule_set: &Value, path: &str) -> Result<(), String> {
    let overrides = rule_set
        .get("rules")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{path} has no `rules` array"))?;
    if overrides.len() != 1 {
        return Err(format!(
            "{path} must override exactly one rule, found {}",
            overrides.len()
        ));
    }
    let rule = overrides
        .first()
        .ok_or_else(|| format!("{path} has an empty `rules` array"))?;
    if string_field(rule, "name")? != RULE {
        return Err(format!("{path} may override only `{RULE}`"));
    }
    let weight = rule
        .get("weight")
        .ok_or_else(|| format!("{path} has no rule weight"))?;
    if !is_zero_weight(weight) {
        return Err(format!(
            "{path} must disable `{RULE}` with weight 0.0, found {weight}"
        ));
    }
    match rule_set.get("thresholds") {
        None => {}
        Some(Value::Array(thresholds)) if thresholds.is_empty() => {}
        Some(Value::Array(_)) => {
            return Err(format!(
                "{path} must not override thresholds; ADR 0007 authorizes exact-path rule weights \
                 only"
            ));
        }
        Some(other) => {
            return Err(format!(
                "{path} has a `thresholds` field that is not an array: {other}"
            ));
        }
    }
    Ok(())
}

/// Reports whether a JSON weight disables the rule.
///
/// `CodeScene` accepts integer `0` and floating-point `0.0`, and `serde_json`
/// keeps those two spellings distinct in `Value` equality, so both must be
/// accepted while any non-numeric value is rejected.
fn is_zero_weight(weight: &Value) -> bool { *weight == json!(0) || *weight == json!(0.0) }

/// The usage note points readers at the decision and the developer guidance.
fn usage_document_reference(document: &Value) -> Result<(), String> {
    let usage = string_field(document, "usage")?;
    if !usage.contains("docs/developers-guide.md#codescene-string-heavy-arguments") {
        return Err(format!(
            "the rule file `usage` must link the developer guidance anchor, found: {usage}"
        ));
    }
    Ok(())
}

/// Validates the rule document and returns the exact path set it declares.
///
/// This is the document-only boundary: it checks the usage note, every rule
/// set, duplicate paths and the authorized path set without touching the
/// filesystem, so the guards can be exercised against mutated fixtures.
fn validated_paths(document: &Value) -> Result<BTreeSet<&str>, String> {
    usage_document_reference(document)?;
    let mut seen = BTreeSet::new();
    for rule_set in rule_sets(document)? {
        let path = exact_path(rule_set)?;
        rationale(rule_set, path)?;
        only_rule_override(rule_set, path)?;
        if !seen.insert(path) {
            return Err(format!("{path} is exempted more than once"));
        }
    }
    let expected: BTreeSet<&str> = EXEMPT_PATHS.into_iter().collect();
    if seen != expected {
        return Err(format!(
            "the rule file paths must match ADR 0007 exactly; expected {expected:?}, found \
             {seen:?}"
        ));
    }
    Ok(seen)
}

/// Every authorized module exists, so no exemption outlives its source.
///
/// The expected path set is compared exactly, so a new exemption cannot be
/// added to the rule file without updating ADR 0007 and this contract.
#[test]
fn rule_sets_are_exact_documented_and_authorized() -> Result<(), String> {
    let document = rule_document()?;
    let directory = project_directory()?;
    for path in validated_paths(&document)? {
        directory
            .read_to_string(path)
            .map_err(|error| format!("the exempt module {path} is not readable: {error}"))?;
    }
    Ok(())
}

/// Exempt modules carry no inline directive, so the remedy stays config-only.
///
/// `String Heavy Function Arguments` is module-level, and `CodeScene` directives
/// apply only to function-level smells. A directive here would be an
/// unauthorized suppression that does not even target the reported factor.
#[test]
fn exempt_modules_carry_no_inline_directive() -> Result<(), String> {
    let directory = project_directory()?;
    for path in EXEMPT_PATHS {
        let source = directory
            .read_to_string(path)
            .map_err(|error| format!("the exempt module {path} is not readable: {error}"))?;
        if source.contains("@codescene") {
            return Err(format!(
                "{path} carries an inline CodeScene directive; ADR 0007 authorizes exact-path \
                 rule weights only"
            ));
        }
    }
    Ok(())
}

/// A valid fixture document holding exactly the authorized rule sets.
fn valid_document() -> Value {
    let rule_sets: Vec<Value> = EXEMPT_PATHS
        .into_iter()
        .map(|path| {
            json!({
                "matching_content_path": path,
                "matching_content_path_doc": "authorized for issue #257",
                "rules": [{ "name": RULE, "weight": 0.0 }]
            })
        })
        .collect();
    json!({
        "usage": "See docs/developers-guide.md#codescene-string-heavy-arguments.",
        "rule_sets": rule_sets
    })
}

/// Returns the mutable rule set array of a fixture document.
fn fixture_rule_sets(document: &mut Value) -> Result<&mut Vec<Value>, String> {
    document
        .get_mut("rule_sets")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| "the fixture has no `rule_sets` array".to_owned())
}

/// Returns the first mutable rule set of a fixture document.
fn first_rule_set_mut(document: &mut Value) -> Result<&mut Value, String> {
    fixture_rule_sets(document)?
        .first_mut()
        .ok_or_else(|| "the fixture has no rule set".to_owned())
}

/// Requires document validation to reject a mutated fixture.
fn expect_rejection(
    description: &str,
    document: &Value,
    validate: fn(&Value) -> Result<BTreeSet<&str>, String>,
) -> Result<(), String> {
    validate(document).map_or_else(
        |_| Ok(()),
        |paths| {
            Err(format!(
                "validation must reject {description}, but accepted {paths:?}"
            ))
        },
    )
}

/// Document validation rejects every change to the authorized path set.
///
/// The cases mutate a valid document, so they fail if the exact set
/// comparison or the duplicate check is removed.
#[test]
fn document_validation_rejects_path_set_changes() -> Result<(), String> {
    validated_paths(&valid_document())?;

    let mut added = valid_document();
    fixture_rule_sets(&mut added)?.push(json!({
        "matching_content_path": "tests/build_standard/unauthorized.rs",
        "matching_content_path_doc": "issue #257",
        "rules": [{ "name": RULE, "weight": 0.0 }]
    }));
    expect_rejection("an added path", &added, validated_paths)?;

    let mut missing = valid_document();
    if fixture_rule_sets(&mut missing)?.pop().is_none() {
        return Err("the fixture has no rule set to remove".to_owned());
    }
    expect_rejection("a missing path", &missing, validated_paths)?;

    let mut duplicated = valid_document();
    let rule_sets = fixture_rule_sets(&mut duplicated)?;
    let first = rule_sets
        .first()
        .cloned()
        .ok_or_else(|| "the fixture has no rule set to duplicate".to_owned())?;
    rule_sets.push(first);
    expect_rejection("a duplicated path", &duplicated, validated_paths)?;
    Ok(())
}

/// Document validation rejects any threshold field except an empty array.
#[test]
fn document_validation_rejects_thresholds() -> Result<(), String> {
    let cases = [
        (
            "a threshold override",
            json!([{ "name": "function_nesting_depth_warning", "value": 99 }]),
        ),
        (
            "an object-typed thresholds field",
            json!({ "name": "function_nesting_depth_warning", "value": 99 }),
        ),
        (
            "a string-typed thresholds field",
            json!("function_nesting_depth_warning"),
        ),
        ("a null thresholds field", Value::Null),
    ];
    for (description, thresholds) in cases {
        let mut document = valid_document();
        first_rule_set_mut(&mut document)?
            .as_object_mut()
            .ok_or_else(|| "the fixture rule set is not an object".to_owned())?
            .insert("thresholds".to_owned(), thresholds);
        expect_rejection(description, &document, validated_paths)?;
    }

    let mut empty = valid_document();
    first_rule_set_mut(&mut empty)?
        .as_object_mut()
        .ok_or_else(|| "the fixture rule set is not an object".to_owned())?
        .insert("thresholds".to_owned(), json!([]));
    validated_paths(&empty)?;
    Ok(())
}

/// Both spellings of a zero weight disable the rule.
///
/// `serde_json` keeps integer `0` and floating-point `0.0` distinct, so a
/// contract that compared `Value`s against one spelling only would reject a
/// configuration `CodeScene` accepts.
#[test]
fn zero_weight_accepts_integer_and_float_spellings() -> Result<(), String> {
    for weight in [json!(0), json!(0.0)] {
        let rule_set = json!({
            "matching_content_path": "tests/build_standard/ci_reachability.rs",
            "matching_content_path_doc": "issue #257",
            "rules": [{ "name": RULE, "weight": weight }]
        });
        only_rule_override(&rule_set, "fixture")?;
    }
    Ok(())
}

/// The per-field guards reject malformed values the document cases omit.
#[test]
fn field_guards_reject_malformed_values() -> Result<(), String> {
    let globbed = json!({
        "matching_content_path": "tests/build_standard/**",
        "matching_content_path_doc": "issue #257",
        "rules": [{ "name": RULE, "weight": 0.0 }]
    });
    if exact_path(&globbed).is_ok() {
        return Err("a glob path must be rejected".to_owned());
    }

    let weighted = json!({
        "matching_content_path": "tests/build_standard/ci_reachability.rs",
        "matching_content_path_doc": "issue #257",
        "rules": [{ "name": RULE, "weight": 0.5 }]
    });
    if only_rule_override(&weighted, "fixture").is_ok() {
        return Err("a non-zero weight must be rejected".to_owned());
    }

    let string_weight = json!({
        "matching_content_path": "tests/build_standard/ci_reachability.rs",
        "matching_content_path_doc": "issue #257",
        "rules": [{ "name": RULE, "weight": "0.0" }]
    });
    if only_rule_override(&string_weight, "fixture").is_ok() {
        return Err("a string weight must be rejected".to_owned());
    }

    let other_rule = json!({
        "matching_content_path": "tests/build_standard/ci_reachability.rs",
        "matching_content_path_doc": "issue #257",
        "rules": [{ "name": "Complex Method", "weight": 0.0 }]
    });
    if only_rule_override(&other_rule, "fixture").is_ok() {
        return Err("an unrelated rule override must be rejected".to_owned());
    }

    let undocumented = json!({
        "matching_content_path": "tests/build_standard/ci_reachability.rs",
        "matching_content_path_doc": "because it is a test",
        "rules": [{ "name": RULE, "weight": 0.0 }]
    });
    if rationale(&undocumented, "fixture").is_ok() {
        return Err("a rationale without the issue reference must be rejected".to_owned());
    }
    Ok(())
}
