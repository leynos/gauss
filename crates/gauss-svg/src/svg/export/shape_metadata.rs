//! Shape-level Gauss metadata attribute serialization helpers.

use super::write_fmt;
use crate::model::{GaussAttribute, Shape};
use crate::svg::metadata::shape_id_to_hex;
use slotmap::Key;

/// Append the non-null ID, optional escaped name, true flags, and opaque
/// metadata.
///
/// Example: A null-ID locked shape emits `gauss:locked="true"` without
/// `gauss:id`.
pub(super) fn write_shape_gauss_metadata(out: &mut String, shape: &Shape) {
    if !shape.id.is_null() {
        let hex = shape_id_to_hex(shape.id);
        write_fmt(out, format_args!(r#" gauss:id="{hex}""#));
    }
    if let Some(name) = shape.name.as_deref() {
        let escaped_name = escape_attr_value(name);
        write_fmt(out, format_args!(r#" gauss:name="{escaped_name}""#));
    }
    if shape.locked {
        out.push_str(r#" gauss:locked="true""#);
    }
    if shape.hidden {
        out.push_str(r#" gauss:hidden="true""#);
    }
    write_opaque_gauss_attrs(out, &shape.gauss_metadata);
}

/// Append opaque Gauss attributes, escaping values while inserting names as
/// stored.
///
/// Example: Value `a&b` becomes `a&amp;b`; its attribute name is unchanged.
fn write_opaque_gauss_attrs(out: &mut String, metadata: &[GaussAttribute]) {
    for attr in metadata {
        let escaped_value = escape_attr_value(&attr.value);
        write_fmt(
            out,
            format_args!(r#" gauss:{}="{escaped_value}""#, attr.name),
        );
    }
}

/// Escape `&`, `"`, `<`, `>`, and `'` for an XML attribute value.
///
/// Example: `a&<b` becomes `a&amp;&lt;b`; other characters pass through.
fn escape_attr_value(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '"' => escaped.push_str("&quot;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '\'' => escaped.push_str("&apos;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}
