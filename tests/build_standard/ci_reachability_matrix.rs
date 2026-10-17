//! Expands finite one-axis matrices and substitutes exact runner expressions.
//!
//! Repeated scalar include and exclude rows fail closed; this reader does not
//! claim whether GitHub accepts those repeated list rows.

use std::collections::{BTreeMap, BTreeSet};

use super::Value;

/// Expands one finite scalar axis, or explicit include-only rows, without a Cartesian product.
pub(super) fn rows(strategy: Option<&Value>) -> Result<Vec<BTreeMap<String, String>>, String> {
    let Some(strategy_value) = strategy else {
        return Ok(vec![BTreeMap::new()]);
    };
    let strategy_fields = strategy_value.mapping("job.strategy")?;
    let Some(matrix) = strategy_fields.get("matrix") else {
        return Ok(vec![BTreeMap::new()]);
    };
    let fields = matrix.mapping("job.strategy.matrix")?;
    let axis_name = scalar_axis(fields)?;
    let include = fields.get("include");
    let exclude = fields.get("exclude");
    axis_name.map_or_else(
        || include_only(include, exclude),
        |axis| expand_axis(fields, &axis, include, exclude),
    )
}

/// Replaces exact whole-scalar matrix references throughout a runner value.
pub(super) fn substitute(value: &Value, row: &BTreeMap<String, String>) -> Result<Value, String> {
    match value {
        Value::Empty => Ok(Value::Empty),
        Value::Scalar(text) => substitute_scalar(text, row),
        Value::Sequence(items) => items
            .iter()
            .map(|item| substitute(item, row))
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Sequence),
        Value::Mapping(entries) => substitute_mapping(entries, row),
    }
}

/// Finds the sole dimension name while reserving GitHub's include and exclude keys.
fn scalar_axis(fields: &BTreeMap<String, Value>) -> Result<Option<String>, String> {
    let mut axis = None;
    for key in fields.keys() {
        if matches!(key.as_str(), "include" | "exclude") {
            continue;
        }
        if key.contains("${{") || key.contains("}}") {
            return Err(format!("computed matrix axis name `{key}` is unsupported"));
        }
        if axis.replace(key.clone()).is_some() {
            return Err("only one matrix axis is supported".to_owned());
        }
    }
    Ok(axis)
}

/// Expands one scalar sequence, applies scalar exclusions, then merges scalar includes.
fn expand_axis(
    fields: &BTreeMap<String, Value>,
    axis: &str,
    include: Option<&Value>,
    exclude: Option<&Value>,
) -> Result<Vec<BTreeMap<String, String>>, String> {
    let values = fields
        .get(axis)
        .ok_or_else(|| format!("missing matrix axis `{axis}`"))?
        .sequence("matrix axis")?;
    if values.is_empty() {
        return Err("matrix axis is empty".to_owned());
    }
    let mut rows = axis_rows(axis, values)?;
    apply_exclusions(&mut rows, axis, exclude)?;
    apply_inclusions(&mut rows, axis, include)?;
    if rows.is_empty() {
        Err("matrix expansion is empty".to_owned())
    } else {
        Ok(rows)
    }
}

/// Builds unique rows from a list of non-empty scalar axis values.
fn axis_rows(axis: &str, values: &[Value]) -> Result<Vec<BTreeMap<String, String>>, String> {
    let mut rows = Vec::new();
    for value in values {
        let text = scalar_text(value, "matrix axis value")?;
        if rows
            .iter()
            .any(|row: &BTreeMap<String, String>| row.get(axis) == Some(&text))
        {
            return Err(format!("matrix axis `{axis}` contains a duplicate value"));
        }
        rows.push(BTreeMap::from([(axis.to_owned(), text)]));
    }
    Ok(rows)
}

/// Removes rows selected by non-empty exclusion maps that mention only the scalar axis.
fn apply_exclusions(
    rows: &mut Vec<BTreeMap<String, String>>,
    axis: &str,
    exclude: Option<&Value>,
) -> Result<(), String> {
    let Some(exclusion) = exclude else {
        return Ok(());
    };
    let exclusions = unique_scalar_rows(
        exclusion.sequence("matrix.exclude")?,
        "matrix.exclude",
        "matrix exclusion",
    )?;
    for fields in exclusions {
        if fields.len() != 1 || !fields.contains_key(axis) {
            return Err("matrix exclusions must name only the scalar axis".to_owned());
        }
        let value = fields
            .get(axis)
            .ok_or_else(|| format!("matrix exclusion omits `{axis}`"))?;
        rows.retain(|row| row.get(axis) != Some(value));
    }
    Ok(())
}

/// Adds explicit scalar include rows or merges fields into their matching axis row.
fn apply_inclusions(
    rows: &mut Vec<BTreeMap<String, String>>,
    axis: &str,
    include: Option<&Value>,
) -> Result<(), String> {
    let Some(inclusion) = include else {
        return Ok(());
    };
    let inclusions = unique_scalar_rows(
        inclusion.sequence("matrix.include")?,
        "matrix.include",
        "matrix inclusion",
    )?;
    for fields in inclusions {
        let axis_value = fields
            .get(axis)
            .ok_or_else(|| format!("matrix inclusion must name `{axis}`"))?;
        let mut matched = false;
        for row in rows.iter_mut() {
            if row.get(axis) == Some(axis_value) {
                merge_include(row, &fields, axis)?;
                matched = true;
            }
        }
        if !matched {
            rows.push(fields);
        }
    }
    Ok(())
}

/// Merges an include into an existing row while refusing conflicting matrix metadata.
fn merge_include(
    row: &mut BTreeMap<String, String>,
    included: &BTreeMap<String, String>,
    axis: &str,
) -> Result<(), String> {
    for (key, value) in included {
        if key == axis {
            continue;
        }
        if row.get(key).is_some_and(|existing| existing != value) {
            return Err(format!("matrix include conflicts on `{key}`"));
        }
        row.insert(key.clone(), value.clone());
    }
    Ok(())
}

/// Parses a finite include-only matrix and rejects an orphaned exclusion list.
fn include_only(
    include: Option<&Value>,
    exclude: Option<&Value>,
) -> Result<Vec<BTreeMap<String, String>>, String> {
    if exclude.is_some() {
        return Err("matrix exclusions require a scalar axis".to_owned());
    }
    let include_rows =
        include.ok_or_else(|| "matrix has no finite scalar axis or include list".to_owned())?;
    let rows = unique_scalar_rows(
        include_rows.sequence("matrix.include")?,
        "matrix.include",
        "matrix inclusion",
    )?;
    if rows.is_empty() || rows.iter().any(BTreeMap::is_empty) {
        Err("matrix include-only expansion is empty or indeterminate".to_owned())
    } else {
        Ok(rows)
    }
}

/// Reads a non-empty mapping whose values are all literal, non-empty scalar strings.
fn scalar_row(value: &Value, context: &str) -> Result<BTreeMap<String, String>, String> {
    let mut row = BTreeMap::new();
    for (key, field_value) in value.mapping(context)? {
        if key.contains("${{") || key.contains("}}") {
            return Err(format!("{context} has a computed key `{key}`"));
        }
        row.insert(key.clone(), scalar_text(field_value, context)?);
    }
    if row.is_empty() {
        Err(format!("{context} is empty"))
    } else {
        Ok(row)
    }
}

/// Parses scalar rows and rejects exact repeats with a list-specific diagnostic.
///
/// For example, two identical maps in `matrix.include` are reported as an
/// unsupported repeated-row form before either row is applied.
fn unique_scalar_rows(
    values: &[Value],
    list_context: &str,
    row_context: &str,
) -> Result<Vec<BTreeMap<String, String>>, String> {
    let mut rows = Vec::new();
    let mut seen = BTreeSet::new();
    for value in values {
        let row = scalar_row(value, row_context)?;
        if !seen.insert(row.clone()) {
            return Err(format!(
                "{list_context}: repeated scalar rows are unsupported by this reader"
            ));
        }
        rows.push(row);
    }
    Ok(rows)
}

/// Rejects empty or computed scalar values before they enter a runner row.
fn scalar_text(value: &Value, context: &str) -> Result<String, String> {
    let text = value.text(context)?;
    if text.is_empty() || contains_expression_marker(text) {
        Err(format!("{context} is empty or computed"))
    } else {
        Ok(text.to_owned())
    }
}

/// Detects either boundary of a GitHub expression in a literal value.
///
/// For example, `ubuntu-latest` has no marker, while `${{ matrix.os }}` does.
fn contains_expression_marker(text: &str) -> bool {
    text.contains("${{") || text.contains("}}")
}

/// Resolves a scalar only when it is one exact matrix field expression.
fn substitute_scalar(text: &str, row: &BTreeMap<String, String>) -> Result<Value, String> {
    if !text.contains("${{") && !text.contains("}}") {
        return Ok(Value::Scalar(text.to_owned()));
    }
    let field = text
        .strip_prefix("${{ matrix.")
        .and_then(|rest| rest.strip_suffix(" }}"))
        .ok_or_else(|| format!("unsupported computed runner expression `{text}`"))?;
    if !valid_field(field) {
        return Err(format!("invalid matrix field reference `{text}`"));
    }
    row.get(field)
        .cloned()
        .map(Value::Scalar)
        .ok_or_else(|| format!("matrix field `{field}` is missing from the row"))
}

/// Validates a matrix field name without interpreting general GitHub expressions.
fn valid_field(field: &str) -> bool {
    let mut characters = field.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && characters
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
}

/// Recursively substitutes mapping values and refuses computed mapping keys.
fn substitute_mapping(
    entries: &BTreeMap<String, Value>,
    row: &BTreeMap<String, String>,
) -> Result<Value, String> {
    let mut resolved = BTreeMap::new();
    for (key, value) in entries {
        if key.contains("${{") || key.contains("}}") {
            return Err(format!(
                "computed matrix mapping key `{key}` is unsupported"
            ));
        }
        resolved.insert(key.clone(), substitute(value, row)?);
    }
    Ok(Value::Mapping(resolved))
}
