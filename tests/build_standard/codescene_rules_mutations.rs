//! Mutation cases for the `CodeScene` string-heavy exemption contract.
//!
//! The parent module owns the repository contract checks and the field-level
//! guards. These cases mutate a valid rule document so that weakening
//! document validation — dropping the exact-set comparison, the duplicate
//! check, the rationale guard or the threshold guard — makes a test fail
//! rather than leaving it green.

use std::collections::BTreeSet;

use serde_json::{Value, json};

use super::{EXEMPT_PATHS, RULE, exact_path, only_rule_override, rationale, validated_paths};

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

/// Document validation rejects a rule set whose rationale is unusable.
///
/// The rationale guard is asserted through [`validated_paths`], not only
/// through the field helper, so removing the guard from document validation
/// fails this test.
#[test]
fn document_validation_rejects_unusable_rationale() -> Result<(), String> {
    let unusable = ["because it is a test", "   "];
    for text in unusable {
        let mut document = valid_document();
        first_rule_set_mut(&mut document)?
            .as_object_mut()
            .ok_or_else(|| "the fixture rule set is not an object".to_owned())?
            .insert("matching_content_path_doc".to_owned(), json!(text));
        expect_rejection(
            "a rationale without the issue reference",
            &document,
            validated_paths,
        )?;
    }
    validated_paths(&valid_document())?;
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
