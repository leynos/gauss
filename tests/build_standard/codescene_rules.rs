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

/// The exact paths authorised by ADR 0007.
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
    if weight != &json!(0.0) {
        return Err(format!(
            "{path} must disable `{RULE}` with weight 0.0, found {weight}"
        ));
    }
    let has_thresholds = rule_set
        .get("thresholds")
        .and_then(Value::as_array)
        .is_some_and(|thresholds| !thresholds.is_empty());
    if has_thresholds {
        return Err(format!(
            "{path} must not override thresholds; ADR 0007 authorises exact-path rule weights only"
        ));
    }
    Ok(())
}

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

/// Every rule set names one exact, existing module with a recorded reason.
///
/// The expected path set is compared exactly, so a new exemption cannot be
/// added to the rule file without updating ADR 0007 and this contract.
#[test]
fn rule_sets_are_exact_documented_and_authorised() -> Result<(), String> {
    let document = rule_document()?;
    usage_document_reference(&document)?;
    let directory = project_directory()?;
    let mut seen = BTreeSet::new();
    for rule_set in rule_sets(&document)? {
        let path = exact_path(rule_set)?;
        rationale(rule_set, path)?;
        only_rule_override(rule_set, path)?;
        directory
            .read_to_string(path)
            .map_err(|error| format!("the exempt module {path} is not readable: {error}"))?;
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
    Ok(())
}

/// Exempt modules carry no inline directive, so the remedy stays config-only.
///
/// `String Heavy Function Arguments` is module-level, and `CodeScene` directives
/// apply only to function-level smells. A directive here would be an
/// unauthorised suppression that does not even target the reported factor.
#[test]
fn exempt_modules_carry_no_inline_directive() -> Result<(), String> {
    let directory = project_directory()?;
    for path in EXEMPT_PATHS {
        let source = directory
            .read_to_string(path)
            .map_err(|error| format!("the exempt module {path} is not readable: {error}"))?;
        if source.contains("@codescene") {
            return Err(format!(
                "{path} carries an inline CodeScene directive; ADR 0007 authorises exact-path \
                 rule weights only"
            ));
        }
    }
    Ok(())
}

/// The contract itself must detect a widened or malformed override set.
///
/// These cases exercise the guards above so the suite cannot pass by checking
/// nothing if the rule file shape changes.
#[test]
fn contract_rejects_malformed_overrides() -> Result<(), String> {
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
